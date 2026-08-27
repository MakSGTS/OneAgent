//! Bounded transport-independent Language Server Protocol 3.17 core.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use serde_json::{Map, Number, Value, json};

use crate::mcp::{ParseFailure, parse_unique_value, value_within_nesting_bound};
use crate::{MAX_MESSAGE_BYTES, MAX_METHOD_NAME_BYTES, RequestId};

/// Programmatic LSP server name advertised during initialization.
pub const LSP_SERVER_NAME: &str = "oneagent";

const INITIALIZE: &str = "initialize";
const INITIALIZED: &str = "initialized";
const SHUTDOWN: &str = "shutdown";
const EXIT: &str = "exit";
const WORKSPACE_SYMBOL: &str = "workspace/symbol";
const DOCUMENT_DIAGNOSTIC: &str = "textDocument/diagnostic";
const MAX_QUERY_BYTES: usize = 256;
const MAX_RESULT_ITEMS: usize = 100;
const LSP_INTEGER_MIN: i64 = i32::MIN as i64;
const LSP_INTEGER_MAX: i64 = i32::MAX as i64;
const LSP_UINTEGER_MAX: u64 = i32::MAX as u64;

/// Optional semantic capabilities installed in one LSP server instance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LspCapabilities {
    workspace_symbols: bool,
    diagnostics: bool,
}

impl LspCapabilities {
    /// Creates lifecycle-only capabilities.
    #[must_use]
    pub const fn lifecycle_only() -> Self {
        Self {
            workspace_symbols: false,
            diagnostics: false,
        }
    }

    /// Adds the bounded workspace-symbol capability.
    #[must_use]
    pub const fn with_workspace_symbols(mut self) -> Self {
        self.workspace_symbols = true;
        self
    }

    /// Adds the bounded pull-diagnostic capability.
    #[must_use]
    pub const fn with_diagnostics(mut self) -> Self {
        self.diagnostics = true;
        self
    }

    /// Returns whether workspace symbols are installed.
    #[must_use]
    pub const fn workspace_symbols(self) -> bool {
        self.workspace_symbols
    }

    /// Returns whether pull diagnostics are installed.
    #[must_use]
    pub const fn diagnostics(self) -> bool {
        self.diagnostics
    }
}

/// Closed failure returned by a Runtime-supplied LSP handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspHandlerError {
    /// The validated wire request is incompatible with Runtime state.
    InvalidParams,
    /// The request cannot complete within the accepted semantic bounds.
    RequestFailed,
    /// The handler violated its internal contract.
    Internal,
}

/// Runtime-supplied semantic boundary for accepted LSP methods.
pub trait LspHandler: Send + Sync {
    /// Validates initialize roots and Runtime compatibility.
    ///
    /// # Errors
    ///
    /// Returns a closed handler error when the initialization request is incompatible.
    fn validate_initialize(&self, params: &Map<String, Value>) -> Result<(), LspHandlerError>;

    /// Projects one validated workspace-symbol query.
    ///
    /// # Errors
    ///
    /// Returns a closed handler error when the bounded projection cannot complete.
    fn workspace_symbols(&self, query: &str) -> Result<Value, LspHandlerError>;

    /// Projects one validated document diagnostic URI.
    ///
    /// # Errors
    ///
    /// Returns a closed handler error when the bounded projection cannot complete.
    fn document_diagnostics(&self, uri: &str) -> Result<Value, LspHandlerError>;
}

/// A response exceeded the accepted JSON encoding bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspEncodeError;

/// Stable LSP/JSON-RPC error vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspErrorCode {
    /// Invalid JSON.
    ParseError,
    /// Invalid JSON-RPC envelope or lifecycle request.
    InvalidRequest,
    /// Unknown request method.
    MethodNotFound,
    /// Invalid method parameters.
    InvalidParams,
    /// Internal handler or encoding failure.
    InternalError,
    /// Request arrived before initialization completed.
    ServerNotInitialized,
    /// A valid request cannot complete within accepted bounds.
    RequestFailed,
}

impl LspErrorCode {
    /// Returns the standard numeric code.
    #[must_use]
    pub const fn value(self) -> i64 {
        match self {
            Self::ParseError => -32_700,
            Self::InvalidRequest => -32_600,
            Self::MethodNotFound => -32_601,
            Self::InvalidParams => -32_602,
            Self::InternalError => -32_603,
            Self::ServerNotInitialized => -32_002,
            Self::RequestFailed => -32_803,
        }
    }

