//! Connection-owned compatibility for negotiated and stateless MCP revisions.

use std::fmt;

use serde_json::{Map, Value, json};

use crate::{
    DecodeOutcome, ErrorCode, ErrorResponse, Implementation, InboundMessage, McpServer, Request,
    RequestId, Response, ResultResponse,
    mcp::{
        RawDecodeOutcome, decode_implementation, decode_raw_message, decode_request,
        validate_legacy_request_metadata,
    },
    server::McpResponseProfile,
};

/// The legacy MCP revision used by the pinned Codex client.
pub const MCP_PROTOCOL_VERSION_2025_06_18: &str = "2025-06-18";
/// The legacy MCP revision used by the pinned Cursor client.
pub const MCP_PROTOCOL_VERSION_2025_11_25: &str = "2025-11-25";
/// Supported MCP revisions in canonical newest-to-oldest order.
pub const SUPPORTED_MCP_PROTOCOL_VERSIONS: [&str; 3] = [
    crate::PROTOCOL_VERSION,
    MCP_PROTOCOL_VERSION_2025_11_25,
    MCP_PROTOCOL_VERSION_2025_06_18,
];

const INITIALIZE_METHOD: &str = "initialize";
const INITIALIZED_NOTIFICATION: &str = "notifications/initialized";
const PING_METHOD: &str = "ping";
const TOOLS_LIST_METHOD: &str = "tools/list";
const TOOLS_CALL_METHOD: &str = "tools/call";

/// A supported MCP protocol revision selected for one connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpProtocolRevision {
    /// Negotiated MCP `2025-06-18`.
    V2025_06_18,
    /// Negotiated MCP `2025-11-25`.
    V2025_11_25,
    /// Stateless MCP `2026-07-28`.
    V2026_07_28,
}

impl McpProtocolRevision {
    /// Returns the exact wire revision.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V2025_06_18 => MCP_PROTOCOL_VERSION_2025_06_18,
            Self::V2025_11_25 => MCP_PROTOCOL_VERSION_2025_11_25,
            Self::V2026_07_28 => crate::PROTOCOL_VERSION,
        }
    }

    fn negotiate_legacy(requested: &str) -> Self {
        match requested {
            MCP_PROTOCOL_VERSION_2025_06_18 => Self::V2025_06_18,
            _ => Self::V2025_11_25,
        }
    }
}

/// One bounded connection-local MCP compatibility state machine.
pub struct McpConnection<'server> {
    server: &'server McpServer,
    state: ConnectionState,
}

impl<'server> McpConnection<'server> {
    pub(crate) const fn new(server: &'server McpServer) -> Self {
        Self {
            server,
            state: ConnectionState::Undetermined,
        }
    }

    /// Returns the selected revision after a valid response-producing message.
    #[must_use]
    pub const fn protocol_revision(&self) -> Option<McpProtocolRevision> {
        match &self.state {
            ConnectionState::Undetermined => None,
            ConnectionState::LegacyAwaitingInitialized(facts)
            | ConnectionState::LegacyActive(facts) => Some(facts.revision),
            ConnectionState::Modern => Some(McpProtocolRevision::V2026_07_28),
        }
    }

