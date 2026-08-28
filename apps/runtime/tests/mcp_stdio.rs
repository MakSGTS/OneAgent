use std::convert::Infallible;
use std::future::{pending, ready};
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::task::{Context, Poll};

use oneagent_protocol::{
    MAX_MESSAGE_BYTES, MCP_PROTOCOL_VERSION_2025_06_18, MCP_PROTOCOL_VERSION_2025_11_25,
    PROTOCOL_VERSION,
};
use oneagent_runtime::{McpStdioErrorKind, McpStdioOutcome, McpStdioTransport};
use serde_json::{Value, json};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

fn request(id: u64, method: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
                "io.modelcontextprotocol/clientCapabilities": {}
            }
        }
    })
    .to_string()
}

async fn run(input: &[u8]) -> (Result<McpStdioOutcome, McpStdioErrorKind>, Vec<u8>) {
    let transport = McpStdioTransport::default();
    run_with_transport(&transport, input).await
}

async fn run_with_transport(
    transport: &McpStdioTransport,
    input: &[u8],
) -> (Result<McpStdioOutcome, McpStdioErrorKind>, Vec<u8>) {
    let mut reader = input;
    let mut output = Vec::new();
    let outcome = transport
        .run(
            &mut reader,
            &mut output,
            pending::<Result<(), Infallible>>(),
        )
        .await
        .map_err(|error| error.kind());
    (outcome, output)
}

fn legacy_initialize(version: &str, name: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "initialize",
        "params": {
            "protocolVersion": version,
            "capabilities": {},
            "clientInfo": {"name": name, "version": "1"}
        }
    })
    .to_string()
}

fn legacy_request(id: u64, method: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method}).to_string()
}

#[tokio::test]
async fn public_transport_frames_orders_flushes_and_suppresses_notifications() {
    let discover = request(1, "server/discover");
    let unknown = request(2, "tools/list");
    let notification = r#"{"jsonrpc":"2.0","method":"server/discover"}"#;
    let input = format!("{discover}\r\n{notification}\n{unknown}\n");

    for _ in 0..3 {
        let (outcome, output) = run(input.as_bytes()).await;
        assert_eq!(outcome, Ok(McpStdioOutcome::EndOfInput));
        let text = String::from_utf8(output).expect("protocol output must be UTF-8");
        assert!(text.ends_with('\n'));
        let lines = text.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        let first: Value = serde_json::from_str(lines[0]).expect("discovery response JSON");
        let second: Value = serde_json::from_str(lines[1]).expect("error response JSON");
        assert_eq!(first["id"], 1);
        assert_eq!(first["result"]["resultType"], "complete");
        assert_eq!(second["id"], 2);
        assert_eq!(second["error"]["code"], -32601);
    }
}

