use std::process::Stdio;
use std::time::Duration;

use oneagent_protocol::{MAX_MESSAGE_BYTES, PROTOCOL_VERSION};
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

const PROCESS_TIMEOUT: Duration = Duration::from_secs(5);

fn request(id: u64, method: &str) -> String {
    request_with_capabilities(id, method, &json!({}))
}

fn request_with_capabilities(id: u64, method: &str, capabilities: &Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
                "io.modelcontextprotocol/clientCapabilities": capabilities
            }
        }
    })
    .to_string()
}

async fn run_process(input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oneagent-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("MCP process must spawn");

    let mut stdin = child.stdin.take().expect("piped stdin must exist");
    stdin
        .write_all(input)
        .await
        .expect("test input must be written");
    stdin.flush().await.expect("test input must flush");
    drop(stdin);

    timeout(PROCESS_TIMEOUT, child.wait_with_output())
        .await
        .expect("MCP process must not hang")
        .expect("MCP process must be waitable")
}

#[tokio::test]
async fn public_mcp_process_serves_requests_and_exits_cleanly_on_eof() {
    let discover = request(1, "server/discover");
    let notification = r#"{"jsonrpc":"2.0","method":"server/discover"}"#;
    let unknown = request(2, "tools/list");
    let malformed = request_with_capabilities(3, "server/discover", &json!({"elicitation": 42}));
    let input = format!("{discover}\n{notification}\n{unknown}\n{malformed}\n");

    for _ in 0..2 {
        let output = run_process(input.as_bytes()).await;
        assert!(output.status.success());
        assert!(output.stderr.is_empty());

        let stdout = String::from_utf8(output.stdout).expect("stdout must be UTF-8 JSON lines");
        assert!(stdout.ends_with('\n'));
        let lines = stdout.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        let discovery: Value = serde_json::from_str(lines[0]).expect("discovery JSON");
        let method_error: Value = serde_json::from_str(lines[1]).expect("method error JSON");
        let malformed_error: Value = serde_json::from_str(lines[2]).expect("invalid params JSON");
        assert_eq!(
            discovery,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "resultType": "complete",
                    "supportedVersions": [PROTOCOL_VERSION],
                    "capabilities": {},
                    "_meta": {
                        "io.modelcontextprotocol/serverInfo": {
                            "name": "oneagent",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    },
                    "ttlMs": 0,
                    "cacheScope": "public"
                }
            })
        );
        assert_eq!(
            method_error,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "error": {"code": -32601, "message": "Method not found"}
            })
        );
        assert_eq!(
            malformed_error,
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "error": {"code": -32602, "message": "Invalid params"}
            })
        );
    }
}

#[tokio::test]
async fn public_mcp_process_classifies_overflow_exponent_id_as_invalid_request() {
    let input = b"{\"jsonrpc\":\"2.0\",\"id\":1e400,\"method\":\"server/discover\",\"params\":{\"_meta\":{\"io.modelcontextprotocol/protocolVersion\":\"2026-07-28\",\"io.modelcontextprotocol/clientCapabilities\":{}}}}\n";
    let output = run_process(input).await;

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let response: Value = serde_json::from_slice(&output.stdout).expect("error response JSON");
    assert_eq!(
        response,
        json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "Invalid Request"}
        })
    );
}

#[tokio::test]
async fn public_mcp_process_reports_only_bounded_terminal_diagnostics() {
    let invalid_utf8 = run_process(&[0xff, b'\n']).await;
    assert!(!invalid_utf8.status.success());
    assert!(invalid_utf8.stdout.is_empty());
    assert_eq!(invalid_utf8.stderr, b"oneagent-mcp: invalid UTF-8 frame\n");

    let incomplete = run_process(request(3, "server/discover").as_bytes()).await;
    assert!(!incomplete.status.success());
    assert!(incomplete.stdout.is_empty());
    assert_eq!(incomplete.stderr, b"oneagent-mcp: incomplete frame\n");

    let oversized = run_process(&vec![b'x'; MAX_MESSAGE_BYTES + 1]).await;
    assert!(!oversized.status.success());
    assert!(oversized.stdout.is_empty());
    assert_eq!(oversized.stderr, b"oneagent-mcp: frame too large\n");
}