    /// Returns the stable redacted message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::ParseError => "Parse error",
            Self::InvalidRequest => "Invalid Request",
            Self::MethodNotFound => "Method not found",
            Self::InvalidParams => "Invalid params",
            Self::InternalError => "Internal error",
            Self::ServerNotInitialized => "Server not initialized",
            Self::RequestFailed => "Request failed",
        }
    }
}

/// One bounded outbound LSP response.
#[derive(Debug, Clone, PartialEq)]
pub struct LspResponse {
    id: Option<RequestId>,
    body: LspResponseBody,
}

#[derive(Debug, Clone, PartialEq)]
enum LspResponseBody {
    Result(Value),
    Error(LspErrorCode),
}

impl LspResponse {
    fn result(id: RequestId, value: Value) -> Result<Self, LspEncodeError> {
        let response = Self {
            id: Some(id),
            body: LspResponseBody::Result(value),
        };
        encode_lsp_response(&response).map(|_| response)
    }

    fn error(id: Option<RequestId>, code: LspErrorCode) -> Self {
        Self {
            id,
            body: LspResponseBody::Error(code),
        }
    }

    /// Returns the echoed request identifier, when known.
    #[must_use]
    pub const fn id(&self) -> Option<&RequestId> {
        self.id.as_ref()
    }

    /// Returns the successful result, when present.
    #[must_use]
    pub const fn result_value(&self) -> Option<&Value> {
        match &self.body {
            LspResponseBody::Result(value) => Some(value),
            LspResponseBody::Error(_) => None,
        }
    }

    /// Returns the closed error code, when present.
    #[must_use]
    pub const fn error_code(&self) -> Option<LspErrorCode> {
        match self.body {
            LspResponseBody::Result(_) => None,
            LspResponseBody::Error(code) => Some(code),
        }
    }
}

/// Terminal process status selected by an `exit` notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspExitStatus {
    /// `exit` followed a successful `shutdown` response.
    Success,
    /// `exit` arrived before successful shutdown.
    Failure,
}

/// Outcome of dispatching one complete UTF-8 LSP JSON body.
#[derive(Debug, Clone, PartialEq)]
pub enum LspDispatchOutcome {
    /// One response must be framed and written.
    Response(LspResponse),
    /// A notification or ignored path writes nothing.
    NoResponse,
    /// The transport must terminate with the selected status.
    Exit(LspExitStatus),
}

/// Stateful transport-independent LSP 3.17 server.
pub struct LspServer {
    state: LspState,
    capabilities: LspCapabilities,
    handler: Arc<dyn LspHandler>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LspState {
    Uninitialized,
    AwaitingInitialized,
    Running,
    Shutdown,
    Exited,
}

impl LspServer {
    /// Creates a lifecycle-only server.
    #[must_use]
    pub fn new(handler: impl LspHandler + 'static) -> Self {
        Self::with_capabilities(LspCapabilities::lifecycle_only(), handler)
    }

    /// Creates a server with the exact installed semantic capabilities.
    #[must_use]
    pub fn with_capabilities(
        capabilities: LspCapabilities,
        handler: impl LspHandler + 'static,
    ) -> Self {
        Self {
            state: LspState::Uninitialized,
            capabilities,
            handler: Arc::new(handler),
        }
    }

    /// Returns the exact static server capabilities for this instance.
    #[must_use]
    pub fn capabilities(&self) -> Map<String, Value> {
        capability_value(self.capabilities)
    }

    /// Decodes and dispatches one complete UTF-8 JSON body.
    #[must_use]
    pub fn dispatch(&mut self, input: &str) -> LspDispatchOutcome {
        if input.len() > MAX_MESSAGE_BYTES {
            return response_error(None, LspErrorCode::InvalidRequest);
        }
        let value = match parse_unique_value(input) {
            Ok(value) => value,
            Err(ParseFailure::Syntax) => {
                return response_error(None, LspErrorCode::ParseError);
            }
            Err(ParseFailure::Duplicate | ParseFailure::Depth) => {
                return response_error(None, LspErrorCode::InvalidRequest);
            }
        };
        let Value::Object(object) = value else {
            return response_error(None, LspErrorCode::InvalidRequest);
        };
        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return response_error(None, LspErrorCode::InvalidRequest);
        }

        let id = match object.get("id") {
            Some(value) if valid_lsp_request_id(value) => match RequestId::from_value(value) {
                Some(id) => Some(id),
                None => return response_error(None, LspErrorCode::InvalidRequest),
            },
            Some(_) => return response_error(None, LspErrorCode::InvalidRequest),
            None => None,
        };
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            return response_error(id, LspErrorCode::InvalidRequest);
        };
        if method.is_empty() || method.len() > MAX_METHOD_NAME_BYTES {
            return response_error(id, LspErrorCode::InvalidRequest);
        }

