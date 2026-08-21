//! Runtime application.

use std::future::Future;
use std::sync::Arc;

use crate::app::{AppBuilder, Lifecycle, LifecycleState};
use crate::error::{CleanupFailure, RuntimeError};
use crate::service::ServiceContainer;
use crate::state::AppState;

/// Root application object for `OneAgent Runtime`.
#[derive(Debug)]
pub struct App {
    state: Arc<AppState>,
    lifecycle: Lifecycle,
    services: ServiceContainer,
}

impl App {
    /// Creates an application builder.
    #[must_use]
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    pub(crate) const fn new(
        state: Arc<AppState>,
        lifecycle: Lifecycle,
        services: ServiceContainer,
    ) -> Self {
        Self {
            state,
            lifecycle,
            services,
        }
    }

    /// Returns shared application state.
    #[must_use]
    pub fn state(&self) -> &AppState {
        self.state.as_ref()
    }

    /// Subscribes to transport-neutral application lifecycle changes.
    #[must_use]
    pub fn subscribe_lifecycle(&self) -> tokio::sync::watch::Receiver<LifecycleState> {
        self.lifecycle.subscribe()
    }

    /// Runs until the injected shutdown source fires or a service terminates.
    ///
    /// # Errors
    ///
    /// Returns a lifecycle, startup, service, task-join, or shutdown-source
    /// error after all Runtime-owned service tasks have terminated.
    pub async fn run<F, E>(self, shutdown: F) -> Result<(), RuntimeError>
    where
        F: Future<Output = Result<(), E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let Self {
            state,
            mut lifecycle,
            services,
        } = self;

        let running = match services
            .start_with_stopping(|| lifecycle.transition_to(LifecycleState::Stopping))
            .await
        {
            Ok(running) => running,
            Err(error) => return finish_stopped(&mut lifecycle, Err(error)),
        };

        if let Err(error) = lifecycle.transition_to(LifecycleState::Running) {
            let cleanup = running
                .run_until_with_stopping(async { Ok::<(), std::convert::Infallible>(()) }, || {
                    lifecycle.transition_to(LifecycleState::Stopping)
                })
                .await;
            let error = match cleanup {
                Ok(()) => error,
                Err(cleanup_error) => {
                    error.with_cleanup(vec![CleanupFailure::from_error(&cleanup_error)])
                }
            };
            return finish_stopped(&mut lifecycle, Err(error));
        }

        println!(
            "{} {} [{}]",
            state.configuration().application_name(),
            env!("CARGO_PKG_VERSION"),
            state.configuration().environment()
        );

        let result = running
            .run_until_with_stopping(shutdown, || {
                lifecycle.transition_to(LifecycleState::Stopping)
            })
            .await;

        finish_stopped(&mut lifecycle, result)
    }
}

