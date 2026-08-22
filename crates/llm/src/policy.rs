//! Provider execution policy values without execution or clock ownership.

use std::time::Duration;

use crate::{LlmError, LlmErrorKind};

/// Maximum represented provider timeout in seconds.
pub const MAX_PROVIDER_TIMEOUT_SECS: u64 = 300;
/// Maximum represented provider timeout.
pub const MAX_PROVIDER_TIMEOUT: Duration = Duration::from_secs(MAX_PROVIDER_TIMEOUT_SECS);

/// Closed Sprint 23 retry policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RetryPolicy {
    /// Invoke the provider exactly once and never replay a request.
    Never,
}

/// Provider execution policy passed to a future concrete adapter.
///
/// This value does not own a clock, timer, executor, or retry loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderExecutionPolicy {
    timeout: Option<Duration>,
    retry: RetryPolicy,
}

impl ProviderExecutionPolicy {
    /// Creates an execution policy with an optional represented total timeout.
    ///
    /// # Errors
    ///
    /// Returns an invalid-configuration failure when a present timeout is zero
    /// or exceeds 300 seconds.
    pub fn new(timeout: Option<Duration>) -> Result<Self, LlmError> {
        if let Some(timeout) = timeout {
            if timeout.is_zero() {
                return Err(LlmError::with_static_diagnostic(
                    LlmErrorKind::InvalidConfiguration,
                    "provider timeout is zero",
                ));
            }
            if timeout > MAX_PROVIDER_TIMEOUT {
                return Err(LlmError::with_static_diagnostic(
                    LlmErrorKind::InvalidConfiguration,
                    "provider timeout exceeds maximum",
                ));
            }
        }

        Ok(Self {
            timeout,
            retry: RetryPolicy::Never,
        })
    }

    /// Returns the represented total timeout, when present.
    #[must_use]
    pub const fn timeout(self) -> Option<Duration> {
        self.timeout
    }

    /// Returns the fixed Sprint 23 retry policy.
    #[must_use]
    pub const fn retry(self) -> RetryPolicy {
        self.retry
    }

    /// Returns the exact provider invocation limit.
    #[must_use]
    pub const fn max_attempts(self) -> usize {
        1
    }
}

impl Default for ProviderExecutionPolicy {
    fn default() -> Self {
        Self {
            timeout: None,
            retry: RetryPolicy::Never,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        MAX_PROVIDER_TIMEOUT, MAX_PROVIDER_TIMEOUT_SECS, ProviderExecutionPolicy, RetryPolicy,
    };
    use crate::LlmErrorKind;

    #[test]
    fn timeout_bounds_and_no_retry_policy_are_exact() {
        let default = ProviderExecutionPolicy::default();
        assert_eq!(default.timeout(), None);
        assert_eq!(default.retry(), RetryPolicy::Never);
        assert_eq!(default.max_attempts(), 1);

        let maximum = ProviderExecutionPolicy::new(Some(MAX_PROVIDER_TIMEOUT))
            .expect("maximum timeout must pass");
        assert_eq!(maximum.timeout(), Some(MAX_PROVIDER_TIMEOUT));

        let zero =
            ProviderExecutionPolicy::new(Some(Duration::ZERO)).expect_err("zero timeout must fail");
        assert_eq!(zero.kind(), LlmErrorKind::InvalidConfiguration);
        assert!(
            ProviderExecutionPolicy::new(Some(Duration::from_secs(MAX_PROVIDER_TIMEOUT_SECS + 1)))
                .is_err()
        );
    }
}
