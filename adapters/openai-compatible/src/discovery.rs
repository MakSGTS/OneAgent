use oneagent_llm::{
    LlmError, LlmErrorKind, LlmProvider, MAX_MODELS_PER_CATALOG, ModelCapability, ModelCatalog,
    ModelDescriptor, ModelId, ModelIdentity, ProviderExecutionContext, ProviderFuture,
    TextGenerationRequest, TextGenerationResponse,
};
use reqwest::header::ACCEPT;

use crate::{
    MAX_MODELS_RESPONSE_BODY_BYTES,
    config::{OpenAiCompatibleProvider, apply_authorization},
    execution::{adapter_error, bounded_success_body, run_with_context, status_error},
    wire::ModelsResponse,
};

impl LlmProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &oneagent_llm::ProviderId {
        self.id()
    }

    fn discover_models<'a>(
        &'a self,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<ModelCatalog, LlmError>> {
        Box::pin(async move {
            if context.cancellation().is_cancelled() {
                return Err(adapter_error(
                    LlmErrorKind::Cancelled,
                    "provider operation was cancelled",
                ));
            }
            run_with_context(context, self.execute_discovery()).await
        })
    }

    fn generate<'a>(
        &'a self,
        request: &'a TextGenerationRequest,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<TextGenerationResponse, LlmError>> {
        Box::pin(async move {
            if request.model().provider() != self.id() {
                return Err(adapter_error(
                    LlmErrorKind::InvalidRequest,
                    "request provider does not match adapter",
                ));
            }
            if context.cancellation().is_cancelled() {
                return Err(adapter_error(
                    LlmErrorKind::Cancelled,
                    "provider operation was cancelled",
                ));
            }
            Err(adapter_error(
                LlmErrorKind::Internal,
                "text generation is not implemented",
            ))
        })
    }
}

impl OpenAiCompatibleProvider {
    async fn execute_discovery(&self) -> Result<ModelCatalog, LlmError> {
        let request = self
            .client()
            .get(self.models_url().clone())
            .header(ACCEPT, "application/json");
        let request = apply_authorization(request, self.authorization());
        let response = request.send().await.map_err(|_| {
            adapter_error(LlmErrorKind::Transport, "provider request transport failed")
        })?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }

        let body = bounded_success_body(response, MAX_MODELS_RESPONSE_BODY_BYTES).await?;
        let wire: ModelsResponse = serde_json::from_slice(&body).map_err(|_| {
            adapter_error(
                LlmErrorKind::Protocol,
                "provider discovery response is not valid JSON",
            )
        })?;
        if wire.object != "list" {
            return Err(adapter_error(
                LlmErrorKind::Protocol,
                "provider discovery response object is invalid",
            ));
        }
        if wire.data.len() > MAX_MODELS_PER_CATALOG {
            return Err(adapter_error(
                LlmErrorKind::InvalidModelCatalog,
                "provider model catalog exceeds model count limit",
            ));
        }

        let mut models = Vec::with_capacity(wire.data.len());
        for entry in wire.data {
            let model = ModelId::new(entry.id).map_err(|_| {
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
        CancellationSignal, LlmErrorKind, LlmProvider, ModelCapability, NeverCancelled,
        ProviderConfiguration, ProviderDiagnostic, ProviderExecutionContext,
        ProviderExecutionPolicy, ProviderFuture, ProviderId, ProviderSecret,
    };
    use tokio::sync::Notify;

    use super::OpenAiCompatibleProvider;
    use crate::{
        MAX_MODELS_RESPONSE_BODY_BYTES,
        test_support::{ControlledServer, http_response, raw_http_response},
    };

    fn provider(base_url: &str, secret: Option<&str>) -> OpenAiCompatibleProvider {
        let configuration = ProviderConfiguration::new(
            ProviderId::new("openai-compatible").expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
        );
        OpenAiCompatibleProvider::new(configuration, base_url).expect("provider must construct")
    }

    fn context<'a>(
        policy: &'a ProviderExecutionPolicy,
        cancellation: &'a dyn CancellationSignal,
    ) -> ProviderExecutionContext<'a> {
        ProviderExecutionContext::new(policy, cancellation)
    }

