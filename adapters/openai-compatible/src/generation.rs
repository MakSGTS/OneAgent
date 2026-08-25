use oneagent_llm::{
    FinishReason, LlmError, LlmErrorKind, TextGenerationRequest, TextGenerationResponse,
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};

use crate::{
    MAX_COMPLETION_REQUEST_BODY_BYTES, MAX_COMPLETION_RESPONSE_BODY_BYTES,
    config::{OpenAiCompatibleProvider, apply_authorization},
    execution::{adapter_error, bounded_success_body, status_error},
    wire::{CompletionRequest, CompletionResponse},
};

impl OpenAiCompatibleProvider {
    pub(crate) async fn execute_generation(
        &self,
        request: &TextGenerationRequest,
    ) -> Result<TextGenerationResponse, LlmError> {
        let wire = CompletionRequest {
            model: request.model().model().as_str(),
            prompt: request.input(),
            max_tokens: request.max_output_bytes(),
            stream: false,
        };
        let body = serde_json::to_vec(&wire).map_err(|_| {
            adapter_error(
                LlmErrorKind::Internal,
                "provider completion request serialization failed",
            )
        })?;
        if body.len() > MAX_COMPLETION_REQUEST_BODY_BYTES {
            return Err(adapter_error(
                LlmErrorKind::InvalidRequest,
                "provider completion request exceeds wire byte limit",
            ));
        }

        let http_request = self
            .client()
            .post(self.completions_url().clone())
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .body(body);
        let http_request = apply_authorization(http_request, self.authorization());
        let response = http_request.send().await.map_err(|_| {
            adapter_error(LlmErrorKind::Transport, "provider request transport failed")
        })?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }

        let body = bounded_success_body(response, MAX_COMPLETION_RESPONSE_BODY_BYTES).await?;
        let wire: CompletionResponse = serde_json::from_slice(&body).map_err(|_| {
            adapter_error(
                LlmErrorKind::Protocol,
                "provider completion response is not valid JSON",
            )
        })?;
        if wire.object != "text_completion" {
            return Err(adapter_error(
                LlmErrorKind::Protocol,
                "provider completion response object is invalid",
            ));
        }
        if wire.model != request.model().model().as_str() {
            return Err(adapter_error(
                LlmErrorKind::InvalidResponse,
                "provider completion response model does not match request",
            ));
        }
        let [choice] = wire.choices.as_slice() else {
            return Err(adapter_error(
                LlmErrorKind::InvalidResponse,
                "provider completion response choice count is invalid",
            ));
        };
        if choice.index != 0 {
            return Err(adapter_error(
                LlmErrorKind::InvalidResponse,
                "provider completion response choice index is invalid",
            ));
        }
        let finish = match &choice.finish_reason {
            serde_json::Value::String(value) if value == "stop" => FinishReason::Completed,
            serde_json::Value::String(value) if value == "length" => FinishReason::OutputLimit,
            serde_json::Value::String(_) | serde_json::Value::Null => {
                return Err(adapter_error(
                    LlmErrorKind::InvalidResponse,
                    "provider completion response finish reason is invalid",
                ));
            }
            _ => {
                return Err(adapter_error(
                    LlmErrorKind::Protocol,
                    "provider completion response finish reason has invalid type",
                ));
            }
        };

        TextGenerationResponse::new(request, choice.text.clone(), finish)
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

    use super::OpenAiCompatibleProvider;
    use crate::{
        MAX_COMPLETION_RESPONSE_BODY_BYTES,
        test_support::{ControlledServer, http_response, raw_http_response},
    };