#[tokio::test]
async fn public_transport_owns_one_fresh_legacy_session_per_run() {
    let transport = McpStdioTransport::default();
    let initialized = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let first_input = format!(
        "{}\n{initialized}\n{}\n",
        legacy_initialize(MCP_PROTOCOL_VERSION_2025_11_25, "first-client"),
        legacy_request(1, "ping")
    );
    let (first_outcome, first_output) =
        run_with_transport(&transport, first_input.as_bytes()).await;
    assert_eq!(first_outcome, Ok(McpStdioOutcome::EndOfInput));
    let first = String::from_utf8(first_output)
        .expect("first run output must be UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("first run response JSON"))
        .collect::<Vec<_>>();
    assert_eq!(first.len(), 2);
    assert_eq!(
        first[0]["result"]["protocolVersion"],
        MCP_PROTOCOL_VERSION_2025_11_25
    );
    assert_eq!(first[1]["result"], json!({}));

    let second_input = format!(
        "{}\r\n{}\r\n{initialized}\r\n{}\r\n",
        legacy_request(2, "tools/list"),
        legacy_initialize(MCP_PROTOCOL_VERSION_2025_06_18, "second-client"),
        legacy_request(3, "ping")
    );
    let (second_outcome, second_output) =
        run_with_transport(&transport, second_input.as_bytes()).await;
    assert_eq!(second_outcome, Ok(McpStdioOutcome::EndOfInput));
    let second = String::from_utf8(second_output)
        .expect("second run output must be UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("second run response JSON"))
        .collect::<Vec<_>>();
    assert_eq!(second.len(), 3);
    assert_eq!(second[0]["error"]["code"], -32002);
    assert_eq!(
        second[1]["result"]["protocolVersion"],
        MCP_PROTOCOL_VERSION_2025_06_18
    );
    assert_eq!(second[2]["result"], json!({}));
}

#[tokio::test]
async fn public_transport_handles_recoverable_frames_without_extra_output() {
    let input = format!("\n{{\n}}\n{}\n", request(3, "server/discover"));
    let (outcome, output) = run(input.as_bytes()).await;
    assert_eq!(outcome, Ok(McpStdioOutcome::EndOfInput));
    let lines = String::from_utf8(output)
        .expect("protocol output must be UTF-8")
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 4);
    for line in &lines[..3] {
        let value: Value = serde_json::from_str(line).expect("parse error response JSON");
        assert_eq!(value["error"]["code"], -32700);
        assert!(value.get("id").is_none());
    }
    let result: Value = serde_json::from_str(&lines[3]).expect("discovery response JSON");
    assert_eq!(result["id"], 3);
}

#[tokio::test]
async fn public_transport_preserves_syntax_precedence_after_structural_failures() {
    let malformed_duplicate = r#"{"jsonrpc":"2.0","x":1,"x":2,"tail":"\q"}"#;
    let malformed_depth = format!(
        "{}0{}x",
        "[".repeat(oneagent_protocol::MAX_JSON_NESTING_DEPTH + 1),
        "]".repeat(oneagent_protocol::MAX_JSON_NESTING_DEPTH + 1)
    );
    let input = format!("{malformed_duplicate}\n{malformed_depth}\n");
    let (outcome, output) = run(input.as_bytes()).await;

    assert_eq!(outcome, Ok(McpStdioOutcome::EndOfInput));
    let lines = String::from_utf8(output).expect("protocol output must be UTF-8");
    assert_eq!(lines.lines().count(), 2);
    for line in lines.lines() {
        let response: Value = serde_json::from_str(line).expect("parse error response JSON");
        assert_eq!(response["error"]["code"], -32700);
        assert!(response.get("id").is_none());
    }
}

#[tokio::test]
async fn public_transport_enforces_utf8_size_and_complete_delimiters() {
    let (outcome, output) = run(&[0xff, b'\n']).await;
    assert_eq!(outcome, Err(McpStdioErrorKind::InvalidUtf8));
    assert!(output.is_empty());

    let oversized = vec![b'x'; MAX_MESSAGE_BYTES + 1];
    let (outcome, output) = run(&oversized).await;
    assert_eq!(outcome, Err(McpStdioErrorKind::FrameTooLarge));
    assert!(output.is_empty());

    let incomplete = request(4, "server/discover");
    let (outcome, output) = run(incomplete.as_bytes()).await;
    assert_eq!(outcome, Err(McpStdioErrorKind::IncompleteFrame));
    assert!(output.is_empty());

    let mut boundary = vec![b' '; MAX_MESSAGE_BYTES - 2];
    boundary.extend_from_slice(b"{}\n");
    let (outcome, output) = run(&boundary).await;
    assert_eq!(outcome, Ok(McpStdioOutcome::EndOfInput));
    assert!(!output.is_empty());

    let mut crlf_boundary = vec![b' '; MAX_MESSAGE_BYTES - 2];
    crlf_boundary.extend_from_slice(b"{}\r\n");
    let (outcome, output) = run(&crlf_boundary).await;
    assert_eq!(outcome, Ok(McpStdioOutcome::EndOfInput));
    assert!(!output.is_empty());
}

struct PublicFailingReader;

impl AsyncRead for PublicFailingReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        _buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::other("private sentinel")))
    }
}

struct PublicChunkedReader {
    input: Vec<u8>,
    position: usize,
    chunk_bytes: usize,
    polls: Arc<AtomicUsize>,
    dropped: Arc<AtomicBool>,
}

impl AsyncRead for PublicChunkedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        if self.position == self.input.len() {
            return Poll::Ready(Ok(()));
        }
        let available = self.input.len() - self.position;
        let count = available.min(self.chunk_bytes).min(buffer.remaining());
        let end = self.position + count;
        buffer.put_slice(&self.input[self.position..end]);
        self.position = end;
        Poll::Ready(Ok(()))
    }
}

impl Drop for PublicChunkedReader {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

struct PublicPendingReader {
    polls: Arc<AtomicUsize>,
    dropped: Arc<AtomicBool>,
}

impl AsyncRead for PublicPendingReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        _buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Poll::Pending
    }
}

