//! Service-bound, one-use cooperative edit transactions.

use super::edit_io::{
    EditBaseline, EditIo, EditIoBudget, EditIoError, MAX_BASELINE, MAX_DOCUMENT, MAX_EDITED,
    MAX_FILES, MAX_OPERATIONS,
};
use super::{WorkspaceSnapshot, WorkspaceSnapshotBuilder, compose_change_impact};
use oneagent_analysis::publication::WorkspacePublicationId;
use oneagent_analysis::refactoring::{
    NeverCancelledRefactoring, PlanId, RefactoringPlan, RefactoringPreview, RefactoringRequest,
};
use oneagent_analysis::safe_edit::{
    SafeEditEvidence, SafeEditModuleInput, SafeEditProducerInput, SafeEditProducerProjection,
    SafeEditProjectionAdmission, compare_plan, replacement_bytes, validate_equivalence,
    validate_postconditions,
};
use oneagent_tool_policy::{
    ActorId, AuthorizationDecisionKind, ToolArguments, ToolAuthorization, ToolConfirmation,
    ToolConfirmationChallenge, ToolEffect, ToolId, ToolPolicy, ToolRequest, ToolRequestId,
    ToolTerminalOutcome, execute_tool,
};
use oneagent_workspace::WorkspaceDetector;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot, watch};

/// Explicit caller commitment to exclude external writers and linked aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceEditOwnership {
    /// Cooperative ownership covers the complete hierarchy through recovery.
    ExclusiveCooperative,
}

/// Closed redacted transaction cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceEditCause {
    /// Edits are disabled, unsupported, unready or lack a complete baseline.
    Unavailable,
    /// Service admission has stopped.
    Stopped,
    /// Another preparation, transaction or publication writer owns admission.
    Busy,
    /// An inclusive transaction resource bound would be exceeded.
    BoundsExceeded,
    /// A capability belongs to another service, attempt or direction.
    AuthorizationMismatch,
    /// The immutable policy denied this request.
    PolicyDenied,
    /// Bare allow or missing confirmation cannot authorize an edit.
    ConfirmationRequired,
    /// Cancellation won before commit.
    Cancelled,
    /// The exact predecessor publication is no longer current.
    PublicationMismatch,
    /// Complete regenerated planning evidence differs.
    PlanMismatch,
    /// The complete source state changed.
    SourceChanged,
    /// Confinement, regular-file identity or cooperative ownership is unprovable.
    ConfinementUnverifiable,
    /// A checked I/O step failed.
    IoFailed,
    /// Complete production rebuild or semantic comparison failed.
    SemanticMismatch,
    /// Recovery failed; current observation is quarantined.
    RecoveryRequired,
}

impl Display for WorkspaceEditCause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "workspace edit failed: {self:?}")
    }
}
impl std::error::Error for WorkspaceEditCause {}
impl From<EditIoError> for WorkspaceEditCause {
    fn from(value: EditIoError) -> Self {
        match value {
            EditIoError::Bounds => Self::BoundsExceeded,
            EditIoError::Confinement => Self::ConfinementUnverifiable,
            EditIoError::Changed => Self::SourceChanged,
            EditIoError::Io => Self::IoFailed,
        }
    }
}
type Result<T> = std::result::Result<T, WorkspaceEditCause>;

/// Recovery disposition for an unsuccessful attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceEditRecovery {
    /// No source replacement was attempted.
    NotNeeded,
    /// Exact original state and cleanup were verified.
    Recovered,
    /// Bounded material is retained and the service is quarantined.
    Required,
}

/// Closed outcome delivered only after the service-owned attempt terminates.
#[derive(Debug)]
pub enum WorkspaceEditOutcome {
    /// A single canonical successor was committed.
    Applied {
        /// Exact predecessor identity.
        previous: WorkspacePublicationId,
        /// Exact committed successor identity.
        current: WorkspacePublicationId,
        /// Canonical original plan identity.
        plan: PlanId,
        /// Number of touched regular files.
        files: usize,
        /// Number of exact identifier replacements.
        operations: usize,
        /// A separately confirmable inverse, present only for apply.
        reversal: Option<WorkspaceEditReceipt>,
    },
    /// No successful successor was published.
    Failed {
        /// Primary closed failure, with recovery failure taking precedence.
        cause: WorkspaceEditCause,
        /// Triggering failure when recovery took precedence.
        secondary: Option<WorkspaceEditCause>,
        /// Verified recovery disposition.
        recovery: WorkspaceEditRecovery,
        /// Number of owned artifacts still retained.
        retained_files: usize,
    },
}

impl WorkspaceEditOutcome {
    fn failure(cause: WorkspaceEditCause) -> Self {
        Self::Failed {
            cause,
            secondary: None,
            recovery: WorkspaceEditRecovery::NotNeeded,
            retained_files: 0,
        }
    }
}

/// Cancellation survives dropping the response future.
#[derive(Debug, Clone)]
pub struct WorkspaceEditCancellation {
    signal: watch::Sender<bool>,
}
impl WorkspaceEditCancellation {
    /// Creates an independent, initially unrequested cancellation token.
    #[must_use]
    pub fn new() -> Self {
        Self {
            signal: watch::channel(false).0,
        }
    }
    /// Requests cancellation; recovery is never interrupted by this token.
    pub fn request(&self) {
        self.signal.send_replace(true);
    }
    fn requested(&self) -> bool {
        *self.signal.borrow()
    }
}
impl Default for WorkspaceEditCancellation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Apply,
    Reverse,
}

struct EditServiceIdentity;
// These are independent lifecycle observations; precedence handles simultaneous
// poison, shutdown and writer state under the same mutex.
#[allow(clippy::struct_excessive_bools)]
struct Admission {
    ready: bool,
    stopped: bool,
    poisoned: bool,
    writer: bool,
    next: u64,
    slot: Option<u64>,
}
struct Shared {
    identity: Arc<EditServiceIdentity>,
    admission: Mutex<Admission>,
    commands: mpsc::Sender<EditCommand>,
}

pub(super) struct Reservation {
    shared: Arc<Shared>,
    id: u64,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if let Ok(mut state) = self.shared.admission.lock()
            && state.slot == Some(self.id)
        {
            state.slot = None;
        }
    }
}

/// Cloneable local request endpoint; it carries no pre-start edit authority.
#[derive(Clone)]
pub struct WorkspaceEditHandle {
    shared: Arc<Shared>,
}
impl Debug for WorkspaceEditHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceEditHandle")
            .finish_non_exhaustive()
    }
}

impl WorkspaceEditHandle {
    fn reserve_attempt(&self) -> Result<Reservation> {
        let mut state = self
            .shared
            .admission
            .lock()
            .map_err(|_| WorkspaceEditCause::Unavailable)?;
        if state.poisoned {
            return Err(WorkspaceEditCause::RecoveryRequired);
        }
        if state.stopped {
            return Err(WorkspaceEditCause::Stopped);
        }
        if !state.ready {
            return Err(WorkspaceEditCause::Unavailable);
        }
        if state.writer || state.slot.is_some() {
            return Err(WorkspaceEditCause::Busy);
        }
        state.next = state
            .next
            .checked_add(1)
            .ok_or(WorkspaceEditCause::BoundsExceeded)?;
        let id = state.next;
        state.slot = Some(id);
        Ok(Reservation {
            shared: Arc::clone(&self.shared),
            id,
        })
    }

    /// Prepares the current complete production plan and exact confirmation.
    ///
    /// # Errors
    /// Fails closed for unavailable, busy, stale, over-bound or unauthorized input.
    pub async fn prepare_apply(
        &self,
        request: RefactoringRequest,
        actor: ActorId,
        request_id: ToolRequestId,
    ) -> Result<(WorkspaceEditChallenge, RefactoringPreview)> {
        let reservation = self.reserve_attempt()?;
        let (response, receive) = oneshot::channel();
        self.shared
            .commands
            .try_send(EditCommand::Prepare {
                reservation,
                input: PrepareInput::Apply(request),
                actor,
                request_id,
                response,
            })
            .map_err(|_| WorkspaceEditCause::Busy)?;
        receive.await.map_err(|_| WorkspaceEditCause::Stopped)?
    }

    /// Consumes the retained receipt and requests separate reversal confirmation.
    ///
    /// # Errors
    /// Denial consumes the receipt; stale and foreign receipts cannot be reused.
    pub async fn prepare_reversal(
        &self,
        receipt: WorkspaceEditReceipt,
        actor: ActorId,
        request_id: ToolRequestId,
    ) -> Result<(WorkspaceEditChallenge, RefactoringPreview)> {
        let reservation = self.reserve_attempt()?;
        let (response, receive) = oneshot::channel();
        self.shared
            .commands
            .try_send(EditCommand::Prepare {
                reservation,
                input: PrepareInput::Reverse(receipt),
                actor,
                request_id,
                response,
            })
            .map_err(|_| WorkspaceEditCause::Busy)?;
        receive.await.map_err(|_| WorkspaceEditCause::Stopped)?
    }

    /// Consumes exact confirmed apply evidence and awaits the joined transaction.
    pub async fn checked_apply(
        &self,
        authorization: WorkspaceEditAuthorization,
        cancellation: WorkspaceEditCancellation,
    ) -> WorkspaceEditOutcome {
        self.submit(authorization, cancellation, Direction::Apply)
            .await
    }
    /// Consumes exact confirmed inverse evidence and awaits the joined transaction.
    pub async fn checked_reversal(
        &self,
        authorization: WorkspaceEditAuthorization,
        cancellation: WorkspaceEditCancellation,
    ) -> WorkspaceEditOutcome {
        self.submit(authorization, cancellation, Direction::Reverse)
            .await
    }
    async fn submit(
        &self,
        authorization: WorkspaceEditAuthorization,
        cancellation: WorkspaceEditCancellation,
        direction: Direction,
    ) -> WorkspaceEditOutcome {
        if !Arc::ptr_eq(
            &self.shared.identity,
            &authorization.attempt.reservation.shared.identity,
        ) || authorization.attempt.direction != direction
        {
            return WorkspaceEditOutcome::failure(WorkspaceEditCause::AuthorizationMismatch);
        }
        let (response, receive) = oneshot::channel();
        if self
            .shared
            .commands
            .try_send(EditCommand::Submit {
                authorization,
                cancellation,
                response,
            })
            .is_err()
        {
            return WorkspaceEditOutcome::failure(WorkspaceEditCause::Stopped);
        }
        receive
            .await
            .unwrap_or_else(|_| WorkspaceEditOutcome::failure(WorkspaceEditCause::Stopped))
    }
}

struct EditAttempt {
    actor: String,
    request_id: String,
    reservation: Reservation,
    direction: Direction,
    previous: Arc<WorkspaceSnapshot>,
    baseline: EditBaseline,
    plan: RefactoringPlan,
    policy: Option<ToolAuthorization>,
    undo: Option<EditUndo>,
    projection: Option<SafeEditProducerProjection>,
}

/// Non-cloneable explicit confirmation opportunity; drop releases its reservation.
pub struct WorkspaceEditChallenge {
    attempt: EditAttempt,
    confirmation: ToolConfirmationChallenge,
}
impl WorkspaceEditChallenge {
    /// Records explicit confirmation without performing source I/O.
    #[must_use]
    pub fn confirm(self) -> WorkspaceEditAuthorization {
        WorkspaceEditAuthorization {
            attempt: self.attempt,
            confirmation: self.confirmation.confirm(),
        }
    }
}
/// Non-cloneable, exact one-use service authorization.
pub struct WorkspaceEditAuthorization {
    attempt: EditAttempt,
    confirmation: ToolConfirmation,
}
/// Opaque receipt for the sole retained successful apply; never authorization.
pub struct WorkspaceEditReceipt {
    identity: Arc<EditServiceIdentity>,
    attempt: u64,
}

macro_rules! redacted_debug {
    ($($name:ty),+) => { $(impl Debug for $name {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.debug_struct(stringify!($name)).finish_non_exhaustive() }
    })+ };
}
redacted_debug!(
    WorkspaceEditChallenge,
    WorkspaceEditAuthorization,
    WorkspaceEditReceipt
);

struct EditUndo {
    attempt: u64,
    before: Arc<WorkspaceSnapshot>,
    applied: Arc<WorkspaceSnapshot>,
    baseline: EditBaseline,
    plan: RefactoringPlan,
    originals: BTreeMap<PathBuf, Arc<[u8]>>,
}
pub(super) enum PrepareInput {
    Apply(RefactoringRequest),
    Reverse(WorkspaceEditReceipt),
}
// The bounded one-slot channel transfers the complete owned attempt inline;
// boxing would add an unnecessary allocation at the admission boundary.
#[allow(clippy::large_enum_variant)]
pub(super) enum EditCommand {
    Prepare {
        reservation: Reservation,
        input: PrepareInput,
        actor: ActorId,
        request_id: ToolRequestId,
        response: oneshot::Sender<Result<(WorkspaceEditChallenge, RefactoringPreview)>>,
    },
    Submit {
        authorization: WorkspaceEditAuthorization,
        cancellation: WorkspaceEditCancellation,
        response: oneshot::Sender<WorkspaceEditOutcome>,
    },
}

struct EditPolicyGate;
impl oneagent_tool_policy::ToolExecutor for EditPolicyGate {
    fn execute<'a>(
        &'a self,
        _request: &'a ToolRequest,
        _cancellation: &'a dyn oneagent_tool_policy::ToolCancellationSignal,
    ) -> oneagent_tool_policy::ToolFuture<'a, oneagent_tool_policy::ToolExecutorOutcome> {
        Box::pin(async {
            oneagent_tool_policy::ToolExecutorOutcome::Completed(
                oneagent_tool_policy::ToolOutput::new("admitted")
                    .expect("fixed bounded gate result"),
            )
        })
    }
}

type PreparedEdit = Result<(WorkspaceEditChallenge, RefactoringPreview)>;
type PendingPreparation = (oneshot::Sender<PreparedEdit>, PreparedEdit);
/// Owned by the Workspace lifecycle; workers cannot publish.
pub(super) struct EditCoordinator {
    handle: WorkspaceEditHandle,
    pub(super) commands: mpsc::Receiver<EditCommand>,
    policy: Option<ToolPolicy>,
    baseline: Option<EditBaseline>,
    undo: Option<EditUndo>,
    recovery: Option<EditIo>,
    terminal: Option<(oneshot::Sender<WorkspaceEditOutcome>, WorkspaceEditOutcome)>,
    prepared: Option<PendingPreparation>,
    #[cfg(test)]
    test_hooks: TestHooks,
}

