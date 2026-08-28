use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use oneagent_protocol::{
    MAX_MESSAGE_BYTES, MCP_PROTOCOL_VERSION_2025_06_18, MCP_PROTOCOL_VERSION_2025_11_25,
    PROTOCOL_VERSION,
};
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

fn codex_initialize() -> String {
    include_str!("../../../tests/fixtures/mcp/external-client-compatibility/codex-initialize.json")
        .trim_end()
        .to_owned()
}

fn cursor_initialize() -> String {
    include_str!("../../../tests/fixtures/mcp/external-client-compatibility/cursor-initialize.json")
        .trim_end()
        .to_owned()
}

fn legacy_request(id: u64, method: &str, params: Option<&Value>) -> String {
    let mut request = json!({"jsonrpc": "2.0", "id": id, "method": method});
    if let Some(params) = params {
        request
            .as_object_mut()
            .expect("legacy request object")
            .insert("params".to_owned(), params.clone());
    }
    request.to_string()
}

fn legacy_lifecycle_input(initialize: &str, separator: &str) -> String {
    let initialized = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let cancelled = r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":4,"reason":"test"}}"#;
    let exit = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    [
        legacy_request(1, "tools/list", None),
        initialize.to_owned(),
        legacy_request(2, "tools/list", None),
        initialized.to_owned(),
        cancelled.to_owned(),
        legacy_request(3, "tools/list", None),
        legacy_request(
            4,
            "tools/call",
            Some(&json!({"name": "oneagent.graph", "arguments": {}})),
        ),
        legacy_request(
            5,
            "tools/call",
            Some(&json!({
                "name": "oneagent.graph",
                "arguments": {"extra": true}
            })),
        ),
        legacy_request(6, "unknown/method", None),
        initialize.to_owned(),
        legacy_request(8, "shutdown", None),
        exit.to_owned(),
        "{".to_owned(),
        legacy_request(9, "ping", None),
    ]
    .join(separator)
        + separator
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
) -> [Value; 7] {
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
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "e", "kinds": ["module", "procedure", "function", "query"], "limit": 2
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
                "oneagent.symbols",
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
async fn public_mcp_process_runs_exact_codex_and_cursor_lifecycles_repeatably() {
    for (initialize, version, separator) in [
        (codex_initialize(), MCP_PROTOCOL_VERSION_2025_06_18, "\n"),
        (cursor_initialize(), MCP_PROTOCOL_VERSION_2025_11_25, "\r\n"),
    ] {
        let input = legacy_lifecycle_input(&initialize, separator);
        let first = run_process(input.as_bytes()).await;
        let repeated = run_process(input.as_bytes()).await;
        assert!(first.status.success());
        assert!(first.stderr.is_empty());
        assert_eq!(first.stdout, repeated.stdout);
        assert_eq!(first.stderr, repeated.stderr);
        assert_eq!(first.status, repeated.status);

        let stdout = String::from_utf8(first.stdout).expect("stdout must be UTF-8 JSON lines");
        assert!(stdout.ends_with('\n'));
        let responses = stdout
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("legacy response JSON"))
            .collect::<Vec<_>>();
        assert_eq!(responses.len(), 11);
        assert_eq!(responses[0]["id"], 1);
        assert_eq!(responses[0]["error"]["code"], -32002);
        assert_eq!(responses[1]["id"], 0);
        assert_eq!(responses[1]["result"]["protocolVersion"], version);
        assert_eq!(responses[1]["result"]["capabilities"], json!({"tools": {}}));
        assert_eq!(responses[1]["result"]["serverInfo"]["name"], "oneagent");
        assert!(responses[1]["result"].get("resultType").is_none());
        assert_eq!(responses[2]["id"], 2);
        assert_eq!(responses[2]["error"]["code"], -32002);

        let listed = &responses[3];
        assert_eq!(listed["id"], 3);
        assert_eq!(
            listed["result"]["tools"]
                .as_array()
                .expect("legacy tool catalog")
                .iter()
                .map(|tool| tool["name"].as_str().expect("tool name"))
                .collect::<Vec<_>>(),
            [
                "oneagent.context",
                "oneagent.diagnostics",
                "oneagent.graph",
                "oneagent.impact",
                "oneagent.query",
                "oneagent.symbols",
                "oneagent.validation"
            ]
        );
        for forbidden in ["resultType", "ttlMs", "cacheScope", "nextCursor"] {
            assert!(listed["result"].get(forbidden).is_none());
        }

        assert_eq!(responses[4]["id"], 4);
        assert!(responses[4]["result"].get("structuredContent").is_some());
        assert!(responses[4]["result"].get("isError").is_none());
        assert!(responses[4]["result"].get("resultType").is_none());
        assert_eq!(responses[5]["id"], 5);
        assert_eq!(responses[5]["result"]["isError"], true);
        assert_eq!(
            responses[5]["result"]["structuredContent"]["code"],
            "invalid_arguments"
        );
        assert!(responses[5]["result"].get("resultType").is_none());
        assert_eq!(responses[6]["error"]["code"], -32601);
        assert_eq!(responses[6]["id"], 6);
        assert_eq!(responses[7]["error"]["code"], -32600);
        assert_eq!(responses[7]["id"], 0);
        assert_eq!(responses[8]["error"]["code"], -32601);
        assert_eq!(responses[8]["id"], 8);
        assert_eq!(responses[9]["error"]["code"], -32700);
        assert!(responses[9].get("id").is_none());
        assert_eq!(responses[10]["id"], 9);
        assert_eq!(responses[10]["result"], json!({}));
    }
}

