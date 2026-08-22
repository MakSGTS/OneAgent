//! Provider-neutral asynchronous discovery and execution seam.

use std::{future::Future, pin::Pin};

use crate::{
    LlmError, ModelCatalog, ProviderExecutionPolicy, ProviderId, TextGenerationRequest,
    TextGenerationResponse,
};

/// Owned provider future using only standard-library async vocabulary.
pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Receiver-only cooperative cancellation observed by provider implementations.
pub trait CancellationSignal: Send + Sync {
    /// Returns whether cancellation has already been requested.
    fn is_cancelled(&self) -> bool;

    /// Completes when cancellation is requested.
    fn cancelled(&self) -> ProviderFuture<'_, ()>;
}

/// Stateless cancellation signal that never requests cancellation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NeverCancelled;

impl CancellationSignal for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }

    fn cancelled(&self) -> ProviderFuture<'_, ()> {
        Box::pin(std::future::pending())
    }
}

/// Borrowed policy and cancellation inputs for one provider operation.
#[derive(Clone, Copy)]
pub struct ProviderExecutionContext<'a> {
    policy: &'a ProviderExecutionPolicy,
    cancellation: &'a dyn CancellationSignal,
}

impl<'a> ProviderExecutionContext<'a> {
    /// Creates a provider execution context without owning execution state.
    #[must_use]
    pub const fn new(
        policy: &'a ProviderExecutionPolicy,
        cancellation: &'a dyn CancellationSignal,
    ) -> Self {
        Self {
            policy,
            cancellation,
        }
    }

    /// Returns the represented timeout and fixed no-retry policy.
    #[must_use]
    pub const fn policy(self) -> &'a ProviderExecutionPolicy {
        self.policy
    }

    /// Returns the receiver-only cooperative cancellation signal.
    #[must_use]
    pub const fn cancellation(self) -> &'a dyn CancellationSignal {
        self.cancellation
    }
}

/// Substitutable provider-neutral model discovery and text generation seam.
///
/// Implementations must reject provider-identity mismatches and already
/// requested cancellation before provider work, invoke a provider at most once,
/// observe in-flight cancellation cooperatively, and validate owned terminal
/// results before returning success. This trait owns no executor or orchestration
/// wrapper that could enforce those implementation obligations.
pub trait LlmProvider: Send + Sync {
    /// Returns the provider identity used by every catalog and request.
    fn id(&self) -> &ProviderId;

    /// Discovers one canonical owned model catalog.
    fn discover_models<'a>(
        &'a self,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<ModelCatalog, LlmError>>;

    /// Generates one complete terminal response for a validated request.
    fn generate<'a>(
        &'a self,
        request: &'a TextGenerationRequest,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<TextGenerationResponse, LlmError>>;
}

