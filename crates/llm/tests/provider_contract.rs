use std::{
    future::{Future, poll_fn},
    pin::Pin,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
    time::Duration,
};

use oneagent_llm::{
    CancellationSignal, FinishReason, LlmError, LlmErrorKind, LlmProvider, MAX_MODEL_ID_BYTES,
    MAX_MODELS_PER_CATALOG, MAX_PROVIDER_DIAGNOSTIC_BYTES, MAX_PROVIDER_ID_BYTES,
    MAX_PROVIDER_SECRET_BYTES, MAX_PROVIDER_TIMEOUT, MAX_PROVIDER_TIMEOUT_SECS,
    MAX_TEXT_INPUT_BYTES, MAX_TEXT_OUTPUT_BYTES, ModelCapability, ModelCatalog, ModelDescriptor,
    ModelId, ModelIdentity, NeverCancelled, ProviderConfiguration, ProviderDiagnostic,
    ProviderExecutionContext, ProviderExecutionPolicy, ProviderFuture, ProviderId, ProviderSecret,
    RetryPolicy, TextGenerationRequest, TextGenerationResponse,
};

fn identity(provider: &ProviderId, model: &str) -> ModelIdentity {
    ModelIdentity::new(
        provider.clone(),
        ModelId::new(model).expect("model ID must pass"),
    )
}

fn descriptor(provider: &ProviderId, model: &str) -> ModelDescriptor {
    ModelDescriptor::new(identity(provider, model), [ModelCapability::TextGeneration])
}

#[test]
fn public_identity_capability_and_catalog_contract_is_bounded_and_canonical() {
    let provider = ProviderId::new("provider").expect("provider ID must pass");
    assert!(ProviderId::new("p".repeat(MAX_PROVIDER_ID_BYTES)).is_ok());
    assert!(ProviderId::new("p".repeat(MAX_PROVIDER_ID_BYTES + 1)).is_err());
    assert!(ProviderId::new("").is_err());
    assert!(ProviderId::new(" provider").is_err());
    assert!(ProviderId::new("provider\n").is_err());

    assert!(ModelId::new("m".repeat(MAX_MODEL_ID_BYTES)).is_ok());
    assert!(ModelId::new("m".repeat(MAX_MODEL_ID_BYTES + 1)).is_err());
    assert!(ModelId::new(" ").is_err());
    assert!(ModelId::new("model\0").is_err());

    let duplicate_capabilities = ModelDescriptor::new(
        identity(&provider, "capable"),
        [
            ModelCapability::TextGeneration,
            ModelCapability::TextGeneration,
        ],
    );
    assert_eq!(duplicate_capabilities.capabilities().len(), 1);
    assert!(duplicate_capabilities.supports(ModelCapability::TextGeneration));

    let catalog = ModelCatalog::new(
        provider.clone(),
        vec![descriptor(&provider, "z"), descriptor(&provider, "a")],
    )
    .expect("reordered catalog must pass");
    assert_eq!(catalog.models()[0].identity().model().as_str(), "a");
    assert_eq!(catalog.models()[1].identity().model().as_str(), "z");
    assert_eq!(
        catalog,
        ModelCatalog::new(
            provider.clone(),
            vec![descriptor(&provider, "z"), descriptor(&provider, "a")],
        )
        .expect("repeated catalog must pass")
    );
    assert!(
        ModelCatalog::new(provider.clone(), Vec::new())
            .expect("empty discovery must pass")
            .is_empty()
    );

    let duplicate = descriptor(&provider, "duplicate");
    assert_eq!(
        ModelCatalog::new(provider.clone(), vec![duplicate.clone(), duplicate])
            .expect_err("duplicate model must fail")
            .kind(),
        LlmErrorKind::InvalidModelCatalog
    );
    let other = ProviderId::new("other").expect("provider ID must pass");
    assert_eq!(
        ModelCatalog::new(provider.clone(), vec![descriptor(&other, "model")])
            .expect_err("provider scope mismatch must fail")
            .kind(),
        LlmErrorKind::InvalidModelCatalog
    );

    let maximum = (0..MAX_MODELS_PER_CATALOG)
        .map(|index| descriptor(&provider, &format!("model-{index}")))
        .collect();
    assert!(ModelCatalog::new(provider.clone(), maximum).is_ok());
    let over_limit = (0..=MAX_MODELS_PER_CATALOG)
        .map(|index| descriptor(&provider, &format!("model-{index}")))
        .collect();
    assert_eq!(
        ModelCatalog::new(provider, over_limit)
            .expect_err("over-limit catalog must fail")
            .kind(),
        LlmErrorKind::InvalidModelCatalog
    );
}

