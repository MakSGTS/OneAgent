//! Deterministic fail-closed Tool Execution Policy evaluation.

use std::fmt::{Debug, Formatter};

use crate::confirmation::ToolConfirmationChallenge;
use crate::{
    ActorId, PolicyRevision, ToolEffect, ToolId, ToolPolicyError, ToolPolicyErrorKind, ToolRequest,
};

/// Maximum number of input rules accepted by one policy.
pub const MAX_TOOL_POLICY_RULES: usize = 4_096;

/// Actor matching scope for one policy rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActorScope {
    /// Matches every accepted actor label.
    Any,
    /// Matches only the exact accepted actor label.
    Exact(ActorId),
}

impl ActorScope {
    fn matches(&self, actor: &ActorId) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => expected == actor,
        }
    }
}

/// Tool matching scope for one policy rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolScope {
    /// Matches every accepted tool identity.
    Any,
    /// Matches only the exact accepted tool identity.
    Exact(ToolId),
}

impl ToolScope {
    fn matches(&self, tool: &ToolId) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => expected == tool,
        }
    }
}

/// Closed action applied by one matching policy rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleAction {
    /// Rejects execution.
    Deny,
    /// Requires an exact confirmation before execution.
    RequireConfirmation,
    /// Allows execution without confirmation when no stricter rule matches.
    Allow,
}

/// One immutable actor/tool/effect policy rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolRule {
    actor: ActorScope,
    tool: ToolScope,
    effect: ToolEffect,
    action: RuleAction,
}

impl ToolRule {
    /// Creates one exact source-independent policy rule.
    #[must_use]
    pub const fn new(
        actor: ActorScope,
        tool: ToolScope,
        effect: ToolEffect,
        action: RuleAction,
    ) -> Self {
        Self {
            actor,
            tool,
            effect,
            action,
        }
    }

    /// Returns the actor scope.
    #[must_use]
    pub const fn actor(&self) -> &ActorScope {
        &self.actor
    }

    /// Returns the tool scope.
    #[must_use]
    pub const fn tool(&self) -> &ToolScope {
        &self.tool
    }

    /// Returns the exact matched effect.
    #[must_use]
    pub const fn effect(&self) -> ToolEffect {
        self.effect
    }

    /// Returns the rule action.
    #[must_use]
    pub const fn action(&self) -> RuleAction {
        self.action
    }

    fn matches(&self, request: &ToolRequest, effect: ToolEffect) -> bool {
        self.effect == effect
            && self.actor.matches(request.actor())
            && self.tool.matches(request.tool())
    }
}

/// Closed request-wide authorization outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthorizationDecisionKind {
    /// Execution is denied.
    Deny,
    /// Execution requires exact confirmation.
    RequireConfirmation,
    /// Execution may proceed without confirmation.
    Allow,
}

/// Stable reason for one authorization outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthorizationDecisionReason {
    /// At least one matching rule explicitly denied a declared effect.
    ExplicitDeny,
    /// At least one declared effect had no matching rule.
    NoMatchingRule,
    /// At least one matching rule required confirmation and no deny won.
    ConfirmationRequired,
    /// Every declared effect had an allow and no stricter rule won.
    Allowed,
}

/// One canonical immutable policy revision and rule set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPolicy {
    revision: PolicyRevision,
    rules: Vec<ToolRule>,
}

