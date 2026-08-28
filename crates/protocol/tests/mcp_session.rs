use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

use oneagent_protocol::{
    ErrorCode, MCP_PROTOCOL_VERSION_2025_06_18, MCP_PROTOCOL_VERSION_2025_11_25, McpConnection,
    McpProtocolRevision, McpServer, McpToolAnnotations, McpToolCallHandler, McpToolCallOutcome,
    McpToolDefinition, McpToolFuture, PROTOCOL_VERSION, Response, SUPPORTED_MCP_PROTOCOL_VERSIONS,
    encode_response,
};
use serde_json::{Map, Value, json};

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

fn response_json(response: &Response) -> Value {
    serde_json::from_slice(&encode_response(response).expect("response must encode"))
        .expect("response must be JSON")
}

fn dispatch_json(connection: &mut McpConnection<'_>, input: &str) -> Value {
    response_json(&block_on(connection.dispatch(input)).expect("request must respond"))
}

fn initialize(id: &Value, version: &str, capabilities: &Value, client_info: &Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "protocolVersion": version,
            "capabilities": capabilities,
            "clientInfo": client_info
        }
    })
    .to_string()
}

fn codex_initialize() -> String {
    initialize(
        &json!(0),
        MCP_PROTOCOL_VERSION_2025_06_18,
        &json!({"elicitation": {"form": {}, "url": {}}}),
        &json!({
            "name": "codex-mcp-client",
            "title": "Codex",
            "version": "0.150.0-alpha.8"
        }),
    )
}

fn cursor_initialize() -> String {
    initialize(
        &json!(0),
        MCP_PROTOCOL_VERSION_2025_11_25,
        &json!({"elicitation": {"form": {}}}),
        &json!({"name": "Cursor", "version": "1.0.0"}),
    )
}

fn initialized() -> &'static str {
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
}

fn legacy_request(id: &Value, method: &str, params: Option<&Value>) -> String {
    let mut request = json!({"jsonrpc": "2.0", "id": id, "method": method});
    if let Some(params) = params {
        request
            .as_object_mut()
            .expect("request object")
            .insert("params".to_owned(), params.clone());
    }
    request.to_string()
}

fn modern_request(id: &Value, method: &str, fields: &Value) -> String {
    modern_request_with_version(id, method, fields, PROTOCOL_VERSION)
}

fn modern_request_with_version(
    id: &Value,
    method: &str,
    fields: &Value,
    protocol_version: &str,
) -> String {
    let mut params = fields.as_object().expect("field object").clone();
    params.insert(
        "_meta".to_owned(),
        json!({
            "io.modelcontextprotocol/protocolVersion": protocol_version,
            "io.modelcontextprotocol/clientCapabilities": {}
        }),
    );
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

struct CountingHandler {
    calls: Arc<AtomicUsize>,
}

impl McpToolCallHandler for CountingHandler {
    fn call<'a>(&'a self, name: &'a str, arguments: &'a Map<String, Value>) -> McpToolFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if arguments.get("fail") == Some(&Value::Bool(true)) {
                McpToolCallOutcome::error("invalid_arguments", "The arguments are invalid.")
            } else {
                McpToolCallOutcome::Success(json!({"arguments": arguments, "tool": name}))
            }
        })
    }
}

fn tool_server() -> (McpServer, Arc<AtomicUsize>) {
    let schema = json!({
        "type": "object",
        "properties": {"fail": {"type": "boolean"}},
        "additionalProperties": false
    })
    .as_object()
    .expect("schema object")
    .clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let server = McpServer::with_tools(
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
        CountingHandler {
            calls: Arc::clone(&calls),
        },
    )
    .expect("valid catalog");
    (server, calls)
}

fn initialize_active<'server>(server: &'server McpServer, version: &str) -> McpConnection<'server> {
    let mut connection = server.connection();
    let response = dispatch_json(
        &mut connection,
        &initialize(
            &json!(0),
            version,
            &json!({}),
            &json!({"name": "test-client", "version": "1"}),
        ),
    );
    assert_eq!(response["result"]["protocolVersion"], version);
    assert!(block_on(connection.dispatch(initialized())).is_none());
    assert!(connection.is_initialized());
    connection
}

