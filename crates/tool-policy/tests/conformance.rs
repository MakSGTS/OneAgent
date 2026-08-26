use std::{
    future::{Future, pending},
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
};

use oneagent_tool_policy::{
    ActorId, ActorScope, AuthorizationDecisionKind, AuthorizationDecisionReason, ConfirmationState,
    MAX_TOOL_ARGUMENT_BYTES, MAX_TOOL_DIAGNOSTIC_BYTES, MAX_TOOL_OUTPUT_BYTES, NeverCancelled,
    PolicyRevision, RuleAction, ToolArguments, ToolCancellationSignal, ToolDenialReason,
    ToolDiagnostic, ToolEffect, ToolExecutor, ToolExecutorOutcome, ToolFuture, ToolId, ToolOutput,
    ToolPolicy, ToolPolicyErrorKind, ToolRequest, ToolRequestId, ToolRule, ToolScope,
    ToolTerminalOutcome, execute_tool,
};

#[derive(Clone, Copy)]
enum FakeMode {
    Completed,
    Partial,
    Failed,
    TimedOut,
    Pending,
}

struct FakeExecutor {
    calls: AtomicUsize,
    active: Arc<AtomicUsize>,
    mode: FakeMode,
}

impl FakeExecutor {
    fn new(mode: FakeMode) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            active: Arc::new(AtomicUsize::new(0)),
            mode,
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn active(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }
}

struct ActiveGuard(Arc<AtomicUsize>);

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl ToolExecutor for FakeExecutor {
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
            match self.mode {
                FakeMode::Completed => ToolExecutorOutcome::Completed(
                    ToolOutput::new("synthetic-sensitive-output").expect("output must pass"),
                ),
                FakeMode::Partial => ToolExecutorOutcome::Partial(
                    ToolOutput::new("synthetic-partial-output").expect("output must pass"),
                ),
                FakeMode::Failed => ToolExecutorOutcome::Failed(Some(
                    ToolDiagnostic::new("synthetic-sensitive-diagnostic")
                        .expect("diagnostic must pass"),
                )),
                FakeMode::TimedOut => ToolExecutorOutcome::TimedOut,
                FakeMode::Pending => pending().await,
            }
        })
    }
}

struct ManualCancellation {
    cancelled: AtomicBool,
}

