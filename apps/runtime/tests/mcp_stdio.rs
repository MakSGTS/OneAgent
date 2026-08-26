use std::convert::Infallible;
use std::future::{pending, ready};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use oneagent_protocol::{MAX_MESSAGE_BYTES, PROTOCOL_VERSION};
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
