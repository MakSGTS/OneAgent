//! Bounded LSP stdio transport and immutable Runtime composition.

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::path::Path;

use oneagent_analysis::diagnostics::{
    DiagnosticDisposition, DiagnosticFinding, DiagnosticSeverity,
};
use oneagent_graph::{GraphNode, NodeKind};
use oneagent_protocol::{
    LspCapabilities, LspDispatchOutcome, LspExitStatus, LspHandler, LspHandlerError, LspServer,
    MAX_MESSAGE_BYTES, encode_lsp_response,
};
use oneagent_workspace::WorkspaceFormat;
use serde_json::{Map, Value};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::mcp_tools::unique_symbol_location;
use crate::{WorkspaceConfigurationSnapshot, WorkspaceSnapshot};

const MAX_HEADER_BYTES: usize = 8_192;
const READ_SCRATCH_BYTES: usize = 8_192;
const HEADER_DELIMITER: &[u8] = b"\r\n\r\n";
const CONTENT_TYPE: &str = "application/vscode-jsonrpc; charset=utf-8";
const MAX_SYMBOL_RESULTS: usize = 100;
const MAX_DIAGNOSTIC_RESULTS: usize = 100;
const LSP_UINTEGER_MAX: u64 = i32::MAX as u64;

/// Successful terminal outcomes for an LSP stdio stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspStdioOutcome {
    /// The peer sent `exit`; the value records whether shutdown preceded it.
    Exited(LspExitStatus),
    /// The injected shutdown source requested cancellation.
    Cancelled,
}

/// Closed terminal failure categories for the LSP stream adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspStdioErrorKind {
    /// Reading the input stream failed.
    Read,
    /// A header was malformed or unsupported.
    InvalidHeader,
    /// A header exceeded the accepted byte bound.
    HeaderTooLarge,
    /// A declared body exceeded the accepted byte bound.
    FrameTooLarge,
    /// EOF arrived before a complete header.
    IncompleteHeader,
    /// EOF arrived before a complete declared body.
    IncompleteBody,
    /// EOF arrived between frames before an `exit` notification.
    UnexpectedEndOfInput,
    /// A complete body was not valid UTF-8.
    InvalidUtf8,
    /// A protocol response could not be encoded.
    Encode,
    /// Writing protocol output failed.
    Write,
    /// Flushing protocol output failed.
    Flush,
    /// The injected shutdown source failed.
    Shutdown,
}

impl LspStdioErrorKind {
    /// Returns the stable diagnostic category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read failure",
            Self::InvalidHeader => "invalid header",
            Self::HeaderTooLarge => "header too large",
            Self::FrameTooLarge => "frame too large",
            Self::IncompleteHeader => "incomplete header",
            Self::IncompleteBody => "incomplete body",
            Self::UnexpectedEndOfInput => "unexpected end of input",
            Self::InvalidUtf8 => "invalid UTF-8 body",
            Self::Encode => "response encoding failure",
            Self::Write => "write failure",
            Self::Flush => "flush failure",
            Self::Shutdown => "shutdown source failure",
        }
    }
}

/// A redacted terminal LSP stream failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspStdioError {
    kind: LspStdioErrorKind,
}

impl LspStdioError {
    const fn new(kind: LspStdioErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(&self) -> LspStdioErrorKind {
        self.kind
    }
}

impl fmt::Display for LspStdioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.kind.as_str())
    }
}

impl Error for LspStdioError {}

/// Failure to construct a Runtime-owned LSP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspServerConstructionError;

impl fmt::Display for LspServerConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("workspace root URI construction failure")
    }
}

impl Error for LspServerConstructionError {}

struct RuntimeLspHandler {
    snapshot: WorkspaceSnapshot,
    root_uri: String,
}

impl LspHandler for RuntimeLspHandler {
    fn validate_initialize(&self, params: &Map<String, Value>) -> Result<(), LspHandlerError> {
        if params.get("rootUri").and_then(Value::as_str) != Some(&self.root_uri)
            || params
                .get("rootPath")
                .is_some_and(|root_path| !root_path.is_null())
        {
            return Err(LspHandlerError::InvalidParams);
        }
        if let Some(folders) = params.get("workspaceFolders")
            && !folders.is_null()
            && !folders.as_array().is_some_and(|folders| {
                folders.len() == 1
                    && folders[0].as_object().is_some_and(|folder| {
                        folder.get("uri").and_then(Value::as_str) == Some(&self.root_uri)
                            && folder.get("name").is_some_and(Value::is_string)
                    })
            })
        {
            return Err(LspHandlerError::InvalidParams);
        }
        Ok(())
    }

    fn workspace_symbols(&self, query: &str) -> Result<Value, LspHandlerError> {
        project_workspace_symbols(&self.snapshot, &self.root_uri, query)
    }

    fn document_diagnostics(&self, uri: &str) -> Result<Value, LspHandlerError> {
        if !valid_document_uri(&self.root_uri, uri) {
            return Err(LspHandlerError::InvalidParams);
        }
        project_document_diagnostics(&self.snapshot, &self.root_uri, uri)
    }
}

