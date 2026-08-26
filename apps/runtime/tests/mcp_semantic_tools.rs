use std::fs;
use std::path::Path;

use oneagent_protocol::{McpServer, PROTOCOL_VERSION, encode_response};
use oneagent_runtime::{WorkspaceSnapshotBuilder, semantic_server};
use serde_json::{Value, json};
use tempfile::{TempDir, tempdir};

fn fixture_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workspace_service")
        .leak()
}

fn request(id: u64, method: &str, fields: &Value) -> String {
    let mut params = fields
        .as_object()
        .expect("fields must be an object")
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

fn reordered_request(id: u64, method: &str, fields: &Value) -> String {
    let value = serde_json::from_str::<Value>(&request(id, method, fields))
        .expect("canonical request must be JSON");
    let mut output = String::new();
    encode_reordered(&value, &mut output);
    output
}

fn encode_reordered(value: &Value, output: &mut String) {
    match value {
        Value::Object(fields) => {
            output.push('{');
            for (index, (key, value)) in fields.iter().rev().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key).expect("object key must encode"));
                output.push(':');
                encode_reordered(value, output);
            }
            output.push('}');
        }
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                encode_reordered(value, output);
            }
            output.push(']');
        }
        _ => output.push_str(&serde_json::to_string(value).expect("scalar must encode")),
    }
}

async fn dispatch(server: &McpServer, input: &str) -> Value {
    let response = server
        .dispatch(input)
        .await
        .expect("request must produce a response");
    serde_json::from_slice(&encode_response(&response).expect("response must encode"))
        .expect("response must be JSON")
}

fn assert_catalog(listed: &Value) {
    let names = listed["result"]["tools"]
        .as_array()
        .expect("tool array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "oneagent.context",
            "oneagent.diagnostics",
            "oneagent.graph",
            "oneagent.impact",
            "oneagent.query",
            "oneagent.validation"
        ]
    );
    assert!(
        listed["result"]["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .all(|tool| tool["annotations"]
                == json!({
                    "readOnlyHint": true,
                    "destructiveHint": false,
                    "idempotentHint": true,
                    "openWorldHint": false
                }))
    );
    let query = listed["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|tool| tool["name"] == "oneagent.query")
        .expect("query definition");
    assert_eq!(
        query["inputSchema"]["properties"]["edgeKinds"]["items"]["enum"],
        json!([
            "contains",
            "calls",
            "references",
            "reads",
            "writes",
            "grants",
            "includes",
            "extends",
            "depends_on",
            "opens",
            "triggers"
        ])
    );
}

fn tool_cases(
    configuration_id: &str,
    node_id: &str,
    diagnostic_configuration: &str,
    current_configuration: &str,
) -> Vec<Value> {
    vec![
        json!({"name": "oneagent.graph", "arguments": {"limit": 1}}),
        json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id, "operation": "node", "nodeId": node_id
        }}),
        json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id, "operation": "relations", "nodeId": node_id,
            "direction": "both",
            "edgeKinds": [
                "contains", "calls", "references", "reads", "writes", "grants",
                "includes", "extends", "depends_on", "opens", "triggers"
            ],
            "limit": 1
        }}),
        json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id, "operation": "traverse", "nodeId": node_id,
            "direction": "both", "maxDepth": 1, "limit": 2
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
async fn public_semantic_tools_are_truthful_policy_gated_and_repeatable() {
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(fixture_root())
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
    let server = semantic_server(snapshot).expect("fixed semantic server must build");

    let discovery = dispatch(&server, &request(1, "server/discover", &json!({}))).await;
    assert_eq!(discovery["result"]["capabilities"], json!({"tools": {}}));

    let listed = dispatch(&server, &request(2, "tools/list", &json!({}))).await;
    assert_catalog(&listed);
    let cases = tool_cases(
        &configuration_id,
        &node_id,
        &diagnostic_configuration,
        &current_configuration,
    );
    for (offset, fields) in cases.iter().enumerate() {
        let first = dispatch(
            &server,
            &request(
                10 + u64::try_from(offset).expect("small offset"),
                "tools/call",
                fields,
            ),
        )
        .await;
        let repeated = dispatch(
            &server,
            &request(
                20 + u64::try_from(offset).expect("small offset"),
                "tools/call",
                fields,
            ),
        )
        .await;
        let reordered = dispatch(
            &server,
            &reordered_request(
                30 + u64::try_from(offset).expect("small offset"),
                "tools/call",
                fields,
            ),
        )
        .await;
        assert!(first["result"].get("isError").is_none());
        assert_eq!(
            first["result"]["structuredContent"],
            repeated["result"]["structuredContent"]
        );
        assert_eq!(
            first["result"]["structuredContent"],
            reordered["result"]["structuredContent"]
        );
        let text = first["result"]["content"][0]["text"]
            .as_str()
            .expect("tool result text");
        assert_eq!(
            serde_json::from_str::<Value>(text).expect("tool result text must be JSON"),
            first["result"]["structuredContent"]
        );
        let serialized = first["result"]["structuredContent"].to_string();
        assert!(!serialized.contains(fixture_root().to_str().expect("UTF-8 fixture path")));
        assert!(!serialized.contains("provenance"));
        assert!(!serialized.contains("Configuration.xml"));
        assert_projection(offset, &first["result"]["structuredContent"]);
    }
}

