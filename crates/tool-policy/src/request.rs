//! Validated source-independent tool requests and side effects.

use std::{
    collections::BTreeSet,
    fmt::{Debug, Formatter},
};

use crate::{ActorId, ToolId, ToolPolicyError, ToolPolicyErrorKind, ToolRequestId};

/// Maximum tool argument size in UTF-8 bytes.
pub const MAX_TOOL_ARGUMENT_BYTES: usize = 65_536;

/// Opaque bounded tool arguments with explicit content access.
pub struct ToolArguments(Box<str>);

impl ToolArguments {
    /// Creates bounded arguments and preserves every accepted UTF-8 byte.
    ///
    /// Empty arguments are valid for a zero-argument tool.
    ///
    /// # Errors
    ///
    /// Returns [`ToolPolicyErrorKind::InvalidArguments`] when the byte maximum
    /// is exceeded without retaining or reporting the rejected value.
    pub fn new(value: impl Into<String>) -> Result<Self, ToolPolicyError> {
        let value = value.into();
        if value.len() > MAX_TOOL_ARGUMENT_BYTES {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidArguments,
                "tool arguments exceed byte limit",
            ));
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the exact accepted arguments through an explicit access point.
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

impl Debug for ToolArguments {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolArguments")
            .field("bytes", &self.byte_len())
            .finish_non_exhaustive()
    }
}

/// Closed conservative side-effect vocabulary for one tool request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolEffect {
    /// Observes state without intentionally changing it.
    ReadOnly,
    /// Changes local process or workspace state.
    LocalMutation,
    /// Changes a remote or third-party-visible system.
    ExternalMutation,
    /// Can delete, overwrite, irreversibly replace, or hinder recovery.
    Destructive,
    /// Requires elevated or specially trusted authority.
    Privileged,
    /// Can disclose sensitive content to another boundary.
    SensitiveDataExposure,
}

/// One immutable validated source-independent tool request.
pub struct ToolRequest {
    id: ToolRequestId,
    actor: ActorId,
    tool: ToolId,
    arguments: ToolArguments,
    effects: BTreeSet<ToolEffect>,
}

impl ToolRequest {
    /// Creates a request and canonicalizes its side-effect set.
    ///
    /// # Errors
    ///
    /// Returns [`ToolPolicyErrorKind::InvalidEffectSet`] when effects are empty
    /// or `ReadOnly` is combined with another effect.
    pub fn new(
        id: ToolRequestId,
        actor: ActorId,
        tool: ToolId,
        arguments: ToolArguments,
        effects: impl IntoIterator<Item = ToolEffect>,
    ) -> Result<Self, ToolPolicyError> {
        let effects: BTreeSet<_> = effects.into_iter().collect();
        if effects.is_empty() {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidEffectSet,
                "tool effect set is empty",
            ));
        }
        if effects.len() > 1 && effects.contains(&ToolEffect::ReadOnly) {
            return Err(ToolPolicyError::new(
                ToolPolicyErrorKind::InvalidEffectSet,
                "read-only effect is combined with another effect",
            ));
        }

        Ok(Self {
            id,
            actor,
            tool,
            arguments,
            effects,
        })
    }

    /// Returns the caller-supplied request identity.
    #[must_use]
    pub const fn id(&self) -> &ToolRequestId {
        &self.id
    }

    /// Returns the actor label presented to policy evaluation.
    #[must_use]
    pub const fn actor(&self) -> &ActorId {
        &self.actor
    }

    /// Returns the requested tool identity.
    #[must_use]
    pub const fn tool(&self) -> &ToolId {
        &self.tool
    }

    /// Returns the exact accepted arguments through an explicit access point.
    #[must_use]
    pub const fn arguments(&self) -> &ToolArguments {
        &self.arguments
    }

    /// Returns side effects in stable enum order.
    #[must_use]
    pub const fn effects(&self) -> &BTreeSet<ToolEffect> {
        &self.effects
    }
}

