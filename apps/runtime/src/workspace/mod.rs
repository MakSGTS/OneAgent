//! Immutable Workspace semantic snapshots and deterministic initial builds.

mod cache;
mod change;
mod git;
mod graph_query;
mod repository_change;

pub use cache::{WorkspaceCacheLoadOutcome, WorkspaceCacheWriteOutcome};

pub use graph_query::{
    GraphQueryConfiguration, GraphQueryConfigurationList, GraphQueryDirection, GraphQueryEdgeKind,
    GraphQueryError, GraphQueryErrorKind, GraphQueryLimit, GraphQueryMaxDepth,
    GraphQueryMetadataKind, GraphQueryNode, GraphQueryNodeKind, GraphQueryNodeResult,
    GraphQueryRelation, GraphQueryRelationResult, GraphQueryService, GraphQueryTraversalNode,
    GraphQueryTraversalResult, GraphQueryWorkspaceFormat,
};

pub use git::{GitRepositoryReadError, GitRepositoryReadErrorKind, GitRepositoryReader};

pub use repository_change::{
    GitChangeCompleteness, GitChangeSet, GitChangeSetError, GitChangeSetErrorKind, GitCommitId,
    GitCommitIdError, GitCommitIdErrorKind, GitCurrentEndpoint, MAX_REPOSITORY_CHANGE_PATH_BYTES,
    MAX_REPOSITORY_CHANGES, RepositoryChange, RepositoryChangeError, RepositoryChangeErrorKind,
    RepositoryChangeKind, RepositoryChangePath, RepositoryChangePathError,
    RepositoryChangePathErrorKind,
};

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use oneagent_analysis::change_impact::{
    ChangeImpactCancellationSignal, ChangeImpactConfiguration, ChangeImpactError,
    ChangeImpactEvaluator, ChangeImpactPublicationId, ChangeImpactReport,
};
use oneagent_analysis::diagnostics::{DiagnosticEngine, DiagnosticPolicy, DiagnosticReport};
use oneagent_analysis::rules::{
    NeverCancelled as NeverRuleCancelled, Rule, RuleCancellationSignal, RuleConfiguration,
    RuleContext, RuleEngine, RuleExecutionReport, RulePlan, RuleRegistry,
};
use oneagent_common::{EntityId, EntityName};
use oneagent_designer_xml::{
    DesignerXmlBuildScope, DesignerXmlSemanticGraphBuilder,
    FileSystemDesignerXmlSemanticGraphBuilder,
};
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    NodeKind, SemanticDiagnostic, SemanticGraph, SemanticGraphReport,
    SemanticGraphValidationResult, SemanticGraphValidator, SemanticReferenceRequestLedger,
    SemanticReferenceStatistics,
};
use oneagent_metadata::MetadataKind;
use oneagent_workspace::{DiscoveredConfiguration, WorkspaceDetector, WorkspaceFormat};
use oneagent_workspace_fs::FileSystemWorkspaceDetector;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinError;

use crate::{BoxError, RuntimeService, ServiceContext, ServiceStartFuture, ServiceTask};
use cache::{WorkspaceCacheStorage, WorkspaceCacheStore};
use change::{
    RunningWorkspaceChangeSource, WorkspaceChangeOutcome, WorkspaceChangeSource,
    WorkspaceChangeSourceError, WorkspaceFileState,
};

impl ChangeImpactCancellationSignal for crate::Cancellation {
    fn is_cancelled(&self) -> bool {
        self.is_requested()
    }
}

/// Stable source-neutral category for an initial Workspace build failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceBuildErrorKind {
    /// The configured root could not be observed completely.
    ObservationFailed,
    /// The configured root could not be discovered.
    DiscoveryFailed,
    /// A detector returned a source format outside the accepted first slice.
    UnsupportedFormat,
    /// A production semantic builder failed.
    SemanticBuildFailed,
    /// A returned graph or build result failed validation.
    GraphValidationFailed,
    /// A graph did not contain exactly one Configuration node.
    InvalidConfigurationCardinality,
    /// Two roots produced the same canonical Configuration identity.
    DuplicateConfigurationIdentity,
    /// The Runtime-owned blocking initial-build task failed to join.
    BuildTaskFailed,
}

/// Source-neutral error produced while constructing a complete Workspace snapshot.
#[derive(Debug)]
pub enum WorkspaceBuildError {
    /// Filesystem observation failed for the configured root.
    Observation {
        /// Configured Workspace root.
        root_path: PathBuf,
        /// Original observation error.
        source: BoxError,
    },
    /// Discovery failed for the configured root.
    Discovery {
        /// Configured Workspace root.
        root_path: PathBuf,
        /// Original detector error.
        source: BoxError,
    },
    /// A discovered source format has no accepted production builder.
    UnsupportedFormat {
        /// Discovered project root.
        root_path: PathBuf,
        /// Unsupported source format.
        format: WorkspaceFormat,
    },
    /// A format-specific semantic build failed.
    SemanticBuild {
        /// Discovered project root.
        root_path: PathBuf,
        /// Source format selected for the build.
        format: WorkspaceFormat,
        /// Original adapter or source-independent evidence error.
        source: BoxError,
    },
    /// A returned graph or build result failed semantic validation.
    GraphValidation {
        /// Discovered project root.
        root_path: PathBuf,
        /// Source format selected for the build.
        format: WorkspaceFormat,
        /// Complete deterministic validation result.
        validation: Box<SemanticGraphValidationResult>,
    },
    /// A returned graph did not contain exactly one Configuration node.
    InvalidConfigurationCardinality {
        /// Discovered project root.
        root_path: PathBuf,
        /// Source format selected for the build.
        format: WorkspaceFormat,
        /// Number of Configuration nodes found.
        actual: usize,
    },
    /// Two distinct discovered roots produced the same Configuration identity.
    DuplicateConfigurationIdentity {
        /// Duplicate canonical Configuration identity.
        configuration_id: EntityId,
        /// Root retained from the first deterministic observation.
        first_root: PathBuf,
        /// Later root that produced the same identity.
        duplicate_root: PathBuf,
    },
    /// The Runtime-owned blocking initial-build task panicked or was cancelled.
    BuildTask {
        /// Configured Workspace root.
        root_path: PathBuf,
        /// Original Tokio task join failure.
        source: JoinError,
    },
}

impl WorkspaceBuildError {
    /// Returns the stable source-neutral failure category.
    #[must_use]
    pub const fn kind(&self) -> WorkspaceBuildErrorKind {
        match self {
            Self::Observation { .. } => WorkspaceBuildErrorKind::ObservationFailed,
            Self::Discovery { .. } => WorkspaceBuildErrorKind::DiscoveryFailed,
            Self::UnsupportedFormat { .. } => WorkspaceBuildErrorKind::UnsupportedFormat,
            Self::SemanticBuild { .. } => WorkspaceBuildErrorKind::SemanticBuildFailed,
            Self::GraphValidation { .. } => WorkspaceBuildErrorKind::GraphValidationFailed,
            Self::InvalidConfigurationCardinality { .. } => {
                WorkspaceBuildErrorKind::InvalidConfigurationCardinality
            }
            Self::DuplicateConfigurationIdentity { .. } => {
                WorkspaceBuildErrorKind::DuplicateConfigurationIdentity
            }
            Self::BuildTask { .. } => WorkspaceBuildErrorKind::BuildTaskFailed,
        }
    }

    /// Returns the most specific project or configured root associated with the failure.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        match self {
            Self::Observation { root_path, .. }
            | Self::Discovery { root_path, .. }
            | Self::UnsupportedFormat { root_path, .. }
            | Self::SemanticBuild { root_path, .. }
            | Self::GraphValidation { root_path, .. }
            | Self::InvalidConfigurationCardinality { root_path, .. }
            | Self::BuildTask { root_path, .. } => root_path,
            Self::DuplicateConfigurationIdentity { duplicate_root, .. } => duplicate_root,
        }
    }

    /// Returns the source format associated with the failure, when applicable.
    #[must_use]
    pub const fn format(&self) -> Option<WorkspaceFormat> {
        match self {
            Self::UnsupportedFormat { format, .. }
            | Self::SemanticBuild { format, .. }
            | Self::GraphValidation { format, .. }
            | Self::InvalidConfigurationCardinality { format, .. } => Some(*format),
            Self::Observation { .. }
            | Self::Discovery { .. }
            | Self::DuplicateConfigurationIdentity { .. }
            | Self::BuildTask { .. } => None,
        }
    }

    /// Returns the duplicate Configuration identity, when applicable.
    #[must_use]
    pub const fn configuration_id(&self) -> Option<&EntityId> {
        match self {
            Self::DuplicateConfigurationIdentity {
                configuration_id, ..
            } => Some(configuration_id),
            _ => None,
        }
    }

    /// Returns the deterministic validation evidence, when applicable.
    #[must_use]
    pub fn validation(&self) -> Option<&SemanticGraphValidationResult> {
        match self {
            Self::GraphValidation { validation, .. } => Some(validation.as_ref()),
            _ => None,
        }
    }
}

impl Display for WorkspaceBuildError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observation { root_path, source } => write!(
                formatter,
                "Workspace observation failed for {}: {source}",
                root_path.display()
            ),
            Self::Discovery { root_path, source } => write!(
                formatter,
                "Workspace discovery failed for {}: {source}",
                root_path.display()
            ),
            Self::UnsupportedFormat { root_path, format } => write!(
                formatter,
                "Workspace format {format:?} is unsupported at {}",
                root_path.display()
            ),
            Self::SemanticBuild {
                root_path,
                format,
                source,
            } => write!(
                formatter,
                "Workspace semantic build failed for {format:?} at {}: {source}",
                root_path.display()
            ),
            Self::GraphValidation {
                root_path,
                format,
                validation,
            } => write!(
                formatter,
                "Workspace graph validation failed for {format:?} at {} with {} issue(s)",
                root_path.display(),
                validation.summary().total()
            ),
            Self::InvalidConfigurationCardinality {
                root_path,
                format,
                actual,
            } => write!(
                formatter,
                "Workspace graph for {format:?} at {} contains {actual} Configuration node(s)",
                root_path.display()
            ),
            Self::DuplicateConfigurationIdentity {
                configuration_id,
                first_root,
                duplicate_root,
            } => write!(
                formatter,
                "Workspace Configuration {configuration_id} is duplicated at {} and {}",
                first_root.display(),
                duplicate_root.display()
            ),
            Self::BuildTask { root_path, source } => write!(
                formatter,
                "Workspace blocking build task failed for {}: {source}",
                root_path.display()
            ),
        }
    }
}

impl Error for WorkspaceBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Observation { source, .. } => Some(source.as_ref()),
            Self::Discovery { source, .. } | Self::SemanticBuild { source, .. } => {
                Some(source.as_ref())
            }
            Self::BuildTask { source, .. } => Some(source),
            Self::UnsupportedFormat { .. }
            | Self::GraphValidation { .. }
            | Self::InvalidConfigurationCardinality { .. }
            | Self::DuplicateConfigurationIdentity { .. } => None,
        }
    }
}

/// Stable transport-neutral phase of Runtime Workspace update orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceUpdatePhase {
    /// Initial observation and complete build have not finished.
    Starting,
    /// The latest complete build is published and filesystem changes are observed.
    Watching,
    /// A complete replacement build is running.
    Rebuilding,
    /// The latest observation or rebuild attempt failed recoverably.
    Failed,
    /// Runtime-owned observation and publication have stopped.
    Stopped,
}

/// Stable source-neutral category for a recoverable Workspace update failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceUpdateFailureKind {
    /// A complete filesystem observation failed.
    Observation,
    /// Workspace discovery failed.
    Discovery,
    /// Discovery returned an unsupported format.
    UnsupportedFormat,
    /// A production semantic build failed.
    SemanticBuild,
    /// A graph failed validation.
    GraphValidation,
    /// A graph had invalid Configuration cardinality.
    InvalidConfigurationCardinality,
    /// Two roots produced the same Configuration identity.
    DuplicateConfigurationIdentity,
    /// A blocking complete-build task failed to join.
    BuildTask,
}

impl From<WorkspaceBuildErrorKind> for WorkspaceUpdateFailureKind {
    fn from(kind: WorkspaceBuildErrorKind) -> Self {
        match kind {
            WorkspaceBuildErrorKind::ObservationFailed => Self::Observation,
            WorkspaceBuildErrorKind::DiscoveryFailed => Self::Discovery,
            WorkspaceBuildErrorKind::UnsupportedFormat => Self::UnsupportedFormat,
            WorkspaceBuildErrorKind::SemanticBuildFailed => Self::SemanticBuild,
            WorkspaceBuildErrorKind::GraphValidationFailed => Self::GraphValidation,
            WorkspaceBuildErrorKind::InvalidConfigurationCardinality => {
                Self::InvalidConfigurationCardinality
            }
            WorkspaceBuildErrorKind::DuplicateConfigurationIdentity => {
                Self::DuplicateConfigurationIdentity
            }
            WorkspaceBuildErrorKind::BuildTaskFailed => Self::BuildTask,
        }
    }
}

