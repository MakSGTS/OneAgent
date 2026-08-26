//! Confirmation-gated execution, cancellation, terminal results, and audit evidence.

use std::{
    collections::BTreeSet,
    fmt::{Debug, Formatter},
    future::{Future, poll_fn},
    pin::Pin,
    task::Poll,
};

use crate::{
    ActorId, AuthorizationDecisionKind, AuthorizationDecisionReason, PolicyRevision,
    ToolAuthorization, ToolConfirmation, ToolEffect, ToolId, ToolPolicyError, ToolPolicyErrorKind,
    ToolRequest, ToolRequestId,
};

/// Maximum executor output size in UTF-8 bytes.
pub const MAX_TOOL_OUTPUT_BYTES: usize = 65_536;

/// Maximum retained executor diagnostic size in UTF-8 bytes.
pub const MAX_TOOL_DIAGNOSTIC_BYTES: usize = 512;

/// Boxed borrowed `Send` future used by execution boundary interfaces.
pub type ToolFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Receiver-only cancellation interface for one accepted attempt.
pub trait ToolCancellationSignal: Send + Sync {
    /// Returns whether cancellation is already requested.
    fn is_cancelled(&self) -> bool;

    /// Resolves when cancellation is requested.
    fn cancelled(&self) -> ToolFuture<'_, ()>;
}

/// Stateless cancellation signal that never requests cancellation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NeverCancelled;

impl ToolCancellationSignal for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }

    fn cancelled(&self) -> ToolFuture<'_, ()> {
        Box::pin(std::future::pending())
    }
}

/// Bounded executor output with explicit content access.
pub struct ToolOutput(Box<str>);

impl ToolOutput {
    /// Creates bounded output while preserving every accepted UTF-8 byte.
    ///
    /// Empty output is valid.
    ///
    /// # Errors
    ///
    /// Returns [`ToolPolicyErrorKind::InvalidOutput`] when the byte maximum is
    /// exceeded without retaining or reporting the rejected value.
    pub fn new(value: impl Into<String>) -> Result<Self, ToolPolicyError> {
        let value = value.into();
        if value.len() > MAX_TOOL_OUTPUT_BYTES {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidOutput,
                "tool output exceeds byte limit",
            ));
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns exact accepted output through an explicit access point.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Returns the accepted UTF-8 byte length.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.0.len()
    }
}

impl Debug for ToolOutput {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolOutput")
            .field("bytes", &self.byte_len())
            .finish_non_exhaustive()
    }
}

/// Bounded non-empty executor diagnostic with explicit content access.
pub struct ToolDiagnostic(Box<str>);

impl ToolDiagnostic {
    /// Creates one bounded diagnostic.
    ///
    /// # Errors
    ///
    /// Returns [`ToolPolicyErrorKind::InvalidDiagnostic`] when the value is
    /// empty, contains only whitespace, or exceeds the byte maximum.
    pub fn new(value: impl Into<String>) -> Result<Self, ToolPolicyError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidDiagnostic,
                "tool diagnostic is empty",
            ));
        }
        if value.len() > MAX_TOOL_DIAGNOSTIC_BYTES {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidDiagnostic,
                "tool diagnostic exceeds byte limit",
            ));
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns exact accepted diagnostic through an explicit access point.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Returns the accepted UTF-8 byte length.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.0.len()
    }
}

impl Debug for ToolDiagnostic {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolDiagnostic")
            .field("bytes", &self.byte_len())
            .finish_non_exhaustive()
    }
}

/// Closed outcome reported by a substitutable executor.
#[derive(Debug)]
pub enum ToolExecutorOutcome {
    /// The attempt completed and returned output.
    Completed(ToolOutput),
    /// The executor reports partial effect before failure.
    Partial(ToolOutput),
    /// The attempt failed with an optional bounded diagnostic.
    Failed(Option<ToolDiagnostic>),
    /// The executor reports that its owner-enforced timeout elapsed.
    TimedOut,
}

/// Substitutable one-attempt executor boundary.
pub trait ToolExecutor: Send + Sync {
    /// Borrows the exact validated request and cancellation signal for one attempt.
    fn execute<'a>(
        &'a self,
        request: &'a ToolRequest,
        cancellation: &'a dyn ToolCancellationSignal,
    ) -> ToolFuture<'a, ToolExecutorOutcome>;
}

