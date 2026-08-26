//! Transport-independent MCP discovery and dispatch.

use std::collections::BTreeMap;
use std::fmt;

use serde_json::{Map, Value, json};

use crate::{
    DecodeOutcome, ErrorCode, ErrorResponse, InboundMessage, MAX_METHOD_NAME_BYTES, Notification,
    PROTOCOL_VERSION, Request, Response, ResultResponse, decode_message,
};

/// Programmatic MCP server name advertised by discovery.
pub const MCP_SERVER_NAME: &str = "oneagent";

const DISCOVER_METHOD: &str = "server/discover";

/// The stateless, transport-independent `OneAgent` MCP server.
pub struct McpServer {
    methods: BTreeMap<String, Registration>,
    capabilities: Map<String, Value>,
}

impl McpServer {
    /// Creates the truthful discovery-only Sprint 28 server.
    #[must_use]
    pub fn new() -> Self {
        ServerBuilder::new().build()
    }

    /// Decodes and dispatches one complete UTF-8 JSON frame.
    ///
    /// Returns `None` for every notification path. The method performs no I/O,
    /// starts no task, and retains no request state.
    #[must_use]
    pub fn dispatch(&self, input: &str) -> Option<Response> {
        match decode_message(input) {
            DecodeOutcome::Error(error) => Some(Response::Error(error)),
            DecodeOutcome::IgnoredNotification => None,
            DecodeOutcome::Message(InboundMessage::Request(request)) => {
                Some(self.dispatch_request(&request))
            }
            DecodeOutcome::Message(InboundMessage::Notification(notification)) => {
                self.dispatch_notification(&notification);
                None
            }
        }
    }

    /// Returns the exact advertised server capability object.
    #[must_use]
    pub const fn capabilities(&self) -> &Map<String, Value> {
        &self.capabilities
    }

    /// Returns registered method names in canonical order.
    pub fn registered_methods(&self) -> impl ExactSizeIterator<Item = &str> {
        self.methods.keys().map(String::as_str)
    }

    fn dispatch_request(&self, request: &Request) -> Response {
        let Some(registration) = self.methods.get(request.method()) else {
            return Response::Error(ErrorResponse::new(
                Some(request.id().clone()),
                ErrorCode::MethodNotFound,
            ));
        };
        if !registration.mode.accepts_requests() {
            return Response::Error(ErrorResponse::new(
                Some(request.id().clone()),
                ErrorCode::MethodNotFound,
            ));
        }
        if !has_required_capabilities(
            request.metadata().client_capabilities(),
            &registration.required_capabilities,
        ) {
            return Response::Error(ErrorResponse::missing_capability(
                request.id().clone(),
                &registration.required_capabilities,
            ));
        }

        match registration.handler.handle_request(request) {
            Ok(result) => Response::Result(ResultResponse::complete(request.id().clone(), result)),
            #[cfg(test)]
            Err(HandlerFailure::InvalidParams) => Response::Error(ErrorResponse::new(
                Some(request.id().clone()),
                ErrorCode::InvalidParams,
            )),
            Err(HandlerFailure::Internal) => Response::Error(ErrorResponse::new(
                Some(request.id().clone()),
                ErrorCode::InternalError,
            )),
        }
    }

