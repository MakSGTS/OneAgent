//! Secret-safe provider construction values.

use std::fmt::{Debug, Formatter};

use crate::{LlmError, LlmErrorKind, ProviderId};

/// Maximum provider secret size in UTF-8 bytes.
pub const MAX_PROVIDER_SECRET_BYTES: usize = 4_096;

/// Opaque provider credential with explicit content access and redacted debug output.
pub struct ProviderSecret(Box<str>);

impl ProviderSecret {
    /// Creates a bounded non-empty provider secret.
    ///
    /// # Errors
    ///
    /// Returns a configuration failure without retaining or reporting the raw
    /// value when it is empty or exceeds the byte maximum.
    pub fn new(value: impl Into<String>) -> Result<Self, LlmError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidConfiguration,
                "provider secret is empty",
            ));
        }
        if value.len() > MAX_PROVIDER_SECRET_BYTES {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidConfiguration,
                "provider secret exceeds byte limit",
            ));
        }

        Ok(Self(value.into_boxed_str()))
    }

    /// Exposes credential content explicitly for concrete provider construction.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl Debug for ProviderSecret {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProviderSecret([REDACTED])")
    }
}

/// Provider-scoped construction input with an optional credential.
pub struct ProviderConfiguration {
    provider: ProviderId,
    credential: Option<ProviderSecret>,
}

impl ProviderConfiguration {
    /// Creates provider construction input without reading an external source.
    #[must_use]
    pub const fn new(provider: ProviderId, credential: Option<ProviderSecret>) -> Self {
        Self {
            provider,
            credential,
        }
    }

    /// Returns the configured provider identity.
    #[must_use]
    pub const fn provider(&self) -> &ProviderId {
        &self.provider
    }

    /// Returns the explicitly accessible credential when present.
    #[must_use]
    pub const fn credential(&self) -> Option<&ProviderSecret> {
        self.credential.as_ref()
    }

    /// Splits the configuration into provider identity and credential ownership.
    #[must_use]
    pub fn into_parts(self) -> (ProviderId, Option<ProviderSecret>) {
        (self.provider, self.credential)
    }
}

impl Debug for ProviderConfiguration {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProviderConfiguration")
            .field("provider", &self.provider)
            .field("has_credential", &self.credential.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_PROVIDER_SECRET_BYTES, ProviderConfiguration, ProviderSecret};
    use crate::{LlmErrorKind, ProviderId};

    #[test]
    fn secret_bounds_are_exact_and_rejected_values_are_not_reported() {
        let sentinel = "synthetic-secret-sentinel";
        let secret = ProviderSecret::new(sentinel).expect("secret must pass");
        assert_eq!(secret.expose(), sentinel);
        assert_eq!(format!("{secret:?}"), "ProviderSecret([REDACTED])");

        let empty = ProviderSecret::new(" ").expect_err("empty secret must fail");
        assert_eq!(empty.kind(), LlmErrorKind::InvalidConfiguration);
        assert!(!format!("{empty:?}").contains(sentinel));

        assert!(ProviderSecret::new("x".repeat(MAX_PROVIDER_SECRET_BYTES)).is_ok());
        assert!(ProviderSecret::new("x".repeat(MAX_PROVIDER_SECRET_BYTES + 1)).is_err());
    }

    #[test]
    fn configuration_debug_reports_only_credential_presence() {
        let sentinel = "synthetic-secret-sentinel";
        let configuration = ProviderConfiguration::new(
            ProviderId::new("provider").expect("provider ID must pass"),
            Some(ProviderSecret::new(sentinel).expect("secret must pass")),
        );
        let debug = format!("{configuration:?}");

        assert!(debug.contains("has_credential: true"));
        assert!(!debug.contains(sentinel));
        assert_eq!(configuration.provider().as_str(), "provider");
        assert_eq!(
            configuration.credential().map(ProviderSecret::expose),
            Some(sentinel)
        );
    }
}
