use oneagent_protocol::{
    CLIENT_CAPABILITIES_META_KEY, CLIENT_INFO_META_KEY, DecodeOutcome, ErrorCode, InboundMessage,
    MAX_REQUEST_ID_BYTES, PROTOCOL_VERSION, PROTOCOL_VERSION_META_KEY, RequestId, Response,
    decode_message, encode_response,
};
use serde_json::{Value, json};

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

        let response = Response::Error(oneagent_protocol::ErrorResponse::new(
            Some(request.id().clone()),
            ErrorCode::MethodNotFound,
        ));
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
    let encoded = encode_response(&Response::Error(oneagent_protocol::ErrorResponse::new(
        None,
        ErrorCode::ParseError,
    )))
    .expect("error response must encode");
    assert_eq!(
        String::from_utf8(encoded).expect("JSON is UTF-8"),
        r#"{"error":{"code":-32700,"message":"Parse error"},"jsonrpc":"2.0"}"#
    );
}