    /// Returns whether this connection can accept operational requests.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::LegacyActive(_) | ConnectionState::Modern
        )
    }

    /// Decodes and dispatches one complete UTF-8 JSON frame in connection context.
    ///
    /// Returns `None` for notifications. The method performs no I/O, starts no
    /// task, and retains only negotiated protocol and lifecycle facts.
    #[must_use]
    pub async fn dispatch(&mut self, input: &str) -> Option<Response> {
        match self.state.kind() {
            ConnectionStateKind::Undetermined => self.dispatch_undetermined(input).await,
            ConnectionStateKind::LegacyAwaitingInitialized => {
                self.dispatch_awaiting_initialized(input)
            }
            ConnectionStateKind::LegacyActive => self.dispatch_legacy_active(input).await,
            ConnectionStateKind::Modern => self.server.dispatch(input).await,
        }
    }

    async fn dispatch_undetermined(&mut self, input: &str) -> Option<Response> {
        match decode_raw_message(input) {
            RawDecodeOutcome::Error(error) => Some(Response::Error(error)),
            RawDecodeOutcome::IgnoredNotification | RawDecodeOutcome::Notification(_) => None,
            RawDecodeOutcome::Request { id, method, params } if method == INITIALIZE_METHOD => {
                if matches!(
                    decode_request(id.clone(), method.clone(), params.as_ref()),
                    DecodeOutcome::Message(InboundMessage::Request(_))
                ) {
                    self.state = ConnectionState::Modern;
                    return self.server.dispatch(input).await;
                }
                match decode_initialize(self.server, id, params.as_ref()) {
                    Ok((facts, response)) => {
                        self.state = ConnectionState::LegacyAwaitingInitialized(facts);
                        Some(response)
                    }
                    Err(response) => Some(response),
                }
            }
            RawDecodeOutcome::Request { id, method, params } => {
                match decode_request(id.clone(), method.clone(), params.as_ref()) {
                    DecodeOutcome::Message(InboundMessage::Request(_)) => {
                        self.state = ConnectionState::Modern;
                        self.server.dispatch(input).await
                    }
                    DecodeOutcome::Error(_) if is_legacy_operational_method(&method) => {
                        Some(server_not_initialized(id))
                    }
                    DecodeOutcome::Error(error) => Some(Response::Error(error)),
                    DecodeOutcome::IgnoredNotification
                    | DecodeOutcome::Message(InboundMessage::Notification(_)) => {
                        unreachable!("a raw request cannot decode as a notification")
                    }
                }
            }
        }
    }

    fn dispatch_awaiting_initialized(&mut self, input: &str) -> Option<Response> {
        match decode_raw_message(input) {
            RawDecodeOutcome::Error(error) => Some(Response::Error(error)),
            RawDecodeOutcome::Notification(notification)
                if notification.method() == INITIALIZED_NOTIFICATION =>
            {
                let metadata_is_valid = notification
                    .params()
                    .is_none_or(|params| params.get("_meta").is_none_or(Value::is_object));
                if metadata_is_valid
                    && let ConnectionState::LegacyAwaitingInitialized(facts) = &self.state
                {
                    self.state = ConnectionState::LegacyActive(facts.clone());
                }
                None
            }
            RawDecodeOutcome::IgnoredNotification | RawDecodeOutcome::Notification(_) => None,
            RawDecodeOutcome::Request { id, method, .. } if method == INITIALIZE_METHOD => {
                Some(standard_error(Some(id), ErrorCode::InvalidRequest))
            }
            RawDecodeOutcome::Request { id, method, .. }
                if is_legacy_operational_method(&method) =>
            {
                Some(server_not_initialized(id))
            }
            RawDecodeOutcome::Request { id, .. } => {
                Some(standard_error(Some(id), ErrorCode::MethodNotFound))
            }
        }
    }

    async fn dispatch_legacy_active(&self, input: &str) -> Option<Response> {
        let ConnectionState::LegacyActive(facts) = &self.state else {
            unreachable!("legacy-active dispatch requires legacy facts")
        };
        match decode_raw_message(input) {
            RawDecodeOutcome::Error(error) => Some(Response::Error(error)),
            RawDecodeOutcome::IgnoredNotification | RawDecodeOutcome::Notification(_) => None,
            RawDecodeOutcome::Request { id, method, .. } if method == INITIALIZE_METHOD => {
                Some(standard_error(Some(id), ErrorCode::InvalidRequest))
            }
            RawDecodeOutcome::Request { id, method, params } => {
                let request = match decode_legacy_request(id, method, params.as_ref(), facts) {
                    Ok(request) => request,
                    Err(response) => return Some(response),
                };
                match request.method() {
                    PING_METHOD if request.params().keys().all(|key| key == "_meta") => {
                        Some(legacy_result(request.id().clone(), Map::new()))
                    }
                    PING_METHOD => Some(standard_error(
                        Some(request.id().clone()),
                        ErrorCode::InvalidParams,
                    )),
                    TOOLS_LIST_METHOD | TOOLS_CALL_METHOD => Some(
                        self.server
                            .dispatch_request_with_profile(&request, McpResponseProfile::Legacy)
                            .await,
                    ),
                    _ => Some(standard_error(
                        Some(request.id().clone()),
                        ErrorCode::MethodNotFound,
                    )),
                }
            }
        }
    }
}

