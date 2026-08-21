//! Ordered Runtime service registration and execution.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use tokio::task::{Id, JoinError, JoinSet};

use crate::{AppState, BoxError, CleanupFailure, RuntimeError};

use super::cancellation::CancellationSource;
use super::{RuntimeService, ServiceContext, ServiceTask};

struct ServiceRegistration {
    name: String,
    service: Box<dyn RuntimeService>,
}

/// Collects ordered Runtime service registrations before execution.
#[derive(Default)]
pub struct ServiceContainerBuilder {
    registrations: Vec<ServiceRegistration>,
}

impl std::fmt::Debug for ServiceContainerBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServiceContainerBuilder")
            .field(
                "services",
                &self
                    .registrations
                    .iter()
                    .map(|registration| registration.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl ServiceContainerBuilder {
    /// Creates an empty service-container builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registrations: Vec::new(),
        }
    }

    /// Registers one uniquely named service in startup order.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::InvalidServiceName`] for an empty name and
    /// [`RuntimeError::DuplicateServiceName`] for a name already registered in
    /// this builder.
    pub fn register<S>(self, name: impl Into<String>, service: S) -> Result<Self, RuntimeError>
    where
        S: RuntimeService,
    {
        self.register_boxed(name, Box::new(service))
    }

    /// Registers one boxed service in startup order.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::InvalidServiceName`] for an empty name and
    /// [`RuntimeError::DuplicateServiceName`] for a name already registered in
    /// this builder.
    pub fn register_boxed(
        mut self,
        name: impl Into<String>,
        service: Box<dyn RuntimeService>,
    ) -> Result<Self, RuntimeError> {
        let name = name.into();
        if name.is_empty() {
            return Err(RuntimeError::InvalidServiceName);
        }
        if self
            .registrations
            .iter()
            .any(|registration| registration.name == name)
        {
            return Err(RuntimeError::DuplicateServiceName { service: name });
        }

        self.registrations
            .push(ServiceRegistration { name, service });
        Ok(self)
    }

    /// Builds a service container around immutable shared application state.
    #[must_use]
    pub fn build(self, state: Arc<AppState>) -> ServiceContainer {
        ServiceContainer {
            state,
            registrations: self.registrations,
        }
    }
}

/// A built, not-yet-started ordered Runtime service container.
pub struct ServiceContainer {
    state: Arc<AppState>,
    registrations: Vec<ServiceRegistration>,
}