#[test]
fn public_supported_revision_values_are_exact_and_ordered() {
    assert_eq!(
        SUPPORTED_MCP_PROTOCOL_VERSIONS,
        [
            "2026-07-28",
            MCP_PROTOCOL_VERSION_2025_11_25,
            MCP_PROTOCOL_VERSION_2025_06_18
        ]
    );
    assert_eq!(
        McpProtocolRevision::V2025_06_18.as_str(),
        MCP_PROTOCOL_VERSION_2025_06_18
    );
    assert_eq!(
        McpProtocolRevision::V2025_11_25.as_str(),
        MCP_PROTOCOL_VERSION_2025_11_25
    );
    assert_eq!(McpProtocolRevision::V2026_07_28.as_str(), PROTOCOL_VERSION);
}

#[test]
fn exact_codex_and_cursor_initialize_requests_negotiate_and_require_initialized() {
    let (server, _) = tool_server();
    for (request, version) in [
        (codex_initialize(), MCP_PROTOCOL_VERSION_2025_06_18),
        (cursor_initialize(), MCP_PROTOCOL_VERSION_2025_11_25),
    ] {
        let mut connection = server.connection();
        assert_eq!(connection.protocol_revision(), None);
        assert!(!connection.is_initialized());
        let response = dispatch_json(&mut connection, &request);
        assert_eq!(response["id"], 0);
        assert_eq!(response["result"]["protocolVersion"], version);
        assert_eq!(response["result"]["capabilities"], json!({"tools": {}}));
        assert_eq!(response["result"]["serverInfo"]["name"], "oneagent");
        assert!(response["result"].get("resultType").is_none());
        assert!(response["result"].get("instructions").is_none());
        assert_eq!(
            connection
                .protocol_revision()
                .map(McpProtocolRevision::as_str),
            Some(version)
        );
        assert!(!connection.is_initialized());

        let before = dispatch_json(
            &mut connection,
            &legacy_request(&json!(1), "tools/list", None),
        );
        assert_eq!(before["error"]["code"], -32002);
        assert_eq!(before["error"]["message"], "Server not initialized");
        assert!(block_on(connection.dispatch(initialized())).is_none());
        assert!(connection.is_initialized());
    }
}

#[test]
fn unsupported_legacy_versions_fallback_and_invalid_initialize_does_not_select() {
    let (server, _) = tool_server();
    let mut connection = server.connection();
    let invalid = dispatch_json(
        &mut connection,
        &initialize(
            &json!(1),
            MCP_PROTOCOL_VERSION_2025_06_18,
            &json!({"elicitation": 1}),
            &json!({"name": "client", "version": "1"}),
        ),
    );
    assert_eq!(invalid["error"]["code"], -32602);
    assert_eq!(connection.protocol_revision(), None);

    let fallback = dispatch_json(
        &mut connection,
        &initialize(
            &json!("fallback"),
            "2024-11-05",
            &json!({}),
            &json!({"name": "client", "version": "1"}),
        ),
    );
    assert_eq!(fallback["id"], "fallback");
    assert_eq!(
        fallback["result"]["protocolVersion"],
        MCP_PROTOCOL_VERSION_2025_11_25
    );
    assert_eq!(
        connection.protocol_revision(),
        Some(McpProtocolRevision::V2025_11_25)
    );
}

