//! Transport-independent MCP discovery and dispatch.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::{Map, Value, json};

use crate::{
    DecodeOutcome, ErrorCode, ErrorResponse, InboundMessage, MAX_METHOD_NAME_BYTES, Notification,
    PROTOCOL_VERSION, Request, RequestId, Response, ResultResponse, decode_message,
    mcp::validate_client_capabilities,
};

/// Programmatic MCP server name advertised by discovery.
pub const MCP_SERVER_NAME: &str = "oneagent";

const DISCOVER_METHOD: &str = "server/discover";
const TOOLS_LIST_METHOD: &str = "tools/list";
const TOOLS_CALL_METHOD: &str = "tools/call";
const MAX_TOOL_DESCRIPTION_BYTES: usize = 1_024;
const MAX_TOOL_ERROR_CODE_BYTES: usize = 128;
const MAX_TOOL_ERROR_MESSAGE_BYTES: usize = 512;

/// Boxed borrowed future returned by an MCP tool-call handler.
pub type McpToolFuture<'a> = Pin<Box<dyn Future<Output = McpToolCallOutcome> + Send + 'a>>;

/// Advisory MCP annotations for one tool definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpToolAnnotations {
    /// Deterministic local read-only behavior.
    ReadOnly,
}

impl McpToolAnnotations {
    /// Returns the canonical annotation set for a deterministic local read-only tool.
    #[must_use]
    pub const fn read_only() -> Self {
        Self::ReadOnly
    }

    fn as_value(self) -> Value {
        match self {
            Self::ReadOnly => json!({
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false,
            }),
        }
    }
}

/// Validated immutable MCP tool definition.
#[derive(Debug, Clone, PartialEq)]
pub struct McpToolDefinition {
    name: String,
    description: String,
    input_schema: Map<String, Value>,
    annotations: McpToolAnnotations,
}

impl McpToolDefinition {
    /// Creates a validated tool definition with an object-root input schema.
    ///
    /// # Errors
    ///
    /// Returns [`McpToolDefinitionError`] for an invalid name, description, or schema.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Map<String, Value>,
        annotations: McpToolAnnotations,
    ) -> Result<Self, McpToolDefinitionError> {
        let name = name.into();
        let description = description.into();
        if name.is_empty()
            || name.len() > 128
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_-.".contains(&byte))
        {
            return Err(McpToolDefinitionError::InvalidName);
        }
        if description.trim().is_empty() || description.len() > MAX_TOOL_DESCRIPTION_BYTES {
            return Err(McpToolDefinitionError::InvalidDescription);
        }
        if input_schema.get("type") != Some(&Value::String("object".to_owned())) {
            return Err(McpToolDefinitionError::InvalidInputSchema);
        }
        let definition = Self {
            name,
            description,
            input_schema,
            annotations,
        };
        validate_tool_list(std::slice::from_ref(&definition))
            .map_err(|()| McpToolDefinitionError::InvalidInputSchema)?;
        Ok(definition)
    }

    /// Returns the stable tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    fn as_value(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema,
            "annotations": self.annotations.as_value(),
        })
    }
}

/// Closed validation failure for an MCP tool definition or catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpToolDefinitionError {
    /// The name is empty, too long, or outside the recommended ASCII vocabulary.
    InvalidName,
    /// The description is empty or exceeds the accepted bound.
    InvalidDescription,
    /// The input schema is not rooted at an object.
    InvalidInputSchema,
    /// The catalog contains duplicate tool names.
    DuplicateName,
    /// The catalog is empty.
    EmptyCatalog,
    /// The catalog cannot be registered by the closed server builder.
    InvalidCatalog,
}

/// Closed semantic outcome for one known MCP tool invocation.
#[derive(Debug, Clone, PartialEq)]
pub enum McpToolCallOutcome {
    /// The tool completed with one structured JSON value.
    Success(Value),
    /// The known tool failed with a stable bounded code and message.
    Error { code: String, message: String },
    /// The handler boundary failed and must become a protocol internal error.
    Internal,
}

impl McpToolCallOutcome {
    /// Creates a bounded known-tool error, or [`Self::Internal`] for invalid diagnostics.
    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        let code = code.into();
        let message = message.into();
        if code.is_empty()
            || code.len() > MAX_TOOL_ERROR_CODE_BYTES
            || message.trim().is_empty()
            || message.len() > MAX_TOOL_ERROR_MESSAGE_BYTES
        {
            return Self::Internal;
        }
        Self::Error { code, message }
    }
}

/// Runtime-supplied asynchronous executor for known MCP tool calls.
pub trait McpToolCallHandler: Send + Sync {
    /// Executes one validated known tool name and object argument.
    fn call<'a>(&'a self, name: &'a str, arguments: &'a Map<String, Value>) -> McpToolFuture<'a>;
}