/// Immutable observable state of Runtime Workspace update orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceUpdateStatus {
    attempt: u64,
    published: u64,
    phase: WorkspaceUpdatePhase,
    failure: Option<WorkspaceUpdateFailureKind>,
}

impl WorkspaceUpdateStatus {
    const fn starting() -> Self {
        Self {
            attempt: 0,
            published: 0,
            phase: WorkspaceUpdatePhase::Starting,
            failure: None,
        }
    }

    /// Returns the number of started initial or replacement build attempts.
    #[must_use]
    pub const fn attempt(self) -> u64 {
        self.attempt
    }

    /// Returns the number of complete snapshots published by this service.
    #[must_use]
    pub const fn published(self) -> u64 {
        self.published
    }

    /// Returns the current update phase.
    #[must_use]
    pub const fn phase(self) -> WorkspaceUpdatePhase {
        self.phase
    }

    /// Returns the latest recoverable failure kind only while the phase is `Failed`.
    #[must_use]
    pub const fn failure(self) -> Option<WorkspaceUpdateFailureKind> {
        self.failure
    }
}

/// Cloneable transport-neutral observation of Workspace update status.
#[derive(Debug, Clone)]
pub struct WorkspaceUpdateObserver {
    status: watch::Receiver<WorkspaceUpdateStatus>,
}

impl WorkspaceUpdateObserver {
    /// Returns the current immutable update status.
    #[must_use]
    pub fn status(&self) -> WorkspaceUpdateStatus {
        *self.status.borrow()
    }

    /// Creates an owned subscription to future update status changes.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<WorkspaceUpdateStatus> {
        self.status.clone()
    }
}

/// Immutable transport-neutral state of persistent Workspace cache activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceCacheStatus {
    load: WorkspaceCacheLoadOutcome,
    write: WorkspaceCacheWriteOutcome,
}

impl WorkspaceCacheStatus {
    const fn starting() -> Self {
        Self {
            load: WorkspaceCacheLoadOutcome::NotAttempted,
            write: WorkspaceCacheWriteOutcome::NotAttempted,
        }
    }

    /// Returns the latest cache load outcome.
    #[must_use]
    pub const fn load(self) -> WorkspaceCacheLoadOutcome {
        self.load
    }

    /// Returns the latest cache write outcome.
    #[must_use]
    pub const fn write(self) -> WorkspaceCacheWriteOutcome {
        self.write
    }
}

/// Cloneable transport-neutral observation of persistent Workspace cache status.
#[derive(Debug, Clone)]
pub struct WorkspaceCacheObserver {
    status: watch::Receiver<WorkspaceCacheStatus>,
}

impl WorkspaceCacheObserver {
    /// Returns the current immutable cache status.
    #[must_use]
    pub fn status(&self) -> WorkspaceCacheStatus {
        *self.status.borrow()
    }

    /// Creates an owned subscription to future cache status changes.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<WorkspaceCacheStatus> {
        self.status.clone()
    }
}

/// Cloneable transport-neutral observation of the published Workspace snapshot.
#[derive(Debug, Clone)]
pub struct WorkspaceSnapshotObserver {
    snapshot: watch::Receiver<Option<Arc<WorkspaceSnapshot>>>,
}

/// Closed outcome of one non-blocking Workspace change-input submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceChangeSubmissionOutcome {
    /// The supplied change set was empty and queued no work.
    IgnoredEmpty,
    /// One non-empty complete-rebuild request entered the bounded slot.
    Accepted,
    /// The one-slot input already contains a pending request.
    Backpressure,
    /// The Workspace service no longer owns the input receiver.
    Closed,
}

/// Cloneable pre-registration input for explicit complete Workspace rebuilds.
#[derive(Clone)]
pub struct WorkspaceChangeInputHandle {
    sender: mpsc::Sender<WorkspaceChangeRequest>,
}

impl WorkspaceChangeInputHandle {
    /// Submits one validated Git-derived change set without blocking.
    ///
    /// Only normalized path and status evidence enters the private request;
    /// repository endpoints and completeness are discarded before submission.
    #[must_use]
    pub fn submit(&self, change_set: GitChangeSet) -> WorkspaceChangeSubmissionOutcome {
        if change_set.is_empty() {
            return WorkspaceChangeSubmissionOutcome::IgnoredEmpty;
        }
        match self
            .sender
            .try_send(WorkspaceChangeRequest::from(change_set))
        {
            Ok(()) => WorkspaceChangeSubmissionOutcome::Accepted,
            Err(mpsc::error::TrySendError::Full(_)) => {
                WorkspaceChangeSubmissionOutcome::Backpressure
            }
            Err(mpsc::error::TrySendError::Closed(_)) => WorkspaceChangeSubmissionOutcome::Closed,
        }
    }
}

impl std::fmt::Debug for WorkspaceChangeInputHandle {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkspaceChangeInputHandle")
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct WorkspaceChangeRequest {
    _changes: Box<[WorkspaceChangeRecord]>,
}

impl From<GitChangeSet> for WorkspaceChangeRequest {
    fn from(change_set: GitChangeSet) -> Self {
        let changes = change_set
            .changes()
            .iter()
            .map(|change| WorkspaceChangeRecord {
                _kind: change.kind(),
                _previous_path: change
                    .previous_path()
                    .map(|path| Box::<str>::from(path.as_str())),
                _current_path: change
                    .current_path()
                    .map(|path| Box::<str>::from(path.as_str())),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self { _changes: changes }
    }
}

#[derive(Debug)]
struct WorkspaceChangeRecord {
    _kind: RepositoryChangeKind,
    _previous_path: Option<Box<str>>,
    _current_path: Option<Box<str>>,
}

impl WorkspaceSnapshotObserver {
    /// Returns the currently published complete snapshot, when present.
    #[must_use]
    pub fn snapshot(&self) -> Option<Arc<WorkspaceSnapshot>> {
        self.snapshot.borrow().clone()
    }

    /// Creates an owned subscription to future snapshot changes.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<Option<Arc<WorkspaceSnapshot>>> {
        self.snapshot.clone()
    }
}

enum WorkspaceCacheBackend {
    Production,
    #[cfg(test)]
    Controlled(Arc<dyn WorkspaceCacheStorage>),
}

impl WorkspaceCacheBackend {
    fn open(self, workspace_root: PathBuf) -> Arc<dyn WorkspaceCacheStorage> {
        match self {
            Self::Production => Arc::new(WorkspaceCacheStore::new(workspace_root)),
            #[cfg(test)]
            Self::Controlled(storage) => storage,
        }
    }
}

impl std::fmt::Debug for WorkspaceCacheBackend {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Production => formatter.write_str("Production"),
            #[cfg(test)]
            Self::Controlled(_) => formatter.write_str("Controlled"),
        }
    }
}

/// Runtime-owned service for one complete initial Workspace build.
#[derive(Debug)]
pub struct WorkspaceService<D = FileSystemWorkspaceDetector> {
    builder: WorkspaceSnapshotBuilder<D>,
    cache_backend: WorkspaceCacheBackend,
    change_input: WorkspaceChangeInputHandle,
    change_requests: mpsc::Receiver<WorkspaceChangeRequest>,
    cache_status: watch::Sender<WorkspaceCacheStatus>,
    snapshot: watch::Sender<Option<Arc<WorkspaceSnapshot>>>,
    updates: watch::Sender<WorkspaceUpdateStatus>,
    #[cfg(test)]
    controlled_change_ticks: Option<tokio::sync::mpsc::Receiver<tokio::sync::oneshot::Sender<()>>>,
}

impl WorkspaceService<FileSystemWorkspaceDetector> {
    /// Creates the production Workspace service with no published snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::with_builder(WorkspaceSnapshotBuilder::new())
    }
}

impl<D> WorkspaceService<D> {
    fn with_builder(builder: WorkspaceSnapshotBuilder<D>) -> Self {
        let (cache_status, _receiver) = watch::channel(WorkspaceCacheStatus::starting());
        let (snapshot, _receiver) = watch::channel(None);
        let (updates, _receiver) = watch::channel(WorkspaceUpdateStatus::starting());
        let (change_sender, change_requests) = mpsc::channel(1);
        Self {
            builder,
            cache_backend: WorkspaceCacheBackend::Production,
            change_input: WorkspaceChangeInputHandle {
                sender: change_sender,
            },
            change_requests,
            cache_status,
            snapshot,
            updates,
            #[cfg(test)]
            controlled_change_ticks: None,
        }
    }

    /// Creates a cloneable cache-status observer before this service is registered.
    #[must_use]
    pub fn cache_observer(&self) -> WorkspaceCacheObserver {
        WorkspaceCacheObserver {
            status: self.cache_status.subscribe(),
        }
    }

    /// Creates a cloneable observer before this service is registered.
    #[must_use]
    pub fn snapshot_observer(&self) -> WorkspaceSnapshotObserver {
        WorkspaceSnapshotObserver {
            snapshot: self.snapshot.subscribe(),
        }
    }

    /// Creates a cloneable update-status observer before this service is registered.
    #[must_use]
    pub fn update_observer(&self) -> WorkspaceUpdateObserver {
        WorkspaceUpdateObserver {
            status: self.updates.subscribe(),
        }
    }

    /// Creates the cloneable explicit change-input handle before registration.
    #[must_use]
    pub fn change_input_handle(&self) -> WorkspaceChangeInputHandle {
        self.change_input.clone()
    }

    #[cfg(test)]
    fn with_controlled_change_ticks(
        mut self,
        ticks: tokio::sync::mpsc::Receiver<tokio::sync::oneshot::Sender<()>>,
    ) -> Self {
        self.controlled_change_ticks = Some(ticks);
        self
    }

    #[cfg(test)]
    fn with_cache_storage(mut self, storage: Arc<dyn WorkspaceCacheStorage>) -> Self {
        self.cache_backend = WorkspaceCacheBackend::Controlled(storage);
        self
    }
}

impl Default for WorkspaceService<FileSystemWorkspaceDetector> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D> RuntimeService for WorkspaceService<D>
where
    D: WorkspaceDetector + Clone + Send + 'static,
{
    fn start(self: Box<Self>, context: ServiceContext) -> ServiceStartFuture {
        Box::pin(async move {
            let root_path = context
                .state()
                .configuration()
                .workspace_root()
                .to_path_buf();
            let WorkspaceService {
                builder,
                cache_backend,
                change_input,
                change_requests,
                cache_status,
                snapshot,
                updates,
                #[cfg(test)]
                controlled_change_ticks,
            } = *self;
            drop(change_input);
            let cache = cache_backend.open(root_path.clone());
            updates.send_replace(WorkspaceUpdateStatus {
                attempt: 1,
                published: 0,
                phase: WorkspaceUpdatePhase::Starting,
                failure: None,
            });

            let initial_root = root_path.clone();
            let initial_builder = builder.clone();
            let initial_cache = Arc::clone(&cache);
            let initial_cache_status = cache_status.clone();
            let error_root = root_path.clone();
            let initial = tokio::task::spawn_blocking(move || {
                initialize_workspace(
                    &initial_builder,
                    &initial_root,
                    initial_cache.as_ref(),
                    &initial_cache_status,
                )
            })
            .await
            .map_err(|source| WorkspaceBuildError::BuildTask {
                root_path: error_root,
                source,
            })?
            .map_err(|error| Box::new(error) as BoxError)?;
            snapshot.send_replace(Some(Arc::new(initial.snapshot)));
            updates.send_replace(WorkspaceUpdateStatus {
                attempt: 1,
                published: 1,
                phase: WorkspaceUpdatePhase::Watching,
                failure: None,
            });

            #[cfg(test)]
            let source = if let Some(ticks) = controlled_change_ticks {
                WorkspaceChangeSource::with_controlled_ticks(
                    root_path.clone(),
                    initial.source_state.clone(),
                    ticks,
                )
            } else {
                WorkspaceChangeSource::new(root_path.clone(), initial.source_state.clone())
            };
            #[cfg(not(test))]
            let source =
                WorkspaceChangeSource::new(root_path.clone(), initial.source_state.clone());

            let cancellation = context.cancellation();
            let source = source
                .with_initial_change(initial.follow_up_required)
                .start(cancellation.clone());

            let task: ServiceTask = Box::pin(async move {
                run_workspace_updates(
                    builder,
                    root_path,
                    cache,
                    cache_status,
                    snapshot,
                    updates,
                    cancellation,
                    source,
                    change_requests,
                )
                .await
                .map_err(|error| Box::new(error) as BoxError)
            });
            Ok(task)
        })
    }
}

struct WorkspaceInitialization {
    snapshot: WorkspaceSnapshot,
    source_state: WorkspaceFileState,
    follow_up_required: bool,
}

