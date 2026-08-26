//! Stable Tool Execution Policy identities.

use std::fmt::{Display, Formatter};

use crate::{ToolPolicyError, ToolPolicyErrorKind};

/// Maximum tool identifier size in UTF-8 bytes.
pub const MAX_TOOL_ID_BYTES: usize = 128;
/// Maximum actor identifier size in UTF-8 bytes.
pub const MAX_ACTOR_ID_BYTES: usize = 128;
/// Maximum request identifier size in UTF-8 bytes.
pub const MAX_TOOL_REQUEST_ID_BYTES: usize = 128;
/// Maximum policy revision size in UTF-8 bytes.
pub const MAX_POLICY_REVISION_BYTES: usize = 128;

fn validate_identifier(
    value: &str,
    maximum: usize,
    kind: ToolPolicyErrorKind,
    empty: &'static str,
    over_limit: &'static str,
    boundary_whitespace: &'static str,
    control: &'static str,
) -> Result<(), ToolPolicyError> {
    if value.trim().is_empty() {
        return Err(ToolPolicyError::new(kind, empty));
    }
    if value.len() > maximum {
        return Err(ToolPolicyError::new(kind, over_limit));
    }
    if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
        return Err(ToolPolicyError::new(kind, boundary_whitespace));
    }
    if value.chars().any(char::is_control) {
        return Err(ToolPolicyError::new(kind, control));
    }
    Ok(())
}

macro_rules! identity_type {
    (
        $name:ident,
        $maximum:ident,
        $kind:ident,
        $description:literal,
        $empty:literal,
        $over:literal,
        $boundary:literal,
        $control:literal
    ) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Box<str>);

        impl $name {
            /// Creates an accepted identity without normalization.
            ///
            /// # Errors
            ///
            /// Returns a typed failure for empty, over-limit,
            /// boundary-whitespace, or control-character input.
            pub fn new(value: impl Into<String>) -> Result<Self, ToolPolicyError> {
                let value = value.into();
                validate_identifier(
                    &value,
                    $maximum,
                    ToolPolicyErrorKind::$kind,
                    $empty,
                    $over,
                    $boundary,
                    $control,
                )?;
                Ok(Self(value.into_boxed_str()))
            }

            /// Returns the exact accepted identity bytes as UTF-8.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

identity_type!(
    ToolId,
    MAX_TOOL_ID_BYTES,
    InvalidToolId,
    "Stable case-sensitive source-independent tool identifier.",
    "tool_id is empty",
    "tool_id exceeds byte limit",
    "tool_id has boundary whitespace",
    "tool_id contains a control character"
);

identity_type!(
    ActorId,
    MAX_ACTOR_ID_BYTES,
    InvalidActorId,
    "Stable case-sensitive actor label presented to policy evaluation.",
    "actor_id is empty",
    "actor_id exceeds byte limit",
    "actor_id has boundary whitespace",
    "actor_id contains a control character"
);

identity_type!(
    ToolRequestId,
    MAX_TOOL_REQUEST_ID_BYTES,
    InvalidRequestId,
    "Stable case-sensitive caller-supplied tool request identifier.",
    "request_id is empty",
    "request_id exceeds byte limit",
    "request_id has boundary whitespace",
    "request_id contains a control character"
);

identity_type!(
    PolicyRevision,
    MAX_POLICY_REVISION_BYTES,
    InvalidPolicyRevision,
    "Stable case-sensitive caller-supplied policy revision.",
    "policy_revision is empty",
    "policy_revision exceeds byte limit",
    "policy_revision has boundary whitespace",
    "policy_revision contains a control character"
);

#[cfg(test)]
mod tests {
    use super::{
        ActorId, MAX_ACTOR_ID_BYTES, MAX_POLICY_REVISION_BYTES, MAX_TOOL_ID_BYTES,
        MAX_TOOL_REQUEST_ID_BYTES, PolicyRevision, ToolId, ToolRequestId,
    };
    use crate::ToolPolicyErrorKind;

    #[test]
    fn identity_validation_precedence_and_exact_bounds_are_stable() {
        let cases = [
            (
                ToolId::new(" ").expect_err("empty tool ID must fail"),
                ToolPolicyErrorKind::InvalidToolId,
                "tool_id is empty",
            ),
            (
                ActorId::new("x".repeat(MAX_ACTOR_ID_BYTES + 1))
                    .expect_err("over-limit actor ID must fail"),
                ToolPolicyErrorKind::InvalidActorId,
                "actor_id exceeds byte limit",
            ),
            (
                ToolRequestId::new(" request").expect_err("boundary whitespace must fail"),
                ToolPolicyErrorKind::InvalidRequestId,
                "request_id has boundary whitespace",
            ),
            (
                PolicyRevision::new("revision\n")
                    .expect_err("boundary control must fail after whitespace"),
                ToolPolicyErrorKind::InvalidPolicyRevision,
                "policy_revision has boundary whitespace",
            ),
        ];

        for (error, kind, diagnostic) in cases {
            assert_eq!(error.kind(), kind);
            assert_eq!(error.diagnostic(), diagnostic);
        }

        assert!(ToolId::new("x".repeat(MAX_TOOL_ID_BYTES)).is_ok());
        assert!(ActorId::new("x".repeat(MAX_ACTOR_ID_BYTES)).is_ok());
        assert!(ToolRequestId::new("x".repeat(MAX_TOOL_REQUEST_ID_BYTES)).is_ok());
        assert!(PolicyRevision::new("x".repeat(MAX_POLICY_REVISION_BYTES)).is_ok());
    }

    #[test]
    fn accepted_identity_bytes_are_preserved_and_totally_ordered() {
        let first = ToolId::new("Tool-A").expect("tool ID must pass");
        let second = ToolId::new("tool-a").expect("tool ID must pass");

        assert_eq!(first.as_str(), "Tool-A");
        assert_eq!(first.to_string(), "Tool-A");
        assert!(first < second);
    }

    #[test]
    fn control_characters_are_rejected_without_echoing_input() {
        let sentinel = "actor\u{0007}sentinel";
        let error = ActorId::new(sentinel).expect_err("control character must fail");

        assert_eq!(error.kind(), ToolPolicyErrorKind::InvalidActorId);
        assert_eq!(error.diagnostic(), "actor_id contains a control character");
        assert!(!format!("{error:?}").contains(sentinel));
        assert!(!format!("{error}").contains(sentinel));
    }
}
