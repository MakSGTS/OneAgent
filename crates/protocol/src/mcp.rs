//! Bounded MCP `2026-07-28` JSON-RPC values and codec.

use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

/// The only MCP protocol revision supported by this crate.
pub const PROTOCOL_VERSION: &str = "2026-07-28";
/// Required per-request protocol-version metadata key.
pub const PROTOCOL_VERSION_META_KEY: &str = "io.modelcontextprotocol/protocolVersion";
/// Required per-request client-capabilities metadata key.
pub const CLIENT_CAPABILITIES_META_KEY: &str = "io.modelcontextprotocol/clientCapabilities";
/// Optional per-request client-information metadata key.
pub const CLIENT_INFO_META_KEY: &str = "io.modelcontextprotocol/clientInfo";
/// Maximum encoded message bytes, excluding a transport delimiter.
pub const MAX_MESSAGE_BYTES: usize = 1_048_576;
/// Maximum UTF-8 bytes in a string request identifier.
pub const MAX_REQUEST_ID_BYTES: usize = 256;
/// Maximum UTF-8 bytes in a method name.
pub const MAX_METHOD_NAME_BYTES: usize = 256;
/// Maximum nested JSON array/object levels accepted by the decoder.
pub const MAX_JSON_NESTING_DEPTH: usize = 128;

const DUPLICATE_KEY_MARKER: &str = "duplicate JSON object key";
const DEPTH_MARKER: &str = "maximum JSON nesting depth exceeded";
const PROGRESS_TOKEN_META_KEY: &str = "progressToken";
const LOG_LEVEL_META_KEY: &str = "io.modelcontextprotocol/logLevel";

/// A JSON-RPC request identifier accepted by MCP.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum RequestId {
    /// A bounded string identifier.
    String(String),
    /// A signed JSON integer identifier.
    Signed(i64),
    /// An unsigned JSON integer identifier.
    Unsigned(u64),
}

impl RequestId {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::String(value) if value.len() <= MAX_REQUEST_ID_BYTES => {
                Some(Self::String(value.clone()))
            }
            Value::Number(value) => value
                .as_i64()
                .map(Self::Signed)
                .or_else(|| value.as_u64().map(Self::Unsigned)),
            _ => None,
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Signed(value) => Value::Number(Number::from(*value)),
            Self::Unsigned(value) => Value::Number(Number::from(*value)),
        }
    }

    /// Returns the string identifier, when this value is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            Self::Signed(_) | Self::Unsigned(_) => None,
        }
    }

    /// Returns the signed integer identifier, when representable as `i64`.
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Signed(value) => Some(*value),
            Self::Unsigned(value) => i64::try_from(*value).ok(),
            Self::String(_) => None,
        }
    }

    /// Returns the unsigned integer identifier, when representable as `u64`.
    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Unsigned(value) => Some(*value),
            Self::Signed(value) => u64::try_from(*value).ok(),
            Self::String(_) => None,
        }
    }
}

impl fmt::Debug for RequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => formatter
                .debug_struct("RequestId")
                .field("kind", &"string")
                .field("bytes", &value.len())
                .finish(),
            Self::Signed(_) => formatter.write_str("RequestId { kind: signed_integer }"),
            Self::Unsigned(_) => formatter.write_str("RequestId { kind: unsigned_integer }"),
        }
    }
}

/// Self-reported MCP implementation identity.
#[derive(Clone, PartialEq, Eq)]
pub struct Implementation {
    name: String,
    version: String,
}

impl Implementation {
    /// Creates an implementation identity.
    ///
    /// Empty names or versions are rejected.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Option<Self> {
        let name = name.into();
        let version = version.into();
        if name.is_empty() || version.is_empty() {
            return None;
        }
        Some(Self { name, version })
    }

    /// Returns the implementation name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the implementation version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl fmt::Debug for Implementation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Implementation")
            .field("name_bytes", &self.name.len())
            .field("version_bytes", &self.version.len())
            .finish()
    }
}

