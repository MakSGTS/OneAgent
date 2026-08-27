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
            "oneagent.symbols",
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
    let symbols = listed["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|tool| tool["name"] == "oneagent.symbols")
        .expect("symbols definition");
    assert_eq!(symbols["inputSchema"]["required"], json!(["query"]));
    assert_eq!(
        symbols["inputSchema"]["properties"]["kinds"]["items"]["enum"],
        json!(["module", "procedure", "function", "query"])
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
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "e", "limit": 2
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
        8 => {
            let results = content["results"].as_array().expect("symbol results");
            assert_eq!(results.len(), 2);
            assert_eq!(content["total"], 5);
            assert_eq!(content["truncated"], true);
            assert!(results.iter().all(|result| {
                result["configurationId"].is_string()
                    && result["configurationName"].is_string()
                    && result["nodeId"].is_string()
                    && result["name"].is_string()
                    && result["kind"].is_string()
                    && result["location"]["path"].is_string()
            }));
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
        json!({"name": "oneagent.symbols", "arguments": {"query": ""}}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "kinds": []
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "kinds": ["module", "module"]
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "kinds": ["metadata"]
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "configurationId": "missing"
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "extra": true
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
async fn public_symbols_validate_all_arguments_before_lookup() {
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(fixture_root())
        .expect("mixed fixture must build");
    let server = semantic_server(snapshot).expect("fixed semantic server must build");

    for arguments in [
        json!({"query": "x", "configurationId": "missing", "limit": 0}),
        json!({"query": "x", "configurationId": "missing", "kinds": []}),
        json!({"query": "x", "configurationId": "missing", "kinds": ["module", "module"]}),
        json!({"query": "x", "configurationId": "missing", "kinds": ["metadata"]}),
        json!({"query": "", "configurationId": "missing"}),
        json!({"query": "x", "configurationId": "missing", "extra": true}),
    ] {
        let fields = json!({"name": "oneagent.symbols", "arguments": arguments});
        let response = dispatch(&server, &request(31, "tools/call", &fields)).await;
        assert_eq!(response["result"]["isError"], true, "{response}");
        assert_eq!(
            response["result"]["structuredContent"]["code"], "invalid_arguments",
            "{response}"
        );
    }

    let fields = json!({"name": "oneagent.symbols", "arguments": {
        "query": "x", "configurationId": "missing", "limit": 1
    }});
    let response = dispatch(&server, &request(32, "tools/call", &fields)).await;
    assert_eq!(response["result"]["isError"], true, "{response}");
    assert_eq!(
        response["result"]["structuredContent"]["code"], "not_found",
        "{response}"
    );
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
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x".repeat(256), "limit": 100
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
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x".repeat(257)
        }}),
        json!({"name": "oneagent.symbols", "arguments": {
            "query": "x", "limit": 101
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
async fn symbol_search_preserves_matching_filtering_ordering_and_locations() {
    let snapshot = WorkspaceSnapshotBuilder::new()
        .build(fixture_root())
        .expect("mixed fixture must build");
    let designer_id = snapshot
        .configurations()
        .iter()
        .find(|configuration| configuration.configuration_name().as_str() == "DNSWorldEdition")
        .expect("Designer fixture")
        .configuration_id()
        .as_str()
        .to_owned();
    let server = semantic_server(snapshot).expect("fixed semantic server must build");

    let all = dispatch(
        &server,
        &request(
            60,
            "tools/call",
            &json!({"name": "oneagent.symbols", "arguments": {
                "query": "e", "limit": 100
            }}),
        ),
    )
    .await;
    let results = all["result"]["structuredContent"]["results"]
        .as_array()
        .expect("symbol results");
    assert_eq!(results.len(), 5);
    assert_eq!(
        results
            .iter()
            .map(|result| result["name"].as_str().expect("symbol name"))
            .collect::<Vec<_>>(),
        [
            "DynamicSecurityOverridable",
            "FillSecurityCollection",
            "ObjectModule",
            "Query",
            "ReadMissingCatalog"
        ]
    );
    assert!(
        results.iter().any(|result| {
            result["kind"] == "module" && result["location"].get("span").is_none()
        })
    );
    assert!(results.iter().any(|result| {
        result["kind"] == "procedure"
            && result["location"]["span"]["start"]["line"].is_u64()
            && result["location"]["span"]["start"]["column"] == 1
    }));

    let filtered = dispatch(
        &server,
        &request(
            61,
            "tools/call",
            &json!({"name": "oneagent.symbols", "arguments": {
                "query": "SECURITY", "configurationId": designer_id,
                "kinds": ["procedure"]
            }}),
        ),
    )
    .await;
    assert_eq!(filtered["result"]["structuredContent"]["total"], 1);
    assert_eq!(
        filtered["result"]["structuredContent"]["results"][0]["name"],
        "FillSecurityCollection"
    );

    let whitespace = dispatch(
        &server,
        &request(
            62,
            "tools/call",
            &json!({"name": "oneagent.symbols", "arguments": {
                "query": " "
            }}),
        ),
    )
    .await;
    assert_eq!(
        whitespace["result"]["structuredContent"]["results"],
        json!([])
    );
    assert_eq!(whitespace["result"]["structuredContent"]["total"], 0);
    assert_eq!(
        whitespace["result"]["structuredContent"]["truncated"],
        false
    );
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
