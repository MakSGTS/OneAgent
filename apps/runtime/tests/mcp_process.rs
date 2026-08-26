use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use oneagent_protocol::{MAX_MESSAGE_BYTES, PROTOCOL_VERSION};
use oneagent_runtime::WorkspaceSnapshotBuilder;
use serde_json::{Value, json};
use tempfile::tempdir;
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
    let root = tempdir().expect("temporary empty Workspace must be created");
    run_process_in(root.path(), input).await
}

async fn run_process_in(root: &Path, input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oneagent-mcp"))
        .current_dir(root)
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

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_service")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("fixture destination must be created");
    let mut entries = fs::read_dir(source)
        .expect("fixture source must be readable")
        .map(|entry| entry.expect("fixture entry must be readable"))
        .collect::<Vec<_>>();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture file type must be readable")
            .is_dir()
        {
            copy_tree(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("fixture file must be copied");
        }
    }
}

fn semantic_calls(
    configuration_id: &str,
    node_id: &str,
    diagnostic_configuration: &str,
    current_configuration: &str,
) -> [Value; 6] {
    [
        json!({"name": "oneagent.graph", "arguments": {"limit": 1}}),
        json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id, "operation": "node", "nodeId": node_id
        }}),
        json!({"name": "oneagent.validation", "arguments": {
            "configurationId": configuration_id, "limit": 1
        }}),
        json!({"name": "oneagent.diagnostics", "arguments": {
            "configurationId": diagnostic_configuration, "limit": 1
        }}),
        json!({"name": "oneagent.impact", "arguments": {
            "previousConfigurationId": configuration_id,
            "currentConfigurationId": current_configuration,
            "maxDepth": 1,
            "limit": 2
        }}),
        json!({"name": "oneagent.context", "arguments": {
            "configurationId": configuration_id,
            "nodeId": node_id,
            "direction": "both",
            "maxDepth": 1,
            "maxCandidates": 4,
            "budgetBytes": 4096
        }}),
    ]
}