/// Validated per-request MCP metadata.
#[derive(Clone, PartialEq)]
pub struct RequestMetadata {
    protocol_version: String,
    client_capabilities: Map<String, Value>,
    client_info: Option<Implementation>,
}

impl RequestMetadata {
    /// Returns the requested protocol revision.
    #[must_use]
    pub fn protocol_version(&self) -> &str {
        &self.protocol_version
    }

    /// Returns the exact per-request client capability object.
    #[must_use]
    pub fn client_capabilities(&self) -> &Map<String, Value> {
        &self.client_capabilities
    }

    /// Returns optional self-reported client information.
    #[must_use]
    pub fn client_info(&self) -> Option<&Implementation> {
        self.client_info.as_ref()
    }
}

impl fmt::Debug for RequestMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestMetadata")
            .field("protocol_version", &self.protocol_version)
            .field("client_capability_count", &self.client_capabilities.len())
            .field("has_client_info", &self.client_info.is_some())
            .finish()
    }
}

/// A validated response-producing MCP request.
#[derive(Clone, PartialEq)]
pub struct Request {
    id: RequestId,
    method: String,
    params: Map<String, Value>,
    metadata: RequestMetadata,
}

impl Request {
    /// Returns the request identifier.
    #[must_use]
    pub fn id(&self) -> &RequestId {
        &self.id
    }

    /// Returns the exact method name.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the validated parameter object.
    #[must_use]
    pub fn params(&self) -> &Map<String, Value> {
        &self.params
    }

    /// Returns the validated request metadata.
    #[must_use]
    pub fn metadata(&self) -> &RequestMetadata {
        &self.metadata
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Request")
            .field("id", &self.id)
            .field("method_bytes", &self.method.len())
            .field("parameter_count", &self.params.len())
            .field("metadata", &self.metadata)
            .finish()
    }
}

/// A validated, non-response-producing JSON-RPC notification.
#[derive(Clone, PartialEq)]
pub struct Notification {
    method: String,
    params: Option<Map<String, Value>>,
}

impl Notification {
    /// Returns the exact method name.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the optional notification parameter object.
    #[must_use]
    pub fn params(&self) -> Option<&Map<String, Value>> {
        self.params.as_ref()
    }
}

impl fmt::Debug for Notification {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Notification")
            .field("method_bytes", &self.method.len())
            .field("parameter_count", &self.params.as_ref().map_or(0, Map::len))
            .finish()
    }
}

/// A validated inbound server message.
#[derive(Debug, Clone, PartialEq)]
pub enum InboundMessage {
    /// A response-producing request.
    Request(Request),
    /// A one-way notification.
    Notification(Notification),
}

/// JSON-RPC and MCP error codes emitted by the first server slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    /// Invalid JSON text.
    ParseError = -32_700,
    /// Invalid JSON-RPC request shape.
    InvalidRequest = -32_600,
    /// Unknown method.
    MethodNotFound = -32_601,
    /// Invalid request parameters.
    InvalidParams = -32_602,
    /// Unexpected handler failure.
    InternalError = -32_603,
    /// A registered method requires an undeclared client capability.
    MissingRequiredClientCapability = -32_021,
    /// The request names an unsupported MCP revision.
    UnsupportedProtocolVersion = -32_022,
}

impl ErrorCode {
    /// Returns the numeric JSON-RPC error code.
    #[must_use]
    pub const fn value(self) -> i32 {
        self as i32
    }

    /// Returns the stable wire message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::ParseError => "Parse error",
            Self::InvalidRequest => "Invalid Request",
            Self::MethodNotFound => "Method not found",
            Self::InvalidParams => "Invalid params",
            Self::InternalError => "Internal error",
            Self::MissingRequiredClientCapability => "Missing required client capability",
            Self::UnsupportedProtocolVersion => "Unsupported protocol version",
        }
    }
}

/// A bounded JSON-RPC error response.
#[derive(Clone, PartialEq)]
pub struct ErrorResponse {
    id: Option<RequestId>,
    code: ErrorCode,
    data: Option<Value>,
}