/// Confirmation observation recorded for one gate call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationState {
    /// Confirmation was not required and none was supplied.
    NotRequired,
    /// Required confirmation was not supplied.
    Missing,
    /// Exact required confirmation was supplied.
    Confirmed,
    /// Supplied confirmation was rejected.
    Rejected,
    /// Confirmation was irrelevant to an authorization denial.
    NotApplicable,
}

/// Stable reason for a gate-level denial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolDenialReason {
    /// Policy denied the request.
    AuthorizationDenied,
    /// Required confirmation was absent.
    ConfirmationMissing,
    /// Confirmation did not bind the current decision and request.
    ConfirmationMismatch,
    /// Confirmation was supplied to a decision that did not require it.
    UnexpectedConfirmation,
}

/// Closed terminal execution outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolTerminalOutcome {
    /// Work was rejected before execution.
    Denied(ToolDenialReason),
    /// The attempt completed.
    Completed,
    /// The executor reports partial effect before failure.
    Partial,
    /// The attempt failed.
    Failed,
    /// The executor reports owner-enforced timeout.
    TimedOut,
    /// Cancellation won before an executor outcome.
    Cancelled,
}

/// Bounded content-free correlation evidence for one gate call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolAuditRecord {
    request_id: ToolRequestId,
    actor: ActorId,
    tool: ToolId,
    policy_revision: PolicyRevision,
    effects: BTreeSet<ToolEffect>,
    argument_bytes: usize,
    authorization_kind: AuthorizationDecisionKind,
    authorization_reason: AuthorizationDecisionReason,
    confirmation_state: ConfirmationState,
    attempt_count: u8,
    terminal_outcome: ToolTerminalOutcome,
    output_bytes: usize,
}

impl ToolAuditRecord {
    /// Returns the request identity.
    #[must_use]
    pub const fn request_id(&self) -> &ToolRequestId {
        &self.request_id
    }
    /// Returns the actor label.
    #[must_use]
    pub const fn actor(&self) -> &ActorId {
        &self.actor
    }
    /// Returns the tool identity.
    #[must_use]
    pub const fn tool(&self) -> &ToolId {
        &self.tool
    }
    /// Returns the evaluated policy revision.
    #[must_use]
    pub const fn policy_revision(&self) -> &PolicyRevision {
        &self.policy_revision
    }
    /// Returns canonical declared effects.
    #[must_use]
    pub const fn effects(&self) -> &BTreeSet<ToolEffect> {
        &self.effects
    }
    /// Returns argument byte count without retaining argument content.
    #[must_use]
    pub const fn argument_bytes(&self) -> usize {
        self.argument_bytes
    }
    /// Returns the authorization kind.
    #[must_use]
    pub const fn authorization_kind(&self) -> AuthorizationDecisionKind {
        self.authorization_kind
    }
    /// Returns the authorization reason.
    #[must_use]
    pub const fn authorization_reason(&self) -> AuthorizationDecisionReason {
        self.authorization_reason
    }
    /// Returns the confirmation observation.
    #[must_use]
    pub const fn confirmation_state(&self) -> ConfirmationState {
        self.confirmation_state
    }
    /// Returns the attempt count, constrained to zero or one.
    #[must_use]
    pub const fn attempt_count(&self) -> u8 {
        self.attempt_count
    }
    /// Returns the terminal outcome.
    #[must_use]
    pub const fn terminal_outcome(&self) -> ToolTerminalOutcome {
        self.terminal_outcome
    }
    /// Returns output byte count without retaining output content.
    #[must_use]
    pub const fn output_bytes(&self) -> usize {
        self.output_bytes
    }
}

/// Owned terminal result for every execution gate call.
pub struct ToolExecutionResult {
    audit: ToolAuditRecord,
    output: Option<ToolOutput>,
    diagnostic: Option<ToolDiagnostic>,
}

