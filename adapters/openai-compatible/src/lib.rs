//! Concrete bounded adapter for the accepted OpenAI-compatible HTTP contract.
//!
//! The provider-neutral domain remains owned by `oneagent-llm`. This crate owns
//! only explicit client construction and private provider wire values until the
//! discovery and generation operations are implemented.

mod config;
mod wire;

pub use config::OpenAiCompatibleProvider;

const PROVIDER_ID: &str = "openai-compatible";
const MODELS_PATH: &str = "v1/models";
const COMPLETIONS_PATH: &str = "v1/completions";
const USER_AGENT: &str = "oneagent-openai-compatible/0.1.0";

const MAX_BASE_URL_BYTES: usize = 2_048;
#[allow(dead_code)]
const MAX_COMPLETION_REQUEST_BODY_BYTES: usize = 512 * 1_024;
#[allow(dead_code)]
const MAX_MODELS_RESPONSE_BODY_BYTES: usize = 1_024 * 1_024;
#[allow(dead_code)]
const MAX_COMPLETION_RESPONSE_BODY_BYTES: usize = 512 * 1_024;