        match id {
            Some(id) => self.dispatch_request(id, method, object.get("params")),
            None => self.dispatch_notification(method, object.get("params")),
        }
    }

    fn dispatch_request(
        &mut self,
        id: RequestId,
        method: &str,
        params: Option<&Value>,
    ) -> LspDispatchOutcome {
        match self.state {
            LspState::Uninitialized => {
                if method != INITIALIZE {
                    return response_error(Some(id), LspErrorCode::ServerNotInitialized);
                }
                let Some(params) = object_params(params) else {
                    return response_error(Some(id), LspErrorCode::InvalidParams);
                };
                if !validate_initialize_params(params) {
                    return response_error(Some(id), LspErrorCode::InvalidParams);
                }
                if let Err(error) = self.handler.validate_initialize(params) {
                    return response_error(Some(id), handler_error(error));
                }
                let result = json!({
                    "capabilities": capability_value(self.capabilities),
                    "serverInfo": {
                        "name": LSP_SERVER_NAME,
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                match LspResponse::result(id.clone(), result) {
                    Ok(response) => {
                        self.state = LspState::AwaitingInitialized;
                        LspDispatchOutcome::Response(response)
                    }
                    Err(LspEncodeError) => response_error(Some(id), LspErrorCode::InternalError),
                }
            }
            LspState::AwaitingInitialized => {
                response_error(Some(id), LspErrorCode::ServerNotInitialized)
            }
            LspState::Running => self.dispatch_running_request(id, method, params),
            LspState::Shutdown | LspState::Exited => {
                response_error(Some(id), LspErrorCode::InvalidRequest)
            }
        }
    }

    fn dispatch_running_request(
        &mut self,
        id: RequestId,
        method: &str,
        params: Option<&Value>,
    ) -> LspDispatchOutcome {
        match method {
            INITIALIZE => response_error(Some(id), LspErrorCode::InvalidRequest),
            SHUTDOWN => {
                if params.is_some() {
                    return response_error(Some(id), LspErrorCode::InvalidParams);
                }
                match LspResponse::result(id.clone(), Value::Null) {
                    Ok(response) => {
                        self.state = LspState::Shutdown;
                        LspDispatchOutcome::Response(response)
                    }
                    Err(LspEncodeError) => response_error(Some(id), LspErrorCode::InternalError),
                }
            }
            WORKSPACE_SYMBOL if self.capabilities.workspace_symbols => {
                let Some(params) = object_params(params) else {
                    return response_error(Some(id), LspErrorCode::InvalidParams);
                };
                let Some(query) = validate_symbol_params(params) else {
                    return response_error(Some(id), LspErrorCode::InvalidParams);
                };
                let result = self.handler.workspace_symbols(query);
                Self::handler_result(id, result, validate_workspace_symbols)
            }
            DOCUMENT_DIAGNOSTIC if self.capabilities.diagnostics => {
                let Some(params) = object_params(params) else {
                    return response_error(Some(id), LspErrorCode::InvalidParams);
                };
                let Some(uri) = validate_diagnostic_params(params) else {
                    return response_error(Some(id), LspErrorCode::InvalidParams);
                };
                let result = self.handler.document_diagnostics(uri);
                Self::handler_result(id, result, validate_document_diagnostics)
            }
            _ => response_error(Some(id), LspErrorCode::MethodNotFound),
        }
    }

    fn handler_result(
        id: RequestId,
        result: Result<Value, LspHandlerError>,
        validate: fn(&Value) -> bool,
    ) -> LspDispatchOutcome {
        match result {
            Ok(value) if validate(&value) => match LspResponse::result(id.clone(), value) {
                Ok(response) => LspDispatchOutcome::Response(response),
                Err(LspEncodeError) => response_error(Some(id), LspErrorCode::RequestFailed),
            },
            Ok(_) => response_error(Some(id), LspErrorCode::InternalError),
            Err(error) => response_error(Some(id), handler_error(error)),
        }
    }

    fn dispatch_notification(
        &mut self,
        method: &str,
        params: Option<&Value>,
    ) -> LspDispatchOutcome {
        if params.is_some_and(|value| !value.is_object()) {
            return LspDispatchOutcome::NoResponse;
        }
        if method == EXIT && params.is_none() {
            let status = if self.state == LspState::Shutdown {
                LspExitStatus::Success
            } else {
                LspExitStatus::Failure
            };
            self.state = LspState::Exited;
            return LspDispatchOutcome::Exit(status);
        }

        if self.state == LspState::AwaitingInitialized
            && method == INITIALIZED
            && params.is_none_or(Value::is_object)
        {
            self.state = LspState::Running;
        }
        LspDispatchOutcome::NoResponse
    }
}

impl fmt::Debug for LspServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LspServer")
            .field("state", &self.state)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

/// Serializes one response as bounded compact JSON without transport framing.
///
/// # Errors
///
/// Returns [`LspEncodeError`] when the response violates the accepted size/depth bound.
pub fn encode_lsp_response(response: &LspResponse) -> Result<Vec<u8>, LspEncodeError> {
    let mut object = Map::new();
    object.insert("jsonrpc".to_owned(), Value::String("2.0".to_owned()));
    object.insert(
        "id".to_owned(),
        response
            .id
            .as_ref()
            .map_or(Value::Null, RequestId::to_value),
    );
    match &response.body {
        LspResponseBody::Result(value) => {
            object.insert("result".to_owned(), value.clone());
        }
        LspResponseBody::Error(code) => {
            object.insert(
                "error".to_owned(),
                json!({"code": code.value(), "message": code.message()}),
            );
        }
    }
    let value = Value::Object(object);
    if !value_within_nesting_bound(&value, 0) {
        return Err(LspEncodeError);
    }
    let encoded = serde_json::to_vec(&value).map_err(|_| LspEncodeError)?;
    (encoded.len() <= MAX_MESSAGE_BYTES)
        .then_some(encoded)
        .ok_or(LspEncodeError)
}

fn response_error(id: Option<RequestId>, code: LspErrorCode) -> LspDispatchOutcome {
    LspDispatchOutcome::Response(LspResponse::error(id, code))
}

const fn handler_error(error: LspHandlerError) -> LspErrorCode {
    match error {
        LspHandlerError::InvalidParams => LspErrorCode::InvalidParams,
        LspHandlerError::RequestFailed => LspErrorCode::RequestFailed,
        LspHandlerError::Internal => LspErrorCode::InternalError,
    }
}

fn capability_value(capabilities: LspCapabilities) -> Map<String, Value> {
    let mut value = Map::new();
    value.insert(
        "positionEncoding".to_owned(),
        Value::String("utf-16".to_owned()),
    );
    value.insert(
        "textDocumentSync".to_owned(),
        Value::Number(Number::from(0)),
    );
    if capabilities.workspace_symbols {
        value.insert("workspaceSymbolProvider".to_owned(), Value::Bool(true));
    }
    if capabilities.diagnostics {
        value.insert(
            "diagnosticProvider".to_owned(),
            json!({
                "identifier": "oneagent",
                "interFileDependencies": true,
                "workspaceDiagnostics": false
            }),
        );
    }
    value
}

fn object_params(value: Option<&Value>) -> Option<&Map<String, Value>> {
    value.and_then(Value::as_object)
}

fn valid_lsp_request_id(value: &Value) -> bool {
    value.is_string() || lsp_integer(value).is_some()
}

fn lsp_integer(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .filter(|value| (LSP_INTEGER_MIN..=LSP_INTEGER_MAX).contains(value))
}

fn validate_initialize_params(params: &Map<String, Value>) -> bool {
    if !params
        .get("processId")
        .is_some_and(|value| value.is_null() || lsp_integer(value).is_some())
        || params
            .get("rootUri")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || !params.get("capabilities").is_some_and(Value::is_object)
    {
        return false;
    }
    if params.get("clientInfo").is_some_and(|value| {
        !value.as_object().is_some_and(|object| {
            object.get("name").is_some_and(Value::is_string)
                && object.get("version").is_none_or(Value::is_string)
        })
    }) || params.get("locale").is_some_and(|value| !value.is_string())
        || params
            .get("rootPath")
            .is_some_and(|value| !value.is_null() && !value.is_string())
        || params
            .get("trace")
            .is_some_and(|value| !matches!(value.as_str(), Some("off" | "messages" | "verbose")))
        || !valid_progress_token(params.get("workDoneToken"))
        || !validate_workspace_folders(params.get("workspaceFolders"))
    {
        return false;
    }

    params
        .get("capabilities")
        .and_then(Value::as_object)
        .is_some_and(validate_position_encodings)
}

fn validate_workspace_folders(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.is_null()
            || value.as_array().is_some_and(|folders| {
                folders.iter().all(|folder| {
                    folder.as_object().is_some_and(|folder| {
                        folder.get("uri").is_some_and(Value::is_string)
                            && folder.get("name").is_some_and(Value::is_string)
                    })
                })
            })
    })
}

