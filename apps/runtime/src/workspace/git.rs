//! Bounded local Git repository change reader.

use super::{
    GitChangeSet, GitChangeSetErrorKind, GitCommitId, RepositoryChange, RepositoryChangeKind,
    RepositoryChangePath,
};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::{Future, pending};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::time::{Instant, sleep_until};

const READ_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_CLEANUP_RESERVE: Duration = Duration::from_secs(1);
const STDOUT_LIMIT: usize = 16 * 1024 * 1024;
const STDERR_LIMIT: usize = 64 * 1024;

type CancellationFuture = dyn Future<Output = ()> + Send;
type CommandFuture<'a> =
    Pin<Box<dyn Future<Output = Result<CommandOutput, GitRepositoryReadErrorKind>> + Send + 'a>>;

#[derive(Debug, Clone, Copy)]
struct ReadDeadlines {
    operation: Instant,
    complete: Instant,
}

impl ReadDeadlines {
    fn new(timeout: Duration) -> Self {
        let complete = Instant::now() + timeout;
        let cleanup_reserve = std::cmp::min(timeout / 2, MAX_CLEANUP_RESERVE);
        Self {
            operation: complete - cleanup_reserve,
            complete,
        }
    }
}

/// Closed failure vocabulary for one local Git repository read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitRepositoryReadErrorKind {
    /// The supplied root does not exist or cannot be inspected.
    RootUnavailable,
    /// The supplied root is not a directory.
    RootNotDirectory,
    /// The supplied root is not inside a readable Git repository.
    NotRepository,
    /// Git resolved a worktree root different from the supplied root.
    WorktreeRootMismatch,
    /// The supplied repository is bare and has no accepted worktree.
    BareRepository,
    /// The repository has no readable `HEAD` commit.
    MissingBaseline,
    /// The index contains an unmerged entry.
    ConflictedRepository,
    /// A changed entry has an unsupported repository mode.
    UnsupportedEntryKind,
    /// A repository-relative path is not UTF-8.
    UnsupportedPathEncoding,
    /// A repository-relative path violates the confinement contract.
    InvalidPath,
    /// The normalized result exceeds the accepted change count.
    TooManyChanges,
    /// A child process exceeded a bounded output limit.
    OutputLimitExceeded,
    /// Git returned unsupported or malformed machine output.
    IncompatibleGit,
    /// The two complete observation passes were not equal.
    UnstableRepository,
    /// The local Git executable could not be started.
    SpawnFailed,
    /// A Git command exited or failed before producing accepted evidence.
    ProcessFailed,
    /// The complete repository read exceeded its fixed deadline.
    TimedOut,
    /// The caller cancelled the repository read.
    Cancelled,
}

/// Redacted error returned by [`GitRepositoryReader`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitRepositoryReadError {
    kind: GitRepositoryReadErrorKind,
}

impl GitRepositoryReadError {
    const fn new(kind: GitRepositoryReadErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> GitRepositoryReadErrorKind {
        self.kind
    }
}

impl Display for GitRepositoryReadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.kind {
            GitRepositoryReadErrorKind::RootUnavailable => "Git repository root is unavailable",
            GitRepositoryReadErrorKind::RootNotDirectory => {
                "Git repository root is not a directory"
            }
            GitRepositoryReadErrorKind::NotRepository => "Git repository metadata is unavailable",
            GitRepositoryReadErrorKind::WorktreeRootMismatch => {
                "Git worktree root does not match the supplied root"
            }
            GitRepositoryReadErrorKind::BareRepository => "Bare Git repositories are unsupported",
            GitRepositoryReadErrorKind::MissingBaseline => "Git repository baseline is unavailable",
            GitRepositoryReadErrorKind::ConflictedRepository => {
                "Git repository contains unmerged entries"
            }
            GitRepositoryReadErrorKind::UnsupportedEntryKind => {
                "Git repository contains an unsupported changed entry kind"
            }
            GitRepositoryReadErrorKind::UnsupportedPathEncoding => {
                "Git repository contains a path with unsupported encoding"
            }
            GitRepositoryReadErrorKind::InvalidPath => {
                "Git repository contains an invalid change path"
            }
            GitRepositoryReadErrorKind::TooManyChanges => {
                "Git repository contains too many normalized changes"
            }
            GitRepositoryReadErrorKind::OutputLimitExceeded => {
                "Git command output exceeds the supported limit"
            }
            GitRepositoryReadErrorKind::IncompatibleGit => {
                "Git produced incompatible repository evidence"
            }
            GitRepositoryReadErrorKind::UnstableRepository => {
                "Git repository changed during observation"
            }
            GitRepositoryReadErrorKind::SpawnFailed => "Git command could not be started",
            GitRepositoryReadErrorKind::ProcessFailed => "Git command failed",
            GitRepositoryReadErrorKind::TimedOut => "Git repository read timed out",
            GitRepositoryReadErrorKind::Cancelled => "Git repository read was cancelled",
        })
    }
}

impl Error for GitRepositoryReadError {}

/// Explicit-demand reader for one exact local Git worktree root.
#[derive(Clone)]
pub struct GitRepositoryReader {
    runner: Arc<dyn GitCommandRunner>,
    timeout: Duration,
}

impl GitRepositoryReader {
    /// Creates a reader backed by the caller's local `git` executable.
    #[must_use]
    pub fn new() -> Self {
        Self {
            runner: Arc::new(ProductionGitCommandRunner),
            timeout: READ_TIMEOUT,
        }
    }

