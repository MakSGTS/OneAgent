use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use oneagent_runtime::workspace_root_uri;
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

const PROCESS_TIMEOUT: Duration = Duration::from_secs(5);

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
            "capabilities": {"general": {"positionEncodings": ["utf-16"]}},
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

async fn run_process(root: &Path, input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oneagent-lsp"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("LSP process must spawn");
    let mut stdin = child.stdin.take().expect("piped stdin must exist");
    stdin.write_all(input).await.expect("test input must write");
    stdin.flush().await.expect("test input must flush");
    drop(stdin);
    timeout(PROCESS_TIMEOUT, child.wait_with_output())
        .await
        .expect("LSP process must not hang")
        .expect("LSP process must be waitable")
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
            .expect("response length");
        let start = delimiter + 4;
        let end = start + length;
        values.push(serde_json::from_slice(&bytes[start..end]).expect("response JSON"));
        bytes = &bytes[end..];
    }
    values
}

#[tokio::test]
async fn public_lsp_process_completes_lifecycle_with_pure_framed_stdout() {
    let root = tempdir().expect("temporary Workspace must be created");
    let root_uri = workspace_root_uri(root.path()).expect("root URI must encode");
    let input = [
        frame(&initialize(&root_uri)),
        frame(&notification("initialized")),
        frame(&request(2, "shutdown")),
        frame(&notification("exit")),
    ]
    .concat();

    let first = run_process(root.path(), &input).await;
    let repeated = run_process(root.path(), &input).await;
    assert!(
        first.status.success(),
        "status={:?}, stdout={}, stderr={}",
        first.status,
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, repeated.stdout);
    assert_eq!(first.stderr, repeated.stderr);
    let responses = decode_frames(&first.stdout);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "oneagent");
    assert_eq!(
        responses[0]["result"]["capabilities"],
        json!({"positionEncoding": "utf-16", "textDocumentSync": 0})
    );
    assert_eq!(responses[1]["result"], Value::Null);
}

#[tokio::test]
async fn public_lsp_process_rejects_conflicting_root_and_early_exit() {
    let root = tempdir().expect("temporary Workspace must be created");
    let input = [
        frame(&initialize("file:///different")),
        frame(&notification("exit")),
    ]
    .concat();
    let output = run_process(root.path(), &input).await;
    assert!(!output.status.success());
    assert_eq!(output.stderr, b"oneagent-lsp: lifecycle failure\n");
    let responses = decode_frames(&output.stdout);
    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0]["error"],
        json!({"code": -32602, "message": "Invalid params"})
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains(root.path().to_string_lossy().as_ref())
    );
}

#[tokio::test]
async fn public_lsp_process_reports_bounded_terminal_categories() {
    let root = tempdir().expect("temporary Workspace must be created");
    let eof = run_process(root.path(), b"").await;
    assert!(!eof.status.success());
    assert!(eof.stdout.is_empty());
    assert_eq!(eof.stderr, b"oneagent-lsp: unexpected end of input\n");

    let malformed = run_process(root.path(), b"Content-Length: 2\n\n{}").await;
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    assert_eq!(malformed.stderr, b"oneagent-lsp: invalid header\n");
}