fn initialize_workspace<D>(
    builder: &WorkspaceSnapshotBuilder<D>,
    root_path: &Path,
    cache: &dyn WorkspaceCacheStorage,
    cache_status: &watch::Sender<WorkspaceCacheStatus>,
) -> Result<WorkspaceInitialization, WorkspaceBuildError>
where
    D: WorkspaceDetector,
{
    let initial_state = observe_workspace(root_path)?;
    let loaded = cache.load(&initial_state);
    publish_cache_load(cache_status, loaded.outcome());
    let accepted_state = observe_workspace(root_path)?;

    if initial_state == accepted_state
        && loaded.outcome() == WorkspaceCacheLoadOutcome::Hit
        && let Some(snapshot) = loaded.into_snapshot()
    {
        return Ok(WorkspaceInitialization {
            snapshot,
            source_state: accepted_state,
            follow_up_required: false,
        });
    }

    let snapshot = builder.build(root_path)?;
    let final_state = observe_workspace(root_path)?;
    let write = if accepted_state == final_state {
        cache.write(&final_state, &snapshot)
    } else {
        WorkspaceCacheWriteOutcome::SkippedUnstableSource
    };
    publish_cache_write(cache_status, write);
    Ok(WorkspaceInitialization {
        snapshot,
        follow_up_required: accepted_state != final_state,
        source_state: final_state,
    })
}

fn publish_cache_load(
    cache_status: &watch::Sender<WorkspaceCacheStatus>,
    load: WorkspaceCacheLoadOutcome,
) {
    cache_status.send_modify(|status| status.load = load);
}

fn publish_cache_write(
    cache_status: &watch::Sender<WorkspaceCacheStatus>,
    write: WorkspaceCacheWriteOutcome,
) {
    cache_status.send_modify(|status| status.write = write);
}

fn observe_workspace(root_path: &Path) -> Result<WorkspaceFileState, WorkspaceBuildError> {
    WorkspaceFileState::scan(root_path).map_err(|source| WorkspaceBuildError::Observation {
        root_path: root_path.to_path_buf(),
        source: Box::new(source),
    })
}

#[derive(Debug)]
enum WorkspaceUpdateRuntimeError {
    ChangeSourceStopped,
    ChangeSource(WorkspaceChangeSourceError),
    ChangeSourceTask(JoinError),
    StatusCounterOverflow,
    SnapshotUnavailable,
}

impl Display for WorkspaceUpdateRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChangeSourceStopped => {
                formatter.write_str("Workspace change source stopped before cancellation")
            }
            Self::ChangeSource(error) => {
                write!(formatter, "Workspace change source failed: {error}")
            }
            Self::ChangeSourceTask(error) => {
                write!(formatter, "Workspace change source task failed: {error}")
            }
            Self::StatusCounterOverflow => {
                formatter.write_str("Workspace update status counter overflowed")
            }
            Self::SnapshotUnavailable => {
                formatter.write_str("Workspace update has no current published snapshot")
            }
        }
    }
}

impl Error for WorkspaceUpdateRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ChangeSource(error) => Some(error),
            Self::ChangeSourceTask(error) => Some(error),
            Self::ChangeSourceStopped | Self::StatusCounterOverflow | Self::SnapshotUnavailable => {
                None
            }
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the select loop keeps source, build, cancellation, and publication ownership together"
)]
async fn run_workspace_updates<D>(
    builder: WorkspaceSnapshotBuilder<D>,
    root_path: PathBuf,
    cache: Arc<dyn WorkspaceCacheStorage>,
    cache_status: watch::Sender<WorkspaceCacheStatus>,
    snapshot: watch::Sender<Option<Arc<WorkspaceSnapshot>>>,
    updates: watch::Sender<WorkspaceUpdateStatus>,
    mut cancellation: crate::Cancellation,
    source: RunningWorkspaceChangeSource,
    mut change_requests: mpsc::Receiver<WorkspaceChangeRequest>,
) -> Result<(), WorkspaceUpdateRuntimeError>
where
    D: WorkspaceDetector + Clone + Send + 'static,
{
    let (mut observations, mut source_task) = source.into_parts();
    let mut processed_revision = 0_u64;
    let mut status = *updates.borrow();
    let mut explicit_rebuild_pending = false;
    let mut change_input_open = true;

    loop {
        let mut rebuild_requested = explicit_rebuild_pending;
        explicit_rebuild_pending = false;
        let observation = *observations.borrow_and_update();
        if !rebuild_requested && observation.revision() > processed_revision {
            processed_revision = observation.revision();
            match observation.outcome() {
                Some(WorkspaceChangeOutcome::Changed) => {
                    rebuild_requested = true;
                }
                Some(WorkspaceChangeOutcome::ObservationFailed(_)) => {
                    status.phase = WorkspaceUpdatePhase::Failed;
                    status.failure = Some(WorkspaceUpdateFailureKind::Observation);
                    updates.send_replace(status);
                    continue;
                }
                None => {}
            }
        }

        if rebuild_requested {
            status.attempt = status
                .attempt
                .checked_add(1)
                .ok_or(WorkspaceUpdateRuntimeError::StatusCounterOverflow)?;
            status.phase = WorkspaceUpdatePhase::Rebuilding;
            status.failure = None;
            updates.send_replace(status);

            let build_root = root_path.clone();
            let build_builder = builder.clone();
            let build_cache = Arc::clone(&cache);
            let previous = snapshot
                .borrow()
                .clone()
                .ok_or(WorkspaceUpdateRuntimeError::SnapshotUnavailable)?;
            let build_previous = Arc::clone(&previous);
            let build_cancellation = cancellation.clone();
            let mut build = tokio::task::spawn_blocking(move || {
                rebuild_workspace(
                    &build_builder,
                    &build_root,
                    build_cache.as_ref(),
                    &build_previous,
                    &build_cancellation,
                )
            });
            let build_result = tokio::select! {
                biased;
                () = cancellation.cancelled() => {
                    change_requests.close();
                    let _ = (&mut build).await;
                    let source_result = source_task.await;
                    return finish_workspace_updates(
                        &snapshot,
                        &updates,
                        source_result,
                        true,
                    );
                }
                source_result = &mut source_task => {
                    change_requests.close();
                    let _ = (&mut build).await;
                    return finish_workspace_updates(
                        &snapshot,
                        &updates,
                        source_result,
                        false,
                    );
                }
                result = &mut build => result,
            };

            match build_result {
                Ok(Ok(rebuilt)) => {
                    let predecessor_is_current = snapshot
                        .borrow()
                        .as_ref()
                        .is_some_and(|current| Arc::ptr_eq(current, &previous));
                    let next_published = status
                        .published
                        .checked_add(1)
                        .ok_or(WorkspaceUpdateRuntimeError::StatusCounterOverflow)?;
                    if predecessor_is_current
                        && rebuilt.snapshot.publication_id().get() == next_published
                    {
                        publish_cache_write(&cache_status, rebuilt.write);
                        snapshot.send_replace(Some(Arc::new(rebuilt.snapshot)));
                        status.published = next_published;
                        status.phase = WorkspaceUpdatePhase::Watching;
                        status.failure = None;
                    } else {
                        status.phase = WorkspaceUpdatePhase::Failed;
                        status.failure = Some(WorkspaceUpdateFailureKind::SemanticBuild);
                    }
                }
                Ok(Err(error)) => {
                    status.phase = WorkspaceUpdatePhase::Failed;
                    status.failure = Some(error.failure_kind());
                }
                Err(_) => {
                    status.phase = WorkspaceUpdatePhase::Failed;
                    status.failure = Some(WorkspaceUpdateFailureKind::BuildTask);
                }
            }
            updates.send_replace(status);
            continue;
        }

        tokio::select! {
            biased;
            () = cancellation.cancelled() => {
                change_requests.close();
                let source_result = source_task.await;
                return finish_workspace_updates(&snapshot, &updates, source_result, true);
            }
            source_result = &mut source_task => {
                change_requests.close();
                return finish_workspace_updates(
                    &snapshot,
                    &updates,
                    source_result,
                    false,
                );
            }
            changed = observations.changed() => {
                if changed.is_err() {
                    change_requests.close();
                    let source_result = source_task.await;
                    return finish_workspace_updates(
                        &snapshot,
                        &updates,
                        source_result,
                        false,
                    );
                }
            }
            request = change_requests.recv(), if change_input_open => {
                if request.is_some() {
                    explicit_rebuild_pending = true;
                } else {
                    change_input_open = false;
                }
            }
        }
    }
}

#[derive(Debug)]
struct WorkspaceRebuild {
    snapshot: WorkspaceSnapshot,
    write: WorkspaceCacheWriteOutcome,
}

#[derive(Debug)]
enum WorkspaceRebuildError {
    Build(WorkspaceBuildError),
    ChangeImpact,
}

impl WorkspaceRebuildError {
    fn failure_kind(&self) -> WorkspaceUpdateFailureKind {
        match self {
            Self::Build(error) => error.kind().into(),
            Self::ChangeImpact => WorkspaceUpdateFailureKind::SemanticBuild,
        }
    }
}

impl From<WorkspaceBuildError> for WorkspaceRebuildError {
    fn from(error: WorkspaceBuildError) -> Self {
        Self::Build(error)
    }
}

impl From<ChangeImpactError> for WorkspaceRebuildError {
    fn from(_error: ChangeImpactError) -> Self {
        Self::ChangeImpact
    }
}

fn rebuild_workspace<D>(
    builder: &WorkspaceSnapshotBuilder<D>,
    root_path: &Path,
    cache: &dyn WorkspaceCacheStorage,
    previous: &WorkspaceSnapshot,
    cancellation: &dyn ChangeImpactCancellationSignal,
) -> Result<WorkspaceRebuild, WorkspaceRebuildError>
where
    D: WorkspaceDetector,
{
    let initial_state = observe_workspace(root_path)?;
    let mut snapshot = builder.build(root_path)?;
    let final_state = observe_workspace(root_path)?;
    compose_change_impact(previous, &mut snapshot, cancellation)?;
    let write = if initial_state == final_state {
        cache.write(&final_state, &snapshot)
    } else {
        WorkspaceCacheWriteOutcome::SkippedUnstableSource
    };
    Ok(WorkspaceRebuild { snapshot, write })
}

fn compose_change_impact(
    previous: &WorkspaceSnapshot,
    current: &mut WorkspaceSnapshot,
    cancellation: &dyn ChangeImpactCancellationSignal,
) -> Result<(), ChangeImpactError> {
    let previous_configurations = previous
        .configurations()
        .iter()
        .map(|configuration| {
            ChangeImpactConfiguration::new(configuration.configuration_id(), configuration.graph())
        })
        .collect::<Vec<_>>();
    let current_configurations = current
        .configurations()
        .iter()
        .map(|configuration| {
            ChangeImpactConfiguration::new(configuration.configuration_id(), configuration.graph())
        })
        .collect::<Vec<_>>();
    let report = ChangeImpactEvaluator.evaluate(
        previous.publication_id(),
        &previous_configurations,
        &current_configurations,
        cancellation,
    )?;
    current.change_impact = WorkspaceChangeImpact::Available(report);
    Ok(())
}

fn finish_workspace_updates(
    snapshot: &watch::Sender<Option<Arc<WorkspaceSnapshot>>>,
    updates: &watch::Sender<WorkspaceUpdateStatus>,
    source_result: Result<Result<(), WorkspaceChangeSourceError>, JoinError>,
    expected_stop: bool,
) -> Result<(), WorkspaceUpdateRuntimeError> {
    snapshot.send_replace(None);
    let status = *updates.borrow();
    updates.send_replace(WorkspaceUpdateStatus {
        phase: WorkspaceUpdatePhase::Stopped,
        failure: None,
        ..status
    });

    match source_result {
        Ok(Ok(())) if expected_stop => Ok(()),
        Ok(Ok(())) => Err(WorkspaceUpdateRuntimeError::ChangeSourceStopped),
        Ok(Err(error)) => Err(WorkspaceUpdateRuntimeError::ChangeSource(error)),
        Err(error) => Err(WorkspaceUpdateRuntimeError::ChangeSourceTask(error)),
    }
}

/// Immutable semantic state for one discovered Configuration.
#[derive(Debug, Clone)]
pub struct WorkspaceConfigurationSnapshot {
    root_path: PathBuf,
    format: WorkspaceFormat,
    configuration_id: EntityId,
    configuration_name: EntityName,
    graph: Arc<SemanticGraph>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    reference_requests: Arc<SemanticReferenceRequestLedger>,
    reference_statistics: SemanticReferenceStatistics,
    report: SemanticGraphReport,
    validation: Arc<SemanticGraphValidationResult>,
    rule_execution_report: Arc<RuleExecutionReport>,
    diagnostic_report: Arc<DiagnosticReport>,
}

impl WorkspaceConfigurationSnapshot {
    /// Returns the discovered project root.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Returns the detected source format.
    #[must_use]
    pub const fn format(&self) -> WorkspaceFormat {
        self.format
    }

    /// Returns the canonical Configuration identity.
    #[must_use]
    pub const fn configuration_id(&self) -> &EntityId {
        &self.configuration_id
    }

    /// Returns the exact Configuration name.
    #[must_use]
    pub const fn configuration_name(&self) -> &EntityName {
        &self.configuration_name
    }

    /// Returns the canonical immutable semantic graph.
    #[must_use]
    pub fn graph(&self) -> &SemanticGraph {
        &self.graph
    }