fn assert_projection(offset: usize, content: &Value) {
    match offset {
        2 => {
            let relation = &content["relations"][0];
            assert!(relation["edgeId"].is_string());
            assert!(relation["edgeKind"].is_string());
            assert!(relation["relatedNode"]["id"].is_string());
        }
        3 => {
            let nodes = content["nodes"].as_array().expect("traversal nodes");
            assert!(nodes[0]["viaEdgeId"].is_null());
            assert!(
                nodes
                    .iter()
                    .skip(1)
                    .all(|node| node["viaEdgeId"].is_string())
            );
        }
        6 => {
            let affected = content["affectedNodes"].as_array().expect("affected nodes");
            assert!(!affected.is_empty());
            assert!(affected.iter().all(|node| node["reasons"].is_array()));
        }
        7 => {
            let items = content["items"].as_array().expect("context items");
            assert!(
                items
                    .iter()
                    .all(|item| item["reason"].is_string() && item["relations"].is_array())
            );
            assert!(items.iter().any(|item| item["reason"] == "related"));
        }
        _ => {}
    }
}

#[tokio::test]
async fn public_semantic_tools_fail_closed_at_argument_and_lookup_boundaries() {
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(fixture_root())
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
    let server = semantic_server(snapshot).expect("fixed semantic server must build");
    for fields in [
        json!({"name": "oneagent.validation", "arguments": {
            "configurationId": configuration_id, "limit": 0
        }}),
        json!({"name": "oneagent.validation", "arguments": {
            "configurationId": configuration_id, "extra": true
        }}),
        json!({"name": "oneagent.validation", "arguments": {
            "configurationId": "missing", "limit": 1
        }}),
        json!({"name": "oneagent.impact", "arguments": {
            "previousConfigurationId": configuration_id,
            "currentConfigurationId": configuration_id
        }}),
        json!({"name": "oneagent.context", "arguments": {
            "configurationId": configuration_id,
            "nodeId": node_id,
            "maxCandidates": 129
        }}),
        json!({"name": "oneagent.context", "arguments": {
            "configurationId": configuration_id,
            "nodeId": "missing",
            "budgetBytes": 4096
        }}),
        json!({"name": "oneagent.context", "arguments": {
            "configurationId": configuration_id,
            "nodeId": node_id,
            "budgetBytes": 1
        }}),
        json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id,
            "operation": "relations",
            "nodeId": node_id,
            "edgeKinds": ["calls", "calls"]
        }}),
    ] {
        let response = dispatch(&server, &request(30, "tools/call", &fields)).await;
        assert_eq!(response["result"]["isError"], true);
        assert!(
            matches!(
                response["result"]["structuredContent"]["code"].as_str(),
                Some("invalid_arguments" | "not_found")
            ),
            "{response}"
        );
    }
}

