//! One-use confirmation evidence bound to an exact authorization.

use std::{
    collections::BTreeSet,
    fmt::{Debug, Formatter},
};

use crate::{PolicyRevision, ToolAuthorization, ToolEffect};

struct ConfirmationBinding {
    policy_revision: PolicyRevision,
    request_id: crate::ToolRequestId,
    actor: crate::ActorId,
    tool: crate::ToolId,
    effects: BTreeSet<ToolEffect>,
    arguments: Box<str>,
}

impl ConfirmationBinding {
    fn new(authorization: &ToolAuthorization) -> Self {
        let request = authorization.request();
        Self {
            policy_revision: authorization.policy_revision().clone(),
            request_id: request.id().clone(),
            actor: request.actor().clone(),
            tool: request.tool().clone(),
            effects: request.effects().clone(),
            arguments: request.arguments().expose().into(),
        }
    }

    fn matches(&self, authorization: &ToolAuthorization) -> bool {
        let request = authorization.request();
        self.policy_revision == *authorization.policy_revision()
            && self.request_id == *request.id()
            && self.actor == *request.actor()
            && self.tool == *request.tool()
            && self.effects == *request.effects()
            && self.arguments.as_ref() == request.arguments().expose()
    }
}

impl Debug for ConfirmationBinding {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ConfirmationBinding")
            .field("policy_revision", &self.policy_revision)
            .field("request_id", &self.request_id)
            .field("actor", &self.actor)
            .field("tool", &self.tool)
            .field("effects", &self.effects)
            .field("argument_bytes", &self.arguments.len())
            .finish_non_exhaustive()
    }
}

/// One non-cloneable opportunity to confirm an exact authorization.
pub struct ToolConfirmationChallenge {
    binding: ConfirmationBinding,
}

impl ToolConfirmationChallenge {
    pub(crate) fn new(authorization: &ToolAuthorization) -> Self {
        Self {
            binding: ConfirmationBinding::new(authorization),
        }
    }

    /// Consumes the challenge and records confirmation of its exact binding.
    #[must_use]
    pub fn confirm(self) -> ToolConfirmation {
        ToolConfirmation {
            binding: self.binding,
        }
    }
}

impl Debug for ToolConfirmationChallenge {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("ToolConfirmationChallenge")
            .field(&self.binding)
            .finish()
    }
}

/// Non-cloneable accepted confirmation evidence for one exact authorization.
pub struct ToolConfirmation {
    binding: ConfirmationBinding,
}

impl ToolConfirmation {
    pub(crate) fn matches(&self, authorization: &ToolAuthorization) -> bool {
        self.binding.matches(authorization)
    }
}

impl Debug for ToolConfirmation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("ToolConfirmation")
            .field(&self.binding)
            .finish()
    }
}