    /// Returns ordered recoverable semantic diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[SemanticDiagnostic] {
        &self.diagnostics
    }

    /// Returns canonical semantic reference requests in stable identity order.
    #[must_use]
    pub fn reference_requests(&self) -> &SemanticReferenceRequestLedger {
        &self.reference_requests
    }

    /// Returns total semantic reference outcome statistics.
    #[must_use]
    pub const fn reference_statistics(&self) -> SemanticReferenceStatistics {
        self.reference_statistics
    }

    /// Returns the deterministic report for the preserved build evidence.
    #[must_use]
    pub const fn report(&self) -> &SemanticGraphReport {
        &self.report
    }

    /// Returns the complete Graph validation result used by diagnostics.
    #[must_use]
    pub fn validation(&self) -> &SemanticGraphValidationResult {
        &self.validation
    }

    /// Returns the complete deterministic Rules Engine execution report.
    #[must_use]
    pub fn rule_execution_report(&self) -> &RuleExecutionReport {
        &self.rule_execution_report
    }

    /// Returns the complete normalized diagnostic report.
    #[must_use]
    pub fn diagnostic_report(&self) -> &DiagnosticReport {
        &self.diagnostic_report
    }
}

/// Change-impact availability embedded atomically in one Workspace publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceChangeImpact {
    /// This is the first complete publication of a fresh Workspace service.
    NoPreviousPublication {
        /// Process-local identity of the current publication.
        current_publication_id: ChangeImpactPublicationId,
    },
    /// Complete bounded impact from the immediately preceding publication.
    Available(ChangeImpactReport),
}

impl WorkspaceChangeImpact {
    /// Returns the process-local identity of the snapshot containing this value.
    #[must_use]
    pub const fn current_publication_id(&self) -> ChangeImpactPublicationId {
        match self {
            Self::NoPreviousPublication {
                current_publication_id,
            } => *current_publication_id,
            Self::Available(report) => report.current_publication_id(),
        }
    }

    /// Returns the complete adjacent-publication report when a predecessor exists.
    #[must_use]
    pub const fn report(&self) -> Option<&ChangeImpactReport> {
        match self {
            Self::NoPreviousPublication { .. } => None,
            Self::Available(report) => Some(report),
        }
    }
}

/// Complete immutable semantic state for one configured Workspace root.
#[derive(Debug, Clone)]
pub struct WorkspaceSnapshot {
    root_path: PathBuf,
    configurations: Vec<WorkspaceConfigurationSnapshot>,
    change_impact: WorkspaceChangeImpact,
}

impl WorkspaceSnapshot {
    fn initial(root_path: PathBuf, configurations: Vec<WorkspaceConfigurationSnapshot>) -> Self {
        Self {
            root_path,
            configurations,
            change_impact: WorkspaceChangeImpact::NoPreviousPublication {
                current_publication_id: ChangeImpactPublicationId::initial(),
            },
        }
    }

    /// Returns the startup Workspace root retained by this immutable snapshot.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Returns configuration snapshots in canonical Configuration identity order.
    #[must_use]
    pub fn configurations(&self) -> &[WorkspaceConfigurationSnapshot] {
        &self.configurations
    }

    /// Returns the process-local identity of this complete publication.
    #[must_use]
    pub const fn publication_id(&self) -> ChangeImpactPublicationId {
        self.change_impact.current_publication_id()
    }

    /// Returns change-impact availability paired atomically with this snapshot.
    #[must_use]
    pub const fn change_impact(&self) -> &WorkspaceChangeImpact {
        &self.change_impact
    }

    /// Finds a configuration snapshot by canonical identity.
    #[must_use]
    pub fn configuration(&self, id: &EntityId) -> Option<&WorkspaceConfigurationSnapshot> {
        self.configurations
            .binary_search_by(|candidate| candidate.configuration_id.cmp(id))
            .ok()
            .map(|index| &self.configurations[index])
    }

    /// Returns the number of discovered and built configurations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.configurations.len()
    }

    /// Returns `true` when discovery confirmed no supported configurations.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.configurations.is_empty()
    }
}

impl Default for WorkspaceSnapshot {
    fn default() -> Self {
        Self::initial(PathBuf::new(), Vec::new())
    }
}

/// Synchronous deterministic builder for one complete initial Workspace snapshot.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceSnapshotBuilder<D = FileSystemWorkspaceDetector> {
    detector: D,
}

impl WorkspaceSnapshotBuilder<FileSystemWorkspaceDetector> {
    /// Creates the production filesystem-backed snapshot builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            detector: FileSystemWorkspaceDetector::default(),
        }
    }
}

impl<D> WorkspaceSnapshotBuilder<D> {
    /// Creates a snapshot builder with an explicit discovery port implementation.
    #[must_use]
    pub const fn with_detector(detector: D) -> Self {
        Self { detector }
    }
}

impl<D> WorkspaceSnapshotBuilder<D>
where
    D: WorkspaceDetector,
{
    /// Discovers and atomically constructs a complete immutable snapshot.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic discovery, dispatch, semantic build,
    /// validation, cardinality, or duplicate-identity failure. No snapshot is
    /// returned on failure.
    pub fn build(&self, root: &Path) -> Result<WorkspaceSnapshot, WorkspaceBuildError> {
        let discovered =
            self.detector
                .discover(root)
                .map_err(|source| WorkspaceBuildError::Discovery {
                    root_path: root.to_path_buf(),
                    source,
                })?;
        let mut configurations: BTreeMap<EntityId, WorkspaceConfigurationSnapshot> =
            BTreeMap::new();

        for project in discovered {
            let snapshot = build_configuration(&project)?;
            let configuration_id = snapshot.configuration_id.clone();
            if let Some(existing) = configurations.get(&configuration_id) {
                return Err(WorkspaceBuildError::DuplicateConfigurationIdentity {
                    configuration_id,
                    first_root: existing.root_path.clone(),
                    duplicate_root: snapshot.root_path,
                });
            }
            configurations.insert(configuration_id, snapshot);
        }

        Ok(WorkspaceSnapshot::initial(
            root.to_path_buf(),
            configurations.into_values().collect(),
        ))
    }
}

impl Default for WorkspaceSnapshotBuilder<FileSystemWorkspaceDetector> {
    fn default() -> Self {
        Self::new()
    }
}

fn build_configuration(
    project: &DiscoveredConfiguration,
) -> Result<WorkspaceConfigurationSnapshot, WorkspaceBuildError> {
    match project.format() {
        WorkspaceFormat::Edt => build_edt(project),
        WorkspaceFormat::DesignerXml => build_designer_xml(project),
        format @ (WorkspaceFormat::Extension | WorkspaceFormat::Unknown) => {
            Err(WorkspaceBuildError::UnsupportedFormat {
                root_path: project.root_path().to_path_buf(),
                format,
            })
        }
    }
}

fn build_edt(
    project: &DiscoveredConfiguration,
) -> Result<WorkspaceConfigurationSnapshot, WorkspaceBuildError> {
    let root_path = project.root_path();
    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root_path)
        .map_err(|source| semantic_build_error(root_path, WorkspaceFormat::Edt, source))?;
    let validation = result.validate();
    if !validation.is_valid() {
        return Err(WorkspaceBuildError::GraphValidation {
            root_path: root_path.to_path_buf(),
            format: WorkspaceFormat::Edt,
            validation: Box::new(validation),
        });
    }
    let reference_requests =
        SemanticReferenceRequestLedger::from_requests(result.reference_requests().iter().cloned())
            .map_err(|source| semantic_build_error(root_path, WorkspaceFormat::Edt, source))?;
    let graph = result.graph().clone();
    let diagnostics = result.diagnostics().to_vec();
    let reference_statistics = *result.reference_statistics();
    let report = result.report();

    snapshot_from_parts(
        root_path,
        WorkspaceFormat::Edt,
        graph,
        diagnostics,
        reference_requests,
        reference_statistics,
        report,
        validation,
    )
}

fn build_designer_xml(
    project: &DiscoveredConfiguration,
) -> Result<WorkspaceConfigurationSnapshot, WorkspaceBuildError> {
    let root_path = project.root_path();
    let graph = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph(root_path, DesignerXmlBuildScope::Complete)
        .map_err(|source| semantic_build_error(root_path, WorkspaceFormat::DesignerXml, source))?;
    let diagnostics = Vec::new();
    let reference_requests = SemanticReferenceRequestLedger::new();
    let reference_statistics = SemanticReferenceStatistics::new();
    let report = graph.report();
    let validation = validate_complete_build(
        &graph,
        &diagnostics,
        &reference_requests,
        reference_statistics,
        &report,
    );
    if !validation.is_valid() {
        return Err(WorkspaceBuildError::GraphValidation {
            root_path: root_path.to_path_buf(),
            format: WorkspaceFormat::DesignerXml,
            validation: Box::new(validation),
        });
    }
    snapshot_from_parts(
        root_path,
        WorkspaceFormat::DesignerXml,
        graph,
        diagnostics,
        reference_requests,
        reference_statistics,
        report,
        validation,
    )
}

fn validate_complete_build(
    graph: &SemanticGraph,
    diagnostics: &[SemanticDiagnostic],
    reference_requests: &SemanticReferenceRequestLedger,
    legacy_reference_statistics: SemanticReferenceStatistics,
    report: &SemanticGraphReport,
) -> SemanticGraphValidationResult {
    SemanticGraphValidator::new().validate_build_result_with_reference_requests_and_report(
        graph,
        diagnostics,
        reference_requests,
        legacy_reference_statistics,
        report,
    )
}

fn semantic_build_error(
    root_path: &Path,
    format: WorkspaceFormat,
    source: impl Error + Send + Sync + 'static,
) -> WorkspaceBuildError {
    WorkspaceBuildError::SemanticBuild {
        root_path: root_path.to_path_buf(),
        format,
        source: Box::new(source),
    }
}

#[allow(clippy::too_many_arguments)]
fn snapshot_from_parts(
    root_path: &Path,
    format: WorkspaceFormat,
    graph: SemanticGraph,
    diagnostics: Vec<SemanticDiagnostic>,
    reference_requests: SemanticReferenceRequestLedger,
    reference_statistics: SemanticReferenceStatistics,
    report: SemanticGraphReport,
    validation: SemanticGraphValidationResult,
) -> Result<WorkspaceConfigurationSnapshot, WorkspaceBuildError> {
    let (configuration_id, configuration_name) =
        configuration_identity(&graph).map_err(|actual| {
            WorkspaceBuildError::InvalidConfigurationCardinality {
                root_path: root_path.to_path_buf(),
                format,
                actual,
            }
        })?;
    let registry = RuleRegistry::<Arc<dyn Rule>>::new([])
        .map_err(|source| semantic_build_error(root_path, format, source))?;
    let configuration = RuleConfiguration::default();
    let (rule_execution_report, diagnostic_report) = compose_rule_evidence(
        &registry,
        &configuration,
        &graph,
        &validation,
        &diagnostics,
        &NeverRuleCancelled,
    )
    .map_err(|source| WorkspaceBuildError::SemanticBuild {
        root_path: root_path.to_path_buf(),
        format,
        source,
    })?;

    Ok(WorkspaceConfigurationSnapshot {
        root_path: root_path.to_path_buf(),
        format,
        configuration_id,
        configuration_name,
        graph: Arc::new(graph),
        diagnostics: Arc::from(diagnostics.into_boxed_slice()),
        reference_requests: Arc::new(reference_requests),
        reference_statistics,
        report,
        validation: Arc::new(validation),
        rule_execution_report: Arc::new(rule_execution_report),
        diagnostic_report: Arc::new(diagnostic_report),
    })
}

fn compose_rule_evidence<R>(
    registry: &RuleRegistry<R>,
    configuration: &RuleConfiguration,
    graph: &SemanticGraph,
    validation: &SemanticGraphValidationResult,
    diagnostics: &[SemanticDiagnostic],
    cancellation: &dyn RuleCancellationSignal,
) -> Result<(RuleExecutionReport, DiagnosticReport), BoxError>
where
    R: Rule,
{
    let policy = DiagnosticPolicy::default();
    let base = DiagnosticEngine
        .build(diagnostics, validation, &policy)
        .map_err(|error| Box::new(error) as BoxError)?;
    let plan =
        RulePlan::new(registry, configuration).map_err(|error| Box::new(error) as BoxError)?;
    let context = RuleContext::new(graph, validation, &base);
    let rule_report = RuleEngine
        .execute(registry, &plan, configuration, &context, cancellation)
        .map_err(|error| Box::new(error) as BoxError)?;
    let final_report = DiagnosticEngine
        .build_with_rules(diagnostics, validation, rule_report.diagnostics(), &policy)
        .map_err(|error| Box::new(error) as BoxError)?;
    Ok((rule_report, final_report))
}

