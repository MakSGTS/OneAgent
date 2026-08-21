//! Runtime error types.

use std::fmt::{Display, Formatter};

/// A thread-safe owned error used at Runtime service boundaries.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Stable in-process classification of a Runtime error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    /// Required Runtime configuration was not supplied.
    MissingConfiguration,
    /// A lifecycle transition violated the accepted state machine.
    InvalidLifecycleTransition,
    /// A service name was empty.
    InvalidServiceName,
    /// A service name was registered more than once.
    DuplicateServiceName,
    /// A service could not acknowledge startup.
    ServiceStartFailed,
    /// A service completed successfully before cancellation.
    UnexpectedServiceExit,
    /// A service returned an execution error.
    ServiceFailed,
    /// A Runtime-owned service task failed to join.
    ServiceTaskJoinFailed,
    /// The injected shutdown source failed.
    ShutdownSourceFailed,
}

/// A secondary failure observed while cleaning up after a primary cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupFailure {
    service: Option<String>,
    kind: RuntimeErrorKind,
    message: String,
}

impl CleanupFailure {
    pub(crate) fn from_error(error: &RuntimeError) -> Self {
        Self {
            service: error.service_name().map(str::to_owned),
            kind: error.kind(),
            message: error.to_string(),
        }
    }

    /// Returns the service associated with the cleanup failure, when applicable.
    #[must_use]
    pub fn service(&self) -> Option<&str> {
        self.service.as_deref()
    }

    /// Returns the stable failure classification.
    #[must_use]
    pub const fn kind(&self) -> RuntimeErrorKind {
        self.kind
    }

    /// Returns the diagnostic failure message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Errors produced while building or running `OneAgent Runtime`.
#[derive(Debug)]
pub enum RuntimeError {
    /// The application was built without required configuration.
    MissingConfiguration,
    /// The application lifecycle transition was invalid.
    InvalidLifecycleTransition {
        /// Current lifecycle state.
        from: &'static str,
        /// Requested lifecycle state.
        to: &'static str,
    },
    /// A service was registered with an empty name.
    InvalidServiceName,
    /// A service name was registered more than once.
    DuplicateServiceName {
        /// Duplicate stable service name.
        service: String,
    },
    /// A named service failed while acknowledging startup.
    ServiceStartFailed {
        /// Stable service name.
        service: String,
        /// Original startup error.
        source: BoxError,
        /// Secondary rollback failures.
        cleanup: Vec<CleanupFailure>,
    },
    /// A service returned successfully before cancellation was requested.
    UnexpectedServiceExit {
        /// Stable service name.
        service: String,
        /// Secondary cleanup failures.
        cleanup: Vec<CleanupFailure>,
    },
    /// A named service returned an execution error.
    ServiceFailed {
        /// Stable service name.
        service: String,
        /// Original service error.
        source: BoxError,
        /// Secondary cleanup failures.
        cleanup: Vec<CleanupFailure>,
    },
    /// A Runtime-owned service task panicked or was externally cancelled.
    ServiceTaskJoinFailed {
        /// Stable service name.
        service: String,
        /// Original join error.
        source: BoxError,
        /// Secondary cleanup failures.
        cleanup: Vec<CleanupFailure>,
    },
    /// The injected shutdown source returned an error.
    ShutdownSourceFailed {
        /// Original shutdown-source error.
        source: BoxError,
        /// Secondary cleanup failures.
        cleanup: Vec<CleanupFailure>,
    },
}

impl RuntimeError {
    /// Returns the stable in-process error classification.
    #[must_use]
    pub const fn kind(&self) -> RuntimeErrorKind {
        match self {
            Self::MissingConfiguration => RuntimeErrorKind::MissingConfiguration,
            Self::InvalidLifecycleTransition { .. } => RuntimeErrorKind::InvalidLifecycleTransition,
            Self::InvalidServiceName => RuntimeErrorKind::InvalidServiceName,
            Self::DuplicateServiceName { .. } => RuntimeErrorKind::DuplicateServiceName,
            Self::ServiceStartFailed { .. } => RuntimeErrorKind::ServiceStartFailed,
            Self::UnexpectedServiceExit { .. } => RuntimeErrorKind::UnexpectedServiceExit,
            Self::ServiceFailed { .. } => RuntimeErrorKind::ServiceFailed,
            Self::ServiceTaskJoinFailed { .. } => RuntimeErrorKind::ServiceTaskJoinFailed,
            Self::ShutdownSourceFailed { .. } => RuntimeErrorKind::ShutdownSourceFailed,
        }
    }

