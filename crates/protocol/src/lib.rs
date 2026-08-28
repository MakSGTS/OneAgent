//! Protocol contracts used by `OneAgent` clients and services.

mod lsp;
mod mcp;
mod server;
mod session;

pub use lsp::{
    LSP_SERVER_NAME, LspCapabilities, LspDispatchOutcome, LspEncodeError, LspErrorCode,
    LspExitStatus, LspHandler, LspHandlerError, LspResponse, LspServer, encode_lsp_response,
};

pub use mcp::{
    CLIENT_CAPABILITIES_META_KEY, CLIENT_INFO_META_KEY, DecodeOutcome, EncodeError, ErrorCode,
    ErrorResponse, Implementation, InboundMessage, MAX_JSON_NESTING_DEPTH, MAX_MESSAGE_BYTES,
    MAX_METHOD_NAME_BYTES, MAX_REQUEST_ID_BYTES, Notification, PROTOCOL_VERSION,
    PROTOCOL_VERSION_META_KEY, Request, RequestId, RequestMetadata, Response, ResultResponse,
    decode_message, encode_response,
};
pub use server::{
    MCP_SERVER_NAME, McpServer, McpToolAnnotations, McpToolCallHandler, McpToolCallOutcome,
    McpToolDefinition, McpToolDefinitionError, McpToolFuture,
};
pub use session::{
    MCP_PROTOCOL_VERSION_2025_06_18, MCP_PROTOCOL_VERSION_2025_11_25, McpConnection,
    McpProtocolRevision, SUPPORTED_MCP_PROTOCOL_VERSIONS,
};

/// Returns the protocol component name.
#[must_use]
pub const fn component_name() -> &'static str {
    "protocol"
}
