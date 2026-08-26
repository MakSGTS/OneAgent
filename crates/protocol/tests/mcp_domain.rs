use oneagent_protocol::{
    CLIENT_CAPABILITIES_META_KEY, CLIENT_INFO_META_KEY, DecodeOutcome, ErrorCode, InboundMessage,
    MAX_JSON_NESTING_DEPTH, MAX_MESSAGE_BYTES, MAX_REQUEST_ID_BYTES, PROTOCOL_VERSION,
    PROTOCOL_VERSION_META_KEY, RequestId, Response, ResultResponse, decode_message,
    encode_response,
};
use serde_json::{Map, Value, json};

fn request(id: &Value, method: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {
            "_meta": {
                PROTOCOL_VERSION_META_KEY: PROTOCOL_VERSION,
                CLIENT_CAPABILITIES_META_KEY: {},
                CLIENT_INFO_META_KEY: {
                    "name": "test-client",
                    "version": "1.0"
                }
            }
        }
    })
    .to_string()
}

fn error(input: &str) -> oneagent_protocol::ErrorResponse {
    match decode_message(input) {
        DecodeOutcome::Error(error) => error,
        other => panic!("expected error, received {other:?}"),
    }
}

#[test]
fn request_identifiers_and_metadata_round_trip() {
    let identifiers = [
        json!(""),
        json!("request-α"),
        json!(-1),
        json!(0),
        json!(u64::MAX),
    ];

    for identifier in identifiers {
        let decoded = decode_message(&request(&identifier, "server/discover"));
        let DecodeOutcome::Message(InboundMessage::Request(request)) = decoded else {
            panic!("request must decode: {decoded:?}");
        };
        assert_eq!(request.method(), "server/discover");
        assert_eq!(request.metadata().protocol_version(), PROTOCOL_VERSION);
        assert!(request.metadata().client_capabilities().is_empty());
        assert_eq!(
            request
                .metadata()
                .client_info()
                .map(oneagent_protocol::Implementation::name),
            Some("test-client")
        );

        let response = Response::Error(
            oneagent_protocol::ErrorResponse::new(
                Some(request.id().clone()),
                ErrorCode::MethodNotFound,
            )
            .expect("standard error must be constructible"),
        );
        let encoded: Value = serde_json::from_slice(
            &encode_response(&response).expect("closed response must encode"),
        )
        .expect("response must be JSON");
        assert_eq!(encoded["id"], identifier);
    }
}

#[test]
fn invalid_identifier_matrix_uses_invalid_request() {
    let oversized = "x".repeat(MAX_REQUEST_ID_BYTES + 1);
    for identifier in [
        Value::Null,
        json!(true),
        json!(1.5),
        json!(1e20),
        json!([]),
        json!({}),
        json!(oversized),
    ] {
        let failure = error(&request(&identifier, "server/discover"));
        assert_eq!(failure.code(), ErrorCode::InvalidRequest);
        assert!(failure.id().is_none());
    }

    let overflow_exponent = r#"{"jsonrpc":"2.0","id":1e400,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}"#;
    let failure = error(overflow_exponent);
    assert_eq!(failure.code(), ErrorCode::InvalidRequest);
    assert!(failure.id().is_none());
}

#[test]
fn public_identifier_and_json_depth_bounds_are_exact() {
    let exact = "x".repeat(MAX_REQUEST_ID_BYTES);
    assert_eq!(
        RequestId::string(exact.clone())
            .expect("exact public request ID boundary must construct")
            .as_str()
            .map(str::len),
        Some(MAX_REQUEST_ID_BYTES)
    );
    assert!(RequestId::string("x".repeat(MAX_REQUEST_ID_BYTES + 1)).is_none());
    let DecodeOutcome::Message(InboundMessage::Request(request)) =
        decode_message(&request(&json!(exact), "server/discover"))
    else {
        panic!("exact request ID boundary must decode");
    };
    assert_eq!(
        request.id().as_str().map(str::len),
        Some(MAX_REQUEST_ID_BYTES)
    );

    let too_deep = format!(
        "{}0{}",
        "[".repeat(MAX_JSON_NESTING_DEPTH + 1),
        "]".repeat(MAX_JSON_NESTING_DEPTH + 1)
    );
    assert_eq!(error(&too_deep).code(), ErrorCode::InvalidRequest);
}