/// Constructs an LSP server owning one immutable snapshot and symbol handler.
///
/// # Errors
///
/// Returns [`LspServerConstructionError`] when the Workspace root is not an
/// absolute UTF-8 path representable as a canonical file URI.
pub fn lsp_server(snapshot: WorkspaceSnapshot) -> Result<LspServer, LspServerConstructionError> {
    let root_uri = workspace_root_uri(snapshot.root_path())?;
    Ok(LspServer::with_capabilities(
        LspCapabilities::lifecycle_only()
            .with_workspace_symbols()
            .with_diagnostics(),
        RuntimeLspHandler { snapshot, root_uri },
    ))
}

/// Converts an existing Workspace root into its canonical absolute file URI.
///
/// # Errors
///
/// Returns [`LspServerConstructionError`] when the path cannot be canonicalized
/// at startup or is not valid UTF-8.
pub fn workspace_root_uri(root: &Path) -> Result<String, LspServerConstructionError> {
    let absolute = std::fs::canonicalize(root).map_err(|_| LspServerConstructionError)?;
    let path = absolute.to_str().ok_or(LspServerConstructionError)?;

    #[cfg(windows)]
    return windows_file_uri(path);
    #[cfg(not(windows))]
    return Ok(format!("file://{}", percent_encode_path(path, false)));
}

#[cfg(any(windows, test))]
fn windows_file_uri(path: &str) -> Result<String, LspServerConstructionError> {
    let normalized = if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!("//{}", unc.replace('\\', "/"))
    } else if let Some(local) = path.strip_prefix(r"\\?\") {
        if !windows_drive_path(local) {
            return Err(LspServerConstructionError);
        }
        local.replace('\\', "/")
    } else if path.starts_with(r"\\") || windows_drive_path(path) {
        path.replace('\\', "/")
    } else {
        return Err(LspServerConstructionError);
    };

    if let Some(authority_path) = normalized.strip_prefix("//") {
        let mut components = authority_path.split('/');
        if components.next().is_none_or(str::is_empty)
            || components.next().is_none_or(str::is_empty)
        {
            return Err(LspServerConstructionError);
        }
        return Ok(format!("file:{}", percent_encode_path(&normalized, false)));
    }
    Ok(format!(
        "file:///{}",
        percent_encode_path(&normalized, true)
    ))
}

#[cfg(any(windows, test))]
fn windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    matches!(bytes, [drive, b':', b'\\', ..] if drive.is_ascii_alphabetic())
}

fn percent_encode_path(path: &str, allow_drive_separator: bool) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(path.len());
    for (index, byte) in path.bytes().enumerate() {
        let drive_separator = allow_drive_separator && index == 1 && byte == b':';
        if byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/')
            || drive_separator
        {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

struct WorkspaceSymbol<'a> {
    configuration: &'a WorkspaceConfigurationSnapshot,
    node: &'a GraphNode,
    folded_name: String,
    kind: u8,
    location: Value,
}

fn project_workspace_symbols(
    snapshot: &WorkspaceSnapshot,
    root_uri: &str,
    query: &str,
) -> Result<Value, LspHandlerError> {
    let folded_query = query.to_lowercase();
    let mut symbols = Vec::new();
    for configuration in snapshot.configurations() {
        for node in configuration.graph().nodes() {
            let Some(kind) = lsp_symbol_kind(configuration.format(), node.kind()) else {
                continue;
            };
            let folded_name = node.name().as_str().to_lowercase();
            if !folded_name.contains(&folded_query) {
                continue;
            }
            let Some(location) = lsp_symbol_location(snapshot, configuration, node, root_uri)?
            else {
                continue;
            };
            symbols.push(WorkspaceSymbol {
                configuration,
                node,
                folded_name,
                kind,
                location,
            });
        }
    }
    enforce_symbol_result_bound(symbols.len())?;
    symbols.sort_by(|left, right| {
        (
            left.folded_name.as_str(),
            left.node.name().as_str(),
            left.kind,
            left.node.id().as_str(),
            left.configuration.configuration_id().as_str(),
        )
            .cmp(&(
                right.folded_name.as_str(),
                right.node.name().as_str(),
                right.kind,
                right.node.id().as_str(),
                right.configuration.configuration_id().as_str(),
            ))
    });
    complete_workspace_symbol_projection(
        symbols
            .into_iter()
            .map(|symbol| {
                serde_json::json!({
                    "name": symbol.node.name().as_str(),
                    "kind": symbol.kind,
                    "containerName": symbol.configuration.configuration_name().as_str(),
                    "location": symbol.location
                })
            })
            .collect(),
    )
}

fn enforce_symbol_result_bound(count: usize) -> Result<(), LspHandlerError> {
    (count <= MAX_SYMBOL_RESULTS)
        .then_some(())
        .ok_or(LspHandlerError::RequestFailed)
}

fn complete_workspace_symbol_projection(items: Vec<Value>) -> Result<Value, LspHandlerError> {
    enforce_symbol_result_bound(items.len())?;
    Ok(Value::Array(items))
}

const fn lsp_symbol_kind(format: WorkspaceFormat, kind: NodeKind) -> Option<u8> {
    match (format, kind) {
        (_, NodeKind::Procedure | NodeKind::Function) => Some(12),
        (WorkspaceFormat::Edt, NodeKind::Query) => Some(19),
        _ => None,
    }
}

fn lsp_symbol_location(
    snapshot: &WorkspaceSnapshot,
    configuration: &WorkspaceConfigurationSnapshot,
    node: &GraphNode,
    root_uri: &str,
) -> Result<Option<Value>, LspHandlerError> {
    let Some(projected) = unique_symbol_location(snapshot, configuration, node) else {
        return Ok(None);
    };
    let path = projected
        .get("path")
        .and_then(Value::as_str)
        .ok_or(LspHandlerError::Internal)?;
    let Some(span) = projected.get("span").and_then(Value::as_object) else {
        return Ok(None);
    };
    let start = lsp_position(span.get("start").ok_or(LspHandlerError::Internal)?)?;
    let end = lsp_position(span.get("end").ok_or(LspHandlerError::Internal)?)?;
    let uri = if root_uri.ends_with('/') {
        format!("{root_uri}{}", percent_encode_path(path, false))
    } else {
        format!("{root_uri}/{}", percent_encode_path(path, false))
    };
    Ok(Some(serde_json::json!({
        "uri": uri,
        "range": {"start": start, "end": end}
    })))
}

fn lsp_position(value: &Value) -> Result<Value, LspHandlerError> {
    let line = value
        .get("line")
        .and_then(Value::as_u64)
        .and_then(|value| value.checked_sub(1))
        .ok_or(LspHandlerError::Internal)?;
    let character = value
        .get("column")
        .and_then(Value::as_u64)
        .and_then(|value| value.checked_sub(1))
        .ok_or(LspHandlerError::Internal)?;
    if line > LSP_UINTEGER_MAX || character > LSP_UINTEGER_MAX {
        return Err(LspHandlerError::Internal);
    }
    Ok(serde_json::json!({"line": line, "character": character}))
}

struct DocumentDiagnostic<'a> {
    finding: &'a DiagnosticFinding,
    range: Value,
    start: (u64, u64),
    end: (u64, u64),
}

