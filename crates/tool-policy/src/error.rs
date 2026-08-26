//! Stable Tool Execution Policy construction failures.

use std::fmt::{Debug, Display, Formatter};

/// Stable source-independent construction failure classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolPolicyErrorKind {
    /// A tool identifier violated the accepted identity contract.
    InvalidToolId,
    /// An actor identifier violated the accepted identity contract.
    InvalidActorId,
    /// A request identifier violated the accepted identity contract.
    InvalidRequestId,
    /// A policy revision violated the accepted identity contract.
    InvalidPolicyRevision,
    /// Tool arguments exceeded the accepted byte bound.
    InvalidArguments,
    /// A side-effect set was empty or contradictory.
    InvalidEffectSet,
    /// A policy violated its accepted construction contract.
    InvalidPolicy,
    /// A confirmation operation violated its accepted state contract.
    InvalidConfirmation,
    /// Executor output violated its accepted byte bound.
    InvalidOutput,
    /// A retained executor diagnostic violated its accepted contract.
    InvalidDiagnostic,
    /// The Tool Execution Policy boundary encountered an internal failure.
    Internal,
}

impl ToolPolicyErrorKind {
    const fn message(self) -> &'static str {
        match self {
            Self::InvalidToolId => "tool identifier is invalid",
            Self::InvalidActorId => "tool actor identifier is invalid",
            Self::InvalidRequestId => "tool request identifier is invalid",
            Self::InvalidPolicyRevision => "tool policy revision is invalid",
            Self::InvalidArguments => "tool arguments are invalid",
            Self::InvalidEffectSet => "tool effect set is invalid",
            Self::InvalidPolicy => "tool policy is invalid",
            Self::InvalidConfirmation => "tool confirmation is invalid",
            Self::InvalidOutput => "tool executor output is invalid",
            Self::InvalidDiagnostic => "tool executor diagnostic is invalid",
            Self::Internal => "tool policy boundary failed internally",
        }
    }
}

/// A typed Tool Execution Policy failure with a static non-sensitive diagnostic.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ToolPolicyError {
    kind: ToolPolicyErrorKind,
    diagnostic: &'static str,
}

impl ToolPolicyError {
    pub(crate) const fn new(kind: ToolPolicyErrorKind, diagnostic: &'static str) -> Self {
        Self { kind, diagnostic }
    }

    /// Returns the stable failure classification.
    #[must_use]
    pub const fn kind(self) -> ToolPolicyErrorKind {
        self.kind
    }

    /// Returns the explicitly requested static non-sensitive diagnostic.
    #[must_use]
    pub const fn diagnostic(self) -> &'static str {
        self.diagnostic
    }
}

impl Debug for ToolPolicyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolPolicyError")
            .field("kind", &self.kind)
            .field("has_diagnostic", &true)
            .finish_non_exhaustive()
    }
}

impl Display for ToolPolicyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.kind.message())
    }
}

impl std::error::Error for ToolPolicyError {}

#[cfg(test)]
mod tests {
    use super::{ToolPolicyError, ToolPolicyErrorKind};

    #[test]
    fn implicit_error_formatting_uses_only_the_stable_kind() {
        let sentinel = "synthetic-rejected-sensitive-value";
        let error = ToolPolicyError::new(ToolPolicyErrorKind::InvalidArguments, sentinel);

        assert_eq!(error.kind(), ToolPolicyErrorKind::InvalidArguments);
        assert_eq!(error.diagnostic(), sentinel);
        assert_eq!(error.to_string(), "tool arguments are invalid");
        assert!(!format!("{error}").contains(sentinel));
        assert!(!format!("{error:?}").contains(sentinel));
    }
}