#[test]
fn public_request_response_contract_preserves_text_and_rejects_invalid_shapes() {
    let provider = ProviderId::new("provider").expect("provider ID must pass");
    let capable = descriptor(&provider, "model");
    let incompatible = ModelDescriptor::new(identity(&provider, "incompatible"), []);
    let sentinel = "  sensitive-request-Привет\r\n";
    let request = TextGenerationRequest::new(&capable, sentinel, MAX_TEXT_OUTPUT_BYTES)
        .expect("request must pass");
    assert_eq!(request.input(), sentinel);
    assert_eq!(request.model(), capable.identity());
    assert!(!format!("{request:?}").contains(sentinel));

    let exact_unicode = "я".repeat(MAX_TEXT_INPUT_BYTES / "я".len());
    assert_eq!(exact_unicode.len(), MAX_TEXT_INPUT_BYTES);
    assert!(TextGenerationRequest::new(&capable, exact_unicode, 1).is_ok());

    let precedence = [
        (" \n".to_owned(), 0, "request input is empty"),
        (
            "x".repeat(MAX_TEXT_INPUT_BYTES + 1),
            0,
            "request input exceeds byte limit",
        ),
        ("x".to_owned(), 0, "request output bound is zero"),
        (
            "x".to_owned(),
            MAX_TEXT_OUTPUT_BYTES + 1,
            "request output bound exceeds byte limit",
        ),
    ];
    for (input, bound, expected) in precedence {
        let error = TextGenerationRequest::new(&incompatible, input, bound)
            .expect_err("the earlier request failure must win");
        assert_eq!(error.kind(), LlmErrorKind::InvalidRequest);
        assert_eq!(
            error.diagnostic().map(ProviderDiagnostic::as_str),
            Some(expected)
        );
    }
    assert_eq!(
        TextGenerationRequest::new(&incompatible, "x", 1)
            .expect_err("incompatible model must fail")
            .kind(),
        LlmErrorKind::IncompatibleModel
    );

    let equivalent = ModelDescriptor::new(
        capable.identity().clone(),
        [
            ModelCapability::TextGeneration,
            ModelCapability::TextGeneration,
        ],
    );
    assert_eq!(
        TextGenerationRequest::new(&capable, "same", 16).expect("request must pass"),
        TextGenerationRequest::new(&equivalent, "same", 16).expect("request must pass")
    );

    let bounded = TextGenerationRequest::new(&capable, sentinel, "response".len())
        .expect("request must pass");
    let response = TextGenerationResponse::new(&bounded, "response", FinishReason::OutputLimit)
        .expect("bounded response must pass");
    assert_eq!(response.model(), bounded.model());
    assert_eq!(response.output(), "response");
    assert_eq!(response.usage().input_bytes(), sentinel.len());
    assert_eq!(response.usage().output_bytes(), "response".len());
    assert_eq!(
        response.usage().total_bytes(),
        sentinel.len() + "response".len()
    );
    assert_eq!(response.finish(), FinishReason::OutputLimit);
    assert!(!format!("{response:?}").contains("response"));
    assert_eq!(
        TextGenerationResponse::new(&bounded, "", FinishReason::Completed)
            .expect_err("empty response must fail")
            .kind(),
        LlmErrorKind::InvalidResponse
    );
    assert_eq!(
        TextGenerationResponse::new(&bounded, "response!", FinishReason::Completed)
            .expect_err("over-bound response must fail")
            .kind(),
        LlmErrorKind::InvalidResponse
    );
}