#[test]
fn parse_shape_metadata_and_version_precedence_is_stable() {
    let cases = [
        ("{", ErrorCode::ParseError, false),
        ("[]", ErrorCode::InvalidRequest, false),
        (
            r#"{"jsonrpc":"1.0","id":1,"method":"x","params":{}}"#,
            ErrorCode::InvalidRequest,
            false,
        ),
        (
            r#"{"jsonrpc":"2.0","id":1,"method":"x"}"#,
            ErrorCode::InvalidParams,
            true,
        ),
        (
            r#"{"jsonrpc":"2.0","id":1,"method":"x","params":{}}"#,
            ErrorCode::InvalidParams,
            true,
        ),
        (
            r#"{"jsonrpc":"2.0","id":1,"method":"x","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#,
            ErrorCode::InvalidParams,
            true,
        ),
    ];

    for (input, code, has_id) in cases {
        let failure = error(input);
        assert_eq!(failure.code(), code);
        assert_eq!(failure.id().is_some(), has_id);
    }

    let unsupported = request(&json!(7), "x").replace(PROTOCOL_VERSION, "1900-01-01");
    let failure = error(&unsupported);
    assert_eq!(failure.code(), ErrorCode::UnsupportedProtocolVersion);
    assert_eq!(failure.id().and_then(RequestId::as_i64), Some(7));
    assert_eq!(
        failure.data(),
        Some(&json!({
            "requested": "1900-01-01",
            "supported": [PROTOCOL_VERSION]
        }))
    );

    let invalid_client = request(&json!(8), "x").replace(
        r#"{"name":"test-client","version":"1.0"}"#,
        r#"{"name":"test-client"}"#,
    );
    let failure = error(&invalid_client);
    assert_eq!(failure.code(), ErrorCode::InvalidParams);
    assert_eq!(failure.id().and_then(RequestId::as_i64), Some(8));
}