impl ToolPolicy {
    /// Creates a canonical policy and deduplicates identical rules.
    ///
    /// Empty policies are valid and deny every request by default.
    ///
    /// # Errors
    ///
    /// Returns [`ToolPolicyErrorKind::InvalidPolicy`] when the input rule count
    /// exceeds [`MAX_TOOL_POLICY_RULES`].
    pub fn new(
        revision: PolicyRevision,
        mut rules: Vec<ToolRule>,
    ) -> Result<Self, ToolPolicyError> {
        if rules.len() > MAX_TOOL_POLICY_RULES {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidPolicy,
                "tool policy exceeds rule count limit",
            ));
        }
        rules.sort();
        rules.dedup();
        Ok(Self { revision, rules })
    }

    /// Returns the exact accepted policy revision.
    #[must_use]
    pub const fn revision(&self) -> &PolicyRevision {
        &self.revision
    }

    /// Returns canonical rules.
    #[must_use]
    pub fn rules(&self) -> &[ToolRule] {
        &self.rules
    }

    /// Consumes a validated request and produces one request-owning decision.
    #[must_use]
    pub fn evaluate(&self, request: ToolRequest) -> ToolAuthorization {
        let (kind, reason) = self.decision(&request);
        ToolAuthorization {
            request,
            policy_revision: self.revision.clone(),
            kind,
            reason,
            confirmation_challenge_issued: false,
        }
    }

    fn decision(
        &self,
        request: &ToolRequest,
    ) -> (AuthorizationDecisionKind, AuthorizationDecisionReason) {
        if request.effects().iter().copied().any(|effect| {
            self.rules
                .iter()
                .any(|rule| rule.matches(request, effect) && rule.action == RuleAction::Deny)
        }) {
            return (
                AuthorizationDecisionKind::Deny,
                AuthorizationDecisionReason::ExplicitDeny,
            );
        }

        if request
            .effects()
            .iter()
            .copied()
            .any(|effect| !self.rules.iter().any(|rule| rule.matches(request, effect)))
        {
            return (
                AuthorizationDecisionKind::Deny,
                AuthorizationDecisionReason::NoMatchingRule,
            );
        }

        if request.effects().iter().copied().any(|effect| {
            self.rules.iter().any(|rule| {
                rule.matches(request, effect) && rule.action == RuleAction::RequireConfirmation
            })
        }) {
            return (
                AuthorizationDecisionKind::RequireConfirmation,
                AuthorizationDecisionReason::ConfirmationRequired,
            );
        }

        (
            AuthorizationDecisionKind::Allow,
            AuthorizationDecisionReason::Allowed,
        )
    }
}

/// One non-cloneable policy decision that owns its exact evaluated request.
pub struct ToolAuthorization {
    request: ToolRequest,
    policy_revision: PolicyRevision,
    kind: AuthorizationDecisionKind,
    reason: AuthorizationDecisionReason,
    confirmation_challenge_issued: bool,
}

impl ToolAuthorization {
    /// Returns the exact evaluated request.
    #[must_use]
    pub const fn request(&self) -> &ToolRequest {
        &self.request
    }

    /// Returns the exact policy revision used for evaluation.
    #[must_use]
    pub const fn policy_revision(&self) -> &PolicyRevision {
        &self.policy_revision
    }

    /// Returns the request-wide decision kind.
    #[must_use]
    pub const fn kind(&self) -> AuthorizationDecisionKind {
        self.kind
    }

    /// Returns the stable decision reason.
    #[must_use]
    pub const fn reason(&self) -> AuthorizationDecisionReason {
        self.reason
    }

    /// Issues the only confirmation challenge for this decision.
    ///
    /// # Errors
    ///
    /// Returns [`ToolPolicyErrorKind::InvalidConfirmation`] when this decision
    /// does not require confirmation or already issued its challenge. Decision
    /// kind is checked first.
    pub fn take_confirmation_challenge(
        &mut self,
    ) -> Result<ToolConfirmationChallenge, ToolPolicyError> {
        if self.kind != AuthorizationDecisionKind::RequireConfirmation {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidConfirmation,
                "authorization does not require confirmation",
            ));
        }
        if self.confirmation_challenge_issued {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidConfirmation,
                "confirmation challenge was already issued",
            ));
        }
        self.confirmation_challenge_issued = true;
        Ok(ToolConfirmationChallenge::new(self))
    }
}