impl std::fmt::Debug for ServiceContainer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServiceContainer")
            .field("state", &self.state)
            .field(
                "services",
                &self
                    .registrations
                    .iter()
                    .map(|registration| registration.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl ServiceContainer {
    /// Starts services sequentially in registration order.
    ///
    /// # Errors
    ///
    /// Returns a named startup or task failure after all acknowledged services
    /// have been cancelled and joined.
    pub async fn start(self) -> Result<RunningServices, RuntimeError> {
        self.start_with_stopping(|| Ok(())).await
    }

    pub(crate) async fn start_with_stopping<C>(
        self,
        on_stopping: C,
    ) -> Result<RunningServices, RuntimeError>
    where
        C: FnOnce() -> Result<(), RuntimeError>,
    {
        let mut running = RunningServices::new();
        let mut on_stopping = Some(on_stopping);

        for registration in self.registrations {
            let ServiceRegistration { name, service } = registration;
            let (source, cancellation) = CancellationSource::new();
            let context = ServiceContext::new(Arc::clone(&self.state), cancellation);
            let mut start_handle = tokio::spawn(async move { service.start(context).await });

            let start_result = if running.has_active_tasks() {
                tokio::select! {
                    biased;
                    joined = running.join_next() => {
                        let mut primary = running.classify_before_cancellation(joined);
                        primary.error = apply_stopping_transition(
                            primary.error,
                            on_stopping
                                .take()
                                .expect("stopping transition callback must be available"),
                        );

                        start_handle.abort();
                        let start_cleanup = match start_handle.await {
                            Ok(Ok(task)) => {
                                running.spawn(name, source, task);
                                None
                            }
                            Ok(Err(error)) => Some(CleanupFailure::from_error(
                                &RuntimeError::ServiceStartFailed {
                                    service: name,
                                    source: error,
                                    cleanup: Vec::new(),
                                },
                            )),
                            Err(error) if error.is_cancelled() => None,
                            Err(error) => Some(CleanupFailure::from_error(
                                &RuntimeError::ServiceStartFailed {
                                    service: name,
                                    source: Box::new(error),
                                    cleanup: Vec::new(),
                                },
                            )),
                        };

                        let mut candidates = running.cleanup().await;
                        let mut error = select_task_primary(primary, &mut candidates);
                        if let Some(failure) = start_cleanup {
                            error = error.with_cleanup(vec![failure]);
                        }
                        return Err(error);
                    }
                    result = &mut start_handle => result,
                }
            } else {
                start_handle.await
            };

            match start_result {
                Ok(Ok(task)) => running.spawn(name, source, task),
                Ok(Err(error)) => {
                    let primary = apply_stopping_transition(
                        RuntimeError::ServiceStartFailed {
                            service: name,
                            source: error,
                            cleanup: Vec::new(),
                        },
                        on_stopping
                            .take()
                            .expect("stopping transition callback must be available"),
                    );
                    return Err(running.finish_fixed_primary(primary).await);
                }
                Err(error) => {
                    let primary = apply_stopping_transition(
                        RuntimeError::ServiceStartFailed {
                            service: name,
                            source: Box::new(error),
                            cleanup: Vec::new(),
                        },
                        on_stopping
                            .take()
                            .expect("stopping transition callback must be available"),
                    );
                    return Err(running.finish_fixed_primary(primary).await);
                }
            }
        }

        if let Some(joined) = running.try_join_next() {
            let mut primary = running.classify_before_cancellation(joined);
            primary.error = apply_stopping_transition(
                primary.error,
                on_stopping
                    .take()
                    .expect("stopping transition callback must be available"),
            );
            let mut candidates = running.cleanup().await;
            return Err(select_task_primary(primary, &mut candidates));
        }

        Ok(running)
    }
}

struct ServiceControl {
    name: String,
    cancellation: CancellationSource,
    active: bool,
}

struct TaskCompletion {
    index: usize,
    result: Result<(), BoxError>,
    cancellation_requested: bool,
}

enum JoinedTask {
    Completed(TaskCompletion),
    JoinFailed { index: usize, error: JoinError },
}

struct FailureCandidate {
    index: usize,
    error: RuntimeError,
}

/// Runtime-owned handles for services that acknowledged startup.
pub struct RunningServices {
    controls: Vec<ServiceControl>,
    tasks: JoinSet<TaskCompletion>,
    task_indices: HashMap<Id, usize>,
}

impl std::fmt::Debug for RunningServices {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RunningServices")
            .field(
                "active_services",
                &self
                    .controls
                    .iter()
                    .filter(|control| control.active)
                    .map(|control| control.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl RunningServices {
    fn new() -> Self {
        Self {
            controls: Vec::new(),
            tasks: JoinSet::new(),
            task_indices: HashMap::new(),
        }
    }

    /// Returns the number of services that acknowledged startup.
    #[must_use]
    pub fn service_count(&self) -> usize {
        self.controls.len()
    }

    /// Waits for requested shutdown or a terminal service outcome.
    ///
    /// Every acknowledged service is cancelled in reverse registration order
    /// and joined before this future returns.
    ///
    /// # Errors
    ///
    /// Returns a shutdown-source, unexpected-exit, service, or task-join error
    /// after complete cleanup.
    pub async fn run_until<F, E>(self, shutdown: F) -> Result<(), RuntimeError>
    where
        F: Future<Output = Result<(), E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.run_until_with_stopping(shutdown, || Ok(())).await
    }

    pub(crate) async fn run_until_with_stopping<F, E, C>(
        mut self,
        shutdown: F,
        on_stopping: C,
    ) -> Result<(), RuntimeError>
    where
        F: Future<Output = Result<(), E>>,
        E: std::error::Error + Send + Sync + 'static,
        C: FnOnce() -> Result<(), RuntimeError>,
    {
        let mut on_stopping = Some(on_stopping);
        if !self.has_active_tasks() {
            return match shutdown.await {
                Ok(()) => on_stopping
                    .take()
                    .expect("stopping transition callback must be available")(
                ),
                Err(error) => Err(apply_stopping_transition(
                    RuntimeError::ShutdownSourceFailed {
                        source: Box::new(error),
                        cleanup: Vec::new(),
                    },
                    on_stopping
                        .take()
                        .expect("stopping transition callback must be available"),
                )),
            };
        }

        tokio::pin!(shutdown);
        tokio::select! {
            biased;
            joined = self.join_next() => {
                let mut primary = self.classify_before_cancellation(joined);
                primary.error = apply_stopping_transition(
                    primary.error,
                    on_stopping
                        .take()
                        .expect("stopping transition callback must be available"),
                );
                let mut candidates = self.cleanup().await;
                Err(select_task_primary(primary, &mut candidates))
            }
            result = &mut shutdown => {
                match result {
                    Ok(()) => {
                        let transition_error = on_stopping
                            .take()
                            .expect("stopping transition callback must be available")()
                            .err();
                        let mut candidates = self.cleanup().await;
                        if candidates.is_empty() {
                            transition_error.map_or(Ok(()), Err)
                        } else {
                            let primary = candidates.remove(0);
                            let mut error = select_task_primary(primary, &mut candidates);
                            if let Some(transition_error) = transition_error {
                                error = error.with_cleanup(vec![CleanupFailure::from_error(
                                    &transition_error,
                                )]);
                            }
                            Err(error)
                        }
                    }
                    Err(error) => {
                        let primary = apply_stopping_transition(
                            RuntimeError::ShutdownSourceFailed {
                                source: Box::new(error),
                                cleanup: Vec::new(),
                            },
                            on_stopping
                                .take()
                                .expect("stopping transition callback must be available"),
                        );
                        Err(self.finish_fixed_primary(primary).await)
                    }
                }
            }
        }
    }

    fn has_active_tasks(&self) -> bool {
        self.controls.iter().any(|control| control.active)
    }

    fn spawn(&mut self, name: String, cancellation: CancellationSource, task: ServiceTask) {
        let index = self.controls.len();
        let cancellation_observer = cancellation.subscribe();
        let handle = self.tasks.spawn(async move {
            let result = task.await;
            TaskCompletion {
                index,
                result,
                cancellation_requested: cancellation_observer.is_requested(),
            }
        });
        let previous = self.task_indices.insert(handle.id(), index);
        debug_assert!(previous.is_none());
        self.controls.push(ServiceControl {
            name,
            cancellation,
            active: true,
        });
    }

    async fn join_next(&mut self) -> JoinedTask {
        let result = self
            .tasks
            .join_next_with_id()
            .await
            .expect("an active service must have an owned task");
        self.record_join(result)
    }

    fn try_join_next(&mut self) -> Option<JoinedTask> {
        let result = self.tasks.try_join_next_with_id()?;
        Some(self.record_join(result))
    }

    fn record_join(&mut self, result: Result<(Id, TaskCompletion), JoinError>) -> JoinedTask {
        match result {
            Ok((task_id, completion)) => {
                let index = self
                    .task_indices
                    .remove(&task_id)
                    .expect("joined service task must be registered");
                debug_assert_eq!(index, completion.index);
                self.controls[index].active = false;
                JoinedTask::Completed(completion)
            }
            Err(error) => {
                let index = self
                    .task_indices
                    .remove(&error.id())
                    .expect("failed service task must be registered");
                self.controls[index].active = false;
                JoinedTask::JoinFailed { index, error }
            }
        }
    }

    fn classify(&self, joined: JoinedTask) -> Option<FailureCandidate> {
        match joined {
            JoinedTask::Completed(completion) => {
                let service = self.controls[completion.index].name.clone();
                match completion.result {
                    Ok(()) if completion.cancellation_requested => None,
                    Ok(()) => Some(FailureCandidate {
                        index: completion.index,
                        error: RuntimeError::UnexpectedServiceExit {
                            service,
                            cleanup: Vec::new(),
                        },
                    }),
                    Err(source) => Some(FailureCandidate {
                        index: completion.index,
                        error: RuntimeError::ServiceFailed {
                            service,
                            source,
                            cleanup: Vec::new(),
                        },
                    }),
                }
            }
            JoinedTask::JoinFailed { index, error } => Some(FailureCandidate {
                index,
                error: RuntimeError::ServiceTaskJoinFailed {
                    service: self.controls[index].name.clone(),
                    source: Box::new(error),
                    cleanup: Vec::new(),
                },
            }),
        }
    }

    fn classify_before_cancellation(&self, joined: JoinedTask) -> FailureCandidate {
        match joined {
            JoinedTask::Completed(completion) => {
                let service = self.controls[completion.index].name.clone();
                let error = match completion.result {
                    Ok(()) => RuntimeError::UnexpectedServiceExit {
                        service,
                        cleanup: Vec::new(),
                    },
                    Err(source) => RuntimeError::ServiceFailed {
                        service,
                        source,
                        cleanup: Vec::new(),
                    },
                };
                FailureCandidate {
                    index: completion.index,
                    error,
                }
            }
            JoinedTask::JoinFailed { index, error } => FailureCandidate {
                index,
                error: RuntimeError::ServiceTaskJoinFailed {
                    service: self.controls[index].name.clone(),
                    source: Box::new(error),
                    cleanup: Vec::new(),
                },
            },
        }
    }

    async fn cleanup(&mut self) -> Vec<FailureCandidate> {
        let mut failures = Vec::new();

        for target in (0..self.controls.len()).rev() {
            if !self.controls[target].active {
                continue;
            }

            self.controls[target].cancellation.request();
            while self.controls[target].active {
                let joined = self.join_next().await;
                if let Some(failure) = self.classify(joined) {
                    failures.push(failure);
                }
            }
        }

        debug_assert!(self.tasks.is_empty());
        failures
    }

    async fn finish_fixed_primary(mut self, primary: RuntimeError) -> RuntimeError {
        let failures = self.cleanup().await;
        let cleanup = failures
            .iter()
            .map(|failure| CleanupFailure::from_error(&failure.error))
            .collect();
        primary.with_cleanup(cleanup)
    }
}

fn select_task_primary(
    primary: FailureCandidate,
    additional: &mut Vec<FailureCandidate>,
) -> RuntimeError {
    additional.push(primary);
    additional.sort_by_key(|candidate| candidate.index);
    let selected = additional.remove(0);
    let cleanup = additional
        .iter()
        .map(|candidate| CleanupFailure::from_error(&candidate.error))
        .collect();
    selected.error.with_cleanup(cleanup)
}

fn apply_stopping_transition<C>(primary: RuntimeError, on_stopping: C) -> RuntimeError
where
    C: FnOnce() -> Result<(), RuntimeError>,
{
    match on_stopping() {
        Ok(()) => primary,
        Err(error) => primary.with_cleanup(vec![CleanupFailure::from_error(&error)]),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::future::pending;
    use std::io;
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::{mpsc, oneshot};
    use tokio::time::timeout;

    use crate::{
        AppState, BoxError, RuntimeConfig, RuntimeErrorKind, RuntimeService, ServiceContext,
        ServiceStartFuture, ServiceTask,
    };

    use super::ServiceContainerBuilder;

    #[derive(Debug, PartialEq, Eq)]
    enum Event {
        Started(&'static str),
        StartAttempted(&'static str),
        Stopped(&'static str),
    }

    fn state() -> Arc<AppState> {
        Arc::new(AppState::new(RuntimeConfig::default()))
    }

    fn waiting_service(
        name: &'static str,
        events: mpsc::UnboundedSender<Event>,
    ) -> impl RuntimeService {
        move |context: ServiceContext| -> ServiceStartFuture {
            Box::pin(async move {
                events
                    .send(Event::Started(name))
                    .expect("event receiver must remain available");
                let mut cancellation = context.cancellation();
                let task: ServiceTask = Box::pin(async move {
                    cancellation.cancelled().await;
                    events
                        .send(Event::Stopped(name))
                        .expect("event receiver must remain available");
                    Ok(())
                });
                Ok(task)
            })
        }
    }

    fn failing_start_service(
        name: &'static str,
        events: mpsc::UnboundedSender<Event>,
    ) -> impl RuntimeService {
        move |_context: ServiceContext| -> ServiceStartFuture {
            Box::pin(async move {
                events
                    .send(Event::StartAttempted(name))
                    .expect("event receiver must remain available");
                Err(Box::new(io::Error::other("start failed")) as BoxError)
            })
        }
    }

    fn failing_service(
        name: &'static str,
        events: mpsc::UnboundedSender<Event>,
        trigger: oneshot::Receiver<()>,
    ) -> impl RuntimeService {
        move |_context: ServiceContext| -> ServiceStartFuture {
            Box::pin(async move {
                events
                    .send(Event::Started(name))
                    .expect("event receiver must remain available");
                let task: ServiceTask = Box::pin(async move {
                    trigger.await.expect("failure trigger must be sent");
                    Err(Box::new(io::Error::other("service failed")) as BoxError)
                });
                Ok(task)
            })
        }
    }

    fn successful_exit_service(trigger: oneshot::Receiver<()>) -> impl RuntimeService {
        move |_context: ServiceContext| -> ServiceStartFuture {
            Box::pin(async move {
                let task: ServiceTask = Box::pin(async move {
                    trigger.await.expect("exit trigger must be sent");
                    Ok(())
                });
                Ok(task)
            })
        }
    }

    fn panicking_service(trigger: oneshot::Receiver<()>) -> impl RuntimeService {
        move |_context: ServiceContext| -> ServiceStartFuture {
            Box::pin(async move {
                let task: ServiceTask = Box::pin(async move {
                    trigger.await.expect("panic trigger must be sent");
                    panic!("service task panic");
                });
                Ok(task)
            })
        }
    }

    async fn next_event(events: &mut mpsc::UnboundedReceiver<Event>) -> Event {
        timeout(Duration::from_secs(1), events.recv())
            .await
            .expect("event wait must not hang")
            .expect("event sender must remain available")
    }

    #[test]
    fn service_container_rejects_invalid_and_duplicate_names() {
        let (sender, _receiver) = mpsc::unbounded_channel();
        let error = ServiceContainerBuilder::new()
            .register("", waiting_service("empty", sender.clone()))
            .expect_err("an empty service name must fail");
        assert_eq!(error.kind(), RuntimeErrorKind::InvalidServiceName);

        let error = ServiceContainerBuilder::new()
            .register("worker", waiting_service("first", sender.clone()))
            .expect("the first registration must succeed")
            .register("worker", waiting_service("second", sender))
            .expect_err("a duplicate service name must fail");
        assert_eq!(error.kind(), RuntimeErrorKind::DuplicateServiceName);
        assert_eq!(error.service_name(), Some("worker"));
    }

    #[tokio::test]
    async fn service_container_starts_in_order_and_stops_in_reverse_order() {
        let (sender, mut events) = mpsc::unbounded_channel();
        let container = ServiceContainerBuilder::new()
            .register("first", waiting_service("first", sender.clone()))
            .expect("first service must register")
            .register("second", waiting_service("second", sender))
            .expect("second service must register")
            .build(state());

        let running = container.start().await.expect("services must start");
        assert_eq!(running.service_count(), 2);
        assert_eq!(next_event(&mut events).await, Event::Started("first"));
        assert_eq!(next_event(&mut events).await, Event::Started("second"));

        running
            .run_until(async { Ok::<(), Infallible>(()) })
            .await
            .expect("requested shutdown must succeed");

        assert_eq!(next_event(&mut events).await, Event::Stopped("second"));
        assert_eq!(next_event(&mut events).await, Event::Stopped("first"));
    }

    #[tokio::test]
    async fn service_container_rolls_back_partial_startup() {
        let (sender, mut events) = mpsc::unbounded_channel();
        let container = ServiceContainerBuilder::new()
            .register("first", waiting_service("first", sender.clone()))
            .expect("first service must register")
            .register("second", failing_start_service("second", sender))
            .expect("second service must register")
            .build(state());

        let error = container
            .start()
            .await
            .expect_err("the second startup must fail");

        assert_eq!(error.kind(), RuntimeErrorKind::ServiceStartFailed);
        assert_eq!(error.service_name(), Some("second"));
        assert!(error.cleanup_failures().is_empty());
        assert_eq!(next_event(&mut events).await, Event::Started("first"));
        assert_eq!(
            next_event(&mut events).await,
            Event::StartAttempted("second")
        );
        assert_eq!(next_event(&mut events).await, Event::Stopped("first"));
    }

    #[tokio::test]
    async fn service_container_propagates_failure_and_cleans_siblings() {
        let (sender, mut events) = mpsc::unbounded_channel();
        let (trigger_sender, trigger) = oneshot::channel();
        let container = ServiceContainerBuilder::new()
            .register("first", waiting_service("first", sender.clone()))
            .expect("first service must register")
            .register(
                "failing",
                failing_service("failing", sender.clone(), trigger),
            )
            .expect("failing service must register")
            .register("last", waiting_service("last", sender))
            .expect("last service must register")
            .build(state());
        let running = container.start().await.expect("services must start");
        assert_eq!(next_event(&mut events).await, Event::Started("first"));
        assert_eq!(next_event(&mut events).await, Event::Started("failing"));
        assert_eq!(next_event(&mut events).await, Event::Started("last"));

        trigger_sender
            .send(())
            .expect("failure trigger must be observed");
        let error = running
            .run_until(pending::<Result<(), Infallible>>())
            .await
            .expect_err("service failure must reach the owner");

        assert_eq!(error.kind(), RuntimeErrorKind::ServiceFailed);
        assert_eq!(error.service_name(), Some("failing"));
        assert_eq!(next_event(&mut events).await, Event::Stopped("last"));
        assert_eq!(next_event(&mut events).await, Event::Stopped("first"));
    }

    #[tokio::test]
    async fn service_container_classifies_unexpected_successful_exit() {
        let (trigger_sender, trigger) = oneshot::channel();
        let container = ServiceContainerBuilder::new()
            .register("early", successful_exit_service(trigger))
            .expect("service must register")
            .build(state());
        let running = container.start().await.expect("service must start");

        trigger_sender
            .send(())
            .expect("exit trigger must be observed");
        let error = running
            .run_until(pending::<Result<(), Infallible>>())
            .await
            .expect_err("early success must fail the run");

        assert_eq!(error.kind(), RuntimeErrorKind::UnexpectedServiceExit);
        assert_eq!(error.service_name(), Some("early"));
    }

    #[tokio::test]
    async fn service_container_classifies_task_panic_and_joins_it() {
        let (trigger_sender, trigger) = oneshot::channel();
        let container = ServiceContainerBuilder::new()
            .register("panic", panicking_service(trigger))
            .expect("service must register")
            .build(state());
        let running = container.start().await.expect("service must start");

        trigger_sender
            .send(())
            .expect("panic trigger must be observed");
        let error = running
            .run_until(pending::<Result<(), Infallible>>())
            .await
            .expect_err("task panic must fail the run");

        assert_eq!(error.kind(), RuntimeErrorKind::ServiceTaskJoinFailed);
        assert_eq!(error.service_name(), Some("panic"));
    }

    #[tokio::test]
    async fn service_container_supports_repeated_fresh_construction() {
        for name in ["first-run", "second-run"] {
            let (sender, mut events) = mpsc::unbounded_channel();
            let container = ServiceContainerBuilder::new()
                .register(name, waiting_service(name, sender))
                .expect("fresh service must register")
                .build(state());
            let running = container.start().await.expect("fresh service must start");

            assert_eq!(next_event(&mut events).await, Event::Started(name));
            running
                .run_until(async { Ok::<(), Infallible>(()) })
                .await
                .expect("fresh shutdown must succeed");
            assert_eq!(next_event(&mut events).await, Event::Stopped(name));
        }
    }
}