#[test]
fn public_secret_diagnostic_and_error_contract_is_bounded_and_redacted() {
    let sentinel = "synthetic-secret-sentinel";
    let secret = ProviderSecret::new(sentinel).expect("secret must pass");
    assert_eq!(secret.expose(), sentinel);
    assert_eq!(format!("{secret:?}"), "ProviderSecret([REDACTED])");
    assert!(ProviderSecret::new("s".repeat(MAX_PROVIDER_SECRET_BYTES)).is_ok());
    assert!(ProviderSecret::new("s".repeat(MAX_PROVIDER_SECRET_BYTES + 1)).is_err());
    assert!(ProviderSecret::new(" \n").is_err());

    let configuration = ProviderConfiguration::new(
        ProviderId::new("provider").expect("provider ID must pass"),
        Some(secret),
    );
    let configuration_debug = format!("{configuration:?}");
    assert!(configuration_debug.contains("has_credential: true"));
    assert!(!configuration_debug.contains(sentinel));

    assert!(ProviderDiagnostic::new("d".repeat(MAX_PROVIDER_DIAGNOSTIC_BYTES)).is_ok());
    assert!(ProviderDiagnostic::new("d".repeat(MAX_PROVIDER_DIAGNOSTIC_BYTES + 1)).is_err());
    assert!(ProviderDiagnostic::new("").is_err());

    let kinds = [
        LlmErrorKind::InvalidProviderId,
        LlmErrorKind::InvalidModelId,
        LlmErrorKind::InvalidModelCatalog,
        LlmErrorKind::InvalidConfiguration,
        LlmErrorKind::InvalidRequest,
        LlmErrorKind::IncompatibleModel,
        LlmErrorKind::InvalidResponse,
        LlmErrorKind::ProviderUnavailable,
        LlmErrorKind::ProviderRejected,
        LlmErrorKind::Transport,
        LlmErrorKind::Protocol,
        LlmErrorKind::Timeout,
        LlmErrorKind::Cancelled,
        LlmErrorKind::Internal,
    ];
    for kind in kinds {
        let diagnostic = ProviderDiagnostic::new(sentinel).expect("diagnostic must pass");
        let error = LlmError::new(kind).with_diagnostic(diagnostic);
        assert!(!format!("{error}").contains(sentinel));
        assert!(!format!("{error:?}").contains(sentinel));
        assert_eq!(
            kind.is_retryable(),
            matches!(
                kind,
                LlmErrorKind::ProviderUnavailable | LlmErrorKind::Transport | LlmErrorKind::Timeout
            )
        );
    }
}

#[test]
fn public_policy_contract_represents_timeout_and_never_retries() {
    let default = ProviderExecutionPolicy::default();
    assert_eq!(default.timeout(), None);
    assert_eq!(default.retry(), RetryPolicy::Never);
    assert_eq!(default.max_attempts(), 1);
    assert_eq!(
        ProviderExecutionPolicy::new(Some(MAX_PROVIDER_TIMEOUT))
            .expect("maximum timeout must pass")
            .timeout(),
        Some(MAX_PROVIDER_TIMEOUT)
    );
    assert_eq!(
        ProviderExecutionPolicy::new(Some(Duration::ZERO))
            .expect_err("zero timeout must fail")
            .kind(),
        LlmErrorKind::InvalidConfiguration
    );
    assert_eq!(
        ProviderExecutionPolicy::new(Some(Duration::from_secs(MAX_PROVIDER_TIMEOUT_SECS + 1,)))
            .expect_err("over-limit timeout must fail")
            .kind(),
        LlmErrorKind::InvalidConfiguration
    );
}

#[derive(Clone, Copy)]
enum FakeMode {
    Success,
    Failure(LlmErrorKind),
    WaitForCancellation,
}