impl ErrorResponse {
    /// Creates a bounded standard JSON-RPC error without data.
    ///
    /// MCP-specific error codes with required data are rejected. Use their
    /// dedicated constructors instead.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] when the code requires data or the complete
    /// response violates an outbound bound.
    pub fn new(id: Option<RequestId>, code: ErrorCode) -> Result<Self, EncodeError> {
        if matches!(
            code,
            ErrorCode::MissingRequiredClientCapability | ErrorCode::UnsupportedProtocolVersion
        ) {
            return Err(EncodeError);
        }
        let response = Self {
            id,
            code,
            data: None,
        };
        ensure_response_bound(&Response::Error(response.clone()))?;
        Ok(response)
    }

    /// Creates the exact unsupported-version error.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] when the complete response violates an
    /// outbound bound.
    pub fn unsupported_version(id: RequestId, requested: &str) -> Result<Self, EncodeError> {
        let response = Self {
            id: Some(id),
            code: ErrorCode::UnsupportedProtocolVersion,
            data: Some(serde_json::json!({
                "requested": requested,
                "supported": [PROTOCOL_VERSION]
            })),
        };
        ensure_response_bound(&Response::Error(response.clone()))?;
        Ok(response)
    }

    /// Creates the exact missing-capability error.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] when the capability object is not
    /// schema-conformant or the complete response violates an outbound bound.
    pub fn missing_capability(
        id: RequestId,
        required: &Map<String, Value>,
    ) -> Result<Self, EncodeError> {
        if !validate_client_capabilities(required) {
            return Err(EncodeError);
        }
        let response = Self {
            id: Some(id),
            code: ErrorCode::MissingRequiredClientCapability,
            data: Some(serde_json::json!({ "requiredCapabilities": required })),
        };
        ensure_response_bound(&Response::Error(response.clone()))?;
        Ok(response)
    }

    /// Returns the response identifier, if a valid request ID was available.
    #[must_use]
    pub const fn id(&self) -> Option<&RequestId> {
        self.id.as_ref()
    }

    /// Returns the closed error code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns the bounded protocol error data.
    #[must_use]
    pub const fn data(&self) -> Option<&Value> {
        self.data.as_ref()
    }
}

impl fmt::Debug for ErrorResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ErrorResponse")
            .field("has_id", &self.id.is_some())
            .field("code", &self.code)
            .field("has_data", &self.data.is_some())
            .finish()
    }
}

/// A successful JSON-RPC result response.
#[derive(Clone, PartialEq)]
pub struct ResultResponse {
    id: RequestId,
    result: Map<String, Value>,
}

impl ResultResponse {
    /// Creates a result whose `resultType` is exactly `complete`.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] when the complete response violates an
    /// outbound size or nesting bound.
    pub fn complete(id: RequestId, mut fields: Map<String, Value>) -> Result<Self, EncodeError> {
        fields.insert(
            "resultType".to_owned(),
            Value::String("complete".to_owned()),
        );
        let response = Self { id, result: fields };
        ensure_response_bound(&Response::Result(response.clone()))?;
        Ok(response)
    }

    /// Returns the response identifier.
    #[must_use]
    pub const fn id(&self) -> &RequestId {
        &self.id
    }

    /// Returns the complete result object.
    #[must_use]
    pub const fn result(&self) -> &Map<String, Value> {
        &self.result
    }
}

impl fmt::Debug for ResultResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResultResponse")
            .field("id", &self.id)
            .field("field_count", &self.result.len())
            .finish()
    }
}

/// An outbound JSON-RPC response.
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// A successful complete result.
    Result(ResultResponse),
    /// A closed error result.
    Error(ErrorResponse),
}

/// Decoder output that preserves notification response suppression.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeOutcome {
    /// One validated inbound message.
    Message(InboundMessage),
    /// A response-producing decode error.
    Error(ErrorResponse),
    /// A recognizable notification that failed notification-specific validation.
    IgnoredNotification,
}

