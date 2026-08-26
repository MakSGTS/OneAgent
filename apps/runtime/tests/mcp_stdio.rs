use std::convert::Infallible;
use std::future::pending;

use oneagent_protocol::{MAX_MESSAGE_BYTES, PROTOCOL_VERSION};
use oneagent_runtime::{McpStdioErrorKind, McpStdioOutcome, McpStdioTransport};
use serde_json::{Value, json};

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