struct ContractProvider {
    id: ProviderId,
    models: Vec<ModelDescriptor>,
    mode: FakeMode,
    discoveries: AtomicUsize,
    generations: AtomicUsize,
    active: AtomicUsize,
}

impl ContractProvider {
    fn new(provider: &str, models: &[&str], mode: FakeMode) -> Self {
        let id = ProviderId::new(provider).expect("provider ID must pass");
        Self {
            models: models.iter().map(|model| descriptor(&id, model)).collect(),
            id,
            mode,
            discoveries: AtomicUsize::new(0),
            generations: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
        }
    }

    fn request(&self) -> TextGenerationRequest {
        TextGenerationRequest::new(&self.models[0], "provider input", 64)
            .expect("request must pass")
    }
}

impl LlmProvider for ContractProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn discover_models<'a>(
        &'a self,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<ModelCatalog, LlmError>> {
        Box::pin(async move {
            if context.cancellation().is_cancelled() {
                return Err(LlmError::new(LlmErrorKind::Cancelled));
            }
            self.discoveries.fetch_add(1, Ordering::SeqCst);
            ModelCatalog::new(self.id.clone(), self.models.clone())
        })
    }

    fn generate<'a>(
        &'a self,
        request: &'a TextGenerationRequest,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<TextGenerationResponse, LlmError>> {
        Box::pin(async move {
            if request.model().provider() != &self.id {
                return Err(LlmError::new(LlmErrorKind::InvalidRequest));
            }
            if context.cancellation().is_cancelled() {
                return Err(LlmError::new(LlmErrorKind::Cancelled));
            }
            self.generations.fetch_add(1, Ordering::SeqCst);
            match self.mode {
                FakeMode::Success => {
                    TextGenerationResponse::new(request, "provider output", FinishReason::Completed)
                }
                FakeMode::Failure(kind) => Err(LlmError::new(kind)),
                FakeMode::WaitForCancellation => {
                    self.active.fetch_add(1, Ordering::SeqCst);
                    let _guard = ActiveGuard(&self.active);
                    context.cancellation().cancelled().await;
                    Err(LlmError::new(LlmErrorKind::Cancelled))
                }
            }
        })
    }
}

struct ActiveGuard<'a>(&'a AtomicUsize);

impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Default)]
struct TestCancellation {
    cancelled: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl TestCancellation {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Some(waker) = self.waker.lock().expect("waker lock must pass").take() {
            waker.wake();
        }
    }
}

impl CancellationSignal for TestCancellation {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    fn cancelled(&self) -> ProviderFuture<'_, ()> {
        Box::pin(poll_fn(move |context| {
            if self.is_cancelled() {
                Poll::Ready(())
            } else {
                self.waker
                    .lock()
                    .expect("waker lock must pass")
                    .replace(context.waker().clone());
                Poll::Pending
            }
        }))
    }
}

fn poll<T>(future: Pin<&mut (dyn Future<Output = T> + Send)>) -> Poll<T> {
    future.poll(&mut Context::from_waker(Waker::noop()))
}

#[test]
fn public_provider_seam_substitutes_independent_fakes_and_repeats_exactly() {
    let first = ContractProvider::new("first", &["z", "a"], FakeMode::Success);
    let second = ContractProvider::new("second", &["model"], FakeMode::Success);
    let empty = ContractProvider::new("empty", &[], FakeMode::Success);
    let policy = ProviderExecutionPolicy::default();
    let cancellation = NeverCancelled;
    let context = ProviderExecutionContext::new(&policy, &cancellation);

    for provider in [
        &first as &dyn LlmProvider,
        &second as &dyn LlmProvider,
        &empty as &dyn LlmProvider,
    ] {
        let mut discovery = provider.discover_models(context);
        let Poll::Ready(Ok(catalog)) = poll(discovery.as_mut()) else {
            panic!("discovery must complete successfully");
        };
        assert_eq!(catalog.provider(), provider.id());
    }
    let mut discovery = first.discover_models(context);
    let Poll::Ready(Ok(catalog)) = poll(discovery.as_mut()) else {
        panic!("discovery must complete successfully");
    };
    assert_eq!(catalog.models()[0].identity().model().as_str(), "a");
    assert_eq!(catalog.models()[1].identity().model().as_str(), "z");

    for provider in [&first, &second] {
        let request = provider.request();
        for _ in 0..2 {
            let mut generation = (provider as &dyn LlmProvider).generate(&request, context);
            let Poll::Ready(Ok(response)) = poll(generation.as_mut()) else {
                panic!("generation must complete successfully");
            };
            assert_eq!(response.model(), request.model());
            assert_eq!(response.output(), "provider output");
        }
        assert_eq!(provider.generations.load(Ordering::SeqCst), 2);
    }
}

