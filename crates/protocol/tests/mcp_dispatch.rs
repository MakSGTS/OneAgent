use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use oneagent_protocol::{
    ErrorCode, McpServer, McpToolAnnotations, McpToolCallHandler, McpToolCallOutcome,
    McpToolDefinition, McpToolFuture, PROTOCOL_VERSION, Response, encode_response,
};
use serde_json::Map;
use serde_json::{Value, json};

fn request(id: &Value, method: &str, version: &str) -> String {
    request_with_capabilities(id, method, version, &json!({}))
}

fn request_with_capabilities(
    id: &Value,
    method: &str,
    version: &str,
    capabilities: &Value,
) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": version,
                "io.modelcontextprotocol/clientCapabilities": capabilities
            }
        }
    })
    .to_string()
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn dispatch_json(server: &McpServer, input: &str) -> Value {
    let response = block_on(server.dispatch(input)).expect("request must respond");
    serde_json::from_slice(&encode_response(&response).expect("response must encode"))
        .expect("response must be JSON")
}

fn tool_request(id: i64, method: &str, fields: &Value) -> String {
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

struct TestToolHandler;

impl McpToolCallHandler for TestToolHandler {
    fn call<'a>(&'a self, name: &'a str, arguments: &'a Map<String, Value>) -> McpToolFuture<'a> {
        Box::pin(async move {
            if arguments.get("fail") == Some(&Value::Bool(true)) {
                return McpToolCallOutcome::error(
                    "invalid_arguments",
                    "The arguments are invalid.",
                );
            }
            McpToolCallOutcome::Success(json!({"arguments": arguments, "tool": name}))
        })
    }
}

fn tool_server() -> McpServer {
    let schema = json!({
        "type": "object",
        "properties": {"fail": {"type": "boolean"}},
        "additionalProperties": false
    })
    .as_object()
    .expect("schema object")
    .clone();
    McpServer::with_tools(
        vec![
            McpToolDefinition::new(
                "oneagent.query",
                "Query semantic graph data.",
                schema.clone(),
                McpToolAnnotations::read_only(),
            )
            .expect("valid definition"),
            McpToolDefinition::new(
                "oneagent.graph",
                "Summarize semantic graph data.",
                schema,
                McpToolAnnotations::read_only(),
            )
            .expect("valid definition"),
        ],
        TestToolHandler,
    )
    .expect("valid catalog")
}