#[cfg(test)]
mod tests {
    use std::{
        future::{Future, poll_fn},
        pin::Pin,
        sync::{
            Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        task::{Context, Poll, Waker},
    };

    use super::{
        CancellationSignal, LlmProvider, NeverCancelled, ProviderExecutionContext, ProviderFuture,
    };
    use crate::{
        FinishReason, LlmError, LlmErrorKind, ModelCapability, ModelCatalog, ModelDescriptor,
        ModelId, ModelIdentity, ProviderExecutionPolicy, ProviderId, TextGenerationRequest,
        TextGenerationResponse,
    };

    #[derive(Clone, Copy)]
    enum GenerationMode {
        Success,
        Failure(LlmErrorKind),
        WaitForCancellation,
    }

    struct FakeProvider {
        id: ProviderId,
        discovered: Vec<ModelDescriptor>,
        mode: GenerationMode,
        discovery_calls: AtomicUsize,
        generation_calls: AtomicUsize,
        active_operations: AtomicUsize,
    }

    impl FakeProvider {
        fn new(provider: &str, models: &[&str], mode: GenerationMode) -> Self {
            let id = ProviderId::new(provider).expect("provider ID must pass");
            let discovered = models
                .iter()
                .map(|model| {
                    ModelDescriptor::new(
                        ModelIdentity::new(
                            id.clone(),
                            ModelId::new(*model).expect("model ID must pass"),
                        ),
                        [ModelCapability::TextGeneration],
                    )
                })
                .collect();
            Self {
                id,
                discovered,
                mode,
                discovery_calls: AtomicUsize::new(0),
                generation_calls: AtomicUsize::new(0),
                active_operations: AtomicUsize::new(0),
            }
        }

        fn request(&self, model_index: usize) -> TextGenerationRequest {
            TextGenerationRequest::new(&self.discovered[model_index], "sensitive input", 64)
                .expect("fake request must pass")
        }
    }

    impl LlmProvider for FakeProvider {
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
                self.discovery_calls.fetch_add(1, Ordering::SeqCst);
                ModelCatalog::new(self.id.clone(), self.discovered.clone())
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

                self.generation_calls.fetch_add(1, Ordering::SeqCst);
                match self.mode {
                    GenerationMode::Success => {
                        TextGenerationResponse::new(request, "fake output", FinishReason::Completed)
                    }
                    GenerationMode::Failure(kind) => Err(LlmError::new(kind)),
                    GenerationMode::WaitForCancellation => {
                        self.active_operations.fetch_add(1, Ordering::SeqCst);
                        let _guard = ActiveOperationGuard(&self.active_operations);
                        context.cancellation().cancelled().await;
                        Err(LlmError::new(LlmErrorKind::Cancelled))
                    }
                }
            })
        }
    }

    struct ActiveOperationGuard<'a>(&'a AtomicUsize);

    impl Drop for ActiveOperationGuard<'_> {
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
    fn independent_providers_are_substitutable_and_discovery_is_canonical() {
        let first = FakeProvider::new("first", &["z", "a"], GenerationMode::Success);
        let second = FakeProvider::new("second", &["model"], GenerationMode::Success);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;
        let context = ProviderExecutionContext::new(&policy, &cancellation);

        for provider in [&first as &dyn LlmProvider, &second as &dyn LlmProvider] {
            let mut discovery = provider.discover_models(context);
            let Poll::Ready(Ok(catalog)) = poll(discovery.as_mut()) else {
                panic!("fake discovery must complete successfully");
            };
            assert_eq!(catalog.provider(), provider.id());
        }
        let mut discovery = first.discover_models(context);
        let Poll::Ready(Ok(catalog)) = poll(discovery.as_mut()) else {
            panic!("fake discovery must complete successfully");
        };
        assert_eq!(catalog.models()[0].identity().model().as_str(), "a");
        assert_eq!(catalog.models()[1].identity().model().as_str(), "z");

        for provider in [&first, &second] {
            let request = provider.request(0);
            let mut generation = (provider as &dyn LlmProvider).generate(&request, context);
            let Poll::Ready(Ok(response)) = poll(generation.as_mut()) else {
                panic!("fake generation must complete successfully");
            };
            assert_eq!(response.model(), request.model());
            assert_eq!(response.output(), "fake output");
        }
    }

    #[test]
    fn provider_mismatch_and_existing_cancellation_precede_work() {
        let provider = FakeProvider::new("provider", &["model"], GenerationMode::Success);
        let other = FakeProvider::new("other", &["model"], GenerationMode::Success);
        let request = other.request(0);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = TestCancellation::default();
        let context = ProviderExecutionContext::new(&policy, &cancellation);

        let mut mismatched = provider.generate(&request, context);
        let Poll::Ready(Err(error)) = poll(mismatched.as_mut()) else {
            panic!("provider mismatch must fail immediately");
        };
        assert_eq!(error.kind(), LlmErrorKind::InvalidRequest);
        assert_eq!(provider.generation_calls.load(Ordering::SeqCst), 0);

        cancellation.cancel();
        let mut discovery = provider.discover_models(context);
        let Poll::Ready(Err(error)) = poll(discovery.as_mut()) else {
            panic!("existing cancellation must fail immediately");
        };
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);
        assert_eq!(provider.discovery_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn retryable_failure_is_returned_after_exactly_one_attempt() {
        let provider = FakeProvider::new(
            "provider",
            &["model"],
            GenerationMode::Failure(LlmErrorKind::Transport),
        );
        let request = provider.request(0);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = NeverCancelled;
        let context = ProviderExecutionContext::new(&policy, &cancellation);

        let mut generation = provider.generate(&request, context);
        let Poll::Ready(Err(error)) = poll(generation.as_mut()) else {
            panic!("fake failure must complete immediately");
        };
        assert_eq!(error.kind(), LlmErrorKind::Transport);
        assert!(error.kind().is_retryable());
        assert_eq!(provider.generation_calls.load(Ordering::SeqCst), 1);
        assert_eq!(context.policy().max_attempts(), 1);
    }

    #[test]
    fn in_flight_cancellation_wins_and_releases_operation_state() {
        let provider =
            FakeProvider::new("provider", &["model"], GenerationMode::WaitForCancellation);
        let request = provider.request(0);
        let policy = ProviderExecutionPolicy::default();
        let cancellation = TestCancellation::default();
        let context = ProviderExecutionContext::new(&policy, &cancellation);
        let mut generation = provider.generate(&request, context);

        assert!(poll(generation.as_mut()).is_pending());
        assert_eq!(provider.active_operations.load(Ordering::SeqCst), 1);
        cancellation.cancel();

        let Poll::Ready(Err(error)) = poll(generation.as_mut()) else {
            panic!("cancellation must complete pending generation");
        };
        assert_eq!(error.kind(), LlmErrorKind::Cancelled);
        assert_eq!(provider.generation_calls.load(Ordering::SeqCst), 1);
        assert_eq!(provider.active_operations.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn never_cancelled_is_false_and_remains_pending() {
        let signal = NeverCancelled;
        assert!(!signal.is_cancelled());
        let mut cancelled = signal.cancelled();
        assert!(poll(cancelled.as_mut()).is_pending());
    }
}
