use std::collections::VecDeque;
use std::convert::Infallible;
use std::future::{pending, ready};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use oneagent_protocol::{LspExitStatus, MAX_MESSAGE_BYTES};
use oneagent_runtime::{
    LspStdioErrorKind, LspStdioOutcome, LspStdioTransport, WorkspaceSnapshotBuilder, lsp_server,
    workspace_root_uri,
};
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

const CONTENT_TYPE: &str = "application/vscode-jsonrpc; charset=utf-8";

fn frame(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

fn initialize(root_uri: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {},
            "workspaceFolders": [{"uri": root_uri, "name": "workspace"}]
        }
    })
    .to_string()
}

fn notification(method: &str) -> String {
    json!({"jsonrpc": "2.0", "method": method}).to_string()
}

fn request(id: u64, method: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method}).to_string()
}

fn decode_frames(mut bytes: &[u8]) -> Vec<Value> {
    let mut values = Vec::new();
    while !bytes.is_empty() {
        let delimiter = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("response header delimiter");
        let header = std::str::from_utf8(&bytes[..delimiter]).expect("ASCII response header");
        let length = header
            .strip_prefix("Content-Length: ")
            .expect("Content-Length response header")
            .parse::<usize>()
            .expect("decimal response length");
        let body_start = delimiter + 4;
        let body_end = body_start + length;
        values.push(serde_json::from_slice(&bytes[body_start..body_end]).expect("response JSON"));
        bytes = &bytes[body_end..];
    }
    values
}

fn new_transport() -> (LspStdioTransport, String) {
    let root = tempdir().expect("temporary Workspace must be created");
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(root.path())
        .expect("empty Workspace must build");
    let root_uri = workspace_root_uri(root.path()).expect("root URI must encode");
    let server = lsp_server(snapshot).expect("LSP server must construct");
    (LspStdioTransport::new(server), root_uri)
}

#[derive(Default)]
struct ChunkedReader {
    chunks: VecDeque<Vec<u8>>,
}

impl ChunkedReader {
    fn new(bytes: &[u8], widths: &[usize]) -> Self {
        let mut chunks = VecDeque::new();
        let mut offset = 0;
        for width in widths.iter().copied().cycle() {
            if offset == bytes.len() {
                break;
            }
            let end = (offset + width).min(bytes.len());
            chunks.push_back(bytes[offset..end].to_vec());
            offset = end;
        }
        Self { chunks }
    }
}

impl AsyncRead for ChunkedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if let Some(chunk) = self.chunks.pop_front() {
            buffer.put_slice(&chunk);
        }
        Poll::Ready(Ok(()))
    }
}

struct FailingReader;

impl AsyncRead for FailingReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        _buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::other("controlled read failure")))
    }
}

#[derive(Default)]
struct FailingWriter;

impl AsyncWrite for FailingWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        _buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Err(io::Error::other("controlled write failure")))
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Default)]
struct FailingFlushWriter(Vec<u8>);

impl AsyncWrite for FailingFlushWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.0.extend_from_slice(buffer);
        Poll::Ready(Ok(buffer.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::other("controlled flush failure")))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn public_lsp_transport_frames_lifecycle_and_fragmentation_repeatably() {
    for widths in [&[1, 2, 3, 5, 8][..], &[8_192][..]] {
        let (mut transport, root_uri) = new_transport();
        let input = [
            frame(&initialize(&root_uri)),
            frame(&notification("initialized")),
            frame(&request(2, "shutdown")),
            frame(&notification("exit")),
        ]
        .concat();
        let mut reader = ChunkedReader::new(&input, widths);
        let mut output = Vec::new();
        let outcome = transport
            .run(
                &mut reader,
                &mut output,
                pending::<Result<(), Infallible>>(),
            )
            .await
            .expect("complete lifecycle must succeed");
        assert_eq!(outcome, LspStdioOutcome::Exited(LspExitStatus::Success));

        let responses = decode_frames(&output);
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0]["id"], 1);
        assert_eq!(
            responses[0]["result"]["capabilities"],
            json!({
                "positionEncoding": "utf-16",
                "textDocumentSync": 0,
                "workspaceSymbolProvider": true,
                "diagnosticProvider": {
                    "identifier": "oneagent",
                    "interFileDependencies": true,
                    "workspaceDiagnostics": false
                }
            })
        );
        assert_eq!(
            responses[1],
            json!({"jsonrpc": "2.0", "id": 2, "result": null})
        );
    }
}