    /// Reads one stable normalized change set without an explicit cancellation
    /// signal. Dropping the returned future synchronously terminates and reaps
    /// any active child before the drop completes.
    ///
    /// # Errors
    ///
    /// Returns [`GitRepositoryReadError`] when the root, repository, process,
    /// output, stability, path, or normalized result violates the accepted
    /// contract.
    pub async fn read(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<GitChangeSet, GitRepositoryReadError> {
        self.read_with_cancellation(root, pending()).await
    }

    /// Reads one stable normalized change set with caller-owned cancellation.
    ///
    /// The cancellation future is polled throughout filesystem probes and Git
    /// commands. Cancellation terminates and joins an active child before this
    /// method returns.
    ///
    /// # Errors
    ///
    /// Returns [`GitRepositoryReadError`] under the same conditions as
    /// [`Self::read`], or with [`GitRepositoryReadErrorKind::Cancelled`] when
    /// the supplied future completes first.
    pub async fn read_with_cancellation<F>(
        &self,
        root: impl AsRef<Path>,
        cancellation: F,
    ) -> Result<GitChangeSet, GitRepositoryReadError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let root = root.as_ref().to_path_buf();
        let deadlines = ReadDeadlines::new(self.timeout);
        let mut cancellation: Pin<Box<CancellationFuture>> = Box::pin(cancellation);
        self.read_inner(&root, deadlines, &mut cancellation)
            .await
            .map_err(GitRepositoryReadError::new)
    }

    async fn read_inner(
        &self,
        root: &Path,
        deadlines: ReadDeadlines,
        cancellation: &mut Pin<Box<CancellationFuture>>,
    ) -> Result<GitChangeSet, GitRepositoryReadErrorKind> {
        let metadata = controlled(
            tokio::fs::metadata(root),
            deadlines.operation,
            cancellation.as_mut(),
        )
        .await?
        .map_err(|_| GitRepositoryReadErrorKind::RootUnavailable)?;
        if !metadata.is_dir() {
            return Err(GitRepositoryReadErrorKind::RootNotDirectory);
        }
        let canonical_root = controlled(
            tokio::fs::canonicalize(root),
            deadlines.operation,
            cancellation.as_mut(),
        )
        .await?
        .map_err(|_| GitRepositoryReadErrorKind::RootUnavailable)?;

        let bare = self
            .run(
                GitCommand::BareRepository,
                &canonical_root,
                deadlines,
                cancellation.as_mut(),
            )
            .await
            .map_err(|kind| contextual_process_error(GitCommand::BareRepository, kind))?;
        match trim_one_line(&bare.stdout)? {
            b"false" => {}
            b"true" => return Err(GitRepositoryReadErrorKind::BareRepository),
            _ => return Err(GitRepositoryReadErrorKind::IncompatibleGit),
        }

        let first = self
            .read_pass(&canonical_root, deadlines, cancellation)
            .await?;
        let second = self
            .read_pass(&canonical_root, deadlines, cancellation)
            .await?;
        if first != second {
            return Err(GitRepositoryReadErrorKind::UnstableRepository);
        }
        Ok(first.change_set)
    }

    async fn read_pass(
        &self,
        root: &Path,
        deadlines: ReadDeadlines,
        cancellation: &mut Pin<Box<CancellationFuture>>,
    ) -> Result<PassEvidence, GitRepositoryReadErrorKind> {
        let top_level = self
            .run(GitCommand::TopLevel, root, deadlines, cancellation.as_mut())
            .await
            .map_err(|kind| contextual_process_error(GitCommand::TopLevel, kind))?;
        let top_level = parse_top_level(&top_level.stdout)?;
        let top_level = controlled(
            tokio::fs::canonicalize(top_level),
            deadlines.operation,
            cancellation.as_mut(),
        )
        .await?
        .map_err(|_| GitRepositoryReadErrorKind::WorktreeRootMismatch)?;
        if top_level != root {
            return Err(GitRepositoryReadErrorKind::WorktreeRootMismatch);
        }

        let head = self
            .run(GitCommand::Head, root, deadlines, cancellation.as_mut())
            .await
            .map_err(|kind| contextual_process_error(GitCommand::Head, kind))?;
        let baseline = parse_head(&head.stdout)?;

        let conflicts = self
            .run(
                GitCommand::Conflicts,
                root,
                deadlines,
                cancellation.as_mut(),
            )
            .await?;
        if !conflicts.stdout.is_empty() {
            return Err(GitRepositoryReadErrorKind::ConflictedRepository);
        }

        let tracked = self
            .run(
                GitCommand::TrackedChanges,
                root,
                deadlines,
                cancellation.as_mut(),
            )
            .await?;
        let mut tracked = parse_tracked_changes(&tracked.stdout)?;
        tracked.sort_unstable();
        tracked.dedup();

        let untracked = self
            .run(
                GitCommand::UntrackedPaths,
                root,
                deadlines,
                cancellation.as_mut(),
            )
            .await?;
        let mut changes = tracked
            .iter()
            .map(|evidence| evidence.change.clone())
            .collect::<Vec<_>>();
        changes.extend(parse_untracked_changes(&untracked.stdout)?);
        let change_set =
            GitChangeSet::new(baseline, changes).map_err(|error| match error.kind() {
                GitChangeSetErrorKind::TooManyChanges => GitRepositoryReadErrorKind::TooManyChanges,
                GitChangeSetErrorKind::ConflictingChange => {
                    GitRepositoryReadErrorKind::IncompatibleGit
                }
            })?;

        Ok(PassEvidence {
            top_level,
            tracked: tracked.into_boxed_slice(),
            change_set,
        })
    }

    async fn run(
        &self,
        command: GitCommand,
        root: &Path,
        deadlines: ReadDeadlines,
        cancellation: Pin<&mut CancellationFuture>,
    ) -> Result<CommandOutput, GitRepositoryReadErrorKind> {
        self.runner
            .run(command, root, deadlines, cancellation)
            .await
    }

    #[cfg(test)]
    fn with_runner(runner: Arc<dyn GitCommandRunner>, timeout: Duration) -> Self {
        Self { runner, timeout }
    }
}

