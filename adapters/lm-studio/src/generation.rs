use oneagent_llm::{
    LlmError, LlmErrorKind, LlmProvider, ModelCapability, ModelCatalog, ModelDescriptor,
    ModelIdentity, ProviderExecutionContext, ProviderFuture, TextGenerationRequest,
    TextGenerationResponse,
};

use crate::{OPENAI_COMPATIBLE_PROVIDER_ID, config::LmStudioProvider, execution::adapter_error};

impl LlmProvider for LmStudioProvider {
    fn id(&self) -> &oneagent_llm::ProviderId {
        self.id()
    }

    fn discover_models<'a>(
        &'a self,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<ModelCatalog, LlmError>> {
        Box::pin(async move { LmStudioProvider::discover_models(self, context).await })
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

            let generation_provider_id =
                oneagent_llm::ProviderId::new(OPENAI_COMPATIBLE_PROVIDER_ID).map_err(|_| {
                    adapter_error(
                        LlmErrorKind::Internal,
                        "internal provider identifier construction failed",
                    )
                })?;
            let generation_model = ModelDescriptor::new(
                ModelIdentity::new(generation_provider_id, request.model().model().clone()),
                [ModelCapability::TextGeneration],
            );
            let generation_request = TextGenerationRequest::new(
                &generation_model,
                request.input(),
                request.max_output_bytes(),
            )
            .map_err(|_| {
                adapter_error(
                    LlmErrorKind::Internal,
                    "provider request translation failed",
                )
            })?;

