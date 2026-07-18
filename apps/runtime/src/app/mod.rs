//! Runtime composition root.

mod builder;
mod lifecycle;
mod runtime;

pub use builder::AppBuilder;
pub use lifecycle::{Lifecycle, LifecycleState};
pub use runtime::App;