#[test]
fn legacy_initialize_validation_is_revision_aware_and_atomic() {
    let invalid_requests = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_11_25,
                "capabilities": {},
                "clientInfo": {"name": "client", "version": "1"},
                "_meta": 42
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_06_18,
                "capabilities": {},
                "clientInfo": {"name": "client", "version": "1"},
                "_meta": {"progressToken": {}}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_06_18,
                "capabilities": {"roots": {"listChanged": 1}},
                "clientInfo": {"name": "client", "version": "1"}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 4, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_11_25,
                "capabilities": {"roots": {"listChanged": "yes"}},
                "clientInfo": {"name": "client", "version": "1"}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 5, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_06_18,
                "capabilities": {"tasks": {}},
                "clientInfo": {"name": "client", "version": "1"}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 6, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_06_18,
                "capabilities": {},
                "clientInfo": {"name": "client", "version": "1", "description": "late"}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 7, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_11_25,
                "capabilities": {"sampling": {"context": false}},
                "clientInfo": {"name": "client", "version": "1"}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 8, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_11_25,
                "capabilities": {"elicitation": {"form": []}},
                "clientInfo": {"name": "client", "version": "1"}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 9, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_11_25,
                "capabilities": {"tasks": {"requests": {"sampling": {"createMessage": true}}}},
                "clientInfo": {"name": "client", "version": "1"}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 10, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_11_25,
                "capabilities": {},
                "clientInfo": {"name": "client", "version": "1", "description": 42}
            }
        }),
    ];

    let (server, _) = tool_server();
    for invalid_request in invalid_requests {
        let mut connection = server.connection();
        let invalid = dispatch_json(&mut connection, &invalid_request.to_string());
        assert_eq!(invalid["error"]["code"], ErrorCode::InvalidParams.value());
        assert_eq!(connection.protocol_revision(), None);
        assert!(!connection.is_initialized());

        let valid = dispatch_json(&mut connection, &cursor_initialize());
        assert_eq!(
            valid["result"]["protocolVersion"],
            MCP_PROTOCOL_VERSION_2025_11_25
        );
    }
}

#[test]
fn legacy_initialize_accepts_exact_revision_specific_shapes() {
    let accepted = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_06_18,
                "capabilities": {
                    "experimental": {"example/feature": {}},
                    "roots": {"listChanged": true},
                    "sampling": {"revision-open-field": false},
                    "elicitation": {"revision-open-field": []}
                },
                "clientInfo": {"name": "client", "title": "Client", "version": "1"},
                "_meta": {"progressToken": "initialize", "example/trace": 1}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION_2025_11_25,
                "capabilities": {
                    "roots": {"listChanged": false},
                    "sampling": {"context": {}, "tools": {}},
                    "elicitation": {"form": {}, "url": {}},
                    "tasks": {
                        "list": {},
                        "cancel": {},
                        "requests": {
                            "sampling": {"createMessage": {}},
                            "elicitation": {"create": {}}
                        }
                    }
                },
                "clientInfo": {
                    "name": "client", "title": "Client", "version": "1",
                    "description": "Test client", "websiteUrl": "https://example.invalid",
                    "icons": [{"src": "https://example.invalid/icon.png", "sizes": ["32x32"], "theme": "light"}]
                },
                "_meta": {"progressToken": 1}
            }
        }),
    ];

    let (server, _) = tool_server();
    for request in accepted {
        let expected_revision = request["params"]["protocolVersion"]
            .as_str()
            .expect("protocol version");
        let mut connection = server.connection();
        let response = dispatch_json(&mut connection, &request.to_string());
        assert_eq!(response["result"]["protocolVersion"], expected_revision);
        assert_eq!(
            connection
                .protocol_revision()
                .map(McpProtocolRevision::as_str),
            Some(expected_revision)
        );
    }
}