impl Default for GitRepositoryReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for GitRepositoryReader {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GitRepositoryReader")
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GitCommand {
    BareRepository,
    TopLevel,
    Head,
    Conflicts,
    TrackedChanges,
    UntrackedPaths,
}

#[derive(Debug)]
struct CommandOutput {
    stdout: Vec<u8>,
}

trait GitCommandRunner: Send + Sync {
    fn run<'a>(
        &'a self,
        command: GitCommand,
        root: &'a Path,
        deadlines: ReadDeadlines,
        cancellation: Pin<&'a mut CancellationFuture>,
    ) -> CommandFuture<'a>;
}

#[derive(Debug)]
struct ProductionGitCommandRunner;

impl GitCommandRunner for ProductionGitCommandRunner {
    fn run<'a>(
        &'a self,
        command: GitCommand,
        root: &'a Path,
        deadlines: ReadDeadlines,
        cancellation: Pin<&'a mut CancellationFuture>,
    ) -> CommandFuture<'a> {
        Box::pin(run_production_command(
            command,
            root,
            deadlines,
            cancellation,
        ))
    }
}

async fn run_production_command(
    operation: GitCommand,
    root: &Path,
    deadlines: ReadDeadlines,
    cancellation: Pin<&mut CancellationFuture>,
) -> Result<CommandOutput, GitRepositoryReadErrorKind> {
    let child = production_command(operation, root)
        .spawn()
        .map_err(|_| GitRepositoryReadErrorKind::SpawnFailed)?;
    Box::pin(collect_child_output(child, deadlines, cancellation)).await
}

fn production_command(operation: GitCommand, root: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("--no-pager")
        .args(["-c", "color.ui=false"])
        .args(["-c", "core.quotePath=false"])
        .args(["-c", "diff.renames=false"])
        .args(["-c", "core.fsmonitor=false"])
        .args(["-c", "core.untrackedCache=false"])
        .args(["-c", "core.excludesFile="])
        .args(["-c", "credential.helper="])
        .arg("-C")
        .arg(root);
    match operation {
        GitCommand::BareRepository => {
            command.args(["rev-parse", "--is-bare-repository"]);
        }
        GitCommand::TopLevel => {
            command.args(["rev-parse", "--show-toplevel"]);
        }
        GitCommand::Head => {
            command.args(["rev-parse", "--verify", "HEAD"]);
        }
        GitCommand::Conflicts => {
            command.args(["ls-files", "--unmerged", "-z"]);
        }
        GitCommand::TrackedChanges => {
            command.args([
                "diff",
                "--raw",
                "-z",
                "--abbrev=64",
                "--no-renames",
                "HEAD",
                "--",
            ]);
        }
        GitCommand::UntrackedPaths => {
            command.args(["ls-files", "--others", "--exclude-standard", "-z"]);
        }
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env("GIT_LITERAL_PATHSPECS", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("LC_ALL", "C")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_NAMESPACE")
        .env_remove("GIT_CONFIG_COUNT");
    command
}

async fn collect_child_output(
    child: Child,
    deadlines: ReadDeadlines,
    cancellation: Pin<&mut CancellationFuture>,
) -> Result<CommandOutput, GitRepositoryReadErrorKind> {
    Box::pin(collect_guarded_child_output(
        ChildGuard::new(child),
        deadlines,
        cancellation,
    ))
    .await
}

async fn collect_guarded_child_output(
    mut child: ChildGuard,
    deadlines: ReadDeadlines,
    mut cancellation: Pin<&mut CancellationFuture>,
) -> Result<CommandOutput, GitRepositoryReadErrorKind> {
    let stdout = child
        .child_mut()
        .stdout
        .take()
        .ok_or(GitRepositoryReadErrorKind::ProcessFailed)?;
    let stderr = child
        .child_mut()
        .stderr
        .take()
        .ok_or(GitRepositoryReadErrorKind::ProcessFailed)?;
    let stdout_read = read_bounded(stdout, STDOUT_LIMIT);
    let stderr_read = read_bounded(stderr, STDERR_LIMIT);
    tokio::pin!(stdout_read);
    tokio::pin!(stderr_read);
    let mut stdout_finished = false;
    let mut stderr_finished = false;

    let outcome = {
        let mut wait = Box::pin(child.child_mut().wait());
        let mut status = None;
        let mut stdout = None;
        let mut stderr = None;
        let mut timeout = Box::pin(sleep_until(deadlines.operation));

        'completion: loop {
            tokio::select! {
                biased;
                () = cancellation.as_mut() => {
                    break 'completion ProcessOutcome::Interrupted(
                        GitRepositoryReadErrorKind::Cancelled,
                    );
                }
                () = &mut timeout => {
                    break 'completion ProcessOutcome::Interrupted(
                        GitRepositoryReadErrorKind::TimedOut,
                    );
                }
                result = &mut wait, if status.is_none() => {
                    match result {
                        Ok(value) => status = Some(value),
                        Err(_) => {
                            break 'completion ProcessOutcome::Interrupted(
                                GitRepositoryReadErrorKind::ProcessFailed,
                            );
                        }
                    }
                }
                result = &mut stdout_read, if !stdout_finished =>
                {
                    stdout_finished = true;
                    match result {
                        Ok(value) => stdout = Some(value),
                        Err(kind) => break 'completion ProcessOutcome::Interrupted(kind),
                    }
                }
                result = &mut stderr_read, if !stderr_finished =>
                {
                    stderr_finished = true;
                    match result {
                        Ok(value) => stderr = Some(value),
                        Err(kind) => break 'completion ProcessOutcome::Interrupted(kind),
                    }
                }
            }

            if status.is_some() && stdout.is_some() && stderr.is_some() {
                break ProcessOutcome::Complete {
                    status: status.take().expect("completed process status is present"),
                    stdout: stdout.take().expect("completed stdout is present"),
                    _stderr: stderr.take().expect("completed stderr is present"),
                };
            }
        }
    };

    match outcome {
        ProcessOutcome::Complete {
            status,
            stdout,
            _stderr: _,
        } => {
            child.disarm();
            if status.success() {
                Ok(CommandOutput { stdout })
            } else {
                Err(GitRepositoryReadErrorKind::ProcessFailed)
            }
        }
        ProcessOutcome::Interrupted(kind) => {
            child.terminate_and_wait(deadlines.complete).await;
            Err(kind)
        }
    }
}