/// A response serialization failure with a redacted stable diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeError;

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MCP response encoding failed")
    }
}

impl std::error::Error for EncodeError {}

/// Decodes one UTF-8 JSON frame according to ADR-0050 precedence.
#[must_use]
pub fn decode_message(input: &str) -> DecodeOutcome {
    if input.len() > MAX_MESSAGE_BYTES {
        return invalid_request(None);
    }

    let value = match parse_unique_value(input) {
        Ok(value) => value,
        Err(ParseFailure::Syntax) => {
            return DecodeOutcome::Error(standard_error(None, ErrorCode::ParseError));
        }
        Err(ParseFailure::Duplicate | ParseFailure::Depth) => return invalid_request(None),
    };

    let Value::Object(object) = value else {
        return invalid_request(None);
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return invalid_request(None);
    }

    let request_id = if let Some(value) = object.get("id") {
        let Some(id) = RequestId::from_value(value) else {
            return invalid_request(None);
        };
        Some(id)
    } else {
        None
    };

    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return invalid_request(request_id);
    };
    if method.is_empty() || method.len() > MAX_METHOD_NAME_BYTES {
        return invalid_request(request_id);
    }

    match request_id {
        Some(id) => decode_request(id, method.to_owned(), object.get("params")),
        None => decode_notification(method.to_owned(), object.get("params")),
    }
}

/// Serializes one response as compact JSON without a transport delimiter.
///
/// # Errors
///
/// Returns [`EncodeError`] if the closed response cannot be serialized.
pub fn encode_response(response: &Response) -> Result<Vec<u8>, EncodeError> {
    let value = response_value(response);
    if !value_within_nesting_bound(&value, 0) {
        return Err(EncodeError);
    }
    let encoded = serde_json::to_vec(&value).map_err(|_| EncodeError)?;
    if encoded.len() > MAX_MESSAGE_BYTES {
        return Err(EncodeError);
    }
    Ok(encoded)
}

fn response_value(response: &Response) -> Value {
    match response {
        Response::Result(response) => {
            let mut object = Map::new();
            object.insert("jsonrpc".to_owned(), Value::String("2.0".to_owned()));
            object.insert("id".to_owned(), response.id.to_value());
            object.insert("result".to_owned(), Value::Object(response.result.clone()));
            Value::Object(object)
        }
        Response::Error(response) => {
            let mut error = Map::new();
            error.insert(
                "code".to_owned(),
                Value::Number(Number::from(response.code.value())),
            );
            error.insert(
                "message".to_owned(),
                Value::String(response.code.message().to_owned()),
            );
            if let Some(data) = &response.data {
                error.insert("data".to_owned(), data.clone());
            }

            let mut object = Map::new();
            object.insert("jsonrpc".to_owned(), Value::String("2.0".to_owned()));
            if let Some(id) = &response.id {
                object.insert("id".to_owned(), id.to_value());
            }
            object.insert("error".to_owned(), Value::Object(error));
            Value::Object(object)
        }
    }
}

fn ensure_response_bound(response: &Response) -> Result<(), EncodeError> {
    encode_response(response).map(|_| ())
}

