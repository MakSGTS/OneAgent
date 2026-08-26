use oneagent_llm::{
    LlmError, LlmErrorKind, MAX_MODELS_PER_CATALOG, ModelCapability, ModelCatalog, ModelDescriptor,
    ModelId, ModelIdentity, ProviderExecutionContext,
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};

use crate::{
    MAX_SHOW_REQUEST_BODY_BYTES, MAX_SHOW_RESPONSE_BODY_BYTES, MAX_TAGS_RESPONSE_BODY_BYTES,
    config::OllamaProvider,
    execution::{adapter_error, bounded_success_body, run_with_context, status_error},
    wire::{OptionalString, ShowRequest, ShowResponse, TagsResponse},
};

impl OllamaProvider {
    pub(crate) async fn discover_models(
        &self,
        context: ProviderExecutionContext<'_>,
    ) -> Result<ModelCatalog, LlmError> {
        if context.cancellation().is_cancelled() {
            return Err(adapter_error(
                LlmErrorKind::Cancelled,
                "provider operation was cancelled",
            ));
        }
        run_with_context(context, self.execute_discovery()).await
    }

    async fn execute_discovery(&self) -> Result<ModelCatalog, LlmError> {
        let response = self
            .client()
            .get(self.tags_url().clone())
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| {
                adapter_error(LlmErrorKind::Transport, "provider request transport failed")
            })?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }

        let body = bounded_success_body(response, MAX_TAGS_RESPONSE_BODY_BYTES).await?;
        let wire: TagsResponse = serde_json::from_slice(&body).map_err(|_| {
            adapter_error(
                LlmErrorKind::Protocol,
                "provider tags response is not valid JSON",
            )
        })?;
        drop(body);
        let candidates = decode_candidates(wire)?;

        let mut models = Vec::with_capacity(candidates.len());
        for model in candidates {
            let body = serde_json::to_vec(&ShowRequest {
                model: model.as_str(),
                verbose: false,
            })
            .map_err(|_| {
                adapter_error(
                    LlmErrorKind::Internal,
                    "provider show request serialization failed",
                )
            })?;
            if body.len() > MAX_SHOW_REQUEST_BODY_BYTES {
                return Err(adapter_error(
                    LlmErrorKind::InvalidRequest,
                    "provider show request exceeds byte limit",
                ));
            }

            let response = self
                .client()
                .post(self.show_url().clone())
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT, "application/json")
                .body(body)
                .send()
                .await
                .map_err(|_| {
                    adapter_error(LlmErrorKind::Transport, "provider request transport failed")
                })?;
            if !response.status().is_success() {
                return Err(status_error(response.status()));
            }

            let body = bounded_success_body(response, MAX_SHOW_RESPONSE_BODY_BYTES).await?;
            let wire: ShowResponse = serde_json::from_slice(&body).map_err(|_| {
                adapter_error(
                    LlmErrorKind::Protocol,
                    "provider show response is not valid JSON",
                )
            })?;
            drop(body);
            if wire
                .capabilities
                .iter()
                .any(|capability| capability == "completion")
            {
                models.push(ModelDescriptor::new(
                    ModelIdentity::new(self.id().clone(), model),
                    [ModelCapability::TextGeneration],
                ));
            }
        }

        ModelCatalog::new(self.id().clone(), models).map_err(|_| {
            adapter_error(
                LlmErrorKind::Internal,
                "provider model catalog construction failed",
            )
        })
    }
}