#[tokio::test]
async fn public_mcp_process_falls_back_unknown_legacy_version_with_string_id() {
    let initialize = json!({
        "jsonrpc": "2.0",
        "id": "fallback-request",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "fallback-client", "version": "1"}
        }
    });
    let input = format!(
        "{initialize}\n{}\n{}\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#,
        legacy_request(60, "ping", None)
    );
    let output = run_process(input.as_bytes()).await;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let responses = String::from_utf8(output.stdout)
        .expect("fallback stdout must be UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("fallback response JSON"))
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], "fallback-request");
    assert_eq!(
        responses[0]["result"]["protocolVersion"],
        MCP_PROTOCOL_VERSION_2025_11_25
    );
    assert_eq!(responses[1]["id"], 60);
    assert_eq!(responses[1]["result"], json!({}));
}

#[tokio::test]
async fn public_mcp_process_keeps_two_client_sessions_isolated() {
    let codex_input = format!(
        "{}\n{}\n{}\n",
        codex_initialize(),
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        legacy_request(61, "ping", None)
    );
    let cursor_input = format!(
        "{}\r\n{}\r\n{}\r\n",
        cursor_initialize(),
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        legacy_request(62, "ping", None)
    );
    let (codex, cursor) = tokio::join!(
        run_process(codex_input.as_bytes()),
        run_process(cursor_input.as_bytes())
    );
    for output in [&codex, &cursor] {
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
    let codex_responses = String::from_utf8(codex.stdout)
        .expect("Codex stdout must be UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("Codex response JSON"))
        .collect::<Vec<_>>();
    let cursor_responses = String::from_utf8(cursor.stdout)
        .expect("Cursor stdout must be UTF-8")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("Cursor response JSON"))
        .collect::<Vec<_>>();
    assert_eq!(codex_responses.len(), 2);
    assert_eq!(cursor_responses.len(), 2);
    assert_eq!(
        codex_responses[0]["result"]["protocolVersion"],
        MCP_PROTOCOL_VERSION_2025_06_18
    );
    assert_eq!(codex_responses[1]["id"], 61);
    assert_eq!(
        cursor_responses[0]["result"]["protocolVersion"],
        MCP_PROTOCOL_VERSION_2025_11_25
    );
    assert_eq!(cursor_responses[1]["id"], 62);
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
    frames.push(request_with_fields(
        22,
        "tools/call",
        &json!({"name": "oneagent.context", "arguments": {
            "configurationId": configuration_id, "nodeId": node_id, "budgetBytes": 1
        }}),
    ));
    frames.push(request_with_fields(
        23,
        "tools/call",
        &json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id, "operation": "relations", "nodeId": node_id,
            "edgeKinds": ["calls", "calls"]
        }}),
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
    assert_semantic_responses(&responses);
}

