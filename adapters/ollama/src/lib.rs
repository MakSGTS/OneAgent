//! Client foundation for the accepted bounded Ollama provider contract.
//!
//! The provider-neutral domain remains owned by `oneagent-llm`. This crate owns
//! deterministic local Ollama client construction and private native endpoint
//! state. Discovery and generation are introduced by later Sprint 26 tasks.

mod config;

pub use config::OllamaProvider;

const PROVIDER_ID: &str = "ollama";
const LOCAL_BASE_URL: &str = "http://127.0.0.1:11434";
const TAGS_PATH: &str = "api/tags";
const SHOW_PATH: &str = "api/show";
const GENERATE_PATH: &str = "api/generate";
const USER_AGENT: &str = "oneagent-ollama/0.1.0";

const MAX_BASE_URL_BYTES: usize = 2_048;