enum ProcessOutcome {
    Complete {
        status: ExitStatus,
        stdout: Vec<u8>,
        _stderr: Vec<u8>,
    },
    Interrupted(GitRepositoryReadErrorKind),
}

struct ChildGuard {
    child: Option<Child>,
    #[cfg(test)]
    reaped: Option<Arc<std::sync::atomic::AtomicBool>>,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self {
            child: Some(child),
            #[cfg(test)]
            reaped: None,
        }
    }

    #[cfg(test)]
    fn with_reap_observer(child: Child, reaped: Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self {
            child: Some(child),
            reaped: Some(reaped),
        }
    }

    fn child_mut(&mut self) -> &mut Child {
        self.child
            .as_mut()
            .expect("active child guard always owns its child")
    }

    fn disarm(&mut self) {
        self.child.take();
        Self::mark_reaped(self);
    }

    async fn terminate_and_wait(&mut self, deadline: Instant) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
            let reaped = {
                let wait = child.wait();
                tokio::pin!(wait);
                let timeout = sleep_until(deadline);
                tokio::pin!(timeout);
                tokio::select! {
                    biased;
                    result = &mut wait => result.is_ok(),
                    () = &mut timeout => false,
                }
            };
            if reaped || reap_child_blocking(&mut child) {
                Self::mark_reaped(self);
            }
        }
    }

    fn mark_reaped(guard: &Self) {
        #[cfg(test)]
        if let Some(reaped) = &guard.reaped {
            reaped.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        #[cfg(not(test))]
        let _ = guard;
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
            if reap_child_blocking(&mut child) {
                Self::mark_reaped(self);
            }
        }
    }
}

fn reap_child_blocking(child: &mut Child) -> bool {
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => std::thread::sleep(Duration::from_millis(1)),
            Err(_) => return false,
        }
    }
}