impl ManualCancellation {
    const fn new(cancelled: bool) -> Self {
        Self {
            cancelled: AtomicBool::new(cancelled),
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

fn request(
    id: &str,
    actor: &str,
    tool: &str,
    arguments: &str,
    effects: impl IntoIterator<Item = ToolEffect>,
) -> ToolRequest {
    ToolRequest::new(
        ToolRequestId::new(id).expect("request ID must pass"),
        ActorId::new(actor).expect("actor must pass"),
        ToolId::new(tool).expect("tool must pass"),
        ToolArguments::new(arguments).expect("arguments must pass"),
        effects,
    )
    .expect("request must pass")
}

fn policy(revision: &str, effect: ToolEffect, action: RuleAction) -> ToolPolicy {
    ToolPolicy::new(
        PolicyRevision::new(revision).expect("revision must pass"),
        vec![ToolRule::new(
            ActorScope::Any,
            ToolScope::Any,
            effect,
            action,
        )],
    )
    .expect("policy must pass")
}

fn allow(id: &str) -> oneagent_tool_policy::ToolAuthorization {
    policy("revision-1", ToolEffect::ReadOnly, RuleAction::Allow).evaluate(request(
        id,
        "actor-1",
        "tool-1",
        "synthetic-sensitive-arguments",
        [ToolEffect::ReadOnly],
    ))
}

fn confirm_required(
    revision: &str,
    id: &str,
    actor: &str,
    tool: &str,
    arguments: &str,
) -> oneagent_tool_policy::ToolAuthorization {
    policy(
        revision,
        ToolEffect::ExternalMutation,
        RuleAction::RequireConfirmation,
    )
    .evaluate(request(
        id,
        actor,
        tool,
        arguments,
        [ToolEffect::ExternalMutation],
    ))
}

fn block_on_ready<T>(mut future: Pin<Box<dyn Future<Output = T> + '_>>) -> T {
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("future must be ready"),
    }
}

#[test]
fn public_construction_effects_and_policy_are_bounded_canonical_and_fail_closed() {
    let unicode = "я".repeat(MAX_TOOL_ARGUMENT_BYTES / "я".len());
    assert_eq!(unicode.len(), MAX_TOOL_ARGUMENT_BYTES);
    assert!(ToolArguments::new(unicode).is_ok());
    assert_eq!(
        ToolArguments::new("x".repeat(MAX_TOOL_ARGUMENT_BYTES + 1))
            .expect_err("over-limit arguments must fail")
            .kind(),
        ToolPolicyErrorKind::InvalidArguments
    );
    assert_eq!(
        ToolRequest::new(
            ToolRequestId::new("request").expect("request ID must pass"),
            ActorId::new("actor").expect("actor must pass"),
            ToolId::new("tool").expect("tool must pass"),
            ToolArguments::new("").expect("arguments must pass"),
            [ToolEffect::ReadOnly, ToolEffect::LocalMutation],
        )
        .expect_err("contradictory effects must fail")
        .kind(),
        ToolPolicyErrorKind::InvalidEffectSet
    );

    let allow = ToolRule::new(
        ActorScope::Any,
        ToolScope::Any,
        ToolEffect::LocalMutation,
        RuleAction::Allow,
    );
    let deny = ToolRule::new(
        ActorScope::Any,
        ToolScope::Any,
        ToolEffect::LocalMutation,
        RuleAction::Deny,
    );
    let policy = ToolPolicy::new(
        PolicyRevision::new("revision").expect("revision must pass"),
        vec![allow.clone(), deny, allow],
    )
    .expect("policy must pass");
    assert_eq!(policy.rules().len(), 2);
    let authorization = policy.evaluate(request(
        "request",
        "actor",
        "tool",
        "sensitive-policy-arguments",
        [ToolEffect::LocalMutation],
    ));
    assert_eq!(authorization.kind(), AuthorizationDecisionKind::Deny);
    assert_eq!(
        authorization.reason(),
        AuthorizationDecisionReason::ExplicitDeny
    );
    assert!(!format!("{authorization:?}").contains("sensitive-policy-arguments"));

    let default_denial = ToolPolicy::new(
        PolicyRevision::new("revision").expect("revision must pass"),
        Vec::new(),
    )
    .expect("empty policy must pass")
    .evaluate(request(
        "default-deny",
        "actor",
        "tool",
        "arguments",
        [ToolEffect::ReadOnly],
    ));
    assert_eq!(default_denial.kind(), AuthorizationDecisionKind::Deny);
    assert_eq!(
        default_denial.reason(),
        AuthorizationDecisionReason::NoMatchingRule
    );
}

#[test]
fn public_missing_and_exact_confirmation_have_zero_or_one_attempt() {
    let executor = FakeExecutor::new(FakeMode::Completed);
    let missing = confirm_required("revision-1", "missing", "actor-1", "tool-1", "arguments-1");
    let result = block_on_ready(execute_tool(missing, None, &executor, &NeverCancelled));
    assert_eq!(
        result.audit().confirmation_state(),
        ConfirmationState::Missing
    );
    assert_eq!(
        result.audit().terminal_outcome(),
        ToolTerminalOutcome::Denied(ToolDenialReason::ConfirmationMissing)
    );
    assert_eq!(executor.calls(), 0);

    let mut exact = confirm_required(
        "revision-1",
        "exact",
        "actor-1",
        "tool-1",
        "exact-arguments",
    );
    let confirmation = exact
        .take_confirmation_challenge()
        .expect("challenge must pass")
        .confirm();
    let result = block_on_ready(execute_tool(
        exact,
        Some(confirmation),
        &executor,
        &NeverCancelled,
    ));
    assert_eq!(
        result.audit().confirmation_state(),
        ConfirmationState::Confirmed
    );
    assert_eq!(
        result.audit().terminal_outcome(),
        ToolTerminalOutcome::Completed
    );
    assert_eq!(result.audit().attempt_count(), 1);
    assert_eq!(executor.calls(), 1);
    assert_eq!(executor.active(), 0);
}

#[test]
fn public_confirmation_is_one_use_exactly_bound_and_rejects_replay_before_execution() {
    let executor = FakeExecutor::new(FakeMode::Completed);

    let mut source = confirm_required(
        "revision-1",
        "request-1",
        "actor-1",
        "tool-1",
        "arguments-1",
    );
    let confirmation = source
        .take_confirmation_challenge()
        .expect("first challenge must pass")
        .confirm();
    assert_eq!(
        source
            .take_confirmation_challenge()
            .expect_err("second challenge must fail")
            .kind(),
        ToolPolicyErrorKind::InvalidConfirmation
    );

    let stale = confirm_required(
        "revision-2",
        "request-1",
        "actor-1",
        "tool-1",
        "arguments-1",
    );
    let result = block_on_ready(execute_tool(
        stale,
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

    let mut source = confirm_required(
        "revision-1",
        "request-1",
        "actor-1",
        "tool-1",
        "arguments-1",
    );
    let confirmation = source
        .take_confirmation_challenge()
        .expect("challenge must pass")
        .confirm();
    let other_request = confirm_required(
        "revision-1",
        "request-2",
        "actor-2",
        "tool-2",
        "arguments-2",
    );
    let result = block_on_ready(execute_tool(
        other_request,
        Some(confirmation),
        &executor,
        &NeverCancelled,
    ));
    assert_eq!(
        result.audit().terminal_outcome(),
        ToolTerminalOutcome::Denied(ToolDenialReason::ConfirmationMismatch)
    );
    assert_eq!(executor.calls(), 0);
}

#[test]
fn public_denial_and_allow_paths_have_zero_or_one_attempt_and_complete_audit() {
    let executor = FakeExecutor::new(FakeMode::Completed);
    let denied = ToolPolicy::new(
        PolicyRevision::new("revision-1").expect("revision must pass"),
        Vec::new(),
    )
    .expect("empty policy must pass")
    .evaluate(request(
        "denied",
        "actor-1",
        "tool-1",
        "denied-sensitive-arguments",
        [ToolEffect::ReadOnly],
    ));
    let denied = block_on_ready(execute_tool(denied, None, &executor, &NeverCancelled));
    assert_eq!(denied.audit().attempt_count(), 0);
    assert_eq!(
        denied.audit().confirmation_state(),
        ConfirmationState::NotApplicable
    );
    assert_eq!(executor.calls(), 0);

    let allowed = block_on_ready(execute_tool(
        allow("allowed"),
        None,
        &executor,
        &NeverCancelled,
    ));
    let audit = allowed.audit();
    assert_eq!(audit.request_id().as_str(), "allowed");
    assert_eq!(audit.actor().as_str(), "actor-1");
    assert_eq!(audit.tool().as_str(), "tool-1");
    assert_eq!(audit.policy_revision().as_str(), "revision-1");
    assert_eq!(
        audit.effects().iter().copied().collect::<Vec<_>>(),
        vec![ToolEffect::ReadOnly]
    );
    assert_eq!(
        audit.argument_bytes(),
        "synthetic-sensitive-arguments".len()
    );
    assert_eq!(audit.authorization_kind(), AuthorizationDecisionKind::Allow);
    assert_eq!(
        audit.authorization_reason(),
        AuthorizationDecisionReason::Allowed
    );
    assert_eq!(audit.confirmation_state(), ConfirmationState::NotRequired);
    assert_eq!(audit.attempt_count(), 1);
    assert_eq!(audit.terminal_outcome(), ToolTerminalOutcome::Completed);
    assert_eq!(audit.output_bytes(), "synthetic-sensitive-output".len());
    assert_eq!(executor.calls(), 1);
    assert_eq!(executor.active(), 0);

    let formatted = format!("{allowed:?}");
    assert!(!formatted.contains("synthetic-sensitive-arguments"));
    assert!(!formatted.contains("synthetic-sensitive-output"));
    let audit_debug = format!("{audit:?}");
    let ordered_fields = [
        "request_id",
        "actor",
        "tool",
        "policy_revision",
        "effects",
        "argument_bytes",
        "authorization_kind",
        "authorization_reason",
        "confirmation_state",
        "attempt_count",
        "terminal_outcome",
        "output_bytes",
    ];
    let mut previous = 0;
    for field in ordered_fields {
        let position = audit_debug
            .find(field)
            .expect("audit field must be present");
        assert!(position >= previous, "audit fields must have stable order");
        previous = position;
    }
}

#[test]
fn public_terminal_outcome_matrix_is_bounded_redacted_and_has_no_retry() {
    let cases = [
        (FakeMode::Partial, ToolTerminalOutcome::Partial, true, false),
        (FakeMode::Failed, ToolTerminalOutcome::Failed, false, true),
        (
            FakeMode::TimedOut,
            ToolTerminalOutcome::TimedOut,
            false,
            false,
        ),
    ];
    for (mode, expected, has_output, has_diagnostic) in cases {
        let executor = FakeExecutor::new(mode);
        let result = block_on_ready(execute_tool(
            allow("terminal"),
            None,
            &executor,
            &NeverCancelled,
        ));
        assert_eq!(result.audit().terminal_outcome(), expected);
        assert_eq!(result.audit().attempt_count(), 1);
        assert_eq!(result.output().is_some(), has_output);
        assert_eq!(result.diagnostic().is_some(), has_diagnostic);
        assert_eq!(executor.calls(), 1);
        assert_eq!(executor.active(), 0);
        let debug = format!("{result:?}");
        assert!(!debug.contains("synthetic-partial-output"));
        assert!(!debug.contains("synthetic-sensitive-diagnostic"));
    }

    assert!(ToolOutput::new("x".repeat(MAX_TOOL_OUTPUT_BYTES)).is_ok());
    assert_eq!(
        ToolOutput::new("x".repeat(MAX_TOOL_OUTPUT_BYTES + 1))
            .expect_err("over-limit output must fail")
            .kind(),
        ToolPolicyErrorKind::InvalidOutput
    );
    assert!(ToolDiagnostic::new("d".repeat(MAX_TOOL_DIAGNOSTIC_BYTES)).is_ok());
    assert_eq!(
        ToolDiagnostic::new("d".repeat(MAX_TOOL_DIAGNOSTIC_BYTES + 1))
            .expect_err("over-limit diagnostic must fail")
            .kind(),
        ToolPolicyErrorKind::InvalidDiagnostic
    );
}

#[test]
fn public_cancellation_precedence_drops_all_active_work_before_return() {
    let pre_cancelled = ManualCancellation::new(true);
    let executor = FakeExecutor::new(FakeMode::Completed);
    let result = block_on_ready(execute_tool(
        allow("pre-cancelled"),
        None,
        &executor,
        &pre_cancelled,
    ));
    assert_eq!(
        result.audit().terminal_outcome(),
        ToolTerminalOutcome::Cancelled
    );
    assert_eq!(result.audit().attempt_count(), 0);
    assert_eq!(executor.calls(), 0);
    assert_eq!(executor.active(), 0);

    let cancellation = ManualCancellation::new(false);
    let executor = FakeExecutor::new(FakeMode::Pending);
    let mut future = execute_tool(allow("in-flight"), None, &executor, &cancellation);
    let mut context = Context::from_waker(Waker::noop());
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert_eq!(executor.calls(), 1);
    assert_eq!(executor.active(), 1);
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
    assert_eq!(executor.calls(), 1);
    assert_eq!(executor.active(), 0);
}

#[test]
fn public_repeated_operations_are_fresh_equal_and_resource_free() {
    let executor = FakeExecutor::new(FakeMode::Completed);
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
    assert_eq!(
        first.output().expect("first output must exist").expose(),
        second.output().expect("second output must exist").expose()
    );
    assert_eq!(executor.calls(), 2);
    assert_eq!(executor.active(), 0);
}