fn project_document_diagnostics(
    snapshot: &WorkspaceSnapshot,
    root_uri: &str,
    requested_uri: &str,
) -> Result<Value, LspHandlerError> {
    let mut projected = Vec::new();
    for configuration in snapshot.configurations() {
        for finding in configuration.diagnostic_report().findings() {
            if finding.disposition() != DiagnosticDisposition::Active {
                continue;
            }
            let Some(item) = project_document_diagnostic(
                snapshot,
                configuration,
                finding,
                root_uri,
                requested_uri,
            )?
            else {
                continue;
            };
            projected.push(item);
        }
    }
    enforce_diagnostic_result_bound(projected.len())?;
    projected.sort_by(|left, right| {
        (
            left.finding,
            requested_uri,
            left.start,
            left.end,
            left.finding.code().as_str(),
            left.finding.message(),
        )
            .cmp(&(
                right.finding,
                requested_uri,
                right.start,
                right.end,
                right.finding.code().as_str(),
                right.finding.message(),
            ))
    });
    let items = projected
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "range": item.range,
                "severity": lsp_diagnostic_severity(item.finding.severity()),
                "code": item.finding.code().as_str(),
                "source": "oneagent",
                "message": item.finding.message()
            })
        })
        .collect::<Vec<_>>();
    complete_document_diagnostic_projection(items)
}

fn project_document_diagnostic<'a>(
    snapshot: &WorkspaceSnapshot,
    configuration: &WorkspaceConfigurationSnapshot,
    finding: &'a DiagnosticFinding,
    root_uri: &str,
    requested_uri: &str,
) -> Result<Option<DocumentDiagnostic<'a>>, LspHandlerError> {
    if finding.disposition() != DiagnosticDisposition::Active {
        return Ok(None);
    }
    let [source_node_id] = finding.node_anchors() else {
        return Ok(None);
    };
    let Some(source_node) = configuration.graph().node(source_node_id) else {
        return Ok(None);
    };
    project_document_diagnostic_from_source_node(
        snapshot,
        configuration,
        finding,
        source_node,
        root_uri,
        requested_uri,
    )
}

fn project_document_diagnostic_from_source_node<'a>(
    snapshot: &WorkspaceSnapshot,
    configuration: &WorkspaceConfigurationSnapshot,
    finding: &'a DiagnosticFinding,
    source_node: &GraphNode,
    root_uri: &str,
    requested_uri: &str,
) -> Result<Option<DocumentDiagnostic<'a>>, LspHandlerError> {
    let Some(location) = lsp_symbol_location(snapshot, configuration, source_node, root_uri)?
    else {
        return Ok(None);
    };
    if location.get("uri").and_then(Value::as_str) != Some(requested_uri) {
        return Ok(None);
    }
    let range = location
        .get("range")
        .cloned()
        .ok_or(LspHandlerError::Internal)?;
    let start = lsp_position_tuple(range.get("start").ok_or(LspHandlerError::Internal)?)
        .ok_or(LspHandlerError::Internal)?;
    let end = lsp_position_tuple(range.get("end").ok_or(LspHandlerError::Internal)?)
        .ok_or(LspHandlerError::Internal)?;
    Ok(Some(DocumentDiagnostic {
        finding,
        range,
        start,
        end,
    }))
}

fn lsp_position_tuple(value: &Value) -> Option<(u64, u64)> {
    Some((
        value.get("line")?.as_u64()?,
        value.get("character")?.as_u64()?,
    ))
}

const fn lsp_diagnostic_severity(severity: DiagnosticSeverity) -> u8 {
    match severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
    }
}