#[cfg(test)]
type PhaseGate = (&'static str, Box<dyn FnOnce() + Send + Sync>);
#[cfg(test)]
type CandidateMutation = Box<dyn FnOnce(&mut WorkspaceSnapshot) + Send + Sync>;
#[cfg(test)]
#[derive(Default)]
#[allow(clippy::struct_excessive_bools)] // Independent fault axes, not production states.
struct TestHooks {
    reversal_io_failures: Vec<(&'static str, usize)>,
    external_after_cleanup: bool,
    wrong_impact: bool,
    stale_predecessor: bool,
    restore_gate: Option<Box<dyn FnOnce() + Send + Sync>>,
    builder_failure: bool,
    external_bytes: Option<&'static str>,
    resolved_query: bool,
    phase_gate: Option<PhaseGate>,
    reversal_fail: Option<&'static str>,
    publication_overflow: bool,
    projection_overflow: bool,
    fail: Option<&'static str>,
    events: Vec<&'static str>,
    candidate: Option<CandidateMutation>,
    io_failures: Vec<(&'static str, usize)>,
    observed: Arc<Mutex<Vec<&'static str>>>,
}
impl Debug for EditCoordinator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditCoordinator")
            .field("enabled", &self.enabled())
            .finish_non_exhaustive()
    }
}

impl EditCoordinator {
    pub(super) fn new() -> Self {
        let (commands, receive) = mpsc::channel(1);
        let shared = Arc::new(Shared {
            identity: Arc::new(EditServiceIdentity),
            admission: Mutex::new(Admission {
                ready: false,
                stopped: false,
                poisoned: false,
                writer: false,
                next: 0,
                slot: None,
            }),
            commands,
        });
        Self {
            handle: WorkspaceEditHandle { shared },
            commands: receive,
            policy: None,
            baseline: None,
            undo: None,
            recovery: None,
            terminal: None,
            prepared: None,
            #[cfg(test)]
            test_hooks: TestHooks::default(),
        }
    }
    pub(super) fn configure(&mut self, policy: ToolPolicy, _ownership: WorkspaceEditOwnership) {
        self.policy = Some(policy);
    }
    pub(super) fn enabled(&self) -> bool {
        self.policy.is_some()
    }
    pub(super) fn handle(&self) -> WorkspaceEditHandle {
        self.handle.clone()
    }
    pub(super) fn poisoned(&self) -> bool {
        self.recovery.is_some()
    }
    pub(super) fn writer(&self, busy: bool) {
        self.handle
            .shared
            .admission
            .lock()
            .expect("admission mutex")
            .writer = busy;
    }
    pub(super) fn publish_baseline(&mut self, baseline: Option<EditBaseline>) {
        self.undo = None;
        self.baseline = baseline;
        let mut admission = self
            .handle
            .shared
            .admission
            .lock()
            .expect("admission mutex");
        admission.ready = self.enabled() && self.baseline.is_some();
        admission.slot = None;
        admission.writer = false;
    }
    pub(super) fn shutdown(&mut self) {
        self.commands.close();
        self.undo = None;
        self.baseline = None;
        let mut admission = self
            .handle
            .shared
            .admission
            .lock()
            .expect("admission mutex");
        admission.stopped = true;
        admission.ready = false;
        admission.slot = None;
    }
    pub(super) fn unchanged(&self) -> impl std::future::Future<Output = bool> + Send + use<> {
        let baseline = self.baseline.clone();
        async move {
            tokio::task::spawn_blocking(move || {
                baseline.as_ref().is_some_and(|before| {
                    EditBaseline::capture(before.root(), &[])
                        .is_ok_and(|after| before.equals(&after))
                })
            })
            .await
            .unwrap_or(false)
        }
    }

    #[allow(clippy::too_many_lines)] // Keep the normative ordered admission together.
    fn prepare(
        &mut self,
        reservation: Reservation,
        input: PrepareInput,
        actor: ActorId,
        request_id: ToolRequestId,
        previous: Arc<WorkspaceSnapshot>,
    ) -> Result<(WorkspaceEditChallenge, RefactoringPreview)> {
        let baseline = self
            .baseline
            .clone()
            .ok_or(WorkspaceEditCause::Unavailable)?;
        let (direction, plan, preview, undo) = match input {
            PrepareInput::Apply(request) => {
                self.undo = None;
                let evaluated = previous
                    .plan_refactoring(&request, &NeverCancelledRefactoring)
                    .map_err(|_| WorkspaceEditCause::PlanMismatch)?;
                let (plan, preview) = evaluated.into_parts();
                (Direction::Apply, plan, preview, None)
            }
            PrepareInput::Reverse(receipt) => {
                let undo = self
                    .undo
                    .take()
                    .ok_or(WorkspaceEditCause::AuthorizationMismatch)?;
                if !Arc::ptr_eq(&receipt.identity, &self.handle.shared.identity)
                    || receipt.attempt != undo.attempt
                    || !Arc::ptr_eq(&previous, &undo.applied)
                {
                    return Err(WorkspaceEditCause::AuthorizationMismatch);
                }
                let preview = undo
                    .before
                    .plan_refactoring(undo.plan.request(), &NeverCancelledRefactoring)
                    .map_err(|_| WorkspaceEditCause::PlanMismatch)?
                    .into_parts()
                    .1;
                (Direction::Reverse, undo.plan.clone(), preview, Some(undo))
            }
        };
        check_admission(&plan, &previous, &baseline)?;
        let observed = EditBaseline::capture(baseline.root(), &[])?;
        if !baseline.equals(&observed) {
            return Err(WorkspaceEditCause::SourceChanged);
        }
        let tool_name = match direction {
            Direction::Apply => "oneagent.workspace.edit.apply",
            Direction::Reverse => "oneagent.workspace.edit.reverse",
        };
        let mut count = oneagent_analysis::safe_edit::SafeEditCountingSink::default();
        write_arguments(
            &mut count,
            reservation.id,
            tool_name,
            previous.publication_id(),
            &plan,
        )
        .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        if count.bytes() > oneagent_tool_policy::MAX_TOOL_ARGUMENT_BYTES {
            return Err(WorkspaceEditCause::BoundsExceeded);
        }
        let mut admission =
            SafeEditProjectionAdmission::new(0).map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        let actor_binding = admission
            .copy_string(actor.as_str())
            .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        let request_binding = admission
            .copy_string(request_id.as_str())
            .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        let arguments = admission
            .string(count.bytes(), |sink| {
                write_arguments(
                    sink,
                    reservation.id,
                    tool_name,
                    previous.publication_id(),
                    &plan,
                )
            })
            .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        let request = ToolRequest::new(
            request_id,
            actor,
            ToolId::new(tool_name).map_err(|_| WorkspaceEditCause::BoundsExceeded)?,
            ToolArguments::new(arguments).map_err(|_| WorkspaceEditCause::BoundsExceeded)?,
            [ToolEffect::LocalMutation],
        )
        .map_err(|_| WorkspaceEditCause::PolicyDenied)?;
        let mut policy = self
            .policy
            .as_ref()
            .ok_or(WorkspaceEditCause::Unavailable)?
            .evaluate(request);
        match policy.kind() {
            AuthorizationDecisionKind::Deny => return Err(WorkspaceEditCause::PolicyDenied),
            AuthorizationDecisionKind::Allow => {
                return Err(WorkspaceEditCause::ConfirmationRequired);
            }
            AuthorizationDecisionKind::RequireConfirmation => {}
        }
        let confirmation = policy
            .take_confirmation_challenge()
            .map_err(|_| WorkspaceEditCause::ConfirmationRequired)?;
        Ok((
            WorkspaceEditChallenge {
                attempt: EditAttempt {
                    actor: actor_binding,
                    request_id: request_binding,
                    reservation,
                    direction,
                    previous,
                    baseline,
                    plan,
                    policy: Some(policy),
                    undo,
                    projection: None,
                },
                confirmation,
            },
            preview,
        ))
    }
}

fn write_arguments(
    output: &mut (impl std::fmt::Write + ?Sized),
    attempt: u64,
    tool: &str,
    publication: WorkspacePublicationId,
    plan: &RefactoringPlan,
) -> std::fmt::Result {
    fn field(
        output: &mut (impl std::fmt::Write + ?Sized),
        value: impl Display,
    ) -> std::fmt::Result {
        use std::fmt::Write;
        let mut count = oneagent_analysis::safe_edit::SafeEditCountingSink::default();
        write!(&mut count, "{value}")?;
        write!(output, "{}:{value}", count.bytes())
    }
    field(output, attempt)?;
    field(output, tool)?;
    field(output, publication.get())?;
    field(output, plan.request().configuration_id().as_str())?;
    field(output, plan.request().target_node_id().as_str())?;
    field(output, plan.id().as_str())
}

fn policy_matches(
    attempt: &EditAttempt,
    policy: &ToolAuthorization,
    configured: Option<&ToolPolicy>,
) -> bool {
    struct Match<'a>(&'a str);
    impl std::fmt::Write for Match<'_> {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            self.0 = self.0.strip_prefix(value).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    let tool = if attempt.direction == Direction::Apply {
        "oneagent.workspace.edit.apply"
    } else {
        "oneagent.workspace.edit.reverse"
    };
    let request = policy.request();
    let mut sink = Match(request.arguments().expose());
    configured.is_some_and(|configured| configured.revision() == policy.policy_revision())
        && request.actor().as_str() == attempt.actor
        && request.id().as_str() == attempt.request_id
        && request.tool().as_str() == tool
        && request.effects().len() == 1
        && request.effects().contains(&ToolEffect::LocalMutation)
        && write_arguments(
            &mut sink,
            attempt.reservation.id,
            tool,
            attempt.previous.publication_id(),
            &attempt.plan,
        )
        .is_ok()
        && sink.0.is_empty()
}

fn check_admission(
    plan: &RefactoringPlan,
    snapshot: &WorkspaceSnapshot,
    baseline: &EditBaseline,
) -> Result<()> {
    if plan.operations().len() > MAX_OPERATIONS
        || plan.preconditions().documents().len() > MAX_FILES
    {
        return Err(WorkspaceEditCause::BoundsExceeded);
    }
    let mut total = 0usize;
    for configuration in snapshot.configurations() {
        let relative_root = configuration
            .root_path()
            .strip_prefix(snapshot.root_path())
            .map_err(|_| WorkspaceEditCause::ConfinementUnverifiable)?;
        if !baseline.contains_directory(relative_root) {
            return Err(WorkspaceEditCause::ConfinementUnverifiable);
        }
        for document in configuration.source_evidence().documents() {
            let bytes = baseline.bytes(Path::new(document.path().path().as_str()))?;
            if bytes.as_ref() != document.raw_content() {
                return Err(WorkspaceEditCause::SourceChanged);
            }
            if plan
                .preconditions()
                .documents()
                .iter()
                .any(|p| p.document_id() == document.id())
            {
                if bytes.len() > MAX_DOCUMENT {
                    return Err(WorkspaceEditCause::BoundsExceeded);
                }
                total = total
                    .checked_add(bytes.len())
                    .ok_or(WorkspaceEditCause::BoundsExceeded)?;
            }
        }
    }
    if total > MAX_EDITED {
        return Err(WorkspaceEditCause::BoundsExceeded);
    }
    EditIoBudget::buffers(&[baseline.raw_bytes(), MAX_BASELINE * 2, MAX_EDITED * 2])?;
    Ok(())
}

fn evidence(configuration: &super::WorkspaceConfigurationSnapshot) -> SafeEditEvidence<'_> {
    SafeEditEvidence {
        root: configuration.root_path(),
        graph: configuration.graph(),
        sources: configuration.source_evidence(),
        diagnostics: configuration.diagnostics(),
        references: configuration.reference_requests(),
        reference_statistics: configuration.reference_statistics(),
        report: configuration.report(),
        validation: configuration.validation(),
        rules: configuration.rule_execution_report(),
        findings: configuration.diagnostic_report(),
    }
}

fn freeze_projection(
    plan: &RefactoringPlan,
    snapshot: &WorkspaceSnapshot,
    results: &BTreeMap<PathBuf, Arc<[u8]>>,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<SafeEditProducerProjection> {
    let configuration = snapshot
        .configuration(plan.request().configuration_id())
        .ok_or(WorkspaceEditCause::PlanMismatch)?;
    let documents = configuration.source_evidence().documents();
    let root = snapshot
        .root_path()
        .to_str()
        .ok_or(WorkspaceEditCause::ConfinementUnverifiable)?;
    let mut paths = admission
        .vector::<oneagent_common::SourcePath>(documents.len())
        .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
    let mut path_bytes = 0usize;
    for document in documents {
        let relative = document.path().path().as_str();
        let length = root
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_add(relative.len()))
            .ok_or(WorkspaceEditCause::BoundsExceeded)?;
        let value = admission
            .string(length, |sink| write!(sink, "{root}/{relative}"))
            .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        admission
            .reserve(length)
            .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        let path = oneagent_common::SourcePath::new(value)
            .map_err(|_| WorkspaceEditCause::ConfinementUnverifiable)?;
        admission
            .release(length)
            .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
        path_bytes = path_bytes
            .checked_add(length)
            .ok_or(WorkspaceEditCause::BoundsExceeded)?;
        paths.push(path);
    }
    let mut modules = admission
        .vector::<SafeEditModuleInput<'_>>(documents.len())
        .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
    for (document, path) in documents.iter().zip(&paths) {
        let module = configuration
            .graph()
            .node(document.id().module_id())
            .ok_or(WorkspaceEditCause::SemanticMismatch)?;
        let owner = configuration
            .graph()
            .edges()
            .find(|edge| {
                edge.kind() == oneagent_graph::EdgeKind::Contains && edge.target() == module.id()
            })
            .ok_or(WorkspaceEditCause::SemanticMismatch)?
            .source();
        let result = results
            .get(Path::new(document.path().path().as_str()))
            .map_or(document.raw_content(), AsRef::as_ref);
        modules.push(
            SafeEditModuleInput::new(document, owner, module.name(), path, result)
                .map_err(|_| WorkspaceEditCause::SemanticMismatch)?,
        );
    }
    let input = SafeEditProducerInput::new(
        snapshot.publication_id(),
        snapshot.root_path(),
        configuration.root_path(),
        &configuration.graph,
        &configuration.source_evidence,
        &configuration.diagnostics,
        &configuration.reference_requests,
        plan,
        &modules,
    )
    .map_err(|_| WorkspaceEditCause::SemanticMismatch)?;
    let result = match configuration.format() {
        oneagent_workspace::WorkspaceFormat::Edt => {
            oneagent_edt::project_safe_edit_provenance(input, admission)
        }
        oneagent_workspace::WorkspaceFormat::DesignerXml => {
            oneagent_designer_xml::project_safe_edit_provenance(input, admission)
        }
        _ => return Err(WorkspaceEditCause::Unavailable),
    }
    .map_err(|error| match error {
        oneagent_analysis::safe_edit::SafeEditError::ProjectionBounds => {
            WorkspaceEditCause::BoundsExceeded
        }
        _ => WorkspaceEditCause::SemanticMismatch,
    });
    let scratch = path_bytes
        .checked_add(paths.capacity() * std::mem::size_of::<oneagent_common::SourcePath>())
        .and_then(|n| {
            n.checked_add(modules.capacity() * std::mem::size_of::<SafeEditModuleInput<'_>>())
        })
        .ok_or(WorkspaceEditCause::BoundsExceeded)?;
    drop(modules);
    drop(paths);
    admission
        .release(scratch)
        .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
    result
}

fn compare_snapshot(
    plan: &RefactoringPlan,
    before: &WorkspaceSnapshot,
    after: &WorkspaceSnapshot,
    projection: &SafeEditProducerProjection,
) -> Result<()> {
    if before.root_path() != after.root_path() || before.len() != after.len() {
        return Err(WorkspaceEditCause::SemanticMismatch);
    }
    for (before, after) in before.configurations().iter().zip(after.configurations()) {
        if before.configuration_id() != after.configuration_id()
            || before.configuration_name() != after.configuration_name()
            || before.format() != after.format()
        {
            return Err(WorkspaceEditCause::SemanticMismatch);
        }
        if before.configuration_id() == plan.request().configuration_id() {
            validate_postconditions(plan, &evidence(before), &evidence(after), projection)
                .map_err(|_| WorkspaceEditCause::SemanticMismatch)?;
        } else {
            validate_equivalence(&evidence(before), &evidence(after))
                .map_err(|_| WorkspaceEditCause::SemanticMismatch)?;
        }
    }
    Ok(())
}

pub(super) struct EditCommit {
    cancellation: WorkspaceEditCancellation,
    attempt: EditAttempt,
    pub(super) candidate: Arc<WorkspaceSnapshot>,
    baseline: EditBaseline,
    io: EditIo,
    response: oneshot::Sender<WorkspaceEditOutcome>,
}

struct GateCancellation<'a> {
    service: &'a crate::Cancellation,
    edit: &'a WorkspaceEditCancellation,
}
impl oneagent_tool_policy::ToolCancellationSignal for GateCancellation<'_> {
    fn is_cancelled(&self) -> bool {
        self.service.is_requested() || self.edit.requested()
    }
    fn cancelled(&self) -> oneagent_tool_policy::ToolFuture<'_, ()> {
        Box::pin(async {
            let mut service = self.service.clone();
            let mut edit = self.edit.signal.subscribe();
            tokio::select! { () = service.cancelled() => {}, _ = edit.changed() => {} }
        })
    }
}

