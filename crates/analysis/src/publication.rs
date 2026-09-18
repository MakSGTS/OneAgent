//! Canonical process-local Workspace publication identity.

/// Process-local identity of one successful complete Workspace publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspacePublicationId(u64);

impl WorkspacePublicationId {
    /// Returns the first publication identity of a fresh service run.
    #[must_use]
    pub const fn initial() -> Self {
        Self(1)
    }

    /// Creates a non-zero process-local publication identity.
    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    /// Returns the numeric process-local identity.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) const fn checked_successor(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Self::new(value),
            None => None,
        }
    }
}