fn enforce_diagnostic_result_bound(count: usize) -> Result<(), LspHandlerError> {
    (count <= MAX_DIAGNOSTIC_RESULTS)
        .then_some(())
        .ok_or(LspHandlerError::RequestFailed)
}

fn complete_document_diagnostic_projection(items: Vec<Value>) -> Result<Value, LspHandlerError> {
    enforce_diagnostic_result_bound(items.len())?;
    let mut report = Map::new();
    report.insert("kind".to_owned(), Value::String("full".to_owned()));
    report.insert("items".to_owned(), Value::Array(items));
    Ok(Value::Object(report))
}

fn valid_document_uri(root_uri: &str, uri: &str) -> bool {
    if uri == root_uri {
        return true;
    }
    let relative = if root_uri.ends_with('/') {
        uri.strip_prefix(root_uri)
    } else {
        uri.strip_prefix(root_uri)
            .and_then(|relative| relative.strip_prefix('/'))
    };
    relative.is_some_and(canonical_relative_uri_path)
}

fn canonical_relative_uri_path(encoded: &str) -> bool {
    if encoded.is_empty() {
        return false;
    }
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Some(high) = bytes.get(index + 1).and_then(|byte| hex_value(*byte)) else {
                return false;
            };
            let Some(low) = bytes.get(index + 2).and_then(|byte| hex_value(*byte)) else {
                return false;
            };
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    let Ok(decoded) = std::str::from_utf8(&decoded) else {
        return false;
    };
    if decoded.as_bytes().contains(&b'\\') {
        return false;
    }
    decoded
        .split('/')
        .all(|component| !component.is_empty() && component != "." && component != "..")
        && percent_encode_path(decoded, false) == encoded
}

const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Stateful Content-Length adapter around a transport-independent LSP server.
pub struct LspStdioTransport {
    server: LspServer,
}

impl LspStdioTransport {
    /// Creates an adapter around the supplied server.
    #[must_use]
    pub const fn new(server: LspServer) -> Self {
        Self { server }
    }

    /// Runs sequential framing until `exit`, cancellation, or one terminal error.
    ///
    /// # Errors
    ///
    /// Returns a closed [`LspStdioError`] for framing, I/O, encoding, EOF, or
    /// shutdown-source failure.
    pub async fn run<R, W, F, E>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        shutdown: F,
    ) -> Result<LspStdioOutcome, LspStdioError>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
        F: Future<Output = Result<(), E>>,
        E: Error,
    {
        tokio::pin!(shutdown);
        let mut header = Vec::new();
        let mut body = Vec::new();
        let mut expected_body = None;
        let mut scratch = [0_u8; READ_SCRATCH_BYTES];

        loop {
            let read = tokio::select! {
                biased;
                outcome = &mut shutdown => return shutdown_outcome(outcome),
                outcome = reader.read(&mut scratch) => {
                    outcome.map_err(|_| LspStdioError::new(LspStdioErrorKind::Read))?
                }
            };
            if read == 0 {
                return Err(LspStdioError::new(match expected_body {
                    Some(_) => LspStdioErrorKind::IncompleteBody,
                    None if header.is_empty() => LspStdioErrorKind::UnexpectedEndOfInput,
                    None => LspStdioErrorKind::IncompleteHeader,
                }));
            }

            for byte in &scratch[..read] {
                if let Some(expected) = expected_body {
                    body.push(*byte);
                    if body.len() == expected {
                        if let Some(outcome) = self.process_body(&body, writer).await? {
                            return Ok(outcome);
                        }
                        body.clear();
                        expected_body = None;
                        if poll_shutdown(&mut shutdown).await? {
                            return Ok(LspStdioOutcome::Cancelled);
                        }
                    }
                    continue;
                }

                if *byte == b'\n' && header.last() != Some(&b'\r') {
                    return Err(LspStdioError::new(LspStdioErrorKind::InvalidHeader));
                }
                if !byte.is_ascii() {
                    return Err(LspStdioError::new(LspStdioErrorKind::InvalidHeader));
                }
                header.push(*byte);
                if header.len() > MAX_HEADER_BYTES {
                    return Err(LspStdioError::new(LspStdioErrorKind::HeaderTooLarge));
                }
                if header.ends_with(HEADER_DELIMITER) {
                    let length = parse_header(&header)?;
                    header.clear();
                    if length == 0 {
                        if let Some(outcome) = self.process_body(&[], writer).await? {
                            return Ok(outcome);
                        }
                        if poll_shutdown(&mut shutdown).await? {
                            return Ok(LspStdioOutcome::Cancelled);
                        }
                    } else {
                        expected_body = Some(length);
                    }
                }
            }
        }
    }

    async fn process_body<W>(
        &mut self,
        body: &[u8],
        writer: &mut W,
    ) -> Result<Option<LspStdioOutcome>, LspStdioError>
    where
        W: AsyncWrite + Unpin,
    {
        let input = std::str::from_utf8(body)
            .map_err(|_| LspStdioError::new(LspStdioErrorKind::InvalidUtf8))?;
        match self.server.dispatch(input) {
            LspDispatchOutcome::Response(response) => {
                let payload = encode_lsp_response(&response)
                    .map_err(|_| LspStdioError::new(LspStdioErrorKind::Encode))?;
                let header = format!("Content-Length: {}\r\n\r\n", payload.len());
                writer
                    .write_all(header.as_bytes())
                    .await
                    .map_err(|_| LspStdioError::new(LspStdioErrorKind::Write))?;
                writer
                    .write_all(&payload)
                    .await
                    .map_err(|_| LspStdioError::new(LspStdioErrorKind::Write))?;
                writer
                    .flush()
                    .await
                    .map_err(|_| LspStdioError::new(LspStdioErrorKind::Flush))?;
                Ok(None)
            }
            LspDispatchOutcome::NoResponse => Ok(None),
            LspDispatchOutcome::Exit(status) => Ok(Some(LspStdioOutcome::Exited(status))),
        }
    }
}

