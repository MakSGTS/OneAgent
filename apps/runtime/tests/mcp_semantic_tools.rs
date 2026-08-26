use std::path::Path;

use oneagent_protocol::{McpServer, PROTOCOL_VERSION, encode_response};
use oneagent_runtime::{WorkspaceSnapshotBuilder, semantic_server};
use serde_json::{Value, json};

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
            .all(|tool| tool["annotations"]["readOnlyHint"] == true
                && tool["annotations"]["destructiveHint"] == false)
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
            "direction": "both", "limit": 1
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
        assert!(first["result"].get("isError").is_none());
        assert_eq!(
            first["result"]["structuredContent"],
            repeated["result"]["structuredContent"]
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
