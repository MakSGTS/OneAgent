//! Client foundation for the accepted bounded Ollama provider contract.
//!
//! The provider-neutral domain remains owned by `oneagent-llm`. This crate owns
//! deterministic local Ollama client construction, bounded native model
//! discovery, and one non-streaming raw text-generation operation.

mod config;
mod discovery;
mod execution;
mod generation;
mod wire;

#[cfg(test)]
mod test_support;

pub use config::OllamaProvider;

const PROVIDER_ID: &str = "ollama";
const LOCAL_BASE_URL: &str = "http://127.0.0.1:11434";
const TAGS_PATH: &str = "api/tags";
const SHOW_PATH: &str = "api/show";
const GENERATE_PATH: &str = "api/generate";
const USER_AGENT: &str = "oneagent-ollama/0.1.0";

const MAX_BASE_URL_BYTES: usize = 2_048;
const MAX_TAGS_RESPONSE_BODY_BYTES: usize = 1_024 * 1_024;
const MAX_SHOW_REQUEST_BODY_BYTES: usize = 4 * 1_024;
const MAX_SHOW_RESPONSE_BODY_BYTES: usize = 1_024 * 1_024;
const MAX_GENERATE_REQUEST_BODY_BYTES: usize = 512 * 1_024;
const MAX_GENERATE_RESPONSE_BODY_BYTES: usize = 512 * 1_024;
