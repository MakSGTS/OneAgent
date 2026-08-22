//! Provider-scoped identity values.

use std::fmt::{Display, Formatter};

use crate::{LlmError, LlmErrorKind};

/// Maximum provider identifier size in UTF-8 bytes.
pub const MAX_PROVIDER_ID_BYTES: usize = 128;
/// Maximum model identifier size in UTF-8 bytes.
pub const MAX_MODEL_ID_BYTES: usize = 256;

fn validate_identifier(
    value: &str,
    maximum: usize,
    kind: LlmErrorKind,
    field: &'static str,
) -> Result<(), LlmError> {
    if value.trim().is_empty() {
        return Err(LlmError::with_static_diagnostic(
            kind,
            match field {
                "provider_id" => "provider_id is empty",
                "model_id" => "model_id is empty",
                _ => "identifier is empty",
            },
        ));
    }
    if value.len() > maximum {
        return Err(LlmError::with_static_diagnostic(
            kind,
            match field {
                "provider_id" => "provider_id exceeds byte limit",
                "model_id" => "model_id exceeds byte limit",
                _ => "identifier exceeds byte limit",
            },
        ));
    }
    if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
        return Err(LlmError::with_static_diagnostic(
            kind,
            match field {
                "provider_id" => "provider_id has boundary whitespace",
                "model_id" => "model_id has boundary whitespace",
                _ => "identifier has boundary whitespace",
            },
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(LlmError::with_static_diagnostic(
            kind,
            match field {
                "provider_id" => "provider_id contains a control character",
                "model_id" => "model_id contains a control character",
                _ => "identifier contains a control character",
            },
        ));
    }

    Ok(())
}

/// Stable case-sensitive provider identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderId(Box<str>);

impl ProviderId {
    /// Creates an accepted provider identifier without normalization.
    ///
    /// # Errors
    ///
    /// Returns a typed failure for empty, over-limit, boundary-whitespace, or
    /// control-character input without echoing the rejected value.
    pub fn new(value: impl Into<String>) -> Result<Self, LlmError> {
        let value = value.into();
        validate_identifier(
            &value,
            MAX_PROVIDER_ID_BYTES,
            LlmErrorKind::InvalidProviderId,
            "provider_id",
        )?;
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the exact accepted provider identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ProviderId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Stable case-sensitive model identifier scoped by a provider.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelId(Box<str>);

impl ModelId {
    /// Creates an accepted model identifier without normalization.
    ///
    /// # Errors
    ///
    /// Returns a typed failure for empty, over-limit, boundary-whitespace, or
    /// control-character input without echoing the rejected value.
    pub fn new(value: impl Into<String>) -> Result<Self, LlmError> {
        let value = value.into();
        validate_identifier(
            &value,
            MAX_MODEL_ID_BYTES,
            LlmErrorKind::InvalidModelId,
            "model_id",
        )?;
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the exact accepted model identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ModelId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Provider-scoped model identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelIdentity {
    provider: ProviderId,
    model: ModelId,
}

impl ModelIdentity {
    /// Creates a provider-scoped model identity from validated components.
    #[must_use]
    pub const fn new(provider: ProviderId, model: ModelId) -> Self {
        Self { provider, model }
    }

    /// Returns the provider scope.
    #[must_use]
    pub const fn provider(&self) -> &ProviderId {
        &self.provider
    }

    /// Returns the model identifier inside the provider scope.
    #[must_use]
    pub const fn model(&self) -> &ModelId {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_MODEL_ID_BYTES, MAX_PROVIDER_ID_BYTES, ModelId, ModelIdentity, ProviderId};
    use crate::LlmErrorKind;

    #[test]
    fn provider_identity_validation_has_accepted_precedence_and_boundaries() {
        for value in ["", " ", " provider", "provider ", "provider\n"] {
            let error = ProviderId::new(value).expect_err("provider ID must fail");
            assert_eq!(error.kind(), LlmErrorKind::InvalidProviderId);
        }
        assert!(ProviderId::new("x".repeat(MAX_PROVIDER_ID_BYTES)).is_ok());
        assert!(ProviderId::new("x".repeat(MAX_PROVIDER_ID_BYTES + 1)).is_err());

        let preserved = ProviderId::new("OpenAI-Compatible").expect("provider ID must pass");
        assert_eq!(preserved.as_str(), "OpenAI-Compatible");
        assert_eq!(preserved.to_string(), "OpenAI-Compatible");
    }

    #[test]
    fn model_identity_is_provider_scoped_and_totally_ordered() {
        assert!(ModelId::new("x".repeat(MAX_MODEL_ID_BYTES)).is_ok());
        assert!(ModelId::new("x".repeat(MAX_MODEL_ID_BYTES + 1)).is_err());

        let provider_a = ProviderId::new("a").expect("provider ID must pass");
        let provider_b = ProviderId::new("b").expect("provider ID must pass");
        let model_a = ModelId::new("model").expect("model ID must pass");
        let model_b = ModelId::new("model").expect("model ID must pass");
        let first = ModelIdentity::new(provider_a, model_a);
        let second = ModelIdentity::new(provider_b, model_b);

        assert!(first < second);
        assert_eq!(first.model().as_str(), "model");
    }
}
