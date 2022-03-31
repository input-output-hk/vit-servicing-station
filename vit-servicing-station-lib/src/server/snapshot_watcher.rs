use crate::{
    db::{models::snapshot::SnapshotEntry, schema, DbConnectionPool},
    v0::context::SharedContext,
};
use diesel::{Connection, RunQueryDsl};
use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};
use notify::{
    event::{self, AccessKind, AccessMode, CreateKind, MetadataKind, ModifyKind, RemoveKind},
    EventKind, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::{debug, error, field, span, trace, warn, Instrument, Level};

pub type VotingGroup = String;

/// Define High Level Intermediate Representation (HIR) for voting
/// entities in the Catalyst ecosystem.
///
/// This is intended as a high level description of the setup, which is not
/// enough on its own to spin a blockchain, but it's slimmer, easier to understand
/// and free from implementation constraints.
///
/// You can roughly read this as
/// "voting_key will participate in this voting round with role voting_group and will have voting_power influence"
#[derive(Serialize, Deserialize)]
pub struct VotingHIR {
    pub voting_key: Identifier,
    /// Voting group this key belongs to.
    /// If this key belong to multiple voting groups, multiple records for the same
    /// key will be used.
    pub voting_group: VotingGroup,
    /// Voting power as processed by the snapshot
    pub voting_power: Value,
}

type RawSnapshot = Vec<VotingHIR>;

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

#[must_use]
pub struct WatcherGuard(notify::RecommendedWatcher);

#[tracing::instrument(skip(context))]
pub async fn async_watch(path: PathBuf, context: SharedContext) -> Result<WatcherGuard, Error> {
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
        let watcher_callback_span = span!(Level::DEBUG, "filesystem event", event = field::Empty);

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

                    trace!(?event);

                    if !event
                        .paths
                        .iter()
                        .filter_map(|p| p.file_name())
                        .any(|p| p == file_name)
                    {
                        return;
                    }

                    match event.kind {
                        EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime))
                        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any))
                        | EventKind::Create(CreateKind::File)
                        | EventKind::Remove(RemoveKind::File)
                        | EventKind::Access(AccessKind::Close(AccessMode::Write))
                        | EventKind::Modify(ModifyKind::Name(_)) => {
                            if tx.send(()).is_err() {
                                warn!(
                            "failed to propagate snapshot file update event, this shouldn't happen"
                        );
                            }
                        }
                        _ => {
                            trace!(context = "filesystem event ignored", ?event);
                        }
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
        }
        .instrument(tracing::info_span!("snapshot reload")),
    );

    Ok(WatcherGuard(watcher))
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
                        .map(
                            |VotingHIR {
                                 voting_key,
                                 voting_power,
                                 ..
                             }| SnapshotEntry {
                                voting_key: voting_key.to_hex(),
                                voting_power: u64::from(*voting_power) as i64,
                            },
                        )
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