fn decode_candidates(wire: TagsResponse) -> Result<Vec<ModelId>, LlmError> {
    if wire.models.len() > MAX_MODELS_PER_CATALOG {
        return Err(adapter_error(
            LlmErrorKind::InvalidModelCatalog,
            "provider model catalog exceeds model count limit",
        ));
    }

    let mut candidates = Vec::with_capacity(wire.models.len());
    for entry in wire.models {
        if entry.name != entry.model {
            return Err(adapter_error(
                LlmErrorKind::Protocol,
                "provider tags identities conflict",
            ));
        }
        let model = ModelId::new(entry.name).map_err(|_| {
            adapter_error(
                LlmErrorKind::InvalidModelCatalog,
                "provider model catalog contains an invalid model identifier",
            )
        })?;

        match (entry.remote_model, entry.remote_host) {
            (OptionalString::Missing, OptionalString::Missing) => candidates.push(model),
            (OptionalString::Present(remote_model), OptionalString::Present(remote_host))
                if !remote_model.is_empty() && !remote_host.is_empty() => {}
            _ => {
                return Err(adapter_error(
                    LlmErrorKind::Protocol,
                    "provider tags remote markers conflict",
                ));
            }
        }
    }

    candidates.sort_unstable();
    if candidates.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(adapter_error(
            LlmErrorKind::InvalidModelCatalog,
            "provider model catalog contains a duplicate model identifier",
        ));
    }
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };

    use oneagent_llm::{
        CancellationSignal, LlmErrorKind, ModelCapability, NeverCancelled, ProviderConfiguration,
        ProviderDiagnostic, ProviderExecutionContext, ProviderExecutionPolicy, ProviderFuture,
        ProviderId,
    };
    use serde_json::{Value, json};
    use tokio::sync::Notify;

    use super::OllamaProvider;
    use crate::{
        MAX_SHOW_RESPONSE_BODY_BYTES, MAX_TAGS_RESPONSE_BODY_BYTES,
        test_support::{ControlledServer, http_response, raw_http_response},
    };

    fn provider(base_url: &str) -> OllamaProvider {
        let configuration = ProviderConfiguration::new(
            ProviderId::new("ollama").expect("provider ID must pass"),
            None,
        );
        OllamaProvider::new(configuration, base_url).expect("provider must construct")
    }

    fn context<'a>(
        policy: &'a ProviderExecutionPolicy,
        cancellation: &'a dyn CancellationSignal,
    ) -> ProviderExecutionContext<'a> {
        ProviderExecutionContext::new(policy, cancellation)
    }

    fn tags(models: impl IntoIterator<Item = Value>) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "models": models.into_iter().collect::<Vec<_>>(),
            "future": true
        }))
        .expect("tags fixture must serialize")
    }

    fn local(model: &str) -> Value {
        json!({"name": model, "model": model, "capabilities": ["completion"], "future": true})
    }

    fn remote(model: &str) -> Value {
        json!({
            "name": model,
            "model": model,
            "remote_model": "opaque-remote-model",
            "remote_host": "opaque-remote-host"
        })
    }

    fn show(capabilities: &[&str]) -> Vec<u8> {
        serde_json::to_vec(&json!({"capabilities": capabilities, "future": true}))
            .expect("show fixture must serialize")
    }

    fn request_parts(request: &[u8]) -> (&str, Value) {
        let header_end = request
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("request headers must terminate")
            + 4;
        let headers = std::str::from_utf8(&request[..header_end]).expect("headers must be UTF-8");
        let body = if request.len() == header_end {
            Value::Null
        } else {
            serde_json::from_slice(&request[header_end..]).expect("body must be JSON")
        };
        (headers, body)
    }

    #[tokio::test]
    async fn discovery_sends_exact_tags_and_canonical_show_wires() {
        let responses = vec![
            Some(http_response(
                200,
                &tags([local("z"), remote("cloud"), local("b"), local("a")]),
            )),
            Some(http_response(200, &show(&["completion", "tools"]))),
            Some(http_response(200, &show(&["embedding", "vision"]))),
            Some(http_response(200, &show(&["completion", "completion"]))),
        ];
        let mut server = ControlledServer::spawn(responses).await;
        let provider = provider(server.base_url());
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let catalog = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("discovery must succeed");
        assert_eq!(catalog.provider(), provider.id());
        assert_eq!(catalog.models().len(), 2);
        assert_eq!(catalog.models()[0].identity().model().as_str(), "a");
        assert_eq!(catalog.models()[1].identity().model().as_str(), "z");
        assert!(catalog.models().iter().all(|model| {
            model.capabilities().len() == 1 && model.supports(ModelCapability::TextGeneration)
        }));

        let tags_request = server.next_request().await;
        let (headers, body) = request_parts(&tags_request);
        assert!(headers.starts_with("GET /api/tags HTTP/1.1\r\n"));
        assert!(headers.contains("accept: application/json\r\n"));
        assert!(headers.contains("user-agent: oneagent-ollama/0.1.0\r\n"));
        assert!(!headers.contains("authorization:"));
        assert!(!headers.contains("content-type:"));
        assert_eq!(body, Value::Null);

        for expected in ["a", "b", "z"] {
            let request = server.next_request().await;
            let (headers, body) = request_parts(&request);
            assert!(headers.starts_with("POST /api/show HTTP/1.1\r\n"));
            assert!(headers.contains("accept: application/json\r\n"));
            assert!(headers.contains("content-type: application/json\r\n"));
            assert!(headers.contains("user-agent: oneagent-ollama/0.1.0\r\n"));
            assert!(!headers.contains("authorization:"));
            assert_eq!(body, json!({"model": expected, "verbose": false}));
        }
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_accepts_empty_remote_only_and_repeated_fresh_calls() {
        let responses = vec![
            Some(http_response(200, &tags(Vec::new()))),
            Some(http_response(200, &tags([remote("cloud-only")]))),
        ];
        let mut server = ControlledServer::spawn(responses).await;
        let provider = provider(server.base_url());
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let first = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("empty tags must succeed");
        let second = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("remote-only tags must succeed");
        assert!(first.is_empty());
        assert!(second.is_empty());
        assert!(!server.next_request().await.is_empty());
        assert!(!server.next_request().await.is_empty());
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_accepts_the_maximum_local_catalog() {
        let entries = (0..oneagent_llm::MAX_MODELS_PER_CATALOG)
            .map(|index| local(&format!("model-{index:04}")))
            .collect::<Vec<_>>();
        let mut responses = Vec::with_capacity(entries.len() + 1);
        responses.push(Some(http_response(200, &tags(entries))));
        responses.extend(
            (0..oneagent_llm::MAX_MODELS_PER_CATALOG)
                .map(|_| Some(http_response(200, &show(&["completion"])))),
        );
        let mut server = ControlledServer::spawn(responses).await;
        let provider = provider(server.base_url());
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let catalog = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("maximum catalog must succeed");
        assert_eq!(catalog.models().len(), oneagent_llm::MAX_MODELS_PER_CATALOG);
        for _ in 0..=oneagent_llm::MAX_MODELS_PER_CATALOG {
            assert!(!server.next_request().await.is_empty());
        }
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_rejects_invalid_tags_before_any_show() {
        let over_count = (0..=oneagent_llm::MAX_MODELS_PER_CATALOG)
            .map(|index| local(&format!("model-{index}")))
            .collect::<Vec<_>>();
        let cases = vec![
            (b"not-json".to_vec(), LlmErrorKind::Protocol),
            (
                br#"{"models":[]} trailing"#.to_vec(),
                LlmErrorKind::Protocol,
            ),
            (br"{}".to_vec(), LlmErrorKind::Protocol),
            (tags([json!({"name": "a"})]), LlmErrorKind::Protocol),
            (
                tags([json!({"name": "a", "model": "b"})]),
                LlmErrorKind::Protocol,
            ),
            (tags([local("")]), LlmErrorKind::InvalidModelCatalog),
            (
                tags([local("duplicate"), local("duplicate")]),
                LlmErrorKind::InvalidModelCatalog,
            ),
            (tags(over_count), LlmErrorKind::InvalidModelCatalog),
            (
                tags([json!({"name": "a", "model": "a", "remote_model": "remote"})]),
                LlmErrorKind::Protocol,
            ),
            (
                tags([json!({
                    "name": "a", "model": "a", "remote_model": "", "remote_host": "host"
                })]),
                LlmErrorKind::Protocol,
            ),
            (
                tags([json!({
                    "name": "a", "model": "a", "remote_model": null, "remote_host": "host"
                })]),
                LlmErrorKind::Protocol,
            ),
        ];

        for (body, expected) in cases {
            let server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
            let provider = provider(server.base_url());
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .discover_models(context(&policy, &cancellation))
                .await
                .expect_err("invalid tags must fail");
            assert_eq!(error.kind(), expected);
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn discovery_rejects_invalid_show_and_late_failure_atomically() {
        let cases = [
            (b"not-json".to_vec(), LlmErrorKind::Protocol),
            (
                br#"{"capabilities":[]} trailing"#.to_vec(),
                LlmErrorKind::Protocol,
            ),
            (br"{}".to_vec(), LlmErrorKind::Protocol),
            (br#"{"capabilities":null}"#.to_vec(), LlmErrorKind::Protocol),
            (
                br#"{"capabilities":["completion",1]}"#.to_vec(),
                LlmErrorKind::Protocol,
            ),
        ];
        for (body, expected) in cases {
            let server = ControlledServer::spawn(vec![
                Some(http_response(200, &tags([local("a")]))),
                Some(http_response(200, &body)),
            ])
            .await;
            let provider = provider(server.base_url());
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .discover_models(context(&policy, &cancellation))
                .await
                .expect_err("invalid show must fail");
            assert_eq!(error.kind(), expected);
            server.finish().await;
        }

        let server = ControlledServer::spawn(vec![
            Some(http_response(200, &tags([local("a"), local("b")]))),
            Some(http_response(200, &show(&["completion"]))),
            Some(http_response(503, b"opaque body")),
        ])
        .await;
        let provider = provider(server.base_url());
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;
        let error = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect_err("late show failure must reject the whole catalog");
        assert_eq!(error.kind(), LlmErrorKind::ProviderUnavailable);
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_maps_status_redirect_partial_and_body_bounds_without_retry() {
        let cases = [
            (
                http_response(408, b"opaque"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(429, b"opaque"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(503, b"opaque"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(400, b"opaque"),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "302 Found",
                    &["Location: http://127.0.0.1:9/api/tags"],
                    b"opaque",
                ),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[&format!(
                        "Content-Length: {}",
                        MAX_TAGS_RESPONSE_BODY_BYTES + 1
                    )],
                    b"",
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response("200 OK", &[], &vec![b'x'; MAX_TAGS_RESPONSE_BODY_BYTES + 1]),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response("200 OK", &["Content-Length: 100"], br#"{"models":[]"#),
                LlmErrorKind::Transport,
            ),
        ];

        for (response, expected) in cases {
            let server = ControlledServer::spawn(vec![Some(response)]).await;
            let provider = provider(server.base_url());
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .discover_models(context(&policy, &cancellation))
                .await
                .expect_err("terminal tags response must fail");
            assert_eq!(error.kind(), expected);
            server.finish().await;
        }

        let server = ControlledServer::spawn(vec![
            Some(http_response(200, &tags([local("a")]))),
            Some(raw_http_response(
                "200 OK",
                &[&format!(
                    "Content-Length: {}",
                    MAX_SHOW_RESPONSE_BODY_BYTES + 1
                )],
                b"",
            )),
        ])
        .await;
        let provider = provider(server.base_url());
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;
        let error = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect_err("over-bound show must fail");
        assert_eq!(error.kind(), LlmErrorKind::Protocol);
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_enforces_timeout_cancellation_transport_redaction_and_cleanup() {
        let mut timeout_server = ControlledServer::spawn(vec![None]).await;
        let timeout_provider = provider(timeout_server.base_url());
        let timeout_policy = ProviderExecutionPolicy::new(Some(Duration::from_millis(20)))
            .expect("timeout policy must pass");
        let never = NeverCancelled;
        let error = timeout_provider
            .discover_models(context(&timeout_policy, &never))
            .await
            .expect_err("pending tags must time out");
        assert_eq!(error.kind(), LlmErrorKind::Timeout);
        assert!(!timeout_server.next_request().await.is_empty());
        timeout_server.finish().await;

        let mut cancel_server =
            ControlledServer::spawn(vec![Some(http_response(200, &tags([local("a")]))), None])
                .await;
        let cancel_provider = provider(cancel_server.base_url());
        let signal = TestCancellation::default();
        let policy = ProviderExecutionPolicy::default();
        let operation = cancel_provider.discover_models(context(&policy, &signal));
        tokio::pin!(operation);
        for _ in 0..2 {
            tokio::select! {
                request = cancel_server.next_request() => assert!(!request.is_empty()),
                result = &mut operation => panic!("operation completed before cancellation: {:?}", result.err().map(|error| error.kind())),
            }
        }
        signal.cancel();
        let error = operation
            .await
            .expect_err("in-flight cancellation must win");
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);
        cancel_server.finish().await;

        let pre_cancelled = TestCancellation::default();
        pre_cancelled.cancel();
        let unreachable = provider("http://127.0.0.1:9");
        let error = unreachable
            .discover_models(context(&policy, &pre_cancelled))
            .await
            .expect_err("existing cancellation must precede transport");
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);
        let error = unreachable
            .discover_models(context(&policy, &never))
            .await
            .expect_err("closed loopback port must be transport failure");
        assert_eq!(error.kind(), LlmErrorKind::Transport);

        let sentinel = "synthetic-provider-body-sentinel";
        let server =
            ControlledServer::spawn(vec![Some(http_response(400, sentinel.as_bytes()))]).await;
        let provider = provider(server.base_url());
        let error = provider
            .discover_models(context(&policy, &never))
            .await
            .expect_err("provider rejection must fail");
        assert_eq!(error.kind(), LlmErrorKind::ProviderRejected);
        assert!(!format!("{error}").contains(sentinel));
        assert!(!format!("{error:?}").contains(sentinel));
        assert!(
            !error
                .diagnostic()
                .map(ProviderDiagnostic::as_str)
                .unwrap_or_default()
                .contains(sentinel)
        );
        server.finish().await;
    }

    #[derive(Default)]
    struct TestCancellation {
        cancelled: AtomicBool,
        notify: Notify,
    }

    impl TestCancellation {
        fn cancel(&self) {
            self.cancelled.store(true, Ordering::SeqCst);
            self.notify.notify_waiters();
        }
    }

    impl CancellationSignal for TestCancellation {
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

    #[test]
    fn discovery_cancellation_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<TestCancellation>();
    }
}
