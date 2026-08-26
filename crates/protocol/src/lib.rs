//! Protocol contracts used by `OneAgent` clients and services.

mod mcp;
mod server;

pub use mcp::{
    CLIENT_CAPABILITIES_META_KEY, CLIENT_INFO_META_KEY, DecodeOutcome, EncodeError, ErrorCode,
    ErrorResponse, Implementation, InboundMessage, MAX_JSON_NESTING_DEPTH, MAX_MESSAGE_BYTES,
    MAX_METHOD_NAME_BYTES, MAX_REQUEST_ID_BYTES, Notification, PROTOCOL_VERSION,
    PROTOCOL_VERSION_META_KEY, Request, RequestId, RequestMetadata, Response, ResultResponse,
    decode_message, encode_response,
};
pub use server::{MCP_SERVER_NAME, McpServer};

/// Returns the protocol component name.
#[must_use]
pub const fn component_name() -> &'static str {
    "protocol"
}