fn finish_stopped(
    lifecycle: &mut Lifecycle,
    result: Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    match lifecycle.transition_to(LifecycleState::Stopped) {
        Ok(()) => result,
        Err(lifecycle_error) => match result {
            Ok(()) => Err(lifecycle_error),
            Err(error) => {
                Err(error.with_cleanup(vec![CleanupFailure::from_error(&lifecycle_error)]))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::future::pending;
    use std::io;
    use std::time::Duration;

    use tokio::sync::oneshot;
    use tokio::time::timeout;

    use crate::config::DefaultConfigurationProvider;
    use crate::error::{BoxError, RuntimeErrorKind};
    use crate::service::{ServiceContext, ServiceStartFuture, ServiceTask};

    use super::{App, LifecycleState};

    async fn wait_for_state(
        lifecycle: &mut tokio::sync::watch::Receiver<LifecycleState>,
        expected: LifecycleState,
    ) {
        while *lifecycle.borrow() != expected {
            timeout(Duration::from_secs(1), lifecycle.changed())
                .await
                .expect("lifecycle wait must not hang")
                .expect("application must retain lifecycle ownership");
        }
    }

    #[tokio::test]
    async fn app_run_remains_pending_and_stops_after_owned_service_cleanup() {
        let (started_sender, started) = oneshot::channel();
        let (cancelled_sender, cancelled) = oneshot::channel();
        let (release_sender, release) = oneshot::channel();
        let service = move |context: ServiceContext| -> ServiceStartFuture {
            Box::pin(async move {
                started_sender
                    .send(())
                    .expect("startup acknowledgement must be observed");
                let mut cancellation = context.cancellation();
                let task: ServiceTask = Box::pin(async move {
                    cancellation.cancelled().await;
                    cancelled_sender
                        .send(())
                        .expect("cancellation acknowledgement must be observed");
                    release.await.expect("service release must be sent");
                    Ok(())
                });
                Ok(task)
            })
        };
        let app = App::builder()
            .configure(&DefaultConfigurationProvider)
            .expect("default configuration must load")
            .register_service("controlled", service)
            .expect("service must register")
            .build()
            .expect("application must build");
        let mut lifecycle = app.subscribe_lifecycle();
        assert_eq!(*lifecycle.borrow(), LifecycleState::Initializing);
        let (shutdown_sender, shutdown) = oneshot::channel();

        let run = tokio::spawn(app.run(shutdown));
        timeout(Duration::from_secs(1), started)
            .await
            .expect("startup must not hang")
            .expect("service must acknowledge startup");
        wait_for_state(&mut lifecycle, LifecycleState::Running).await;
        assert!(!run.is_finished());

        shutdown_sender
            .send(())
            .expect("shutdown request must be observed");
        timeout(Duration::from_secs(1), cancelled)
            .await
            .expect("cancellation must not hang")
            .expect("service must acknowledge cancellation");
        assert_eq!(*lifecycle.borrow(), LifecycleState::Stopping);
        assert!(!run.is_finished());

        release_sender
            .send(())
            .expect("service release must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("application shutdown must not hang")
            .expect("application task must join")
            .expect("requested shutdown must succeed");
        wait_for_state(&mut lifecycle, LifecycleState::Stopped).await;
    }

    #[tokio::test]
    async fn app_run_propagates_service_failure_after_stopping() {
        let (started_sender, started) = oneshot::channel();
        let (failure_sender, failure) = oneshot::channel();
        let service = move |_context: ServiceContext| -> ServiceStartFuture {
            Box::pin(async move {
                started_sender
                    .send(())
                    .expect("startup acknowledgement must be observed");
                let task: ServiceTask = Box::pin(async move {
                    failure.await.expect("failure trigger must be sent");
                    Err(Box::new(io::Error::other("service failed")) as BoxError)
                });
                Ok(task)
            })
        };
        let app = App::builder()
            .configure(&DefaultConfigurationProvider)
            .expect("default configuration must load")
            .register_service("failing", service)
            .expect("service must register")
            .build()
            .expect("application must build");
        let mut lifecycle = app.subscribe_lifecycle();
        let run = tokio::spawn(app.run(pending::<Result<(), Infallible>>()));
        timeout(Duration::from_secs(1), started)
            .await
            .expect("startup must not hang")
            .expect("service must acknowledge startup");
        wait_for_state(&mut lifecycle, LifecycleState::Running).await;

        failure_sender
            .send(())
            .expect("failure trigger must be observed");
        let error = timeout(Duration::from_secs(1), run)
            .await
            .expect("failed application must not hang")
            .expect("application task must join")
            .expect_err("service failure must reach the App caller");

        assert_eq!(error.kind(), RuntimeErrorKind::ServiceFailed);
        assert_eq!(error.service_name(), Some("failing"));
        wait_for_state(&mut lifecycle, LifecycleState::Stopped).await;
    }

    #[tokio::test]
    async fn app_run_supports_repeated_fresh_empty_applications() {
        for _ in 0..2 {
            let app = App::builder()
                .configure(&DefaultConfigurationProvider)
                .expect("default configuration must load")
                .build()
                .expect("application must build");
            let mut lifecycle = app.subscribe_lifecycle();

            app.run(async { Ok::<(), Infallible>(()) })
                .await
                .expect("fresh application shutdown must succeed");
            wait_for_state(&mut lifecycle, LifecycleState::Stopped).await;
        }
    }
}