impl ToolExecutionResult {
    /// Returns safe audit evidence.
    #[must_use]
    pub const fn audit(&self) -> &ToolAuditRecord {
        &self.audit
    }
    /// Returns explicitly accessible output when present.
    #[must_use]
    pub const fn output(&self) -> Option<&ToolOutput> {
        self.output.as_ref()
    }
    /// Returns explicitly accessible diagnostic when present.
    #[must_use]
    pub const fn diagnostic(&self) -> Option<&ToolDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl Debug for ToolExecutionResult {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolExecutionResult")
            .field("audit", &self.audit)
            .field("has_output", &self.output.is_some())
            .field("has_diagnostic", &self.diagnostic.is_some())
            .finish_non_exhaustive()
    }
}

/// Consumes authorization and optional confirmation, then performs at most one attempt.
#[must_use]
pub fn execute_tool<'a>(
    authorization: ToolAuthorization,
    confirmation: Option<ToolConfirmation>,
    executor: &'a dyn ToolExecutor,
    cancellation: &'a dyn ToolCancellationSignal,
) -> ToolFuture<'a, ToolExecutionResult> {
    Box::pin(async move {
        let confirmation_state = match check_confirmation(&authorization, confirmation) {
            Ok(state) => state,
            Err((state, reason)) => {
                return terminal(
                    &authorization,
                    state,
                    0,
                    ToolTerminalOutcome::Denied(reason),
                    None,
                    None,
                );
            }
        };

        if cancellation.is_cancelled() {
            return terminal(
                &authorization,
                confirmation_state,
                0,
                ToolTerminalOutcome::Cancelled,
                None,
                None,
            );
        }

        let executor_outcome = {
            let mut execution = executor.execute(authorization.request(), cancellation);
            let mut cancelled = cancellation.cancelled();
            poll_fn(|context| {
                if cancellation.is_cancelled() || cancelled.as_mut().poll(context).is_ready() {
                    return Poll::Ready(None);
                }
                execution.as_mut().poll(context).map(Some)
            })
            .await
        };

        match executor_outcome {
            None => terminal(
                &authorization,
                confirmation_state,
                1,
                ToolTerminalOutcome::Cancelled,
                None,
                None,
            ),
            Some(ToolExecutorOutcome::Completed(output)) => terminal(
                &authorization,
                confirmation_state,
                1,
                ToolTerminalOutcome::Completed,
                Some(output),
                None,
            ),
            Some(ToolExecutorOutcome::Partial(output)) => terminal(
                &authorization,
                confirmation_state,
                1,
                ToolTerminalOutcome::Partial,
                Some(output),
                None,
            ),
            Some(ToolExecutorOutcome::Failed(diagnostic)) => terminal(
                &authorization,
                confirmation_state,
                1,
                ToolTerminalOutcome::Failed,
                None,
                diagnostic,
            ),
            Some(ToolExecutorOutcome::TimedOut) => terminal(
                &authorization,
                confirmation_state,
                1,
                ToolTerminalOutcome::TimedOut,
                None,
                None,
            ),
        }
    })
}

fn check_confirmation(
    authorization: &ToolAuthorization,
    confirmation: Option<ToolConfirmation>,
) -> Result<ConfirmationState, (ConfirmationState, ToolDenialReason)> {
    match authorization.kind() {
        AuthorizationDecisionKind::Deny => Err((
            if confirmation.is_some() {
                ConfirmationState::Rejected
            } else {
                ConfirmationState::NotApplicable
            },
            ToolDenialReason::AuthorizationDenied,
        )),
        AuthorizationDecisionKind::RequireConfirmation => match confirmation {
            None => Err((
                ConfirmationState::Missing,
                ToolDenialReason::ConfirmationMissing,
            )),
            Some(confirmation) if confirmation.matches(authorization) => {
                Ok(ConfirmationState::Confirmed)
            }
            Some(_) => Err((
                ConfirmationState::Rejected,
                ToolDenialReason::ConfirmationMismatch,
            )),
        },
        AuthorizationDecisionKind::Allow if confirmation.is_some() => Err((
            ConfirmationState::Rejected,
            ToolDenialReason::UnexpectedConfirmation,
        )),
        AuthorizationDecisionKind::Allow => Ok(ConfirmationState::NotRequired),
    }
}

