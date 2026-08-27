use oneagent_protocol::{
    LspCapabilities, LspDispatchOutcome, LspErrorCode, LspExitStatus, LspHandler, LspHandlerError,
    LspServer, MAX_MESSAGE_BYTES, MAX_REQUEST_ID_BYTES, encode_lsp_response,
};
use serde_json::{Map, Value, json};

const ROOT: &str = "file:///workspace";
const DOCUMENT: &str = "file:///workspace/src/Module.bsl";

#[derive(Clone, Copy, Default)]
struct Handler {
    invalid_symbols: bool,
    fail_diagnostics: bool,
}

impl LspHandler for Handler {
    fn validate_initialize(&self, params: &Map<String, Value>) -> Result<(), LspHandlerError> {
        (params.get("rootUri").and_then(Value::as_str) == Some(ROOT))
            .then_some(())
            .ok_or(LspHandlerError::InvalidParams)
    }

    fn workspace_symbols(&self, query: &str) -> Result<Value, LspHandlerError> {
        if query == "fail" {
            return Err(LspHandlerError::RequestFailed);
        }
        if self.invalid_symbols {
            return Ok(json!({"invalid": true}));
        }
        Ok(json!([{
            "name": query,
            "kind": 12,
            "containerName": "Main",
            "location": {
                "uri": DOCUMENT,
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 0}
                }
            }
        }]))
    }

    fn document_diagnostics(&self, _uri: &str) -> Result<Value, LspHandlerError> {
        if self.fail_diagnostics {
            return Err(LspHandlerError::Internal);
        }
        Ok(json!({
            "kind": "full",
            "items": [{
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 0}
                },
                "severity": 1,
                "code": "semantic.reference.unresolved",
                "source": "oneagent",
                "message": "semantic reference target could not be resolved"
            }]
        }))
    }
}

#[derive(Clone, Copy)]
struct BoundaryHandler {
    symbol_count: usize,
    diagnostic_count: usize,
    oversized_symbol: bool,
    position: u64,
}

impl LspHandler for BoundaryHandler {
    fn validate_initialize(&self, params: &Map<String, Value>) -> Result<(), LspHandlerError> {
        Handler::default().validate_initialize(params)
    }

    fn workspace_symbols(&self, _query: &str) -> Result<Value, LspHandlerError> {
        if self.symbol_count > 100 {
            return Err(LspHandlerError::RequestFailed);
        }
        let name = if self.oversized_symbol {
            "x".repeat(MAX_MESSAGE_BYTES)
        } else {
            "Symbol".to_owned()
        };
        Ok(Value::Array(
            (0..self.symbol_count)
                .map(|index| {
                    json!({
                        "name": format!("{name}{index}"),
                        "kind": 12,
                        "containerName": "Main",
                        "location": {
                            "uri": DOCUMENT,
                            "range": {
                                "start": {"line": self.position, "character": self.position},
                                "end": {"line": self.position, "character": self.position}
                            }
                        }
                    })
                })
                .collect(),
        ))
    }

    fn document_diagnostics(&self, _uri: &str) -> Result<Value, LspHandlerError> {
        if self.diagnostic_count > 100 {
            return Err(LspHandlerError::RequestFailed);
        }
        Ok(json!({
            "kind": "full",
            "items": (0..self.diagnostic_count)
                .map(|index| json!({
                    "range": {
                        "start": {"line": self.position, "character": self.position},
                        "end": {"line": self.position, "character": self.position}
                    },
                    "severity": 1,
                    "code": format!("diagnostic.{index}"),
                    "source": "oneagent",
                    "message": "bounded diagnostic"
                }))
                .collect::<Vec<_>>()
        }))
    }
}

fn request(id: impl Into<Value>, method: &str, params: Option<Value>) -> String {
    let mut value = json!({"jsonrpc": "2.0", "id": id.into(), "method": method});
    if let Some(params) = params {
        value
            .as_object_mut()
            .expect("request must be object")
            .insert("params".to_owned(), params);
    }
    serde_json::to_string(&value).expect("request must encode")
}

