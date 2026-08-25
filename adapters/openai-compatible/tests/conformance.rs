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
use oneagent_openai_compatible::OpenAiCompatibleProvider;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Notify, mpsc},
    task::JoinHandle,
};

fn provider(base: &str, secret: Option<&str>) -> OpenAiCompatibleProvider {
    OpenAiCompatibleProvider::new(
        ProviderConfiguration::new(
            ProviderId::new("openai-compatible").expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
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
fn public_construction_is_explicit_bounded_and_redacted() {
    let sentinel = "synthetic-public-secret-sentinel";
    let accepted = provider("http://127.0.0.1:8080", Some(sentinel));
    assert_eq!(accepted.id().as_str(), "openai-compatible");

    for value in [
        "",
        "ftp://example.invalid",
        "http:///missing-host",
        "http://user:password@example.invalid",
        "http://example.invalid/v1",
        "http://example.invalid/?secret=synthetic-public-secret-sentinel",
        "http://example.invalid/#synthetic-public-secret-sentinel",
    ] {
        let result = OpenAiCompatibleProvider::new(
            ProviderConfiguration::new(
                ProviderId::new("openai-compatible").expect("provider ID must pass"),
                None,
            ),
            value,
        );
        let error = construction_error(result);
        assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
        assert_redacted(&error, sentinel);
    }

    let invalid_header = OpenAiCompatibleProvider::new(
        ProviderConfiguration::new(
            ProviderId::new("openai-compatible").expect("provider ID must pass"),
            Some(
                ProviderSecret::new("synthetic-public-secret-sentinel\ninvalid")
                    .expect("secret domain accepts opaque content"),
            ),
        ),
        "http://example.invalid",
    );
    assert_redacted(&construction_error(invalid_header), sentinel);
}

#[tokio::test]
async fn public_discovery_and_generation_use_exact_authenticated_wires() {
    let discovery = br#"{"object":"list","data":[{"id":"z"},{"id":"a"}],"future":true}"#;
    let completion = r#"{"object":"text_completion","model":"a","choices":[{"text":"ответ","index":0,"finish_reason":"stop"}],"usage":{"ignored":1}}"#
        .as_bytes();
    let mut server = Server::spawn(vec![
        Some(response("200 OK", discovery)),
        Some(response("200 OK", completion)),
    ])
    .await;
    let adapter = provider(server.base(), Some("synthetic-public-secret-sentinel"));
    let policy = ProviderExecutionPolicy::default();
    let never = NeverCancelled;
    let provider: &dyn LlmProvider = &adapter;

    let catalog = provider
        .discover_models(context(&policy, &never))
        .await
        .expect("discovery must succeed");
    assert_eq!(catalog.models()[0].identity().model().as_str(), "a");
    assert_eq!(catalog.models()[1].identity().model().as_str(), "z");
    assert!(catalog.models().iter().all(|model| {
        model.capabilities().len() == 1 && model.supports(ModelCapability::TextGeneration)
    }));

    let generation = request("openai-compatible", "a", "точный вход", "ответ".len());
    let result = provider
        .generate(&generation, context(&policy, &never))
        .await
        .expect("generation must succeed");
    assert_eq!(result.output(), "ответ");
    assert_eq!(result.finish(), FinishReason::Completed);
    assert_eq!(result.usage().input_bytes(), "точный вход".len());
    assert_eq!(result.usage().output_bytes(), "ответ".len());

    let discovery_request =
        String::from_utf8(server.request().await).expect("request must be text");
    assert!(discovery_request.starts_with("GET /v1/models HTTP/1.1\r\n"));
    assert!(discovery_request.contains("accept: application/json\r\n"));
    assert!(discovery_request.contains("authorization: Bearer synthetic-public-secret-sentinel"));
    assert!(discovery_request.contains("user-agent: oneagent-openai-compatible/0.1.0\r\n"));
    assert!(!discovery_request.contains("content-length:"));
    let generation_request = server.request().await;
    let header_end = generation_request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("headers must terminate")
        + 4;
    let headers =
        std::str::from_utf8(&generation_request[..header_end]).expect("headers must decode");
    assert!(headers.starts_with("POST /v1/completions HTTP/1.1\r\n"));
    assert!(headers.contains("content-type: application/json"));
    let body: Value =
        serde_json::from_slice(&generation_request[header_end..]).expect("body must decode");
    assert_eq!(
        body,
        json!({"model":"a","prompt":"точный вход","max_tokens":"ответ".len(),"stream":false})
    );
    server.finish().await;
}

#[tokio::test]
async fn public_empty_catalog_finish_mapping_and_repeated_calls_are_fresh() {
    let responses = vec![
        Some(response("200 OK", br#"{"object":"list","data":[]}"#)),
        Some(response(
            "200 OK",
            br#"{"object":"list","data":[{"id":"a"}]}"#,
        )),
        Some(response(
            "200 OK",
            br#"{"object":"text_completion","model":"a","choices":[{"text":"x","index":0,"finish_reason":"length"}]}"#,
        )),
        Some(response(
            "200 OK",
            br#"{"object":"text_completion","model":"a","choices":[{"text":"y","index":0,"finish_reason":"stop"}]}"#,
        )),
    ];
    let mut server = Server::spawn(responses).await;
    let adapter = provider(server.base(), None);
    let provider: &dyn LlmProvider = &adapter;
    let policy = ProviderExecutionPolicy::default();
    let never = NeverCancelled;

    assert!(
        provider
            .discover_models(context(&policy, &never))
            .await
            .expect("empty catalog must succeed")
            .is_empty()
    );
    assert_eq!(
        provider
            .discover_models(context(&policy, &never))
            .await
            .expect("second discovery must be fresh")
            .models()
            .len(),
        1
    );
    let generation = request("openai-compatible", "a", "input", 1);
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
        let captured = String::from_utf8(server.request().await).expect("request must be text");
        assert!(!captured.contains("authorization:"));
    }
    server.finish().await;
}

#[tokio::test]
async fn public_discovery_rejects_missing_malformed_duplicate_over_count_and_over_body() {
    let over_count = (0..=MAX_MODELS_PER_CATALOG)
        .map(|index| format!(r#"{{"id":"model-{index}"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let oversized = vec![b'x'; 1_024 * 1_024 + 1];
    let cases = vec![
        (b"not-json".to_vec(), LlmErrorKind::Protocol),
        (br#"{"object":"list"}"#.to_vec(), LlmErrorKind::Protocol),
        (
            br#"{"object":"wrong","data":[]}"#.to_vec(),
            LlmErrorKind::Protocol,
        ),
        (
            br#"{"object":"list","data":[{}]}"#.to_vec(),
            LlmErrorKind::Protocol,
        ),
        (
            br#"{"object":"list","data":[{"id":""}]}"#.to_vec(),
            LlmErrorKind::InvalidModelCatalog,
        ),
        (
            br#"{"object":"list","data":[{"id":"a"},{"id":"a"}]}"#.to_vec(),
            LlmErrorKind::InvalidModelCatalog,
        ),
        (
            format!(r#"{{"object":"list","data":[{over_count}]}}"#).into_bytes(),
            LlmErrorKind::InvalidModelCatalog,
        ),
        (oversized, LlmErrorKind::Protocol),
    ];

    for (body, expected) in cases {
        let mut server = Server::spawn(vec![Some(response_without_length("200 OK", &body))]).await;
        let adapter = provider(server.base(), None);
        let provider: &dyn LlmProvider = &adapter;
        let policy = ProviderExecutionPolicy::default();
        let never = NeverCancelled;
        let error = provider
            .discover_models(context(&policy, &never))
            .await
            .expect_err("invalid discovery response must fail atomically");
        assert_eq!(error.kind(), expected);
        assert!(!server.request().await.is_empty());
        server.finish().await;
    }
}

#[tokio::test]
async fn public_provider_rejects_fallback_malformed_bounds_status_and_redirect_without_retry() {
    let oversized = vec![b'x'; 512 * 1_024 + 1];
    let cases = vec![
        (response("200 OK", b"not-json"), LlmErrorKind::Protocol),
        (
            response(
                "200 OK",
                br#"{"object":"text_completion","model":"fallback","choices":[{"text":"x","index":0,"finish_reason":"stop"}]}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"object":"text_completion","model":"a","choices":[{"text":"xx","index":0,"finish_reason":"stop"}]}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"object":"text_completion","model":"a","choices":[]}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"object":"text_completion","model":"a","choices":[{"text":"x","index":0,"finish_reason":"stop"},{"text":"y","index":1,"finish_reason":"stop"}]}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"object":"text_completion","model":"a","choices":[{"text":"x","index":1,"finish_reason":"stop"}]}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"object":"text_completion","model":"a","choices":[{"text":"x","index":0,"finish_reason":"future"}]}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (
            response(
                "200 OK",
                br#"{"object":"text_completion","model":"a","choices":[{"text":"","index":0,"finish_reason":"stop"}]}"#,
            ),
            LlmErrorKind::InvalidResponse,
        ),
        (response("503 Test", b"provider body"), LlmErrorKind::ProviderUnavailable),
        (response("400 Test", b"provider body"), LlmErrorKind::ProviderRejected),
        (
            response_with_headers(
                "302 Found",
                &["Location: http://external.invalid/v1/completions"],
                b"provider body",
            ),
            LlmErrorKind::ProviderRejected,
        ),
        (response_without_length("200 OK", &oversized), LlmErrorKind::Protocol),
    ];

    for (wire, expected) in cases {
        let mut server = Server::spawn(vec![Some(wire)]).await;
        let adapter = provider(server.base(), None);
        let provider: &dyn LlmProvider = &adapter;
        let request = request("openai-compatible", "a", "input", 1);
        let policy = ProviderExecutionPolicy::default();
        let never = NeverCancelled;
        let error = provider
            .generate(&request, context(&policy, &never))
            .await
            .expect_err("invalid terminal response must fail");
        assert_eq!(error.kind(), expected);
        assert!(!server.request().await.is_empty());
        server.finish().await;
    }
}

#[tokio::test]
async fn public_timeout_cancellation_identity_redaction_and_cleanup_are_terminal() {
    let policy = ProviderExecutionPolicy::default();
    let never = NeverCancelled;
    let unreachable = provider("http://127.0.0.1:9", None);
    let llm: &dyn LlmProvider = &unreachable;
    let wrong = request("other", "a", "input", 1);
    assert_eq!(
        llm.generate(&wrong, context(&policy, &never))
            .await
            .expect_err("identity mismatch must fail")
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
            .expect_err("closed loopback port must be a transport failure")
            .kind(),
        LlmErrorKind::Transport
    );

    let mut timeout_server = Server::spawn(vec![None]).await;
    let adapter = provider(timeout_server.base(), None);
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
    let adapter = provider(cancel_server.base(), None);
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
    let adapter = provider(server.base(), Some(sentinel));
    let llm: &dyn LlmProvider = &adapter;
    let sensitive = request("openai-compatible", "a", sentinel, 1);
    let error = llm
        .generate(&sensitive, context(&policy, &never))
        .await
        .expect_err("provider rejection must fail");
    assert_redacted(&error, sentinel);
    server.finish().await;
}

fn construction_error(result: Result<OpenAiCompatibleProvider, LlmError>) -> LlmError {
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

async fn read_request(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
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
