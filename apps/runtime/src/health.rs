//! Transport-neutral Runtime health observation.

use tokio::sync::watch;

use crate::LifecycleState;

/// Read-only observation of canonical Runtime lifecycle health.
#[derive(Debug, Clone)]
pub struct RuntimeHealth {
    lifecycle: watch::Receiver<LifecycleState>,
}

impl RuntimeHealth {
    pub(crate) const fn new(lifecycle: watch::Receiver<LifecycleState>) -> Self {
        Self { lifecycle }
    }

    pub(crate) fn detached(state: LifecycleState) -> Self {
        let (_sender, lifecycle) = watch::channel(state);
        Self::new(lifecycle)
    }

    /// Returns a point-in-time projection of canonical Runtime lifecycle state.
    #[must_use]
    pub fn snapshot(&self) -> RuntimeHealthSnapshot {
        RuntimeHealthSnapshot::new(*self.lifecycle.borrow())
    }
}

/// Immutable point-in-time Runtime health projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeHealthSnapshot {
    lifecycle: LifecycleState,
}

impl RuntimeHealthSnapshot {
    const fn new(lifecycle: LifecycleState) -> Self {
        Self { lifecycle }
    }

    /// Returns the lifecycle state used to derive this snapshot.
    #[must_use]
    pub const fn lifecycle(self) -> LifecycleState {
        self.lifecycle
    }

    /// Returns whether Runtime has started every required service and is not stopping.
    #[must_use]
    pub const fn is_ready(self) -> bool {
        matches!(self.lifecycle, LifecycleState::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Lifecycle, LifecycleState};

    use super::{RuntimeHealth, RuntimeHealthSnapshot};

    #[test]
    fn health_snapshot_maps_every_lifecycle_state() {
        let cases = [
            (LifecycleState::Created, false),
            (LifecycleState::Building, false),
            (LifecycleState::Initializing, false),
            (LifecycleState::Running, true),
            (LifecycleState::Stopping, false),
            (LifecycleState::Stopped, false),
        ];

        for (lifecycle, expected_ready) in cases {
            let snapshot = RuntimeHealthSnapshot::new(lifecycle);
            assert_eq!(snapshot.lifecycle(), lifecycle);
            assert_eq!(snapshot.is_ready(), expected_ready);
        }
    }

    #[test]
    fn health_tracks_the_canonical_lifecycle_watch() {
        let mut lifecycle = Lifecycle::new();
        let health = RuntimeHealth::new(lifecycle.subscribe());

        assert_eq!(health.snapshot().lifecycle(), LifecycleState::Created);
        assert!(!health.snapshot().is_ready());

        lifecycle
            .transition_to(LifecycleState::Building)
            .expect("building transition must succeed");
        lifecycle
            .transition_to(LifecycleState::Initializing)
            .expect("initializing transition must succeed");
        lifecycle
            .transition_to(LifecycleState::Running)
            .expect("running transition must succeed");
        assert!(health.snapshot().is_ready());

        lifecycle
            .transition_to(LifecycleState::Stopping)
            .expect("stopping transition must succeed");
        assert!(!health.snapshot().is_ready());
        lifecycle
            .transition_to(LifecycleState::Stopped)
            .expect("stopped transition must succeed");
        assert_eq!(health.snapshot().lifecycle(), LifecycleState::Stopped);
    }

    #[test]
    fn health_observers_from_fresh_lifecycles_are_independent() {
        let mut first_lifecycle = Lifecycle::new();
        let first_health = RuntimeHealth::new(first_lifecycle.subscribe());
        let second_lifecycle = Lifecycle::new();
        let second_health = RuntimeHealth::new(second_lifecycle.subscribe());

        first_lifecycle
            .transition_to(LifecycleState::Building)
            .expect("first building transition must succeed");

        assert_eq!(
            first_health.snapshot().lifecycle(),
            LifecycleState::Building
        );
        assert_eq!(
            second_health.snapshot().lifecycle(),
            LifecycleState::Created
        );
    }
}