fn notification(method: &str, params: Option<Value>) -> String {
    let mut value = json!({"jsonrpc": "2.0", "method": method});
    if let Some(params) = params {
        value
            .as_object_mut()
            .expect("notification must be object")
            .insert("params".to_owned(), params);
    }
    serde_json::to_string(&value).expect("notification must encode")
}

fn initialize_params(root: &str) -> Value {
    json!({
        "processId": null,
        "rootUri": root,
        "capabilities": {"general": {"positionEncodings": ["utf-16"]}},
        "workspaceFolders": [{"uri": root, "name": "workspace"}]
    })
}

fn response(outcome: LspDispatchOutcome) -> Value {
    let LspDispatchOutcome::Response(response) = outcome else {
        panic!("response outcome expected");
    };
    serde_json::from_slice(&encode_lsp_response(&response).expect("response must encode"))
        .expect("response JSON must parse")
}

fn initialized_server(
    capabilities: LspCapabilities,
    handler: impl LspHandler + 'static,
) -> LspServer {
    let mut server = LspServer::with_capabilities(capabilities, handler);
    assert_eq!(
        response(server.dispatch(&request(
            json!(1),
            "initialize",
            Some(initialize_params(ROOT))
        )))["result"]["serverInfo"]["name"],
        "oneagent"
    );
    assert_eq!(
        server.dispatch(&notification("initialized", Some(json!({})))),
        LspDispatchOutcome::NoResponse
    );
    server
}

#[test]
fn public_capabilities_are_truthful_and_additive() {
    let lifecycle = LspServer::new(Handler::default());
    assert_eq!(
        lifecycle.capabilities(),
        json!({"positionEncoding": "utf-16", "textDocumentSync": 0})
            .as_object()
            .expect("capabilities must be object")
            .clone()
    );

    let complete = LspServer::with_capabilities(
        LspCapabilities::lifecycle_only()
            .with_workspace_symbols()
            .with_diagnostics(),
        Handler::default(),
    );
    assert_eq!(complete.capabilities()["workspaceSymbolProvider"], true);
    assert_eq!(
        complete.capabilities()["diagnosticProvider"],
        json!({
            "identifier": "oneagent",
            "interFileDependencies": true,
            "workspaceDiagnostics": false
        })
    );
}

#[test]
fn public_lifecycle_requires_initialize_initialized_shutdown_and_exit() {
    let capabilities = LspCapabilities::lifecycle_only().with_workspace_symbols();
    let mut server = initialized_server(capabilities, Handler::default());

    let symbols = response(server.dispatch(&request(
        json!(2),
        "workspace/symbol",
        Some(json!({"query": "Fill"})),
    )));
    assert_eq!(symbols["result"][0]["name"], "Fill");

    let shutdown = response(server.dispatch(&request(json!(3), "shutdown", None)));
    assert_eq!(shutdown, json!({"jsonrpc": "2.0", "id": 3, "result": null}));
    assert_eq!(
        response(server.dispatch(&request(
            json!(4),
            "workspace/symbol",
            Some(json!({"query": "x"}))
        )))["error"]["code"],
        -32600
    );
    assert_eq!(
        server.dispatch(&notification("exit", None)),
        LspDispatchOutcome::Exit(LspExitStatus::Success)
    );
}

#[test]
fn public_preinitialize_and_early_exit_behavior_is_closed() {
    let mut server = LspServer::new(Handler::default());
    let preinit = response(server.dispatch(&request(
        json!(1),
        "workspace/symbol",
        Some(json!({"query": "x"})),
    )));
    assert_eq!(preinit["error"]["code"], -32002);
    assert_eq!(preinit["error"]["message"], "Server not initialized");
    assert_eq!(
        server.dispatch(&notification("initialized", Some(json!({})))),
        LspDispatchOutcome::NoResponse
    );
    assert_eq!(
        server.dispatch(&notification("exit", None)),
        LspDispatchOutcome::Exit(LspExitStatus::Failure)
    );
}

