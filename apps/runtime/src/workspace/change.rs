//! Runtime-owned portable Workspace filesystem change observation.

#![allow(
    dead_code,
    reason = "the accepted source boundary is consumed by the following rebuild task"
)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{Interval, MissedTickBehavior};

use crate::Cancellation;

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const IGNORED_DIRECTORIES: [&str; 5] = [".git", ".idea", ".vscode", "target", "node_modules"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceFileState {
    entries: BTreeMap<PathBuf, WorkspaceFileEntry>,
}

impl WorkspaceFileState {
    pub(super) fn scan(root: &Path) -> Result<Self, WorkspaceObservationError> {
        let metadata = fs::metadata(root).map_err(|source| WorkspaceObservationError::Root {
            path: root.to_path_buf(),
            source,
        })?;
        if !metadata.is_dir() {
            return Err(WorkspaceObservationError::RootNotDirectory(
                root.to_path_buf(),
            ));
        }

        let mut entries = BTreeMap::new();
        scan_directory(root, root, &mut entries)?;
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkspaceFileEntry {
    Directory,
    RegularFile(Vec<u8>),
    Other,
}

fn scan_directory(
    root: &Path,
    directory: &Path,
    entries: &mut BTreeMap<PathBuf, WorkspaceFileEntry>,
) -> Result<(), WorkspaceObservationError> {
    let mut children = fs::read_dir(directory)
        .map_err(|source| WorkspaceObservationError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| WorkspaceObservationError::ReadDirectoryEntry {
            path: directory.to_path_buf(),
            source,
        })?;
    children.sort_by_key(std::fs::DirEntry::path);

    for child in children {
        let path = child.path();
        let relative_path = path
            .strip_prefix(root)
            .expect("scanned entry must remain below the Workspace root")
            .to_path_buf();
        let file_type =
            child
                .file_type()
                .map_err(|source| WorkspaceObservationError::ReadFileType {
                    path: path.clone(),
                    source,
                })?;

        if file_type.is_dir() {
            entries.insert(relative_path, WorkspaceFileEntry::Directory);
            if !is_ignored_directory(&path) {
                scan_directory(root, &path, entries)?;
            }
        } else if file_type.is_file() {
            let bytes = fs::read(&path)
                .map_err(|source| WorkspaceObservationError::ReadFile { path, source })?;
            entries.insert(relative_path, WorkspaceFileEntry::RegularFile(bytes));
        } else {
            entries.insert(relative_path, WorkspaceFileEntry::Other);
        }
    }

    Ok(())
}

fn is_ignored_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| IGNORED_DIRECTORIES.contains(&name))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkspaceObservationErrorKind {
    Root,
    RootNotDirectory,
    ReadDirectory,
    ReadDirectoryEntry,
    ReadFileType,
    ReadFile,
}

#[derive(Debug)]
pub(super) enum WorkspaceObservationError {
    Root {
        path: PathBuf,
        source: std::io::Error,
    },
    RootNotDirectory(PathBuf),
    ReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadDirectoryEntry {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadFileType {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl WorkspaceObservationError {
    pub(super) const fn kind(&self) -> WorkspaceObservationErrorKind {
        match self {
            Self::Root { .. } => WorkspaceObservationErrorKind::Root,
            Self::RootNotDirectory(_) => WorkspaceObservationErrorKind::RootNotDirectory,
            Self::ReadDirectory { .. } => WorkspaceObservationErrorKind::ReadDirectory,
            Self::ReadDirectoryEntry { .. } => WorkspaceObservationErrorKind::ReadDirectoryEntry,
            Self::ReadFileType { .. } => WorkspaceObservationErrorKind::ReadFileType,
            Self::ReadFile { .. } => WorkspaceObservationErrorKind::ReadFile,
        }
    }
}

impl Display for WorkspaceObservationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root { path, source } => write!(
                formatter,
                "failed to inspect Workspace root {}: {source}",
                path.display()
            ),
            Self::RootNotDirectory(path) => {
                write!(
                    formatter,
                    "Workspace root is not a directory: {}",
                    path.display()
                )
            }
            Self::ReadDirectory { path, source } => write!(
                formatter,
                "failed to read Workspace directory {}: {source}",
                path.display()
            ),
            Self::ReadDirectoryEntry { path, source } => write!(
                formatter,
                "failed to read an entry in Workspace directory {}: {source}",
                path.display()
            ),
            Self::ReadFileType { path, source } => write!(
                formatter,
                "failed to read Workspace file type for {}: {source}",
                path.display()
            ),
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read Workspace file {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for WorkspaceObservationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Root { source, .. }
            | Self::ReadDirectory { source, .. }
            | Self::ReadDirectoryEntry { source, .. }
            | Self::ReadFileType { source, .. }
            | Self::ReadFile { source, .. } => Some(source),
            Self::RootNotDirectory(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkspaceChangeOutcome {
    Changed,
    ObservationFailed(WorkspaceObservationErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorkspaceChangeObservation {
    revision: u64,
    outcome: Option<WorkspaceChangeOutcome>,
}

impl WorkspaceChangeObservation {
    const fn initial() -> Self {
        Self {
            revision: 0,
            outcome: None,
        }
    }

    pub(super) const fn revision(self) -> u64 {
        self.revision
    }

    pub(super) const fn outcome(self) -> Option<WorkspaceChangeOutcome> {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkspaceChangeSourceError {
    RevisionOverflow,
}

impl Display for WorkspaceChangeSourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevisionOverflow => formatter.write_str("Workspace change revision overflowed"),
        }
    }
}

impl Error for WorkspaceChangeSourceError {}

#[derive(Debug)]
pub(super) struct WorkspaceChangeSource {
    root_path: PathBuf,
    state: WorkspaceFileState,
    ticks: ScanTicks,
}

impl WorkspaceChangeSource {
    pub(super) fn new(root_path: PathBuf, state: WorkspaceFileState) -> Self {
        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self {
            root_path,
            state,
            ticks: ScanTicks::Interval(interval),
        }
    }

    pub(super) fn start(self, mut cancellation: Cancellation) -> RunningWorkspaceChangeSource {
        let (observations, receiver) = watch::channel(WorkspaceChangeObservation::initial());
        let task = tokio::spawn(async move {
            self.run(async move { cancellation.cancelled().await }, observations)
                .await
        });
        RunningWorkspaceChangeSource {
            observations: receiver,
            task,
        }
    }

    async fn run<F>(
        mut self,
        shutdown: F,
        observations: watch::Sender<WorkspaceChangeObservation>,
    ) -> Result<(), WorkspaceChangeSourceError>
    where
        F: Future<Output = ()>,
    {
        tokio::pin!(shutdown);
        let mut revision = 0_u64;
        let mut failed = false;

        loop {
            tokio::select! {
                () = &mut shutdown => return Ok(()),
                tick = self.ticks.next() => {
                    if !tick {
                        return Ok(());
                    }
                }
            }

            let outcome = match WorkspaceFileState::scan(&self.root_path) {
                Ok(state) if state != self.state || failed => {
                    self.state = state;
                    failed = false;
                    Some(WorkspaceChangeOutcome::Changed)
                }
                Err(error) if !failed => {
                    failed = true;
                    Some(WorkspaceChangeOutcome::ObservationFailed(error.kind()))
                }
                Ok(_) | Err(_) => None,
            };

            if let Some(outcome) = outcome {
                revision = revision
                    .checked_add(1)
                    .ok_or(WorkspaceChangeSourceError::RevisionOverflow)?;
                observations.send_replace(WorkspaceChangeObservation {
                    revision,
                    outcome: Some(outcome),
                });
            }

            #[cfg(test)]
            self.ticks.acknowledge();
        }
    }
}

#[derive(Debug)]
pub(super) struct RunningWorkspaceChangeSource {
    observations: watch::Receiver<WorkspaceChangeObservation>,
    task: JoinHandle<Result<(), WorkspaceChangeSourceError>>,
}

impl RunningWorkspaceChangeSource {
    pub(super) fn observations(&self) -> watch::Receiver<WorkspaceChangeObservation> {
        self.observations.clone()
    }

    pub(super) async fn join(
        self,
    ) -> Result<Result<(), WorkspaceChangeSourceError>, tokio::task::JoinError> {
        self.task.await
    }
}

#[derive(Debug)]
enum ScanTicks {
    Interval(Interval),
    #[cfg(test)]
    Controlled {
        ticks: tokio::sync::mpsc::Receiver<tokio::sync::oneshot::Sender<()>>,
        acknowledgement: Option<tokio::sync::oneshot::Sender<()>>,
    },
}

impl ScanTicks {
    async fn next(&mut self) -> bool {
        match self {
            Self::Interval(interval) => {
                interval.tick().await;
                true
            }
            #[cfg(test)]
            Self::Controlled {
                ticks,
                acknowledgement,
            } => {
                *acknowledgement = ticks.recv().await;
                acknowledgement.is_some()
            }
        }
    }

    #[cfg(test)]
    fn acknowledge(&mut self) {
        if let Self::Controlled {
            acknowledgement, ..
        } = self
            && let Some(acknowledgement) = acknowledgement.take()
        {
            let _ = acknowledgement.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use tempfile::tempdir;
    use tokio::sync::{mpsc, oneshot, watch};
    use tokio::time::timeout;

    use super::{
        ScanTicks, WorkspaceChangeObservation, WorkspaceChangeOutcome, WorkspaceChangeSource,
        WorkspaceFileEntry, WorkspaceFileState, WorkspaceObservationErrorKind,
    };

    fn controlled_source(
        root_path: std::path::PathBuf,
        state: WorkspaceFileState,
    ) -> (WorkspaceChangeSource, mpsc::Sender<oneshot::Sender<()>>) {
        let (ticks, receiver) = mpsc::channel(8);
        (
            WorkspaceChangeSource {
                root_path,
                state,
                ticks: ScanTicks::Controlled {
                    ticks: receiver,
                    acknowledgement: None,
                },
            },
            ticks,
        )
    }

    async fn scan_once(ticks: &mpsc::Sender<oneshot::Sender<()>>) {
        let (acknowledgement, acknowledged) = oneshot::channel();
        ticks
            .send(acknowledgement)
            .await
            .expect("controlled scan tick must be accepted");
        timeout(Duration::from_secs(1), acknowledged)
            .await
            .expect("controlled scan must not hang")
            .expect("controlled scan must acknowledge completion");
    }

    async fn next_observation(
        observations: &mut watch::Receiver<WorkspaceChangeObservation>,
    ) -> WorkspaceChangeObservation {
        timeout(Duration::from_secs(1), observations.changed())
            .await
            .expect("change observation must not hang")
            .expect("change observation sender must remain open");
        *observations.borrow_and_update()
    }

    #[test]
    fn scan_tracks_complete_regular_file_bytes_paths_and_entry_kinds() {
        let root = tempdir().expect("temporary Workspace must be created");
        fs::create_dir(root.path().join("source")).expect("source directory must be created");
        fs::write(root.path().join("source/module.bsl"), b"first")
            .expect("source file must be created");

        let first = WorkspaceFileState::scan(root.path()).expect("first scan must succeed");
        fs::write(root.path().join("source/module.bsl"), b"other")
            .expect("equal-length source change must succeed");
        let content_changed =
            WorkspaceFileState::scan(root.path()).expect("content scan must succeed");
        assert_ne!(first, content_changed);

        fs::rename(
            root.path().join("source/module.bsl"),
            root.path().join("source/renamed.bsl"),
        )
        .expect("source rename must succeed");
        let renamed = WorkspaceFileState::scan(root.path()).expect("rename scan must succeed");
        assert_ne!(content_changed, renamed);

        let mut other_kind = renamed.clone();
        other_kind.entries.insert(
            std::path::PathBuf::from("source/renamed.bsl"),
            WorkspaceFileEntry::Other,
        );
        assert_ne!(renamed, other_kind);
    }

    #[test]
    fn scan_ignores_descendants_of_every_accepted_ignored_directory() {
        let root = tempdir().expect("temporary Workspace must be created");
        fs::create_dir(root.path().join("source")).expect("source directory must be created");
        fs::write(root.path().join("source/module.bsl"), b"stable")
            .expect("source file must be created");
        for directory in super::IGNORED_DIRECTORIES {
            let ignored = root.path().join(directory);
            fs::create_dir(&ignored).expect("ignored directory must be created");
            fs::write(ignored.join("transient.tmp"), b"first")
                .expect("ignored file must be created");
        }

        let first = WorkspaceFileState::scan(root.path()).expect("first scan must succeed");
        for directory in super::IGNORED_DIRECTORIES {
            fs::write(root.path().join(directory).join("transient.tmp"), b"second")
                .expect("ignored file must be changed");
        }
        let changed = WorkspaceFileState::scan(root.path()).expect("second scan must succeed");

        assert_eq!(first, changed);
    }

    #[tokio::test]
    async fn source_normalizes_changes_failures_recovery_and_latest_revision() {
        let root = tempdir().expect("temporary Workspace must be created");
        let source_file = root.path().join("module.bsl");
        fs::write(&source_file, b"first").expect("source file must be created");
        let state = WorkspaceFileState::scan(root.path()).expect("baseline scan must succeed");
        let (source, ticks) = controlled_source(root.path().to_path_buf(), state);
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let (sender, mut observations) = watch::channel(WorkspaceChangeObservation::initial());
        let task = tokio::spawn(source.run(
            async move {
                let _ = shutdown.await;
            },
            sender,
        ));

        scan_once(&ticks).await;
        assert_eq!(observations.borrow().revision(), 0);

        fs::write(&source_file, b"second").expect("source change must succeed");
        scan_once(&ticks).await;
        let changed = next_observation(&mut observations).await;
        assert_eq!(changed.revision(), 1);
        assert_eq!(changed.outcome(), Some(WorkspaceChangeOutcome::Changed));

        fs::write(&source_file, b"third").expect("second source change must succeed");
        scan_once(&ticks).await;
        fs::write(&source_file, b"fourth").expect("third source change must succeed");
        scan_once(&ticks).await;
        assert_eq!(observations.borrow().revision(), 3);
        assert_eq!(
            observations.borrow().outcome(),
            Some(WorkspaceChangeOutcome::Changed)
        );
        let _ = observations.borrow_and_update();

        fs::remove_dir_all(root.path()).expect("Workspace root removal must succeed");
        scan_once(&ticks).await;
        let failed = next_observation(&mut observations).await;
        assert_eq!(failed.revision(), 4);
        assert_eq!(
            failed.outcome(),
            Some(WorkspaceChangeOutcome::ObservationFailed(
                WorkspaceObservationErrorKind::Root
            ))
        );
        scan_once(&ticks).await;
        assert_eq!(observations.borrow().revision(), 4);

        fs::create_dir(root.path()).expect("Workspace root recovery must succeed");
        fs::write(root.path().join("module.bsl"), b"fourth")
            .expect("recovered source must be created");
        scan_once(&ticks).await;
        let recovered = next_observation(&mut observations).await;
        assert_eq!(recovered.revision(), 5);
        assert_eq!(recovered.outcome(), Some(WorkspaceChangeOutcome::Changed));

        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), task)
            .await
            .expect("source shutdown must not hang")
            .expect("source task must join")
            .expect("source shutdown must succeed");
        assert!(observations.changed().await.is_err());
    }

    #[tokio::test]
    async fn controlled_sources_cancel_cleanly_and_repeat_fresh() {
        async fn run_once() -> WorkspaceChangeObservation {
            let root = tempdir().expect("temporary Workspace must be created");
            let source_file = root.path().join("module.bsl");
            fs::write(&source_file, b"first").expect("source file must be created");
            let state = WorkspaceFileState::scan(root.path()).expect("baseline scan must succeed");
            let (source, ticks) = controlled_source(root.path().to_path_buf(), state);
            let (shutdown_sender, shutdown) = oneshot::channel::<()>();
            let (sender, mut observations) = watch::channel(WorkspaceChangeObservation::initial());
            let task = tokio::spawn(source.run(
                async move {
                    let _ = shutdown.await;
                },
                sender,
            ));

            fs::write(&source_file, b"second").expect("source change must succeed");
            scan_once(&ticks).await;
            let observation = next_observation(&mut observations).await;
            shutdown_sender.send(()).expect("shutdown must be observed");
            timeout(Duration::from_secs(1), task)
                .await
                .expect("source shutdown must not hang")
                .expect("source task must join")
                .expect("source shutdown must succeed");
            assert!(observations.changed().await.is_err());
            observation
        }

        assert_eq!(run_once().await, run_once().await);
    }
}