impl Debug for ToolAuthorization {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolAuthorization")
            .field("request", &self.request)
            .field("policy_revision", &self.policy_revision)
            .field("kind", &self.kind)
            .field("reason", &self.reason)
            .field(
                "confirmation_challenge_issued",
                &self.confirmation_challenge_issued,
            )
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActorScope, AuthorizationDecisionKind, AuthorizationDecisionReason, MAX_TOOL_POLICY_RULES,
        RuleAction, ToolPolicy, ToolRule, ToolScope,
    };
    use crate::{
        ActorId, PolicyRevision, ToolArguments, ToolEffect, ToolId, ToolPolicyErrorKind,
        ToolRequest, ToolRequestId,
    };

    fn actor(value: &str) -> ActorId {
        ActorId::new(value).expect("actor ID must pass")
    }

    fn tool(value: &str) -> ToolId {
        ToolId::new(value).expect("tool ID must pass")
    }

    fn rule(
        actor: ActorScope,
        tool: ToolScope,
        effect: ToolEffect,
        action: RuleAction,
    ) -> ToolRule {
        ToolRule::new(actor, tool, effect, action)
    }

    fn policy(rules: Vec<ToolRule>) -> ToolPolicy {
        ToolPolicy::new(
            PolicyRevision::new("revision-1").expect("revision must pass"),
            rules,
        )
        .expect("policy must pass")
    }

    fn request(
        request_id: &str,
        actor_id: &str,
        tool_id: &str,
        arguments: &str,
        effects: impl IntoIterator<Item = ToolEffect>,
    ) -> ToolRequest {
        ToolRequest::new(
            ToolRequestId::new(request_id).expect("request ID must pass"),
            actor(actor_id),
            tool(tool_id),
            ToolArguments::new(arguments).expect("arguments must pass"),
            effects,
        )
        .expect("request must pass")
    }

    #[test]
    fn policy_canonicalizes_order_and_identical_duplicates() {
        let allow = rule(
            ActorScope::Any,
            ToolScope::Any,
            ToolEffect::ReadOnly,
            RuleAction::Allow,
        );
        let confirm = rule(
            ActorScope::Exact(actor("actor")),
            ToolScope::Exact(tool("tool")),
            ToolEffect::ExternalMutation,
            RuleAction::RequireConfirmation,
        );

        let first = policy(vec![confirm.clone(), allow.clone(), confirm.clone()]);
        let second = policy(vec![allow, confirm]);
        assert_eq!(first, second);
        assert_eq!(first.rules().len(), 2);
    }

    #[test]
    fn policy_rule_count_bound_precedes_canonicalization() {
        let duplicate = rule(
            ActorScope::Any,
            ToolScope::Any,
            ToolEffect::ReadOnly,
            RuleAction::Allow,
        );
        assert_eq!(
            policy(vec![duplicate.clone(); MAX_TOOL_POLICY_RULES])
                .rules()
                .len(),
            1
        );

        let error = ToolPolicy::new(
            PolicyRevision::new("revision").expect("revision must pass"),
            vec![duplicate; MAX_TOOL_POLICY_RULES + 1],
        )
        .expect_err("input over the maximum must fail before deduplication");
        assert_eq!(error.kind(), ToolPolicyErrorKind::InvalidPolicy);
    }

    #[test]
    fn empty_and_unmatched_policy_default_to_deny() {
        let empty = policy(Vec::new()).evaluate(request(
            "request-empty",
            "actor",
            "tool",
            "sensitive",
            [ToolEffect::ReadOnly],
        ));
        assert_eq!(empty.kind(), AuthorizationDecisionKind::Deny);
        assert_eq!(empty.reason(), AuthorizationDecisionReason::NoMatchingRule);

        let unmatched = policy(vec![rule(
            ActorScope::Exact(actor("other")),
            ToolScope::Any,
            ToolEffect::ReadOnly,
            RuleAction::Allow,
        )])
        .evaluate(request(
            "request-unmatched",
            "actor",
            "tool",
            "sensitive",
            [ToolEffect::ReadOnly],
        ));
        assert_eq!(unmatched.kind(), AuthorizationDecisionKind::Deny);
        assert_eq!(
            unmatched.reason(),
            AuthorizationDecisionReason::NoMatchingRule
        );
    }

    #[test]
    fn matching_deny_globally_overrides_specific_allow_and_input_order() {
        let deny = rule(
            ActorScope::Any,
            ToolScope::Any,
            ToolEffect::ExternalMutation,
            RuleAction::Deny,
        );
        let allow = rule(
            ActorScope::Exact(actor("actor")),
            ToolScope::Exact(tool("tool")),
            ToolEffect::ExternalMutation,
            RuleAction::Allow,
        );
        for rules in [
            vec![deny.clone(), allow.clone()],
            vec![allow.clone(), deny.clone()],
        ] {
            let decision = policy(rules).evaluate(request(
                "request-deny",
                "actor",
                "tool",
                "sensitive",
                [ToolEffect::ExternalMutation],
            ));
            assert_eq!(decision.kind(), AuthorizationDecisionKind::Deny);
            assert_eq!(decision.reason(), AuthorizationDecisionReason::ExplicitDeny);
        }
    }

    #[test]
    fn confirmation_overrides_allow_and_every_effect_must_be_covered() {
        let decision = policy(vec![
            rule(
                ActorScope::Any,
                ToolScope::Any,
                ToolEffect::LocalMutation,
                RuleAction::Allow,
            ),
            rule(
                ActorScope::Any,
                ToolScope::Any,
                ToolEffect::LocalMutation,
                RuleAction::RequireConfirmation,
            ),
        ])
        .evaluate(request(
            "request-confirm",
            "actor",
            "tool",
            "sensitive",
            [ToolEffect::LocalMutation],
        ));
        assert_eq!(
            decision.kind(),
            AuthorizationDecisionKind::RequireConfirmation
        );
        assert_eq!(
            decision.reason(),
            AuthorizationDecisionReason::ConfirmationRequired
        );

        let partial = policy(vec![rule(
            ActorScope::Any,
            ToolScope::Any,
            ToolEffect::LocalMutation,
            RuleAction::Allow,
        )])
        .evaluate(request(
            "request-partial",
            "actor",
            "tool",
            "sensitive",
            [ToolEffect::LocalMutation, ToolEffect::Privileged],
        ));
        assert_eq!(partial.kind(), AuthorizationDecisionKind::Deny);
        assert_eq!(
            partial.reason(),
            AuthorizationDecisionReason::NoMatchingRule
        );
    }

    #[test]
    fn exact_and_wildcard_scopes_allow_only_complete_matches() {
        let policy = policy(vec![
            rule(
                ActorScope::Exact(actor("actor")),
                ToolScope::Any,
                ToolEffect::LocalMutation,
                RuleAction::Allow,
            ),
            rule(
                ActorScope::Any,
                ToolScope::Exact(tool("tool")),
                ToolEffect::Privileged,
                RuleAction::Allow,
            ),
        ]);
        let allowed = policy.evaluate(request(
            "request-allowed",
            "actor",
            "tool",
            "sensitive",
            [ToolEffect::Privileged, ToolEffect::LocalMutation],
        ));
        assert_eq!(allowed.kind(), AuthorizationDecisionKind::Allow);
        assert_eq!(allowed.reason(), AuthorizationDecisionReason::Allowed);
    }

    #[test]
    fn authorization_owns_exact_request_revision_and_redacts_arguments() {
        let sentinel = "synthetic-policy-sensitive-arguments";
        let policy = ToolPolicy::new(
            PolicyRevision::new("revision-sensitive").expect("revision must pass"),
            vec![rule(
                ActorScope::Any,
                ToolScope::Any,
                ToolEffect::ReadOnly,
                RuleAction::Allow,
            )],
        )
        .expect("policy must pass");

        let build = |id| {
            policy.evaluate(request(
                id,
                "actor",
                "tool",
                sentinel,
                [ToolEffect::ReadOnly],
            ))
        };
        let first = build("request-repeat");
        let second = build("request-repeat");

        assert_eq!(first.policy_revision().as_str(), "revision-sensitive");
        assert_eq!(first.request().arguments().expose(), sentinel);
        assert_eq!(first.kind(), AuthorizationDecisionKind::Allow);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
        assert!(!format!("{first:?}").contains(sentinel));
    }
}