#[test]
fn known_request_metadata_and_client_capability_shapes_match_the_schema() {
    let valid = json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "server/discover",
        "params": {
            "_meta": {
                PROTOCOL_VERSION_META_KEY: PROTOCOL_VERSION,
                CLIENT_CAPABILITIES_META_KEY: {
                    "experimental": {"feature": {}},
                    "roots": {},
                    "sampling": {"context": {}, "tools": {}},
                    "elicitation": {"form": {}, "url": {}},
                    "extensions": {"com.example/feature": {}},
                    "futureCapability": 42
                },
                CLIENT_INFO_META_KEY: {
                    "name": "test-client",
                    "version": "1.0",
                    "title": "Test Client",
                    "description": "Controlled client",
                    "websiteUrl": "https://example.invalid",
                    "icons": [{
                        "src": "data:image/png;base64,AA==",
                        "mimeType": "image/png",
                        "sizes": ["16x16", "any"],
                        "theme": "dark"
                    }]
                },
                "progressToken": 1.5,
                "io.modelcontextprotocol/logLevel": "warning",
                "com.example/extension": true
            }
        }
    })
    .to_string();
    assert!(matches!(
        decode_message(&valid),
        DecodeOutcome::Message(InboundMessage::Request(_))
    ));

    let invalid_capabilities = [
        json!({"elicitation": 42}),
        json!({"elicitation": {"form": 42}}),
        json!({"sampling": false}),
        json!({"sampling": {"tools": []}}),
        json!({"roots": null}),
        json!({"experimental": {"feature": 1}}),
        json!({"extensions": {"unprefixed": {}}}),
        json!({"extensions": {"com.example/feature": 1}}),
    ];
    for capabilities in invalid_capabilities {
        let input = json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "server/discover",
            "params": {"_meta": {
                PROTOCOL_VERSION_META_KEY: PROTOCOL_VERSION,
                CLIENT_CAPABILITIES_META_KEY: capabilities
            }}
        })
        .to_string();
        let failure = error(&input);
        assert_eq!(failure.code(), ErrorCode::InvalidParams);
        assert_eq!(failure.id().and_then(RequestId::as_i64), Some(10));
    }

    let invalid_metadata = [
        json!({"progressToken": true}),
        json!({"io.modelcontextprotocol/logLevel": "trace"}),
        json!({"bad key": true}),
        json!({CLIENT_INFO_META_KEY: {"name": "client", "version": "1", "title": false}}),
        json!({CLIENT_INFO_META_KEY: {"name": "client", "version": "1", "description": 1}}),
        json!({CLIENT_INFO_META_KEY: {"name": "client", "version": "1", "websiteUrl": {}}}),
        json!({CLIENT_INFO_META_KEY: {"name": "client", "version": "1", "icons": {}}}),
        json!({CLIENT_INFO_META_KEY: {"name": "client", "version": "1", "icons": [{"src": 1}]}}),
        json!({CLIENT_INFO_META_KEY: {"name": "client", "version": "1", "icons": [{"src": "x", "sizes": [1]}]}}),
        json!({CLIENT_INFO_META_KEY: {"name": "client", "version": "1", "icons": [{"src": "x", "theme": "sepia"}]}}),
    ];
    for addition in invalid_metadata {
        let mut metadata = Map::new();
        metadata.insert(
            PROTOCOL_VERSION_META_KEY.to_owned(),
            json!(PROTOCOL_VERSION),
        );
        metadata.insert(CLIENT_CAPABILITIES_META_KEY.to_owned(), json!({}));
        metadata.extend(addition.as_object().expect("metadata addition").clone());
        let input = json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "server/discover",
            "params": {"_meta": metadata}
        })
        .to_string();
        assert_eq!(error(&input).code(), ErrorCode::InvalidParams);
    }
}

#[test]
fn schema_valid_empty_implementation_strings_are_preserved() {
    let input = json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "server/discover",
        "params": {"_meta": {
            PROTOCOL_VERSION_META_KEY: PROTOCOL_VERSION,
            CLIENT_CAPABILITIES_META_KEY: {},
            CLIENT_INFO_META_KEY: {"name": "", "version": ""}
        }}
    })
    .to_string();
    let DecodeOutcome::Message(InboundMessage::Request(request)) = decode_message(&input) else {
        panic!("schema-valid empty implementation strings must decode");
    };
    let client_info = request
        .metadata()
        .client_info()
        .expect("client information must be retained");
    assert_eq!(client_info.name(), "");
    assert_eq!(client_info.version(), "");
}

#[test]
fn schema_valid_arbitrary_precision_progress_token_is_accepted() {
    let input = r#"{"jsonrpc":"2.0","id":9,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{},"progressToken":1e400}}}"#;
    assert!(matches!(
        decode_message(input),
        DecodeOutcome::Message(InboundMessage::Request(_))
    ));
}

#[test]
fn literal_arbitrary_precision_token_objects_preserve_schema_and_error_precedence() {
    for key in [
        "$serde_json::private::Number",
        "$serde_json::private::\\u004eumber",
    ] {
        let input = format!(
            r#"{{"jsonrpc":"2.0","id":12,"method":"server/discover","params":{{"_meta":{{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{{}},"progressToken":{{"{key}":"1"}}}}}}}}"#
        );
        let failure = error(&input);
        assert_eq!(failure.code(), ErrorCode::InvalidParams);
        assert_eq!(failure.id().and_then(RequestId::as_i64), Some(12));
    }

    let malformed = r#"{"jsonrpc":"2.0","id":12,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{},"progressToken":{"$serde_json::private::Number":"not-a-number"}}}}"#;
    let failure = error(malformed);
    assert_eq!(failure.code(), ErrorCode::InvalidParams);
    assert_eq!(failure.id().and_then(RequestId::as_i64), Some(12));

    let duplicate = r#"{"jsonrpc":"2.0","id":12,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{},"progressToken":{"$serde_json::private::Number":"1","$serde_json::private::Number":"2"}}}}"#;
    let failure = error(duplicate);
    assert_eq!(failure.code(), ErrorCode::InvalidRequest);
    assert!(failure.id().is_none());
}