impl fmt::Debug for LspStdioTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LspStdioTransport")
            .finish_non_exhaustive()
    }
}

fn parse_header(header: &[u8]) -> Result<usize, LspStdioError> {
    let text = std::str::from_utf8(&header[..header.len() - HEADER_DELIMITER.len()])
        .map_err(|_| LspStdioError::new(LspStdioErrorKind::InvalidHeader))?;
    let mut content_length = None;
    let mut content_type = None;
    for line in text.split("\r\n") {
        let (name, value) = line
            .split_once(": ")
            .ok_or_else(|| LspStdioError::new(LspStdioErrorKind::InvalidHeader))?;
        if name.eq_ignore_ascii_case("Content-Length") {
            if content_length.is_some()
                || value.is_empty()
                || (value.starts_with('0') && value != "0")
                || !value.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(LspStdioError::new(LspStdioErrorKind::InvalidHeader));
            }
            let length = value
                .parse::<usize>()
                .map_err(|_| LspStdioError::new(LspStdioErrorKind::FrameTooLarge))?;
            if length > MAX_MESSAGE_BYTES {
                return Err(LspStdioError::new(LspStdioErrorKind::FrameTooLarge));
            }
            content_length = Some(length);
        } else if name.eq_ignore_ascii_case("Content-Type") {
            if content_type.replace(()).is_some()
                || !value.trim_matches(' ').eq_ignore_ascii_case(CONTENT_TYPE)
            {
                return Err(LspStdioError::new(LspStdioErrorKind::InvalidHeader));
            }
        } else {
            return Err(LspStdioError::new(LspStdioErrorKind::InvalidHeader));
        }
    }
    content_length.ok_or_else(|| LspStdioError::new(LspStdioErrorKind::InvalidHeader))
}

fn shutdown_outcome<E>(outcome: Result<(), E>) -> Result<LspStdioOutcome, LspStdioError> {
    outcome.map_or_else(
        |_| Err(LspStdioError::new(LspStdioErrorKind::Shutdown)),
        |()| Ok(LspStdioOutcome::Cancelled),
    )
}