#[test]
fn public_discovery_is_exact_truthful_and_repeatable() {
    for id in [json!("discover-1"), json!(7)] {
        for _ in 0..3 {
            let server = McpServer::new();
            assert!(server.capabilities().is_empty());
            assert_eq!(
                server.registered_methods().collect::<Vec<_>>(),
                ["server/discover"]
            );
            let response =
                dispatch_json(&server, &request(&id, "server/discover", PROTOCOL_VERSION));
            assert_eq!(response["jsonrpc"], "2.0");
            assert_eq!(response["id"], id);
            assert_eq!(response["result"]["resultType"], "complete");
            assert_eq!(
                response["result"]["supportedVersions"],
                json!([PROTOCOL_VERSION])
            );
            assert_eq!(response["result"]["capabilities"], json!({}));
            assert_eq!(response["result"]["ttlMs"], 0);
            assert_eq!(response["result"]["cacheScope"], "public");
            assert_eq!(
                response["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
                "oneagent"
            );
            assert_eq!(
                response["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["version"],
                env!("CARGO_PKG_VERSION")
            );
            assert!(response["result"].get("tools").is_none());
            assert!(response["result"].get("instructions").is_none());
        }
    }
}

#[test]
fn public_dispatch_has_closed_version_method_and_notification_behavior() {
    let server = McpServer::new();

    let unsupported =
        block_on(server.dispatch(&request(&json!(1), "server/discover", "1900-01-01")))
            .expect("request must respond");
    let Response::Error(unsupported) = unsupported else {
        panic!("unsupported version must fail");
    };
    assert_eq!(unsupported.code(), ErrorCode::UnsupportedProtocolVersion);

    let unknown = block_on(server.dispatch(&request(&json!(2), "tools/list", PROTOCOL_VERSION)))
        .expect("request must respond");
    let Response::Error(unknown) = unknown else {
        panic!("unknown method must fail");
    };
    assert_eq!(unknown.code(), ErrorCode::MethodNotFound);

    let malformed_capability = block_on(server.dispatch(&request_with_capabilities(
        &json!(3),
        "server/discover",
        PROTOCOL_VERSION,
        &json!({"elicitation": 42}),
    )))
    .expect("malformed request must respond");
    let Response::Error(malformed_capability) = malformed_capability else {
        panic!("malformed capability must fail");
    };
    assert_eq!(malformed_capability.code(), ErrorCode::InvalidParams);
    assert_eq!(
        malformed_capability
            .id()
            .and_then(oneagent_protocol::RequestId::as_i64),
        Some(3)
    );
    assert!(malformed_capability.data().is_none());

    assert!(block_on(server.dispatch(r#"{"jsonrpc":"2.0","method":"server/discover"}"#)).is_none());
    assert!(
        block_on(server.dispatch(r#"{"jsonrpc":"2.0","method":"unknown","params":[]}"#)).is_none()
    );
}

#[test]
fn public_dispatch_enforces_exact_method_name_bounds_before_lookup() {
    let server = McpServer::new();
    let exact = "m".repeat(oneagent_protocol::MAX_METHOD_NAME_BYTES);
    let response = block_on(server.dispatch(&request(&json!(3), &exact, PROTOCOL_VERSION)))
        .expect("exact-bound request must respond");
    let Response::Error(error) = response else {
        panic!("unregistered exact-bound method must fail");
    };
    assert_eq!(error.code(), ErrorCode::MethodNotFound);

    let oversized = "m".repeat(oneagent_protocol::MAX_METHOD_NAME_BYTES + 1);
    let response = block_on(server.dispatch(&request(&json!(4), &oversized, PROTOCOL_VERSION)))
        .expect("oversized method must respond");
    let Response::Error(error) = response else {
        panic!("oversized method must fail");
    };
    assert_eq!(error.code(), ErrorCode::InvalidRequest);
}

#[test]
fn public_tool_catalog_is_truthful_ordered_and_bounded() {
    let server = tool_server();
    assert_eq!(
        server.capabilities(),
        json!({"tools": {}}).as_object().expect("object")
    );
    assert_eq!(
        server.registered_methods().collect::<Vec<_>>(),
        ["server/discover", "tools/call", "tools/list"]
    );
    let discovery = dispatch_json(
        &server,
        &request(&json!(10), "server/discover", PROTOCOL_VERSION),
    );
    assert_eq!(discovery["result"]["capabilities"], json!({"tools": {}}));

    let listed = dispatch_json(&server, &tool_request(11, "tools/list", &json!({})));
    assert_eq!(listed["result"]["tools"][0]["name"], "oneagent.graph");
    assert_eq!(listed["result"]["tools"][1]["name"], "oneagent.query");
    assert_eq!(
        listed["result"]["tools"][0]["inputSchema"]["type"],
        "object"
    );
    assert_eq!(
        listed["result"]["tools"][0]["annotations"]["readOnlyHint"],
        true
    );
    assert_eq!(listed["result"]["ttlMs"], 0);

    let invalid_cursor = dispatch_json(
        &server,
        &tool_request(12, "tools/list", &json!({"cursor": "next"})),
    );
    assert_eq!(
        invalid_cursor["error"]["code"],
        ErrorCode::InvalidParams.value()
    );
}

#[test]
fn public_tool_call_separates_protocol_and_known_tool_errors() {
    let server = tool_server();
    let success = dispatch_json(
        &server,
        &tool_request(
            20,
            "tools/call",
            &json!({"name": "oneagent.graph", "arguments": {}}),
        ),
    );
    assert_eq!(
        success["result"]["structuredContent"]["tool"],
        "oneagent.graph"
    );
    assert_eq!(success["result"]["content"][0]["type"], "text");
    assert!(success["result"].get("isError").is_none());

    let known_error = dispatch_json(
        &server,
        &tool_request(
            21,
            "tools/call",
            &json!({"name": "oneagent.query", "arguments": {"fail": true}}),
        ),
    );
    assert_eq!(known_error["result"]["isError"], true);
    assert_eq!(
        known_error["result"]["structuredContent"]["code"],
        "invalid_arguments"
    );

    for fields in [
        json!({"name": "unknown"}),
        json!({"name": "oneagent.graph", "arguments": []}),
        json!({"name": "oneagent.graph", "extra": true}),
    ] {
        let invalid = dispatch_json(&server, &tool_request(22, "tools/call", &fields));
        assert_eq!(invalid["error"]["code"], ErrorCode::InvalidParams.value());
    }
}
