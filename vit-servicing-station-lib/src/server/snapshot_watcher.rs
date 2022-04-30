use notify::{
    event::{self, AccessKind, AccessMode, CreateKind, MetadataKind, ModifyKind, RemoveKind},
    EventKind, RecursiveMode, Watcher,
};
use snapshot_service::UpdateHandle;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::{debug, error, field, span, trace, warn, Instrument, Level};
pub(crate) use voting_hir::VoterHIR;

const DEBOUNCE_TIME: Duration = Duration::from_millis(100);
const PAT: &str = "-snapshot.json";

type Snapshot = Vec<VoterHIR>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid snapshot format")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Notify(#[from] notify::Error),

    #[error("snapshot path is not a file with a proper parent directory")]
    InvalidPath,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SnapshotService(#[from] snapshot_service::Error),
}

#[must_use]
pub struct WatcherGuard(notify::RecommendedWatcher);

type Tag = String;

fn extract_tag_from_filename(path: impl AsRef<OsStr>) -> Option<Tag> {
    let path = path.as_ref().to_string_lossy();

    path.find(&PAT)
        // check that the pattern is at the end, that means, that there is nothing trailing, just
        // in case
        .filter(|start| start + PAT.as_bytes().len() == path.as_bytes().len())
        .map(|start| path[..start].to_string())
}

async fn load_from_paths(
    mut debouncer: Option<&mut HashMap<Tag, Instant>>,
    paths: impl Iterator<Item = PathBuf>,
    context: &mut UpdateHandle,
) {
    for path in paths {
        let tag = match path.file_name().and_then(extract_tag_from_filename) {
            Some(tag) => tag,
            None => {
                trace!("skipping {:?}", path);
                continue;
            }
        };

        if let Some(ref mut debouncer) = debouncer {
            // A simple debounce to avoid useless reloads, since a single write can trigger many
            // events. We could do something more complex by coalescing events in timeframe
            // 'buckets', but since we do the same thing regardless of the event I don't see much
            // point.
            let now = Instant::now();

            let last_update = debouncer
                .get(&tag)
                .copied()
                // set the last update in the past if the entry doesn't exist, so the we don't
                // reach the `continue` the first time.
                .unwrap_or_else(|| now - DEBOUNCE_TIME);

            if now.duration_since(last_update) < DEBOUNCE_TIME {
                continue;
            }

            debouncer.insert(tag.clone(), now);
        }

        match load_snapshot_table_from_file(path, tag.clone(), context).await {
            Ok(inserted) => {
                // Maybe this logic can be simplified, I'm not sure. This is mostly to not have a
                // "memory leak", but it is a minor concern since there shouldn't ever be that many
                // values for that to cause a problem. As an alternative, we could periodically
                // remove entries that are too old.
                if inserted == 0 {
                    if let Some(ref mut debouncer) = debouncer {
                        let _ = debouncer.remove(&tag);
                    }
                }
            }
            Err(error) => {
                error!(
                    context = "failed to fill snapshot table from file",
                    %error
                );
            }
        }
    }
}

#[tracing::instrument(skip(context))]
pub async fn async_watch(path: PathBuf, mut context: UpdateHandle) -> Result<WatcherGuard, Error> {
    let _ = tokio::fs::create_dir_all(path.as_path()).await;

    if !&path.is_dir() {
        return Err(Error::InvalidPath);
    }

    {
        let path = path.clone();
        let dir_entries = tokio::task::spawn_blocking(move || {
            std::fs::read_dir(&path).map(|entries| entries.filter_map(|entry| entry.ok()))
        })
        .await
        .unwrap()?;

        load_from_paths(None, dir_entries.map(|de| de.path()), &mut context).await;
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

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

                    match event.kind {
                        EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime))
                        | EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any))
                        | EventKind::Create(CreateKind::File)
                        | EventKind::Remove(RemoveKind::File)
                        | EventKind::Access(AccessKind::Close(AccessMode::Write))
                        | EventKind::Modify(ModifyKind::Name(_)) => {
                            if tx.blocking_send(event.paths).is_err() {
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

            watcher.watch(&path, RecursiveMode::NonRecursive)?;

            debug!("watching snapshot directory");

            Ok::<_, notify::Error>(watcher)
        })
        .instrument(span!(Level::INFO, "snapshot dir watcher initialization",))
        .await
        .unwrap()
    }?;

    tokio::task::spawn(
        async move {
            let mut debouncer: HashMap<Tag, Instant> = HashMap::new();

            while let Some(paths) = rx.recv().await {
                load_from_paths(Some(&mut debouncer), paths.into_iter(), &mut context).await;
            }
        }
        .instrument(tracing::info_span!("snapshot reload")),
    );

    Ok(WatcherGuard(watcher))
}

#[tracing::instrument(skip(db))]
async fn load_snapshot_table_from_file(
    path: PathBuf,
    tag: Tag,
    db: &mut UpdateHandle,
) -> Result<usize, Error> {
    let snapshot: Snapshot = match tokio::fs::read(path.as_path()).await {
        Ok(raw) => serde_json::from_slice(&raw)?,
        Err(_) => {
            warn!("snapshot file not found, asumming empty data set");
            vec![]
        }
    };

    let size = snapshot.len();

    db.update(&tag, snapshot).await?;

    Ok(size)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_snapshot_extract_tag() {
        assert_eq!(
            extract_tag_from_filename("test-snapshot.json"),
            Some("test".to_string())
        );

        assert_eq!(
            extract_tag_from_filename("tést-snapshot.json"),
            Some("tést".to_string())
        );

        assert_eq!(extract_tag_from_filename("test-snapshot.json.tmp"), None);

        assert_eq!(extract_tag_from_filename("test-snapshot"), None);

        assert_eq!(extract_tag_from_filename("snapshot"), None);

        assert_eq!(extract_tag_from_filename(""), None);
    }
}