impl McpServer {
    /// Creates one fresh compatibility state machine borrowing this server.
    #[must_use]
    pub const fn connection(&self) -> McpConnection<'_> {
        McpConnection::new(self)
    }
}

impl fmt::Debug for McpConnection<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("McpConnection")
            .field("state", &self.state.kind())
            .field("protocol_revision", &self.protocol_revision())
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
enum ConnectionState {
    Undetermined,
    LegacyAwaitingInitialized(LegacyFacts),
    LegacyActive(LegacyFacts),
    Modern,
}

impl ConnectionState {
    const fn kind(&self) -> ConnectionStateKind {
        match self {
            Self::Undetermined => ConnectionStateKind::Undetermined,
            Self::LegacyAwaitingInitialized(_) => ConnectionStateKind::LegacyAwaitingInitialized,
            Self::LegacyActive(_) => ConnectionStateKind::LegacyActive,
            Self::Modern => ConnectionStateKind::Modern,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionStateKind {
    Undetermined,
    LegacyAwaitingInitialized,
    LegacyActive,
    Modern,
}

#[derive(Clone)]
struct LegacyFacts {
    revision: McpProtocolRevision,
    client_capabilities: Map<String, Value>,
    client_info: Implementation,
}

fn decode_initialize(
    server: &McpServer,
    id: RequestId,
    params: Option<&Value>,
) -> Result<(LegacyFacts, Response), Response> {
    let Some(Value::Object(params)) = params else {
        return Err(standard_error(Some(id), ErrorCode::InvalidParams));
    };
    let Some(requested) = params.get("protocolVersion").and_then(Value::as_str) else {
        return Err(standard_error(Some(id), ErrorCode::InvalidParams));
    };
    let Some(Value::Object(client_capabilities)) = params.get("capabilities") else {
        return Err(standard_error(Some(id), ErrorCode::InvalidParams));
    };
    let revision = McpProtocolRevision::negotiate_legacy(requested);
    if !validate_initialize_metadata(params)
        || !validate_legacy_client_capabilities(revision, client_capabilities)
    {
        return Err(standard_error(Some(id), ErrorCode::InvalidParams));
    }
    let Some(client_info_value) = params.get("clientInfo") else {
        return Err(standard_error(Some(id), ErrorCode::InvalidParams));
    };
    let Some(client_info) = decode_legacy_implementation(revision, client_info_value) else {
        return Err(standard_error(Some(id), ErrorCode::InvalidParams));
    };
    let result = json!({
        "protocolVersion": revision.as_str(),
        "capabilities": server.capabilities(),
        "serverInfo": {
            "name": crate::MCP_SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION")
        }
    })
    .as_object()
    .expect("initialize result is an object")
    .clone();
    let response_id = id.clone();
    let response = ResultResponse::legacy(id, result)
        .map(Response::Result)
        .map_err(|_| standard_error(Some(response_id), ErrorCode::InternalError))?;
    Ok((
        LegacyFacts {
            revision,
            client_capabilities: client_capabilities.clone(),
            client_info,
        },
        response,
    ))
}

fn validate_initialize_metadata(params: &Map<String, Value>) -> bool {
    params.get("_meta").is_none_or(|value| {
        value
            .as_object()
            .is_some_and(validate_legacy_request_metadata)
    })
}

fn validate_legacy_client_capabilities(
    revision: McpProtocolRevision,
    capabilities: &Map<String, Value>,
) -> bool {
    map_values_are_objects(capabilities.get("experimental"))
        && validate_roots_capability(capabilities.get("roots"))
        && match revision {
            McpProtocolRevision::V2025_06_18 => {
                optional_object(capabilities.get("sampling"))
                    && optional_object(capabilities.get("elicitation"))
                    && capabilities.get("tasks").is_none()
            }
            McpProtocolRevision::V2025_11_25 => {
                object_options_are_objects(capabilities.get("sampling"), &["context", "tools"])
                    && object_options_are_objects(capabilities.get("elicitation"), &["form", "url"])
                    && validate_tasks_capability(capabilities.get("tasks"))
            }
            McpProtocolRevision::V2026_07_28 => false,
        }
}

fn validate_roots_capability(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_object()
            .is_some_and(|roots| roots.get("listChanged").is_none_or(Value::is_boolean))
    })
}

fn validate_tasks_capability(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_object().is_some_and(|tasks| {
            object_fields_are_objects(tasks, &["list", "cancel"])
                && tasks.get("requests").is_none_or(|value| {
                    value.as_object().is_some_and(|requests| {
                        requests.get("sampling").is_none_or(|value| {
                            value.as_object().is_some_and(|sampling| {
                                object_fields_are_objects(sampling, &["createMessage"])
                            })
                        }) && requests.get("elicitation").is_none_or(|value| {
                            value.as_object().is_some_and(|elicitation| {
                                object_fields_are_objects(elicitation, &["create"])
                            })
                        })
                    })
                })
        })
    })
}

