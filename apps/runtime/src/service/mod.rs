//! Runtime-owned long-lived service execution.

mod cancellation;
mod container;
mod definition;

pub use cancellation::Cancellation;
pub use container::{RunningServices, ServiceContainer, ServiceContainerBuilder};
pub use definition::{RuntimeService, ServiceContext, ServiceStartFuture, ServiceTask};
