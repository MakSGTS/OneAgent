//! Reusable `OneAgent Runtime` composition and service boundaries.

mod app;
mod config;
mod error;
mod health;
mod service;
mod state;

pub use app::{App, AppBuilder, Lifecycle, LifecycleState};
pub use config::{ConfigurationProvider, DefaultConfigurationProvider, RuntimeConfig};
pub use error::{BoxError, CleanupFailure, RuntimeError, RuntimeErrorKind};
pub use health::{RuntimeHealth, RuntimeHealthSnapshot};
pub use service::{
    Cancellation, RunningServices, RuntimeService, ServiceContainer, ServiceContainerBuilder,
    ServiceContext, ServiceStartFuture, ServiceTask,
};
pub use state::AppState;
