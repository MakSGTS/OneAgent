use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use oneagent_llm::{
    CancellationSignal, FinishReason, LlmError, LlmErrorKind, LlmProvider, MAX_MODELS_PER_CATALOG,
    ModelCapability, ModelDescriptor, ModelId, ModelIdentity, NeverCancelled,
    ProviderConfiguration, ProviderDiagnostic, ProviderExecutionContext, ProviderExecutionPolicy,
    ProviderFuture, ProviderId, ProviderSecret, TextGenerationRequest,
};
use oneagent_ollama::OllamaProvider;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Notify, mpsc},
    task::JoinHandle,
};

fn provider(base: &str) -> OllamaProvider {
    OllamaProvider::new(
        ProviderConfiguration::new(
            ProviderId::new("ollama").expect("provider ID must pass"),
            None,
        ),
        base,
    )
    .expect("provider must construct")
}

fn request(provider: &str, model: &str, input: &str, maximum: usize) -> TextGenerationRequest {
    TextGenerationRequest::new(
        &ModelDescriptor::new(
            ModelIdentity::new(
                ProviderId::new(provider).expect("provider ID must pass"),
                ModelId::new(model).expect("model ID must pass"),
            ),
            [ModelCapability::TextGeneration],
        ),
        input,
        maximum,
    )
    .expect("request must construct")
}

fn context<'a>(
    policy: &'a ProviderExecutionPolicy,
    cancellation: &'a dyn CancellationSignal,
) -> ProviderExecutionContext<'a> {
    ProviderExecutionContext::new(policy, cancellation)
}

#[test]
fn public_construction_is_explicit_numeric_loopback_and_redacted() {
    let explicit = provider("http://127.0.0.1:8080");
    assert_eq!(explicit.id().as_str(), "ollama");
    let local = OllamaProvider::new_local(ProviderConfiguration::new(
        ProviderId::new("ollama").expect("provider ID must pass"),
        None,
    ))
    .expect("local construction must not perform I/O");
    assert_eq!(local.id().as_str(), "ollama");

    let sentinel = "synthetic-public-sensitive-sentinel";
    for value in [
        "",
        "https://127.0.0.1:11434",
        "http://localhost:11434",
        "http://127.1:11434",
        "http://127.0.0.1:11434/api",
        "http://127.0.0.1:11434/?value=synthetic-public-sensitive-sentinel",
    ] {
        let result = OllamaProvider::new(
            ProviderConfiguration::new(
                ProviderId::new("ollama").expect("provider ID must pass"),
                None,
            ),
            value,
        );
        let error = construction_error(result);
        assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
        assert_redacted(&error, sentinel);
    }

    let credential = OllamaProvider::new(
        ProviderConfiguration::new(
            ProviderId::new("ollama").expect("provider ID must pass"),
            Some(ProviderSecret::new(sentinel).expect("secret must pass")),
        ),
        "not a URL",
    );
    let error = construction_error(credential);
    assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
    assert_redacted(&error, sentinel);

    let mismatch = OllamaProvider::new(
        ProviderConfiguration::new(
            ProviderId::new("other").expect("provider ID must pass"),
            None,
        ),
        "not a URL",
    );
    assert_eq!(
        construction_error(mismatch).kind(),
        LlmErrorKind::InvalidConfiguration
    );
}

