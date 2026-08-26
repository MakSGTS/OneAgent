//! Client foundation for the accepted bounded LM Studio provider contract.
//!
//! The provider-neutral domain remains owned by `oneagent-llm`. This crate owns
//! deterministic LM Studio client construction, native model discovery, and a
//! private generation bridge through the existing OpenAI-compatible adapter.

mod config;
mod discovery;
mod execution;
mod generation;
mod wire;

#[cfg(test)]
mod test_support;

pub use config::LmStudioProvider;

const PROVIDER_ID: &str = "lm-studio";
const OPENAI_COMPATIBLE_PROVIDER_ID: &str = "openai-compatible";
const LOCAL_BASE_URL: &str = "http://127.0.0.1:1234";
const NATIVE_MODELS_PATH: &str = "api/v1/models";
const USER_AGENT: &str = "oneagent-lm-studio/0.1.0";

const MAX_BASE_URL_BYTES: usize = 2_048;
const MAX_NATIVE_MODELS_RESPONSE_BODY_BYTES: usize = 1_024 * 1_024;