impl EditCoordinator {
    /// Runs only in a joined blocking worker and never owns a snapshot sender.
    #[allow(clippy::too_many_lines)] // One coordinator owns admission and terminal transfer.
    pub(super) fn execute<D: WorkspaceDetector>(
        &mut self,
        command: EditCommand,
        previous: Option<Arc<WorkspaceSnapshot>>,
        builder: &WorkspaceSnapshotBuilder<D>,
        root: &Path,
        service: &crate::Cancellation,
    ) -> Option<EditCommit> {
        match command {
            EditCommand::Prepare {
                reservation,
                input,
                actor,
                request_id,
                response,
            } => {
                let result = previous
                    .ok_or(WorkspaceEditCause::Unavailable)
                    .and_then(|previous| {
                        self.prepare(reservation, input, actor, request_id, previous)
                    });
                self.prepared = Some((response, result));
                None
            }
            EditCommand::Submit {
                mut authorization,
                cancellation,
                response,
            } => {
                let guard = (|| {
                    let admission = self
                        .handle
                        .shared
                        .admission
                        .lock()
                        .map_err(|_| WorkspaceEditCause::Unavailable)?;
                    if admission.poisoned {
                        return Err(WorkspaceEditCause::RecoveryRequired);
                    }
                    if admission.stopped {
                        return Err(WorkspaceEditCause::Stopped);
                    }
                    if admission.slot != Some(authorization.attempt.reservation.id)
                        || !Arc::ptr_eq(
                            &self.handle.shared.identity,
                            &authorization.attempt.reservation.shared.identity,
                        )
                    {
                        return Err(WorkspaceEditCause::AuthorizationMismatch);
                    }
                    Ok(())
                })();
                if let Err(cause) = guard {
                    self.terminal = Some((response, WorkspaceEditOutcome::failure(cause)));
                    return None;
                }
                let Some(policy) = authorization.attempt.policy.take() else {
                    self.terminal = Some((
                        response,
                        WorkspaceEditOutcome::failure(WorkspaceEditCause::AuthorizationMismatch),
                    ));
                    return None;
                };
                let gate_cancellation = GateCancellation {
                    service,
                    edit: &cancellation,
                };
                if !policy_matches(&authorization.attempt, &policy, self.policy.as_ref()) {
                    self.terminal = Some((
                        response,
                        WorkspaceEditOutcome::failure(WorkspaceEditCause::AuthorizationMismatch),
                    ));
                    return None;
                }
                let gate = tokio::runtime::Handle::current().block_on(execute_tool(
                    policy,
                    Some(authorization.confirmation),
                    &EditPolicyGate,
                    &gate_cancellation,
                ));
                if gate.audit().terminal_outcome() != ToolTerminalOutcome::Completed {
                    let cause = if gate.audit().terminal_outcome() == ToolTerminalOutcome::Cancelled
                    {
                        WorkspaceEditCause::Cancelled
                    } else {
                        WorkspaceEditCause::ConfirmationRequired
                    };
                    self.terminal = Some((response, WorkspaceEditOutcome::failure(cause)));
                    return None;
                }
                if !previous
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &authorization.attempt.previous))
                {
                    self.terminal = Some((
                        response,
                        WorkspaceEditOutcome::failure(WorkspaceEditCause::PublicationMismatch),
                    ));
                    return None;
                }
                match self.run_attempt(
                    &mut authorization.attempt,
                    &cancellation,
                    service,
                    builder,
                    root,
                ) {
                    Ok((candidate, baseline, io)) => {
                        #[cfg(test)]
                        let candidate = {
                            let mut candidate = candidate;
                            if std::mem::take(&mut self.test_hooks.wrong_impact) {
                                candidate.change_impact =
                                    super::WorkspaceChangeImpact::NoPreviousPublication {
                                        current_publication_id: WorkspacePublicationId::initial(),
                                    };
                            }
                            if std::mem::take(&mut self.test_hooks.stale_predecessor) {
                                authorization.attempt.previous =
                                    Arc::new((*authorization.attempt.previous).clone());
                            }
                            candidate
                        };
                        Some(EditCommit {
                            cancellation: cancellation.clone(),
                            attempt: authorization.attempt,
                            candidate: Arc::new(candidate),
                            baseline,
                            io,
                            response,
                        })
                    }
                    Err(outcome) => {
                        self.terminal = Some((response, outcome));
                        None
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)] // Ordered transaction and its recovery share one owner.
    fn run_attempt<D: WorkspaceDetector>(
        &mut self,
        attempt: &mut EditAttempt,
        cancellation: &WorkspaceEditCancellation,
        service: &crate::Cancellation,
        builder: &WorkspaceSnapshotBuilder<D>,
        root: &Path,
    ) -> std::result::Result<(WorkspaceSnapshot, EditBaseline, EditIo), WorkspaceEditOutcome> {
        #[cfg(test)]
        struct ClearFaults;
        #[cfg(test)]
        impl Drop for ClearFaults {
            fn drop(&mut self) {
                super::edit_io::faults::set(Vec::new());
            }
        }
        #[cfg(test)]
        let _clear_faults = ClearFaults;
        let cancelled = || service.is_requested() || cancellation.requested();
        let mut edit_io = None;
        #[cfg(test)]
        if attempt.direction == Direction::Reverse {
            self.test_hooks.fail = self.test_hooks.reversal_fail.take();
            self.test_hooks.io_failures = std::mem::take(&mut self.test_hooks.reversal_io_failures);
        }
        let result = (|| -> Result<(WorkspaceSnapshot, EditBaseline)> {
            #[cfg(test)]
            if std::mem::take(&mut self.test_hooks.publication_overflow) {
                let mut previous = (*attempt.previous).clone();
                previous.change_impact = super::WorkspaceChangeImpact::NoPreviousPublication {
                    current_publication_id: WorkspacePublicationId::new(u64::MAX).unwrap(),
                };
                attempt.previous = Arc::new(previous);
            }
            if cancelled() {
                return Err(WorkspaceEditCause::Cancelled);
            }
            if attempt
                .previous
                .publication_id()
                .get()
                .checked_add(1)
                .is_none()
            {
                return Err(WorkspaceEditCause::BoundsExceeded);
            }
            if attempt.direction == Direction::Apply {
                let regenerated = attempt
                    .previous
                    .plan_refactoring(attempt.plan.request(), &NeverCancelledRefactoring)
                    .map_err(|_| WorkspaceEditCause::PlanMismatch)?;
                compare_plan(&attempt.plan, regenerated.plan())
                    .map_err(|_| WorkspaceEditCause::PlanMismatch)?;
            }
            check_admission(&attempt.plan, &attempt.previous, &attempt.baseline)?;
            let observed = EditBaseline::capture(root, &[])?;
            if !attempt.baseline.equals(&observed) {
                return Err(WorkspaceEditCause::SourceChanged);
            }
            drop(observed);
            let results = if let Some(undo) = &attempt.undo {
                undo.originals.clone()
            } else {
                let configuration = attempt
                    .previous
                    .configuration(attempt.plan.request().configuration_id())
                    .ok_or(WorkspaceEditCause::PlanMismatch)?;
                let mut results = BTreeMap::new();
                let mut aggregate = 0usize;
                // Reserve the entire worst-case result allowance before creating buffers.
                EditIoBudget::buffers(&[
                    attempt.baseline.raw_bytes(),
                    MAX_BASELINE * 2,
                    MAX_EDITED * 2,
                ])?;
                for document in configuration.source_evidence().documents() {
                    let operations: Vec<_> = attempt
                        .plan
                        .operations()
                        .iter()
                        .filter(|o| o.document_id() == document.id())
                        .collect();
                    if operations.is_empty() {
                        continue;
                    }
                    let bytes = replacement_bytes(document, &operations, MAX_DOCUMENT)
                        .map_err(|_| WorkspaceEditCause::PlanMismatch)?;
                    aggregate = aggregate
                        .checked_add(bytes.len())
                        .ok_or(WorkspaceEditCause::BoundsExceeded)?;
                    if aggregate > MAX_EDITED {
                        return Err(WorkspaceEditCause::BoundsExceeded);
                    }
                    results.insert(
                        PathBuf::from(document.path().path().as_str()),
                        Arc::from(bytes),
                    );
                }
                results
            };
            if attempt.direction == Direction::Apply {
                let reserved = attempt
                    .baseline
                    .raw_bytes()
                    .checked_add(MAX_BASELINE * 2)
                    .and_then(|n| n.checked_add(MAX_EDITED * 2))
                    .ok_or(WorkspaceEditCause::BoundsExceeded)?;
                let mut admission = SafeEditProjectionAdmission::new(reserved)
                    .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
                #[cfg(test)]
                if std::mem::take(&mut self.test_hooks.projection_overflow) {
                    admission
                        .reserve(
                            oneagent_analysis::safe_edit::MAX_SAFE_EDIT_BUFFER_BYTES - reserved,
                        )
                        .map_err(|_| WorkspaceEditCause::BoundsExceeded)?;
                }
                attempt.projection = Some(freeze_projection(
                    &attempt.plan,
                    &attempt.previous,
                    &results,
                    &mut admission,
                )?);
            }
            #[cfg(test)]
            self.test_phase("projection_frozen")?;
            edit_io = Some(EditIo::new(
                attempt.baseline.clone(),
                results,
                attempt.reservation.id,
            )?);
            let io = edit_io.as_mut().expect("created edit I/O");
            io.stage_all()?;
            #[cfg(test)]
            self.test_phase("staged")?;
            if cancelled() {
                return Err(WorkspaceEditCause::Cancelled);
            }
            io.replace_checked(cancelled)?;
            #[cfg(test)]
            self.test_phase("replaced")?;
            let before_build = io.verify_results()?;
            let mut candidate = builder
                .build(root)
                .map_err(|_| WorkspaceEditCause::SemanticMismatch)?;
            #[cfg(test)]
            {
                self.test_phase("built")?;
                if let Some(mutate) = self.test_hooks.candidate.take() {
                    mutate(&mut candidate);
                }
            }
            let after_build = io.verify_results()?;
            if !before_build.equals(&after_build) {
                return Err(WorkspaceEditCause::SourceChanged);
            }
            if let Some(undo) = &attempt.undo
                && !undo.baseline.equivalent(&after_build)
            {
                return Err(WorkspaceEditCause::SourceChanged);
            }
            drop(before_build);
            drop(after_build);
            if let Some(undo) = &attempt.undo {
                compare_exact_snapshot(&undo.before, &candidate)?;
            } else {
                compare_snapshot(
                    &attempt.plan,
                    &attempt.previous,
                    &candidate,
                    attempt
                        .projection
                        .as_ref()
                        .ok_or(WorkspaceEditCause::SemanticMismatch)?,
                )?;
            }
            if cancelled() {
                return Err(WorkspaceEditCause::Cancelled);
            }
            #[cfg(test)]
            self.test_phase("compared")?;
            compose_change_impact(&attempt.previous, &mut candidate, service)
                .map_err(|_| WorkspaceEditCause::SemanticMismatch)?;
            io.cleanup_owned()?;
            #[cfg(test)]
            self.test_phase("cleaned")?;
            let baseline = io.verify_results()?;
            if cancelled() {
                return Err(WorkspaceEditCause::Cancelled);
            }
            #[cfg(test)]
            self.test_phase("final_guard")?;
            Ok((candidate, baseline))
        })();
        match result {
            Ok((candidate, baseline)) => Ok((
                candidate,
                baseline,
                edit_io.expect("successful edit owns I/O"),
            )),
            Err(mut cause) => {
                if cancelled() {
                    cause = WorkspaceEditCause::Cancelled;
                }
                let Some(mut io) = edit_io else {
                    return Err(WorkspaceEditOutcome::failure(cause));
                };
                let attempted = io.attempted();
                let recovered = if attempted {
                    io.restore_checked().map(Some)
                } else {
                    io.cleanup_owned().map(|()| None)
                };
                if recovered.is_err() {
                    let retained_files = io.retained_files();
                    self.recovery = Some(io);
                    self.undo = None;
                    self.baseline = None;
                    let mut admission = self
                        .handle
                        .shared
                        .admission
                        .lock()
                        .expect("admission mutex");
                    admission.poisoned = true;
                    admission.ready = false;
                    Err(WorkspaceEditOutcome::Failed {
                        cause: WorkspaceEditCause::RecoveryRequired,
                        secondary: Some(cause),
                        recovery: WorkspaceEditRecovery::Required,
                        retained_files,
                    })
                } else {
                    if let Ok(Some(baseline)) = recovered {
                        self.baseline = Some(baseline);
                    }
                    Err(WorkspaceEditOutcome::Failed {
                        cause,
                        secondary: None,
                        recovery: if attempted {
                            WorkspaceEditRecovery::Recovered
                        } else {
                            WorkspaceEditRecovery::NotNeeded
                        },
                        retained_files: 0,
                    })
                }
            }
        }
    }

    #[cfg(test)]
    fn test_phase(&mut self, phase: &'static str) -> Result<()> {
        self.test_hooks.events.push(phase);
        self.test_hooks.observed.lock().unwrap().push(phase);
        if self
            .test_hooks
            .phase_gate
            .as_ref()
            .is_some_and(|(at, _)| *at == phase)
        {
            let (_, gate) = self.test_hooks.phase_gate.take().unwrap();
            gate();
        }
        if phase == "projection_frozen" {
            super::edit_io::faults::set(std::mem::take(&mut self.test_hooks.io_failures));
            if let Some(gate) = self.test_hooks.restore_gate.take() {
                super::edit_io::faults::action("restore_before", gate);
            }
        }
        if self.test_hooks.fail == Some(phase) {
            self.test_hooks.fail = None;
            Err(WorkspaceEditCause::SemanticMismatch)
        } else {
            Ok(())
        }
    }

    pub(super) fn commit(
        &mut self,
        commit: EditCommit,
    ) -> (oneshot::Sender<WorkspaceEditOutcome>, WorkspaceEditOutcome) {
        let previous = commit.attempt.previous.publication_id();
        let current = commit.candidate.publication_id();
        let plan = commit.attempt.plan.id().clone();
        let operations = commit.attempt.plan.operations().len();
        let originals = commit.io.originals();
        let files = originals.len();
        let reversal = if commit.attempt.direction == Direction::Apply {
            self.undo = Some(EditUndo {
                attempt: commit.attempt.reservation.id,
                before: Arc::clone(&commit.attempt.previous),
                applied: Arc::clone(&commit.candidate),
                baseline: commit.attempt.baseline,
                plan: commit.attempt.plan,
                originals,
            });
            Some(WorkspaceEditReceipt {
                identity: Arc::clone(&self.handle.shared.identity),
                attempt: commit.attempt.reservation.id,
            })
        } else {
            self.undo = None;
            None
        };
        self.baseline = Some(commit.baseline);
        (
            commit.response,
            WorkspaceEditOutcome::Applied {
                previous,
                current,
                plan,
                files,
                operations,
                reversal,
            },
        )
    }

    pub(super) fn deliver_terminal(&mut self) {
        if let Some((response, result)) = self.prepared.take() {
            let _ = response.send(result);
        }
        if let Some((response, outcome)) = self.terminal.take() {
            let _ = response.send(outcome);
        }
    }

    pub(super) fn abandon_commit(&mut self, mut commit: EditCommit, cause: WorkspaceEditCause) {
        let outcome = if let Ok(baseline) = commit.io.restore_checked() {
            self.baseline = Some(baseline);
            WorkspaceEditOutcome::Failed {
                cause,
                secondary: None,
                recovery: WorkspaceEditRecovery::Recovered,
                retained_files: 0,
            }
        } else {
            let retained_files = commit.io.retained_files();
            self.recovery = Some(commit.io);
            self.undo = None;
            self.baseline = None;
            let mut admission = self
                .handle
                .shared
                .admission
                .lock()
                .expect("admission mutex");
            admission.poisoned = true;
            admission.ready = false;
            WorkspaceEditOutcome::Failed {
                cause: WorkspaceEditCause::RecoveryRequired,
                secondary: Some(cause),
                recovery: WorkspaceEditRecovery::Required,
                retained_files,
            }
        };
        self.terminal = Some((commit.response, outcome));
    }

    pub(super) fn precommit_cancelled(commit: &EditCommit) -> bool {
        commit.cancellation.requested()
    }

    pub(super) fn predecessor_matches(
        commit: &EditCommit,
        current: Option<&Arc<WorkspaceSnapshot>>,
    ) -> bool {
        current.is_some_and(|current| Arc::ptr_eq(current, &commit.attempt.previous))
            && matches!(&commit.candidate.change_impact, super::WorkspaceChangeImpact::Available(report)
                if report.previous_publication_id() == commit.attempt.previous.publication_id()
                    && commit.attempt.previous.publication_id().get().checked_add(1)
                        == Some(report.current_publication_id().get()))
    }
}

fn compare_exact_snapshot(before: &WorkspaceSnapshot, after: &WorkspaceSnapshot) -> Result<()> {
    if before.root_path() != after.root_path() || before.len() != after.len() {
        return Err(WorkspaceEditCause::SemanticMismatch);
    }
    for (old, new) in before.configurations().iter().zip(after.configurations()) {
        if old.format() != new.format()
            || old.configuration_id() != new.configuration_id()
            || old.configuration_name() != new.configuration_name()
        {
            return Err(WorkspaceEditCause::SemanticMismatch);
        }
        validate_equivalence(&evidence(old), &evidence(new))
            .map_err(|_| WorkspaceEditCause::SemanticMismatch)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Keep each accepted named matrix oracle and its full fault table together.
    #![allow(clippy::too_many_lines)]
    use super::super::WorkspaceService;
    use super::*;
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph};
    use oneagent_tool_policy::RuleAction;
    use std::fs;

    #[derive(Clone)]
    struct MutatingEditDetector {
        calls: Arc<std::sync::atomic::AtomicUsize>,
        mutate_at: usize,
        fail: bool,
    }
    type BuildGate = Arc<Mutex<Option<(oneshot::Sender<()>, oneshot::Receiver<()>)>>>;
    #[derive(Clone)]
    struct GatedEditDetector {
        at: usize,
        calls: Arc<std::sync::atomic::AtomicUsize>,
        gate: BuildGate,
    }
    impl WorkspaceDetector for GatedEditDetector {
        fn discover(
            &self,
            root: &Path,
        ) -> std::result::Result<Vec<oneagent_workspace::DiscoveredConfiguration>, crate::BoxError>
        {
            if self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1 == self.at {
                let (entered, release) = self.gate.lock().unwrap().take().unwrap();
                entered.send(()).unwrap();
                release.blocking_recv().unwrap();
            }
            oneagent_workspace_fs::FileSystemWorkspaceDetector::default().discover(root)
        }
    }
    struct EditCacheProbe {
        root: PathBuf,
        store: super::super::cache::WorkspaceCacheStore,
        mutate_load: bool,
        write_gate: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    }
    impl super::super::cache::WorkspaceCacheStorage for EditCacheProbe {
        fn prepare_edit_namespace(&self) -> std::io::Result<()> {
            super::super::cache::WorkspaceCacheStorage::prepare_edit_namespace(&self.store)
        }
        fn load(
            &self,
            state: &super::super::WorkspaceFileState,
        ) -> super::super::cache::WorkspaceCacheLoad {
            let loaded = self.store.load(state);
            assert_eq!(
                loaded.outcome(),
                super::super::WorkspaceCacheLoadOutcome::Hit
            );
            if self.mutate_load {
                fs::write(
                    self.root.join("changed-during-cache.txt"),
                    b"external input",
                )
                .unwrap();
            }
            loaded
        }
        fn write(
            &self,
            state: &super::super::WorkspaceFileState,
            snapshot: &WorkspaceSnapshot,
        ) -> super::super::WorkspaceCacheWriteOutcome {
            if let Some(gate) = self.write_gate.lock().unwrap().take() {
                gate();
            }
            self.store.write(state, snapshot)
        }
    }
    impl WorkspaceDetector for MutatingEditDetector {
        fn discover(
            &self,
            root: &Path,
        ) -> std::result::Result<Vec<oneagent_workspace::DiscoveredConfiguration>, crate::BoxError>
        {
            let projects =
                oneagent_workspace_fs::FileSystemWorkspaceDetector::default().discover(root)?;
            let call = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if call == self.mutate_at {
                if self.fail {
                    return Err(Box::new(std::io::Error::other(
                        "SECRET_BUILD_ERROR /absolute/private/path SECRET_SOURCE_TOKEN",
                    )));
                }
                fs::write(root.join("changed-during-build.txt"), b"external input")?;
            }
            Ok(projects)
        }
    }

    mod fixtures {
        use crate as oneagent_runtime;
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/safe_edit_transactions.rs"
        ));
    }

    async fn rejected(mut hooks: TestHooks, recovery: WorkspaceEditRecovery) {
        fn material(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
            let mut result = BTreeMap::new();
            for entry in fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    result.extend(material(&path));
                } else if path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".oneagent-edit-")
                {
                    result.insert(path.clone(), fs::read(path).unwrap());
                }
            }
            result
        }
        let external_bytes = hooks.external_bytes;
        let producer_rejection = hooks.projection_overflow || hooks.publication_overflow;
        let root = fixtures::fixture("edt");
        if hooks.external_after_cleanup {
            let path = root
                .path()
                .join("src/CommonModules/SecondaryCaller/Module.bsl");
            hooks.external_bytes = Some("src/CommonModules/SecondaryCaller/Module.bsl");
            hooks.phase_gate = Some((
                "cleaned",
                Box::new(move || fs::write(path, b"external input").unwrap()),
            ));
        }
        let external_bytes = hooks.external_bytes.or(external_bytes);
        fixtures::add_query_fixture(
            root.path(),
            "edt",
            if hooks.resolved_query {
                "resolved"
            } else {
                "missing"
            },
        );
        let recorded = Arc::clone(&hooks.observed);
        let (ticks, receive_ticks) = mpsc::channel(2);
        let mut service = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
            MutatingEditDetector {
                calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
                mutate_at: if hooks.builder_failure { 2 } else { usize::MAX },
                fail: hooks.builder_failure,
            },
        ))
        .with_edit_policy(
            fixtures::policy(RuleAction::RequireConfirmation),
            WorkspaceEditOwnership::ExclusiveCooperative,
        )
        .with_controlled_change_ticks(receive_ticks);
        let input = service.change_input_handle();
        service.edits.test_hooks = hooks;
        let (handle, observer, stop, task) = fixtures::start_service(root.path(), service).await;
        let before = observer.snapshot().unwrap();
        let (challenge, _) = handle
            .prepare_apply(
                fixtures::request(&before, "Changed"),
                fixtures::actor(),
                fixtures::request_id(),
            )
            .await
            .unwrap();
        let outcome = handle
            .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
            .await;
        assert_eq!(
            handle.shared.admission.lock().unwrap().slot,
            None,
            "terminal attempt releases its slot"
        );
        assert!(
            matches!(outcome, WorkspaceEditOutcome::Failed { recovery: actual, .. } if actual == recovery),
            "{outcome:?}"
        );
        let rendered = format!("{outcome:?}");
        for secret in [
            "SECRET_BUILD_ERROR",
            "SECRET_SOURCE_TOKEN",
            "/absolute/private/path",
            "FillSecurityCollection",
        ] {
            assert!(!rendered.contains(secret));
        }
        assert!(!rendered.contains(root.path().to_str().unwrap()));
        {
            let phases = recorded.lock().unwrap();
            if producer_rejection {
                assert!(phases.is_empty(), "producer admission must precede staging");
            } else {
                assert_eq!(phases.first(), Some(&"projection_frozen"));
            }
            if phases.contains(&"built") {
                assert!(
                    phases.iter().position(|p| *p == "projection_frozen")
                        < phases.iter().position(|p| *p == "built")
                );
            }
        }
        if recovery == WorkspaceEditRecovery::Required {
            if let Some(path) = external_bytes {
                assert_eq!(fs::read(root.path().join(path)).unwrap(), b"external input");
            }
            assert!(observer.snapshot().is_none());
            assert_eq!(
                handle
                    .prepare_apply(
                        fixtures::request(&before, "Next"),
                        fixtures::actor(),
                        fixtures::request_id()
                    )
                    .await
                    .unwrap_err(),
                WorkspaceEditCause::RecoveryRequired
            );
            let (ack, receive) = oneshot::channel();
            ticks.send(ack).await.unwrap();
            receive.await.unwrap();
            let path = super::super::RepositoryChangePath::new("source-after-poison.bsl").unwrap();
            let submission = input.submit(
                super::super::GitChangeSet::new(
                    super::super::GitCommitId::new("0123456789abcdef0123456789abcdef01234567")
                        .unwrap(),
                    [super::super::RepositoryChange::new(
                        super::super::RepositoryChangeKind::Modified,
                        Some(path.clone()),
                        Some(path),
                    )
                    .unwrap()],
                )
                .unwrap(),
            );
            assert_eq!(
                submission,
                super::super::WorkspaceChangeSubmissionOutcome::Accepted
            );
            tokio::task::yield_now().await;
            assert!(
                observer.snapshot().is_none(),
                "poisoned watcher/explicit input cannot republish"
            );
        } else {
            assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
            for document in before.configurations()[0].source_evidence().documents() {
                assert_eq!(
                    fs::read(root.path().join(document.path().path().as_str())).unwrap(),
                    document.raw_content()
                );
            }
            // A fresh confirmation must work after recovery changed the inode.
            let (challenge, _) = handle
                .prepare_apply(
                    fixtures::request(&before, "Retry"),
                    fixtures::actor(),
                    fixtures::request_id(),
                )
                .await
                .unwrap();
            let outcome = handle
                .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                .await;
            assert!(
                matches!(outcome, WorkspaceEditOutcome::Applied { .. }),
                "fresh retry: {outcome:?}"
            );
        }
        let retained = material(root.path());
        if recovery == WorkspaceEditRecovery::Required {
            let WorkspaceEditOutcome::Failed { retained_files, .. } = outcome else {
                unreachable!()
            };
            assert_eq!(retained.len(), retained_files);
        } else {
            assert!(retained.is_empty());
        }
        stop.send(()).unwrap();
        task.await.unwrap().unwrap();
        assert_eq!(
            material(root.path()),
            retained,
            "shutdown preserves recovery material exactly"
        );
    }

    #[tokio::test]
    async fn attempt_lifetime_and_bounds() {
        for count in [MAX_OPERATIONS, MAX_OPERATIONS + 1] {
            let root = fixtures::fixture("edt");
            let path = root
                .path()
                .join("src/CommonModules/DynamicSecurityOverridable/Module.bsl");
            let mut bytes = b"Procedure A() Export\nEndProcedure\nProcedure Caller()\n".to_vec();
            // The tracked secondary caller contributes two qualified calls.
            for _ in 0..count - 3 {
                bytes.extend_from_slice(b"A();\n");
            }
            bytes.extend_from_slice(b"EndProcedure\n");
            fs::write(path, bytes).unwrap();
            let caller = root
                .path()
                .join("src/CommonModules/SecondaryCaller/Module.bsl");
            fs::write(
                &caller,
                fs::read_to_string(&caller)
                    .unwrap()
                    .replace("FillSecurityCollection", "A"),
            )
            .unwrap();
            let service = WorkspaceService::new().with_edit_policy(
                fixtures::policy(RuleAction::RequireConfirmation),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            let (handle, observer, stop, task) =
                fixtures::start_service(root.path(), service).await;
            let before = observer.snapshot().unwrap();
            let configuration = &before.configurations()[0];
            let target = configuration
                .graph()
                .nodes()
                .find(|node| node.name().as_str() == "A")
                .unwrap()
                .id()
                .clone();
            assert_eq!(
                configuration
                    .source_evidence()
                    .documents()
                    .iter()
                    .flat_map(oneagent_analysis::refactoring::SourceDocument::occurrences)
                    .filter(|o| o.mapped_target_id() == Some(&target))
                    .count(),
                count
            );
            let request = RefactoringRequest::new(
                oneagent_analysis::refactoring::RefactoringFamily::BslCallableRenameV1,
                before.publication_id(),
                configuration.configuration_id().clone(),
                target,
                "B",
            )
            .unwrap();
            let prepared = handle
                .prepare_apply(request, fixtures::actor(), fixtures::request_id())
                .await;
            if count == MAX_OPERATIONS {
                let (challenge, _) = prepared.unwrap();
                assert_eq!(challenge.attempt.plan.operations().len(), count);
                drop(challenge);
            } else {
                assert_eq!(prepared.unwrap_err(), WorkspaceEditCause::BoundsExceeded);
            }
            assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
            assert_eq!(handle.shared.admission.lock().unwrap().slot, None);
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
        }
        let root = fixtures::fixture("edt");
        let snapshot = WorkspaceSnapshotBuilder::new().build(root.path()).unwrap();
        let mut queued = EditCoordinator::new();
        queued.handle.shared.admission.lock().unwrap().ready = true;
        let handle = queued.handle();
        let request = fixtures::request(&snapshot, "Changed");
        let pending = tokio::spawn(async move {
            handle
                .prepare_apply(request, fixtures::actor(), fixtures::request_id())
                .await
        });
        let command = queued.commands.recv().await.unwrap();
        assert!(matches!(
            queued.handle.reserve_attempt(),
            Err(WorkspaceEditCause::Busy)
        ));
        pending.abort();
        assert!(
            matches!(
                queued.handle.reserve_attempt(),
                Err(WorkspaceEditCause::Busy)
            ),
            "queued owner retains the slot after receiver drop"
        );
        drop(command);
        assert_eq!(queued.handle.shared.admission.lock().unwrap().slot, None);
        let service = WorkspaceService::new().with_edit_policy(
            fixtures::policy(RuleAction::RequireConfirmation),
            WorkspaceEditOwnership::ExclusiveCooperative,
        );
        let (handle, observer, stop, task) = fixtures::start_service(root.path(), service).await;
        for confirmed in [false, true] {
            let (challenge, _) = handle
                .prepare_apply(
                    fixtures::request(&observer.snapshot().unwrap(), "Changed"),
                    fixtures::actor(),
                    fixtures::request_id(),
                )
                .await
                .unwrap();
            assert!(handle.shared.admission.lock().unwrap().slot.is_some());
            if confirmed {
                drop(challenge.confirm());
            } else {
                drop(challenge);
            }
            assert_eq!(handle.shared.admission.lock().unwrap().slot, None);
        }
        stop.send(()).unwrap();
        task.await.unwrap().unwrap();
        let coordinator = EditCoordinator::new();
        coordinator.handle.shared.admission.lock().unwrap().ready = true;
        let first = coordinator.handle.reserve_attempt().unwrap();
        assert!(matches!(
            coordinator.handle.reserve_attempt(),
            Err(WorkspaceEditCause::Busy)
        ));
        drop(first);
        let second = coordinator.handle.reserve_attempt().unwrap();
        assert_eq!(second.id, 2);
        drop(second);
        coordinator.handle.shared.admission.lock().unwrap().writer = true;
        assert!(matches!(
            coordinator.handle.reserve_attempt(),
            Err(WorkspaceEditCause::Busy)
        ));
        let mut state = coordinator.handle.shared.admission.lock().unwrap();
        state.writer = false;
        state.next = u64::MAX;
        drop(state);
        assert!(matches!(
            coordinator.handle.reserve_attempt(),
            Err(WorkspaceEditCause::BoundsExceeded)
        ));
        assert_eq!(
            coordinator.handle.shared.admission.lock().unwrap().slot,
            None
        );
    }

    #[tokio::test]
    async fn projection_freeze_precedes_io() {
        rejected(
            TestHooks {
                projection_overflow: true,
                ..TestHooks::default()
            },
            WorkspaceEditRecovery::NotNeeded,
        )
        .await;
        rejected(
            TestHooks {
                fail: Some("projection_frozen"),
                ..TestHooks::default()
            },
            WorkspaceEditRecovery::NotNeeded,
        )
        .await;
    }

    #[tokio::test]
    async fn publication_barriers_and_overflow() {
        for at in [1, 2] {
            let root = fixtures::fixture("edt");
            let frozen = WorkspaceSnapshotBuilder::new().build(root.path()).unwrap();
            let (entered, waiting) = oneshot::channel();
            let (release, blocked) = oneshot::channel();
            let service = WorkspaceService::with_builder(WorkspaceSnapshotBuilder::with_detector(
                GatedEditDetector {
                    at,
                    calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
                    gate: Arc::new(Mutex::new(Some((entered, blocked)))),
                },
            ))
            .with_edit_policy(
                fixtures::policy(RuleAction::RequireConfirmation),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            let input = service.change_input_handle();
            let handle = service.edit_handle();
            let observer = service.snapshot_observer();
            let mut changes = observer.subscribe();
            let app = crate::App::builder()
                .configure(&fixtures::Provider(root.path().to_owned()))
                .unwrap()
                .register_service("workspace", service)
                .unwrap()
                .build()
                .unwrap();
            let (stop, signal) = oneshot::channel();
            let task = tokio::spawn(app.run(signal));
            if at == 2 {
                while changes.borrow().is_none() {
                    changes.changed().await.unwrap();
                }
                let path = super::super::RepositoryChangePath::new("explicit-build.bsl").unwrap();
                assert_eq!(
                    input.submit(
                        super::super::GitChangeSet::new(
                            super::super::GitCommitId::new(
                                "0123456789abcdef0123456789abcdef01234567"
                            )
                            .unwrap(),
                            [super::super::RepositoryChange::new(
                                super::super::RepositoryChangeKind::Modified,
                                Some(path.clone()),
                                Some(path)
                            )
                            .unwrap()]
                        )
                        .unwrap()
                    ),
                    super::super::WorkspaceChangeSubmissionOutcome::Accepted
                );
            }
            waiting.await.unwrap();
            assert_eq!(
                handle
                    .prepare_apply(
                        fixtures::request(&frozen, "Changed"),
                        fixtures::actor(),
                        fixtures::request_id()
                    )
                    .await
                    .unwrap_err(),
                if at == 1 {
                    WorkspaceEditCause::Unavailable
                } else {
                    WorkspaceEditCause::Busy
                }
            );
            if at == 1 {
                assert!(observer.snapshot().is_none());
            } else {
                assert_eq!(observer.snapshot().unwrap().publication_id().get(), 1);
            }
            release.send(()).unwrap();
            tokio::time::timeout(std::time::Duration::from_secs(10), async {
                while changes
                    .borrow()
                    .as_ref()
                    .is_none_or(|s| s.publication_id().get() != at as u64)
                {
                    changes.changed().await.unwrap();
                }
            })
            .await
            .unwrap();
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
        }
        {
            let root = fixtures::fixture("edt");
            let (entered, waiting) = oneshot::channel();
            let (release, blocked) = oneshot::channel();
            let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut service = WorkspaceService::with_builder(
                WorkspaceSnapshotBuilder::with_detector(MutatingEditDetector {
                    calls: Arc::clone(&calls),
                    mutate_at: usize::MAX,
                    fail: false,
                }),
            )
            .with_edit_policy(
                fixtures::policy(RuleAction::RequireConfirmation),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            let input = service.change_input_handle();
            service.edits.test_hooks.phase_gate = Some((
                "staged",
                Box::new(move || {
                    entered.send(()).unwrap();
                    blocked.blocking_recv().unwrap();
                }),
            ));
            let (handle, observer, stop, task) =
                fixtures::start_service(root.path(), service).await;
            let before = observer.snapshot().unwrap();
            let (challenge, _) = handle
                .prepare_apply(
                    fixtures::request(&before, "Changed"),
                    fixtures::actor(),
                    fixtures::request_id(),
                )
                .await
                .unwrap();
            let submitting = handle.clone();
            let response = tokio::spawn(async move {
                submitting
                    .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await
            });
            waiting.await.unwrap();
            let path = super::super::RepositoryChangePath::new("explicit-during-edit.bsl").unwrap();
            assert_eq!(
                input.submit(
                    super::super::GitChangeSet::new(
                        super::super::GitCommitId::new("0123456789abcdef0123456789abcdef01234567")
                            .unwrap(),
                        [super::super::RepositoryChange::new(
                            super::super::RepositoryChangeKind::Modified,
                            Some(path.clone()),
                            Some(path)
                        )
                        .unwrap()]
                    )
                    .unwrap()
                ),
                super::super::WorkspaceChangeSubmissionOutcome::Accepted
            );
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
            assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
            assert_eq!(
                handle
                    .prepare_apply(
                        fixtures::request(&before, "Other"),
                        fixtures::actor(),
                        fixtures::request_id()
                    )
                    .await
                    .unwrap_err(),
                WorkspaceEditCause::Busy
            );
            let mut changes = observer.subscribe();
            release.send(()).unwrap();
            let WorkspaceEditOutcome::Applied {
                reversal: Some(receipt),
                ..
            } = response.await.unwrap()
            else {
                panic!("edit must precede explicit successor");
            };
            tokio::time::timeout(std::time::Duration::from_secs(10), async {
                while changes.borrow().as_ref().unwrap().publication_id().get() < 3 {
                    changes.changed().await.unwrap();
                }
            })
            .await
            .unwrap();
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
            assert_eq!(observer.snapshot().unwrap().publication_id().get(), 3);
            assert!(
                handle
                    .prepare_reversal(receipt, fixtures::actor(), fixtures::request_id())
                    .await
                    .is_err()
            );
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
        }
        for wrong_impact in [false, true] {
            rejected(
                TestHooks {
                    wrong_impact,
                    stale_predecessor: !wrong_impact,
                    ..TestHooks::default()
                },
                WorkspaceEditRecovery::Recovered,
            )
            .await;
        }
        rejected(
            TestHooks {
                publication_overflow: true,
                ..TestHooks::default()
            },
            WorkspaceEditRecovery::NotNeeded,
        )
        .await;
        let root = fixtures::fixture("edt");
        let mut previous = WorkspaceSnapshotBuilder::new().build(root.path()).unwrap();
        let mut candidate = previous.clone();
        previous.change_impact = super::super::WorkspaceChangeImpact::NoPreviousPublication {
            current_publication_id: WorkspacePublicationId::new(u64::MAX).unwrap(),
        };
        assert!(
            super::super::compose_change_impact(
                &previous,
                &mut candidate,
                &oneagent_analysis::change_impact::NeverCancelledChangeImpact
            )
            .is_err()
        );
        assert_eq!(candidate.publication_id().get(), 1);
    }

    #[tokio::test]
    async fn post_write_build_and_tree_failures_recover() {
        rejected(
            TestHooks {
                builder_failure: true,
                ..TestHooks::default()
            },
            WorkspaceEditRecovery::Recovered,
        )
        .await;
        for path in [
            "src/Configuration/Configuration.mdo",
            "src/CommonModules/SecondaryCaller/Module.bsl",
            ".oneagent/unrelated-input",
        ] {
            rejected(
                TestHooks {
                    external_bytes: Some(path),
                    candidate: Some(Box::new(move |candidate| {
                        fs::write(candidate.root_path().join(path), b"external input").unwrap();
                    })),
                    ..TestHooks::default()
                },
                WorkspaceEditRecovery::Required,
            )
            .await;
        }
        for phase in ["replaced", "built"] {
            rejected(
                TestHooks {
                    fail: Some(phase),
                    ..TestHooks::default()
                },
                WorkspaceEditRecovery::Recovered,
            )
            .await;
        }
    }

    #[tokio::test]
    async fn final_cleanup_scan_and_commit_barrier() {
        rejected(
            TestHooks {
                external_after_cleanup: true,
                ..TestHooks::default()
            },
            WorkspaceEditRecovery::Required,
        )
        .await;
        for phase in ["compared", "cleaned", "final_guard"] {
            rejected(
                TestHooks {
                    fail: Some(phase),
                    ..TestHooks::default()
                },
                WorkspaceEditRecovery::Recovered,
            )
            .await;
        }
    }

    #[tokio::test]
    async fn recovery_outcomes_and_precedence() {
        for point in [
            "replace_after",
            "restore_check",
            "restore_before",
            "restore_after",
        ] {
            for ordinal in 1..=2 {
                let recovery = if point == "replace_after" {
                    WorkspaceEditRecovery::Recovered
                } else {
                    WorkspaceEditRecovery::Required
                };
                rejected(
                    TestHooks {
                        fail: if point == "replace_after" {
                            None
                        } else {
                            Some("built")
                        },
                        io_failures: vec![(point, ordinal)],
                        ..TestHooks::default()
                    },
                    recovery,
                )
                .await;
            }
        }
    }

    #[tokio::test]
    async fn reversal_failures_restore_applied_state() {
        let mut cases = Vec::new();
        for phase in [
            "projection_frozen",
            "staged",
            "replaced",
            "built",
            "compared",
            "cleaned",
            "final_guard",
        ] {
            cases.push((
                Some(phase),
                vec![],
                if matches!(phase, "projection_frozen" | "staged") {
                    WorkspaceEditRecovery::NotNeeded
                } else {
                    WorkspaceEditRecovery::Recovered
                },
            ));
        }
        for point in [
            "create",
            "write",
            "permissions",
            "sync",
            "close_observation",
            "readback",
        ] {
            for ordinal in 1..=4 {
                cases.push((
                    None,
                    vec![(point, ordinal)],
                    WorkspaceEditRecovery::NotNeeded,
                ));
            }
            // Original backup recreation after final cleanup shares the same I/O owner.
            for ordinal in 5..=6 {
                cases.push((
                    Some("cleaned"),
                    vec![(point, ordinal)],
                    WorkspaceEditRecovery::Required,
                ));
            }
        }
        for point in [
            "replace_before",
            "replace_after",
            "cleanup",
            "restore_check",
            "restore_before",
            "restore_after",
        ] {
            for ordinal in 1..=2 {
                cases.push((
                    if point.starts_with("restore") {
                        Some("built")
                    } else {
                        None
                    },
                    vec![(point, ordinal)],
                    if point.starts_with("restore") {
                        WorkspaceEditRecovery::Required
                    } else if point == "replace_before" && ordinal == 1 {
                        WorkspaceEditRecovery::NotNeeded
                    } else {
                        WorkspaceEditRecovery::Recovered
                    },
                ));
            }
        }
        for format in ["edt", "designer"] {
            for (phase, faults, expected) in &cases {
                let root = fixtures::fixture(format);
                fixtures::add_query_fixture(root.path(), format, "missing");
                let mut service = WorkspaceService::new().with_edit_policy(
                    fixtures::policy(RuleAction::RequireConfirmation),
                    WorkspaceEditOwnership::ExclusiveCooperative,
                );
                service.edits.test_hooks.reversal_fail = *phase;
                service.edits.test_hooks.reversal_io_failures = faults.clone();
                let (handle, observer, stop, task) =
                    fixtures::start_service(root.path(), service).await;
                let before = observer.snapshot().unwrap();
                let (challenge, _) = handle
                    .prepare_apply(
                        fixtures::request(&before, "Changed"),
                        fixtures::actor(),
                        fixtures::request_id(),
                    )
                    .await
                    .unwrap();
                let WorkspaceEditOutcome::Applied {
                    reversal: Some(receipt),
                    ..
                } = handle
                    .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await
                else {
                    panic!("apply prerequisite");
                };
                let applied = observer.snapshot().unwrap();
                let (challenge, _) = handle
                    .prepare_reversal(receipt, fixtures::actor(), fixtures::request_id())
                    .await
                    .unwrap();
                let outcome = handle
                    .checked_reversal(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await;
                assert!(
                    matches!(outcome, WorkspaceEditOutcome::Failed { recovery, .. } if recovery == *expected),
                    "{format}/{phase:?}/{faults:?}: {outcome:?}"
                );
                assert_eq!(
                    Arc::strong_count(&before),
                    1,
                    "submitted reversal releases original semantic retention on every terminal outcome"
                );
                if *expected == WorkspaceEditRecovery::Required {
                    assert!(observer.snapshot().is_none());
                } else {
                    assert!(Arc::ptr_eq(&applied, &observer.snapshot().unwrap()));
                    for document in applied.configurations()[0].source_evidence().documents() {
                        assert_eq!(
                            fs::read(root.path().join(document.path().path().as_str())).unwrap(),
                            document.raw_content()
                        );
                    }
                }
                stop.send(()).unwrap();
                task.await.unwrap().unwrap();
            }
        }
    }

    fn mutate(candidate: &mut WorkspaceSnapshot, kind: &str) {
        if kind == "extra_configuration" {
            candidate
                .configurations
                .push(candidate.configurations[0].clone());
            return;
        }
        let configuration = &mut candidate.configurations[0];
        if kind.starts_with("source_") {
            use oneagent_analysis::refactoring::*;
            let mut documents = configuration.source_evidence.documents().to_vec();
            let old = &documents[0];
            let identity = if matches!(kind, "source_identity" | "source_extra") {
                SourceDocumentId::new(
                    configuration.configuration_id.clone(),
                    EntityId::new("different.module").unwrap(),
                )
                .unwrap()
            } else {
                old.id().clone()
            };
            let mut bytes = old.raw_content().to_vec();
            if kind == "source_bytes" {
                bytes.extend_from_slice(b"// unrelated\n");
            }
            let version = SourceContentVersion::from_bytes(&bytes);
            let occurrences = old
                .occurrences()
                .iter()
                .enumerate()
                .filter_map(|(index, o)| {
                    if kind == "source_omission" && index == 0 {
                        return None;
                    }
                    let ambiguous = kind == "source_ambiguity" && index == 0;
                    Some(
                        SourceOccurrence::new_with_lexical_owner(
                            identity.clone(),
                            version,
                            o.range(),
                            o.kind(),
                            o.token(),
                            o.lexical_owner_token().map(str::to_owned),
                            if ambiguous {
                                None
                            } else {
                                o.mapped_target_id().cloned()
                            },
                            if ambiguous {
                                SourceOccurrenceResolution::Ambiguous
                            } else {
                                o.resolution()
                            },
                        )
                        .unwrap(),
                    )
                })
                .collect();
            let changed = SourceDocument::new(
                identity,
                old.format(),
                if kind == "source_role" {
                    if old.module_role() == BslModuleRole::Object {
                        BslModuleRole::Manager
                    } else {
                        BslModuleRole::Object
                    }
                } else {
                    old.module_role()
                },
                if matches!(kind, "source_path" | "source_extra") {
                    ConfinedSourcePath::new_at_workspace_root(
                        oneagent_common::SourcePath::new("src/Different.bsl").unwrap(),
                    )
                    .unwrap()
                } else {
                    old.path().clone()
                },
                bytes,
                occurrences,
                old.completeness(),
            )
            .unwrap();
            assert_ne!(&changed, old, "{kind}");
            if kind == "source_extra" {
                documents.push(changed);
            } else {
                documents[0] = changed;
            }
            configuration.source_evidence =
                SourceEvidenceSet::new(configuration.configuration_id.clone(), documents).unwrap();
            return;
        }
        if kind.starts_with("rule_") {
            use oneagent_analysis::rules::*;
            struct ChangedRule(RuleDefinition, &'static str);
            impl RuleRegistration for ChangedRule {
                fn definition(&self) -> &RuleDefinition {
                    &self.0
                }
            }
            impl Rule for ChangedRule {
                fn evaluate(
                    &self,
                    _: &RuleContext<'_>,
                    _: &dyn RuleCancellationSignal,
                ) -> RuleEvaluation {
                    match self.1 {
                        "failed" => {
                            RuleEvaluation::Failed(RuleFailureCode::new("fixture.failed").unwrap())
                        }
                        "completed" => RuleEvaluation::Completed(vec![]),
                        _ => RuleEvaluation::NotApplicable,
                    }
                }
            }
            let action = match kind {
                "rule_failed" => "failed",
                "rule_completed" => "completed",
                _ => "not_applicable",
            };
            let rule: Arc<dyn Rule> = Arc::new(ChangedRule(
                RuleDefinition::new(RuleId::new("fixture.changed").unwrap(), []).unwrap(),
                action,
            ));
            let registry = RuleRegistry::new([rule]).unwrap();
            let (rules, findings) = super::super::compose_rule_evidence(
                &registry,
                &RuleConfiguration::default(),
                &configuration.graph,
                &configuration.validation,
                &configuration.diagnostics,
                &NeverCancelled,
            )
            .unwrap();
            assert_ne!(configuration.rule_execution_report.as_ref(), &rules);
            configuration.rule_execution_report = Arc::new(rules);
            configuration.diagnostic_report = Arc::new(findings);
            return;
        }
        match kind {
            "format" => {
                configuration.format = oneagent_workspace::WorkspaceFormat::DesignerXml;
                return;
            }
            "report" => {
                configuration.report = oneagent_graph::SemanticGraphReport::from_graph_diagnostics_and_reference_requests(&configuration.graph, &[], &oneagent_graph::SemanticReferenceRequestLedger::new());
                return;
            }
            "validation" => {
                let mut graph = SemanticGraph::new();
                graph.insert_node(GraphNode::new(
                    EntityId::new("changed").unwrap(),
                    EntityName::new("Changed").unwrap(),
                    NodeKind::Module,
                ));
                configuration.validation =
                    Arc::new(oneagent_graph::SemanticGraphValidator::new().validate(&graph));
                return;
            }
            "findings" => {
                configuration.diagnostic_report = Arc::new(
                    oneagent_analysis::diagnostics::DiagnosticEngine
                        .build(
                            &[],
                            &oneagent_graph::SemanticGraphValidator::new()
                                .validate(&SemanticGraph::new()),
                            &oneagent_analysis::diagnostics::DiagnosticPolicy::default(),
                        )
                        .unwrap(),
                );
                return;
            }
            "root" => {
                configuration.root_path.push("other");
                return;
            }
            "configuration" => {
                configuration.configuration_id = EntityId::new("other").unwrap();
                return;
            }
            "documents" => {
                configuration.source_evidence =
                    oneagent_analysis::refactoring::SourceEvidenceSet::new(
                        configuration.configuration_id.clone(),
                        Vec::new(),
                    )
                    .unwrap();
                return;
            }
            "diagnostics" => {
                assert!(!configuration.diagnostics.is_empty());
                configuration.diagnostics = Arc::from([]);
                return;
            }
            "diagnostic_order" => {
                let mut values = configuration.diagnostics.to_vec();
                assert!(values.len() > 1);
                values.swap(0, 1);
                configuration.diagnostics = Arc::from(values);
                return;
            }
            "requests" => {
                assert!(!configuration.reference_requests.is_empty());
                configuration.reference_requests =
                    Arc::new(oneagent_graph::SemanticReferenceRequestLedger::new());
                return;
            }
            _ => {}
        }
        if kind.starts_with("diagnostic_") {
            use oneagent_graph::*;
            let old = &configuration.diagnostics[0];
            let mut value = SemanticDiagnostic::new(
                if kind == "diagnostic_code" {
                    SemanticDiagnosticCode::ReferenceInvalidOwner
                } else {
                    old.code()
                },
                if kind == "diagnostic_severity" {
                    if old.severity() == SemanticDiagnosticSeverity::Warning {
                        SemanticDiagnosticSeverity::Error
                    } else {
                        SemanticDiagnosticSeverity::Warning
                    }
                } else {
                    old.severity()
                },
                if kind == "diagnostic_kind" {
                    SemanticDiagnosticKind::MalformedReferenceFormat
                } else {
                    old.kind()
                },
                if kind == "diagnostic_message" {
                    "changed"
                } else {
                    old.message()
                },
                if kind == "diagnostic_reference" {
                    SemanticReference::Raw("changed".into())
                } else {
                    old.reference().clone()
                },
            )
            .with_expected_kinds(if kind == "diagnostic_expected" {
                vec![NodeKind::Function]
            } else {
                old.expected_kinds().to_vec()
            })
            .with_candidates(if kind == "diagnostic_candidates" {
                vec![EntityId::new("different.candidate").unwrap()]
            } else {
                old.candidates().to_vec()
            })
            .with_provenance(if kind == "diagnostic_provenance" {
                vec![]
            } else if kind == "diagnostic_location" {
                old.provenance()
                    .iter()
                    .cloned()
                    .map(|p| {
                        p.with_location(oneagent_common::SourceLocation::new(
                            oneagent_common::SourcePath::new("different-diagnostic.bsl").unwrap(),
                            None,
                        ))
                    })
                    .collect()
            } else {
                old.provenance().to_vec()
            });
            if let Some(source) = old.source_node() {
                value = value.with_source_node(if kind == "diagnostic_source" {
                    EntityId::new("changed").unwrap()
                } else {
                    source.clone()
                });
            }
            if let Some(actual) = old.actual_kind() {
                value = value.with_actual_kind(actual);
            }
            if kind == "diagnostic_actual" {
                value = value.with_actual_kind(NodeKind::Function);
            }
            assert_ne!(&value, old, "{kind}");
            let mut values = configuration.diagnostics.to_vec();
            values[0] = value;
            values.sort();
            configuration.diagnostics = Arc::from(values);
            return;
        }
        if kind.starts_with("request_") {
            use oneagent_graph::*;
            let index = if kind == "request_candidate" {
                configuration
                    .reference_requests
                    .requests()
                    .iter()
                    .position(|r| r.outcome() == SemanticReferenceRequestOutcome::Resolved)
                    .unwrap()
            } else {
                0
            };
            let old = &configuration.reference_requests.requests()[index];
            let value = if kind == "request_outcome" {
                let collected = Provenance::new(
                    None,
                    ProducerId::new("canonical.collection"),
                    FactOrigin::Parsed,
                    Confidence::Exact,
                    ResolutionState::Unresolved,
                );
                let resolved = Provenance::new(
                    None,
                    ProducerId::new("canonical.resolution"),
                    FactOrigin::Resolved,
                    Confidence::Exact,
                    ResolutionState::Resolved,
                );
                SemanticReferenceRequest::collected(
                    old.source_node().clone(),
                    old.category(),
                    old.reference().clone(),
                    old.expected_kinds().iter().copied(),
                    [collected],
                )
                .unwrap()
                .into_resolved(
                    EntityId::new(if kind == "request_candidate" {
                        "different.candidate"
                    } else {
                        "resolved.candidate"
                    })
                    .unwrap(),
                    old.expected_kinds()[0],
                    [resolved],
                )
                .unwrap()
            } else {
                SemanticReferenceRequest::reconstruct_terminal(
                    if kind == "request_source" {
                        EntityId::new("changed").unwrap()
                    } else {
                        old.source_node().clone()
                    },
                    if kind == "request_category" {
                        SemanticReferenceCategory::ExtensionTarget
                    } else {
                        old.category()
                    },
                    if kind == "request_reference" {
                        SemanticReference::Raw("changed".into())
                    } else {
                        old.reference().clone()
                    },
                    if kind == "request_expected" {
                        vec![NodeKind::Function]
                    } else {
                        old.expected_kinds().to_vec()
                    },
                    if kind == "request_candidate" {
                        vec![EntityId::new("different.candidate").unwrap()]
                    } else {
                        old.candidates().to_vec()
                    },
                    old.state(),
                    old.outcome(),
                    old.provenance().iter().map(|p| {
                        if kind == "request_producer" {
                            Provenance::new(
                                p.source().cloned(),
                                ProducerId::new("changed"),
                                p.origin(),
                                p.confidence(),
                                p.resolution(),
                            )
                        } else {
                            p.clone()
                        }
                    }),
                )
                .unwrap()
            };
            assert_ne!(&value, old, "{kind}");
            let mut values = configuration.reference_requests.requests().to_vec();
            values[index] = value;
            configuration.reference_requests =
                Arc::new(SemanticReferenceRequestLedger::from_requests(values).unwrap());
            return;
        }
        let mut graph = SemanticGraph::new();
        let target = configuration
            .graph
            .nodes()
            .find(|node| node.name().as_str() == "Changed")
            .unwrap()
            .id()
            .clone();
        let query = configuration
            .graph
            .nodes()
            .find(|node| {
                node.kind() == NodeKind::Query
                    && configuration.graph.edges().any(|edge| {
                        edge.kind() == EdgeKind::Contains
                            && edge.target() == node.id()
                            && (edge.source() == &target) != (kind == "query_other_callable")
                    })
            })
            .unwrap()
            .id()
            .clone();
        for node in configuration.graph.nodes() {
            let selected = node.id()
                == if kind.starts_with("query") {
                    &query
                } else {
                    &target
                };
            if selected && (kind == "missing_target" || kind == "query_missing") {
                continue;
            }
            let changed = if kind.starts_with("fact_") && node.id() == &target {
                use oneagent_graph::*;
                let mut provenance = node.provenance().to_vec();
                assert!(!provenance.is_empty());
                let p = &provenance[0];
                provenance[0] = Provenance::new_with_location(
                    if kind == "fact_source" {
                        Some(EntityId::new("different.source").unwrap())
                    } else {
                        p.source().cloned()
                    },
                    if kind == "fact_location" {
                        Some(oneagent_common::SourceLocation::new(
                            oneagent_common::SourcePath::new("different.bsl").unwrap(),
                            None,
                        ))
                    } else {
                        p.location().cloned()
                    },
                    if kind == "fact_producer" {
                        ProducerId::new("different")
                    } else {
                        p.producer().clone()
                    },
                    if kind == "fact_origin" {
                        FactOrigin::External
                    } else {
                        p.origin()
                    },
                    if kind == "fact_confidence" {
                        Confidence::Unknown
                    } else {
                        p.confidence()
                    },
                    if kind == "fact_resolution" {
                        ResolutionState::Ambiguous
                    } else {
                        p.resolution()
                    },
                );
                assert_ne!(provenance, node.provenance());
                Some(
                    GraphNode::new_with_payload_and_provenance(
                        node.id().clone(),
                        node.name().clone(),
                        node.kind(),
                        node.payload().clone(),
                        provenance,
                    )
                    .unwrap(),
                )
            } else if kind == "metadata_payload"
                && node.payload() != &oneagent_graph::GraphNodePayload::None
            {
                Some(GraphNode::new_with_provenance(
                    node.id().clone(),
                    node.name().clone(),
                    node.kind(),
                    node.provenance().to_vec(),
                ))
            } else if selected {
                match kind {
                    "name" | "query_binding" | "query_other_callable" => Some(
                        GraphNode::new_with_payload_and_provenance(
                            node.id().clone(),
                            EntityName::new("Other").unwrap(),
                            node.kind(),
                            node.payload().clone(),
                            node.provenance().to_vec(),
                        )
                        .unwrap(),
                    ),
                    "kind" => Some(GraphNode::new_with_provenance(
                        node.id().clone(),
                        node.name().clone(),
                        NodeKind::Function,
                        node.provenance().to_vec(),
                    )),
                    "provenance" | "query_provenance" => Some(
                        GraphNode::new_with_payload_and_provenance(
                            node.id().clone(),
                            node.name().clone(),
                            node.kind(),
                            node.payload().clone(),
                            Vec::new(),
                        )
                        .unwrap(),
                    ),
                    _ => None,
                }
            } else {
                None
            };
            graph.insert_node(changed.unwrap_or_else(|| node.clone()));
        }
        if kind == "query_extra" {
            let node = configuration.graph.node(&query).unwrap();
            graph.insert_node(
                GraphNode::new_with_payload_and_provenance(
                    EntityId::new("extra.query").unwrap(),
                    node.name().clone(),
                    node.kind(),
                    node.payload().clone(),
                    node.provenance().to_vec(),
                )
                .unwrap(),
            );
        }
        let mut changed_edge = false;
        let edge_kind = if kind.starts_with("reads_") {
            EdgeKind::Reads
        } else if kind.starts_with("depends_") {
            EdgeKind::DependsOn
        } else {
            EdgeKind::Calls
        };
        let edge_mutation =
            kind.starts_with("edge_") || kind.starts_with("reads_") || kind.starts_with("depends_");
        for edge in configuration.graph.edges() {
            if graph.node(edge.source()).is_none() || graph.node(edge.target()).is_none() {
                continue;
            }
            if kind == "query_owner" && edge.kind() == EdgeKind::Contains && edge.target() == &query
            {
                changed_edge = true;
                let other_owner = configuration
                    .graph
                    .nodes()
                    .find(|node| node.name().as_str() == "ExerciseSecurityCollection")
                    .unwrap();
                assert_ne!(edge.source(), other_owner.id());
                graph
                    .insert_edge(GraphEdge::new_with_provenance(
                        other_owner.id().clone(),
                        edge.target().clone(),
                        edge.kind(),
                        edge.provenance().to_vec(),
                    ))
                    .unwrap();
            } else if !changed_edge && edge.kind() == edge_kind && edge_mutation {
                changed_edge = true;
                if kind.ends_with("_missing") {
                    continue;
                }
                graph
                    .insert_edge(GraphEdge::new_with_provenance(
                        edge.source().clone(),
                        if kind.ends_with("_target") {
                            configuration
                                .graph
                                .nodes()
                                .find(|n| n.kind() == NodeKind::Module)
                                .unwrap()
                                .id()
                                .clone()
                        } else {
                            edge.target().clone()
                        },
                        edge.kind(),
                        Vec::new(),
                    ))
                    .unwrap();
            } else {
                graph.insert_edge(edge.clone()).unwrap();
            }
        }
        if edge_mutation || kind == "query_owner" {
            assert!(changed_edge);
        }
        configuration.graph = Arc::new(graph);
    }

    #[tokio::test]
    async fn semantic_candidate_faults_recover() {
        for kind in [
            "extra_configuration",
            "source_role",
            "source_path",
            "source_identity",
            "source_extra",
            "source_bytes",
            "source_omission",
            "source_ambiguity",
            "fact_source",
            "fact_location",
            "fact_producer",
            "fact_origin",
            "fact_confidence",
            "fact_resolution",
            "metadata_payload",
            "query_extra",
            "rule_not_applicable",
            "rule_failed",
            "rule_completed",
            "reads_missing",
            "reads_provenance",
            "reads_target",
            "depends_missing",
            "depends_provenance",
            "depends_target",
            "format",
            "report",
            "validation",
            "findings",
            "diagnostic_severity",
            "diagnostic_code",
            "diagnostic_kind",
            "diagnostic_expected",
            "diagnostic_candidates",
            "diagnostic_actual",
            "diagnostic_order",
            "diagnostic_message",
            "diagnostic_reference",
            "diagnostic_source",
            "diagnostic_provenance",
            "diagnostic_location",
            "request_source",
            "request_category",
            "request_expected",
            "request_candidate",
            "request_outcome",
            "request_reference",
            "request_producer",
            "root",
            "configuration",
            "documents",
            "missing_target",
            "name",
            "kind",
            "provenance",
            "query_missing",
            "query_binding",
            "query_other_callable",
            "query_owner",
            "query_provenance",
            "edge_missing",
            "edge_provenance",
            "diagnostics",
            "requests",
        ] {
            rejected(
                TestHooks {
                    resolved_query: kind.starts_with("reads_")
                        || kind.starts_with("depends_")
                        || kind == "request_candidate",
                    candidate: Some(Box::new(move |candidate| mutate(candidate, kind))),
                    ..TestHooks::default()
                },
                WorkspaceEditRecovery::Recovered,
            )
            .await;
        }
    }

    #[tokio::test]
    async fn structured_plan_and_capability_tampering_reject() {
        for kind in [
            "actor",
            "request",
            "direction",
            "plan",
            "arc",
            "baseline",
            "same_id_summary",
            "same_id_category",
            "policy_revision",
            "policy_arguments",
            "policy_effects",
            "confirmation",
            "attempt_id",
            "missing_policy",
            "expired_slot",
            "confirmation_cancelled",
            "actor_cancelled",
            "arc_cancelled",
        ] {
            let cancelled = kind.ends_with("_cancelled");
            let kind = kind.strip_suffix("_cancelled").unwrap_or(kind);
            let root = fixtures::fixture("edt");
            let service = WorkspaceService::new().with_edit_policy(
                fixtures::policy(RuleAction::RequireConfirmation),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            let (handle, observer, stop, task) =
                fixtures::start_service(root.path(), service).await;
            let before = observer.snapshot().unwrap();
            let (mut challenge, _) = handle
                .prepare_apply(
                    fixtures::request(&before, "Changed"),
                    fixtures::actor(),
                    fixtures::request_id(),
                )
                .await
                .unwrap();
            match kind {
                "attempt_id" => challenge.attempt.reservation.id += 1,
                "missing_policy" => {
                    challenge.attempt.policy = None;
                }
                "expired_slot" => {
                    handle.shared.admission.lock().unwrap().slot = None;
                }
                "policy_revision" | "policy_arguments" | "policy_effects" | "confirmation" => {
                    use oneagent_tool_policy::*;
                    let original = challenge.attempt.policy.as_ref().unwrap().request();
                    let request = ToolRequest::new(
                        original.id().clone(),
                        original.actor().clone(),
                        original.tool().clone(),
                        ToolArguments::new(
                            if kind == "policy_arguments" || kind == "confirmation" {
                                "changed"
                            } else {
                                original.arguments().expose()
                            },
                        )
                        .unwrap(),
                        if kind == "policy_effects" {
                            vec![ToolEffect::LocalMutation, ToolEffect::Destructive]
                        } else {
                            vec![ToolEffect::LocalMutation]
                        },
                    )
                    .unwrap();
                    let mut rules = fixtures::policy(RuleAction::RequireConfirmation)
                        .rules()
                        .to_vec();
                    rules.push(ToolRule::new(
                        ActorScope::Any,
                        ToolScope::Any,
                        ToolEffect::Destructive,
                        RuleAction::RequireConfirmation,
                    ));
                    let policy = ToolPolicy::new(
                        PolicyRevision::new(if kind == "policy_revision" {
                            "other"
                        } else {
                            "edit-test-1"
                        })
                        .unwrap(),
                        rules,
                    )
                    .unwrap();
                    let mut evaluated = policy.evaluate(request);
                    challenge.confirmation = evaluated.take_confirmation_challenge().unwrap();
                    if kind != "confirmation" {
                        challenge.attempt.policy = Some(evaluated);
                    }
                }
                "same_id_summary" | "same_id_category" => {
                    use oneagent_analysis::refactoring::{
                        RefactoringOperation, SourceOccurrenceKind,
                    };
                    let old = &challenge.attempt.plan;
                    let mut operations = old.operations().to_vec();
                    if kind == "same_id_summary" {
                        operations.push(operations[0].clone());
                    } else {
                        let operation = operations
                            .iter_mut()
                            .find(|o| o.occurrence_kind() == SourceOccurrenceKind::LocalCall)
                            .unwrap();
                        *operation = RefactoringOperation::new(
                            operation.kind(),
                            SourceOccurrenceKind::QualifiedCall,
                            operation.document_id().clone(),
                            operation.content_version(),
                            operation.range(),
                            operation.expected(),
                            operation.replacement(),
                            operation.dependencies(),
                        )
                        .unwrap();
                    }
                    let changed = RefactoringPlan::new(
                        old.request().clone(),
                        old.target().clone(),
                        old.preconditions().clone(),
                        operations,
                    )
                    .unwrap();
                    assert_eq!(old.id(), changed.id());
                    assert_ne!(old, &changed);
                    challenge.attempt.plan = changed;
                }
                "actor" => challenge.attempt.actor.push('x'),
                "request" => challenge.attempt.request_id.push('x'),
                "direction" => challenge.attempt.direction = Direction::Reverse,
                "plan" => {
                    challenge.attempt.plan = before
                        .plan_refactoring(
                            &fixtures::request(&before, "Other"),
                            &NeverCancelledRefactoring,
                        )
                        .unwrap()
                        .into_parts()
                        .0;
                }
                "arc" => {
                    challenge.attempt.previous =
                        Arc::new(WorkspaceSnapshotBuilder::new().build(root.path()).unwrap());
                }
                "baseline" => {
                    challenge.attempt.baseline = EditBaseline::capture(
                        &fixtures::fixture("edt").path().canonicalize().unwrap(),
                        &[],
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
            let cancellation = WorkspaceEditCancellation::new();
            if cancelled {
                cancellation.request();
            }
            assert!(!format!("{challenge:?}").contains("FillSecurityCollection"));
            let outcome = handle
                .checked_apply(challenge.confirm(), cancellation)
                .await;
            if cancelled {
                let expected = match kind {
                    "confirmation" => WorkspaceEditCause::ConfirmationRequired,
                    "actor" => WorkspaceEditCause::AuthorizationMismatch,
                    _ => WorkspaceEditCause::Cancelled,
                };
                assert!(
                    matches!(outcome, WorkspaceEditOutcome::Failed { cause, .. } if cause == expected),
                    "{kind}/cancelled: {outcome:?}"
                );
            }
            assert!(
                matches!(
                    outcome,
                    WorkspaceEditOutcome::Failed {
                        recovery: WorkspaceEditRecovery::NotNeeded,
                        ..
                    }
                ),
                "{kind}: {outcome:?}"
            );
            assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
            for document in before.configurations()[0].source_evidence().documents() {
                assert_eq!(
                    fs::read(root.path().join(document.path().path().as_str())).unwrap(),
                    document.raw_content()
                );
            }
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn policy_gate_is_exact_confirmed_and_side_effect_free() {
        struct GateSignal(bool);
        impl oneagent_tool_policy::ToolCancellationSignal for GateSignal {
            fn is_cancelled(&self) -> bool {
                self.0
            }
            fn cancelled(&self) -> oneagent_tool_policy::ToolFuture<'_, ()> {
                if self.0 {
                    Box::pin(async {})
                } else {
                    Box::pin(std::future::pending())
                }
            }
        }
        for cancelled in [false, true] {
            let request = ToolRequest::new(
                fixtures::request_id(),
                fixtures::actor(),
                ToolId::new("oneagent.workspace.edit.apply").unwrap(),
                ToolArguments::new("SECRET_SOURCE_TOKEN /absolute/private/path SECRET_DIGEST")
                    .unwrap(),
                [ToolEffect::LocalMutation],
            )
            .unwrap();
            let mut authorization =
                fixtures::policy(RuleAction::RequireConfirmation).evaluate(request);
            let confirmation = authorization
                .take_confirmation_challenge()
                .unwrap()
                .confirm();
            super::super::edit_io::faults::set(vec![]);
            let outcome = execute_tool(
                authorization,
                Some(confirmation),
                &EditPolicyGate,
                &GateSignal(cancelled),
            )
            .await;
            assert_eq!(
                outcome.audit().terminal_outcome(),
                if cancelled {
                    ToolTerminalOutcome::Cancelled
                } else {
                    ToolTerminalOutcome::Completed
                }
            );
            assert!(super::super::edit_io::faults::events().is_empty());
            let rendered = format!("{:?}", outcome.audit());
            for secret in [
                "SECRET_SOURCE_TOKEN",
                "/absolute/private/path",
                "SECRET_DIGEST",
            ] {
                assert!(!rendered.contains(secret));
            }
        }
        for (action, expected) in [
            (RuleAction::Deny, WorkspaceEditCause::PolicyDenied),
            (RuleAction::Allow, WorkspaceEditCause::ConfirmationRequired),
        ] {
            let root = fixtures::fixture("edt");
            let service = WorkspaceService::new().with_edit_policy(
                fixtures::policy(action),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            let (handle, observer, stop, task) =
                fixtures::start_service(root.path(), service).await;
            let before = observer.snapshot().unwrap();
            assert_eq!(
                handle
                    .prepare_apply(
                        fixtures::request(&before, "Changed"),
                        fixtures::actor(),
                        fixtures::request_id()
                    )
                    .await
                    .unwrap_err(),
                expected
            );
            assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
            assert_eq!(handle.shared.admission.lock().unwrap().slot, None);
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn cancellation_drop_and_shutdown_join() {
        for phase in [
            "staged",
            "replaced",
            "built",
            "compared",
            "cleaned",
            "final_guard",
            "restore_before",
        ] {
            let root = fixtures::fixture("edt");
            let (started, waiting) = oneshot::channel();
            let (release, blocked) = oneshot::channel();
            let mut service = WorkspaceService::new().with_edit_policy(
                fixtures::policy(RuleAction::RequireConfirmation),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            let gate: Box<dyn FnOnce() + Send + Sync> = Box::new(move || {
                started.send(()).unwrap();
                blocked.blocking_recv().unwrap();
            });
            if phase == "restore_before" {
                service.edits.test_hooks.fail = Some("built");
                service.edits.test_hooks.restore_gate = Some(gate);
            } else {
                service.edits.test_hooks.phase_gate = Some((phase, gate));
            }
            let (handle, observer, stop, task) =
                fixtures::start_service(root.path(), service).await;
            let before = observer.snapshot().unwrap();
            let (challenge, _) = handle
                .prepare_apply(
                    fixtures::request(&before, "Changed"),
                    fixtures::actor(),
                    fixtures::request_id(),
                )
                .await
                .unwrap();
            let submitting = handle.clone();
            let response = tokio::spawn(async move {
                submitting
                    .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                    .await
            });
            tokio::time::timeout(std::time::Duration::from_secs(10), waiting)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                handle
                    .prepare_apply(
                        fixtures::request(&before, "Other"),
                        fixtures::actor(),
                        fixtures::request_id()
                    )
                    .await
                    .unwrap_err(),
                WorkspaceEditCause::Busy
            );
            response.abort();
            stop.send(()).unwrap();
            tokio::task::yield_now().await;
            assert!(!task.is_finished(), "shutdown must join the {phase} worker");
            release.send(()).unwrap();
            tokio::time::timeout(std::time::Duration::from_secs(10), task)
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(observer.snapshot().is_none());
            for document in before.configurations()[0].source_evidence().documents() {
                assert_eq!(
                    fs::read(root.path().join(document.path().path().as_str())).unwrap(),
                    document.raw_content()
                );
            }
        }
        for after_build in [false, true] {
            let root = fixtures::fixture("edt");
            let cancellation = WorkspaceEditCancellation::new();
            let signal = cancellation.clone();
            let mut service = WorkspaceService::new().with_edit_policy(
                fixtures::policy(RuleAction::RequireConfirmation),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            if after_build {
                service.edits.test_hooks.candidate = Some(Box::new(move |_| signal.request()));
            }
            let (handle, observer, stop, task) =
                fixtures::start_service(root.path(), service).await;
            let before = observer.snapshot().unwrap();
            let (challenge, _) = handle
                .prepare_apply(
                    fixtures::request(&before, "Changed"),
                    fixtures::actor(),
                    fixtures::request_id(),
                )
                .await
                .unwrap();
            if !after_build {
                cancellation.request();
            }
            let outcome = handle
                .checked_apply(challenge.confirm(), cancellation)
                .await;
            assert!(
                matches!(outcome, WorkspaceEditOutcome::Failed { cause: WorkspaceEditCause::Cancelled, recovery, .. }
                if recovery == if after_build { WorkspaceEditRecovery::Recovered } else { WorkspaceEditRecovery::NotNeeded }),
                "{outcome:?}"
            );
            assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
            assert_eq!(
                handle
                    .prepare_apply(
                        fixtures::request(&before, "Changed"),
                        fixtures::actor(),
                        fixtures::request_id()
                    )
                    .await
                    .unwrap_err(),
                WorkspaceEditCause::Stopped
            );
        }
    }

    #[test]
    fn closed_precedence_and_redaction() {
        let mut coordinator = EditCoordinator::new();
        let mut state = coordinator.handle.shared.admission.lock().unwrap();
        state.poisoned = true;
        state.stopped = true;
        state.writer = true;
        drop(state);
        assert!(matches!(
            coordinator.handle.reserve_attempt(),
            Err(WorkspaceEditCause::RecoveryRequired)
        ));
        coordinator.handle.shared.admission.lock().unwrap().poisoned = false;
        assert!(matches!(
            coordinator.handle.reserve_attempt(),
            Err(WorkspaceEditCause::Stopped)
        ));
        coordinator.shutdown();
        let rendered = format!(
            "{:?} {:?} {}",
            coordinator.handle(),
            WorkspaceEditOutcome::failure(WorkspaceEditCause::SourceChanged),
            WorkspaceEditCause::IoFailed
        );
        assert!(!rendered.contains(env!("CARGO_MANIFEST_DIR")));
        assert!(!rendered.contains("FillSecurityCollection"));
    }

    #[tokio::test]
    async fn publication_baseline_admission() {
        {
            let root = fixtures::fixture("edt");
            fs::File::create(root.path().join("over-bound-unrecognized.bin"))
                .unwrap()
                .set_len((super::super::edit_io::MAX_FILE + 1) as u64)
                .unwrap();
            let service = WorkspaceService::new().with_edit_policy(
                fixtures::policy(RuleAction::RequireConfirmation),
                WorkspaceEditOwnership::ExclusiveCooperative,
            );
            let (handle, observer, stop, task) =
                fixtures::start_service(root.path(), service).await;
            let before = observer.snapshot().unwrap();
            assert_eq!(
                handle
                    .prepare_apply(
                        fixtures::request(&before, "Changed"),
                        fixtures::actor(),
                        fixtures::request_id()
                    )
                    .await
                    .unwrap_err(),
                WorkspaceEditCause::Unavailable
            );
            assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
        }
        {
            let root = fixtures::fixture("edt");
            let before = WorkspaceSnapshotBuilder::new().build(root.path()).unwrap();
            let plan = before
                .plan_refactoring(
                    &fixtures::request(&before, "Changed"),
                    &NeverCancelledRefactoring,
                )
                .unwrap()
                .into_parts()
                .0;
            let baseline =
                EditBaseline::capture(&root.path().canonicalize().unwrap(), &[]).unwrap();
            for kind in ["configuration_root", "missing_document", "cache_source"] {
                let mut changed = before.clone();
                let configuration = &mut changed.configurations[0];
                if kind == "configuration_root" {
                    configuration.root_path.push("absent");
                } else {
                    use oneagent_analysis::refactoring::*;
                    let mut documents = configuration.source_evidence.documents().to_vec();
                    let doc = &documents[0];
                    documents[0] = SourceDocument::new(
                        doc.id().clone(),
                        doc.format(),
                        doc.module_role(),
                        ConfinedSourcePath::new_at_workspace_root(
                            oneagent_common::SourcePath::new(if kind == "cache_source" {
                                super::super::edit_io::CACHE_PATH
                            } else {
                                "missing.bsl"
                            })
                            .unwrap(),
                        )
                        .unwrap(),
                        doc.raw_content().to_vec(),
                        doc.occurrences().to_vec(),
                        doc.completeness(),
                    )
                    .unwrap();
                    configuration.source_evidence =
                        SourceEvidenceSet::new(configuration.configuration_id.clone(), documents)
                            .unwrap();
                }
                super::super::edit_io::faults::set(vec![]);
                assert!(
                    check_admission(&plan, &changed, &baseline).is_err(),
                    "{kind}"
                );
                assert!(!super::super::edit_io::faults::events().contains(&"create"));
            }
        }
        for mutate_at in [1, 2] {
            let root = fixtures::fixture("edt");
            let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let builder = WorkspaceSnapshotBuilder::with_detector(MutatingEditDetector {
                calls: Arc::clone(&calls),
                mutate_at,
                fail: false,
            });
            let (_ticks, receive) = mpsc::channel(1);
            let service = WorkspaceService::with_builder(builder)
                .with_edit_policy(
                    fixtures::policy(RuleAction::RequireConfirmation),
                    WorkspaceEditOwnership::ExclusiveCooperative,
                )
                .with_controlled_change_ticks(receive);
            let updates = service.update_observer();
            let input = service.change_input_handle();
            if mutate_at == 1 {
                let observer = service.snapshot_observer();
                let app = crate::App::builder()
                    .configure(&fixtures::Provider(root.path().to_owned()))
                    .unwrap()
                    .register_service("workspace", service)
                    .unwrap()
                    .build()
                    .unwrap();
                let (_stop, signal) = oneshot::channel();
                assert!(
                    tokio::time::timeout(std::time::Duration::from_secs(10), app.run(signal))
                        .await
                        .unwrap()
                        .is_err()
                );
                assert!(observer.snapshot().is_none());
            } else {
                let (handle, observer, stop, task) =
                    fixtures::start_service(root.path(), service).await;
                let before = observer.snapshot().unwrap();
                let path =
                    super::super::RepositoryChangePath::new("changed-during-build.txt").unwrap();
                let change = super::super::RepositoryChange::new(
                    super::super::RepositoryChangeKind::Modified,
                    Some(path.clone()),
                    Some(path),
                )
                .unwrap();
                let mut statuses = updates.subscribe();
                assert_eq!(
                    input.submit(
                        super::super::GitChangeSet::new(
                            super::super::GitCommitId::new(
                                "0123456789abcdef0123456789abcdef01234567"
                            )
                            .unwrap(),
                            [change]
                        )
                        .unwrap()
                    ),
                    super::super::WorkspaceChangeSubmissionOutcome::Accepted
                );
                tokio::time::timeout(std::time::Duration::from_secs(10), async {
                    while statuses.borrow().phase() != super::super::WorkspaceUpdatePhase::Failed {
                        statuses.changed().await.unwrap();
                    }
                })
                .await
                .unwrap();
                assert!(Arc::ptr_eq(&before, &observer.snapshot().unwrap()));
                assert_eq!(
                    handle
                        .prepare_apply(
                            fixtures::request(&before, "Changed"),
                            fixtures::actor(),
                            fixtures::request_id()
                        )
                        .await
                        .unwrap_err(),
                    WorkspaceEditCause::SourceChanged
                );
                stop.send(()).unwrap();
                task.await.unwrap().unwrap();
            }
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), mutate_at);
        }
        let root = fixtures::fixture("edt");
        let mut coordinator = EditCoordinator::new();
        coordinator.configure(
            fixtures::policy(RuleAction::RequireConfirmation),
            WorkspaceEditOwnership::ExclusiveCooperative,
        );
        coordinator.publish_baseline(None);
        assert!(matches!(
            coordinator.handle.reserve_attempt(),
            Err(WorkspaceEditCause::Unavailable)
        ));
        coordinator.publish_baseline(Some(
            EditBaseline::capture(&root.path().canonicalize().unwrap(), &[]).unwrap(),
        ));
        let reservation = coordinator.handle.reserve_attempt().unwrap();
        coordinator.publish_baseline(None);
        assert_eq!(
            coordinator.handle.shared.admission.lock().unwrap().slot,
            None
        );
        drop(reservation);
        assert!(matches!(
            coordinator.handle.reserve_attempt(),
            Err(WorkspaceEditCause::Unavailable)
        ));
    }

    #[tokio::test]
    async fn cache_namespace_and_scan_exclusions() {
        for mutate_load in [true, false] {
            let root = fixtures::fixture("edt");
            let (_, _, stop, task) =
                fixtures::start_service(root.path(), WorkspaceService::new()).await;
            stop.send(()).unwrap();
            task.await.unwrap().unwrap();
            let (entered, waiting) = oneshot::channel();
            let (release, blocked) = oneshot::channel();
            let gate: Box<dyn FnOnce() + Send> = Box::new(move || {
                entered.send(()).unwrap();
                blocked.blocking_recv().unwrap();
            });
            let cache = Arc::new(EditCacheProbe {
                root: root.path().to_owned(),
                store: super::super::cache::WorkspaceCacheStore::new(root.path().to_owned()),
                mutate_load,
                write_gate: Mutex::new(if mutate_load { None } else { Some(gate) }),
            });
            let service = WorkspaceService::new()
                .with_edit_policy(
                    fixtures::policy(RuleAction::RequireConfirmation),
                    WorkspaceEditOwnership::ExclusiveCooperative,
                )
                .with_cache_storage(cache);
            if mutate_load {
                let observer = service.snapshot_observer();
                let app = crate::App::builder()
                    .configure(&fixtures::Provider(root.path().to_owned()))
                    .unwrap()
                    .register_service("workspace", service)
                    .unwrap()
                    .build()
                    .unwrap();
                let (_stop, signal) = oneshot::channel();
                assert!(
                    tokio::time::timeout(std::time::Duration::from_secs(10), app.run(signal))
                        .await
                        .unwrap()
                        .is_err()
                );
                assert!(observer.snapshot().is_none());
            } else {
                let (handle, observer, stop, task) =
                    fixtures::start_service(root.path(), service).await;
                let before = observer.snapshot().unwrap();
                let (challenge, _) = handle
                    .prepare_apply(
                        fixtures::request(&before, "Changed"),
                        fixtures::actor(),
                        fixtures::request_id(),
                    )
                    .await
                    .unwrap();
                let submitting = handle.clone();
                let response = tokio::spawn(async move {
                    submitting
                        .checked_apply(challenge.confirm(), WorkspaceEditCancellation::new())
                        .await
                });
                tokio::time::timeout(std::time::Duration::from_secs(10), waiting)
                    .await
                    .unwrap()
                    .unwrap();
                let committed = observer.snapshot().unwrap();
                assert_eq!(committed.publication_id().get(), 2);
                assert!(!response.is_finished());
                assert_eq!(
                    handle
                        .prepare_apply(
                            fixtures::request(&before, "Other"),
                            fixtures::actor(),
                            fixtures::request_id()
                        )
                        .await
                        .unwrap_err(),
                    WorkspaceEditCause::Busy
                );
                stop.send(()).unwrap();
                tokio::task::yield_now().await;
                assert!(!task.is_finished());
                release.send(()).unwrap();
                assert!(matches!(
                    response.await.unwrap(),
                    WorkspaceEditOutcome::Applied { .. }
                ));
                task.await.unwrap().unwrap();
                for document in committed.configurations()[0].source_evidence().documents() {
                    assert_eq!(
                        fs::read(root.path().join(document.path().path().as_str())).unwrap(),
                        document.raw_content()
                    );
                }
            }
        }
        let root = fixtures::fixture("edt");
        fs::create_dir_all(root.path().join(".oneagent/cache")).unwrap();
        fs::write(
            root.path().join(".oneagent/cache/workspace-v1.json"),
            b"cache",
        )
        .unwrap();
        let baseline = EditBaseline::capture(&root.path().canonicalize().unwrap(), &[]).unwrap();
        fs::write(
            root.path().join(".oneagent/cache/workspace-v1.json"),
            b"different cache",
        )
        .unwrap();
        assert!(baseline.equals(&EditBaseline::capture(baseline.root(), &[]).unwrap()));
        fs::write(root.path().join(".oneagent/unknown"), b"not excluded").unwrap();
        assert!(!baseline.equals(&EditBaseline::capture(baseline.root(), &[]).unwrap()));
        fs::remove_file(root.path().join(".oneagent/cache/workspace-v1.json")).unwrap();
        fs::create_dir(root.path().join(".oneagent/cache/workspace-v1.json")).unwrap();
        assert!(EditBaseline::capture(baseline.root(), &[]).is_err());
    }
}