#[tokio::test]
async fn public_mcp_process_serves_requests_and_exits_cleanly_on_eof() {
    let discover = request(1, "server/discover");
    let notification = r#"{"jsonrpc":"2.0","method":"server/discover"}"#;
    let list = request(2, "tools/list");
    let malformed = request_with_capabilities(3, "server/discover", &json!({"elicitation": 42}));
    let input = format!("{discover}\n{notification}\n{list}\n{malformed}\n");

    for _ in 0..2 {
        let output = run_process(input.as_bytes()).await;
        assert!(output.status.success());
        assert!(output.stderr.is_empty());

        let stdout = String::from_utf8(output.stdout).expect("stdout must be UTF-8 JSON lines");
        assert!(stdout.ends_with('\n'));
        let lines = stdout.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        let discovery: Value = serde_json::from_str(lines[0]).expect("discovery JSON");
        let listed: Value = serde_json::from_str(lines[1]).expect("tool list JSON");
        let malformed_error: Value = serde_json::from_str(lines[2]).expect("invalid params JSON");
        assert_eq!(
            discovery,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "resultType": "complete",
                    "supportedVersions": [PROTOCOL_VERSION],
                    "capabilities": {"tools": {}},
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
        assert_eq!(listed["jsonrpc"], "2.0");
        assert_eq!(listed["id"], 2);
        assert_eq!(
            listed["result"]["tools"]
                .as_array()
                .expect("tool catalog")
                .iter()
                .map(|tool| tool["name"].as_str().expect("tool name"))
                .collect::<Vec<_>>(),
            [
                "oneagent.context",
                "oneagent.diagnostics",
                "oneagent.graph",
                "oneagent.impact",
                "oneagent.query",
                "oneagent.validation"
            ]
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
async fn public_mcp_process_serves_every_semantic_tool_family_repeatably() {
    let root = fixture_root();
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(&root)
        .expect("mixed fixture must build");
    let configuration_id = snapshot.configurations()[0]
        .configuration_id()
        .as_str()
        .to_owned();
    let node_id = snapshot.configurations()[0]
        .graph()
        .nodes()
        .next()
        .expect("fixture graph must contain a node")
        .id()
        .as_str()
        .to_owned();
    let diagnostic_configuration = snapshot
        .configurations()
        .iter()
        .find(|configuration| !configuration.diagnostics().is_empty())
        .expect("fixture must contain diagnostics")
        .configuration_id()
        .as_str()
        .to_owned();
    let current_configuration = snapshot.configurations()[1]
        .configuration_id()
        .as_str()
        .to_owned();
    let calls = semantic_calls(
        &configuration_id,
        &node_id,
        &diagnostic_configuration,
        &current_configuration,
    );
    let mut frames = calls
        .iter()
        .enumerate()
        .map(|(offset, fields)| {
            request_with_fields(
                10 + u64::try_from(offset).expect("small offset"),
                "tools/call",
                fields,
            )
        })
        .collect::<Vec<_>>();
    frames.push(request_with_fields(
        20,
        "tools/call",
        &json!({"name": "oneagent.graph", "arguments": {"extra": true}}),
    ));
    frames.push(request_with_fields(
        21,
        "tools/call",
        &json!({"name": "oneagent.unknown", "arguments": {}}),
    ));
    let input = format!("{}\n", frames.join("\n"));

    let first = run_process_in(&root, input.as_bytes()).await;
    let repeated = run_process_in(&root, input.as_bytes()).await;
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, repeated.stdout);
    assert_eq!(first.stderr, repeated.stderr);

    let responses = String::from_utf8(first.stdout).expect("stdout must be UTF-8 JSON lines");
    let responses = responses
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("tool response JSON"))
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 8);
    for response in &responses[..6] {
        assert!(response["result"].get("isError").is_none(), "{response}");
        assert!(response["result"].get("structuredContent").is_some());
    }
    assert_eq!(responses[6]["result"]["isError"], true);
    assert_eq!(
        responses[7],
        json!({
            "jsonrpc": "2.0",
            "id": 21,
            "error": {"code": -32602, "message": "Invalid params"}
        })
    );
}

fn request_with_fields(id: u64, method: &str, fields: &Value) -> String {
    let mut params = fields
        .as_object()
        .expect("request fields must be an object")
        .clone();
    params.insert(
        "_meta".to_owned(),
        json!({
            "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
            "io.modelcontextprotocol/clientCapabilities": {}
        }),
    );
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

#[tokio::test]
async fn public_mcp_process_reports_redacted_workspace_start_failure() {
    let temporary = tempdir().expect("temporary conflicting Workspace must be created");
    copy_tree(&fixture_root().join("designer"), temporary.path());
    fs::copy(
        fixture_root().join("edt/.project"),
        temporary.path().join(".project"),
    )
    .expect("EDT marker must be copied");
    fs::create_dir_all(temporary.path().join("src/Configuration"))
        .expect("EDT Configuration directory must be created");
    fs::copy(
        fixture_root().join("edt/src/Configuration/Configuration.mdo"),
        temporary.path().join("src/Configuration/Configuration.mdo"),
    )
    .expect("EDT Configuration marker must be copied");

    let output = run_process_in(temporary.path(), b"").await;
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"oneagent-mcp: workspace build failure\n");
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
async fn public_mcp_process_does_not_retype_literal_arbitrary_number_token_objects() {
    let input = b"{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"server/discover\",\"params\":{\"_meta\":{\"io.modelcontextprotocol/protocolVersion\":\"2026-07-28\",\"io.modelcontextprotocol/clientCapabilities\":{},\"progressToken\":{\"$serde_json::private::Number\":\"1\"}}}}\n";
    let output = run_process(input).await;

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let response: Value = serde_json::from_slice(&output.stdout).expect("error response JSON");
    assert_eq!(
        response,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "error": {"code": -32602, "message": "Invalid params"}
        })
    );
}

#[tokio::test]
async fn public_mcp_process_preserves_syntax_precedence_after_structural_failures() {
    let duplicate = r#"{"jsonrpc":"2.0","x":1,"x":2,"tail":"\q"}"#;
    let depth = format!(
        "{}0{}x",
        "[".repeat(oneagent_protocol::MAX_JSON_NESTING_DEPTH + 1),
        "]".repeat(oneagent_protocol::MAX_JSON_NESTING_DEPTH + 1)
    );
    let input = format!("{duplicate}\n{depth}\n");
    let output = run_process(input.as_bytes()).await;

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let lines = String::from_utf8(output.stdout).expect("protocol output must be UTF-8");
    assert_eq!(lines.lines().count(), 2);
    for line in lines.lines() {
        let response: Value = serde_json::from_str(line).expect("parse error response JSON");
        assert_eq!(
            response,
            json!({
                "jsonrpc": "2.0",
                "error": {"code": -32700, "message": "Parse error"}
            })
        );
    }
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