#[test]
fn literal_arbitrary_precision_token_objects_observe_exact_nesting_bound() {
    fn nested_request(array_depth: usize) -> String {
        let nested = format!(
            "{}{{\"$serde_json::private::Number\":\"1\"}}{}",
            "[".repeat(array_depth),
            "]".repeat(array_depth)
        );
        format!(
            r#"{{"jsonrpc":"2.0","id":13,"method":"server/discover","params":{{"_meta":{{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{{}}}},"extension":{nested}}}}}"#
        )
    }

    let exact = nested_request(MAX_JSON_NESTING_DEPTH - 3);
    assert!(matches!(
        decode_message(&exact),
        DecodeOutcome::Message(InboundMessage::Request(_))
    ));

    let over = nested_request(MAX_JSON_NESTING_DEPTH - 2);
    let failure = error(&over);
    assert_eq!(failure.code(), ErrorCode::InvalidRequest);
    assert!(failure.id().is_none());
}

#[test]
fn duplicate_reordered_unicode_and_repeated_inputs_are_deterministic() {
    let duplicate = r#"{"jsonrpc":"2.0","id":1,"method":"x","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{},"secret":1,"secret":2}}}"#;
    assert_eq!(error(duplicate).code(), ErrorCode::InvalidRequest);

    let reordered = r#"{"params":{"unknown":"✓","_meta":{"io.modelcontextprotocol/clientCapabilities":{},"io.modelcontextprotocol/protocolVersion":"2026-07-28"}},"method":"server/discover","unknown":true,"id":"α","jsonrpc":"2.0"}"#;
    for _ in 0..3 {
        let DecodeOutcome::Message(InboundMessage::Request(value)) = decode_message(reordered)
        else {
            panic!("reordered request must decode");
        };
        assert_eq!(value.id().as_str(), Some("α"));
    }

    let debug = format!("{:?}", error(duplicate));
    assert!(!debug.contains("secret"));
}

#[test]
fn malformed_json_precedes_earlier_duplicate_and_depth_failures() {
    for input in [
        r#"{"jsonrpc":"2.0","jsonrpc":"2.0","#,
        r#"{"jsonrpc":"2.0","x":1,"x":2,"tail":"\q"}"#,
        r#"{"jsonrpc":"2.0","nested":{"x":1,"x":2},"tail":"\q"}"#,
    ] {
        let failure = error(input);
        assert_eq!(failure.code(), ErrorCode::ParseError);
        assert!(failure.id().is_none());
    }

    let malformed_after_depth = format!(
        "{}0{}x",
        "[".repeat(MAX_JSON_NESTING_DEPTH + 1),
        "]".repeat(MAX_JSON_NESTING_DEPTH + 1)
    );
    let failure = error(&malformed_after_depth);
    assert_eq!(failure.code(), ErrorCode::ParseError);
    assert!(failure.id().is_none());

    let valid_over_depth = format!(
        "{}0{}",
        "[".repeat(MAX_JSON_NESTING_DEPTH + 1),
        "]".repeat(MAX_JSON_NESTING_DEPTH + 1)
    );
    assert_eq!(error(&valid_over_depth).code(), ErrorCode::InvalidRequest);
}