    #[tokio::test]
    async fn discovery_sends_exact_authenticated_request_and_canonicalizes_models() {
        let body = br#"{"object":"list","data":[{"id":"z","future":1},{"id":"a"}],"future":true}"#;
        let mut server = ControlledServer::spawn(vec![Some(http_response(200, body))]).await;
        let provider = provider(server.base_url(), Some("synthetic-secret-sentinel"));
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let catalog = (&provider as &dyn LlmProvider)
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

        let request = server.next_request().await;
        let request = String::from_utf8(request).expect("request must be UTF-8");
        assert!(request.starts_with("GET /v1/models HTTP/1.1\r\n"));
        assert!(request.contains("accept: application/json\r\n"));
        assert!(request.contains("authorization: Bearer synthetic-secret-sentinel\r\n"));
        assert!(!request.contains("content-length:"));
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_accepts_empty_maximum_unknown_and_repeats_fresh_calls() {
        let maximum = (0..oneagent_llm::MAX_MODELS_PER_CATALOG)
            .map(|index| format!(r#"{{"id":"model-{index:04}"}}"#))
            .collect::<Vec<_>>()
            .join(",");
        let maximum = format!(r#"{{"object":"list","data":[{maximum}]}}"#);
        let responses = vec![
            Some(http_response(200, br#"{"object":"list","data":[]}"#)),
            Some(http_response(200, maximum.as_bytes())),
        ];
        let mut server = ControlledServer::spawn(responses).await;
        let provider = provider(server.base_url(), None);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let empty = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("empty catalog must succeed");
        let maximum = provider
            .discover_models(context(&policy, &cancellation))
            .await
            .expect("maximum catalog must succeed");
        assert!(empty.is_empty());
        assert_eq!(maximum.models().len(), oneagent_llm::MAX_MODELS_PER_CATALOG);
        assert!(!server.next_request().await.is_empty());
        assert!(!server.next_request().await.is_empty());
        server.finish().await;
    }

    #[tokio::test]
    async fn discovery_rejects_malformed_shapes_and_catalog_invariants_atomically() {
        let over_count = (0..=oneagent_llm::MAX_MODELS_PER_CATALOG)
            .map(|index| format!(r#"{{"id":"model-{index}"}}"#))
            .collect::<Vec<_>>()
            .join(",");
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
        ];

        for (body, expected) in cases {
            let server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
            let provider = provider(server.base_url(), None);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .discover_models(context(&policy, &cancellation))
                .await
                .expect_err("invalid discovery response must fail");
            assert_eq!(error.kind(), expected);
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn discovery_maps_statuses_rejects_redirects_and_bounds_bodies() {
        let cases = [
            (
                http_response(503, b"provider body"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(429, b"provider body"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(400, b"provider body"),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "302 Found",
                    &["Location: http://external.invalid/v1/models"],
                    b"provider body",
                ),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[&format!(
                        "Content-Length: {}",
                        MAX_MODELS_RESPONSE_BODY_BYTES + 1
                    )],
                    b"",
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[],
                    &vec![b'x'; MAX_MODELS_RESPONSE_BODY_BYTES + 1],
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response("200 OK", &["Content-Length: 100"], br#"{"object":"list""#),
                LlmErrorKind::Transport,
            ),
        ];

        for (response, expected) in cases {
            let server = ControlledServer::spawn(vec![Some(response)]).await;
            let provider = provider(server.base_url(), None);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .discover_models(context(&policy, &cancellation))
                .await
                .expect_err("terminal response must fail");
            assert_eq!(error.kind(), expected);
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

        let pre_cancelled = TestCancellation::default();
        pre_cancelled.cancel();
        let unreachable = provider("http://127.0.0.1:9", None);
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
            .expect_err("rejection must fail");
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
    fn provider_and_cancellation_are_send_sync() {
        assert_send_sync::<OpenAiCompatibleProvider>();
        assert_send_sync::<TestCancellation>();
    }
}