/// The stateless, transport-independent `OneAgent` MCP server.
pub struct McpServer {
    methods: BTreeMap<String, Registration>,
    capabilities: Map<String, Value>,
}

impl McpServer {
    /// Creates the truthful discovery-only Sprint 28 server.
    #[must_use]
    pub fn new() -> Self {
        ServerBuilder::new(Map::new()).build()
    }

    /// Creates a server with one immutable validated tool catalog.
    ///
    /// # Errors
    ///
    /// Returns [`McpToolDefinitionError`] when the catalog is empty or duplicated.
    pub fn with_tools(
        tools: Vec<McpToolDefinition>,
        handler: impl McpToolCallHandler + 'static,
    ) -> Result<Self, McpToolDefinitionError> {
        if tools.is_empty() {
            return Err(McpToolDefinitionError::EmptyCatalog);
        }
        let mut tools = tools;
        tools.sort_by(|left, right| left.name.cmp(&right.name));
        if tools.windows(2).any(|pair| pair[0].name == pair[1].name) {
            return Err(McpToolDefinitionError::DuplicateName);
        }
        validate_tool_list(&tools).map_err(|()| McpToolDefinitionError::InvalidCatalog)?;
        let mut capabilities = Map::new();
        capabilities.insert("tools".to_owned(), Value::Object(Map::new()));
        let tools: Arc<[McpToolDefinition]> = tools.into();
        let names = tools.iter().map(|tool| tool.name.clone()).collect();
        let handler: Arc<dyn McpToolCallHandler> = Arc::new(handler);
        let mut builder = ServerBuilder::new(capabilities);
        builder
            .register(
                TOOLS_LIST_METHOD,
                MethodMode::RequestOnly,
                Map::new(),
                ToolsListHandler {
                    tools: Arc::clone(&tools),
                },
            )
            .map_err(|_| McpToolDefinitionError::InvalidCatalog)?;
        builder
            .register(
                TOOLS_CALL_METHOD,
                MethodMode::RequestOnly,
                Map::new(),
                ToolsCallHandler { names, handler },
            )
            .map_err(|_| McpToolDefinitionError::InvalidCatalog)?;
        Ok(builder.build())
    }

