use oneagent_llm::{
    FinishReason, LlmError, LlmErrorKind, LlmProvider, ModelCatalog, ModelId,
    ProviderExecutionContext, ProviderFuture, TextGenerationRequest, TextGenerationResponse,
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};

use crate::{
    MAX_GENERATE_REQUEST_BODY_BYTES, MAX_GENERATE_RESPONSE_BODY_BYTES,
    config::OllamaProvider,
    execution::{adapter_error, bounded_success_body, run_with_context, status_error},
    wire::{GenerateRequest, GenerateResponse, GenerationOptions, OptionalString},
};

impl LlmProvider for OllamaProvider {
    fn id(&self) -> &oneagent_llm::ProviderId {
        OllamaProvider::id(self)
    }

    fn discover_models<'a>(
        &'a self,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<ModelCatalog, LlmError>> {
        Box::pin(async move { OllamaProvider::discover_models(self, context).await })
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
            run_with_context(context, self.execute_generation(request)).await
        })
    }
}

impl OllamaProvider {
    async fn execute_generation(
        &self,
        request: &TextGenerationRequest,
    ) -> Result<TextGenerationResponse, LlmError> {
        let body = serde_json::to_vec(&GenerateRequest {
            model: request.model().model().as_str(),
            prompt: request.input(),
            stream: false,
            raw: true,
            think: false,
            options: GenerationOptions {
                num_predict: request.max_output_bytes(),
            },
        })
        .map_err(|_| {
            adapter_error(
                LlmErrorKind::Internal,
                "provider generation request serialization failed",
            )
        })?;
        if body.len() > MAX_GENERATE_REQUEST_BODY_BYTES {
            return Err(adapter_error(
                LlmErrorKind::InvalidRequest,
                "provider generation request exceeds byte limit",
            ));
        }

        let response = self
            .client()
            .post(self.generate_url().clone())
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

        let body = bounded_success_body(response, MAX_GENERATE_RESPONSE_BODY_BYTES).await?;
        let wire: GenerateResponse = serde_json::from_slice(&body).map_err(|_| {
            adapter_error(
                LlmErrorKind::Protocol,
                "provider generation response is not valid JSON",
            )
        })?;
        drop(body);

        let response_model = ModelId::new(wire.model).map_err(|_| {
            adapter_error(
                LlmErrorKind::InvalidResponse,
                "provider response model identifier is invalid",
            )
        })?;
        if &response_model != request.model().model() {
            return Err(adapter_error(
                LlmErrorKind::InvalidResponse,
                "provider response model does not match request",
            ));
        }
        if !wire.done {
            return Err(adapter_error(
                LlmErrorKind::InvalidResponse,
                "provider response is not terminal",
            ));
        }
        match wire.thinking {
            OptionalString::Missing => {}
            OptionalString::Present(thinking) if thinking.is_empty() => {}
            OptionalString::Present(_) => {
                return Err(adapter_error(
                    LlmErrorKind::InvalidResponse,
                    "provider response contains unsupported thinking",
                ));
            }
        }
        let finish = match wire.done_reason.as_str() {
            "stop" => FinishReason::Completed,
            "length" => FinishReason::OutputLimit,
            _ => {
                return Err(adapter_error(
                    LlmErrorKind::InvalidResponse,
                    "provider response finish reason is unsupported",
                ));
            }
        };

        TextGenerationResponse::new(request, wire.response, finish)
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
        ProviderId, TextGenerationRequest,
    };
    use serde_json::{Value, json};
    use tokio::sync::Notify;

    use super::OllamaProvider;
    use crate::{
        MAX_GENERATE_REQUEST_BODY_BYTES, MAX_GENERATE_RESPONSE_BODY_BYTES,
        test_support::{ControlledServer, http_response, raw_http_response},
    };