fn value_within_nesting_bound(value: &Value, depth: usize) -> bool {
    match value {
        Value::Array(values) => {
            depth < MAX_JSON_NESTING_DEPTH
                && values
                    .iter()
                    .all(|value| value_within_nesting_bound(value, depth + 1))
        }
        Value::Object(values) => {
            depth < MAX_JSON_NESTING_DEPTH
                && values
                    .values()
                    .all(|value| value_within_nesting_bound(value, depth + 1))
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => true,
    }
}

fn decode_request(id: RequestId, method: String, params: Option<&Value>) -> DecodeOutcome {
    let Some(Value::Object(params)) = params else {
        return invalid_params(id);
    };
    let Some(Value::Object(metadata)) = params.get("_meta") else {
        return invalid_params(id);
    };
    let Some(protocol_version) = metadata
        .get(PROTOCOL_VERSION_META_KEY)
        .and_then(Value::as_str)
    else {
        return invalid_params(id);
    };
    let Some(Value::Object(client_capabilities)) = metadata.get(CLIENT_CAPABILITIES_META_KEY)
    else {
        return invalid_params(id);
    };
    if !metadata.keys().all(|key| valid_meta_key(key))
        || !validate_request_metadata(metadata)
        || !validate_client_capabilities(client_capabilities)
    {
        return invalid_params(id);
    }
    let client_info = match metadata.get(CLIENT_INFO_META_KEY) {
        None => None,
        Some(value) => match decode_implementation(value) {
            Some(value) => Some(value),
            None => return invalid_params(id),
        },
    };

    if protocol_version != PROTOCOL_VERSION {
        return match ErrorResponse::unsupported_version(id.clone(), protocol_version) {
            Ok(error) => DecodeOutcome::Error(error),
            Err(_) => invalid_params(id),
        };
    }

    DecodeOutcome::Message(InboundMessage::Request(Request {
        id,
        method,
        params: params.clone(),
        metadata: RequestMetadata {
            protocol_version: protocol_version.to_owned(),
            client_capabilities: client_capabilities.clone(),
            client_info,
        },
    }))
}

fn decode_notification(method: String, params: Option<&Value>) -> DecodeOutcome {
    let params = match params {
        None => None,
        Some(Value::Object(params)) => Some(params.clone()),
        Some(_) => return DecodeOutcome::IgnoredNotification,
    };
    DecodeOutcome::Message(InboundMessage::Notification(Notification {
        method,
        params,
    }))
}

fn validate_request_metadata(metadata: &Map<String, Value>) -> bool {
    let progress_token_is_valid = metadata
        .get(PROGRESS_TOKEN_META_KEY)
        .is_none_or(|value| value.is_string() || value.is_number());
    let log_level_is_valid = metadata.get(LOG_LEVEL_META_KEY).is_none_or(|value| {
        matches!(
            value.as_str(),
            Some(
                "debug"
                    | "info"
                    | "notice"
                    | "warning"
                    | "error"
                    | "critical"
                    | "alert"
                    | "emergency"
            )
        )
    });
    progress_token_is_valid && log_level_is_valid
}

pub(crate) fn validate_client_capabilities(capabilities: &Map<String, Value>) -> bool {
    map_values_are_objects(capabilities.get("experimental"))
        && optional_object(capabilities.get("roots"))
        && capability_options_are_objects(capabilities.get("sampling"), &["context", "tools"])
        && capability_options_are_objects(capabilities.get("elicitation"), &["form", "url"])
        && extensions_are_valid(capabilities.get("extensions"))
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

fn capability_options_are_objects(value: Option<&Value>, options: &[&str]) -> bool {
    value.is_none_or(|value| {
        value.as_object().is_some_and(|value| {
            options
                .iter()
                .all(|option| value.get(*option).is_none_or(Value::is_object))
        })
    })
}

fn extensions_are_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_object().is_some_and(|extensions| {
            extensions.iter().all(|(name, settings)| {
                name.contains('/') && valid_meta_key(name) && settings.is_object()
            })
        })
    })
}

fn valid_meta_key(key: &str) -> bool {
    let mut segments = key.split('/');
    let first = segments.next().unwrap_or_default();
    let second = segments.next();
    if segments.next().is_some() {
        return false;
    }
    match second {
        None => valid_meta_name(first),
        Some(name) => valid_meta_prefix(first) && valid_meta_name(name),
    }
}

fn valid_meta_prefix(prefix: &str) -> bool {
    !prefix.is_empty() && prefix.split('.').all(valid_meta_label)
}

fn valid_meta_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
}

fn valid_meta_name(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    let bytes = name.as_bytes();
    bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b'.'))
}

fn decode_implementation(value: &Value) -> Option<Implementation> {
    let Value::Object(object) = value else {
        return None;
    };
    let name = object.get("name")?.as_str()?;
    let version = object.get("version")?.as_str()?;
    for field in ["title", "description", "websiteUrl"] {
        if object.get(field).is_some_and(|value| !value.is_string()) {
            return None;
        }
    }
    if object
        .get("icons")
        .is_some_and(|value| !validate_icons(value))
    {
        return None;
    }
    Implementation::new(name, version)
}

