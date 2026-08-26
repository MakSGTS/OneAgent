use oneagent_llm::{
    LlmError, LlmErrorKind, MAX_MODELS_PER_CATALOG, ModelCapability, ModelCatalog, ModelDescriptor,
    ModelId, ModelIdentity, ProviderExecutionContext,
};
use reqwest::header::ACCEPT;

use crate::{
    MAX_NATIVE_MODELS_RESPONSE_BODY_BYTES,
    config::{LmStudioProvider, apply_authorization},
    execution::{adapter_error, bounded_success_body, run_with_context, status_error},
    wire::ModelsResponse,
};

impl LmStudioProvider {
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
        let request = self
            .native_client()
            .get(self.native_models_url().clone())
            .header(ACCEPT, "application/json");
        let request = apply_authorization(request, self.native_authorization());
        let response = request.send().await.map_err(|_| {
            adapter_error(LlmErrorKind::Transport, "provider request transport failed")
        })?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }

        let body = bounded_success_body(response, MAX_NATIVE_MODELS_RESPONSE_BODY_BYTES).await?;
        let wire: ModelsResponse = serde_json::from_slice(&body).map_err(|_| {
            adapter_error(
                LlmErrorKind::Protocol,
                "provider discovery response is not valid JSON",
            )
        })?;

        let mut models = Vec::new();
        for entry in wire.models {
            let _ = entry.key;
            match entry.model_type.as_str() {
                "embedding" => continue,
                "llm" => {}
                _ => {
                    return Err(adapter_error(
                        LlmErrorKind::Protocol,
                        "provider discovery response model type is invalid",
                    ));
                }
            }

            for instance in entry.loaded_instances {
                if models.len() == MAX_MODELS_PER_CATALOG {
                    return Err(adapter_error(
                        LlmErrorKind::InvalidModelCatalog,
                        "provider model catalog exceeds model count limit",
                    ));
                }
                let model = ModelId::new(instance.id).map_err(|_| {
                    adapter_error(
                        LlmErrorKind::InvalidModelCatalog,
                        "provider model catalog contains an invalid model identifier",
                    )
                })?;
                models.push(ModelDescriptor::new(
                    ModelIdentity::new(self.id().clone(), model),
                    [ModelCapability::TextGeneration],
                ));
            }
        }

        ModelCatalog::new(self.id().clone(), models)
    }
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
        ProviderId, ProviderSecret,
    };
    use serde_json::{Value, json};
    use tokio::sync::Notify;

    use super::LmStudioProvider;
    use crate::{
        MAX_NATIVE_MODELS_RESPONSE_BODY_BYTES,
        test_support::{ControlledServer, http_response, raw_http_response},
    };

    fn provider(base_url: &str, secret: Option<&str>) -> LmStudioProvider {
        let configuration = ProviderConfiguration::new(
            ProviderId::new("lm-studio").expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
        );
        LmStudioProvider::new(configuration, base_url).expect("provider must construct")
    }

    fn context<'a>(
        policy: &'a ProviderExecutionPolicy,
        cancellation: &'a dyn CancellationSignal,
    ) -> ProviderExecutionContext<'a> {
        ProviderExecutionContext::new(policy, cancellation)
    }

    fn response_body(models: impl IntoIterator<Item = Value>) -> Vec<u8> {
        let models = models.into_iter().collect::<Vec<_>>();
        serde_json::to_vec(&json!({"models": models, "future": true}))
            .expect("response fixture must serialize")
    }

    fn entry(model_type: &str, key: &str, instances: impl IntoIterator<Item = Value>) -> Value {
        let instances = instances.into_iter().collect::<Vec<_>>();
        json!({
            "type": model_type,
            "key": key,
            "loaded_instances": instances,
            "future": true
        })
    }

    #[tokio::test]
    async fn discovery_sends_exact_wire_and_projects_only_loaded_llm_instances() {
        let body = response_body(vec![
            entry(
                "embedding",
                "embedding-download",
                vec![json!({"id": "embedding-loaded"})],
            ),
            entry(
                "llm",
                "downloaded-parent-key",
                vec![
                    json!({"id": "z-custom-instance", "future": true}),
                    json!({"id": "a-custom-instance"}),
                ],
            ),
            entry("llm", "unloaded-llm", Vec::new()),
        ]);
        let mut server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
        let provider = provider(server.base_url(), Some("synthetic-secret-sentinel"));
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let catalog = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("native discovery must succeed");

        assert_eq!(catalog.provider(), provider.id());
        assert_eq!(catalog.models().len(), 2);
        assert_eq!(
            catalog.models()[0].identity().model().as_str(),
            "a-custom-instance"
        );
        assert_eq!(
            catalog.models()[1].identity().model().as_str(),
            "z-custom-instance"
        );
        assert!(catalog.models().iter().all(|model| {
            model.capabilities().len() == 1 && model.supports(ModelCapability::TextGeneration)
        }));

        let request = server.next_request().await;
        let request = String::from_utf8(request).expect("request must be UTF-8");
        assert!(request.starts_with("GET /api/v1/models HTTP/1.1\r\n"));
        assert!(request.contains("accept: application/json\r\n"));
        assert!(request.contains("user-agent: oneagent-lm-studio/0.1.0\r\n"));
        assert!(request.contains("authorization: Bearer synthetic-secret-sentinel\r\n"));
        assert!(!request.contains("content-length:"));
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_accepts_empty_maximum_and_repeats_fresh_calls() {
        let maximum_instances: Vec<_> = (0..oneagent_llm::MAX_MODELS_PER_CATALOG)
            .map(|index| json!({"id": format!("model-{index:04}")}))
            .collect();
        let responses = vec![
            Some(http_response(200, &response_body(Vec::new()))),
            Some(http_response(
                200,
                &response_body(vec![
                    entry(
                        "embedding",
                        "embedding-only",
                        vec![json!({"id": "ignored-embedding-instance"})],
                    ),
                    entry("llm", "unloaded", Vec::new()),
                ]),
            )),
            Some(http_response(
                200,
                &response_body(vec![entry("llm", "parent", maximum_instances)]),
            )),
        ];
        let mut server = ControlledServer::spawn(responses).await;
        let provider = provider(server.base_url(), None);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let first = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("empty catalog must pass");
        let second = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("unsupported entries must produce empty catalog");
        let maximum = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("maximum catalog must pass");

        assert!(first.is_empty());
        assert!(second.is_empty());
        assert_eq!(maximum.models().len(), oneagent_llm::MAX_MODELS_PER_CATALOG);
        assert!(!server.next_request().await.is_empty());
        assert!(!server.next_request().await.is_empty());
        assert!(!server.next_request().await.is_empty());
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_rejects_malformed_unknown_and_ambiguous_catalogs_atomically() {
        let over_count: Vec<_> = (0..=oneagent_llm::MAX_MODELS_PER_CATALOG)
            .map(|index| json!({"id": format!("model-{index}")}))
            .collect();
        let cases = vec![
            (b"not-json".to_vec(), LlmErrorKind::Protocol),
            (
                br#"{"models":[] } trailing"#.to_vec(),
                LlmErrorKind::Protocol,
            ),
            (
                serde_json::to_vec(&json!({})).expect("fixture must serialize"),
                LlmErrorKind::Protocol,
            ),
            (response_body(vec![json!({})]), LlmErrorKind::Protocol),
            (
                response_body(vec![entry("future", "model", Vec::new())]),
                LlmErrorKind::Protocol,
            ),
            (
                response_body(vec![json!({
                    "type": "llm",
                    "key": "model",
                    "loaded_instances": [{}]
                })]),
                LlmErrorKind::Protocol,
            ),
            (
                response_body(vec![entry("llm", "model", vec![json!({"id": ""})])]),
                LlmErrorKind::InvalidModelCatalog,
            ),
            (
                response_body(vec![entry(
                    "llm",
                    "model",
                    vec![json!({"id": "duplicate"}), json!({"id": "duplicate"})],
                )]),
                LlmErrorKind::InvalidModelCatalog,
            ),
            (
                response_body(vec![entry("llm", "model", over_count)]),
                LlmErrorKind::InvalidModelCatalog,
            ),
        ];

        for (body, expected) in cases {
            let server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
            let provider = provider(server.base_url(), None);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .discover_models(context(&policy, &cancellation))
                .await
                .expect_err("invalid native catalog must fail");

            assert_eq!(error.kind(), expected);
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn discovery_maps_status_redirect_partial_and_body_bounds_without_retry() {
        let cases = [
            (
                http_response(408, b"provider body"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(429, b"provider body"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(503, b"provider body"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(400, b"provider body"),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "302 Found",
                    &["Location: http://external.invalid/api/v1/models"],
                    b"provider body",
                ),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[&format!(
                        "Content-Length: {}",
                        MAX_NATIVE_MODELS_RESPONSE_BODY_BYTES + 1
                    )],
                    b"",
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[],
                    &vec![b'x'; MAX_NATIVE_MODELS_RESPONSE_BODY_BYTES + 1],
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response("200 OK", &["Content-Length: 100"], br#"{"models":["#),
                LlmErrorKind::Transport,
            ),
        ];

        for (response, expected) in cases {
            let mut server = ControlledServer::spawn(vec![Some(response)]).await;
            let provider = provider(server.base_url(), None);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .discover_models(context(&policy, &cancellation))
                .await
                .expect_err("terminal native response must fail");

            assert_eq!(error.kind(), expected);
            assert!(!server.next_request().await.is_empty());
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn discovery_enforces_timeout_cancellation_transport_redaction_and_cleanup() {
        let mut timeout_server = ControlledServer::spawn(vec![None]).await;
        let timeout_provider = provider(timeout_server.base_url(), None);
        let timeout_policy = ProviderExecutionPolicy::new(Some(Duration::from_millis(20)))
            .expect("timeout policy must pass");
        let never = NeverCancelled;
        let error = timeout_provider
            .discover_models(context(&timeout_policy, &never))
            .await
            .expect_err("pending response must time out");
        assert_eq!(error.kind(), LlmErrorKind::Timeout);
        assert!(!timeout_server.next_request().await.is_empty());
        timeout_server.finish().await;

        let mut cancel_server = ControlledServer::spawn(vec![None]).await;
        let cancel_provider = provider(cancel_server.base_url(), None);
        let signal = TestCancellation::default();
        let policy = ProviderExecutionPolicy::default();
        let operation = cancel_provider.discover_models(context(&policy, &signal));
        tokio::pin!(operation);
        tokio::select! {
            request = cancel_server.next_request() => assert!(!request.is_empty()),
            result = &mut operation => panic!("operation completed before cancellation: {:?}", result.err().map(|error| error.kind())),
        }
        signal.cancel();
        let error = operation
            .await
            .expect_err("in-flight cancellation must win");
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);
        cancel_server.finish().await;

        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .expect("temporary listener must bind");
        let closed_address = listener
            .local_addr()
            .expect("temporary listener address must exist");
        drop(listener);
        let unreachable = provider(&format!("http://{closed_address}"), None);

        let pre_cancelled = TestCancellation::default();
        pre_cancelled.cancel();
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

        let sentinel = "synthetic-secret-and-provider-body-sentinel";
        let server =
            ControlledServer::spawn(vec![Some(http_response(400, sentinel.as_bytes()))]).await;
        let provider = provider(server.base_url(), Some(sentinel));
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

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn cancellation_is_send_and_sync() {
        assert_send_sync::<TestCancellation>();
    }
}