fn validate_position_encodings(capabilities: &Map<String, Value>) -> bool {
    let Some(general) = capabilities.get("general") else {
        return true;
    };
    let Some(general) = general.as_object() else {
        return false;
    };
    let Some(encodings) = general.get("positionEncodings") else {
        return true;
    };
    let Some(encodings) = encodings.as_array() else {
        return false;
    };
    let mut unique = BTreeSet::new();
    !encodings.is_empty()
        && encodings.iter().all(|value| {
            value
                .as_str()
                .is_some_and(|value| unique.insert(value.to_owned()))
        })
        && unique.contains("utf-16")
}

fn validate_symbol_params(params: &Map<String, Value>) -> Option<&str> {
    if !fields(params, &["query", "workDoneToken", "partialResultToken"])
        || !valid_progress_token(params.get("workDoneToken"))
        || !valid_progress_token(params.get("partialResultToken"))
    {
        return None;
    }
    params
        .get("query")
        .and_then(Value::as_str)
        .filter(|query| query.len() <= MAX_QUERY_BYTES)
}

fn validate_diagnostic_params(params: &Map<String, Value>) -> Option<&str> {
    if !fields(
        params,
        &[
            "textDocument",
            "identifier",
            "previousResultId",
            "workDoneToken",
            "partialResultToken",
        ],
    ) || params
        .get("identifier")
        .is_some_and(|value| value.as_str() != Some("oneagent"))
        || params
            .get("previousResultId")
            .is_some_and(|value| !value.is_string())
        || !valid_progress_token(params.get("workDoneToken"))
        || !valid_progress_token(params.get("partialResultToken"))
    {
        return None;
    }
    let document = params.get("textDocument")?.as_object()?;
    if !fields(document, &["uri"]) {
        return None;
    }
    document
        .get("uri")
        .and_then(Value::as_str)
        .filter(|uri| !uri.is_empty())
}