fn validate_icons(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|icons| icons.iter().all(validate_icon))
}

fn validate_icon(value: &Value) -> bool {
    let Some(icon) = value.as_object() else {
        return false;
    };
    if !icon.get("src").is_some_and(Value::is_string)
        || icon.get("mimeType").is_some_and(|value| !value.is_string())
    {
        return false;
    }
    if icon.get("sizes").is_some_and(|value| {
        !value
            .as_array()
            .is_some_and(|sizes| sizes.iter().all(Value::is_string))
    }) {
        return false;
    }
    icon.get("theme")
        .is_none_or(|value| matches!(value.as_str(), Some("light" | "dark")))
}

fn invalid_request(id: Option<RequestId>) -> DecodeOutcome {
    DecodeOutcome::Error(standard_error(id, ErrorCode::InvalidRequest))
}

fn invalid_params(id: RequestId) -> DecodeOutcome {
    DecodeOutcome::Error(standard_error(Some(id), ErrorCode::InvalidParams))
}

fn standard_error(id: Option<RequestId>, code: ErrorCode) -> ErrorResponse {
    ErrorResponse::new(id, code).expect("standard MCP errors must satisfy outbound bounds")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseFailure {
    Syntax,
    Duplicate,
    Depth,
}

fn parse_unique_value(input: &str) -> Result<Value, ParseFailure> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    deserializer.disable_recursion_limit();
    let value = UniqueValueSeed { depth: 0 }
        .deserialize(&mut deserializer)
        .map_err(|error| classify_parse_error(&error))?;
    deserializer
        .end()
        .map_err(|error| classify_parse_error(&error))?;
    Ok(value)
}

fn classify_parse_error(error: &serde_json::Error) -> ParseFailure {
    let message = error.to_string();
    if message.starts_with(DUPLICATE_KEY_MARKER) {
        ParseFailure::Duplicate
    } else if message.starts_with(DEPTH_MARKER) || message.contains("recursion limit exceeded") {
        ParseFailure::Depth
    } else {
        ParseFailure::Syntax
    }
}

#[derive(Clone, Copy)]
struct UniqueValueSeed {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for UniqueValueSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor { depth: self.depth })
    }
}

struct UniqueValueVisitor {
    depth: usize,
}

impl UniqueValueVisitor {
    fn nested<E>(self) -> Result<UniqueValueSeed, E>
    where
        E: de::Error,
    {
        if self.depth >= MAX_JSON_NESTING_DEPTH {
            return Err(E::custom(DEPTH_MARKER));
        }
        Ok(UniqueValueSeed {
            depth: self.depth + 1,
        })
    }
}

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("one JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        UniqueValueSeed { depth: self.depth }.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let nested = self.nested()?;
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(nested)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let nested = self.nested()?;
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(<A::Error as de::Error>::custom(DUPLICATE_KEY_MARKER));
            }
            let value = map.next_value_seed(nested)?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_JSON_NESTING_DEPTH, ParseFailure, parse_unique_value};

    #[test]
    fn duplicate_visitor_rejects_nested_duplicates_without_key_disclosure() {
        assert_eq!(
            parse_unique_value(r#"{"outer":{"secret":1,"secret":2}}"#),
            Err(ParseFailure::Duplicate)
        );
    }

    #[test]
    fn duplicate_visitor_enforces_exact_nesting_bound() {
        let accepted = format!(
            "{}0{}",
            "[".repeat(MAX_JSON_NESTING_DEPTH),
            "]".repeat(MAX_JSON_NESTING_DEPTH)
        );
        let rejected = format!("[{accepted}]");

        assert!(parse_unique_value(&accepted).is_ok());
        assert_eq!(parse_unique_value(&rejected), Err(ParseFailure::Depth));
    }
}