impl Debug for ToolRequest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolRequest")
            .field("id", &self.id)
            .field("actor", &self.actor)
            .field("tool", &self.tool)
            .field("argument_bytes", &self.arguments.byte_len())
            .field("effects", &self.effects)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_TOOL_ARGUMENT_BYTES, ToolArguments, ToolEffect, ToolRequest};
    use crate::{ActorId, ToolId, ToolPolicyErrorKind, ToolRequestId};

    fn request(
        arguments: ToolArguments,
        effects: impl IntoIterator<Item = ToolEffect>,
    ) -> Result<ToolRequest, crate::ToolPolicyError> {
        ToolRequest::new(
            ToolRequestId::new("request-1").expect("request ID must pass"),
            ActorId::new("actor-1").expect("actor ID must pass"),
            ToolId::new("tool-1").expect("tool ID must pass"),
            arguments,
            effects,
        )
    }

    #[test]
    fn argument_bounds_count_utf8_bytes_and_preserve_empty_and_unicode() {
        assert_eq!(
            ToolArguments::new("").expect("empty must pass").expose(),
            ""
        );

        let unicode = "я".repeat(MAX_TOOL_ARGUMENT_BYTES / "я".len());
        assert_eq!(unicode.len(), MAX_TOOL_ARGUMENT_BYTES);
        assert_eq!(
            ToolArguments::new(unicode.clone())
                .expect("exact Unicode bound must pass")
                .expose(),
            unicode
        );

        let error = ToolArguments::new("x".repeat(MAX_TOOL_ARGUMENT_BYTES + 1))
            .expect_err("over-limit arguments must fail");
        assert_eq!(error.kind(), ToolPolicyErrorKind::InvalidArguments);
    }

    #[test]
    fn request_canonicalizes_duplicate_and_reordered_effects() {
        let request = request(
            ToolArguments::new("{}").expect("arguments must pass"),
            [
                ToolEffect::Privileged,
                ToolEffect::LocalMutation,
                ToolEffect::Privileged,
            ],
        )
        .expect("mixed effects must pass");

        assert_eq!(
            request.effects().iter().copied().collect::<Vec<_>>(),
            vec![ToolEffect::LocalMutation, ToolEffect::Privileged]
        );
    }

    #[test]
    fn empty_and_contradictory_effect_sets_fail_atomically() {
        let empty = request(ToolArguments::new("").expect("arguments must pass"), [])
            .expect_err("empty effects must fail");
        assert_eq!(empty.kind(), ToolPolicyErrorKind::InvalidEffectSet);
        assert_eq!(empty.diagnostic(), "tool effect set is empty");

        let contradictory = request(
            ToolArguments::new("").expect("arguments must pass"),
            [ToolEffect::LocalMutation, ToolEffect::ReadOnly],
        )
        .expect_err("read-only combination must fail");
        assert_eq!(contradictory.kind(), ToolPolicyErrorKind::InvalidEffectSet);
        assert_eq!(
            contradictory.diagnostic(),
            "read-only effect is combined with another effect"
        );
    }

    #[test]
    fn request_debug_omits_sensitive_arguments_and_accessors_preserve_values() {
        let sentinel = "synthetic-sensitive-arguments-Привет\r\n";
        let request = request(
            ToolArguments::new(sentinel).expect("arguments must pass"),
            [ToolEffect::ReadOnly, ToolEffect::ReadOnly],
        )
        .expect("request must pass");

        assert_eq!(request.id().as_str(), "request-1");
        assert_eq!(request.actor().as_str(), "actor-1");
        assert_eq!(request.tool().as_str(), "tool-1");
        assert_eq!(request.arguments().expose(), sentinel);
        assert!(!format!("{:?}", request.arguments()).contains(sentinel));
        assert!(!format!("{request:?}").contains(sentinel));
    }

    #[test]
    fn repeated_equivalent_construction_has_identical_safe_observations() {
        let build = || {
            request(
                ToolArguments::new("same").expect("arguments must pass"),
                [
                    ToolEffect::SensitiveDataExposure,
                    ToolEffect::ExternalMutation,
                ],
            )
            .expect("request must pass")
        };

        let first = build();
        let second = build();
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
        assert_eq!(first.arguments().expose(), second.arguments().expose());
        assert_eq!(first.effects(), second.effects());
    }
}