fn terminal(
    authorization: &ToolAuthorization,
    confirmation_state: ConfirmationState,
    attempt_count: u8,
    terminal_outcome: ToolTerminalOutcome,
    output: Option<ToolOutput>,
    diagnostic: Option<ToolDiagnostic>,
) -> ToolExecutionResult {
    let request = authorization.request();
    let output_bytes = output.as_ref().map_or(0, ToolOutput::byte_len);
    ToolExecutionResult {
        audit: ToolAuditRecord {
            request_id: request.id().clone(),
            actor: request.actor().clone(),
            tool: request.tool().clone(),
            policy_revision: authorization.policy_revision().clone(),
            effects: request.effects().clone(),
            argument_bytes: request.arguments().byte_len(),
            authorization_kind: authorization.kind(),
            authorization_reason: authorization.reason(),
            confirmation_state,
            attempt_count,
            terminal_outcome,
            output_bytes,
        },
        output,
        diagnostic,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        future::{Future, pending},
        pin::Pin,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        task::{Context, Poll, Waker},
    };

    use super::{
        ConfirmationState, MAX_TOOL_DIAGNOSTIC_BYTES, MAX_TOOL_OUTPUT_BYTES, NeverCancelled,
        ToolCancellationSignal, ToolDenialReason, ToolDiagnostic, ToolExecutor,
        ToolExecutorOutcome, ToolFuture, ToolOutput, ToolTerminalOutcome, execute_tool,
    };
    use crate::{
        ActorId, ActorScope, PolicyRevision, RuleAction, ToolArguments, ToolEffect, ToolId,
        ToolPolicy, ToolPolicyErrorKind, ToolRequest, ToolRequestId, ToolRule, ToolScope,
    };

    #[derive(Clone, Copy)]
    enum Mode {
        Completed,
        Partial,
        Failed,
        TimedOut,
    }

    struct FakeExecutor {
        calls: AtomicUsize,
        mode: Mode,
    }

    impl FakeExecutor {
        const fn new(mode: Mode) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                mode,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl ToolExecutor for FakeExecutor {
        fn execute<'a>(
            &'a self,
            _request: &'a ToolRequest,
            _cancellation: &'a dyn ToolCancellationSignal,
        ) -> ToolFuture<'a, ToolExecutorOutcome> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                match self.mode {
                    Mode::Completed => ToolExecutorOutcome::Completed(
                        ToolOutput::new("completed-sensitive-output").expect("output must pass"),
                    ),
                    Mode::Partial => ToolExecutorOutcome::Partial(
                        ToolOutput::new("partial-sensitive-output").expect("output must pass"),
                    ),
                    Mode::Failed => ToolExecutorOutcome::Failed(Some(
                        ToolDiagnostic::new("bounded-sensitive-diagnostic")
                            .expect("diagnostic must pass"),
                    )),
                    Mode::TimedOut => ToolExecutorOutcome::TimedOut,
                }
            })
        }
    }

    struct PreCancelled;

    impl ToolCancellationSignal for PreCancelled {
        fn is_cancelled(&self) -> bool {
            true
        }

        fn cancelled(&self) -> ToolFuture<'_, ()> {
            Box::pin(async {})
        }
    }

    struct FlipCancellation {
        queries: AtomicUsize,
    }

    impl FlipCancellation {
        const fn new() -> Self {
            Self {
                queries: AtomicUsize::new(0),
            }
        }
    }

    impl ToolCancellationSignal for FlipCancellation {
        fn is_cancelled(&self) -> bool {
            self.queries.fetch_add(1, Ordering::SeqCst) > 0
        }

        fn cancelled(&self) -> ToolFuture<'_, ()> {
            Box::pin(async {})
        }
    }

    struct PendingExecutor {
        calls: AtomicUsize,
        active: Arc<AtomicUsize>,
    }

    struct ActiveGuard(Arc<AtomicUsize>);

    impl Drop for ActiveGuard {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }

    impl ToolExecutor for PendingExecutor {
        fn execute<'a>(
            &'a self,
            _request: &'a ToolRequest,
            _cancellation: &'a dyn ToolCancellationSignal,
        ) -> ToolFuture<'a, ToolExecutorOutcome> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.active.fetch_add(1, Ordering::SeqCst);
            let guard = ActiveGuard(Arc::clone(&self.active));
            Box::pin(async move {
                let _guard = guard;
                pending::<ToolExecutorOutcome>().await
            })
        }
    }

    struct ManualCancellation {
        cancelled: AtomicBool,
    }

    impl ManualCancellation {
        const fn new() -> Self {
            Self {
                cancelled: AtomicBool::new(false),
            }
        }

        fn cancel(&self) {
            self.cancelled.store(true, Ordering::SeqCst);
        }
    }

    impl ToolCancellationSignal for ManualCancellation {
        fn is_cancelled(&self) -> bool {
            self.cancelled.load(Ordering::SeqCst)
        }

        fn cancelled(&self) -> ToolFuture<'_, ()> {
            Box::pin(std::future::poll_fn(|context| {
                if self.is_cancelled() {
                    Poll::Ready(())
                } else {
                    context.waker().wake_by_ref();
                    Poll::Pending
                }
            }))
        }
    }

    fn request(id: &str, arguments: &str, effect: ToolEffect) -> ToolRequest {
        ToolRequest::new(
            ToolRequestId::new(id).expect("request ID must pass"),
            ActorId::new("actor-1").expect("actor must pass"),
            ToolId::new("tool-1").expect("tool must pass"),
            ToolArguments::new(arguments).expect("arguments must pass"),
            [effect],
        )
        .expect("request must pass")
    }

    fn policy(effect: ToolEffect, action: RuleAction) -> ToolPolicy {
        ToolPolicy::new(
            PolicyRevision::new("revision-1").expect("revision must pass"),
            vec![ToolRule::new(
                ActorScope::Any,
                ToolScope::Any,
                effect,
                action,
            )],
        )
        .expect("policy must pass")
    }

    fn allow(id: &str) -> crate::ToolAuthorization {
        policy(ToolEffect::ReadOnly, RuleAction::Allow).evaluate(request(
            id,
            "synthetic-sensitive-arguments",
            ToolEffect::ReadOnly,
        ))
    }

    fn require_confirmation(id: &str, arguments: &str) -> crate::ToolAuthorization {
        policy(
            ToolEffect::ExternalMutation,
            RuleAction::RequireConfirmation,
        )
        .evaluate(request(id, arguments, ToolEffect::ExternalMutation))
    }

    fn block_on_ready<T>(mut future: Pin<Box<dyn Future<Output = T> + '_>>) -> T {
        let mut context = Context::from_waker(Waker::noop());
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("future must be ready"),
        }
    }

    #[test]
    fn output_and_diagnostic_bounds_and_debug_are_redacted() {
        let output_sentinel = "output-sensitive-Привет";
        let output = ToolOutput::new(output_sentinel).expect("output must pass");
        assert_eq!(output.expose(), output_sentinel);
        assert!(!format!("{output:?}").contains(output_sentinel));
        assert_eq!(
            ToolOutput::new("x".repeat(MAX_TOOL_OUTPUT_BYTES + 1))
                .expect_err("oversized output must fail")
                .kind(),
            ToolPolicyErrorKind::InvalidOutput
        );

        let diagnostic_sentinel = "diagnostic-sensitive-Привет";
        let diagnostic = ToolDiagnostic::new(diagnostic_sentinel).expect("diagnostic must pass");
        assert_eq!(diagnostic.expose(), diagnostic_sentinel);
        assert!(!format!("{diagnostic:?}").contains(diagnostic_sentinel));
        assert_eq!(
            ToolDiagnostic::new(" \n\t")
                .expect_err("blank diagnostic must fail")
                .kind(),
            ToolPolicyErrorKind::InvalidDiagnostic
        );
        assert_eq!(
            ToolDiagnostic::new("я".repeat(MAX_TOOL_DIAGNOSTIC_BYTES / "я".len() + 1))
                .expect_err("oversized diagnostic must fail")
                .kind(),
            ToolPolicyErrorKind::InvalidDiagnostic
        );
    }

    #[test]
    fn challenge_is_available_only_once_and_only_when_required() {
        let mut authorization = require_confirmation("request-1", "arguments");
        let challenge = authorization
            .take_confirmation_challenge()
            .expect("first challenge must pass");
        assert!(!format!("{challenge:?}").contains("arguments"));
        assert_eq!(
            authorization
                .take_confirmation_challenge()
                .expect_err("second challenge must fail")
                .kind(),
            ToolPolicyErrorKind::InvalidConfirmation
        );

        let mut allowed = allow("request-2");
        assert_eq!(
            allowed
                .take_confirmation_challenge()
                .expect_err("allow must not issue challenge")
                .kind(),
            ToolPolicyErrorKind::InvalidConfirmation
        );
    }

    #[test]
    fn denial_missing_confirmation_and_unexpected_confirmation_never_execute() {
        let executor = FakeExecutor::new(Mode::Completed);
        let denied = ToolPolicy::new(
            PolicyRevision::new("revision-1").expect("revision must pass"),
            Vec::new(),
        )
        .expect("empty policy must pass")
        .evaluate(request("denied", "denied-arguments", ToolEffect::ReadOnly));
        let result = block_on_ready(execute_tool(denied, None, &executor, &NeverCancelled));
        assert_eq!(
            result.audit().terminal_outcome(),
            ToolTerminalOutcome::Denied(ToolDenialReason::AuthorizationDenied)
        );

        let required = require_confirmation("missing", "missing-arguments");
        let result = block_on_ready(execute_tool(required, None, &executor, &NeverCancelled));
        assert_eq!(
            result.audit().confirmation_state(),
            ConfirmationState::Missing
        );
        assert_eq!(
            result.audit().terminal_outcome(),
            ToolTerminalOutcome::Denied(ToolDenialReason::ConfirmationMissing)
        );

        let mut source = require_confirmation("source", "source-arguments");
        let confirmation = source
            .take_confirmation_challenge()
            .expect("challenge must pass")
            .confirm();
        let result = block_on_ready(execute_tool(
            allow("allowed"),
            Some(confirmation),
            &executor,
            &NeverCancelled,
        ));
        assert_eq!(
            result.audit().terminal_outcome(),
            ToolTerminalOutcome::Denied(ToolDenialReason::UnexpectedConfirmation)
        );
        assert_eq!(executor.calls(), 0);
    }

    #[test]
    fn mismatched_confirmation_cannot_authorize_another_request() {
        let executor = FakeExecutor::new(Mode::Completed);
        let mut source = require_confirmation("request-1", "arguments-a");
        let confirmation = source
            .take_confirmation_challenge()
            .expect("challenge must pass")
            .confirm();
        let target = require_confirmation("request-2", "arguments-b");

        let result = block_on_ready(execute_tool(
            target,
            Some(confirmation),
            &executor,
            &NeverCancelled,
        ));
        assert_eq!(
            result.audit().confirmation_state(),
            ConfirmationState::Rejected
        );
        assert_eq!(
            result.audit().terminal_outcome(),
            ToolTerminalOutcome::Denied(ToolDenialReason::ConfirmationMismatch)
        );
        assert_eq!(result.audit().attempt_count(), 0);
        assert_eq!(executor.calls(), 0);
    }

    #[test]
    fn allowed_and_exactly_confirmed_requests_execute_once_with_redacted_audit() {
        let executor = FakeExecutor::new(Mode::Completed);
        let allowed = block_on_ready(execute_tool(
            allow("allowed"),
            None,
            &executor,
            &NeverCancelled,
        ));
        assert_eq!(allowed.audit().attempt_count(), 1);
        assert_eq!(
            allowed.audit().terminal_outcome(),
            ToolTerminalOutcome::Completed
        );
        assert_eq!(
            allowed.audit().confirmation_state(),
            ConfirmationState::NotRequired
        );
        assert_eq!(
            allowed.output().expect("output must exist").expose(),
            "completed-sensitive-output"
        );

        let mut authorization = require_confirmation("confirmed", "confirmed-sensitive-arguments");
        let confirmation = authorization
            .take_confirmation_challenge()
            .expect("challenge must pass")
            .confirm();
        let confirmed = block_on_ready(execute_tool(
            authorization,
            Some(confirmation),
            &executor,
            &NeverCancelled,
        ));
        assert_eq!(confirmed.audit().attempt_count(), 1);
        assert_eq!(
            confirmed.audit().confirmation_state(),
            ConfirmationState::Confirmed
        );
        assert_eq!(
            confirmed.audit().argument_bytes(),
            "confirmed-sensitive-arguments".len()
        );
        assert_eq!(
            confirmed.audit().output_bytes(),
            "completed-sensitive-output".len()
        );
        let formatted = format!("{confirmed:?}");
        assert!(!formatted.contains("confirmed-sensitive-arguments"));
        assert!(!formatted.contains("completed-sensitive-output"));
        assert_eq!(executor.calls(), 2);
    }

    #[test]
    fn partial_failure_and_timeout_map_without_retry_or_fallback() {
        let partial_executor = FakeExecutor::new(Mode::Partial);
        let partial = block_on_ready(execute_tool(
            allow("partial"),
            None,
            &partial_executor,
            &NeverCancelled,
        ));
        assert_eq!(
            partial.audit().terminal_outcome(),
            ToolTerminalOutcome::Partial
        );
        assert_eq!(partial_executor.calls(), 1);

        let failed_executor = FakeExecutor::new(Mode::Failed);
        let failed = block_on_ready(execute_tool(
            allow("failed"),
            None,
            &failed_executor,
            &NeverCancelled,
        ));
        assert_eq!(
            failed.audit().terminal_outcome(),
            ToolTerminalOutcome::Failed
        );
        assert_eq!(
            failed.diagnostic().expect("diagnostic must exist").expose(),
            "bounded-sensitive-diagnostic"
        );
        assert_eq!(failed_executor.calls(), 1);

        let timeout_executor = FakeExecutor::new(Mode::TimedOut);
        let timed_out = block_on_ready(execute_tool(
            allow("timeout"),
            None,
            &timeout_executor,
            &NeverCancelled,
        ));
        assert_eq!(
            timed_out.audit().terminal_outcome(),
            ToolTerminalOutcome::TimedOut
        );
        assert_eq!(timeout_executor.calls(), 1);
    }

    #[test]
    fn preexisting_cancellation_prevents_executor_construction() {
        let executor = FakeExecutor::new(Mode::Completed);
        let result = block_on_ready(execute_tool(
            allow("cancelled"),
            None,
            &executor,
            &PreCancelled,
        ));
        assert_eq!(
            result.audit().terminal_outcome(),
            ToolTerminalOutcome::Cancelled
        );
        assert_eq!(result.audit().attempt_count(), 0);
        assert_eq!(executor.calls(), 0);
    }

    #[test]
    fn cancellation_wins_a_simultaneously_ready_executor_outcome() {
        let executor = FakeExecutor::new(Mode::Completed);
        let cancellation = FlipCancellation::new();
        let result = block_on_ready(execute_tool(
            allow("simultaneous"),
            None,
            &executor,
            &cancellation,
        ));
        assert_eq!(
            result.audit().terminal_outcome(),
            ToolTerminalOutcome::Cancelled
        );
        assert_eq!(result.audit().attempt_count(), 1);
        assert_eq!(executor.calls(), 1);
    }

    #[test]
    fn in_flight_cancellation_drops_pending_work_before_return() {
        let active = Arc::new(AtomicUsize::new(0));
        let executor = PendingExecutor {
            calls: AtomicUsize::new(0),
            active: Arc::clone(&active),
        };
        let cancellation = ManualCancellation::new();
        let mut future = execute_tool(allow("pending"), None, &executor, &cancellation);
        let mut context = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut context).is_pending());
        assert_eq!(executor.calls.load(Ordering::SeqCst), 1);
        assert_eq!(active.load(Ordering::SeqCst), 1);

        cancellation.cancel();
        let result = match future.as_mut().poll(&mut context) {
            Poll::Ready(result) => result,
            Poll::Pending => panic!("cancellation must complete the gate"),
        };
        assert_eq!(
            result.audit().terminal_outcome(),
            ToolTerminalOutcome::Cancelled
        );
        assert_eq!(result.audit().attempt_count(), 1);
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn repeated_operations_have_identical_safe_results_and_one_attempt_each() {
        let executor = FakeExecutor::new(Mode::Completed);
        let first = block_on_ready(execute_tool(
            allow("repeat"),
            None,
            &executor,
            &NeverCancelled,
        ));
        let second = block_on_ready(execute_tool(
            allow("repeat"),
            None,
            &executor,
            &NeverCancelled,
        ));
        assert_eq!(first.audit(), second.audit());
        assert_eq!(executor.calls(), 2);
    }
}
