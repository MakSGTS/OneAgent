//! Reusable `OneAgent Runtime` composition and service boundaries.

mod app;
mod config;
mod error;
mod health;
mod http;
mod service;
mod state;
mod workspace;

pub use app::{App, AppBuilder, Lifecycle, LifecycleState};
pub use config::{ConfigurationProvider, DefaultConfigurationProvider, RuntimeConfig};
pub use error::{BoxError, CleanupFailure, RuntimeError, RuntimeErrorKind};
pub use health::{RuntimeHealth, RuntimeHealthSnapshot};
pub use http::HttpService;
pub use service::{
    Cancellation, RunningServices, RuntimeService, ServiceContainer, ServiceContainerBuilder,
    ServiceContext, ServiceStartFuture, ServiceTask,
};
pub use state::AppState;
pub use workspace::{
    WorkspaceBuildError, WorkspaceBuildErrorKind, WorkspaceConfigurationSnapshot,
    WorkspaceSnapshot, WorkspaceSnapshotBuilder,
};
