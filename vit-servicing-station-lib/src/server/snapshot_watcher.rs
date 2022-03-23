use crate::{
    db::{models::snapshot::NewSnapshotEntry, schema, DbConnectionPool},
    v0::context::SharedContext,
};
use diesel::{Connection, RunQueryDsl};
use notify::{
    event::{
        self, AccessKind, AccessMode, CreateKind, MetadataKind, ModifyKind, RemoveKind, RenameMode,
    },
    EventKind, RecursiveMode, Watcher,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::{debug, error, span, trace, warn, Instrument, Level};

type RawSnapshot = Vec<(String, String)>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid snapshot format")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    PoolError(#[from] diesel::r2d2::PoolError),

    #[error(transparent)]
    DatabaseError(#[from] diesel::result::Error),

    #[error(transparent)]
    Notify(#[from] notify::Error),

    #[error("snapshot path is not a file with a proper parent directory")]
    InvalidPath,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[tracing::instrument(skip(context))]
pub async fn async_watch(path: PathBuf, context: SharedContext) -> Result<(), Error> {
    if path.is_dir() {
        return Err(Error::InvalidPath);
    }

    let file_name = path.file_name().ok_or(Error::InvalidPath)?.to_owned();

    let parent = match path.parent() {
        Some(parent) => {
            tokio::fs::create_dir_all(parent).await?;
            parent.to_path_buf()
        }
        None => return Err(Error::InvalidPath),
    };

    let pool = context.read().await.db_connection_pool.clone();

    load_snapshot_table_from_file(path.clone(), pool.clone()).await?;

    let (tx, mut rx) = tokio::sync::watch::channel(());

    let watcher = {
        let watcher_callback_span = span!(Level::DEBUG, "filesystem event");

        tokio::task::spawn_blocking(move || {
            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<event::Event>| {
                    let _guard = watcher_callback_span.enter();

                    let event = match res {
                        Ok(event) => event,
                        Err(e) => {
                            error!(?e);
                            return;
                        }
                    };

                    if !event
                        .paths
                        .iter()
                        .filter_map(|p| p.file_name())
                        .any(|p| p == file_name)
                    {
                        return;
                    }

                    trace!(?event);

                    match event.kind {
                        EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)) => {}
                        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)) => {}
                        EventKind::Create(CreateKind::File) => {}
                        EventKind::Remove(RemoveKind::File) => {}
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {}
                        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {}
                        EventKind::Modify(ModifyKind::Name(RenameMode::Both))
                            if event
                                .paths
                                .get(1)
                                .and_then(|p| p.file_name())
                                .map(|fname| fname == file_name)
                                .unwrap_or(false) => {}
                        _ => return,
                    }

                    if tx.send(()).is_err() {
                        warn!(
                            "failed to propagate snapshot file update event, this shouldn't happen"
                        );
                    }
                })?;

            watcher.watch(&parent, RecursiveMode::NonRecursive)?;

            debug!("watching snapshot directory");

            Ok::<_, notify::Error>(watcher)
        })
        .instrument(span!(Level::INFO, "snapshot dir watcher initialization",))
        .await
        .unwrap()
    }?;

    tokio::task::spawn(
        async move {
            let debounce_time = Duration::from_millis(50);
            let mut last_update = Instant::now() - debounce_time;

            while rx.changed().await.is_ok() {
                // a simple debounce to avoid useless reloads, since a single write can trigger
                // many events
                let now = Instant::now();

                if now.duration_since(last_update) < debounce_time {
                    continue;
                }

                last_update = now;

                if let Err(error) = load_snapshot_table_from_file(path.clone(), pool.clone()).await
                {
                    error!(
                        context = "failed to refresh snapshot data from file",
                        %error
                    );
                }
            }

            // just to move the watcher into this future so it never drops
            let _watcher = watcher;
        }
        .instrument(tracing::info_span!("snapshot reload")),
    );

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn load_snapshot_table_from_file(path: PathBuf, pool: DbConnectionPool) -> Result<(), Error> {
    let snapshot: RawSnapshot = match tokio::fs::read(path.as_path()).await {
        Ok(raw) => serde_json::from_slice(&raw)?,
        Err(_) => {
            warn!("snapshot file not found, asumming empty data set");
            vec![]
        }
    };

    tokio::task::spawn_blocking(move || {
        let conn = pool.get()?;

        conn.transaction::<_, diesel::result::Error, _>(|| {
            let deleted = diesel::delete(schema::snapshot::table).execute(&conn)?;

            trace!("deleted {} snapshot entries", deleted);

            let inserted = diesel::insert_into(schema::snapshot::table)
                .values(
                    snapshot
                        .iter()
                        .map(|(vk, vp)| NewSnapshotEntry {
                            voting_key: vk.clone(),
                            voting_power: vp.parse::<u64>().unwrap() as i64,
                        })
                        .collect::<Vec<_>>(),
                )
                .execute(&*conn)?;

            trace!("inserted {} new entries into the snapshot table", inserted);

            Ok(())
        })
        .map_err(Error::from)
    })
    .instrument(span!(Level::INFO, "rebuild snapshot table"))
    .await
    .unwrap()?;

    Ok(())
}