async fn read_bounded<R>(mut reader: R, limit: usize) -> Result<Vec<u8>, GitRepositoryReadErrorKind>
where
    R: AsyncRead + Unpin,
{
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(|_| GitRepositoryReadErrorKind::ProcessFailed)?;
        if count == 0 {
            return Ok(output);
        }
        if output.len().saturating_add(count) > limit {
            return Err(GitRepositoryReadErrorKind::OutputLimitExceeded);
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

async fn controlled<T, F>(
    future: F,
    deadline: Instant,
    mut cancellation: Pin<&mut CancellationFuture>,
) -> Result<T, GitRepositoryReadErrorKind>
where
    F: Future<Output = T>,
{
    tokio::pin!(future);
    let timeout = sleep_until(deadline);
    tokio::pin!(timeout);
    tokio::select! {
        biased;
        () = cancellation.as_mut() => Err(GitRepositoryReadErrorKind::Cancelled),
        () = &mut timeout => Err(GitRepositoryReadErrorKind::TimedOut),
        value = &mut future => Ok(value),
    }
}

fn contextual_process_error(
    command: GitCommand,
    kind: GitRepositoryReadErrorKind,
) -> GitRepositoryReadErrorKind {
    if kind != GitRepositoryReadErrorKind::ProcessFailed {
        return kind;
    }
    match command {
        GitCommand::BareRepository | GitCommand::TopLevel => {
            GitRepositoryReadErrorKind::NotRepository
        }
        GitCommand::Head => GitRepositoryReadErrorKind::MissingBaseline,
        GitCommand::Conflicts | GitCommand::TrackedChanges | GitCommand::UntrackedPaths => kind,
    }
}

fn trim_one_line(output: &[u8]) -> Result<&[u8], GitRepositoryReadErrorKind> {
    let output = output.strip_suffix(b"\n").unwrap_or(output);
    let output = output.strip_suffix(b"\r").unwrap_or(output);
    if output.is_empty() || output.contains(&b'\n') || output.contains(&b'\r') {
        return Err(GitRepositoryReadErrorKind::IncompatibleGit);
    }
    Ok(output)
}

fn parse_top_level(output: &[u8]) -> Result<PathBuf, GitRepositoryReadErrorKind> {
    let output = trim_one_line(output)?;
    let value = std::str::from_utf8(output)
        .map_err(|_| GitRepositoryReadErrorKind::UnsupportedPathEncoding)?;
    Ok(PathBuf::from(value))
}

fn parse_head(output: &[u8]) -> Result<GitCommitId, GitRepositoryReadErrorKind> {
    let output = trim_one_line(output)?;
    let value =
        std::str::from_utf8(output).map_err(|_| GitRepositoryReadErrorKind::IncompatibleGit)?;
    GitCommitId::new(value).map_err(|_| GitRepositoryReadErrorKind::IncompatibleGit)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PassEvidence {
    top_level: PathBuf,
    tracked: Box<[TrackedEvidence]>,
    change_set: GitChangeSet,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TrackedEvidence {
    old_mode: [u8; 6],
    new_mode: [u8; 6],
    change: RepositoryChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Absent,
    Regular,
    Symlink,
}

fn parse_tracked_changes(
    output: &[u8],
) -> Result<Vec<TrackedEvidence>, GitRepositoryReadErrorKind> {
    let mut cursor = 0;
    let mut changes = Vec::new();
    while cursor < output.len() {
        let header = take_nul_field(output, &mut cursor)?;
        let path = take_nul_field(output, &mut cursor)?;
        let mut fields = header.split(|byte| *byte == b' ');
        let old_mode = parse_mode_field(fields.next())?;
        let new_mode = parse_plain_mode_field(fields.next())?;
        let old_object = fields
            .next()
            .ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
        let new_object = fields
            .next()
            .ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
        let status = fields
            .next()
            .ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
        if fields.next().is_some()
            || !valid_raw_object_id(old_object)
            || !valid_raw_object_id(new_object)
            || status.len() != 1
        {
            return Err(GitRepositoryReadErrorKind::IncompatibleGit);
        }
        let old_kind = classify_mode(old_mode)?;
        let new_kind = classify_mode(new_mode)?;
        let path = parse_change_path(path)?;
        let change = match status[0] {
            b'A' if old_kind == EntryKind::Absent && new_kind != EntryKind::Absent => {
                RepositoryChange::new(RepositoryChangeKind::Added, None, Some(path))
            }
            b'D' if old_kind != EntryKind::Absent && new_kind == EntryKind::Absent => {
                RepositoryChange::new(RepositoryChangeKind::Deleted, Some(path), None)
            }
            b'M' if old_kind != EntryKind::Absent && new_kind != EntryKind::Absent => {
                let kind = if old_kind == new_kind {
                    RepositoryChangeKind::Modified
                } else {
                    RepositoryChangeKind::TypeChanged
                };
                RepositoryChange::new(kind, Some(path.clone()), Some(path))
            }
            b'T' if old_kind != EntryKind::Absent
                && new_kind != EntryKind::Absent
                && old_kind != new_kind =>
            {
                RepositoryChange::new(
                    RepositoryChangeKind::TypeChanged,
                    Some(path.clone()),
                    Some(path),
                )
            }
            b'U' => return Err(GitRepositoryReadErrorKind::ConflictedRepository),
            _ => return Err(GitRepositoryReadErrorKind::IncompatibleGit),
        }
        .map_err(|_| GitRepositoryReadErrorKind::IncompatibleGit)?;
        changes.push(TrackedEvidence {
            old_mode,
            new_mode,
            change,
        });
    }
    Ok(changes)
}

fn parse_untracked_changes(
    output: &[u8],
) -> Result<Vec<RepositoryChange>, GitRepositoryReadErrorKind> {
    let mut cursor = 0;
    let mut changes = Vec::new();
    while cursor < output.len() {
        let path = parse_change_path(take_nul_field(output, &mut cursor)?)?;
        changes.push(
            RepositoryChange::new(RepositoryChangeKind::Untracked, None, Some(path))
                .map_err(|_| GitRepositoryReadErrorKind::IncompatibleGit)?,
        );
    }
    Ok(changes)
}

fn take_nul_field<'a>(
    output: &'a [u8],
    cursor: &mut usize,
) -> Result<&'a [u8], GitRepositoryReadErrorKind> {
    let remaining = output
        .get(*cursor..)
        .ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
    let end = remaining
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
    if end == 0 {
        return Err(GitRepositoryReadErrorKind::IncompatibleGit);
    }
    let field = &remaining[..end];
    *cursor = cursor
        .checked_add(end + 1)
        .ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
    Ok(field)
}

fn parse_mode_field(field: Option<&[u8]>) -> Result<[u8; 6], GitRepositoryReadErrorKind> {
    let field = field.ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
    let field = field
        .strip_prefix(b":")
        .ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?;
    parse_mode(field)
}

fn parse_plain_mode_field(field: Option<&[u8]>) -> Result<[u8; 6], GitRepositoryReadErrorKind> {
    parse_mode(field.ok_or(GitRepositoryReadErrorKind::IncompatibleGit)?)
}

fn parse_mode(field: &[u8]) -> Result<[u8; 6], GitRepositoryReadErrorKind> {
    let mode: [u8; 6] = field
        .try_into()
        .map_err(|_| GitRepositoryReadErrorKind::IncompatibleGit)?;
    if !mode.iter().all(u8::is_ascii_digit) || mode.iter().any(|byte| *byte > b'7') {
        return Err(GitRepositoryReadErrorKind::IncompatibleGit);
    }
    Ok(mode)
}

fn classify_mode(mode: [u8; 6]) -> Result<EntryKind, GitRepositoryReadErrorKind> {
    match &mode {
        b"000000" => Ok(EntryKind::Absent),
        b"100644" | b"100755" => Ok(EntryKind::Regular),
        b"120000" => Ok(EntryKind::Symlink),
        _ => Err(GitRepositoryReadErrorKind::UnsupportedEntryKind),
    }
}

fn valid_raw_object_id(value: &[u8]) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn parse_change_path(value: &[u8]) -> Result<RepositoryChangePath, GitRepositoryReadErrorKind> {
    let value = std::str::from_utf8(value)
        .map_err(|_| GitRepositoryReadErrorKind::UnsupportedPathEncoding)?;
    RepositoryChangePath::new(value.to_owned()).map_err(|_| GitRepositoryReadErrorKind::InvalidPath)
}

#[cfg(test)]
mod tests {
    use super::{
        CancellationFuture, ChildGuard, CommandFuture, CommandOutput, GitCommand, GitCommandRunner,
        GitRepositoryReadErrorKind, GitRepositoryReader, ReadDeadlines, STDOUT_LIMIT,
        collect_guarded_child_output, parse_tracked_changes, parse_untracked_changes, read_bounded,
    };
    use crate::{MAX_REPOSITORY_CHANGE_PATH_BYTES, MAX_REPOSITORY_CHANGES, RepositoryChangeKind};
    use std::collections::VecDeque;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::pin::Pin;
    use std::process::Stdio;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::io::{AsyncRead, AsyncWriteExt, ReadBuf};
    use tokio::sync::{Notify, oneshot};
    use tokio::time::{sleep_until, timeout};

    const SHA1: &str = "0123456789abcdef0123456789abcdef01234567";
    const CLEANUP_HELPER_ENV: &str = "ONEAGENT_GIT_CLEANUP_HELPER";

    enum ScriptResult {
        Output(Vec<u8>),
        Failure(ScriptFailure),
        Pending {
            dropped: Arc<AtomicBool>,
            started: Arc<Notify>,
        },
    }

    #[derive(Clone, Copy)]
    enum ScriptFailure {
        Spawn,
        StdoutRead,
        StderrRead,
        Exit,
    }

    impl ScriptFailure {
        const fn kind(self) -> GitRepositoryReadErrorKind {
            match self {
                Self::Spawn => GitRepositoryReadErrorKind::SpawnFailed,
                Self::StdoutRead | Self::StderrRead | Self::Exit => {
                    GitRepositoryReadErrorKind::ProcessFailed
                }
            }
        }
    }

    struct ScriptStep {
        command: GitCommand,
        result: ScriptResult,
    }

    struct ScriptedRunner {
        steps: Mutex<VecDeque<ScriptStep>>,
    }

    impl ScriptedRunner {
        fn new(steps: Vec<ScriptStep>) -> Self {
            Self {
                steps: Mutex::new(steps.into()),
            }
        }
    }

    impl GitCommandRunner for ScriptedRunner {
        fn run<'a>(
            &'a self,
            command: GitCommand,
            _root: &'a Path,
            deadlines: ReadDeadlines,
            mut cancellation: Pin<&'a mut CancellationFuture>,
        ) -> CommandFuture<'a> {
            let step = self
                .steps
                .lock()
                .expect("script lock must remain available")
                .pop_front()
                .expect("script must contain every expected command");
            assert_eq!(step.command, command);
            Box::pin(async move {
                match step.result {
                    ScriptResult::Output(stdout) => Ok(CommandOutput { stdout }),
                    ScriptResult::Failure(failure) => Err(failure.kind()),
                    ScriptResult::Pending { dropped, started } => {
                        struct DropSignal(Arc<AtomicBool>);
                        impl Drop for DropSignal {
                            fn drop(&mut self) {
                                self.0.store(true, Ordering::SeqCst);
                            }
                        }
                        let _signal = DropSignal(dropped);
                        started.notify_one();
                        let timeout = sleep_until(deadlines.operation);
                        tokio::pin!(timeout);
                        tokio::select! {
                            biased;
                            () = cancellation.as_mut() => {
                                Err(GitRepositoryReadErrorKind::Cancelled)
                            }
                            () = &mut timeout => Err(GitRepositoryReadErrorKind::TimedOut),
                        }
                    }
                }
            })
        }
    }

    struct ProductionBoundaryRunner {
        started: Arc<Notify>,
        reaped: Arc<AtomicBool>,
    }

    struct FailingAsyncReader;

    impl AsyncRead for FailingAsyncReader {
        fn poll_read(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            _buffer: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::other("injected read failure")))
        }
    }

    impl GitCommandRunner for ProductionBoundaryRunner {
        fn run<'a>(
            &'a self,
            command: GitCommand,
            _root: &'a Path,
            deadlines: ReadDeadlines,
            cancellation: Pin<&'a mut CancellationFuture>,
        ) -> CommandFuture<'a> {
            assert_eq!(command, GitCommand::BareRepository);
            let started = Arc::clone(&self.started);
            let reaped = Arc::clone(&self.reaped);
            Box::pin(async move {
                let executable =
                    std::env::current_exe().map_err(|_| GitRepositoryReadErrorKind::SpawnFailed)?;
                let child = tokio::process::Command::new(executable)
                    .args([
                        "--exact",
                        "workspace::git::tests::production_cleanup_child_helper",
                        "--nocapture",
                    ])
                    .env(CLEANUP_HELPER_ENV, "1")
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true)
                    .spawn()
                    .map_err(|_| GitRepositoryReadErrorKind::SpawnFailed)?;
                started.notify_one();
                Box::pin(collect_guarded_child_output(
                    ChildGuard::with_reap_observer(child, reaped),
                    deadlines,
                    cancellation,
                ))
                .await
            })
        }
    }

    fn production_boundary_reader(
        read_timeout: Duration,
    ) -> (GitRepositoryReader, Arc<Notify>, Arc<AtomicBool>) {
        let started = Arc::new(Notify::new());
        let reaped = Arc::new(AtomicBool::new(false));
        let runner = ProductionBoundaryRunner {
            started: Arc::clone(&started),
            reaped: Arc::clone(&reaped),
        };
        (
            GitRepositoryReader::with_runner(Arc::new(runner), read_timeout),
            started,
            reaped,
        )
    }

    fn output(command: GitCommand, value: impl Into<Vec<u8>>) -> ScriptStep {
        ScriptStep {
            command,
            result: ScriptResult::Output(value.into()),
        }
    }

    fn pass(root: &Path, tracked: Vec<u8>, untracked: Vec<u8>) -> Vec<ScriptStep> {
        vec![
            output(
                GitCommand::TopLevel,
                format!("{}\n", root.display()).into_bytes(),
            ),
            output(GitCommand::Head, format!("{SHA1}\n").into_bytes()),
            output(GitCommand::Conflicts, Vec::new()),
            output(GitCommand::TrackedChanges, tracked),
            output(GitCommand::UntrackedPaths, untracked),
        ]
    }

    fn raw(status: char, old_mode: &str, new_mode: &str, path: &str) -> Vec<u8> {
        format!(
            ":{old_mode} {new_mode} {SHA1} {} {status}\0{path}\0",
            "0".repeat(40)
        )
        .into_bytes()
    }

    fn reader(steps: Vec<ScriptStep>, timeout: Duration) -> GitRepositoryReader {
        GitRepositoryReader::with_runner(Arc::new(ScriptedRunner::new(steps)), timeout)
    }

    fn root() -> (TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("temporary root must be created");
        let root = std::fs::canonicalize(temp.path()).expect("temporary root must canonicalize");
        (temp, root)
    }

    #[test]
    fn production_cleanup_child_helper() {
        if std::env::var_os(CLEANUP_HELPER_ENV).is_some() {
            std::thread::sleep(Duration::from_secs(10));
        }
    }

    #[tokio::test]
    async fn injected_reader_normalizes_output_order_and_requires_equal_passes() {
        let (_temp, root) = root();
        let modified = raw('M', "100644", "100644", "z/file");
        let added = raw('A', "000000", "100644", "a/file");
        let mut first_raw = modified.clone();
        first_raw.extend(added.clone());
        let mut second_raw = added;
        second_raw.extend(modified);
        let mut steps = vec![output(GitCommand::BareRepository, b"false\n".to_vec())];
        steps.extend(pass(&root, first_raw, b"u/file\0".to_vec()));
        steps.extend(pass(&root, second_raw, b"u/file\0".to_vec()));

        let set = reader(steps, Duration::from_secs(1))
            .read(&root)
            .await
            .expect("equivalent passes must be accepted");
        assert_eq!(set.changes().len(), 3);
        assert_eq!(set.changes()[0].kind(), RepositoryChangeKind::Added);
        assert_eq!(set.changes()[1].kind(), RepositoryChangeKind::Untracked);
        assert_eq!(set.changes()[2].kind(), RepositoryChangeKind::Modified);
    }

    #[tokio::test]
    async fn injected_reader_rejects_pass_drift_malformed_and_unsupported_modes() {
        let (_temp, root) = root();
        let mut unstable = vec![output(GitCommand::BareRepository, b"false\n".to_vec())];
        unstable.extend(pass(&root, Vec::new(), Vec::new()));
        unstable.extend(pass(&root, Vec::new(), b"later\0".to_vec()));
        let error = reader(unstable, Duration::from_secs(1))
            .read(&root)
            .await
            .expect_err("different passes must fail");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::UnstableRepository);

        let malformed = parse_tracked_changes(b"not-raw\0secret/path\0")
            .expect_err("malformed raw output must fail");
        assert_eq!(malformed, GitRepositoryReadErrorKind::IncompatibleGit);
        let gitlink = parse_tracked_changes(&raw('M', "160000", "160000", "submodule"))
            .expect_err("gitlink changes must fail");
        assert_eq!(gitlink, GitRepositoryReadErrorKind::UnsupportedEntryKind);
        let non_utf8 =
            parse_untracked_changes(b"private-\xff\0").expect_err("non-UTF-8 paths must fail");
        assert_eq!(
            non_utf8,
            GitRepositoryReadErrorKind::UnsupportedPathEncoding
        );
    }

    #[tokio::test]
    async fn injected_reader_enforces_exact_and_one_over_change_and_path_bounds() {
        let (_temp, root) = root();
        let exact = (0..MAX_REPOSITORY_CHANGES)
            .flat_map(|index| format!("p/{index:05}\0").into_bytes())
            .collect::<Vec<_>>();
        let mut steps = vec![output(GitCommand::BareRepository, b"false\n".to_vec())];
        steps.extend(pass(&root, Vec::new(), exact.clone()));
        steps.extend(pass(&root, Vec::new(), exact));
        let set = reader(steps, Duration::from_secs(2))
            .read(&root)
            .await
            .expect("exact normalized count must pass through the reader");
        assert_eq!(set.changes().len(), MAX_REPOSITORY_CHANGES);

        let over = (0..=MAX_REPOSITORY_CHANGES)
            .flat_map(|index| format!("p/{index:05}\0").into_bytes())
            .collect::<Vec<_>>();
        let mut steps = vec![output(GitCommand::BareRepository, b"false\n".to_vec())];
        steps.extend(pass(&root, Vec::new(), over));
        let error = reader(steps, Duration::from_secs(2))
            .read(&root)
            .await
            .expect_err("one-over normalized count must fail through the reader");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::TooManyChanges);

        let exact_path = format!("{}\0", "a".repeat(MAX_REPOSITORY_CHANGE_PATH_BYTES));
        let mut steps = vec![output(GitCommand::BareRepository, b"false\n".to_vec())];
        steps.extend(pass(&root, Vec::new(), exact_path.as_bytes().to_vec()));
        steps.extend(pass(&root, Vec::new(), exact_path.into_bytes()));
        let set = reader(steps, Duration::from_secs(1))
            .read(&root)
            .await
            .expect("exact path bound must pass through the reader");
        assert_eq!(set.changes().len(), 1);

        let over_path = format!("{}\0", "a".repeat(MAX_REPOSITORY_CHANGE_PATH_BYTES + 1));
        let mut steps = vec![output(GitCommand::BareRepository, b"false\n".to_vec())];
        steps.extend(pass(&root, Vec::new(), over_path.into_bytes()));
        let error = reader(steps, Duration::from_secs(1))
            .read(&root)
            .await
            .expect_err("one-over path bound must fail through the reader");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::InvalidPath);
    }

    #[tokio::test]
    async fn injected_runner_errors_are_contextual_closed_and_redacted() {
        let (_temp, root) = root();
        let secret = "private-secret-root";
        let kinds = [
            GitRepositoryReadErrorKind::RootUnavailable,
            GitRepositoryReadErrorKind::RootNotDirectory,
            GitRepositoryReadErrorKind::NotRepository,
            GitRepositoryReadErrorKind::WorktreeRootMismatch,
            GitRepositoryReadErrorKind::BareRepository,
            GitRepositoryReadErrorKind::MissingBaseline,
            GitRepositoryReadErrorKind::ConflictedRepository,
            GitRepositoryReadErrorKind::UnsupportedEntryKind,
            GitRepositoryReadErrorKind::UnsupportedPathEncoding,
            GitRepositoryReadErrorKind::InvalidPath,
            GitRepositoryReadErrorKind::TooManyChanges,
            GitRepositoryReadErrorKind::OutputLimitExceeded,
            GitRepositoryReadErrorKind::IncompatibleGit,
            GitRepositoryReadErrorKind::UnstableRepository,
            GitRepositoryReadErrorKind::SpawnFailed,
            GitRepositoryReadErrorKind::ProcessFailed,
            GitRepositoryReadErrorKind::TimedOut,
            GitRepositoryReadErrorKind::Cancelled,
        ];
        for kind in kinds {
            let error = super::GitRepositoryReadError::new(kind);
            assert_eq!(error.kind(), kind);
            assert!(!error.to_string().contains(secret));
            assert!(error.to_string().len() < 96);
        }

        let runner = reader(
            vec![ScriptStep {
                command: GitCommand::BareRepository,
                result: ScriptResult::Failure(ScriptFailure::Exit),
            }],
            Duration::from_secs(1),
        );
        let error = runner
            .read(&root)
            .await
            .expect_err("bare probe process failure must classify repository absence");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::NotRepository);
    }

    #[tokio::test]
    async fn injected_runner_executes_spawn_read_and_exit_failure_matrix() {
        let (_temp, root) = root();

        let spawn = reader(
            vec![ScriptStep {
                command: GitCommand::BareRepository,
                result: ScriptResult::Failure(ScriptFailure::Spawn),
            }],
            Duration::from_secs(1),
        );
        let error = spawn
            .read(&root)
            .await
            .expect_err("injected spawn failure must close the complete read");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::SpawnFailed);

        for (failure, command) in [
            (ScriptFailure::StdoutRead, GitCommand::Conflicts),
            (ScriptFailure::StderrRead, GitCommand::TrackedChanges),
            (ScriptFailure::Exit, GitCommand::UntrackedPaths),
        ] {
            let mut steps = vec![output(GitCommand::BareRepository, b"false\n".to_vec())];
            steps.push(output(
                GitCommand::TopLevel,
                format!("{}\n", root.display()).into_bytes(),
            ));
            steps.push(output(GitCommand::Head, format!("{SHA1}\n").into_bytes()));
            if command != GitCommand::Conflicts {
                steps.push(output(GitCommand::Conflicts, Vec::new()));
            }
            if command == GitCommand::UntrackedPaths {
                steps.push(output(GitCommand::TrackedChanges, Vec::new()));
            }
            steps.push(ScriptStep {
                command,
                result: ScriptResult::Failure(failure),
            });
            let error = reader(steps, Duration::from_secs(1))
                .read(&root)
                .await
                .expect_err("injected post-context process failure must close the complete read");
            assert_eq!(error.kind(), GitRepositoryReadErrorKind::ProcessFailed);
        }

        let error = read_bounded(FailingAsyncReader, STDOUT_LIMIT)
            .await
            .expect_err("injected AsyncRead failure must remain closed");
        assert_eq!(error, GitRepositoryReadErrorKind::ProcessFailed);
    }

    #[tokio::test]
    async fn cancellation_and_timeout_drop_pending_runner_work() {
        let (_temp, root) = root();
        let cancelled_drop = Arc::new(AtomicBool::new(false));
        let cancelled_started = Arc::new(Notify::new());
        let cancelled = reader(
            vec![ScriptStep {
                command: GitCommand::BareRepository,
                result: ScriptResult::Pending {
                    dropped: Arc::clone(&cancelled_drop),
                    started: Arc::clone(&cancelled_started),
                },
            }],
            Duration::from_secs(1),
        );
        let (sender, receiver) = oneshot::channel::<()>();
        let cancelled_root = root.clone();
        let read = tokio::spawn(async move {
            cancelled
                .read_with_cancellation(cancelled_root, async move {
                    let _ = receiver.await;
                })
                .await
        });
        cancelled_started.notified().await;
        sender.send(()).expect("cancellation must be sent");
        let error = read
            .await
            .expect("cancellation read task must join")
            .expect_err("cancellation must stop the read");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::Cancelled);
        assert!(cancelled_drop.load(Ordering::SeqCst));

        let timeout_drop = Arc::new(AtomicBool::new(false));
        let timed = reader(
            vec![ScriptStep {
                command: GitCommand::BareRepository,
                result: ScriptResult::Pending {
                    dropped: Arc::clone(&timeout_drop),
                    started: Arc::new(Notify::new()),
                },
            }],
            Duration::from_millis(10),
        );
        let error = timed
            .read(&root)
            .await
            .expect_err("deadline must stop the read");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::TimedOut);
        assert!(timeout_drop.load(Ordering::SeqCst));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn production_child_boundary_reaps_on_future_drop_and_within_deadline() {
        let (_second_temp, second_root) = self::root();
        let (reader, started, reaped) = production_boundary_reader(Duration::from_millis(400));
        let read = tokio::spawn(async move { reader.read(second_root).await });
        timeout(Duration::from_secs(2), started.notified())
            .await
            .expect("production-boundary child must start");
        read.abort();
        assert!(
            read.await
                .expect_err("aborted read task must be cancelled")
                .is_cancelled()
        );
        assert!(reaped.load(Ordering::SeqCst));

        let (_temp, root) = root();
        let complete_timeout = Duration::from_secs(2);
        let (reader, started, reaped) = production_boundary_reader(complete_timeout);
        let read = tokio::spawn(async move { reader.read(root).await });
        timeout(Duration::from_secs(2), started.notified())
            .await
            .expect("deadline child must start");
        let error = timeout(complete_timeout, read)
            .await
            .expect("complete deadline and cleanup must remain bounded")
            .expect("deadline read task must join")
            .expect_err("deadline must stop the production-boundary child");
        assert_eq!(error.kind(), GitRepositoryReadErrorKind::TimedOut);
        assert!(reaped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn bounded_reader_accepts_exact_and_rejects_one_over() {
        let (mut writer, reader) = tokio::io::duplex(64);
        let write = tokio::spawn(async move {
            writer
                .write_all(&[b'x'; 32])
                .await
                .expect("exact bytes must write");
        });
        assert_eq!(
            read_bounded(reader, 32)
                .await
                .expect("exact bound must pass")
                .len(),
            32
        );
        write.await.expect("writer must finish");

        let (mut writer, reader) = tokio::io::duplex(64);
        let write = tokio::spawn(async move {
            writer
                .write_all(&[b'x'; 33])
                .await
                .expect("one-over bytes must write");
        });
        let error = read_bounded(reader, 32)
            .await
            .expect_err("one-over bound must fail");
        assert_eq!(error, GitRepositoryReadErrorKind::OutputLimitExceeded);
        write.await.expect("writer must finish");
        assert_eq!(STDOUT_LIMIT, 16 * 1024 * 1024);
    }
}