#[test]
fn notifications_are_distinct_and_never_create_error_responses() {
    let valid = r#"{"jsonrpc":"2.0","method":"unknown","params":{"value":1}}"#;
    let DecodeOutcome::Message(InboundMessage::Notification(notification)) = decode_message(valid)
    else {
        panic!("valid notification must remain a notification");
    };
    assert_eq!(notification.method(), "unknown");

    assert_eq!(
        decode_message(r#"{"jsonrpc":"2.0","method":"unknown","params":[]}"#),
        DecodeOutcome::IgnoredNotification
    );
}

#[test]
fn compact_error_serialization_has_exact_closed_shape() {
    let encoded = encode_response(&Response::Error(
        oneagent_protocol::ErrorResponse::new(None, ErrorCode::ParseError)
            .expect("standard error must be constructible"),
    ))
    .expect("error response must encode");
    assert_eq!(
        String::from_utf8(encoded).expect("JSON is UTF-8"),
        r#"{"error":{"code":-32700,"message":"Parse error"},"jsonrpc":"2.0"}"#
    );
}

#[test]
fn public_error_construction_enforces_request_id_precedence() {
    assert!(
        oneagent_protocol::ErrorResponse::new(Some(RequestId::unsigned(1)), ErrorCode::ParseError)
            .is_err()
    );
    assert!(oneagent_protocol::ErrorResponse::new(None, ErrorCode::InvalidRequest).is_ok());
    assert!(
        oneagent_protocol::ErrorResponse::new(
            Some(RequestId::unsigned(1)),
            ErrorCode::InvalidRequest
        )
        .is_ok()
    );
    for code in [
        ErrorCode::MethodNotFound,
        ErrorCode::InvalidParams,
        ErrorCode::InternalError,
    ] {
        assert!(oneagent_protocol::ErrorResponse::new(None, code).is_err());
        assert!(oneagent_protocol::ErrorResponse::new(Some(RequestId::unsigned(1)), code).is_ok());
    }
}

#[test]
fn public_outbound_construction_enforces_error_data_size_and_depth_invariants() {
    assert!(
        oneagent_protocol::ErrorResponse::new(None, ErrorCode::UnsupportedProtocolVersion).is_err()
    );
    assert!(
        oneagent_protocol::ErrorResponse::new(None, ErrorCode::MissingRequiredClientCapability)
            .is_err()
    );
    assert!(
        oneagent_protocol::ErrorResponse::missing_capability(
            RequestId::unsigned(1),
            json!({"elicitation": 42})
                .as_object()
                .expect("capability object")
        )
        .is_err()
    );
    assert!(
        oneagent_protocol::ErrorResponse::unsupported_version(
            RequestId::unsigned(1),
            &"x".repeat(MAX_MESSAGE_BYTES)
        )
        .is_err()
    );

    let mut baseline_fields = Map::new();
    baseline_fields.insert("payload".to_owned(), json!(""));
    let baseline = ResultResponse::complete(RequestId::unsigned(1), baseline_fields)
        .expect("baseline response must fit");
    let baseline_bytes = encode_response(&Response::Result(baseline))
        .expect("baseline response must encode")
        .len();
    let padding_bytes = MAX_MESSAGE_BYTES - baseline_bytes;

    let mut exact_fields = Map::new();
    exact_fields.insert("payload".to_owned(), json!("x".repeat(padding_bytes)));
    let exact = ResultResponse::complete(RequestId::unsigned(1), exact_fields)
        .expect("exact-bound response must be constructible");
    assert_eq!(
        encode_response(&Response::Result(exact))
            .expect("exact-bound response must encode")
            .len(),
        MAX_MESSAGE_BYTES
    );

    let mut oversized_fields = Map::new();
    oversized_fields.insert("payload".to_owned(), json!("x".repeat(padding_bytes + 1)));
    assert!(ResultResponse::complete(RequestId::unsigned(1), oversized_fields).is_err());

    let mut exact_depth = Value::Null;
    for _ in 0..MAX_JSON_NESTING_DEPTH - 2 {
        exact_depth = Value::Array(vec![exact_depth]);
    }
    let mut exact_depth_fields = Map::new();
    exact_depth_fields.insert("nested".to_owned(), exact_depth.clone());
    assert!(
        ResultResponse::complete(RequestId::unsigned(1), exact_depth_fields).is_ok(),
        "exact outbound nesting boundary must be accepted"
    );

    let mut over_depth_fields = Map::new();
    over_depth_fields.insert("nested".to_owned(), Value::Array(vec![exact_depth]));
    assert!(ResultResponse::complete(RequestId::unsigned(1), over_depth_fields).is_err());
}
