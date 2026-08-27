//! Bounded LSP stdio transport and immutable Runtime composition.

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::path::Path;

use oneagent_protocol::{
    LspDispatchOutcome, LspExitStatus, LspHandler, LspHandlerError, LspServer, MAX_MESSAGE_BYTES,
    encode_lsp_response,
};
use serde_json::{Map, Value};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::WorkspaceSnapshot;

const MAX_HEADER_BYTES: usize = 8_192;
const READ_SCRATCH_BYTES: usize = 8_192;
const HEADER_DELIMITER: &[u8] = b"\r\n\r\n";
const CONTENT_TYPE: &str = "application/vscode-jsonrpc; charset=utf-8";

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
    _snapshot: WorkspaceSnapshot,
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

    fn workspace_symbols(&self, _query: &str) -> Result<Value, LspHandlerError> {
        Err(LspHandlerError::Internal)
    }

    fn document_diagnostics(&self, _uri: &str) -> Result<Value, LspHandlerError> {
        Err(LspHandlerError::Internal)
    }
}

/// Constructs a lifecycle-only LSP server owning one immutable snapshot.
///
/// # Errors
///
/// Returns [`LspServerConstructionError`] when the Workspace root is not an
/// absolute UTF-8 path representable as a canonical file URI.
pub fn lsp_server(snapshot: WorkspaceSnapshot) -> Result<LspServer, LspServerConstructionError> {
    let root_uri = workspace_root_uri(snapshot.root_path())?;
    Ok(LspServer::new(RuntimeLspHandler {
        _snapshot: snapshot,
        root_uri,
    }))
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
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/' | b':') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
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
