//! Source-independent Tool Execution Policy domain contracts for `OneAgent`.
//!
//! This crate owns no concrete tool, transport, provider wire, Runtime state,
//! policy storage, confirmation user experience, clock, or external side effect.

mod error;
mod identity;
mod request;

pub use error::{ToolPolicyError, ToolPolicyErrorKind};
pub use identity::{
    ActorId, MAX_ACTOR_ID_BYTES, MAX_POLICY_REVISION_BYTES, MAX_TOOL_ID_BYTES,
    MAX_TOOL_REQUEST_ID_BYTES, PolicyRevision, ToolId, ToolRequestId,
};
pub use request::{MAX_TOOL_ARGUMENT_BYTES, ToolArguments, ToolEffect, ToolRequest};