    fn dispatch_notification(&self, notification: &Notification) {
        let Some(registration) = self.methods.get(notification.method()) else {
            return;
        };
        if registration.mode.accepts_notifications() {
            registration.handler.handle_notification(notification);
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for McpServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("McpServer")
            .field("method_count", &self.methods.len())
            .field("capability_count", &self.capabilities.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodMode {
    RequestOnly,
    #[cfg(test)]
    NotificationOnly,
    #[cfg(test)]
    Both,
}

impl MethodMode {
    const fn accepts_requests(self) -> bool {
        match self {
            Self::RequestOnly => true,
            #[cfg(test)]
            Self::Both => true,
            #[cfg(test)]
            Self::NotificationOnly => false,
        }
    }

    const fn accepts_notifications(self) -> bool {
        match self {
            Self::RequestOnly => false,
            #[cfg(test)]
            Self::NotificationOnly | Self::Both => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandlerFailure {
    #[cfg(test)]
    InvalidParams,
    Internal,
}

trait MethodHandler: Send + Sync {
    fn handle_request(&self, _request: &Request) -> Result<Map<String, Value>, HandlerFailure> {
        Err(HandlerFailure::Internal)
    }

    fn handle_notification(&self, _notification: &Notification) {}
}

struct Registration {
    mode: MethodMode,
    required_capabilities: Map<String, Value>,
    handler: Box<dyn MethodHandler>,
}

impl fmt::Debug for Registration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Registration")
            .field("mode", &self.mode)
            .field(
                "required_capability_count",
                &self.required_capabilities.len(),
            )
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistrationError {
    Invalid,
    Duplicate,
    Reserved,
}

#[derive(Debug)]
struct ServerBuilder {
    methods: BTreeMap<String, Registration>,
}

impl ServerBuilder {
    fn new() -> Self {
        let mut builder = Self {
            methods: BTreeMap::new(),
        };
        builder
            .register(
                DISCOVER_METHOD,
                MethodMode::RequestOnly,
                Map::new(),
                DiscoverHandler,
            )
            .expect("the built-in discovery registration must be valid");
        builder
    }

    fn register(
        &mut self,
        method: &str,
        mode: MethodMode,
        required_capabilities: Map<String, Value>,
        handler: impl MethodHandler + 'static,
    ) -> Result<(), RegistrationError> {
        if method.is_empty() || method.len() > MAX_METHOD_NAME_BYTES {
            return Err(RegistrationError::Invalid);
        }
        if method == DISCOVER_METHOD && self.methods.contains_key(method) {
            return Err(RegistrationError::Reserved);
        }
        if self.methods.contains_key(method) {
            return Err(RegistrationError::Duplicate);
        }
        self.methods.insert(
            method.to_owned(),
            Registration {
                mode,
                required_capabilities,
                handler: Box::new(handler),
            },
        );
        Ok(())
    }

    fn build(self) -> McpServer {
        McpServer {
            methods: self.methods,
            capabilities: Map::new(),
        }
    }
}

struct DiscoverHandler;

impl MethodHandler for DiscoverHandler {
    fn handle_request(&self, _request: &Request) -> Result<Map<String, Value>, HandlerFailure> {
        let mut result = Map::new();
        result.insert("supportedVersions".to_owned(), json!([PROTOCOL_VERSION]));
        result.insert("capabilities".to_owned(), Value::Object(Map::new()));
        result.insert(
            "_meta".to_owned(),
            json!({
                "io.modelcontextprotocol/serverInfo": {
                    "name": MCP_SERVER_NAME,
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        );
        result.insert("ttlMs".to_owned(), json!(0));
        result.insert("cacheScope".to_owned(), json!("public"));
        Ok(result)
    }
}

fn has_required_capabilities(declared: &Map<String, Value>, required: &Map<String, Value>) -> bool {
    required
        .iter()
        .all(|(name, value)| declared.get(name) == Some(value))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use serde_json::{Map, Value, json};

    use crate::{ErrorCode, Response, encode_response};

    use super::{
        DISCOVER_METHOD, HandlerFailure, MethodHandler, MethodMode, Notification,
        RegistrationError, Request, ServerBuilder,
    };

    fn request(method: &str, capabilities: &Value, extra: &Value) -> String {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": {
                "_meta": {
                    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                    "io.modelcontextprotocol/clientCapabilities": capabilities
                },
                "extra": extra
            }
        })
        .to_string()
    }

    struct EchoHandler;

    impl MethodHandler for EchoHandler {
        fn handle_request(&self, request: &Request) -> Result<Map<String, Value>, HandlerFailure> {
            let Some(value) = request.params().get("extra") else {
                return Err(HandlerFailure::InvalidParams);
            };
            let mut result = Map::new();
            result.insert("echo".to_owned(), value.clone());
            Ok(result)
        }
    }

    struct FailingHandler;

    impl MethodHandler for FailingHandler {}

    struct CountingNotificationHandler(Arc<AtomicUsize>);

    impl MethodHandler for CountingNotificationHandler {
        fn handle_notification(&self, _notification: &Notification) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn registration_rejects_invalid_duplicate_and_reserved_methods() {
        let mut builder = ServerBuilder::new();
        assert_eq!(
            builder.register("", MethodMode::RequestOnly, Map::new(), EchoHandler),
            Err(RegistrationError::Invalid)
        );
        assert_eq!(
            builder.register(
                DISCOVER_METHOD,
                MethodMode::RequestOnly,
                Map::new(),
                EchoHandler
            ),
            Err(RegistrationError::Reserved)
        );
        builder
            .register("test/echo", MethodMode::Both, Map::new(), EchoHandler)
            .expect("first registration must succeed");
        assert_eq!(
            builder.register(
                "test/echo",
                MethodMode::RequestOnly,
                Map::new(),
                EchoHandler
            ),
            Err(RegistrationError::Duplicate)
        );
    }

    #[test]
    fn registry_order_capability_and_handler_failures_are_deterministic() {
        let mut first = ServerBuilder::new();
        let required = json!({ "elicitation": {} })
            .as_object()
            .expect("object")
            .clone();
        first
            .register(
                "z/fail",
                MethodMode::RequestOnly,
                Map::new(),
                FailingHandler,
            )
            .expect("registration must succeed");
        first
            .register(
                "a/echo",
                MethodMode::RequestOnly,
                required.clone(),
                EchoHandler,
            )
            .expect("registration must succeed");
        let server = first.build();
        assert_eq!(
            server.registered_methods().collect::<Vec<_>>(),
            ["a/echo", DISCOVER_METHOD, "z/fail"]
        );

        let missing = server
            .dispatch(&request("a/echo", &json!({}), &json!("value")))
            .expect("request must respond");
        let Response::Error(missing) = missing else {
            panic!("missing capability must fail");
        };
        assert_eq!(missing.code(), ErrorCode::MissingRequiredClientCapability);
        assert_eq!(
            missing.data(),
            Some(&json!({ "requiredCapabilities": required }))
        );

        let accepted = server
            .dispatch(&request(
                "a/echo",
                &json!({ "elicitation": {} }),
                &json!("value"),
            ))
            .expect("request must respond");
        let encoded = encode_response(&accepted).expect("result must encode");
        assert_eq!(
            serde_json::from_slice::<Value>(&encoded).expect("result JSON")["result"]["echo"],
            "value"
        );

        let failed = server
            .dispatch(&request("z/fail", &json!({}), &Value::Null))
            .expect("request must respond");
        let Response::Error(failed) = failed else {
            panic!("handler failure must be closed");
        };
        assert_eq!(failed.code(), ErrorCode::InternalError);
    }

    #[test]
    fn notification_modes_never_create_responses_or_call_request_only_handlers() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut builder = ServerBuilder::new();
        builder
            .register(
                "test/notice",
                MethodMode::NotificationOnly,
                Map::new(),
                CountingNotificationHandler(Arc::clone(&calls)),
            )
            .expect("registration must succeed");
        builder
            .register(
                "test/request",
                MethodMode::RequestOnly,
                Map::new(),
                EchoHandler,
            )
            .expect("registration must succeed");
        let server = builder.build();

        assert!(
            server
                .dispatch(r#"{"jsonrpc":"2.0","method":"test/notice"}"#)
                .is_none()
        );
        assert!(
            server
                .dispatch(r#"{"jsonrpc":"2.0","method":"test/request"}"#)
                .is_none()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let response = server
            .dispatch(&request("test/notice", &json!({}), &Value::Null))
            .expect("request to notification-only method must respond");
        let Response::Error(error) = response else {
            panic!("notification-only request must fail");
        };
        assert_eq!(error.code(), ErrorCode::MethodNotFound);
    }
}