#[test]
fn public_provider_failures_are_typed_single_attempts_and_redacted() {
    let policy = ProviderExecutionPolicy::default();
    let cancellation = NeverCancelled;
    let context = ProviderExecutionContext::new(&policy, &cancellation);

    for kind in [
        LlmErrorKind::ProviderUnavailable,
        LlmErrorKind::ProviderRejected,
        LlmErrorKind::Transport,
        LlmErrorKind::Protocol,
        LlmErrorKind::Timeout,
        LlmErrorKind::InvalidResponse,
        LlmErrorKind::Internal,
    ] {
        let provider = ContractProvider::new("provider", &["model"], FakeMode::Failure(kind));
        let request = provider.request();
        let mut generation = provider.generate(&request, context);
        let Poll::Ready(Err(error)) = poll(generation.as_mut()) else {
            panic!("fake failure must complete");
        };
        assert_eq!(error.kind(), kind);
        assert_eq!(provider.generations.load(Ordering::SeqCst), 1);
        assert!(!format!("{error:?}").contains(request.input()));
    }

    let provider = ContractProvider::new("provider", &["model"], FakeMode::Success);
    let other = ContractProvider::new("other", &["model"], FakeMode::Success);
    let request = other.request();
    let mut mismatch = provider.generate(&request, context);
    let Poll::Ready(Err(error)) = poll(mismatch.as_mut()) else {
        panic!("provider mismatch must fail before work");
    };
    assert_eq!(error.kind(), LlmErrorKind::InvalidRequest);
    assert_eq!(provider.generations.load(Ordering::SeqCst), 0);
}

#[test]
fn public_cancellation_precedence_and_cleanup_leave_no_active_work() {
    let provider = ContractProvider::new("provider", &["model"], FakeMode::WaitForCancellation);
    let request = provider.request();
    let policy = ProviderExecutionPolicy::default();
    let cancellation = TestCancellation::default();
    let context = ProviderExecutionContext::new(&policy, &cancellation);
    let mut generation = provider.generate(&request, context);

    assert!(poll(generation.as_mut()).is_pending());
    assert_eq!(provider.active.load(Ordering::SeqCst), 1);
    cancellation.cancel();
    let Poll::Ready(Err(error)) = poll(generation.as_mut()) else {
        panic!("in-flight cancellation must complete");
    };
    assert_eq!(error.kind(), LlmErrorKind::Cancelled);
    assert_eq!(provider.generations.load(Ordering::SeqCst), 1);
    assert_eq!(provider.active.load(Ordering::SeqCst), 0);

    let already_cancelled = ContractProvider::new(
        "already-cancelled",
        &["model"],
        FakeMode::WaitForCancellation,
    );
    let request = already_cancelled.request();
    let mut generation = already_cancelled.generate(&request, context);
    let Poll::Ready(Err(error)) = poll(generation.as_mut()) else {
        panic!("existing cancellation must complete immediately");
    };
    assert_eq!(error.kind(), LlmErrorKind::Cancelled);
    assert_eq!(already_cancelled.generations.load(Ordering::SeqCst), 0);
    assert_eq!(already_cancelled.active.load(Ordering::SeqCst), 0);
}