#[tokio::test]
async fn public_lsp_transport_enforces_header_and_body_bounds() {
    let base = format!("Content-Length: 0\r\nContent-Type: {CONTENT_TYPE}\r\n\r\n");
    let padding = " ".repeat(8_192 - base.len());
    let exact_header =
        format!("Content-Length: 0\r\nContent-Type: {CONTENT_TYPE}{padding}\r\n\r\n");
    assert_eq!(exact_header.len(), 8_192);
    let mut exact_input = exact_header.into_bytes();
    exact_input.extend(frame(&notification("exit")));
    let (mut transport, _) = new_transport();
    let outcome = transport
        .run(
            &mut &exact_input[..],
            &mut Vec::new(),
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect("exact header bound must be accepted");
    assert_eq!(outcome, LspStdioOutcome::Exited(LspExitStatus::Failure));

    let oversized_header = vec![b'A'; 8_193];
    let (mut transport, _) = new_transport();
    let failure = transport
        .run(
            &mut &oversized_header[..],
            &mut Vec::new(),
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("oversized header must fail");
    assert_eq!(failure.kind(), LspStdioErrorKind::HeaderTooLarge);

    let maximum_body = format!("\"{}\"", "a".repeat(MAX_MESSAGE_BYTES - 2));
    let mut maximum_input = frame(&maximum_body);
    maximum_input.extend(frame(&notification("exit")));
    let (mut transport, _) = new_transport();
    let mut output = Vec::new();
    let outcome = transport
        .run(
            &mut &maximum_input[..],
            &mut output,
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect("exact body bound must be accepted");
    assert_eq!(outcome, LspStdioOutcome::Exited(LspExitStatus::Failure));
    assert_eq!(decode_frames(&output)[0]["error"]["code"], -32600);

    let over_body = format!("Content-Length: {}\r\n\r\n", MAX_MESSAGE_BYTES + 1);
    let (mut transport, _) = new_transport();
    let failure = transport
        .run(
            &mut &over_body.as_bytes()[..],
            &mut Vec::new(),
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("over-bound body declaration must fail");
    assert_eq!(failure.kind(), LspStdioErrorKind::FrameTooLarge);
}

#[tokio::test]
async fn public_lsp_transport_closes_malformed_eof_and_cancellation_paths() {
    for (mut input, expected) in [
        (
            b"Content-Length: 2\n\n{}".as_slice(),
            LspStdioErrorKind::InvalidHeader,
        ),
        (
            b"Unknown: 2\r\n\r\n{}".as_slice(),
            LspStdioErrorKind::InvalidHeader,
        ),
        (
            b"Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}".as_slice(),
            LspStdioErrorKind::InvalidHeader,
        ),
        (
            b"Content-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n".as_slice(),
            LspStdioErrorKind::InvalidHeader,
        ),
        (
            b"Content-Length: 2\r\nContent-Type: text/plain\r\n\r\n{}".as_slice(),
            LspStdioErrorKind::InvalidHeader,
        ),
        (
            b"Content-Length: 02\r\n\r\n{}".as_slice(),
            LspStdioErrorKind::InvalidHeader,
        ),
        (
            b"Content-Length: 2\r\n".as_slice(),
            LspStdioErrorKind::IncompleteHeader,
        ),
        (
            b"Content-Length: 2\r\n\r\n{".as_slice(),
            LspStdioErrorKind::IncompleteBody,
        ),
        (b"".as_slice(), LspStdioErrorKind::UnexpectedEndOfInput),
        (
            b"Content-Length: 1\r\n\r\n\xff".as_slice(),
            LspStdioErrorKind::InvalidUtf8,
        ),
    ] {
        let (mut transport, _) = new_transport();
        let failure = transport
            .run(
                &mut input,
                &mut Vec::new(),
                pending::<Result<(), Infallible>>(),
            )
            .await
            .expect_err("terminal malformed input must fail");
        assert_eq!(failure.kind(), expected);
    }

    let (mut transport, _) = new_transport();
    let outcome = transport
        .run(
            &mut pending_reader(),
            &mut Vec::new(),
            ready(Ok::<(), Infallible>(())),
        )
        .await
        .expect("successful cancellation must be classified");
    assert_eq!(outcome, LspStdioOutcome::Cancelled);

    let (mut transport, _) = new_transport();
    let failure = transport
        .run(
            &mut pending_reader(),
            &mut Vec::new(),
            ready(Err(io::Error::other("secret"))),
        )
        .await
        .expect_err("shutdown-source failure must terminate");
    assert_eq!(failure.kind(), LspStdioErrorKind::Shutdown);
    assert!(!failure.to_string().contains("secret"));
}

#[tokio::test]
async fn public_lsp_transport_classifies_injected_io_failures_without_disclosure() {
    let (mut transport, _) = new_transport();
    let failure = transport
        .run(
            &mut FailingReader,
            &mut Vec::new(),
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("reader failure must terminate");
    assert_eq!(failure.kind(), LspStdioErrorKind::Read);
    assert!(!failure.to_string().contains("controlled"));

    let (mut transport, _) = new_transport();
    let mut writer = FailingWriter;
    let failure = transport
        .run(
            &mut &frame("")[..],
            &mut writer,
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("writer failure must terminate");
    assert_eq!(failure.kind(), LspStdioErrorKind::Write);

    let (mut transport, _) = new_transport();
    let mut writer = FailingFlushWriter::default();
    let failure = transport
        .run(
            &mut &frame("")[..],
            &mut writer,
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("flush failure must terminate");
    assert_eq!(failure.kind(), LspStdioErrorKind::Flush);
}

fn pending_reader() -> impl AsyncRead + Unpin {
    tokio::io::empty()
}

#[test]
fn public_workspace_root_uri_is_absolute_and_percent_encoded() {
    let root = tempdir().expect("temporary root must be created");
    let path = root.path().join("space # ü");
    std::fs::create_dir(&path).expect("Unicode path must be created");
    let uri = workspace_root_uri(&path).expect("UTF-8 absolute path must encode");
    assert!(uri.starts_with("file:"));
    assert!(uri.contains("space%20%23%20%C3%BC"));
    assert!(!uri.contains('#'));
}
