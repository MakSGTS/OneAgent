//! Application lifecycle state.

use crate::error::RuntimeError;
use tokio::sync::watch;

/// Lifecycle states of `OneAgent Runtime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// Application object exists but is not initialized.
    Created,
    /// Dependencies are being assembled.
    Building,
    /// Services are initializing.
    Initializing,
    /// Runtime is serving requests and executing work.
    Running,
    /// Runtime is shutting down.
    Stopping,
    /// Runtime has stopped.
    Stopped,
}

impl LifecycleState {
    /// Returns a stable string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Building => "building",
            Self::Initializing => "initializing",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
        }
    }

    /// Checks whether a transition to `next` is allowed.
    #[must_use]
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Building)
                | (Self::Building, Self::Initializing)
                | (Self::Initializing, Self::Running | Self::Stopping)
                | (Self::Running, Self::Stopping)
                | (Self::Stopping, Self::Stopped)
        )
    }
}

/// Mutable lifecycle controller.
#[derive(Debug)]
pub struct Lifecycle {
    state: LifecycleState,
    sender: watch::Sender<LifecycleState>,
}

impl Lifecycle {
    /// Creates a lifecycle in the `Created` state.
    #[must_use]
    pub fn new() -> Self {
        let (sender, _receiver) = watch::channel(LifecycleState::Created);
        Self {
            state: LifecycleState::Created,
            sender,
        }
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> LifecycleState {
        self.state
    }

    /// Subscribes to transport-neutral lifecycle state changes.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<LifecycleState> {
        self.sender.subscribe()
    }

    /// Moves the lifecycle to the requested state.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::InvalidLifecycleTransition`] when the transition is not allowed.
    pub fn transition_to(&mut self, next: LifecycleState) -> Result<(), RuntimeError> {
        if !self.state.can_transition_to(next) {
            return Err(RuntimeError::InvalidLifecycleTransition {
                from: self.state.as_str(),
                to: next.as_str(),
            });
        }

        self.state = next;
        self.sender.send_replace(next);
        Ok(())
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Lifecycle, LifecycleState};

    #[test]
    fn valid_lifecycle_sequence_succeeds() {
        let mut lifecycle = Lifecycle::new();

        lifecycle
            .transition_to(LifecycleState::Building)
            .expect("created -> building must be valid");
        lifecycle
            .transition_to(LifecycleState::Initializing)
            .expect("building -> initializing must be valid");
        lifecycle
            .transition_to(LifecycleState::Running)
            .expect("initializing -> running must be valid");
        lifecycle
            .transition_to(LifecycleState::Stopping)
            .expect("running -> stopping must be valid");
        lifecycle
            .transition_to(LifecycleState::Stopped)
            .expect("stopping -> stopped must be valid");

        assert_eq!(lifecycle.state(), LifecycleState::Stopped);
    }

    #[test]
    fn invalid_transition_returns_error() {
        let mut lifecycle = Lifecycle::new();

        let error = lifecycle
            .transition_to(LifecycleState::Running)
            .expect_err("created -> running must be rejected");

        assert_eq!(
            error.to_string(),
            "invalid lifecycle transition: created -> running"
        );
    }

    #[test]
    fn initialization_can_transition_to_failure_cleanup() {
        let mut lifecycle = Lifecycle::new();
        lifecycle
            .transition_to(LifecycleState::Building)
            .expect("created -> building must be valid");
        lifecycle
            .transition_to(LifecycleState::Initializing)
            .expect("building -> initializing must be valid");
        lifecycle
            .transition_to(LifecycleState::Stopping)
            .expect("initializing -> stopping must be valid for rollback");
        lifecycle
            .transition_to(LifecycleState::Stopped)
            .expect("stopping -> stopped must be valid");

        assert_eq!(lifecycle.state(), LifecycleState::Stopped);
    }
}