#[test]
fn legacy_lifecycle_errors_notifications_ping_and_post_error_state_are_closed() {
    let (server, _) = tool_server();
    let mut connection = server.connection();
    let preinit = dispatch_json(&mut connection, &legacy_request(&json!(1), "ping", None));
    assert_eq!(
        preinit["error"]["code"],
        ErrorCode::ServerNotInitialized.value()
    );
    for (id, method) in [(2, "ping"), (3, "tools/list"), (4, "tools/call")] {
        let unsupported = dispatch_json(
            &mut connection,
            &modern_request_with_version(
                &json!(id),
                method,
                &json!({}),
                MCP_PROTOCOL_VERSION_2025_11_25,
            ),
        );
        assert_eq!(
            unsupported["error"]["code"],
            ErrorCode::ServerNotInitialized.value()
        );
        assert_eq!(connection.protocol_revision(), None);
    }

    let _ = dispatch_json(&mut connection, &cursor_initialize());
    assert!(block_on(connection.dispatch(r#"{"jsonrpc":"2.0","method":"unknown"}"#)).is_none());
    assert!(
        block_on(
            connection
                .dispatch(r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":[]}"#)
        )
        .is_none()
    );
    assert!(
        block_on(connection.dispatch(
            r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{"_meta":42}}"#
        ))
        .is_none()
    );
    let unknown_while_waiting = dispatch_json(
        &mut connection,
        &legacy_request(&json!(10), "review/unknown", Some(&json!({}))),
    );
    assert_eq!(
        unknown_while_waiting["error"]["code"],
        ErrorCode::MethodNotFound.value()
    );
    let still_waiting = dispatch_json(&mut connection, &legacy_request(&json!(11), "ping", None));
    assert_eq!(
        still_waiting["error"]["code"],
        ErrorCode::ServerNotInitialized.value()
    );
    assert!(block_on(connection.dispatch(initialized())).is_none());
    assert!(block_on(connection.dispatch(initialized())).is_none());

    let duplicate = dispatch_json(&mut connection, &cursor_initialize());
    assert_eq!(
        duplicate["error"]["code"],
        ErrorCode::InvalidRequest.value()
    );
    let shutdown = dispatch_json(
        &mut connection,
        &legacy_request(&json!(2), "shutdown", Some(&json!({}))),
    );
    assert_eq!(shutdown["error"]["code"], ErrorCode::MethodNotFound.value());
    assert!(block_on(connection.dispatch(r#"{"jsonrpc":"2.0","method":"exit"}"#)).is_none());
    let ping = dispatch_json(&mut connection, &legacy_request(&json!(3), "ping", None));
    assert_eq!(ping["result"], json!({}));
    assert!(ping["result"].get("resultType").is_none());

    let invalid_ping = dispatch_json(
        &mut connection,
        &legacy_request(&json!(4), "ping", Some(&json!({"unexpected": true}))),
    );
    assert_eq!(
        invalid_ping["error"]["code"],
        ErrorCode::InvalidParams.value()
    );
}

#[test]
fn undetermined_unknown_requests_preserve_metadata_error_precedence() {
    let cases = [
        (
            "absent metadata",
            legacy_request(&json!(1), "review/unknown", None),
            ErrorCode::InvalidParams,
            None,
        ),
        (
            "malformed metadata",
            legacy_request(&json!(2), "review/unknown", Some(&json!({"_meta": 42}))),
            ErrorCode::InvalidParams,
            None,
        ),
        (
            "supported legacy revision metadata",
            modern_request_with_version(
                &json!(3),
                "review/unknown",
                &json!({}),
                MCP_PROTOCOL_VERSION_2025_11_25,
            ),
            ErrorCode::UnsupportedProtocolVersion,
            None,
        ),
        (
            "valid modern metadata",
            modern_request(&json!(4), "review/unknown", &json!({})),
            ErrorCode::MethodNotFound,
            Some(McpProtocolRevision::V2026_07_28),
        ),
    ];

    for (name, request, expected_error, expected_revision) in cases {
        let (server, _) = tool_server();
        let mut connection = server.connection();
        let response = dispatch_json(&mut connection, &request);
        assert_eq!(response["error"]["code"], expected_error.value(), "{name}");
        assert_eq!(connection.protocol_revision(), expected_revision, "{name}");
    }
}

#[test]
fn legacy_initialized_notification_and_ping_accept_revision_generic_metadata() {
    for version in [
        MCP_PROTOCOL_VERSION_2025_06_18,
        MCP_PROTOCOL_VERSION_2025_11_25,
    ] {
        let (server, _) = tool_server();
        let mut connection = server.connection();
        let response = dispatch_json(
            &mut connection,
            &initialize(
                &json!(0),
                version,
                &json!({}),
                &json!({"name": "metadata-client", "version": "1"}),
            ),
        );
        assert_eq!(response["result"]["protocolVersion"], version);

        assert!(
            block_on(connection.dispatch(
                r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{"_meta":42}}"#
            ))
            .is_none()
        );
        assert!(!connection.is_initialized());
        assert!(
            block_on(connection.dispatch(
                r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{"_meta":{"progressToken":{},"example/trace":[1]}}}"#
            ))
            .is_none()
        );
        assert!(connection.is_initialized());

        let empty_params_ping = dispatch_json(
            &mut connection,
            &legacy_request(&json!(1), "ping", Some(&json!({}))),
        );
        assert_eq!(empty_params_ping["result"], json!({}));

        for (id, progress_token) in [(2, json!("progress")), (3, json!(7))] {
            let ping = dispatch_json(
                &mut connection,
                &legacy_request(
                    &json!(id),
                    "ping",
                    Some(&json!({"_meta": {"progressToken": progress_token}})),
                ),
            );
            assert_eq!(ping["result"], json!({}));
        }
        let invalid_ping = dispatch_json(
            &mut connection,
            &legacy_request(
                &json!(4),
                "ping",
                Some(&json!({"_meta": {"progressToken": true}})),
            ),
        );
        assert_eq!(
            invalid_ping["error"]["code"],
            ErrorCode::InvalidParams.value()
        );
        assert!(connection.is_initialized());
    }
}

#[test]
fn legacy_list_and_call_shapes_preserve_catalog_results_and_domain_errors() {
    for version in [
        MCP_PROTOCOL_VERSION_2025_06_18,
        MCP_PROTOCOL_VERSION_2025_11_25,
    ] {
        let (server, calls) = tool_server();
        let mut connection = initialize_active(&server, version);
        let listed = dispatch_json(
            &mut connection,
            &legacy_request(
                &json!(10),
                "tools/list",
                Some(&json!({
                    "_meta": {
                        "progressToken": "list",
                        "example/trace": [1]
                    }
                })),
            ),
        );
        assert_eq!(listed["result"]["tools"][0]["name"], "oneagent.graph");
        assert_eq!(listed["result"]["tools"][1]["name"], "oneagent.query");
        for forbidden in ["resultType", "ttlMs", "cacheScope", "nextCursor"] {
            assert!(
                listed["result"].get(forbidden).is_none(),
                "leaked {forbidden}"
            );
        }

        let success = dispatch_json(
            &mut connection,
            &legacy_request(
                &json!("success"),
                "tools/call",
                Some(&json!({"name": "oneagent.graph", "arguments": {}})),
            ),
        );
        assert_eq!(success["id"], "success");
        assert_eq!(
            success["result"]["structuredContent"]["tool"],
            "oneagent.graph"
        );
        assert!(success["result"].get("resultType").is_none());
        assert!(success["result"].get("isError").is_none());

        let domain = dispatch_json(
            &mut connection,
            &legacy_request(
                &json!(12),
                "tools/call",
                Some(&json!({
                    "name": "oneagent.query",
                    "arguments": {"fail": true},
                    "_meta": {"progressToken": "p"}
                })),
            ),
        );
        assert_eq!(domain["result"]["isError"], true);
        assert_eq!(
            domain["result"]["structuredContent"]["code"],
            "invalid_arguments"
        );
        assert!(domain["result"].get("resultType").is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}

#[test]
fn legacy_malformed_params_unknown_tools_and_metadata_fail_before_execution() {
    let (server, calls) = tool_server();
    let mut connection = initialize_active(&server, MCP_PROTOCOL_VERSION_2025_11_25);
    for request in [
        legacy_request(&json!(1), "tools/list", Some(&json!({"cursor": "next"}))),
        legacy_request(&json!(2), "tools/list", Some(&json!([]))),
        legacy_request(&json!(3), "tools/call", Some(&json!({"name": "missing"}))),
        legacy_request(
            &json!(4),
            "tools/call",
            Some(&json!({"name": "oneagent.graph", "arguments": []})),
        ),
        legacy_request(
            &json!(5),
            "tools/call",
            Some(&json!({
                "name": "oneagent.graph",
                "_meta": {"progressToken": true}
            })),
        ),
    ] {
        let response = dispatch_json(&mut connection, &request);
        assert_eq!(response["error"]["code"], ErrorCode::InvalidParams.value());
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let malformed = dispatch_json(&mut connection, "{");
    assert_eq!(malformed["error"]["code"], ErrorCode::ParseError.value());
    let duplicate = dispatch_json(
        &mut connection,
        r#"{"jsonrpc":"2.0","id":1,"id":2,"method":"ping"}"#,
    );
    assert_eq!(
        duplicate["error"]["code"],
        ErrorCode::InvalidRequest.value()
    );
    assert!(connection.is_initialized());
}

#[test]
fn independent_connections_do_not_share_legacy_or_modern_state() {
    let (server, _) = tool_server();
    let mut codex = server.connection();
    let mut cursor = server.connection();
    let mut modern = server.connection();

    let _ = dispatch_json(&mut codex, &codex_initialize());
    let _ = dispatch_json(&mut cursor, &cursor_initialize());
    let modern_list_request = modern_request(&json!(1), "tools/list", &json!({}));
    let modern_list = dispatch_json(&mut modern, &modern_list_request);
    assert_eq!(modern_list["result"]["resultType"], "complete");
    assert_eq!(modern_list["result"]["ttlMs"], 0);

    assert_eq!(
        codex.protocol_revision(),
        Some(McpProtocolRevision::V2025_06_18)
    );
    assert_eq!(
        cursor.protocol_revision(),
        Some(McpProtocolRevision::V2025_11_25)
    );
    assert_eq!(
        modern.protocol_revision(),
        Some(McpProtocolRevision::V2026_07_28)
    );
    assert!(block_on(codex.dispatch(initialized())).is_none());
    assert!(!cursor.is_initialized());

    let legacy_after_modern = dispatch_json(&mut modern, &cursor_initialize());
    assert_eq!(
        legacy_after_modern["error"]["code"],
        ErrorCode::InvalidParams.value()
    );
    assert_eq!(
        modern.protocol_revision(),
        Some(McpProtocolRevision::V2026_07_28)
    );
}

#[test]
fn modern_connection_dispatch_is_exactly_equal_to_stateless_dispatch() {
    let (server, _) = tool_server();
    let requests = [
        modern_request(&json!(0), "initialize", &json!({})),
        modern_request(&json!(1), "server/discover", &json!({})),
        modern_request(&json!(2), "tools/list", &json!({})),
        modern_request(
            &json!(3),
            "tools/call",
            &json!({"name": "oneagent.graph", "arguments": {}}),
        ),
    ];
    let mut connection = server.connection();
    for request in requests {
        let expected = block_on(server.dispatch(&request)).expect("modern request must respond");
        let actual = block_on(connection.dispatch(&request)).expect("connection must respond");
        assert_eq!(
            encode_response(&actual).expect("actual response"),
            encode_response(&expected).expect("expected response")
        );
    }
    assert_eq!(
        connection.protocol_revision(),
        Some(McpProtocolRevision::V2026_07_28)
    );
}

#[test]
fn connection_debug_never_exposes_client_identity_or_capabilities() {
    let (server, _) = tool_server();
    let mut connection = server.connection();
    let request = initialize(
        &json!(1),
        MCP_PROTOCOL_VERSION_2025_11_25,
        &json!({"experimental": {"secret-capability": {}}}),
        &json!({"name": "secret-client", "version": "secret-version"}),
    );
    let _ = dispatch_json(&mut connection, &request);
    let debug = format!("{connection:?}");
    assert!(!debug.contains("secret"));
    assert!(debug.contains("LegacyAwaitingInitialized"));
}
