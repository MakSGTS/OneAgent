//! Stable provider-neutral failures and redacted diagnostics.

use std::fmt::{Debug, Display, Formatter};

/// Maximum retained provider diagnostic size in UTF-8 bytes.
pub const MAX_PROVIDER_DIAGNOSTIC_BYTES: usize = 512;

/// Stable provider-neutral failure classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LlmErrorKind {
    /// A provider identifier violated the accepted identity contract.
    InvalidProviderId,
    /// A model identifier violated the accepted identity contract.
    InvalidModelId,
    /// A discovered model catalog was inconsistent or out of bounds.
    InvalidModelCatalog,
    /// Provider construction input was invalid.
    InvalidConfiguration,
    /// A text-generation request was invalid.
    InvalidRequest,
    /// The selected model lacked a required capability.
    IncompatibleModel,
    /// A terminal provider result violated the shared response contract.
    InvalidResponse,
    /// The provider was temporarily unavailable.
    ProviderUnavailable,
    /// The provider rejected an otherwise valid request.
    ProviderRejected,
    /// A provider transport failed.
    Transport,
    /// A provider wire response violated its adapter protocol.
    Protocol,
    /// The accepted provider operation timeout elapsed.
    Timeout,
    /// Cooperative cancellation won the terminal race.
    Cancelled,
    /// The provider boundary encountered an internal contract failure.
    Internal,
}

impl LlmErrorKind {
    /// Returns whether a future caller may consider this kind retryable.
    ///
    /// Sprint 23 never performs an automatic retry.
    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::ProviderUnavailable | Self::Transport | Self::Timeout
        )
    }

    const fn message(self) -> &'static str {
        match self {
            Self::InvalidProviderId => "LLM provider identifier is invalid",
            Self::InvalidModelId => "LLM model identifier is invalid",
            Self::InvalidModelCatalog => "LLM model catalog is invalid",
            Self::InvalidConfiguration => "LLM provider configuration is invalid",
            Self::InvalidRequest => "LLM request is invalid",
            Self::IncompatibleModel => "LLM model is incompatible with the request",
            Self::InvalidResponse => "LLM provider response is invalid",
            Self::ProviderUnavailable => "LLM provider is unavailable",
            Self::ProviderRejected => "LLM provider rejected the request",
            Self::Transport => "LLM provider transport failed",
            Self::Protocol => "LLM provider protocol failed",
            Self::Timeout => "LLM provider operation timed out",
            Self::Cancelled => "LLM provider operation was cancelled",
            Self::Internal => "LLM provider boundary failed internally",
        }
    }
}

/// Explicitly accessible provider diagnostic retained under a strict size bound.
///
/// Callers constructing this value must redact credentials, sensitive URLs,
/// headers, request/response bodies, and unrestricted provider payloads first.
pub struct ProviderDiagnostic(Box<str>);

impl ProviderDiagnostic {
    /// Creates a bounded non-empty diagnostic.
    ///
    /// # Errors
    ///
    /// Returns an internal boundary error without retaining the rejected text
    /// when the value is empty or exceeds the accepted byte maximum.
    pub fn new(value: impl Into<String>) -> Result<Self, LlmError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::Internal,
                "provider diagnostic is empty",
            ));
        }
        if value.len() > MAX_PROVIDER_DIAGNOSTIC_BYTES {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::Internal,
                "provider diagnostic exceeds byte limit",
            ));
        }

        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the explicitly requested redacted diagnostic content.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the retained UTF-8 byte length.
    #[must_use]
    pub(crate) fn byte_len(&self) -> usize {
        self.0.len()
    }
}

/// A provider-neutral failure with an optional explicitly accessible diagnostic.
pub struct LlmError {
    kind: LlmErrorKind,
    diagnostic: Option<ProviderDiagnostic>,
}

impl LlmError {
    /// Creates an error without a provider diagnostic.
    #[must_use]
    pub const fn new(kind: LlmErrorKind) -> Self {
        Self {
            kind,
            diagnostic: None,
        }
    }

    /// Attaches a previously bounded and redacted provider diagnostic.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: ProviderDiagnostic) -> Self {
        self.diagnostic = Some(diagnostic);
        self
    }

    pub(crate) fn with_static_diagnostic(kind: LlmErrorKind, diagnostic: &'static str) -> Self {
        let diagnostic = ProviderDiagnostic(diagnostic.into());
        Self::new(kind).with_diagnostic(diagnostic)
    }

    /// Returns the stable failure classification.
    #[must_use]
    pub const fn kind(&self) -> LlmErrorKind {
        self.kind
    }

    /// Returns the explicitly accessible redacted diagnostic, when retained.
    #[must_use]
    pub const fn diagnostic(&self) -> Option<&ProviderDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl Debug for LlmError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LlmError")
            .field("kind", &self.kind)
            .field("has_diagnostic", &self.diagnostic.is_some())
            .field(
                "diagnostic_bytes",
                &self
                    .diagnostic
                    .as_ref()
                    .map_or(0, ProviderDiagnostic::byte_len),
            )
            .finish_non_exhaustive()
    }
}

impl Display for LlmError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.kind.message())
    }
}

impl std::error::Error for LlmError {}

#[cfg(test)]
mod tests {
    use super::{LlmError, LlmErrorKind, MAX_PROVIDER_DIAGNOSTIC_BYTES, ProviderDiagnostic};

    #[test]
    fn retryable_classification_is_closed_and_does_not_retry() {
        for kind in [
            LlmErrorKind::ProviderUnavailable,
            LlmErrorKind::Transport,
            LlmErrorKind::Timeout,
        ] {
            assert!(kind.is_retryable());
        }
        for kind in [
            LlmErrorKind::InvalidProviderId,
            LlmErrorKind::InvalidModelId,
            LlmErrorKind::InvalidModelCatalog,
            LlmErrorKind::InvalidConfiguration,
            LlmErrorKind::InvalidRequest,
            LlmErrorKind::IncompatibleModel,
            LlmErrorKind::InvalidResponse,
            LlmErrorKind::ProviderRejected,
            LlmErrorKind::Protocol,
            LlmErrorKind::Cancelled,
            LlmErrorKind::Internal,
        ] {
            assert!(!kind.is_retryable());
        }
    }

    #[test]
    fn diagnostics_are_bounded_and_implicit_formatting_is_redacted() {
        let sentinel = "synthetic-secret-sentinel";
        let diagnostic = ProviderDiagnostic::new(sentinel).expect("diagnostic must be accepted");
        let error = LlmError::new(LlmErrorKind::Transport).with_diagnostic(diagnostic);

        assert_eq!(
            error.diagnostic().map(ProviderDiagnostic::as_str),
            Some(sentinel)
        );
        assert!(!format!("{error}").contains(sentinel));
        assert!(!format!("{error:?}").contains(sentinel));
        assert_eq!(error.to_string(), "LLM provider transport failed");

        assert!(ProviderDiagnostic::new(" ").is_err());
        assert!(ProviderDiagnostic::new("x".repeat(MAX_PROVIDER_DIAGNOSTIC_BYTES)).is_ok());
        assert!(ProviderDiagnostic::new("x".repeat(MAX_PROVIDER_DIAGNOSTIC_BYTES + 1)).is_err());
    }
}