fn assert_semantic_responses(responses: &[Value]) {
    assert_eq!(responses.len(), 11);
    for response in &responses[..7] {
        assert!(response["result"].get("isError").is_none(), "{response}");
        assert!(response["result"].get("structuredContent").is_some());
    }
    let symbols = &responses[6]["result"]["structuredContent"];
    assert_eq!(symbols["total"], 5);
    assert_eq!(
        symbols["results"].as_array().expect("symbol results").len(),
        2
    );
    assert!(symbols.to_string().contains("Module.bsl"));
    assert!(
        !symbols
            .to_string()
            .contains(fixture_root().to_str().expect("UTF-8 fixture path"))
    );
    assert_eq!(responses[7]["result"]["isError"], true);
    assert_eq!(
        responses[8],
        json!({
            "jsonrpc": "2.0",
            "id": 21,
            "error": {"code": -32602, "message": "Invalid params"}
        })
    );
    for response in &responses[9..] {
        assert_eq!(response["result"]["isError"], true);
        assert_eq!(
            response["result"]["structuredContent"]["code"],
            "invalid_arguments"
        );
    }
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
async fn public_mcp_process_symbols_validate_arguments_before_lookup() {
    let invalid = request_with_fields(
        59,
        "tools/call",
        &json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "configurationId": "missing", "limit": 0
        }}),
    );
    let valid = request_with_fields(
        60,
        "tools/call",
        &json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "configurationId": "missing", "limit": 1
        }}),
    );
    let input = format!("{invalid}\n{valid}\n");
    let output = run_process(input.as_bytes()).await;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let responses = String::from_utf8(output.stdout)
        .expect("stdout must be UTF-8 JSON lines")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("symbol response JSON"))
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    assert_eq!(
        responses[0]["result"]["structuredContent"]["code"],
        "invalid_arguments"
    );
    assert_eq!(
        responses[1]["result"]["structuredContent"]["code"],
        "not_found"
    );
}

#[tokio::test]
async fn public_mcp_process_symbols_cover_all_supported_kinds_and_source_formats() {
    let temporary = tempdir().expect("temporary symbol Workspace must be created");
    copy_tree(&fixture_root(), temporary.path());
    for path in [
        temporary
            .path()
            .join("designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl"),
        temporary
            .path()
            .join("edt/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl"),
    ] {
        let mut source = fs::read_to_string(&path).expect("module source must be readable");
        source.push_str(
            "\nProcedure SearchProcedure()\nEndProcedure\n\nFunction SearchFunction()\nEndFunction\n\nФункция Поиск()\nКонецФункции\n",
        );
        fs::write(path, source).expect("extended module source must be written");
    }
    let edt_module = temporary
        .path()
        .join("edt/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl");
    let mut source = fs::read_to_string(&edt_module).expect("EDT module must be readable");
    source.push_str(
        "\nProcedure BuildSearchQuery()\nSearchQuery = New Query(\"SELECT Ref FROM Catalog.MissingRuntimeFixture\");\nEndProcedure\n",
    );
    fs::write(edt_module, source).expect("EDT Query source must be written");

    let calls = [
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "o", "kinds": ["module"], "limit": 100
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "SearchProcedure", "kinds": ["procedure"], "limit": 100
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "searchfunction", "kinds": ["function"], "limit": 100
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "SearchQuery", "kinds": ["query"], "limit": 100
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "поиск", "kinds": ["function"], "limit": 100
        }}),
    ];
    let input = format!(
        "{}\n",
        calls
            .iter()
            .enumerate()
            .map(|(offset, fields)| request_with_fields(
                60 + u64::try_from(offset).expect("small offset"),
                "tools/call",
                fields
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let output = run_process_in(temporary.path(), input.as_bytes()).await;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let responses = String::from_utf8(output.stdout)
        .expect("stdout must be UTF-8 JSON lines")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("symbol response JSON"))
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 5);
    assert_eq!(responses[0]["result"]["structuredContent"]["total"], 2);
    assert_eq!(responses[1]["result"]["structuredContent"]["total"], 2);
    assert_eq!(responses[2]["result"]["structuredContent"]["total"], 2);
    assert_eq!(responses[3]["result"]["structuredContent"]["total"], 1);
    assert_eq!(responses[4]["result"]["structuredContent"]["total"], 2);
    let duplicate_name_ids = responses[2]["result"]["structuredContent"]["results"]
        .as_array()
        .expect("duplicate-name function results")
        .iter()
        .map(|result| {
            result["configurationId"]
                .as_str()
                .expect("configuration ID")
        })
        .collect::<Vec<_>>();
    assert!(duplicate_name_ids.windows(2).all(|pair| pair[0] < pair[1]));
    for response in responses {
        let serialized = response["result"]["structuredContent"].to_string();
        assert!(!serialized.contains(temporary.path().to_str().expect("UTF-8 temp path")));
        assert!(response["result"].get("isError").is_none(), "{response}");
    }
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
