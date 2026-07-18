//! Runtime application.

use crate::app::{AppBuilder, Lifecycle, LifecycleState};
use crate::error::RuntimeError;
use crate::state::AppState;

/// Root application object for `OneAgent Runtime`.
#[derive(Debug)]
pub struct App {
    state: AppState,
    lifecycle: Lifecycle,
}

impl App {
    /// Creates an application builder.
    #[must_use]
    pub const fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    pub(crate) const fn new(state: AppState, lifecycle: Lifecycle) -> Self {
        Self { state, lifecycle }
    }

    /// Returns shared application state.
    #[cfg(test)]
    #[must_use]
    pub const fn state(&self) -> &AppState {
        &self.state
    }

    /// Runs the core runtime lifecycle.
    ///
    /// # Errors
    ///
    /// Returns an error when a lifecycle transition is invalid.
    pub fn run(mut self) -> Result<(), RuntimeError> {
        self.lifecycle.transition_to(LifecycleState::Running)?;

        println!(
            "{} {} [{}]",
            self.state.configuration().application_name(),
            env!("CARGO_PKG_VERSION"),
            self.state.configuration().environment()
        );

        self.lifecycle.transition_to(LifecycleState::Stopping)?;
        self.lifecycle.transition_to(LifecycleState::Stopped)?;

        Ok(())
    }
}
