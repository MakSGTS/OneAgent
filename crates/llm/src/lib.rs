//! Provider-independent LLM domain contracts for `OneAgent`.
//!
//! This crate owns no provider transport, executor, Runtime state, semantic
//! graph, or provider-specific wire schema.

mod error;
mod identity;
mod model;
mod policy;
mod provider;
mod request;
mod response;
mod secret;

pub use error::{LlmError, LlmErrorKind, MAX_PROVIDER_DIAGNOSTIC_BYTES, ProviderDiagnostic};
pub use identity::{MAX_MODEL_ID_BYTES, MAX_PROVIDER_ID_BYTES, ModelId, ModelIdentity, ProviderId};
pub use model::{MAX_MODELS_PER_CATALOG, ModelCapability, ModelCatalog, ModelDescriptor};
pub use policy::{
    MAX_PROVIDER_TIMEOUT, MAX_PROVIDER_TIMEOUT_SECS, ProviderExecutionPolicy, RetryPolicy,
};
pub use provider::{
    CancellationSignal, LlmProvider, NeverCancelled, ProviderExecutionContext, ProviderFuture,
};
pub use request::TextGenerationRequest;
pub use response::{
    FinishReason, MAX_TEXT_INPUT_BYTES, MAX_TEXT_OUTPUT_BYTES, TextGenerationResponse, TextUsage,
};
pub use secret::{MAX_PROVIDER_SECRET_BYTES, ProviderConfiguration, ProviderSecret};
