//! Transport-independent Runtime service contract.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::{AppState, BoxError};

use super::Cancellation;

/// The owned future executed for one acknowledged Runtime service.
pub type ServiceTask = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'static>>;

/// The owned future that initializes one Runtime service.
pub type ServiceStartFuture =
    Pin<Box<dyn Future<Output = Result<ServiceTask, BoxError>> + Send + 'static>>;

/// Immutable state and receiver-only cancellation supplied during service start.
#[derive(Debug, Clone)]
pub struct ServiceContext {
    state: Arc<AppState>,
    cancellation: Cancellation,
}

impl ServiceContext {
    pub(super) const fn new(state: Arc<AppState>, cancellation: Cancellation) -> Self {
        Self {
            state,
            cancellation,
        }
    }

    /// Returns the immutable shared application state.
    #[must_use]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Returns a receiver-only cancellation handle for the service task.
    #[must_use]
    pub fn cancellation(&self) -> Cancellation {
        self.cancellation.clone()
    }

    /// Splits the context into shared state and cancellation ownership.
    #[must_use]
    pub fn into_parts(self) -> (Arc<AppState>, Cancellation) {
        (self.state, self.cancellation)
    }
}

/// A Runtime-owned long-lived service definition.
pub trait RuntimeService: Send + 'static {
    /// Initializes the service and returns its acknowledged task future.
    fn start(self: Box<Self>, context: ServiceContext) -> ServiceStartFuture;
}

impl<F> RuntimeService for F
where
    F: FnOnce(ServiceContext) -> ServiceStartFuture + Send + 'static,
{
    fn start(self: Box<Self>, context: ServiceContext) -> ServiceStartFuture {
        (*self)(context)
    }
}