async fn poll_shutdown<F, E>(shutdown: &mut std::pin::Pin<&mut F>) -> Result<bool, LspStdioError>
where
    F: Future<Output = Result<(), E>>,
{
    tokio::select! {
        biased;
        outcome = shutdown => outcome.map_or_else(
            |_| Err(LspStdioError::new(LspStdioErrorKind::Shutdown)),
            |()| Ok(true),
        ),
        () = tokio::task::yield_now() => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oneagent_analysis::diagnostics::{
        DiagnosticCategory, DiagnosticEvidence, DiagnosticFinding, DiagnosticIdentity,
        DiagnosticPolicy, DiagnosticSeverity,
    };
    use oneagent_analysis::rules::{RuleDiagnostic, RuleDiagnosticCode, RuleId};
    use oneagent_common::{
        EntityId, EntityName, SourceLocation, SourcePath, SourcePosition, SourceSpan,
    };
    use oneagent_graph::{
        Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeKind, ProducerId, Provenance,
        ResolutionState, SemanticDiagnostic, SemanticGraph, SemanticGraphValidationCode,
    };
    use oneagent_protocol::LspHandlerError;
    use oneagent_workspace::WorkspaceFormat;

    use super::{
        complete_document_diagnostic_projection, complete_workspace_symbol_projection,
        enforce_diagnostic_result_bound, enforce_symbol_result_bound, lsp_diagnostic_severity,
        lsp_position, lsp_symbol_kind, lsp_symbol_location, project_document_diagnostic,
        project_document_diagnostic_from_source_node, valid_document_uri, windows_file_uri,
        workspace_root_uri,
    };
    use crate::WorkspaceSnapshotBuilder;

    fn fixture_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/workspace_service")
            .leak()
    }

    fn location(path: SourcePath, with_span: bool) -> SourceLocation {
        let span = with_span.then(|| {
            let point = SourcePosition::new(3, 1).expect("one-based point");
            SourceSpan::new(point, point).expect("ordered point span")
        });
        SourceLocation::new(path, span)
    }

    fn location_at(path: SourcePath, line: u32, column: u32) -> SourceLocation {
        let point = SourcePosition::new(line, column).expect("one-based point");
        SourceLocation::new(
            path,
            Some(SourceSpan::new(point, point).expect("ordered point span")),
        )
    }

    fn provenance(location: SourceLocation, producer: &str) -> Provenance {
        Provenance::new_with_location(
            None,
            Some(location),
            ProducerId::new(producer),
            FactOrigin::Declared,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )
    }

    fn node(name: &str, provenance: Vec<Provenance>) -> GraphNode {
        GraphNode::new_with_provenance(
            EntityId::new(format!("procedure.{name}")).expect("node ID"),
            EntityName::new(name).expect("node name"),
            NodeKind::Procedure,
            provenance,
        )
    }

    #[test]
    fn symbol_kind_and_bound_contracts_are_exact() {
        assert_eq!(
            lsp_symbol_kind(WorkspaceFormat::Edt, NodeKind::Procedure),
            Some(12)
        );
        assert_eq!(
            lsp_symbol_kind(WorkspaceFormat::Edt, NodeKind::Function),
            Some(12)
        );
        assert_eq!(
            lsp_symbol_kind(WorkspaceFormat::Edt, NodeKind::Query),
            Some(19)
        );
        assert_eq!(
            lsp_symbol_kind(WorkspaceFormat::DesignerXml, NodeKind::Query),
            None
        );
        assert_eq!(
            lsp_symbol_kind(WorkspaceFormat::Edt, NodeKind::Module),
            None
        );
        assert!(enforce_symbol_result_bound(100).is_ok());
        assert!(enforce_symbol_result_bound(101).is_err());
    }

    #[test]
    fn complete_symbol_and_diagnostic_projection_bounds_are_exact() {
        let symbols = complete_workspace_symbol_projection(vec![serde_json::json!({}); 100])
            .expect("100 symbols must project");
        assert_eq!(symbols.as_array().expect("symbol array").len(), 100);
        assert!(complete_workspace_symbol_projection(vec![serde_json::json!({}); 101]).is_err());

        let diagnostics = complete_document_diagnostic_projection(vec![serde_json::json!({}); 100])
            .expect("100 diagnostics must project");
        assert_eq!(
            diagnostics["items"]
                .as_array()
                .expect("diagnostic array")
                .len(),
            100
        );
        assert!(complete_document_diagnostic_projection(vec![serde_json::json!({}); 101]).is_err());
    }

    #[test]
    fn windows_drive_and_unc_uri_oracles_are_standard_and_independent() {
        for (path, expected) in [
            (
                r"\\?\C:\workspace\space # ü",
                "file:///C:/workspace/space%20%23%20%C3%BC",
            ),
            (r"C:\workspace", "file:///C:/workspace"),
            (
                r"\\?\UNC\server\share\space # ü",
                "file://server/share/space%20%23%20%C3%BC",
            ),
            (r"\\server\share\workspace", "file://server/share/workspace"),
        ] {
            assert_eq!(
                windows_file_uri(path).expect("canonical Windows path"),
                expected,
                "{path}"
            );
        }
        for rejected in [
            r"\\?\Volume{00000000-0000-0000-0000-000000000000}\workspace",
            r"relative\workspace",
            r"\\server",
        ] {
            assert!(windows_file_uri(rejected).is_err(), "{rejected}");
        }
    }

    #[test]
    fn runtime_position_projection_enforces_lsp_uinteger() {
        assert_eq!(
            lsp_position(&serde_json::json!({"line": 2_147_483_648_u64, "column": 1})),
            Ok(serde_json::json!({"line": 2_147_483_647_u64, "character": 0}))
        );
        assert_eq!(
            lsp_position(&serde_json::json!({"line": 2_147_483_649_u64, "column": 1})),
            Err(LspHandlerError::Internal)
        );
        assert_eq!(
            lsp_position(&serde_json::json!({"line": 1, "column": 2_147_483_649_u64})),
            Err(LspHandlerError::Internal)
        );
        assert_eq!(
            lsp_position(
                &serde_json::json!({"line": 2_147_483_647_u64, "column": 2_147_483_647_u64})
            ),
            Ok(serde_json::json!({"line": 2_147_483_646_u64, "character": 2_147_483_646_u64}))
        );
    }

    #[test]
    fn lsp_symbol_locations_require_one_confined_spanned_location() {
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(fixture_root())
            .expect("mixed fixture must build");
        let configuration = &snapshot.configurations()[0];
        let root_uri = workspace_root_uri(snapshot.root_path()).expect("root URI");
        let source_path = SourcePath::new(
            configuration
                .root_path()
                .join("Module.bsl")
                .to_string_lossy(),
        )
        .expect("confined source path");
        let spanned = location(source_path.clone(), true);

        assert!(
            lsp_symbol_location(
                &snapshot,
                configuration,
                &node("Missing", Vec::new()),
                &root_uri
            )
            .expect("missing evidence is not a projection error")
            .is_none()
        );
        assert!(
            lsp_symbol_location(
                &snapshot,
                configuration,
                &node(
                    "Spanless",
                    vec![provenance(location(source_path.clone(), false), "a")]
                ),
                &root_uri
            )
            .expect("span-less evidence is not a projection error")
            .is_none()
        );

        let repeated = node(
            "Repeated",
            vec![
                provenance(spanned.clone(), "b"),
                provenance(spanned.clone(), "a"),
            ],
        );
        let projected = lsp_symbol_location(&snapshot, configuration, &repeated, &root_uri)
            .expect("valid location projection must not fail")
            .expect("repeated identical location must collapse");
        assert_eq!(
            projected["range"]["start"],
            serde_json::json!({"line": 2, "character": 0})
        );
        assert_eq!(projected["range"]["start"], projected["range"]["end"]);

        let conflicting = node(
            "Conflicting",
            vec![
                provenance(spanned, "a"),
                provenance(
                    location(
                        SourcePath::new(
                            configuration
                                .root_path()
                                .join("Other.bsl")
                                .to_string_lossy(),
                        )
                        .expect("second source path"),
                        true,
                    ),
                    "b",
                ),
            ],
        );
        assert!(
            lsp_symbol_location(&snapshot, configuration, &conflicting, &root_uri)
                .expect("conflicting evidence is not a projection error")
                .is_none()
        );

        let outside = fixture_root()
            .parent()
            .expect("fixture parent")
            .join("outside.bsl");
        let escaping = node(
            "Escaping",
            vec![provenance(
                location(
                    SourcePath::new(outside.to_string_lossy()).expect("outside source path"),
                    true,
                ),
                "a",
            )],
        );
        assert!(
            lsp_symbol_location(&snapshot, configuration, &escaping, &root_uri)
                .expect("escaping evidence is not a projection error")
                .is_none()
        );
    }

    #[test]
    fn symbol_and_diagnostic_positions_propagate_exact_lsp_bounds() {
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(fixture_root())
            .expect("mixed fixture must build");
        let configuration = snapshot
            .configurations()
            .iter()
            .find(|configuration| !configuration.diagnostics().is_empty())
            .expect("EDT fixture must contain diagnostics");
        let finding = &configuration.diagnostic_report().findings()[0];
        let root_uri = workspace_root_uri(snapshot.root_path()).expect("root URI");
        let source_path = SourcePath::new(
            configuration
                .root_path()
                .join("Synthetic.bsl")
                .to_string_lossy(),
        )
        .expect("confined synthetic source path");
        let exact_max = node(
            "ExactMax",
            vec![provenance(
                location_at(source_path.clone(), 2_147_483_648, 1),
                "a",
            )],
        );
        let exact_location = lsp_symbol_location(&snapshot, configuration, &exact_max, &root_uri)
            .expect("exact maximum LSP position must project")
            .expect("exact maximum location");
        assert_eq!(
            exact_location["range"]["start"],
            serde_json::json!({"line": 2_147_483_647_u64, "character": 0})
        );
        let requested_uri = exact_location["uri"]
            .as_str()
            .expect("projected URI")
            .to_owned();
        assert!(
            project_document_diagnostic_from_source_node(
                &snapshot,
                configuration,
                finding,
                &exact_max,
                &root_uri,
                &requested_uri,
            )
            .expect("exact maximum diagnostic position must project")
            .is_some()
        );

        let one_over = node(
            "OneOver",
            vec![provenance(location_at(source_path, 2_147_483_649, 1), "a")],
        );
        assert_eq!(
            lsp_symbol_location(&snapshot, configuration, &one_over, &root_uri),
            Err(LspHandlerError::Internal)
        );
        assert!(matches!(
            project_document_diagnostic_from_source_node(
                &snapshot,
                configuration,
                finding,
                &one_over,
                &root_uri,
                &requested_uri,
            ),
            Err(LspHandlerError::Internal)
        ));
    }

    #[test]
    fn document_uri_and_diagnostic_bounds_are_canonical_and_exact() {
        let root = "file:///workspace";
        for accepted in [
            "file:///workspace",
            "file:///workspace/source/Module.bsl",
            "file:///workspace/space%20%C3%BC.bsl",
        ] {
            assert!(valid_document_uri(root, accepted), "{accepted}");
        }
        for rejected in [
            "file:///other/Module.bsl",
            "file:///workspace-other/Module.bsl",
            "file:///workspace/../outside.bsl",
            "file:///workspace/source//Module.bsl",
            "file:///workspace/source%2FModule.bsl",
            "file:///workspace/source%5CModule.bsl",
            "file:///workspace/sub%5C..%5Coutside.bsl",
            "file:///workspace/sub%5c..%5coutside.bsl",
            "file:///workspace/space%20%c3%bc.bsl",
            "file:///workspace/raw ü.bsl",
            "file:///workspace/name?query",
            "file:///workspace/name#fragment",
        ] {
            assert!(!valid_document_uri(root, rejected), "{rejected}");
        }
        assert!(enforce_diagnostic_result_bound(100).is_ok());
        assert!(enforce_diagnostic_result_bound(101).is_err());
    }

    #[test]
    fn document_diagnostics_require_existing_located_source_nodes() {
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(fixture_root())
            .expect("mixed fixture must build");
        let configuration = snapshot
            .configurations()
            .iter()
            .find(|configuration| !configuration.diagnostics().is_empty())
            .expect("EDT fixture must contain diagnostics");
        let root_uri = workspace_root_uri(snapshot.root_path()).expect("root URI");
        let document_uri =
            format!("{root_uri}/edt/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl");
        let located = configuration
            .diagnostic_report()
            .findings()
            .iter()
            .find(|finding| {
                project_document_diagnostic(
                    &snapshot,
                    configuration,
                    finding,
                    &root_uri,
                    &document_uri,
                )
                .is_ok_and(|projection| projection.is_some())
            })
            .expect("fixture must contain one located diagnostic");
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                located,
                &root_uri,
                &format!("{root_uri}/missing.bsl")
            )
            .expect("different document is not a projection error")
            .is_none()
        );

        let DiagnosticEvidence::Semantic(located_evidence) = located.evidence() else {
            panic!("fixture located finding must retain semantic evidence");
        };
        let missing_source = located_evidence
            .clone()
            .with_source_node(EntityId::new("missing.source").expect("missing source ID"));
        let missing_source =
            DiagnosticFinding::from_semantic(&missing_source, &DiagnosticPolicy::default())
                .expect("bounded missing-source finding");
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &missing_source,
                &root_uri,
                &document_uri
            )
            .expect("missing source is not a projection error")
            .is_none()
        );

        let absent_source = SemanticDiagnostic::new(
            located_evidence.code(),
            located_evidence.severity(),
            located_evidence.kind(),
            located_evidence.message(),
            located_evidence.reference().clone(),
        );
        let absent_source =
            DiagnosticFinding::from_semantic(&absent_source, &DiagnosticPolicy::default())
                .expect("bounded absent-source finding");
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &absent_source,
                &root_uri,
                &document_uri
            )
            .expect("absent source is not a projection error")
            .is_none()
        );
    }

    #[test]
    fn document_diagnostics_project_active_single_node_validation_only() {
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(fixture_root())
            .expect("mixed fixture must build");
        let root_uri = workspace_root_uri(snapshot.root_path()).expect("root URI");
        let (configuration, source_node, location) = snapshot
            .configurations()
            .iter()
            .find_map(|configuration| {
                configuration.graph().nodes().find_map(|node| {
                    lsp_symbol_location(&snapshot, configuration, node, &root_uri)
                        .ok()
                        .flatten()
                        .map(|location| (configuration, node, location))
                })
            })
            .expect("fixture must contain one confined located node");
        let requested_uri = location["uri"].as_str().expect("location URI");
        let mut validation_graph = SemanticGraph::new();
        validation_graph.insert_node(GraphNode::new(
            source_node.id().clone(),
            EntityName::new("Validation").expect("validation node name"),
            NodeKind::Procedure,
        ));
        let validation = validation_graph.validate();
        let issue = validation
            .issues()
            .iter()
            .find(|issue| issue.code() == SemanticGraphValidationCode::MissingNodeProvenance)
            .expect("validation graph must report missing node provenance");
        let active = DiagnosticFinding::from_validation(issue, &DiagnosticPolicy::default())
            .expect("bounded active validation finding");
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &active,
                &root_uri,
                requested_uri,
            )
            .expect("active validation location")
            .is_some()
        );

        let policy = DiagnosticPolicy::new(std::collections::BTreeSet::from([
            DiagnosticIdentity::from_validation(issue).expect("bounded validation identity"),
        ]))
        .expect("one suppression");
        let suppressed = DiagnosticFinding::from_validation(issue, &policy)
            .expect("bounded suppressed validation finding");
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &suppressed,
                &root_uri,
                requested_uri,
            )
            .expect("suppressed finding is omitted")
            .is_none()
        );

        let related = EntityId::new("metadata.related").expect("related node ID");
        validation_graph.insert_node(GraphNode::new(
            related.clone(),
            EntityName::new("Related").expect("related node name"),
            NodeKind::Procedure,
        ));
        validation_graph
            .insert_edge(GraphEdge::new(
                source_node.id().clone(),
                related,
                EdgeKind::Calls,
            ))
            .expect("validation edge must insert");
        let validation = validation_graph.validate();
        let multiple = validation
            .issues()
            .iter()
            .find(|issue| issue.code() == SemanticGraphValidationCode::MissingEdgeProvenance)
            .expect("validation graph must report missing edge provenance");
        let multiple = DiagnosticFinding::from_validation(multiple, &DiagnosticPolicy::default())
            .expect("bounded multi-node validation finding");
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &multiple,
                &root_uri,
                requested_uri,
            )
            .expect("multiple anchors are omitted")
            .is_none()
        );
    }

    #[test]
    fn document_diagnostics_project_rule_findings_through_the_existing_wire_shape() {
        let snapshot = WorkspaceSnapshotBuilder::new()
            .build(fixture_root())
            .expect("mixed fixture must build");
        let root_uri = workspace_root_uri(snapshot.root_path()).expect("root URI");
        let (configuration, source_node, location) = snapshot
            .configurations()
            .iter()
            .find_map(|configuration| {
                configuration.graph().nodes().find_map(|node| {
                    lsp_symbol_location(&snapshot, configuration, node, &root_uri)
                        .ok()
                        .flatten()
                        .map(|location| (configuration, node, location))
                })
            })
            .expect("fixture must contain one confined located node");
        let requested_uri = location["uri"].as_str().expect("location URI");
        let diagnostic = RuleDiagnostic::new(
            RuleId::new("runtime.rule").expect("rule ID"),
            RuleDiagnosticCode::new("finding").expect("diagnostic code"),
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Semantic,
            "controlled rule finding",
            [source_node.id().clone()],
        );
        let finding = DiagnosticFinding::from_rule(&diagnostic, &DiagnosticPolicy::default())
            .expect("rule finding");
        let projected = project_document_diagnostic(
            &snapshot,
            configuration,
            &finding,
            &root_uri,
            requested_uri,
        )
        .expect("rule projection must not fail")
        .expect("located active rule finding must project");

        assert_eq!(projected.finding.code().as_str(), "finding");
        assert_eq!(projected.finding.message(), "controlled rule finding");
        assert_eq!(projected.range, location["range"]);
        assert_eq!(lsp_diagnostic_severity(projected.finding.severity()), 2);
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &finding,
                &root_uri,
                &format!("{root_uri}/missing.bsl"),
            )
            .expect("different document is not a projection error")
            .is_none()
        );
    }
}
