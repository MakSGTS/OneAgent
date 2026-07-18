//! Runtime error types.

use std::fmt::{Display, Formatter};

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
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingConfiguration => formatter.write_str("runtime configuration is missing"),
            Self::InvalidLifecycleTransition { from, to } => {
                write!(formatter, "invalid lifecycle transition: {from} -> {to}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