            let response = self
                .generation_provider()
                .generate(&generation_request, context)
                .await?;
            TextGenerationResponse::new(request, response.output(), response.finish())
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };

    use oneagent_llm::{
        CancellationSignal, FinishReason, LlmErrorKind, LlmProvider, ModelCapability,
        ModelDescriptor, ModelId, ModelIdentity, NeverCancelled, ProviderConfiguration,
        ProviderDiagnostic, ProviderExecutionContext, ProviderExecutionPolicy, ProviderFuture,
        ProviderId, ProviderSecret, TextGenerationRequest,
    };
    use serde_json::{Value, json};
    use tokio::sync::Notify;

    use super::LmStudioProvider;
    use crate::test_support::{ControlledServer, http_response, raw_http_response};

    const MAX_COMPLETION_RESPONSE_BODY_BYTES: usize = 512 * 1_024;

    fn provider(base_url: &str, secret: Option<&str>) -> LmStudioProvider {
        let configuration = ProviderConfiguration::new(
            ProviderId::new("lm-studio").expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
        );
        LmStudioProvider::new(configuration, base_url).expect("provider must construct")
    }

    fn request(provider: &str, model: &str, input: &str, maximum: usize) -> TextGenerationRequest {
        let descriptor = ModelDescriptor::new(
            ModelIdentity::new(
                ProviderId::new(provider).expect("provider ID must pass"),
                ModelId::new(model).expect("model ID must pass"),
            ),
            [ModelCapability::TextGeneration],
        );
        TextGenerationRequest::new(&descriptor, input, maximum).expect("request must construct")
    }

    fn context<'a>(
        policy: &'a ProviderExecutionPolicy,
        cancellation: &'a dyn CancellationSignal,
    ) -> ProviderExecutionContext<'a> {
        ProviderExecutionContext::new(policy, cancellation)
    }

    fn success(model: &str, text: &str, finish: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "object": "text_completion",
            "model": model,
            "choices": [{"text": text, "index": 0, "finish_reason": finish}],
            "usage": {"prompt_tokens": 999, "completion_tokens": 999},
            "future": true
        }))
        .expect("response fixture must serialize")
    }

    #[tokio::test]
    async fn generation_sends_exact_bridge_wire_and_rebinds_lm_studio_identity() {
        let output = "синтетический ответ";
        let mut server = ControlledServer::spawn(vec![Some(http_response(
            200,
            &success("model-a", output, "stop"),
        ))])
        .await;
        let provider = provider(server.base_url(), Some("synthetic-secret-sentinel"));
        let request = request("lm-studio", "model-a", "точный вход", output.len());
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let response = (&provider as &dyn LlmProvider)
            .generate(&request, context(&policy, &cancellation))
            .await
            .expect("generation must succeed");
        assert_eq!(response.model(), request.model());
        assert_eq!(response.model().provider().as_str(), "lm-studio");
        assert_eq!(response.output(), output);
        assert_eq!(response.finish(), FinishReason::Completed);
        assert_eq!(response.usage().input_bytes(), request.input().len());
        assert_eq!(response.usage().output_bytes(), output.len());

        let captured = server.next_request().await;
        let header_end = captured
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("request headers must terminate")
            + 4;
        let headers = std::str::from_utf8(&captured[..header_end]).expect("headers must be UTF-8");
        assert!(headers.starts_with("POST /v1/completions HTTP/1.1\r\n"));
        assert!(headers.contains("accept: application/json\r\n"));
        assert!(headers.contains("content-type: application/json\r\n"));
        assert!(headers.contains("user-agent: oneagent-openai-compatible/0.1.0\r\n"));
        assert!(headers.contains("authorization: Bearer synthetic-secret-sentinel\r\n"));
        let body: Value =
            serde_json::from_slice(&captured[header_end..]).expect("body must be JSON");
        assert_eq!(
            body,
            json!({
                "model": "model-a",
                "prompt": "точный вход",
                "max_tokens": output.len(),
                "stream": false
            })
        );
        server.finish().await;
    }

    #[tokio::test]
    async fn generation_maps_length_accepts_maximum_escaping_and_repeats_fresh_calls() {
        let responses = vec![
            Some(http_response(200, &success("model-a", "x", "length"))),
            Some(http_response(200, &success("model-a", "y", "stop"))),
        ];
        let mut server = ControlledServer::spawn(responses).await;
        let provider = provider(server.base_url(), None);
        let input = "\0".repeat(oneagent_llm::MAX_TEXT_INPUT_BYTES);
        let request = request("lm-studio", "model-a", &input, 1);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let first = provider
            .generate(&request, context(&policy, &cancellation))
            .await
            .expect("length response must succeed");
        let second = provider
            .generate(&request, context(&policy, &cancellation))
            .await
            .expect("repeated response must succeed");
        assert_eq!(first.model(), request.model());
        assert_eq!(first.finish(), FinishReason::OutputLimit);
        assert_eq!(second.model(), request.model());
        assert_eq!(second.finish(), FinishReason::Completed);
        assert_eq!(
            server.next_request().await.len(),
            server.next_request().await.len()
        );
        server.finish().await;
    }

    #[tokio::test]
    async fn generation_preserves_composed_protocol_and_terminal_validation() {
        let cases = [
            (b"not-json".to_vec(), LlmErrorKind::Protocol),
            (
                serde_json::to_vec(&json!({
                    "object": "wrong",
                    "model": "model-a",
                    "choices": []
                }))
                .expect("fixture must serialize"),
                LlmErrorKind::Protocol,
            ),
            (
                serde_json::to_vec(&json!({
                    "object": "text_completion",
                    "model": "fallback",
                    "choices": [{"text": "x", "index": 0, "finish_reason": "stop"}]
                }))
                .expect("fixture must serialize"),
                LlmErrorKind::InvalidResponse,
            ),
            (
                serde_json::to_vec(&json!({
                    "object": "text_completion",
                    "model": "model-a",
                    "choices": [{"text": "", "index": 0, "finish_reason": "stop"}]
                }))
                .expect("fixture must serialize"),
                LlmErrorKind::InvalidResponse,
            ),
            (
                serde_json::to_vec(&json!({
                    "object": "text_completion",
                    "model": "model-a",
                    "choices": [{"text": "xx", "index": 0, "finish_reason": "stop"}]
                }))
                .expect("fixture must serialize"),
                LlmErrorKind::InvalidResponse,
            ),
        ];

        for (body, expected) in cases {
            let server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
            let provider = provider(server.base_url(), None);
            let request = request("lm-studio", "model-a", "input", 1);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .generate(&request, context(&policy, &cancellation))
                .await
                .expect_err("invalid generation response must fail");
            assert_eq!(error.kind(), expected);
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn generation_preserves_status_redirect_partial_and_response_bounds_without_retry() {
        let cases = [
            (
                http_response(503, b"body"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (http_response(400, b"body"), LlmErrorKind::ProviderRejected),
            (
                raw_http_response(
                    "302 Found",
                    &["Location: http://external.invalid/v1/completions"],
                    b"body",
                ),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[&format!(
                        "Content-Length: {}",
                        MAX_COMPLETION_RESPONSE_BODY_BYTES + 1
                    )],
                    b"",
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[],
                    &vec![b'x'; MAX_COMPLETION_RESPONSE_BODY_BYTES + 1],
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response("200 OK", &["Content-Length: 100"], b"{\"object\":"),
                LlmErrorKind::Transport,
            ),
        ];

        for (response, expected) in cases {
            let mut server = ControlledServer::spawn(vec![Some(response)]).await;
            let provider = provider(server.base_url(), None);
            let request = request("lm-studio", "model-a", "input", 1);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .generate(&request, context(&policy, &cancellation))
                .await
                .expect_err("terminal response must fail");
            assert_eq!(error.kind(), expected);
            assert!(!server.next_request().await.is_empty());
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn generation_enforces_identity_timeout_cancellation_redaction_and_cleanup() {
        let policy = ProviderExecutionPolicy::default();
        let never = NeverCancelled;
        let unreachable = provider("http://127.0.0.1:9", None);
        let wrong = request("other", "model-a", "input", 1);
        let pre_cancelled = TestCancellation::default();
        pre_cancelled.cancel();
        let error = unreachable
            .generate(&wrong, context(&policy, &pre_cancelled))
            .await
            .expect_err("provider mismatch must precede cancellation and I/O");
        assert_eq!(error.kind(), LlmErrorKind::InvalidRequest);

        let valid = request("lm-studio", "model-a", "input", 1);
        let error = unreachable
            .generate(&valid, context(&policy, &pre_cancelled))
            .await
            .expect_err("existing cancellation must precede I/O");
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);

        let mut timeout_server = ControlledServer::spawn(vec![None]).await;
        let timeout_provider = provider(timeout_server.base_url(), None);
        let timeout_policy = ProviderExecutionPolicy::new(Some(Duration::from_millis(20)))
            .expect("timeout policy must pass");
        let error = timeout_provider
            .generate(&valid, context(&timeout_policy, &never))
            .await
            .expect_err("pending response must time out");
        assert_eq!(error.kind(), LlmErrorKind::Timeout);
        assert!(!timeout_server.next_request().await.is_empty());
        timeout_server.finish().await;

        let mut cancel_server = ControlledServer::spawn(vec![None]).await;
        let cancel_provider = provider(cancel_server.base_url(), None);
        let signal = TestCancellation::default();
        let operation = cancel_provider.generate(&valid, context(&policy, &signal));
        tokio::pin!(operation);
        tokio::select! {
            captured = cancel_server.next_request() => assert!(!captured.is_empty()),
            result = &mut operation => panic!("operation completed before cancellation: {:?}", result.err().map(|error| error.kind())),
        }
        signal.cancel();
        let error = operation
            .await
            .expect_err("in-flight cancellation must win");
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);
        cancel_server.finish().await;

        let sentinel = "synthetic-secret-prompt-response-sentinel";
        let server =
            ControlledServer::spawn(vec![Some(http_response(400, sentinel.as_bytes()))]).await;
        let provider = provider(server.base_url(), Some(sentinel));
        let sensitive = request("lm-studio", "model-a", sentinel, 1);
        let error = provider
            .generate(&sensitive, context(&policy, &never))
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
}