#[tokio::test]
async fn public_native_discovery_and_generation_use_exact_local_wires() {
    let tags = br#"{
        "models": [
            {"name":"z","model":"z","capabilities":["completion"]},
            {"name":"cloud","model":"cloud","remote_model":"opaque","remote_host":"opaque"},
            {"name":"a","model":"a","future":true}
        ],
        "future": true
    }"#;
    let mut server = Server::spawn(vec![
        Some(response("200 OK", tags)),
        Some(response(
            "200 OK",
            br#"{"capabilities":["completion","tools"],"future":true}"#,
        )),
        Some(response(
            "200 OK",
            br#"{"capabilities":["embedding","vision"]}"#,
        )),
        Some(response(
            "200 OK",
            r#"{"model":"a","response":"ответ","done":true,"done_reason":"stop","thinking":"","future":true}"#
                .as_bytes(),
        )),
    ])
    .await;
    let adapter = provider(server.base());
    let provider: &dyn LlmProvider = &adapter;
    let policy = ProviderExecutionPolicy::default();
    let never = NeverCancelled;

    let catalog = provider
        .discover_models(context(&policy, &never))
        .await
        .expect("discovery must succeed");
    assert_eq!(catalog.provider().as_str(), "ollama");
    assert_eq!(catalog.models().len(), 1);
    assert_eq!(catalog.models()[0].identity().model().as_str(), "a");
    assert!(catalog.models()[0].supports(ModelCapability::TextGeneration));

    let generation = request("ollama", "a", "точный вход", "ответ".len());
    let result = provider
        .generate(&generation, context(&policy, &never))
        .await
        .expect("generation must succeed");
    assert_eq!(result.model(), generation.model());
    assert_eq!(result.output(), "ответ");
    assert_eq!(result.finish(), FinishReason::Completed);
    assert_eq!(result.usage().input_bytes(), "точный вход".len());
    assert_eq!(result.usage().output_bytes(), "ответ".len());

    let tags_request = String::from_utf8(server.request().await).expect("request must be text");
    assert!(tags_request.starts_with("GET /api/tags HTTP/1.1\r\n"));
    assert!(tags_request.contains("accept: application/json\r\n"));
    assert!(tags_request.contains("user-agent: oneagent-ollama/0.1.0\r\n"));
    assert!(!tags_request.contains("authorization:"));
    assert!(!tags_request.contains("content-type:"));

    for expected in ["a", "z"] {
        let request = server.request().await;
        let (headers, body) = request_parts(&request);
        assert!(headers.starts_with("POST /api/show HTTP/1.1\r\n"));
        assert!(headers.contains("content-type: application/json\r\n"));
        assert!(!headers.contains("authorization:"));
        assert_eq!(body, json!({"model": expected, "verbose": false}));
    }

    let request = server.request().await;
    let (headers, body) = request_parts(&request);
    assert!(headers.starts_with("POST /api/generate HTTP/1.1\r\n"));
    assert!(headers.contains("content-type: application/json\r\n"));
    assert!(!headers.contains("authorization:"));
    assert_eq!(
        body,
        json!({
            "model": "a",
            "prompt": "точный вход",
            "stream": false,
            "raw": true,
            "think": false,
            "options": {"num_predict": "ответ".len()}
        })
    );
    server.finish().await;
}