fn valid_progress_token(value: Option<&Value>) -> bool {
    value.is_none_or(|value| value.is_string() || lsp_integer(value).is_some())
}

fn fields(values: &Map<String, Value>, allowed: &[&str]) -> bool {
    values.keys().all(|key| allowed.contains(&key.as_str()))
}

fn validate_workspace_symbols(value: &Value) -> bool {
    value.as_array().is_some_and(|symbols| {
        symbols.len() <= MAX_RESULT_ITEMS && symbols.iter().all(validate_workspace_symbol)
    })
}

fn validate_workspace_symbol(value: &Value) -> bool {
    let Some(value) = value.as_object() else {
        return false;
    };
    fields(value, &["name", "kind", "containerName", "location"])
        && value
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| !name.is_empty())
        && value
            .get("kind")
            .and_then(Value::as_u64)
            .is_some_and(|kind| (1..=26).contains(&kind))
        && value.get("containerName").is_some_and(Value::is_string)
        && value.get("location").is_some_and(validate_location)
}

fn validate_location(value: &Value) -> bool {
    let Some(value) = value.as_object() else {
        return false;
    };
    fields(value, &["uri", "range"])
        && value
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| !uri.is_empty())
        && value.get("range").is_some_and(validate_range)
}

fn validate_range(value: &Value) -> bool {
    let Some(value) = value.as_object() else {
        return false;
    };
    if !fields(value, &["start", "end"]) {
        return false;
    }
    let Some(start) = value.get("start").and_then(position) else {
        return false;
    };
    let Some(end) = value.get("end").and_then(position) else {
        return false;
    };
    start <= end
}

fn position(value: &Value) -> Option<(u64, u64)> {
    let value = value.as_object()?;
    if !fields(value, &["line", "character"]) {
        return None;
    }
    let line = value.get("line")?.as_u64()?;
    let character = value.get("character")?.as_u64()?;
    (line <= LSP_UINTEGER_MAX && character <= LSP_UINTEGER_MAX).then_some((line, character))
}

fn validate_document_diagnostics(value: &Value) -> bool {
    let Some(value) = value.as_object() else {
        return false;
    };
    if !fields(value, &["kind", "items"])
        || value.get("kind").and_then(Value::as_str) != Some("full")
    {
        return false;
    }
    value
        .get("items")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items.len() <= MAX_RESULT_ITEMS && items.iter().all(validate_diagnostic)
        })
}

fn validate_diagnostic(value: &Value) -> bool {
    let Some(value) = value.as_object() else {
        return false;
    };
    fields(value, &["range", "severity", "code", "source", "message"])
        && value.get("range").is_some_and(validate_range)
        && matches!(value.get("severity").and_then(Value::as_u64), Some(1 | 2))
        && value
            .get("code")
            .and_then(Value::as_str)
            .is_some_and(|code| !code.is_empty())
        && value.get("source").and_then(Value::as_str) == Some("oneagent")
        && value
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| !message.is_empty())
}
