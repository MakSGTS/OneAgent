//! Immutable Workspace semantic snapshots and deterministic initial builds.

mod change;
mod graph_query;

pub use graph_query::{
    GraphQueryConfiguration, GraphQueryConfigurationList, GraphQueryDirection, GraphQueryEdgeKind,
    GraphQueryError, GraphQueryErrorKind, GraphQueryLimit, GraphQueryMaxDepth,
    GraphQueryMetadataKind, GraphQueryNode, GraphQueryNodeKind, GraphQueryNodeResult,
    GraphQueryRelation, GraphQueryRelationResult, GraphQueryService, GraphQueryTraversalNode,
    GraphQueryTraversalResult, GraphQueryWorkspaceFormat,
};

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use oneagent_common::{EntityId, EntityName};
use oneagent_designer_xml::{
    DesignerXmlBuildScope, DesignerXmlSemanticGraphBuilder,
    FileSystemDesignerXmlSemanticGraphBuilder,
};
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    NodeKind, SemanticDiagnostic, SemanticGraph, SemanticGraphReport,
    SemanticGraphValidationResult, SemanticReferenceRequestLedger, SemanticReferenceStatistics,
};
use oneagent_metadata::MetadataKind;
use oneagent_workspace::{DiscoveredConfiguration, WorkspaceDetector, WorkspaceFormat};
use oneagent_workspace_fs::FileSystemWorkspaceDetector;
use tokio::sync::watch;
use tokio::task::JoinError;

use crate::{BoxError, RuntimeService, ServiceContext, ServiceStartFuture, ServiceTask};
use change::{
    RunningWorkspaceChangeSource, WorkspaceChangeOutcome, WorkspaceChangeSource,
    WorkspaceChangeSourceError, WorkspaceFileState,
};

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

/// Cloneable transport-neutral observation of the published Workspace snapshot.
#[derive(Debug, Clone)]
pub struct WorkspaceSnapshotObserver {
    snapshot: watch::Receiver<Option<Arc<WorkspaceSnapshot>>>,
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

/// Runtime-owned service for one complete initial Workspace build.
#[derive(Debug)]
pub struct WorkspaceService<D = FileSystemWorkspaceDetector> {
    builder: WorkspaceSnapshotBuilder<D>,
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
        let (snapshot, _receiver) = watch::channel(None);
        let (updates, _receiver) = watch::channel(WorkspaceUpdateStatus::starting());
        Self {
            builder,
            snapshot,
            updates,
            #[cfg(test)]
            controlled_change_ticks: None,
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

    #[cfg(test)]
    fn with_controlled_change_ticks(
        mut self,
        ticks: tokio::sync::mpsc::Receiver<tokio::sync::oneshot::Sender<()>>,
    ) -> Self {
        self.controlled_change_ticks = Some(ticks);
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
                snapshot,
                updates,
                #[cfg(test)]
                controlled_change_ticks,
            } = *self;
            updates.send_replace(WorkspaceUpdateStatus {
                attempt: 1,
                published: 0,
                phase: WorkspaceUpdatePhase::Starting,
                failure: None,
            });

            let initial_root = root_path.clone();
            let initial_builder = builder.clone();
            let error_root = root_path.clone();
            let (baseline_state, initial_snapshot, post_build_state) =
                tokio::task::spawn_blocking(move || {
                    let baseline_state = WorkspaceFileState::scan(&initial_root);
                    let snapshot = initial_builder.build(&initial_root)?;
                    let baseline_state =
                        baseline_state.map_err(|source| WorkspaceBuildError::Observation {
                            root_path: initial_root.clone(),
                            source: Box::new(source),
                        })?;
                    let post_build_state = observe_workspace(&initial_root)?;
                    Ok::<_, WorkspaceBuildError>((baseline_state, snapshot, post_build_state))
                })
                .await
                .map_err(|source| WorkspaceBuildError::BuildTask {
                    root_path: error_root,
                    source,
                })?
                .map_err(|error| Box::new(error) as BoxError)?;
            snapshot.send_replace(Some(Arc::new(initial_snapshot)));
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
                    post_build_state.clone(),
                    ticks,
                )
            } else {
                WorkspaceChangeSource::new(root_path.clone(), post_build_state.clone())
            };
            #[cfg(not(test))]
            let source = WorkspaceChangeSource::new(root_path.clone(), post_build_state.clone());

            let cancellation = context.cancellation();
            let source = source
                .with_initial_change(baseline_state != post_build_state)
                .start(cancellation.clone());

            let task: ServiceTask = Box::pin(async move {
                run_workspace_updates(builder, root_path, snapshot, updates, cancellation, source)
                    .await
                    .map_err(|error| Box::new(error) as BoxError)
            });
            Ok(task)
        })
    }
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
        }
    }
}