    fn provider(base_url: &str, secret: Option<&str>) -> OpenAiCompatibleProvider {
        let configuration = ProviderConfiguration::new(
            ProviderId::new("openai-compatible").expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
        );
        OpenAiCompatibleProvider::new(configuration, base_url).expect("provider must construct")
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
    async fn generation_sends_exact_wire_and_maps_stop_with_local_byte_usage() {
        let output = "синтетический ответ";
        let mut server = ControlledServer::spawn(vec![Some(http_response(
            200,
            &success("model-a", output, "stop"),
        ))])
        .await;
        let provider = provider(server.base_url(), Some("synthetic-secret-sentinel"));
        let request = request("openai-compatible", "model-a", "точный вход", output.len());
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
        let header_end = captured
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("request headers must terminate")
            + 4;
        let headers = std::str::from_utf8(&captured[..header_end]).expect("headers must be UTF-8");
        assert!(headers.starts_with("POST /v1/completions HTTP/1.1\r\n"));
        assert!(headers.contains("accept: application/json\r\n"));
        assert!(headers.contains("content-type: application/json\r\n"));
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
        let request = request("openai-compatible", "model-a", &input, 1);
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
        assert_eq!(
            server.next_request().await.len(),
            server.next_request().await.len()
        );
        server.finish().await;
    }

    #[tokio::test]
    async fn generation_rejects_protocol_and_terminal_semantic_failures() {
        let protocol_cases = vec![
            b"not-json".to_vec(),
            br#"{"model":"model-a","choices":[]}"#.to_vec(),
            br#"{"object":"wrong","model":"model-a","choices":[]}"#.to_vec(),
            br#"{"object":"text_completion","model":"model-a"}"#.to_vec(),
            br#"{"object":"text_completion","model":"model-a","choices":[{"text":"x","index":0}]}"#.to_vec(),
            br#"{"object":"text_completion","model":"model-a","choices":[{"text":"x","index":"zero","finish_reason":"stop"}]}"#.to_vec(),
        ];
        let semantic_cases = vec![
            json!({"object":"text_completion","model":"fallback","choices":[{"text":"x","index":0,"finish_reason":"stop"}]}),
            json!({"object":"text_completion","model":"model-a","choices":[]}),
            json!({"object":"text_completion","model":"model-a","choices":[{"text":"x","index":0,"finish_reason":"stop"},{"text":"y","index":1,"finish_reason":"stop"}]}),
            json!({"object":"text_completion","model":"model-a","choices":[{"text":"x","index":1,"finish_reason":"stop"}]}),
            json!({"object":"text_completion","model":"model-a","choices":[{"text":"x","index":0,"finish_reason":"future"}]}),
            json!({"object":"text_completion","model":"model-a","choices":[{"text":"x","index":0,"finish_reason":null}]}),
            json!({"object":"text_completion","model":"model-a","choices":[{"text":"","index":0,"finish_reason":"stop"}]}),
            json!({"object":"text_completion","model":"model-a","choices":[{"text":"xx","index":0,"finish_reason":"stop"}]}),
        ];

        for body in protocol_cases {
            assert_generation_error(body, LlmErrorKind::Protocol).await;
        }
        for value in semantic_cases {
            assert_generation_error(
                serde_json::to_vec(&value).expect("fixture must serialize"),
                LlmErrorKind::InvalidResponse,
            )
            .await;
        }
    }

    async fn assert_generation_error(body: Vec<u8>, expected: LlmErrorKind) {
        let server = ControlledServer::spawn(vec![Some(http_response(200, &body))]).await;
        let provider = provider(server.base_url(), None);
        let request = request("openai-compatible", "model-a", "input", 1);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;
        let error = provider
            .generate(&request, context(&policy, &cancellation))
            .await
            .expect_err("invalid generation response must fail");
        assert_eq!(error.kind(), expected);
        server.finish().await;
    }

    #[tokio::test]
    async fn generation_maps_status_redirect_partial_and_response_bounds_without_retry() {
        let cases = [
            (
                http_response(503, b"body"),
                LlmErrorKind::ProviderUnavailable,
            ),
            (
                http_response(429, b"body"),
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
            let request = request("openai-compatible", "model-a", "input", 1);
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
        let error = unreachable
            .generate(&wrong, context(&policy, &never))
            .await
            .expect_err("provider mismatch must precede I/O");
        assert_eq!(error.kind(), LlmErrorKind::InvalidRequest);

        let pre_cancelled = TestCancellation::default();
        pre_cancelled.cancel();
        let valid = request("openai-compatible", "model-a", "input", 1);
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
            request = cancel_server.next_request() => assert!(!request.is_empty()),
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
        let sensitive = request("openai-compatible", "model-a", sentinel, 1);
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