#[test]
fn public_initialize_validation_and_handler_root_gate_are_deterministic() {
    let mut server = LspServer::new(Handler::default());
    let wrong_root = response(server.dispatch(&request(
        json!(1),
        "initialize",
        Some(initialize_params("file:///other")),
    )));
    assert_eq!(wrong_root["error"]["code"], -32602);

    let mut invalid_encoding = initialize_params(ROOT);
    invalid_encoding["capabilities"]["general"]["positionEncodings"] = json!(["utf-8"]);
    let invalid_encoding =
        response(server.dispatch(&request(json!(2), "initialize", Some(invalid_encoding))));
    assert_eq!(invalid_encoding["error"]["code"], -32602);

    let accepted = response(server.dispatch(&request(
        json!(3),
        "initialize",
        Some(initialize_params(ROOT)),
    )));
    assert_eq!(
        accepted["result"]["capabilities"]["positionEncoding"],
        "utf-16"
    );
    let duplicate = response(server.dispatch(&request(
        json!(4),
        "initialize",
        Some(initialize_params(ROOT)),
    )));
    assert_eq!(duplicate["error"]["code"], -32002);
}

#[test]
fn public_decode_precedence_rejects_syntax_duplicates_depth_and_bad_ids() {
    let mut server = LspServer::new(Handler::default());
    assert_eq!(response(server.dispatch("{"))["error"]["code"], -32700);
    assert_eq!(
        response(
            server.dispatch(r#"{"jsonrpc":"2.0","id":1,"id":2,"method":"initialize","params":{}}"#)
        )["error"]["code"],
        -32600
    );
    assert_eq!(
        response(
            server.dispatch(r#"{"jsonrpc":"2.0","id":1.5,"method":"initialize","params":{}}"#)
        )["id"],
        Value::Null
    );
    let over_depth = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":"file:///workspace","capabilities":{"nested":"#,
        &"[".repeat(129),
        "0",
        &"]".repeat(129),
        "}}}",
    ]
    .concat();
    assert_eq!(
        response(server.dispatch(&over_depth))["error"]["code"],
        -32600
    );
}

#[test]
fn public_lsp_integer_and_string_id_bounds_are_exact() {
    for id in [i64::from(i32::MIN), i64::from(i32::MAX)] {
        let mut server = LspServer::new(Handler::default());
        let accepted = response(server.dispatch(&request(
            json!(id),
            "initialize",
            Some(initialize_params(ROOT)),
        )));
        assert_eq!(accepted["id"], id);
    }
    for id in [i64::from(i32::MIN) - 1, i64::from(i32::MAX) + 1] {
        let mut server = LspServer::new(Handler::default());
        let rejected = response(server.dispatch(&request(
            json!(id),
            "initialize",
            Some(initialize_params(ROOT)),
        )));
        assert_eq!(rejected["id"], Value::Null);
        assert_eq!(rejected["error"]["code"], -32600);
    }

    let exact = "i".repeat(MAX_REQUEST_ID_BYTES);
    let mut server = LspServer::new(Handler::default());
    assert_eq!(
        response(server.dispatch(&request(
            json!(exact),
            "initialize",
            Some(initialize_params(ROOT))
        )))["id"],
        "i".repeat(MAX_REQUEST_ID_BYTES)
    );
    let mut server = LspServer::new(Handler::default());
    let rejected = response(server.dispatch(&request(
        json!("i".repeat(MAX_REQUEST_ID_BYTES + 1)),
        "initialize",
        Some(initialize_params(ROOT)),
    )));
    assert_eq!(rejected["id"], Value::Null);
    assert_eq!(rejected["error"]["code"], -32600);
}

#[test]
fn public_process_progress_method_and_query_bounds_are_exact() {
    for process_id in [i64::from(i32::MIN), i64::from(i32::MAX)] {
        let mut params = initialize_params(ROOT);
        params["processId"] = json!(process_id);
        let mut server = LspServer::new(Handler::default());
        assert!(
            response(server.dispatch(&request(json!(1), "initialize", Some(params))))
                .get("result")
                .is_some()
        );
    }
    for process_id in [
        json!(i64::from(i32::MIN) - 1),
        json!(i64::from(i32::MAX) + 1),
        json!(1.5),
    ] {
        let mut params = initialize_params(ROOT);
        params["processId"] = process_id;
        let mut server = LspServer::new(Handler::default());
        assert_eq!(
            response(server.dispatch(&request(json!(1), "initialize", Some(params))))["error"]["code"],
            -32602
        );
    }

    let capabilities = LspCapabilities::lifecycle_only().with_workspace_symbols();
    let mut server = initialized_server(capabilities, Handler::default());
    for token in [json!(i32::MIN), json!(i32::MAX), json!("progress")] {
        assert!(
            response(server.dispatch(&request(
                json!(2),
                "workspace/symbol",
                Some(json!({"query": "x", "workDoneToken": token})),
            )))
            .get("result")
            .is_some()
        );
    }
    for token in [json!(i64::from(i32::MAX) + 1), json!(1.5)] {
        assert_eq!(
            response(server.dispatch(&request(
                json!(3),
                "workspace/symbol",
                Some(json!({"query": "x", "partialResultToken": token})),
            )))["error"]["code"],
            -32602
        );
    }

    assert_eq!(
        response(server.dispatch(&request(json!(4), &"m".repeat(256), None)))["error"]["code"],
        -32601
    );
    assert_eq!(
        response(server.dispatch(&request(json!(5), &"m".repeat(257), None)))["error"]["code"],
        -32600
    );
    for method in ["m".repeat(256), "m".repeat(257)] {
        assert_eq!(
            server.dispatch(&notification(&method, None)),
            LspDispatchOutcome::NoResponse
        );
    }
    assert!(
        response(server.dispatch(&request(
            json!(6),
            "workspace/symbol",
            Some(json!({"query": "q".repeat(256)})),
        )))
        .get("result")
        .is_some()
    );
    assert_eq!(
        response(server.dispatch(&request(
            json!(7),
            "workspace/symbol",
            Some(json!({"query": "q".repeat(257)})),
        )))["error"]["code"],
        -32602
    );
}

#[test]
fn public_complete_semantic_results_and_position_bounds_are_exact() {
    let capabilities = LspCapabilities::lifecycle_only()
        .with_workspace_symbols()
        .with_diagnostics();
    let mut server = initialized_server(
        capabilities,
        BoundaryHandler {
            symbol_count: 100,
            diagnostic_count: 100,
            oversized_symbol: false,
            position: i32::MAX as u64,
        },
    );
    assert_eq!(
        response(server.dispatch(&request(
            json!(2),
            "workspace/symbol",
            Some(json!({"query": ""})),
        )))["result"]
            .as_array()
            .expect("exact symbol result")
            .len(),
        100
    );
    assert_eq!(
        response(server.dispatch(&request(
            json!(3),
            "textDocument/diagnostic",
            Some(json!({"textDocument": {"uri": DOCUMENT}})),
        )))["result"]["items"]
            .as_array()
            .expect("exact diagnostic result")
            .len(),
        100
    );

    for (symbols, diagnostics, method, params) in [
        (101, 0, "workspace/symbol", json!({"query": ""})),
        (
            0,
            101,
            "textDocument/diagnostic",
            json!({"textDocument": {"uri": DOCUMENT}}),
        ),
    ] {
        let mut server = initialized_server(
            capabilities,
            BoundaryHandler {
                symbol_count: symbols,
                diagnostic_count: diagnostics,
                oversized_symbol: false,
                position: 0,
            },
        );
        assert_eq!(
            response(server.dispatch(&request(json!(4), method, Some(params))))["error"]["code"],
            -32803
        );
    }

    let mut oversized = initialized_server(
        capabilities,
        BoundaryHandler {
            symbol_count: 1,
            diagnostic_count: 0,
            oversized_symbol: true,
            position: 0,
        },
    );
    assert_eq!(
        response(oversized.dispatch(&request(
            json!(5),
            "workspace/symbol",
            Some(json!({"query": ""})),
        )))["error"]["code"],
        -32803
    );

    let mut invalid_position = initialized_server(
        capabilities,
        BoundaryHandler {
            symbol_count: 1,
            diagnostic_count: 0,
            oversized_symbol: false,
            position: i32::MAX as u64 + 1,
        },
    );
    assert_eq!(
        response(invalid_position.dispatch(&request(
            json!(6),
            "workspace/symbol",
            Some(json!({"query": ""})),
        )))["error"]["code"],
        -32603
    );
}

#[test]
fn public_semantic_params_capabilities_and_handler_failures_are_closed() {
    let mut lifecycle = initialized_server(LspCapabilities::lifecycle_only(), Handler::default());
    assert_eq!(
        response(lifecycle.dispatch(&request(
            json!(2),
            "workspace/symbol",
            Some(json!({"query": "x"})),
        )))["error"]["code"],
        -32601
    );

    let capabilities = LspCapabilities::lifecycle_only()
        .with_workspace_symbols()
        .with_diagnostics();
    let mut server = initialized_server(capabilities, Handler::default());
    assert_eq!(
        response(server.dispatch(&request(
            json!(2),
            "workspace/symbol",
            Some(json!({"query": "x", "extra": true})),
        )))["error"]["code"],
        -32602
    );
    assert_eq!(
        response(server.dispatch(&request(
            json!(3),
            "workspace/symbol",
            Some(json!({"query": "fail"})),
        )))["error"]["code"],
        -32803
    );
    assert_eq!(
        response(server.dispatch(&request(
            json!(4),
            "textDocument/diagnostic",
            Some(json!({"textDocument": {"uri": DOCUMENT}, "identifier": "other"})),
        )))["error"]["code"],
        -32602
    );
}

#[test]
fn public_handler_results_are_shape_checked_before_encoding() {
    let capabilities = LspCapabilities::lifecycle_only()
        .with_workspace_symbols()
        .with_diagnostics();
    let mut server = initialized_server(
        capabilities,
        Handler {
            invalid_symbols: true,
            fail_diagnostics: true,
        },
    );
    assert_eq!(
        response(server.dispatch(&request(
            json!(2),
            "workspace/symbol",
            Some(json!({"query": "x"})),
        )))["error"]["code"],
        -32603
    );
    assert_eq!(
        response(server.dispatch(&request(
            json!(3),
            "textDocument/diagnostic",
            Some(json!({"textDocument": {"uri": DOCUMENT}})),
        )))["error"]["code"],
        -32603
    );
}

#[test]
fn public_notifications_never_create_responses_or_mutate_wrong_states() {
    let mut server = LspServer::new(Handler::default());
    for input in [
        notification("$/cancelRequest", Some(json!({"id": 1}))),
        notification("unknown", Some(json!({}))),
        notification("initialized", Some(json!(false))),
    ] {
        assert_eq!(server.dispatch(&input), LspDispatchOutcome::NoResponse);
    }
    assert_eq!(
        response(server.dispatch(&request(
            json!(1),
            "workspace/symbol",
            Some(json!({"query": "x"})),
        )))["error"]["code"],
        -32002
    );
}

#[test]
fn public_error_vocabulary_is_stable() {
    let cases = [
        (LspErrorCode::ParseError, -32700, "Parse error"),
        (LspErrorCode::InvalidRequest, -32600, "Invalid Request"),
        (LspErrorCode::MethodNotFound, -32601, "Method not found"),
        (LspErrorCode::InvalidParams, -32602, "Invalid params"),
        (LspErrorCode::InternalError, -32603, "Internal error"),
        (
            LspErrorCode::ServerNotInitialized,
            -32002,
            "Server not initialized",
        ),
        (LspErrorCode::RequestFailed, -32803, "Request failed"),
    ];
    for (code, value, message) in cases {
        assert_eq!(code.value(), value);
        assert_eq!(code.message(), message);
    }
    assert_eq!(MAX_REQUEST_ID_BYTES, 256);
}
