//! Provider-neutral model capabilities and discovery projection.

use std::collections::BTreeSet;

use crate::{LlmError, LlmErrorKind, ModelIdentity, ProviderId};

/// Maximum number of models accepted in one provider catalog.
pub const MAX_MODELS_PER_CATALOG: usize = 1_024;

/// Closed Sprint 23 model capability vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelCapability {
    /// One bounded text input can produce one terminal bounded text output.
    TextGeneration,
}

/// One provider-scoped model and its canonical capability set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDescriptor {
    identity: ModelIdentity,
    capabilities: BTreeSet<ModelCapability>,
}

impl ModelDescriptor {
    /// Creates a descriptor and canonicalizes duplicate capability input.
    #[must_use]
    pub fn new(
        identity: ModelIdentity,
        capabilities: impl IntoIterator<Item = ModelCapability>,
    ) -> Self {
        Self {
            identity,
            capabilities: capabilities.into_iter().collect(),
        }
    }

    /// Returns the provider-scoped model identity.
    #[must_use]
    pub const fn identity(&self) -> &ModelIdentity {
        &self.identity
    }

    /// Returns capabilities in stable enum order.
    #[must_use]
    pub const fn capabilities(&self) -> &BTreeSet<ModelCapability> {
        &self.capabilities
    }

    /// Returns whether the model advertises the supplied capability.
    #[must_use]
    pub fn supports(&self, capability: ModelCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// Deterministic owned result of one provider model-discovery call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalog {
    provider: ProviderId,
    models: Vec<ModelDescriptor>,
}

impl ModelCatalog {
    /// Creates a canonical provider catalog.
    ///
    /// # Errors
    ///
    /// Returns a typed failure when the model count exceeds the accepted bound,
    /// a model has another provider scope, or a model identity is duplicated.
    pub fn new(provider: ProviderId, mut models: Vec<ModelDescriptor>) -> Result<Self, LlmError> {
        if models.len() > MAX_MODELS_PER_CATALOG {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidModelCatalog,
                "model catalog exceeds model count limit",
            ));
        }
        if models
            .iter()
            .any(|model| model.identity().provider() != &provider)
        {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidModelCatalog,
                "model catalog contains another provider scope",
            ));
        }

        models.sort_by(|left, right| left.identity().cmp(right.identity()));
        if models
            .windows(2)
            .any(|pair| pair[0].identity() == pair[1].identity())
        {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidModelCatalog,
                "model catalog contains a duplicate model identity",
            ));
        }

        Ok(Self { provider, models })
    }

    /// Returns the provider that produced this catalog.
    #[must_use]
    pub const fn provider(&self) -> &ProviderId {
        &self.provider
    }

    /// Returns model descriptors in stable full-identity order.
    #[must_use]
    pub fn models(&self) -> &[ModelDescriptor] {
        &self.models
    }

    /// Returns whether discovery produced no models.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_MODELS_PER_CATALOG, ModelCapability, ModelCatalog, ModelDescriptor};
    use crate::{LlmErrorKind, ModelId, ModelIdentity, ProviderId};

    fn identity(provider: &ProviderId, model: &str) -> ModelIdentity {
        ModelIdentity::new(
            provider.clone(),
            ModelId::new(model).expect("model ID must pass"),
        )
    }

    #[test]
    fn capabilities_are_canonical_and_empty_is_compatible_with_discovery() {
        let provider = ProviderId::new("provider").expect("provider ID must pass");
        let model = ModelDescriptor::new(
            identity(&provider, "model"),
            [
                ModelCapability::TextGeneration,
                ModelCapability::TextGeneration,
            ],
        );

        assert_eq!(model.capabilities().len(), 1);
        assert!(model.supports(ModelCapability::TextGeneration));

        let empty = ModelCatalog::new(provider, Vec::new()).expect("empty catalog must pass");
        assert!(empty.is_empty());
    }

    #[test]
    fn catalog_sorts_models_and_rejects_duplicates_and_wrong_provider() {
        let provider = ProviderId::new("provider").expect("provider ID must pass");
        let later = ModelDescriptor::new(identity(&provider, "z"), []);
        let earlier = ModelDescriptor::new(identity(&provider, "a"), []);
        let catalog =
            ModelCatalog::new(provider.clone(), vec![later, earlier]).expect("catalog must pass");

        assert_eq!(catalog.models()[0].identity().model().as_str(), "a");
        assert_eq!(catalog.models()[1].identity().model().as_str(), "z");

        let duplicate = ModelDescriptor::new(identity(&provider, "a"), []);
        let error = ModelCatalog::new(provider.clone(), vec![duplicate.clone(), duplicate])
            .expect_err("duplicate must fail");
        assert_eq!(error.kind(), LlmErrorKind::InvalidModelCatalog);

        let other = ProviderId::new("other").expect("provider ID must pass");
        let wrong = ModelDescriptor::new(identity(&other, "model"), []);
        assert!(ModelCatalog::new(provider, vec![wrong]).is_err());
    }

    #[test]
    fn catalog_model_count_bound_is_exact() {
        let provider = ProviderId::new("provider").expect("provider ID must pass");
        let models = (0..MAX_MODELS_PER_CATALOG)
            .map(|index| ModelDescriptor::new(identity(&provider, &format!("model-{index}")), []))
            .collect();
        assert!(ModelCatalog::new(provider.clone(), models).is_ok());

        let over_limit = (0..=MAX_MODELS_PER_CATALOG)
            .map(|index| ModelDescriptor::new(identity(&provider, &format!("model-{index}")), []))
            .collect();
        assert!(ModelCatalog::new(provider, over_limit).is_err());
    }
}