    fn provider(base_url: &str) -> OllamaProvider {
        let configuration = ProviderConfiguration::new(
            ProviderId::new("ollama").expect("provider ID must pass"),
            None,
        );
        OllamaProvider::new(configuration, base_url).expect("provider must construct")
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

    fn success(model: &str, output: &str, finish: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "model": model,
            "response": output,
            "done": true,
            "done_reason": finish,
            "thinking": "",
            "context": [1, 2],
            "prompt_eval_count": 999,
            "eval_count": 999,
            "future": true
        }))
        .expect("generation fixture must serialize")
    }

    fn captured_body(request: &[u8]) -> (&str, Value) {
        let header_end = request
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("request headers must terminate")
            + 4;
        let headers = std::str::from_utf8(&request[..header_end]).expect("headers must be UTF-8");
        let body = serde_json::from_slice(&request[header_end..]).expect("body must be JSON");
        (headers, body)
    }

    #[tokio::test]
    async fn generation_sends_exact_native_wire_and_preserves_identity_and_usage() {
        let output = "синтетический ответ";
        let mut server = ControlledServer::spawn(vec![Some(http_response(
            200,
            &success("model-a", output, "stop"),
        ))])
        .await;
        let provider = provider(server.base_url());
        let request = request("ollama", "model-a", "точный вход", output.len());
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;

        let response = (&provider as &dyn LlmProvider)
            .generate(&request, context(&policy, &cancellation))
            .await
            .expect("generation must succeed");
        assert_eq!(response.model(), request.model());
        assert_eq!(response.output(), output);
        assert_eq!(response.finish(), FinishReason::Completed);
        assert_eq!(response.usage().input_bytes(), request.input().len());
        assert_eq!(response.usage().output_bytes(), output.len());

        let captured = server.next_request().await;
        let (headers, body) = captured_body(&captured);
        assert!(headers.starts_with("POST /api/generate HTTP/1.1\r\n"));
        assert!(headers.contains("accept: application/json\r\n"));
        assert!(headers.contains("content-type: application/json\r\n"));
        assert!(headers.contains("user-agent: oneagent-ollama/0.1.0\r\n"));
        assert!(!headers.contains("authorization:"));
        assert_eq!(
            body,
            json!({
                "model": "model-a",
                "prompt": "точный вход",
                "stream": false,
                "raw": true,
                "think": false,
                "options": {"num_predict": output.len()}
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
        let provider = provider(server.base_url());
        let input = "\0".repeat(oneagent_llm::MAX_TEXT_INPUT_BYTES);
        let request = request("ollama", "model-a", &input, 1);
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
        assert_eq!(first.finish(), FinishReason::OutputLimit);
        assert_eq!(second.finish(), FinishReason::Completed);
        for _ in 0..2 {
            let captured = server.next_request().await;
            assert!(captured.len() < MAX_GENERATE_REQUEST_BODY_BYTES);
            let (_, body) = captured_body(&captured);
            assert_eq!(body["prompt"], input);
            assert_eq!(body["options"]["num_predict"], 1);
        }
        server.finish().await;
    }

    #[tokio::test]
    async fn generation_rejects_every_terminal_semantic_violation() {
        let cases = [
            (
                json!({"model": "", "response": "x", "done": true, "done_reason": "stop"}),
                1,
            ),
            (
                json!({"model": "other", "response": "x", "done": true, "done_reason": "stop"}),
                1,
            ),
            (
                json!({"model": "model-a", "response": "x", "done": false, "done_reason": "stop"}),
                1,
            ),
            (
                json!({"model": "model-a", "response": "x", "done": true, "done_reason": "future"}),
                1,
            ),
            (
                json!({
                    "model": "model-a", "response": "x", "done": true,
                    "done_reason": "stop", "thinking": "unsupported"
                }),
                1,
            ),
            (
                json!({"model": "model-a", "response": "", "done": true, "done_reason": "stop"}),
                1,
            ),
            (
                json!({"model": "model-a", "response": "xx", "done": true, "done_reason": "stop"}),
                1,
            ),
        ];

        for (body, maximum) in cases {
            let body = serde_json::to_vec(&body).expect("fixture must serialize");
            let server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
            let provider = provider(server.base_url());
            let request = request("ollama", "model-a", "input", maximum);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .generate(&request, context(&policy, &cancellation))
                .await
                .expect_err("terminal semantic violation must fail");
            assert_eq!(error.kind(), LlmErrorKind::InvalidResponse);
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn generation_rejects_malformed_missing_mistyped_and_trailing_wire() {
        let cases = [
            b"not-json".to_vec(),
            br#"{"model":"model-a","response":"x","done":true,"done_reason":"stop"} trailing"#
                .to_vec(),
            br"{}".to_vec(),
            br#"{"model":"model-a","response":"x","done":true}"#.to_vec(),
            br#"{"model":"model-a","response":"x","done":"true","done_reason":"stop"}"#
                .to_vec(),
            br#"{"model":"model-a","response":"x","done":true,"done_reason":"stop","thinking":null}"#
                .to_vec(),
        ];

        for body in cases {
            let server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
            let provider = provider(server.base_url());
            let request = request("ollama", "model-a", "input", 1);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .generate(&request, context(&policy, &cancellation))
                .await
                .expect_err("invalid wire must fail");
            assert_eq!(error.kind(), LlmErrorKind::Protocol);
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn generation_maps_status_redirect_partial_and_response_bounds_once() {
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
                http_response(500, b"opaque"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(404, b"opaque"),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "307 Temporary Redirect",
                    &["Location: http://127.0.0.1:9/api/generate"],
                    b"opaque",
                ),
                LlmErrorKind::ProviderRejected,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[&format!(
                        "Content-Length: {}",
                        MAX_GENERATE_RESPONSE_BODY_BYTES + 1
                    )],
                    b"",
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response(
                    "200 OK",
                    &[],
                    &vec![b'x'; MAX_GENERATE_RESPONSE_BODY_BYTES + 1],
                ),
                LlmErrorKind::Protocol,
            ),
            (
                raw_http_response("200 OK", &["Content-Length: 100"], br#"{"model":"model-a""#),
                LlmErrorKind::Transport,
            ),
        ];

        for (response, expected) in cases {
            let server = ControlledServer::spawn(vec![Some(response)]).await;
            let provider = provider(server.base_url());
            let request = request("ollama", "model-a", "input", 1);
            let policy = ProviderExecutionPolicy::default();
            let cancellation = NeverCancelled;
            let error = provider
                .generate(&request, context(&policy, &cancellation))
                .await
                .expect_err("terminal response must fail");
            assert_eq!(error.kind(), expected);
            server.finish().await;
        }
    }

    #[tokio::test]
    async fn generation_enforces_identity_timeout_cancellation_redaction_and_cleanup() {
        let policy = ProviderExecutionPolicy::default();
        let never = NeverCancelled;
        let unreachable = provider("http://127.0.0.1:9");
        let mismatched = request("other", "model-a", "input", 1);
        let pre_cancelled = TestCancellation::default();
        pre_cancelled.cancel();

        let error = unreachable
            .generate(&mismatched, context(&policy, &pre_cancelled))
            .await
            .expect_err("provider mismatch must precede cancellation");
        assert_eq!(error.kind(), LlmErrorKind::InvalidRequest);
        let matched = request("ollama", "model-a", "input", 1);
        let error = unreachable
            .generate(&matched, context(&policy, &pre_cancelled))
            .await
            .expect_err("existing cancellation must precede transport");
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);
        let error = unreachable
            .generate(&matched, context(&policy, &never))
            .await
            .expect_err("closed loopback port must be transport failure");
        assert_eq!(error.kind(), LlmErrorKind::Transport);

        let mut timeout_server = ControlledServer::spawn(vec![None]).await;
        let timeout_provider = provider(timeout_server.base_url());
        let timeout_policy = ProviderExecutionPolicy::new(Some(Duration::from_millis(20)))
            .expect("timeout policy must pass");
        let error = timeout_provider
            .generate(&matched, context(&timeout_policy, &never))
            .await
            .expect_err("pending generation must time out");
        assert_eq!(error.kind(), LlmErrorKind::Timeout);
        assert!(!timeout_server.next_request().await.is_empty());
        timeout_server.finish().await;

        let mut cancel_server = ControlledServer::spawn(vec![None]).await;
        let cancel_provider = provider(cancel_server.base_url());
        let signal = TestCancellation::default();
        let operation = cancel_provider.generate(&matched, context(&policy, &signal));
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

        let sentinel = "synthetic-provider-body-sentinel";
        let server =
            ControlledServer::spawn(vec![Some(http_response(400, sentinel.as_bytes()))]).await;
        let provider = provider(server.base_url());
        let error = provider
            .generate(&matched, context(&policy, &never))
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