fn object_options_are_objects(value: Option<&Value>, options: &[&str]) -> bool {
    value.is_none_or(|value| {
        value
            .as_object()
            .is_some_and(|object| object_fields_are_objects(object, options))
    })
}

fn object_fields_are_objects(object: &Map<String, Value>, fields: &[&str]) -> bool {
    fields
        .iter()
        .all(|field| object.get(*field).is_none_or(Value::is_object))
}

fn map_values_are_objects(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value
            .as_object()
            .is_some_and(|values| values.values().all(Value::is_object))
    })
}

fn optional_object(value: Option<&Value>) -> bool {
    value.is_none_or(Value::is_object)
}

fn decode_legacy_implementation(
    revision: McpProtocolRevision,
    value: &Value,
) -> Option<Implementation> {
    let object = value.as_object()?;
    if revision == McpProtocolRevision::V2025_06_18
        && ["description", "websiteUrl", "icons"]
            .iter()
            .any(|field| object.contains_key(*field))
    {
        return None;
    }
    decode_implementation(value)
}

fn decode_legacy_request(
    id: RequestId,
    method: String,
    params: Option<&Value>,
    facts: &LegacyFacts,
) -> Result<Request, Response> {
    let params = match params {
        None => Map::new(),
        Some(Value::Object(params)) => params.clone(),
        Some(_) => return Err(standard_error(Some(id), ErrorCode::InvalidParams)),
    };
    if let Some(metadata) = params.get("_meta") {
        let Some(metadata) = metadata.as_object() else {
            return Err(standard_error(Some(id), ErrorCode::InvalidParams));
        };
        if !validate_legacy_request_metadata(metadata) {
            return Err(standard_error(Some(id), ErrorCode::InvalidParams));
        }
    }
    Ok(Request::legacy(
        id,
        method,
        params,
        facts.revision.as_str(),
        facts.client_capabilities.clone(),
        facts.client_info.clone(),
    ))
}

fn is_legacy_operational_method(method: &str) -> bool {
    matches!(method, PING_METHOD | TOOLS_LIST_METHOD | TOOLS_CALL_METHOD)
}

fn server_not_initialized(id: RequestId) -> Response {
    standard_error(Some(id), ErrorCode::ServerNotInitialized)
}

fn legacy_result(id: RequestId, result: Map<String, Value>) -> Response {
    ResultResponse::legacy(id.clone(), result).map_or_else(
        |_| standard_error(Some(id), ErrorCode::InternalError),
        Response::Result,
    )
}

fn standard_error(id: Option<RequestId>, code: ErrorCode) -> Response {
    Response::Error(
        ErrorResponse::new(id, code).expect("closed session errors must satisfy response bounds"),
    )
}