    /// Returns the stable service name associated with the error.
    #[must_use]
    pub fn service_name(&self) -> Option<&str> {
        match self {
            Self::DuplicateServiceName { service }
            | Self::ServiceStartFailed { service, .. }
            | Self::UnexpectedServiceExit { service, .. }
            | Self::ServiceFailed { service, .. }
            | Self::ServiceTaskJoinFailed { service, .. } => Some(service),
            Self::MissingConfiguration
            | Self::InvalidLifecycleTransition { .. }
            | Self::InvalidServiceName
            | Self::ShutdownSourceFailed { .. } => None,
        }
    }

    /// Returns secondary cleanup failures retained with the primary cause.
    #[must_use]
    pub fn cleanup_failures(&self) -> &[CleanupFailure] {
        match self {
            Self::ServiceStartFailed { cleanup, .. }
            | Self::UnexpectedServiceExit { cleanup, .. }
            | Self::ServiceFailed { cleanup, .. }
            | Self::ServiceTaskJoinFailed { cleanup, .. }
            | Self::ShutdownSourceFailed { cleanup, .. } => cleanup,
            Self::MissingConfiguration
            | Self::InvalidLifecycleTransition { .. }
            | Self::InvalidServiceName
            | Self::DuplicateServiceName { .. } => &[],
        }
    }

    pub(crate) fn with_cleanup(mut self, failures: Vec<CleanupFailure>) -> Self {
        match &mut self {
            Self::ServiceStartFailed { cleanup, .. }
            | Self::UnexpectedServiceExit { cleanup, .. }
            | Self::ServiceFailed { cleanup, .. }
            | Self::ServiceTaskJoinFailed { cleanup, .. }
            | Self::ShutdownSourceFailed { cleanup, .. } => cleanup.extend(failures),
            Self::MissingConfiguration
            | Self::InvalidLifecycleTransition { .. }
            | Self::InvalidServiceName
            | Self::DuplicateServiceName { .. } => {}
        }
        self
    }
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingConfiguration => formatter.write_str("runtime configuration is missing"),
            Self::InvalidLifecycleTransition { from, to } => {
                write!(formatter, "invalid lifecycle transition: {from} -> {to}")
            }
            Self::InvalidServiceName => formatter.write_str("runtime service name is empty"),
            Self::DuplicateServiceName { service } => {
                write!(
                    formatter,
                    "runtime service is already registered: {service}"
                )
            }
            Self::ServiceStartFailed {
                service, source, ..
            } => write!(
                formatter,
                "runtime service failed to start: {service}: {source}"
            ),
            Self::UnexpectedServiceExit { service, .. } => {
                write!(
                    formatter,
                    "runtime service exited before cancellation: {service}"
                )
            }
            Self::ServiceFailed {
                service, source, ..
            } => write!(formatter, "runtime service failed: {service}: {source}"),
            Self::ServiceTaskJoinFailed {
                service, source, ..
            } => write!(
                formatter,
                "runtime service task failed to join: {service}: {source}"
            ),
            Self::ShutdownSourceFailed { source, .. } => {
                write!(formatter, "runtime shutdown source failed: {source}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ServiceStartFailed { source, .. }
            | Self::ServiceFailed { source, .. }
            | Self::ServiceTaskJoinFailed { source, .. }
            | Self::ShutdownSourceFailed { source, .. } => Some(source.as_ref()),
            Self::MissingConfiguration
            | Self::InvalidLifecycleTransition { .. }
            | Self::InvalidServiceName
            | Self::DuplicateServiceName { .. }
            | Self::UnexpectedServiceExit { .. } => None,
        }
    }
}