#[tokio::test]
async fn public_semantic_tools_enforce_exact_and_one_over_bounds() {
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(fixture_root())
        .expect("mixed fixture must build");
    let configuration_id = snapshot.configurations()[0]
        .configuration_id()
        .as_str()
        .to_owned();
    let current_configuration = snapshot.configurations()[1]
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
    let server = semantic_server(snapshot).expect("fixed semantic server must build");

    let exact = [
        json!({"name": "oneagent.graph", "arguments": {"limit": 100}}),
        json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id, "operation": "traverse", "nodeId": node_id,
            "maxDepth": 4, "limit": 100
        }}),
        json!({"name": "oneagent.validation", "arguments": {
            "configurationId": configuration_id, "limit": 100
        }}),
        json!({"name": "oneagent.diagnostics", "arguments": {
            "configurationId": configuration_id, "limit": 100
        }}),
        json!({"name": "oneagent.impact", "arguments": {
            "previousConfigurationId": configuration_id,
            "currentConfigurationId": current_configuration,
            "maxDepth": 4, "limit": 100
        }}),
        json!({"name": "oneagent.context", "arguments": {
            "configurationId": configuration_id, "nodeId": node_id,
            "maxDepth": 4, "maxCandidates": 128, "budgetBytes": 32768
        }}),
    ];
    for fields in &exact {
        let response = dispatch(&server, &request(40, "tools/call", fields)).await;
        assert!(response["result"].get("isError").is_none(), "{response}");
    }

    let one_over = [
        json!({"name": "oneagent.graph", "arguments": {"limit": 101}}),
        json!({"name": "oneagent.query", "arguments": {
            "configurationId": configuration_id, "operation": "traverse", "nodeId": node_id,
            "maxDepth": 5
        }}),
        json!({"name": "oneagent.validation", "arguments": {
            "configurationId": configuration_id, "limit": 101
        }}),
        json!({"name": "oneagent.diagnostics", "arguments": {
            "configurationId": configuration_id, "limit": 101
        }}),
        json!({"name": "oneagent.impact", "arguments": {
            "previousConfigurationId": configuration_id,
            "currentConfigurationId": current_configuration,
            "maxDepth": 5
        }}),
        json!({"name": "oneagent.context", "arguments": {
            "configurationId": configuration_id, "nodeId": node_id,
            "budgetBytes": 32769
        }}),
    ];
    for fields in &one_over {
        let response = dispatch(&server, &request(41, "tools/call", fields)).await;
        assert_eq!(
            response["result"]["structuredContent"]["code"], "invalid_arguments",
            "{response}"
        );
    }
}

#[tokio::test]
async fn public_semantic_tools_cover_empty_and_oversized_results() {
    let empty = tempdir().expect("empty workspace root");
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(empty.path())
        .expect("empty workspace must build");
    let server = semantic_server(snapshot).expect("empty semantic server must build");
    let graph = dispatch(
        &server,
        &request(
            50,
            "tools/call",
            &json!({"name": "oneagent.graph", "arguments": {}}),
        ),
    )
    .await;
    assert_eq!(graph["result"]["structuredContent"]["total"], 0);

    let large = large_workspace();
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(large.path())
        .expect("large workspace must build");
    let server = semantic_server(snapshot).expect("large semantic server must build");
    let response = dispatch(
        &server,
        &request(
            51,
            "tools/call",
            &json!({"name": "oneagent.graph", "arguments": {"limit": 100}}),
        ),
    )
    .await;
    assert_eq!(response["result"]["isError"], true);
    assert_eq!(
        response["result"]["structuredContent"]["code"],
        "result_too_large"
    );
}

fn large_workspace() -> TempDir {
    let root = tempdir().expect("large workspace root");
    let configuration = fs::read_to_string(fixture_root().join("designer/Configuration.xml"))
        .expect("Designer Configuration fixture");
    let dump = fs::read(fixture_root().join("designer/ConfigDumpInfo.xml"))
        .expect("Designer dump marker fixture");
    for index in 0..80 {
        let project = root.path().join(format!("project-{index:03}"));
        fs::create_dir(&project).expect("large project directory");
        let identifier = format!("00000000-0000-0000-0000-{index:012}");
        let name = format!("Configuration{index}{}", "x".repeat(900));
        let source = configuration
            .replace("408a41e7-907a-4fb3-8999-83d1e8b6e093", &identifier)
            .replace("DNSWorldEdition", &name);
        fs::write(project.join("Configuration.xml"), source).expect("large Configuration source");
        fs::write(project.join("ConfigDumpInfo.xml"), &dump).expect("large dump marker");
    }
    root
}