impl Drop for PublicPendingReader {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

struct PublicDeferredShutdown(bool);

impl std::future::Future for PublicDeferredShutdown {
    type Output = Result<(), Infallible>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 {
            Poll::Ready(Ok(()))
        } else {
            self.0 = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

enum PublicWriterFailure {
    Write,
    Flush(Vec<u8>),
}

impl AsyncWrite for PublicWriterFailure {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            Self::Write => Poll::Ready(Err(io::Error::other("private sentinel"))),
            Self::Flush(bytes) => {
                bytes.extend_from_slice(buffer);
                Poll::Ready(Ok(buffer.len()))
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &*self {
            Self::Write => Poll::Ready(Ok(())),
            Self::Flush(_) => Poll::Ready(Err(io::Error::other("private sentinel"))),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn public_transport_closes_cancellation_and_io_failure_matrix() {
    let transport = McpStdioTransport::default();
    let mut empty = &b""[..];
    let mut output = Vec::new();
    assert_eq!(
        transport
            .run(&mut empty, &mut output, ready(Ok::<(), io::Error>(())))
            .await,
        Ok(McpStdioOutcome::Cancelled)
    );

    let mut empty = &b""[..];
    let error = transport
        .run(
            &mut empty,
            &mut output,
            ready(Err::<(), _>(io::Error::other("private sentinel"))),
        )
        .await
        .expect_err("shutdown-source failure must terminate");
    assert_eq!(error.kind(), McpStdioErrorKind::Shutdown);
    assert!(!error.to_string().contains("sentinel"));

    let mut reader = PublicFailingReader;
    let error = transport
        .run(
            &mut reader,
            &mut output,
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("read failure must terminate");
    assert_eq!(error.kind(), McpStdioErrorKind::Read);

    let input = format!("{}\n", request(5, "server/discover"));
    let mut reader = input.as_bytes();
    let mut writer = PublicWriterFailure::Write;
    let error = transport
        .run(
            &mut reader,
            &mut writer,
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("write failure must terminate");
    assert_eq!(error.kind(), McpStdioErrorKind::Write);

    let mut reader = input.as_bytes();
    let mut writer = PublicWriterFailure::Flush(Vec::new());
    let error = transport
        .run(
            &mut reader,
            &mut writer,
            pending::<Result<(), Infallible>>(),
        )
        .await
        .expect_err("flush failure must terminate");
    assert_eq!(error.kind(), McpStdioErrorKind::Flush);
}

#[tokio::test]
async fn public_transport_handles_partial_reads_escaped_newlines_and_releases_streams() {
    let input = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "server/discover",
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
                "io.modelcontextprotocol/clientCapabilities": {}
            },
            "extension": "first\nsecond"
        }
    })
    .to_string()
        + "\n";
    assert!(!input[..input.len() - 1].contains('\n'));

    let polls = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicBool::new(false));
    let mut reader = PublicChunkedReader {
        input: input.into_bytes(),
        position: 0,
        chunk_bytes: 1,
        polls: Arc::clone(&polls),
        dropped: Arc::clone(&dropped),
    };
    let mut output = Vec::new();
    let outcome = McpStdioTransport::default()
        .run(
            &mut reader,
            &mut output,
            pending::<Result<(), Infallible>>(),
        )
        .await;
    assert_eq!(outcome, Ok(McpStdioOutcome::EndOfInput));
    assert!(polls.load(Ordering::SeqCst) > 1);
    let lines = String::from_utf8(output).expect("protocol output must be UTF-8");
    assert_eq!(lines.lines().count(), 1);
    assert_eq!(
        serde_json::from_str::<Value>(lines.trim_end()).expect("discovery response")["id"],
        6
    );
    drop(reader);
    assert!(dropped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn public_transport_cancels_a_pending_reader_without_retained_work() {
    let polls = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicBool::new(false));
    let mut reader = PublicPendingReader {
        polls: Arc::clone(&polls),
        dropped: Arc::clone(&dropped),
    };
    let mut output = Vec::new();
    let outcome = McpStdioTransport::default()
        .run(&mut reader, &mut output, PublicDeferredShutdown(false))
        .await;
    assert_eq!(outcome, Ok(McpStdioOutcome::Cancelled));
    assert_eq!(polls.load(Ordering::SeqCst), 1);
    assert!(output.is_empty());
    drop(reader);
    assert!(dropped.load(Ordering::SeqCst));
}