impl Error for WorkspaceUpdateRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ChangeSource(error) => Some(error),
            Self::ChangeSourceTask(error) => Some(error),
            Self::ChangeSourceStopped | Self::StatusCounterOverflow => None,
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the select loop keeps source, build, cancellation, and publication ownership together"
)]
async fn run_workspace_updates<D>(
    builder: WorkspaceSnapshotBuilder<D>,
    root_path: PathBuf,
    snapshot: watch::Sender<Option<Arc<WorkspaceSnapshot>>>,
    updates: watch::Sender<WorkspaceUpdateStatus>,
    mut cancellation: crate::Cancellation,
    source: RunningWorkspaceChangeSource,
) -> Result<(), WorkspaceUpdateRuntimeError>
where
    D: WorkspaceDetector + Clone + Send + 'static,
{
    let (mut observations, mut source_task) = source.into_parts();
    let mut processed_revision = 0_u64;
    let mut status = *updates.borrow();

    loop {
        let observation = *observations.borrow_and_update();
        if observation.revision() > processed_revision {
            processed_revision = observation.revision();
            match observation.outcome() {
                Some(WorkspaceChangeOutcome::Changed) => {
                    status.attempt = status
                        .attempt
                        .checked_add(1)
                        .ok_or(WorkspaceUpdateRuntimeError::StatusCounterOverflow)?;
                    status.phase = WorkspaceUpdatePhase::Rebuilding;
                    status.failure = None;
                    updates.send_replace(status);

                    let build_root = root_path.clone();
                    let build_builder = builder.clone();
                    let mut build =
                        tokio::task::spawn_blocking(move || build_builder.build(&build_root));
                    let build_result = tokio::select! {
                        biased;
                        () = cancellation.cancelled() => {
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
                            snapshot.send_replace(Some(Arc::new(rebuilt)));
                            status.published = status
                                .published
                                .checked_add(1)
                                .ok_or(WorkspaceUpdateRuntimeError::StatusCounterOverflow)?;
                            status.phase = WorkspaceUpdatePhase::Watching;
                            status.failure = None;
                        }
                        Ok(Err(error)) => {
                            status.phase = WorkspaceUpdatePhase::Failed;
                            status.failure = Some(error.kind().into());
                        }
                        Err(_) => {
                            status.phase = WorkspaceUpdatePhase::Failed;
                            status.failure = Some(WorkspaceUpdateFailureKind::BuildTask);
                        }
                    }
                    updates.send_replace(status);
                    continue;
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

        tokio::select! {
            biased;
            () = cancellation.cancelled() => {
                let source_result = source_task.await;
                return finish_workspace_updates(&snapshot, &updates, source_result, true);
            }
            source_result = &mut source_task => {
                return finish_workspace_updates(
                    &snapshot,
                    &updates,
                    source_result,
                    false,
                );
            }
            changed = observations.changed() => {
                if changed.is_err() {
                    let source_result = source_task.await;
                    return finish_workspace_updates(
                        &snapshot,
                        &updates,
                        source_result,
                        false,
                    );
                }
            }
        }
    }
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
}

/// Complete immutable semantic state for one configured Workspace root.
#[derive(Debug, Default, Clone)]
pub struct WorkspaceSnapshot {
    configurations: Vec<WorkspaceConfigurationSnapshot>,
}

impl WorkspaceSnapshot {
    /// Returns configuration snapshots in canonical Configuration identity order.
    #[must_use]
    pub fn configurations(&self) -> &[WorkspaceConfigurationSnapshot] {
        &self.configurations
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

        Ok(WorkspaceSnapshot {
            configurations: configurations.into_values().collect(),
        })
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
    )
}

fn build_designer_xml(
    project: &DiscoveredConfiguration,
) -> Result<WorkspaceConfigurationSnapshot, WorkspaceBuildError> {
    let root_path = project.root_path();
    let graph = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph(root_path, DesignerXmlBuildScope::Complete)
        .map_err(|source| semantic_build_error(root_path, WorkspaceFormat::DesignerXml, source))?;
    let validation = graph.validate();
    if !validation.is_valid() {
        return Err(WorkspaceBuildError::GraphValidation {
            root_path: root_path.to_path_buf(),
            format: WorkspaceFormat::DesignerXml,
            validation: Box::new(validation),
        });
    }
    let report = graph.report();

    snapshot_from_parts(
        root_path,
        WorkspaceFormat::DesignerXml,
        graph,
        Vec::new(),
        SemanticReferenceRequestLedger::new(),
        SemanticReferenceStatistics::new(),
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
) -> Result<WorkspaceConfigurationSnapshot, WorkspaceBuildError> {
    let (configuration_id, configuration_name) =
        configuration_identity(&graph).map_err(|actual| {
            WorkspaceBuildError::InvalidConfigurationCardinality {
                root_path: root_path.to_path_buf(),
                format,
                actual,
            }
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
    })
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
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use oneagent_workspace::WorkspaceFormat;
    use tempfile::tempdir;
    use tokio::sync::{mpsc, oneshot, watch};
    use tokio::time::timeout;

    use crate::{App, ConfigurationProvider, LifecycleState, RuntimeConfig, RuntimeErrorKind};

    use super::{
        DiscoveredConfiguration, WorkspaceBuildErrorKind, WorkspaceDetector, WorkspaceService,
        WorkspaceSnapshot, WorkspaceSnapshotBuilder, WorkspaceUpdateFailureKind,
        WorkspaceUpdatePhase, WorkspaceUpdateStatus,
    };

    const DUMP_INFO: &str = r#"<ConfigDumpInfo xmlns="http://v8.1c.ru/8.3/xcf/dumpinfo" format="Hierarchical" version="2.20"><ConfigVersions /></ConfigDumpInfo>"#;

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
        assert!(first.configurations()[1].diagnostics().is_empty());
        assert!(first.configurations()[1].reference_requests().is_empty());
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

    #[tokio::test]
    async fn workspace_service_reports_named_start_failure_without_publication() {
        let parent = tempdir().expect("temporary parent must be created");
        let missing = parent.path().join("missing");
        let service = WorkspaceService::new();
        let observer = service.snapshot_observer();
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
        assert_eq!(source.kind(), WorkspaceBuildErrorKind::DiscoveryFailed);
        assert!(observer.snapshot().is_none());
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

        let before_ignored_change = updates.status();
        fs::write(root.path().join(".git/transient"), b"ignored")
            .expect("ignored source must be created");
        scan_once(&ticks).await;
        assert_eq!(updates.status(), before_ignored_change);

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

        shutdown_sender.send(()).expect("shutdown must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("Workspace shutdown must not hang")
            .expect("Workspace task must join")
            .expect("requested shutdown must succeed");
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
}
