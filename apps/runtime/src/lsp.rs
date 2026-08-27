//! Bounded LSP stdio transport and immutable Runtime composition.

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::path::Path;

use oneagent_graph::{GraphNode, NodeKind, SemanticDiagnostic, SemanticDiagnosticSeverity};
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
    let path = path.replace('\\', "/");
    #[cfg(not(windows))]
    let path = path.to_owned();

    let encoded = percent_encode_path(&path);
    #[cfg(windows)]
    let uri = if encoded.starts_with("//") {
        format!("file:{encoded}")
    } else {
        format!("file:///{encoded}")
    };
    #[cfg(not(windows))]
    let uri = format!("file://{encoded}");
    Ok(uri)
}

fn percent_encode_path(path: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(path.len());
    for (index, byte) in path.bytes().enumerate() {
        let drive_separator = cfg!(windows) && index == 1 && byte == b':';
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
            let Some(location) = lsp_symbol_location(snapshot, configuration, node, root_uri)
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
            enforce_symbol_result_bound(symbols.len())?;
        }
    }
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
    Ok(Value::Array(
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
    ))
}

fn enforce_symbol_result_bound(count: usize) -> Result<(), LspHandlerError> {
    (count <= MAX_SYMBOL_RESULTS)
        .then_some(())
        .ok_or(LspHandlerError::RequestFailed)
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
) -> Option<Value> {
    let projected = unique_symbol_location(snapshot, configuration, node)?;
    let path = projected.get("path")?.as_str()?;
    let span = projected.get("span")?.as_object()?;
    let start = lsp_position(span.get("start")?)?;
    let end = lsp_position(span.get("end")?)?;
    let uri = if root_uri.ends_with('/') {
        format!("{root_uri}{}", percent_encode_path(path))
    } else {
        format!("{root_uri}/{}", percent_encode_path(path))
    };
    Some(serde_json::json!({
        "uri": uri,
        "range": {"start": start, "end": end}
    }))
}

fn lsp_position(value: &Value) -> Option<Value> {
    let line = value.get("line")?.as_u64()?.checked_sub(1)?;
    let character = value.get("column")?.as_u64()?.checked_sub(1)?;
    u32::try_from(line).ok()?;
    u32::try_from(character).ok()?;
    Some(serde_json::json!({"line": line, "character": character}))
}

struct DocumentDiagnostic<'a> {
    diagnostic: &'a SemanticDiagnostic,
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
        for diagnostic in configuration.diagnostics() {
            let Some(item) = project_document_diagnostic(
                snapshot,
                configuration,
                diagnostic,
                root_uri,
                requested_uri,
            ) else {
                continue;
            };
            projected.push(item);
            enforce_diagnostic_result_bound(projected.len())?;
        }
    }
    projected.sort_by(|left, right| {
        (
            left.diagnostic,
            requested_uri,
            left.start,
            left.end,
            left.diagnostic.code().as_str(),
            left.diagnostic.message(),
        )
            .cmp(&(
                right.diagnostic,
                requested_uri,
                right.start,
                right.end,
                right.diagnostic.code().as_str(),
                right.diagnostic.message(),
            ))
    });
    let items = projected
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "range": item.range,
                "severity": lsp_diagnostic_severity(item.diagnostic.severity()),
                "code": item.diagnostic.code().as_str(),
                "source": "oneagent",
                "message": item.diagnostic.message()
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({"kind": "full", "items": items}))
}

fn project_document_diagnostic<'a>(
    snapshot: &WorkspaceSnapshot,
    configuration: &WorkspaceConfigurationSnapshot,
    diagnostic: &'a SemanticDiagnostic,
    root_uri: &str,
    requested_uri: &str,
) -> Option<DocumentDiagnostic<'a>> {
    let source_node = configuration.graph().node(diagnostic.source_node()?)?;
    let location = lsp_symbol_location(snapshot, configuration, source_node, root_uri)?;
    if location.get("uri")?.as_str()? != requested_uri {
        return None;
    }
    let range = location.get("range")?.clone();
    let start = lsp_position_tuple(range.get("start")?)?;
    let end = lsp_position_tuple(range.get("end")?)?;
    Some(DocumentDiagnostic {
        diagnostic,
        range,
        start,
        end,
    })
}

fn lsp_position_tuple(value: &Value) -> Option<(u64, u64)> {
    Some((
        value.get("line")?.as_u64()?,
        value.get("character")?.as_u64()?,
    ))
}

const fn lsp_diagnostic_severity(severity: SemanticDiagnosticSeverity) -> u8 {
    match severity {
        SemanticDiagnosticSeverity::Error => 1,
        SemanticDiagnosticSeverity::Warning => 2,
    }
}

fn enforce_diagnostic_result_bound(count: usize) -> Result<(), LspHandlerError> {
    (count <= MAX_DIAGNOSTIC_RESULTS)
        .then_some(())
        .ok_or(LspHandlerError::RequestFailed)
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
    decoded
        .split('/')
        .all(|component| !component.is_empty() && component != "." && component != "..")
        && percent_encode_path(decoded) == encoded
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

    use oneagent_common::{
        EntityId, EntityName, SourceLocation, SourcePath, SourcePosition, SourceSpan,
    };
    use oneagent_graph::{
        Confidence, FactOrigin, GraphNode, NodeKind, ProducerId, Provenance, ResolutionState,
        SemanticDiagnostic,
    };
    use oneagent_workspace::WorkspaceFormat;

    use super::{
        enforce_diagnostic_result_bound, enforce_symbol_result_bound, lsp_symbol_kind,
        lsp_symbol_location, project_document_diagnostic, valid_document_uri, workspace_root_uri,
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
        assert!(lsp_symbol_location(&snapshot, configuration, &conflicting, &root_uri).is_none());

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
        assert!(lsp_symbol_location(&snapshot, configuration, &escaping, &root_uri).is_none());
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
            .diagnostics()
            .iter()
            .find(|diagnostic| {
                project_document_diagnostic(
                    &snapshot,
                    configuration,
                    diagnostic,
                    &root_uri,
                    &document_uri,
                )
                .is_some()
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
            .is_none()
        );

        let missing_source = located
            .clone()
            .with_source_node(EntityId::new("missing.source").expect("missing source ID"));
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &missing_source,
                &root_uri,
                &document_uri
            )
            .is_none()
        );

        let absent_source = SemanticDiagnostic::new(
            located.code(),
            located.severity(),
            located.kind(),
            located.message(),
            located.reference().clone(),
        );
        assert!(
            project_document_diagnostic(
                &snapshot,
                configuration,
                &absent_source,
                &root_uri,
                &document_uri
            )
            .is_none()
        );
    }
}