fn configuration_identity(graph: &SemanticGraph) -> Result<(EntityId, EntityName), usize> {
    let configurations = graph.nodes_by_kind(NodeKind::Metadata(MetadataKind::Configuration));
    if configurations.len() != 1 {
        return Err(configurations.len());
    }
    let configuration = configurations[0];
    Ok((configuration.id().clone(), configuration.name().clone()))
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::fs;
    use std::future::pending;
    use std::io;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use oneagent_analysis::change_impact::{
        ChangeImpactCancellationSignal, ChangeImpactErrorKind, ChangeImpactPublicationId,
        NeverCancelledChangeImpact,
    };
    use oneagent_analysis::diagnostics::{
        DiagnosticCategory, DiagnosticFamily, DiagnosticSeverity, MAX_SEMANTIC_DIAGNOSTICS,
    };
    use oneagent_analysis::rules::{
        Rule, RuleCancellationSignal, RuleConfiguration, RuleContext, RuleDefinition,
        RuleDiagnostic, RuleDiagnosticCode, RuleEvaluation, RuleId, RuleRegistration, RuleRegistry,
        RuleStatus,
    };
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        GraphNode, NodeKind, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphReport,
        SemanticGraphValidationCode, SemanticGraphValidator, SemanticReference,
        SemanticReferenceRequestLedger, SemanticReferenceStatistics,
    };
    use oneagent_metadata::MetadataKind;
    use oneagent_workspace::WorkspaceFormat;
    use tempfile::tempdir;
    use tokio::sync::{mpsc, oneshot, watch};
    use tokio::time::timeout;

    use crate::{App, ConfigurationProvider, LifecycleState, RuntimeConfig, RuntimeErrorKind};

    use super::cache::{WorkspaceCacheLoad, WorkspaceCacheStorage};
    use super::{
        DiscoveredConfiguration, GitChangeSet, GitCommitId, RepositoryChange, RepositoryChangeKind,
        RepositoryChangePath, WorkspaceBuildErrorKind, WorkspaceCacheLoadOutcome,
        WorkspaceCacheWriteOutcome, WorkspaceChangeImpact, WorkspaceChangeSubmissionOutcome,
        WorkspaceDetector, WorkspaceFileState, WorkspaceRebuildError, WorkspaceService,
        WorkspaceSnapshot, WorkspaceSnapshotBuilder, WorkspaceUpdateFailureKind,
        WorkspaceUpdatePhase, WorkspaceUpdateStatus, compose_change_impact, compose_rule_evidence,
        initialize_workspace, rebuild_workspace, snapshot_from_parts, validate_complete_build,
    };

    const DUMP_INFO: &str = r#"<ConfigDumpInfo xmlns="http://v8.1c.ru/8.3/xcf/dumpinfo" format="Hierarchical" version="2.20"><ConfigVersions /></ConfigDumpInfo>"#;
    const TEST_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

    fn explicit_change(path: &str) -> GitChangeSet {
        let path = RepositoryChangePath::new(path).expect("test change path must be valid");
        let change = RepositoryChange::new(
            RepositoryChangeKind::Modified,
            Some(path.clone()),
            Some(path),
        )
        .expect("test change must be valid");
        GitChangeSet::new(
            GitCommitId::new(TEST_HEAD).expect("test baseline must be valid"),
            [change],
        )
        .expect("test change set must be valid")
    }

    fn empty_change() -> GitChangeSet {
        GitChangeSet::new(
            GitCommitId::new(TEST_HEAD).expect("test baseline must be valid"),
            [],
        )
        .expect("empty test change set must be valid")
    }

    #[derive(Debug, Clone)]
    struct StaticDetector {
        projects: Vec<DiscoveredConfiguration>,
    }

    #[derive(Debug, Clone)]
    struct TestConfigurationProvider {
        workspace_root: PathBuf,
    }

    impl ConfigurationProvider for TestConfigurationProvider {
        fn load(&self) -> Result<RuntimeConfig, Box<dyn std::error::Error + Send + Sync>> {
            Ok(RuntimeConfig::new("OneAgent Runtime", "test")
                .with_workspace_root(self.workspace_root.clone()))
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct PanickingDetector;

    #[derive(Debug, Clone)]
    struct GatedDetector {
        calls: Arc<AtomicUsize>,
        second_started: std::sync::mpsc::Sender<()>,
        second_release: Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
    }

    #[derive(Debug, Clone)]
    struct CountingDetector {
        calls: Arc<AtomicUsize>,
    }

    #[derive(Debug, Clone)]
    struct MutatingDetector {
        calls: Arc<AtomicUsize>,
    }

    #[derive(Debug, Clone)]
    struct FailingDetector {
        calls: Arc<AtomicUsize>,
    }

    struct ControlledRule {
        definition: RuleDefinition,
        diagnostic: RuleDiagnostic,
    }

    impl RuleRegistration for ControlledRule {
        fn definition(&self) -> &RuleDefinition {
            &self.definition
        }
    }

    impl Rule for ControlledRule {
        fn evaluate(
            &self,
            _context: &RuleContext<'_>,
            _cancellation: &dyn RuleCancellationSignal,
        ) -> RuleEvaluation {
            RuleEvaluation::Completed(vec![self.diagnostic.clone()])
        }
    }

    struct AlwaysCancelled;

    impl RuleCancellationSignal for AlwaysCancelled {
        fn is_cancelled(&self) -> bool {
            true
        }
    }

    impl ChangeImpactCancellationSignal for AlwaysCancelled {
        fn is_cancelled(&self) -> bool {
            true
        }
    }

    struct ControlledCacheStorage {
        load: WorkspaceCacheLoadOutcome,
        snapshot: Option<WorkspaceSnapshot>,
        write: WorkspaceCacheWriteOutcome,
        loads: AtomicUsize,
        writes: AtomicUsize,
    }

    impl ControlledCacheStorage {
        fn new(
            load: WorkspaceCacheLoadOutcome,
            snapshot: Option<WorkspaceSnapshot>,
            write: WorkspaceCacheWriteOutcome,
        ) -> Self {
            Self {
                load,
                snapshot,
                write,
                loads: AtomicUsize::new(0),
                writes: AtomicUsize::new(0),
            }
        }

        fn loads(&self) -> usize {
            self.loads.load(Ordering::SeqCst)
        }

        fn writes(&self) -> usize {
            self.writes.load(Ordering::SeqCst)
        }
    }

    impl WorkspaceCacheStorage for ControlledCacheStorage {
        fn load(&self, _state: &WorkspaceFileState) -> WorkspaceCacheLoad {
            self.loads.fetch_add(1, Ordering::SeqCst);
            if self.load == WorkspaceCacheLoadOutcome::Hit {
                WorkspaceCacheLoad::hit(
                    self.snapshot
                        .clone()
                        .expect("controlled cache hit must contain a snapshot"),
                )
            } else {
                WorkspaceCacheLoad::without_snapshot(self.load)
            }
        }

        fn write(
            &self,
            _state: &WorkspaceFileState,
            _snapshot: &WorkspaceSnapshot,
        ) -> WorkspaceCacheWriteOutcome {
            self.writes.fetch_add(1, Ordering::SeqCst);
            self.write
        }
    }

    impl WorkspaceDetector for PanickingDetector {
        fn discover(
            &self,
            _root: &std::path::Path,
        ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>>
        {
            panic!("controlled Workspace detector panic")
        }
    }

    impl WorkspaceDetector for StaticDetector {
        fn discover(
            &self,
            _root: &std::path::Path,
        ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(self.projects.clone())
        }
    }

    impl WorkspaceDetector for GatedDetector {
        fn discover(
            &self,
            _root: &std::path::Path,
        ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>>
        {
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if call == 2 {
                self.second_started
                    .send(())
                    .expect("second build start must be observed");
                self.second_release
                    .lock()
                    .expect("build release lock must remain available")
                    .recv()
                    .expect("second build must be released");
            }
            Ok(Vec::new())
        }
    }

    impl WorkspaceDetector for CountingDetector {
        fn discover(
            &self,
            _root: &std::path::Path,
        ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>>
        {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(Vec::new())
        }
    }

    impl WorkspaceDetector for MutatingDetector {
        fn discover(
            &self,
            root: &std::path::Path,
        ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>>
        {
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            fs::write(root.join("changed-during-build.bsl"), call.to_string())?;
            Ok(Vec::new())
        }
    }

    impl WorkspaceDetector for FailingDetector {
        fn discover(
            &self,
            _root: &std::path::Path,
        ) -> Result<Vec<DiscoveredConfiguration>, Box<dyn std::error::Error + Send + Sync>>
        {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(Box::new(io::Error::other("controlled discovery failure")))
        }
    }

    async fn wait_for_snapshot(
        snapshot: &mut watch::Receiver<Option<std::sync::Arc<WorkspaceSnapshot>>>,
    ) -> std::sync::Arc<WorkspaceSnapshot> {
        loop {
            if let Some(snapshot) = snapshot.borrow().clone() {
                return snapshot;
            }
            timeout(Duration::from_secs(1), snapshot.changed())
                .await
                .expect("Workspace snapshot wait must not hang")
                .expect("Workspace service must retain snapshot ownership");
        }
    }

    async fn wait_for_snapshot_clear(
        snapshot: &mut watch::Receiver<Option<std::sync::Arc<WorkspaceSnapshot>>>,
    ) {
        while snapshot.borrow().is_some() {
            let _ = timeout(Duration::from_secs(1), snapshot.changed())
                .await
                .expect("Workspace snapshot cleanup must not hang");
        }
    }

    async fn wait_for_lifecycle(
        lifecycle: &mut watch::Receiver<LifecycleState>,
        expected: LifecycleState,
    ) {
        while *lifecycle.borrow() != expected {
            timeout(Duration::from_secs(1), lifecycle.changed())
                .await
                .expect("lifecycle wait must not hang")
                .expect("Runtime must retain lifecycle ownership");
        }
    }

    async fn wait_for_update(
        updates: &mut watch::Receiver<WorkspaceUpdateStatus>,
        predicate: impl Fn(WorkspaceUpdateStatus) -> bool,
    ) -> WorkspaceUpdateStatus {
        loop {
            let status = *updates.borrow_and_update();
            if predicate(status) {
                return status;
            }
            timeout(Duration::from_secs(1), updates.changed())
                .await
                .expect("Workspace update wait must not hang")
                .expect("Workspace service must retain update ownership");
        }
    }

    async fn scan_once(ticks: &mpsc::Sender<oneshot::Sender<()>>) {
        let (acknowledgement, acknowledged) = oneshot::channel();
        ticks
            .send(acknowledgement)
            .await
            .expect("controlled Workspace scan must be accepted");
        timeout(Duration::from_secs(1), acknowledged)
            .await
            .expect("controlled Workspace scan must not hang")
            .expect("controlled Workspace scan must acknowledge completion");
    }

    async fn wait_for_watch_closed<T>(receiver: &mut watch::Receiver<T>) {
        loop {
            match timeout(Duration::from_secs(1), receiver.changed())
                .await
                .expect("watch closure wait must not hang")
            {
                Ok(()) => {}
                Err(_) => return,
            }
        }
    }

    fn edt_configuration(id: &str, name: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="{id}">
    <name>{name}</name>
</mdclass:Configuration>
"#
        )
    }

    fn designer_configuration(id: &str, name: &str) -> String {
        format!(
            r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Configuration uuid="{id}"><Properties><Name>{name}</Name></Properties></Configuration></MetaDataObject>"#
        )
    }

    fn write_edt(root: &std::path::Path, id: &str, name: &str) {
        fs::create_dir_all(root.join("src/Configuration"))
            .expect("EDT Configuration directory must be created");
        fs::write(root.join(".project"), "<projectDescription />")
            .expect("EDT marker must be created");
        fs::write(
            root.join("src/Configuration/Configuration.mdo"),
            edt_configuration(id, name),
        )
        .expect("EDT Configuration must be created");
    }

    fn write_designer(root: &std::path::Path, id: &str, name: &str) {
        fs::create_dir_all(root).expect("Designer root must be created");
        fs::write(root.join("ConfigDumpInfo.xml"), DUMP_INFO)
            .expect("Designer dump marker must be created");
        fs::write(
            root.join("Configuration.xml"),
            designer_configuration(id, name),
        )
        .expect("Designer Configuration must be created");
    }

    #[test]
    fn empty_workspace_builds_one_immutable_empty_snapshot() {
        let root = tempdir().expect("temporary Workspace root must be created");

        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(root.path())
            .expect("empty Workspace must build");

        assert!(snapshot.is_empty());
        assert_eq!(snapshot.len(), 0);
        assert!(snapshot.configurations().is_empty());
        assert_eq!(snapshot.root_path(), root.path());
        assert_eq!(
            snapshot.publication_id(),
            ChangeImpactPublicationId::initial()
        );
        assert!(matches!(
            snapshot.change_impact(),
            WorkspaceChangeImpact::NoPreviousPublication { .. }
        ));
    }

    #[test]
    fn workspace_impact_composition_uses_only_identity_and_graph_and_fails_atomically() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let edt = root.path().join("edt");
        write_edt(&edt, "configuration:impact", "ImpactBefore");
        let previous = WorkspaceSnapshotBuilder::new()
            .build(root.path())
            .expect("previous Workspace snapshot must build");
        let mut current = previous.clone();
        current.configurations[0].format = WorkspaceFormat::DesignerXml;
        current.configurations[0].configuration_name =
            EntityName::new("ImpactAfter").expect("changed test name must be valid");

        compose_change_impact(&previous, &mut current, &NeverCancelledChangeImpact)
            .expect("source-format and name transitions must compare by identity and graph");
        let report = current
            .change_impact()
            .report()
            .expect("successful composition must embed a report");
        assert_eq!(report.previous_publication_id(), previous.publication_id());
        assert_eq!(report.current_publication_id(), current.publication_id());
        assert_eq!(report.summary().compared_configurations(), 1);
        assert_eq!(report.summary().total_affected_nodes(), 0);

        let mut cancelled = WorkspaceSnapshot::default();
        let cancelled_before = cancelled.change_impact().clone();
        let error = compose_change_impact(&previous, &mut cancelled, &AlwaysCancelled)
            .expect_err("cancelled composition must fail without mutation");
        assert_eq!(error.kind(), ChangeImpactErrorKind::Cancelled);
        assert_eq!(cancelled.change_impact(), &cancelled_before);

        let exhausted = WorkspaceSnapshot {
            change_impact: WorkspaceChangeImpact::NoPreviousPublication {
                current_publication_id: ChangeImpactPublicationId::new(u64::MAX)
                    .expect("maximum publication identity is non-zero"),
            },
            ..WorkspaceSnapshot::default()
        };
        let mut candidate = WorkspaceSnapshot::default();
        let candidate_before = candidate.change_impact().clone();
        let error = compose_change_impact(&exhausted, &mut candidate, &NeverCancelledChangeImpact)
            .expect_err("publication overflow must fail without mutation");
        assert_eq!(error.kind(), ChangeImpactErrorKind::SummaryOverflow);
        assert_eq!(candidate.change_impact(), &candidate_before);
    }

    #[test]
    fn diagnostic_composition_accepts_exact_and_rejects_one_over_atomically() {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            EntityId::new("configuration:test").expect("configuration ID must be valid"),
            EntityName::new("Test").expect("configuration name must be valid"),
            NodeKind::Metadata(MetadataKind::Configuration),
        ));
        let diagnostic = SemanticDiagnostic::new(
            SemanticDiagnosticCode::QueryLanguageMalformedSyntax,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::QueryLanguageMalformedSyntax,
            "malformed query",
            SemanticReference::Raw("query".to_owned()),
        );
        let exact = vec![diagnostic.clone(); MAX_SEMANTIC_DIAGNOSTICS];
        let report = SemanticGraphReport::from_graph_diagnostics_and_references(
            &graph,
            &exact,
            SemanticReferenceStatistics::new(),
        );
        let validation = SemanticGraphValidator::new().validate_build_result_with_report(
            &graph,
            &exact,
            SemanticReferenceStatistics::new(),
            &report,
        );
        let snapshot = snapshot_from_parts(
            std::path::Path::new("configuration"),
            WorkspaceFormat::Edt,
            graph.clone(),
            exact,
            SemanticReferenceRequestLedger::new(),
            SemanticReferenceStatistics::new(),
            report,
            validation,
        )
        .expect("exact diagnostic input bound must publish");
        assert_eq!(snapshot.diagnostics().len(), MAX_SEMANTIC_DIAGNOSTICS);
        assert!(snapshot.rule_execution_report().results().is_empty());
        assert!(snapshot.rule_execution_report().diagnostics().is_empty());
        assert_eq!(snapshot.rule_execution_report().summary().total(), 0);
        assert_eq!(snapshot.diagnostic_report().summary().total(), 2);

        let over = vec![diagnostic; MAX_SEMANTIC_DIAGNOSTICS + 1];
        let report = SemanticGraphReport::from_graph_diagnostics_and_references(
            &graph,
            &over,
            SemanticReferenceStatistics::new(),
        );
        let validation = SemanticGraphValidator::new().validate_build_result_with_report(
            &graph,
            &over,
            SemanticReferenceStatistics::new(),
            &report,
        );
        let error = snapshot_from_parts(
            std::path::Path::new("configuration"),
            WorkspaceFormat::Edt,
            graph,
            over,
            SemanticReferenceRequestLedger::new(),
            SemanticReferenceStatistics::new(),
            report,
            validation,
        )
        .expect_err("one-over diagnostic input must not publish");
        assert_eq!(error.kind(), WorkspaceBuildErrorKind::SemanticBuildFailed);
    }

    #[test]
    fn rule_composition_publishes_complete_and_cancelled_evidence_atomically() {
        let configuration_id =
            EntityId::new("configuration:test").expect("configuration ID must be valid");
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            configuration_id.clone(),
            EntityName::new("Test").expect("configuration name must be valid"),
            NodeKind::Metadata(MetadataKind::Configuration),
        ));
        let validation = graph.validate();
        let rule_id = RuleId::new("runtime.rule").expect("rule ID must be valid");
        let rule: Arc<dyn Rule> = Arc::new(ControlledRule {
            definition: RuleDefinition::new(rule_id.clone(), [])
                .expect("rule definition must be valid"),
            diagnostic: RuleDiagnostic::new(
                rule_id,
                RuleDiagnosticCode::new("finding").expect("diagnostic code must be valid"),
                DiagnosticSeverity::Warning,
                DiagnosticCategory::Semantic,
                "controlled runtime rule finding",
                [configuration_id],
            ),
        });
        let registry =
            RuleRegistry::<Arc<dyn Rule>>::new([rule]).expect("controlled registry must be valid");

        let (completed, diagnostics) = compose_rule_evidence(
            &registry,
            &RuleConfiguration::default(),
            &graph,
            &validation,
            &[],
            &super::NeverRuleCancelled,
        )
        .expect("complete rule evidence must compose");
        assert_eq!(completed.results().len(), 1);
        assert_eq!(completed.results()[0].status(), RuleStatus::Completed);
        assert_eq!(completed.diagnostics().len(), 1);
        assert_eq!(diagnostics.summary().total(), 2);
        assert_eq!(
            diagnostics
                .summary()
                .by_family()
                .get(&DiagnosticFamily::Rule),
            Some(&1)
        );

        let (cancelled, diagnostics) = compose_rule_evidence(
            &registry,
            &RuleConfiguration::default(),
            &graph,
            &validation,
            &[],
            &AlwaysCancelled,
        )
        .expect("cancelled rule evidence must compose");
        assert_eq!(cancelled.results().len(), 1);
        assert_eq!(cancelled.results()[0].status(), RuleStatus::Cancelled);
        assert!(cancelled.diagnostics().is_empty());
        assert_eq!(diagnostics.summary().total(), 1);
        assert!(
            diagnostics
                .summary()
                .by_family()
                .get(&DiagnosticFamily::Rule)
                .is_none()
        );
    }

    #[test]
    fn designer_complete_build_validation_rejects_a_mismatched_report() {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            EntityId::new("configuration:designer")
                .expect("Designer configuration ID must be valid"),
            EntityName::new("Designer").expect("Designer configuration name must be valid"),
            NodeKind::Metadata(MetadataKind::Configuration),
        ));
        let mismatched_report = SemanticGraphReport::from_graph(&SemanticGraph::new());
        let reference_requests = SemanticReferenceRequestLedger::new();

        assert!(graph.validate().is_valid());
        let validation = validate_complete_build(
            &graph,
            &[],
            &reference_requests,
            SemanticReferenceStatistics::new(),
            &mismatched_report,
        );

        assert!(!validation.is_valid());
        assert!(
            validation
                .issues()
                .iter()
                .any(|issue| { issue.code() == SemanticGraphValidationCode::InconsistentReport })
        );
    }

    #[test]
    fn supported_builds_are_normalized_by_configuration_identity_and_repeat() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let designer = root.path().join("a-designer");
        let edt = root.path().join("z-edt");
        write_designer(
            &designer,
            "99999999-0000-0000-0000-000000000000",
            "DesignerConfiguration",
        );
        write_edt(
            &edt,
            "11111111-0000-0000-0000-000000000000",
            "EdtConfiguration",
        );
        let builder = WorkspaceSnapshotBuilder::new();

        let first = builder.build(root.path()).expect("Workspace must build");
        let repeated = builder
            .build(root.path())
            .expect("repeated Workspace must build");

        assert_eq!(first.len(), 2);
        assert_eq!(first.root_path(), root.path());
        assert_eq!(repeated.root_path(), root.path());
        assert_eq!(
            first.configurations()[0].configuration_name().as_str(),
            "EdtConfiguration"
        );
        assert_eq!(first.configurations()[0].format(), WorkspaceFormat::Edt);
        assert_eq!(
            first.configurations()[1].configuration_name().as_str(),
            "DesignerConfiguration"
        );
        assert_eq!(
            first.configurations()[1].format(),
            WorkspaceFormat::DesignerXml
        );
        for (actual, expected) in first.configurations().iter().zip(repeated.configurations()) {
            assert_eq!(actual.configuration_id(), expected.configuration_id());
            assert_eq!(actual.root_path(), expected.root_path());
            assert!(actual.graph().diff(expected.graph()).is_empty());
            assert_eq!(actual.diagnostics(), expected.diagnostics());
            assert_eq!(actual.reference_requests(), expected.reference_requests());
            assert_eq!(
                actual.reference_statistics(),
                expected.reference_statistics()
            );
            assert_eq!(actual.report(), expected.report());
            assert!(actual.graph().validate().is_valid());
        }
        let designer = &first.configurations()[1];
        assert!(designer.diagnostics().is_empty());
        assert!(designer.reference_requests().is_empty());
        assert_eq!(
            designer.validation(),
            &SemanticGraphValidator::new()
                .validate_build_result_with_reference_requests_and_report(
                    designer.graph(),
                    designer.diagnostics(),
                    designer.reference_requests(),
                    SemanticReferenceStatistics::new(),
                    designer.report(),
                )
        );
    }

    #[test]
    fn duplicate_configuration_identity_rejects_the_complete_snapshot() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let edt = root.path().join("a-edt");
        let designer = root.path().join("b-designer");
        let duplicate = "11111111-0000-0000-0000-000000000000";
        write_edt(&edt, duplicate, "EdtConfiguration");
        write_designer(&designer, duplicate, "DesignerConfiguration");

        let error = WorkspaceSnapshotBuilder::new()
            .build(root.path())
            .expect_err("duplicate Configuration identity must fail atomically");

        assert_eq!(
            error.kind(),
            WorkspaceBuildErrorKind::DuplicateConfigurationIdentity
        );
        assert_eq!(
            error
                .configuration_id()
                .expect("duplicate identity must be exposed")
                .as_str(),
            duplicate
        );
        assert_eq!(error.root_path(), designer);
    }

    #[test]
    fn unsupported_format_is_rejected_before_source_parsing() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let project = root.path().join("extension");
        let builder = WorkspaceSnapshotBuilder::with_detector(StaticDetector {
            projects: vec![DiscoveredConfiguration::new(
                &project,
                WorkspaceFormat::Extension,
            )],
        });

        let error = builder
            .build(root.path())
            .expect_err("unsupported source format must fail");

        assert_eq!(error.kind(), WorkspaceBuildErrorKind::UnsupportedFormat);
        assert_eq!(error.format(), Some(WorkspaceFormat::Extension));
        assert_eq!(error.root_path(), project);
    }

    #[test]
    fn adapter_failure_preserves_format_root_and_source_chain() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let designer = root.path().join("designer");
        write_designer(
            &designer,
            "99999999-0000-0000-0000-000000000000",
            "DesignerConfiguration",
        );
        fs::create_dir_all(designer.join("Catalogs"))
            .expect("Designer Catalogs directory must be created");
        fs::write(designer.join("Catalogs/Broken.xml"), "<broken>")
            .expect("malformed accepted descriptor must be created");

        let error = WorkspaceSnapshotBuilder::new()
            .build(root.path())
            .expect_err("fatal adapter input must reject the complete snapshot");

        assert_eq!(error.kind(), WorkspaceBuildErrorKind::SemanticBuildFailed);
        assert_eq!(error.format(), Some(WorkspaceFormat::DesignerXml));
        assert_eq!(error.root_path(), designer);
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn missing_root_preserves_discovery_source() {
        let root = tempdir().expect("temporary parent must be created");
        let missing = root.path().join("missing");

        let error = WorkspaceSnapshotBuilder::new()
            .build(&missing)
            .expect_err("missing configured root must fail discovery");

        assert_eq!(error.kind(), WorkspaceBuildErrorKind::DiscoveryFailed);
        assert_eq!(error.root_path(), missing);
        assert!(std::error::Error::source(&error).is_some());
    }

    #[tokio::test]
    async fn workspace_cache_warm_hit_skips_build_and_closes_status_on_shutdown() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let calls = Arc::new(AtomicUsize::new(0));
        let storage = Arc::new(ControlledCacheStorage::new(
            WorkspaceCacheLoadOutcome::Hit,
            Some(WorkspaceSnapshot::default()),
            WorkspaceCacheWriteOutcome::Succeeded,
        ));
        let service = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
            CountingDetector {
                calls: Arc::clone(&calls),
            },
        ))
        .with_cache_storage(storage.clone());
        let snapshot = service.snapshot_observer();
        let mut snapshot_changes = snapshot.subscribe();
        let cache = service.cache_observer();
        let mut cache_changes = cache.subscribe();
        assert_eq!(
            cache.status().load(),
            WorkspaceCacheLoadOutcome::NotAttempted
        );
        assert_eq!(
            cache.status().write(),
            WorkspaceCacheWriteOutcome::NotAttempted
        );
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));

        wait_for_snapshot(&mut snapshot_changes).await;
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(storage.loads(), 1);
        assert_eq!(storage.writes(), 0);
        assert_eq!(cache.status().load(), WorkspaceCacheLoadOutcome::Hit);
        assert_eq!(
            cache.status().write(),
            WorkspaceCacheWriteOutcome::NotAttempted
        );

        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
        wait_for_watch_closed(&mut cache_changes).await;
    }

    #[tokio::test]
    async fn workspace_cache_rejected_loads_build_once_and_write_stable_snapshot() {
        for load in [
            WorkspaceCacheLoadOutcome::Missing,
            WorkspaceCacheLoadOutcome::Corrupt,
            WorkspaceCacheLoadOutcome::Unavailable,
        ] {
            let root = tempdir().expect("temporary Workspace root must be created");
            let calls = Arc::new(AtomicUsize::new(0));
            let storage = Arc::new(ControlledCacheStorage::new(
                load,
                None,
                WorkspaceCacheWriteOutcome::Succeeded,
            ));
            let service = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
                CountingDetector {
                    calls: Arc::clone(&calls),
                },
            ))
            .with_cache_storage(storage.clone());
            let snapshot = service.snapshot_observer();
            let mut snapshot_changes = snapshot.subscribe();
            let cache = service.cache_observer();
            let provider = TestConfigurationProvider {
                workspace_root: root.path().to_path_buf(),
            };
            let app = App::builder()
                .configure(&provider)
                .expect("test configuration must load")
                .register_service("workspace", service)
                .expect("Workspace service must register")
                .build()
                .expect("application must build");
            let (shutdown_sender, shutdown) = oneshot::channel::<()>();
            let run = tokio::spawn(app.run(shutdown));

            wait_for_snapshot(&mut snapshot_changes).await;
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(storage.loads(), 1);
            assert_eq!(storage.writes(), 1);
            assert_eq!(cache.status().load(), load);
            assert_eq!(
                cache.status().write(),
                WorkspaceCacheWriteOutcome::Succeeded
            );

            shutdown_sender.send(()).expect("shutdown must be observed");
            timeout(Duration::from_secs(1), run)
                .await
                .expect("Workspace shutdown must not hang")
                .expect("Workspace task must join")
                .expect("requested shutdown must succeed");
        }
    }

    #[tokio::test]
    async fn workspace_cache_write_failure_is_nonfatal_and_snapshot_is_published() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let calls = Arc::new(AtomicUsize::new(0));
        let (ticks, controlled_ticks) = mpsc::channel(8);
        let storage = Arc::new(ControlledCacheStorage::new(
            WorkspaceCacheLoadOutcome::Missing,
            None,
            WorkspaceCacheWriteOutcome::Failed,
        ));
        let service = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
            CountingDetector {
                calls: Arc::clone(&calls),
            },
        ))
        .with_cache_storage(storage.clone())
        .with_controlled_change_ticks(controlled_ticks);
        let snapshot = service.snapshot_observer();
        let mut snapshot_changes = snapshot.subscribe();
        let cache = service.cache_observer();
        let updates = service.update_observer();
        let mut update_changes = updates.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let mut lifecycle = app.subscribe_lifecycle();
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));

        wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
        wait_for_snapshot(&mut snapshot_changes).await;
        assert!(snapshot.snapshot().is_some());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(storage.writes(), 1);
        assert_eq!(cache.status().write(), WorkspaceCacheWriteOutcome::Failed);

        fs::write(root.path().join("changed.bsl"), b"changed")
            .expect("relevant source must be written");
        scan_once(&ticks).await;
        let rebuilt = wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 2
        })
        .await;
        assert_eq!(rebuilt.failure(), None);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(storage.writes(), 2);
        assert!(snapshot.snapshot().is_some());
        assert_eq!(cache.status().write(), WorkspaceCacheWriteOutcome::Failed);

        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
    }

    #[tokio::test]
    async fn workspace_cache_persists_across_fresh_runtime_runs() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let calls = Arc::new(AtomicUsize::new(0));
        let first = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
            CountingDetector {
                calls: Arc::clone(&calls),
            },
        ));
        let first_snapshot = first.snapshot_observer();
        let mut first_changes = first_snapshot.subscribe();
        let first_cache = first.cache_observer();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let first_app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", first)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (first_shutdown_sender, first_shutdown) = oneshot::channel::<()>();
        let first_run = tokio::spawn(first_app.run(first_shutdown));
        wait_for_snapshot(&mut first_changes).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            first_cache.status().load(),
            WorkspaceCacheLoadOutcome::Missing
        );
        assert_eq!(
            first_cache.status().write(),
            WorkspaceCacheWriteOutcome::Succeeded
        );
        first_shutdown_sender
            .send(())
            .expect("first shutdown must be observed");
        timeout(Duration::from_secs(1), first_run)
            .await
            .expect("first Workspace shutdown must not hang")
            .expect("first Workspace task must join")
            .expect("first requested shutdown must succeed");

        let cache_path = root.path().join(".oneagent/cache/workspace-v1.json");
        assert!(cache_path.is_file());
        let second = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
            PanickingDetector,
        ));
        let second_snapshot = second.snapshot_observer();
        let mut second_changes = second_snapshot.subscribe();
        let second_cache = second.cache_observer();
        let second_app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", second)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (second_shutdown_sender, second_shutdown) = oneshot::channel::<()>();
        let second_run = tokio::spawn(second_app.run(second_shutdown));
        wait_for_snapshot(&mut second_changes).await;
        assert_eq!(second_cache.status().load(), WorkspaceCacheLoadOutcome::Hit);
        assert_eq!(
            second_cache.status().write(),
            WorkspaceCacheWriteOutcome::NotAttempted
        );
        second_shutdown_sender
            .send(())
            .expect("second shutdown must be observed");
        timeout(Duration::from_secs(1), second_run)
            .await
            .expect("second Workspace shutdown must not hang")
            .expect("second Workspace task must join")
            .expect("second requested shutdown must succeed");
        assert!(cache_path.is_file());
    }

    #[test]
    fn workspace_cache_builds_write_only_complete_stable_results() {
        let previous = WorkspaceSnapshot::default();
        let stable_root = tempdir().expect("stable Workspace root must be created");
        let stable_storage = ControlledCacheStorage::new(
            WorkspaceCacheLoadOutcome::NotAttempted,
            None,
            WorkspaceCacheWriteOutcome::Succeeded,
        );
        let stable = rebuild_workspace(
            &WorkspaceSnapshotBuilder::with_detector(CountingDetector {
                calls: Arc::new(AtomicUsize::new(0)),
            }),
            stable_root.path(),
            &stable_storage,
            &previous,
            &NeverCancelledChangeImpact,
        )
        .expect("stable rebuild must succeed");
        assert_eq!(stable.write, WorkspaceCacheWriteOutcome::Succeeded);
        assert_eq!(stable_storage.writes(), 1);

        let unstable_root = tempdir().expect("unstable Workspace root must be created");
        let unstable_storage = ControlledCacheStorage::new(
            WorkspaceCacheLoadOutcome::NotAttempted,
            None,
            WorkspaceCacheWriteOutcome::Succeeded,
        );
        let unstable = rebuild_workspace(
            &WorkspaceSnapshotBuilder::with_detector(MutatingDetector {
                calls: Arc::new(AtomicUsize::new(0)),
            }),
            unstable_root.path(),
            &unstable_storage,
            &previous,
            &NeverCancelledChangeImpact,
        )
        .expect("unstable semantic rebuild remains valid");
        assert_eq!(
            unstable.write,
            WorkspaceCacheWriteOutcome::SkippedUnstableSource
        );
        assert_eq!(unstable_storage.writes(), 0);

        let failed_root = tempdir().expect("failed Workspace root must be created");
        let failed_calls = Arc::new(AtomicUsize::new(0));
        let failed_storage = ControlledCacheStorage::new(
            WorkspaceCacheLoadOutcome::NotAttempted,
            None,
            WorkspaceCacheWriteOutcome::Succeeded,
        );
        let error = rebuild_workspace(
            &WorkspaceSnapshotBuilder::with_detector(FailingDetector {
                calls: Arc::clone(&failed_calls),
            }),
            failed_root.path(),
            &failed_storage,
            &previous,
            &NeverCancelledChangeImpact,
        )
        .expect_err("failed semantic build must retain the prior publication");
        assert!(matches!(
            error,
            WorkspaceRebuildError::Build(error)
                if error.kind() == WorkspaceBuildErrorKind::DiscoveryFailed
        ));
        assert_eq!(failed_calls.load(Ordering::SeqCst), 1);
        assert_eq!(failed_storage.writes(), 0);
    }

    #[test]
    fn workspace_cache_startup_marks_unstable_source_for_follow_up() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let storage = ControlledCacheStorage::new(
            WorkspaceCacheLoadOutcome::Missing,
            None,
            WorkspaceCacheWriteOutcome::Succeeded,
        );
        let (cache_status, _receiver) = watch::channel(super::WorkspaceCacheStatus::starting());
        let initialized = initialize_workspace(
            &WorkspaceSnapshotBuilder::with_detector(MutatingDetector {
                calls: Arc::new(AtomicUsize::new(0)),
            }),
            root.path(),
            &storage,
            &cache_status,
        )
        .expect("unstable startup build remains valid");

        assert!(initialized.follow_up_required);
        assert_eq!(storage.loads(), 1);
        assert_eq!(storage.writes(), 0);
        assert_eq!(
            cache_status.borrow().write(),
            WorkspaceCacheWriteOutcome::SkippedUnstableSource
        );
    }

    #[tokio::test]
    async fn workspace_service_publishes_for_running_and_clears_on_shutdown() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let service = WorkspaceService::new();
        let observer = service.snapshot_observer();
        let mut snapshot_changes = observer.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let mut lifecycle = app.subscribe_lifecycle();
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        assert!(observer.snapshot().is_none());
        assert_eq!(*lifecycle.borrow(), LifecycleState::Initializing);
        let run = tokio::spawn(app.run(shutdown));

        let snapshot = wait_for_snapshot(&mut snapshot_changes).await;
        assert!(snapshot.is_empty());
        assert!(observer.snapshot().is_some());
        wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

        shutdown_sender
            .send(())
            .expect("shutdown request must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace Runtime shutdown must not hang")
            .expect("Runtime task must join")
            .expect("requested shutdown must succeed");
        wait_for_snapshot_clear(&mut snapshot_changes).await;
        wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
        assert!(observer.snapshot().is_none());
    }

    #[test]
    fn workspace_change_input_outcomes_are_exact_bounded_and_redacted() {
        let service = WorkspaceService::new();
        let input = service.change_input_handle();
        let repeated = input.clone();

        assert_eq!(
            input.submit(empty_change()),
            WorkspaceChangeSubmissionOutcome::IgnoredEmpty
        );
        assert_eq!(
            input.submit(explicit_change("private/first.bsl")),
            WorkspaceChangeSubmissionOutcome::Accepted
        );
        assert_eq!(
            repeated.submit(explicit_change("private/second.bsl")),
            WorkspaceChangeSubmissionOutcome::Backpressure
        );
        assert!(!format!("{input:?}").contains("private"));

        drop(service);
        assert_eq!(
            input.submit(explicit_change("private/closed.bsl")),
            WorkspaceChangeSubmissionOutcome::Closed
        );
        assert_eq!(
            input.submit(empty_change()),
            WorkspaceChangeSubmissionOutcome::IgnoredEmpty
        );
    }

    #[tokio::test]
    async fn workspace_service_reports_named_start_failure_without_publication() {
        let parent = tempdir().expect("temporary parent must be created");
        let missing = parent.path().join("missing");
        let service = WorkspaceService::new();
        let observer = service.snapshot_observer();
        let input = service.change_input_handle();
        let provider = TestConfigurationProvider {
            workspace_root: missing,
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");

        let error = timeout(
            Duration::from_secs(1),
            app.run(pending::<Result<(), Infallible>>()),
        )
        .await
        .expect("Workspace startup failure must not hang")
        .expect_err("invalid root must fail Workspace startup");

        assert_eq!(error.kind(), RuntimeErrorKind::ServiceStartFailed);
        assert_eq!(error.service_name(), Some("workspace"));
        let source = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<super::WorkspaceBuildError>())
            .expect("Workspace startup error must preserve the build classification");
        assert_eq!(source.kind(), WorkspaceBuildErrorKind::ObservationFailed);
        assert!(observer.snapshot().is_none());
        assert_eq!(
            input.submit(explicit_change("after-start-failure.bsl")),
            WorkspaceChangeSubmissionOutcome::Closed
        );
    }

    #[tokio::test]
    async fn workspace_service_classifies_blocking_build_panics() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let service = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
            PanickingDetector,
        ));
        let observer = service.snapshot_observer();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");

        let error = timeout(
            Duration::from_secs(1),
            app.run(pending::<Result<(), Infallible>>()),
        )
        .await
        .expect("Workspace task panic must not hang")
        .expect_err("blocking build panic must fail Workspace startup");
        let source = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<super::WorkspaceBuildError>())
            .expect("Workspace startup error must preserve the task classification");

        assert_eq!(error.kind(), RuntimeErrorKind::ServiceStartFailed);
        assert_eq!(error.service_name(), Some("workspace"));
        assert_eq!(source.kind(), WorkspaceBuildErrorKind::BuildTaskFailed);
        assert!(std::error::Error::source(source).is_some());
        assert!(observer.snapshot().is_none());
    }

    #[tokio::test]
    async fn workspace_service_rebuilds_on_changes_and_ignores_confirmed_directories() {
        let root = tempdir().expect("temporary Workspace root must be created");
        fs::create_dir(root.path().join(".git")).expect("ignored directory must be created");
        fs::write(root.path().join(".git/transient"), b"initial")
            .expect("ignored source must be created");
        let (ticks, controlled_ticks) = mpsc::channel(8);
        let service = WorkspaceService::new().with_controlled_change_ticks(controlled_ticks);
        let observer = service.snapshot_observer();
        let cache = service.cache_observer();
        let updates = service.update_observer();
        let mut update_changes = updates.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));

        let initial = wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching
        })
        .await;
        assert_eq!(initial.attempt(), 1);
        assert_eq!(initial.published(), 1);
        assert_eq!(initial.failure(), None);
        assert_eq!(cache.status().load(), WorkspaceCacheLoadOutcome::Missing);
        assert_eq!(
            cache.status().write(),
            WorkspaceCacheWriteOutcome::Succeeded
        );
        let cache_path = root.path().join(".oneagent/cache/workspace-v1.json");
        let initial_cache = fs::read(&cache_path).expect("initial cache entry must exist");

        fs::write(root.path().join("changed.bsl"), b"first")
            .expect("relevant source must be created");
        scan_once(&ticks).await;
        let rebuilt = wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 2
        })
        .await;
        assert_eq!(rebuilt.attempt(), 2);
        assert_eq!(
            observer
                .snapshot()
                .expect("snapshot must remain published")
                .len(),
            0
        );
        let rebuilt_cache = fs::read(&cache_path).expect("rebuilt cache entry must exist");
        assert_ne!(rebuilt_cache, initial_cache);
        assert_eq!(
            cache.status().write(),
            WorkspaceCacheWriteOutcome::Succeeded
        );

        let before_ignored_change = updates.status();
        fs::write(root.path().join(".git/transient"), b"ignored")
            .expect("ignored source must be created");
        scan_once(&ticks).await;
        assert_eq!(updates.status(), before_ignored_change);
        assert_eq!(
            fs::read(&cache_path).expect("cache entry must remain readable"),
            rebuilt_cache
        );

        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
        assert_eq!(updates.status().phase(), WorkspaceUpdatePhase::Stopped);
        assert!(observer.snapshot().is_none());
    }

    #[tokio::test]
    async fn workspace_service_retains_valid_snapshot_and_recovers_after_invalid_build() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let edt = root.path().join("edt");
        let configuration_id = "11111111-0000-0000-0000-000000000000";
        write_edt(&edt, configuration_id, "Initial");
        let (ticks, controlled_ticks) = mpsc::channel(8);
        let service = WorkspaceService::new().with_controlled_change_ticks(controlled_ticks);
        let observer = service.snapshot_observer();
        let cache = service.cache_observer();
        let updates = service.update_observer();
        let mut update_changes = updates.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));
        wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching
        })
        .await;
        let initial = observer
            .snapshot()
            .expect("initial snapshot must be published");
        assert_eq!(
            initial.configurations()[0].configuration_name().as_str(),
            "Initial"
        );
        let cache_path = root.path().join(".oneagent/cache/workspace-v1.json");
        let initial_cache = fs::read(&cache_path).expect("initial cache entry must exist");

        fs::write(edt.join("src/Configuration/Configuration.mdo"), "<broken>")
            .expect("invalid source must be written");
        scan_once(&ticks).await;
        let failed = wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Failed
        })
        .await;
        assert_eq!(failed.attempt(), 2);
        assert_eq!(failed.published(), 1);
        assert_eq!(
            failed.failure(),
            Some(WorkspaceUpdateFailureKind::SemanticBuild)
        );
        assert_eq!(
            observer
                .snapshot()
                .expect("valid snapshot must be retained")
                .configurations()[0]
                .configuration_name()
                .as_str(),
            "Initial"
        );
        assert_eq!(
            fs::read(&cache_path).expect("last valid cache entry must remain readable"),
            initial_cache
        );
        assert_eq!(
            cache.status().write(),
            WorkspaceCacheWriteOutcome::Succeeded
        );

        write_edt(&edt, configuration_id, "Recovered");
        scan_once(&ticks).await;
        let recovered = wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 2
        })
        .await;
        assert_eq!(recovered.attempt(), 3);
        assert_eq!(recovered.failure(), None);
        assert_eq!(
            observer
                .snapshot()
                .expect("recovered snapshot must be published")
                .configurations()[0]
                .configuration_name()
                .as_str(),
            "Recovered"
        );
        assert_ne!(
            fs::read(&cache_path).expect("recovered cache entry must exist"),
            initial_cache
        );

        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
    }

    #[tokio::test]
    async fn workspace_service_rebuilds_each_accepted_input_with_one_bounded_follow_up() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let calls = Arc::new(AtomicUsize::new(0));
        let (second_started_sender, second_started) = std::sync::mpsc::channel();
        let (second_release, second_release_receiver) = std::sync::mpsc::channel();
        let detector = GatedDetector {
            calls: Arc::clone(&calls),
            second_started: second_started_sender,
            second_release: Arc::new(Mutex::new(second_release_receiver)),
        };
        let (_ticks, controlled_ticks) = mpsc::channel(8);
        let service =
            WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(detector))
                .with_controlled_change_ticks(controlled_ticks);
        let input = service.change_input_handle();
        assert_eq!(
            input.submit(explicit_change("first.bsl")),
            WorkspaceChangeSubmissionOutcome::Accepted
        );
        let updates = service.update_observer();
        let mut update_changes = updates.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));
        timeout(
            Duration::from_secs(1),
            tokio::task::spawn_blocking(move || second_started.recv()),
        )
        .await
        .expect("input rebuild must start")
        .expect("input-build observer must join")
        .expect("input-build start must be observed");
        assert_eq!(
            input.submit(explicit_change("second.bsl")),
            WorkspaceChangeSubmissionOutcome::Accepted
        );
        assert_eq!(
            input.submit(explicit_change("third.bsl")),
            WorkspaceChangeSubmissionOutcome::Backpressure
        );
        second_release
            .send(())
            .expect("input rebuild must be released");

        let followed_up = wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 3
        })
        .await;
        assert_eq!(followed_up.attempt(), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
        assert_eq!(
            input.submit(explicit_change("after-shutdown.bsl")),
            WorkspaceChangeSubmissionOutcome::Closed
        );
    }

    #[tokio::test]
    async fn workspace_service_cancellation_joins_input_build_and_closes_pending_slot() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let calls = Arc::new(AtomicUsize::new(0));
        let (second_started_sender, second_started) = std::sync::mpsc::channel();
        let (second_release, second_release_receiver) = std::sync::mpsc::channel();
        let detector = GatedDetector {
            calls: Arc::clone(&calls),
            second_started: second_started_sender,
            second_release: Arc::new(Mutex::new(second_release_receiver)),
        };
        let (_ticks, controlled_ticks) = mpsc::channel(8);
        let service =
            WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(detector))
                .with_controlled_change_ticks(controlled_ticks);
        let input = service.change_input_handle();
        let updates = service.update_observer();
        let mut update_changes = updates.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let mut run = tokio::spawn(app.run(shutdown));
        wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching
        })
        .await;

        assert_eq!(
            input.submit(explicit_change("active.bsl")),
            WorkspaceChangeSubmissionOutcome::Accepted
        );
        timeout(
            Duration::from_secs(1),
            tokio::task::spawn_blocking(move || second_started.recv()),
        )
        .await
        .expect("input rebuild must start")
        .expect("input-build observer must join")
        .expect("input-build start must be observed");
        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), async {
            loop {
                match input.submit(explicit_change("after-cancellation.bsl")) {
                    WorkspaceChangeSubmissionOutcome::Closed => break,
                    WorkspaceChangeSubmissionOutcome::Accepted
                    | WorkspaceChangeSubmissionOutcome::Backpressure => {
                        tokio::task::yield_now().await;
                    }
                    WorkspaceChangeSubmissionOutcome::IgnoredEmpty => {
                        panic!("non-empty test input must not be ignored")
                    }
                }
            }
        })
        .await
        .expect("input receiver must close before the active build is released");
        assert!(
            timeout(Duration::from_millis(50), &mut run).await.is_err(),
            "shutdown must join the active complete rebuild"
        );
        second_release
            .send(())
            .expect("input rebuild must be released");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang after release")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(updates.status().phase(), WorkspaceUpdatePhase::Stopped);
        assert_eq!(
            input.submit(explicit_change("closed.bsl")),
            WorkspaceChangeSubmissionOutcome::Closed
        );
    }

    #[tokio::test]
    async fn workspace_service_coalesces_changes_during_one_serialized_build() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let calls = Arc::new(AtomicUsize::new(0));
        let (second_started_sender, second_started) = std::sync::mpsc::channel();
        let (second_release, second_release_receiver) = std::sync::mpsc::channel();
        let detector = GatedDetector {
            calls: Arc::clone(&calls),
            second_started: second_started_sender,
            second_release: Arc::new(Mutex::new(second_release_receiver)),
        };
        let (ticks, controlled_ticks) = mpsc::channel(8);
        let service =
            WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(detector))
                .with_controlled_change_ticks(controlled_ticks);
        let updates = service.update_observer();
        let mut update_changes = updates.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));
        wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching
        })
        .await;

        fs::write(root.path().join("first.bsl"), b"first").expect("first change must be written");
        scan_once(&ticks).await;
        timeout(
            Duration::from_secs(1),
            tokio::task::spawn_blocking(move || second_started.recv()),
        )
        .await
        .expect("second build must start")
        .expect("second-build observer must join")
        .expect("second-build start must be observed");

        fs::write(root.path().join("second.bsl"), b"second")
            .expect("second change must be written");
        scan_once(&ticks).await;
        fs::write(root.path().join("third.bsl"), b"third").expect("third change must be written");
        scan_once(&ticks).await;
        second_release
            .send(())
            .expect("second build must be released");

        let coalesced = wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching && status.published() == 3
        })
        .await;
        assert_eq!(coalesced.attempt(), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);

        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
    }

    #[tokio::test]
    async fn workspace_service_cancellation_joins_blocking_rebuild_and_closes_observers() {
        let root = tempdir().expect("temporary Workspace root must be created");
        let calls = Arc::new(AtomicUsize::new(0));
        let (second_started_sender, second_started) = std::sync::mpsc::channel();
        let (second_release, second_release_receiver) = std::sync::mpsc::channel();
        let detector = GatedDetector {
            calls: Arc::clone(&calls),
            second_started: second_started_sender,
            second_release: Arc::new(Mutex::new(second_release_receiver)),
        };
        let (ticks, controlled_ticks) = mpsc::channel(8);
        let service =
            WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(detector))
                .with_controlled_change_ticks(controlled_ticks);
        let snapshots = service.snapshot_observer();
        let mut snapshot_changes = snapshots.subscribe();
        let updates = service.update_observer();
        let mut update_changes = updates.subscribe();
        let cache = service.cache_observer();
        let mut cache_changes = cache.subscribe();
        let provider = TestConfigurationProvider {
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("workspace", service)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let mut run = tokio::spawn(app.run(shutdown));
        wait_for_update(&mut update_changes, |status| {
            status.phase() == WorkspaceUpdatePhase::Watching
        })
        .await;

        fs::write(root.path().join("changed.bsl"), b"changed")
            .expect("relevant source must be written");
        scan_once(&ticks).await;
        timeout(
            Duration::from_secs(1),
            tokio::task::spawn_blocking(move || second_started.recv()),
        )
        .await
        .expect("replacement build must start")
        .expect("replacement observer must join")
        .expect("replacement build start must be observed");

        shutdown_sender.send(()).expect("shutdown must be observed");
        assert!(
            timeout(Duration::from_millis(50), &mut run).await.is_err(),
            "shutdown must wait for the owned blocking rebuild"
        );
        assert!(snapshots.snapshot().is_some());
        second_release
            .send(())
            .expect("blocking rebuild must be released");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang after release")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");

        wait_for_snapshot_clear(&mut snapshot_changes).await;
        wait_for_watch_closed(&mut cache_changes).await;
        assert!(snapshots.snapshot().is_none());
        assert_eq!(updates.status().phase(), WorkspaceUpdatePhase::Stopped);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