    /// Decodes and dispatches one complete UTF-8 JSON frame.
    ///
    /// Returns `None` for every notification path. The method performs no I/O,
    /// starts no task, and retains no request state.
    #[must_use]
    pub async fn dispatch(&self, input: &str) -> Option<Response> {
        match decode_message(input) {
            DecodeOutcome::Error(error) => Some(Response::Error(error)),
            DecodeOutcome::IgnoredNotification => None,
            DecodeOutcome::Message(InboundMessage::Request(request)) => {
                Some(self.dispatch_request(&request).await)
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

    async fn dispatch_request(&self, request: &Request) -> Response {
        self.dispatch_request_with_profile(request, McpResponseProfile::Modern)
            .await
    }

    pub(crate) async fn dispatch_request_with_profile(
        &self,
        request: &Request,
        profile: McpResponseProfile,
    ) -> Response {
        let Some(registration) = self.methods.get(request.method()) else {
            return standard_error(Some(request.id().clone()), ErrorCode::MethodNotFound);
        };
        if !registration.mode.accepts_requests() {
            return standard_error(Some(request.id().clone()), ErrorCode::MethodNotFound);
        }
        if !has_required_capabilities(
            request.metadata().client_capabilities(),
            &registration.required_capabilities,
        ) {
            return ErrorResponse::missing_capability(
                request.id().clone(),
                &registration.required_capabilities,
            )
            .map_or_else(|_| internal_error(request.id().clone()), Response::Error);
        }

        match registration.handler.handle_request(request).await {
            Ok(mut result) => {
                if profile == McpResponseProfile::Modern && request.method() == TOOLS_LIST_METHOD {
                    result.insert("ttlMs".to_owned(), json!(0));
                    result.insert("cacheScope".to_owned(), json!("public"));
                }
                profile
                    .response(request.id().clone(), result)
                    .map_or_else(|_| internal_error(request.id().clone()), Response::Result)
            }
            Err(HandlerFailure::InvalidParams) => {
                standard_error(Some(request.id().clone()), ErrorCode::InvalidParams)
            }
            Err(HandlerFailure::Internal) => internal_error(request.id().clone()),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum McpResponseProfile {
    Legacy,
    Modern,
}

impl McpResponseProfile {
    fn response(
        self,
        id: RequestId,
        fields: Map<String, Value>,
    ) -> Result<ResultResponse, crate::EncodeError> {
        match self {
            Self::Legacy => ResultResponse::legacy(id, fields),
            Self::Modern => ResultResponse::complete(id, fields),
        }
    }
}

fn validate_tool_list(tools: &[McpToolDefinition]) -> Result<(), ()> {
    let fields = json!({
        "tools": tools.iter().map(McpToolDefinition::as_value).collect::<Vec<_>>(),
        "ttlMs": 0,
        "cacheScope": "public"
    })
    .as_object()
    .expect("tools/list result is an object")
    .clone();
    ResultResponse::complete(RequestId::unsigned(0), fields)
        .map(|_| ())
        .map_err(|_| ())
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
    InvalidParams,
    Internal,
}

type MethodFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Map<String, Value>, HandlerFailure>> + Send + 'a>>;

trait MethodHandler: Send + Sync {
    fn handle_request<'a>(&'a self, _request: &'a Request) -> MethodFuture<'a> {
        Box::pin(async { Err(HandlerFailure::Internal) })
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
    capabilities: Map<String, Value>,
}

impl ServerBuilder {
    fn new(capabilities: Map<String, Value>) -> Self {
        let mut builder = Self {
            methods: BTreeMap::new(),
            capabilities: capabilities.clone(),
        };
        builder
            .register(
                DISCOVER_METHOD,
                MethodMode::RequestOnly,
                Map::new(),
                DiscoverHandler { capabilities },
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
        if !validate_client_capabilities(&required_capabilities) {
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
            capabilities: self.capabilities,
        }
    }
}

struct DiscoverHandler {
    capabilities: Map<String, Value>,
}

impl MethodHandler for DiscoverHandler {
    fn handle_request<'a>(&'a self, _request: &'a Request) -> MethodFuture<'a> {
        Box::pin(async move {
            let mut result = Map::new();
            result.insert("supportedVersions".to_owned(), json!([PROTOCOL_VERSION]));
            result.insert(
                "capabilities".to_owned(),
                Value::Object(self.capabilities.clone()),
            );
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
        })
    }
}

struct ToolsListHandler {
    tools: Arc<[McpToolDefinition]>,
}

impl MethodHandler for ToolsListHandler {
    fn handle_request<'a>(&'a self, request: &'a Request) -> MethodFuture<'a> {
        Box::pin(async move {
            if request.params().keys().any(|key| key != "_meta") {
                return Err(HandlerFailure::InvalidParams);
            }
            Ok(json!({
                "tools": self.tools.iter().map(McpToolDefinition::as_value).collect::<Vec<_>>()
            })
            .as_object()
            .expect("tools/list result is an object")
            .clone())
        })
    }
}

struct ToolsCallHandler {
    names: BTreeSet<String>,
    handler: Arc<dyn McpToolCallHandler>,
}

impl MethodHandler for ToolsCallHandler {
    fn handle_request<'a>(&'a self, request: &'a Request) -> MethodFuture<'a> {
        Box::pin(async move {
            if request
                .params()
                .keys()
                .any(|key| !matches!(key.as_str(), "_meta" | "name" | "arguments"))
            {
                return Err(HandlerFailure::InvalidParams);
            }
            let Some(name) = request.params().get("name").and_then(Value::as_str) else {
                return Err(HandlerFailure::InvalidParams);
            };
            if !self.names.contains(name) {
                return Err(HandlerFailure::InvalidParams);
            }
            let empty = Map::new();
            let arguments = match request.params().get("arguments") {
                Some(Value::Object(arguments)) => arguments,
                Some(_) => return Err(HandlerFailure::InvalidParams),
                None => &empty,
            };
            let (structured, is_error) = match self.handler.call(name, arguments).await {
                McpToolCallOutcome::Success(value) => (value, false),
                McpToolCallOutcome::Error { code, message } => {
                    (json!({"code": code, "message": message}), true)
                }
                McpToolCallOutcome::Internal => return Err(HandlerFailure::Internal),
            };
            let text = serde_json::to_string(&structured).map_err(|_| HandlerFailure::Internal)?;
            let mut result = json!({
                "content": [{"type": "text", "text": text}],
                "structuredContent": structured,
            })
            .as_object()
            .expect("tools/call result is an object")
            .clone();
            if is_error {
                result.insert("isError".to_owned(), Value::Bool(true));
            }
            Ok(result)
        })
    }
}

fn has_required_capabilities(declared: &Map<String, Value>, required: &Map<String, Value>) -> bool {
    required
        .iter()
        .all(|(name, value)| declared.get(name) == Some(value))
}

fn standard_error(id: Option<crate::RequestId>, code: ErrorCode) -> Response {
    Response::Error(
        ErrorResponse::new(id, code).expect("standard MCP errors must satisfy outbound bounds"),
    )
}

fn internal_error(id: crate::RequestId) -> Response {
    standard_error(Some(id), ErrorCode::InternalError)
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll, Waker};

    use serde_json::{Map, Value, json};

    use crate::{ErrorCode, MAX_MESSAGE_BYTES, Response, encode_response};

    use super::{
        DISCOVER_METHOD, HandlerFailure, MethodFuture, MethodHandler, MethodMode, Notification,
        RegistrationError, Request, ServerBuilder,
    };

    fn block_on<F: Future>(future: F) -> F::Output {
        let mut future = pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

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
        fn handle_request<'a>(&'a self, request: &'a Request) -> MethodFuture<'a> {
            Box::pin(async move {
                let Some(value) = request.params().get("extra") else {
                    return Err(HandlerFailure::InvalidParams);
                };
                let mut result = Map::new();
                result.insert("echo".to_owned(), value.clone());
                Ok(result)
            })
        }
    }

    struct FailingHandler;

    impl MethodHandler for FailingHandler {}

    struct OversizedHandler;

    impl MethodHandler for OversizedHandler {
        fn handle_request<'a>(&'a self, _request: &'a Request) -> MethodFuture<'a> {
            Box::pin(async {
                let mut result = Map::new();
                result.insert("oversized".to_owned(), json!("x".repeat(MAX_MESSAGE_BYTES)));
                Ok(result)
            })
        }
    }

    struct CountingNotificationHandler(Arc<AtomicUsize>);

    impl MethodHandler for CountingNotificationHandler {
        fn handle_notification(&self, _notification: &Notification) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn registration_rejects_invalid_duplicate_and_reserved_methods() {
        let mut builder = ServerBuilder::new(Map::new());
        assert_eq!(
            builder.register("", MethodMode::RequestOnly, Map::new(), EchoHandler),
            Err(RegistrationError::Invalid)
        );
        assert_eq!(
            builder.register(
                "invalid/capability",
                MethodMode::RequestOnly,
                json!({"elicitation": 42})
                    .as_object()
                    .expect("capability object")
                    .clone(),
                EchoHandler
            ),
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
        let mut first = ServerBuilder::new(Map::new());
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
                "z/oversized",
                MethodMode::RequestOnly,
                Map::new(),
                OversizedHandler,
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
            ["a/echo", DISCOVER_METHOD, "z/fail", "z/oversized"]
        );

        let missing = block_on(server.dispatch(&request("a/echo", &json!({}), &json!("value"))))
            .expect("request must respond");
        let Response::Error(missing) = missing else {
            panic!("missing capability must fail");
        };
        assert_eq!(missing.code(), ErrorCode::MissingRequiredClientCapability);
        assert_eq!(
            missing.data(),
            Some(&json!({ "requiredCapabilities": required }))
        );

        let accepted = block_on(server.dispatch(&request(
            "a/echo",
            &json!({ "elicitation": {} }),
            &json!("value"),
        )))
        .expect("request must respond");
        let encoded = encode_response(&accepted).expect("result must encode");
        assert_eq!(
            serde_json::from_slice::<Value>(&encoded).expect("result JSON")["result"]["echo"],
            "value"
        );

        let failed = block_on(server.dispatch(&request("z/fail", &json!({}), &Value::Null)))
            .expect("request must respond");
        let Response::Error(failed) = failed else {
            panic!("handler failure must be closed");
        };
        assert_eq!(failed.code(), ErrorCode::InternalError);

        let oversized =
            block_on(server.dispatch(&request("z/oversized", &json!({}), &Value::Null)))
                .expect("request must respond");
        let Response::Error(oversized) = oversized else {
            panic!("oversized handler result must become a closed error");
        };
        assert_eq!(oversized.code(), ErrorCode::InternalError);
        assert!(encode_response(&Response::Error(oversized)).is_ok());
    }

    #[test]
    fn notification_modes_never_create_responses_or_call_request_only_handlers() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut builder = ServerBuilder::new(Map::new());
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

        assert!(block_on(server.dispatch(r#"{"jsonrpc":"2.0","method":"test/notice"}"#)).is_none());
        assert!(
            block_on(server.dispatch(r#"{"jsonrpc":"2.0","method":"test/request"}"#)).is_none()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let response = block_on(server.dispatch(&request("test/notice", &json!({}), &Value::Null)))
            .expect("request to notification-only method must respond");
        let Response::Error(error) = response else {
            panic!("notification-only request must fail");
        };
        assert_eq!(error.code(), ErrorCode::MethodNotFound);
    }
}