#[tokio::test]
async fn public_empty_remote_catalog_finish_mapping_and_repeated_calls_are_fresh() {
    let mut server = Server::spawn(vec![
        Some(response("200 OK", br#"{"models":[]}"#)),
        Some(response(
            "200 OK",
            br#"{"models":[{"name":"cloud","model":"cloud","remote_model":"opaque","remote_host":"opaque"}]}"#,
        )),
        Some(response(
            "200 OK",
            br#"{"model":"a","response":"x","done":true,"done_reason":"length"}"#,
        )),
        Some(response(
            "200 OK",
            br#"{"model":"a","response":"y","done":true,"done_reason":"stop"}"#,
        )),
    ])
    .await;
    let adapter = provider(server.base());
    let provider: &dyn LlmProvider = &adapter;
    let policy = ProviderExecutionPolicy::default();
    let never = NeverCancelled;

    assert!(
        provider
            .discover_models(context(&policy, &never))
            .await
            .expect("empty discovery must pass")
            .is_empty()
    );
    assert!(
        provider
            .discover_models(context(&policy, &never))
            .await
            .expect("remote-only discovery must pass")
            .is_empty()
    );
    let generation = request("ollama", "a", "input", 1);
    assert_eq!(
        provider
            .generate(&generation, context(&policy, &never))
            .await
            .expect("length must map")
            .finish(),
        FinishReason::OutputLimit
    );
    assert_eq!(
        provider
            .generate(&generation, context(&policy, &never))
            .await
            .expect("repeated generation must be fresh")
            .finish(),
        FinishReason::Completed
    );
    for _ in 0..4 {
        assert!(!server.request().await.is_empty());
    }
    server.finish().await;
}

#[tokio::test]
async fn public_discovery_rejects_malformed_conflicting_duplicate_count_and_bounds() {
    let over_count = (0..=MAX_MODELS_PER_CATALOG)
        .map(|index| format!(r#"{{"name":"model-{index}","model":"model-{index}"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let cases = vec![
        (b"not-json".to_vec(), LlmErrorKind::Protocol),
        (br"{}".to_vec(), LlmErrorKind::Protocol),
        (
            br#"{"models":[{"name":"a","model":"b"}]}"#.to_vec(),
            LlmErrorKind::Protocol,
        ),
        (
            br#"{"models":[{"name":"a","model":"a","remote_model":"opaque"}]}"#.to_vec(),
            LlmErrorKind::Protocol,
        ),
        (
            br#"{"models":[{"name":"same","model":"same"},{"name":"same","model":"same"}]}"#
                .to_vec(),
            LlmErrorKind::InvalidModelCatalog,
        ),
        (
            format!(r#"{{"models":[{over_count}]}}"#).into_bytes(),
            LlmErrorKind::InvalidModelCatalog,
        ),
        (vec![b'x'; 1_024 * 1_024 + 1], LlmErrorKind::Protocol),
    ];

    for (body, expected) in cases {
        let mut server = Server::spawn(vec![Some(response_without_length("200 OK", &body))]).await;
        let adapter = provider(server.base());
        let provider: &dyn LlmProvider = &adapter;
        let policy = ProviderExecutionPolicy::default();
        let never = NeverCancelled;
        let error = provider
            .discover_models(context(&policy, &never))
            .await
            .expect_err("invalid discovery must fail atomically");
        assert_eq!(error.kind(), expected);
        assert!(!server.request().await.is_empty());
        server.finish().await;
    }
}

#[tokio::test]
async fn public_generation_rejects_malformed_terminal_bounds_status_and_redirect_once() {
    let cases = vec![
        (response("200 OK", b"not-json"), LlmErrorKind::Protocol),
        (
            response(
                "200 OK",
                br#"{"model":"fallback","response":"x","done":true,"done_reason":"stop"}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"model":"a","response":"x","done":false,"done_reason":"stop"}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"model":"a","response":"x","done":true,"done_reason":"future"}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"model":"a","response":"xx","done":true,"done_reason":"stop"}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response("503 Test", b"opaque"),
            LlmErrorKind::ProviderUnavailable,
        ),
        (
            response("400 Test", b"opaque"),
            LlmErrorKind::ProviderRejected,
        ),
        (
            response_with_headers(
                "302 Found",
                &["Location: http://127.0.0.1:9/api/generate"],
                b"opaque",
            ),
            LlmErrorKind::ProviderRejected,
        ),
        (
            response_without_length("200 OK", &vec![b'x'; 512 * 1_024 + 1]),
            LlmErrorKind::Protocol,
        ),
    ];

    for (wire, expected) in cases {
        let mut server = Server::spawn(vec![Some(wire)]).await;
        let adapter = provider(server.base());
        let provider: &dyn LlmProvider = &adapter;
        let generation = request("ollama", "a", "input", 1);
        let policy = ProviderExecutionPolicy::default();
        let never = NeverCancelled;
        let error = provider
            .generate(&generation, context(&policy, &never))
            .await
            .expect_err("invalid generation must fail");
        assert_eq!(error.kind(), expected);
        assert!(!server.request().await.is_empty());
        server.finish().await;
    }
}

#[tokio::test]
async fn public_transport_timeout_cancellation_identity_redaction_and_cleanup_are_terminal() {
    let policy = ProviderExecutionPolicy::default();
    let never = NeverCancelled;
    let unreachable = provider("http://127.0.0.1:9");
    let llm: &dyn LlmProvider = &unreachable;
    let wrong = request("other", "a", "input", 1);
    assert_eq!(
        llm.generate(&wrong, context(&policy, &never))
            .await
            .expect_err("identity mismatch must fail before transport")
            .kind(),
        LlmErrorKind::InvalidRequest
    );
    let cancelled = Signal::default();
    cancelled.cancel();
    assert_eq!(
        llm.discover_models(context(&policy, &cancelled))
            .await
            .expect_err("existing cancellation must win")
            .kind(),
        LlmErrorKind::Cancelled
    );
    assert_eq!(
        llm.discover_models(context(&policy, &never))
            .await
            .expect_err("closed loopback must be transport failure")
            .kind(),
        LlmErrorKind::Transport
    );

    let mut timeout_server = Server::spawn(vec![None]).await;
    let adapter = provider(timeout_server.base());
    let llm: &dyn LlmProvider = &adapter;
    let timeout = ProviderExecutionPolicy::new(Some(Duration::from_millis(20)))
        .expect("timeout policy must pass");
    assert_eq!(
        llm.discover_models(context(&timeout, &never))
            .await
            .expect_err("pending discovery must time out")
            .kind(),
        LlmErrorKind::Timeout
    );
    assert!(!timeout_server.request().await.is_empty());
    timeout_server.finish().await;

    let mut cancel_server = Server::spawn(vec![None]).await;
    let adapter = provider(cancel_server.base());
    let llm: &dyn LlmProvider = &adapter;
    let signal = Signal::default();
    let operation = llm.discover_models(context(&policy, &signal));
    tokio::pin!(operation);
    tokio::select! {
        captured = cancel_server.request() => assert!(!captured.is_empty()),
        result = &mut operation => panic!(
            "operation completed before cancellation: {:?}",
            result.err().map(|error| error.kind())
        ),
    }
    signal.cancel();
    assert_eq!(
        operation
            .await
            .expect_err("in-flight cancellation must win")
            .kind(),
        LlmErrorKind::Cancelled
    );
    cancel_server.finish().await;

    let sentinel = "synthetic-public-sensitive-body-sentinel";
    let server = Server::spawn(vec![Some(response("400 Test", sentinel.as_bytes()))]).await;
    let adapter = provider(server.base());
    let llm: &dyn LlmProvider = &adapter;
    let sensitive = request("ollama", "a", sentinel, 1);
    let error = llm
        .generate(&sensitive, context(&policy, &never))
        .await
        .expect_err("provider rejection must fail");
    assert_redacted(&error, sentinel);
    server.finish().await;
}

fn construction_error(result: Result<OllamaProvider, LlmError>) -> LlmError {
    match result {
        Ok(_) => panic!("construction must fail"),
        Err(error) => error,
    }
}

fn assert_redacted(error: &LlmError, sentinel: &str) {
    assert!(!format!("{error}").contains(sentinel));
    assert!(!format!("{error:?}").contains(sentinel));
    assert!(
        !error
            .diagnostic()
            .map(ProviderDiagnostic::as_str)
            .unwrap_or_default()
            .contains(sentinel)
    );
}

#[derive(Default)]
struct Signal {
    cancelled: AtomicBool,
    notify: Notify,
}

impl Signal {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }
}

impl CancellationSignal for Signal {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    fn cancelled(&self) -> ProviderFuture<'_, ()> {
        Box::pin(async move {
            let notified = self.notify.notified();
            if self.is_cancelled() {
                return;
            }
            notified.await;
        })
    }
}

struct Server {
    base: String,
    requests: mpsc::Receiver<Vec<u8>>,
    task: JoinHandle<()>,
}

impl Server {
    async fn spawn(responses: Vec<Option<Vec<u8>>>) -> Self {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("controlled listener must bind");
        let address = listener.local_addr().expect("address must exist");
        let (sender, requests) = mpsc::channel(responses.len().max(1));
        let task = tokio::spawn(async move {
            for response in responses {
                let (mut stream, _) = listener.accept().await.expect("request must connect");
                let request = read_request(&mut stream).await;
                sender
                    .send(request)
                    .await
                    .expect("request must be observed");
                if let Some(response) = response {
                    stream
                        .write_all(&response)
                        .await
                        .expect("response must write");
                    stream.shutdown().await.expect("response must close");
                } else {
                    let mut remainder = Vec::new();
                    stream
                        .read_to_end(&mut remainder)
                        .await
                        .expect("client must close");
                }
            }
        });
        Self {
            base: format!("http://{address}"),
            requests,
            task,
        }
    }

    fn base(&self) -> &str {
        &self.base
    }

    async fn request(&mut self) -> Vec<u8> {
        self.requests
            .recv()
            .await
            .expect("request must be captured")
    }

    async fn finish(self) {
        tokio::time::timeout(Duration::from_secs(2), self.task)
            .await
            .expect("server must stop")
            .expect("server task must succeed");
    }
}

fn request_parts(request: &[u8]) -> (&str, Value) {
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("headers must terminate")
        + 4;
    let headers = std::str::from_utf8(&request[..header_end]).expect("headers must decode");
    let body = serde_json::from_slice(&request[header_end..]).expect("body must decode");
    (headers, body)
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4_096];
    let header_end = loop {
        let count = stream.read(&mut buffer).await.expect("request must read");
        assert!(count > 0 && request.len() + count <= 1_024 * 1_024);
        request.extend_from_slice(&buffer[..count]);
        if let Some(position) = request.windows(4).position(|value| value == b"\r\n\r\n") {
            break position + 4;
        }
    };
    let headers = std::str::from_utf8(&request[..header_end]).expect("headers must decode");
    let length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().expect("length must parse"))
        })
        .unwrap_or(0);
    while request.len() < header_end + length {
        let count = stream.read(&mut buffer).await.expect("body must read");
        assert!(count > 0 && request.len() + count <= 1_024 * 1_024);
        request.extend_from_slice(&buffer[..count]);
    }
    request.truncate(header_end + length);
    request
}

fn response(status: &str, body: &[u8]) -> Vec<u8> {
    response_with_headers(status, &[&format!("Content-Length: {}", body.len())], body)
}

fn response_without_length(status: &str, body: &[u8]) -> Vec<u8> {
    response_with_headers(status, &[], body)
}

fn response_with_headers(status: &str, headers: &[&str], body: &[u8]) -> Vec<u8> {
    let mut value = format!("HTTP/1.1 {status}\r\nConnection: close\r\n");
    for header in headers {
        value.push_str(header);
        value.push_str("\r\n");
    }
    value.push_str("\r\n");
    let mut bytes = value.into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
