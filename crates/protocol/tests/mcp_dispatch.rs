use oneagent_protocol::{ErrorCode, McpServer, PROTOCOL_VERSION, Response, encode_response};
use serde_json::{Value, json};

fn request(id: &Value, method: &str, version: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": version,
                "io.modelcontextprotocol/clientCapabilities": {}
            }
        }
    })
    .to_string()
}

fn dispatch_json(server: &McpServer, input: &str) -> Value {
    let response = server.dispatch(input).expect("request must respond");
    serde_json::from_slice(&encode_response(&response).expect("response must encode"))
        .expect("response must be JSON")
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

    let unsupported = server
        .dispatch(&request(&json!(1), "server/discover", "1900-01-01"))
        .expect("request must respond");
    let Response::Error(unsupported) = unsupported else {
        panic!("unsupported version must fail");
    };
    assert_eq!(unsupported.code(), ErrorCode::UnsupportedProtocolVersion);

    let unknown = server
        .dispatch(&request(&json!(2), "tools/list", PROTOCOL_VERSION))
        .expect("request must respond");
    let Response::Error(unknown) = unknown else {
        panic!("unknown method must fail");
    };
    assert_eq!(unknown.code(), ErrorCode::MethodNotFound);

    assert!(
        server
            .dispatch(r#"{"jsonrpc":"2.0","method":"server/discover"}"#)
            .is_none()
    );
    assert!(
        server
            .dispatch(r#"{"jsonrpc":"2.0","method":"unknown","params":[]}"#)
            .is_none()
    );
}
