//! Provider-neutral terminal text response domain.

use std::fmt::{Debug, Formatter};

use crate::{LlmError, LlmErrorKind, ModelIdentity, TextGenerationRequest};

/// Maximum accepted request input in UTF-8 bytes.
pub const MAX_TEXT_INPUT_BYTES: usize = 65_536;
/// Maximum accepted requested and returned output in UTF-8 bytes.
pub const MAX_TEXT_OUTPUT_BYTES: usize = 65_536;

/// Closed provider-neutral terminal reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FinishReason {
    /// The provider mapped a normal terminal completion.
    Completed,
    /// The provider mapped its bounded-output terminal condition.
    OutputLimit,
}

/// Exact local UTF-8 byte accounting for one text request and response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextUsage {
    input: usize,
    output: usize,
    total: usize,
}

impl TextUsage {
    fn new_checked(input_bytes: usize, output_bytes: usize) -> Result<Self, LlmError> {
        let total_bytes = input_bytes.checked_add(output_bytes).ok_or_else(|| {
            LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidResponse,
                "response byte accounting overflowed",
            )
        })?;
        Ok(Self {
            input: input_bytes,
            output: output_bytes,
            total: total_bytes,
        })
    }

    /// Returns exact request input UTF-8 bytes.
    #[must_use]
    pub const fn input_bytes(self) -> usize {
        self.input
    }

    /// Returns exact response output UTF-8 bytes.
    #[must_use]
    pub const fn output_bytes(self) -> usize {
        self.output
    }

    /// Returns exact combined local UTF-8 bytes.
    #[must_use]
    pub const fn total_bytes(self) -> usize {
        self.total
    }
}

/// One complete provider-neutral terminal text response.
#[derive(Clone, PartialEq, Eq)]
pub struct TextGenerationResponse {
    model: ModelIdentity,
    output: Box<str>,
    usage: TextUsage,
    finish: FinishReason,
}

impl TextGenerationResponse {
    /// Creates a terminal response against its originating validated request.
    ///
    /// # Errors
    ///
    /// Returns [`LlmErrorKind::InvalidResponse`] when output is empty or exceeds
    /// the request's explicit UTF-8 byte bound.
    pub fn new(
        request: &TextGenerationRequest,
        output: impl Into<String>,
        finish: FinishReason,
    ) -> Result<Self, LlmError> {
        Self::new_checked(
            request.model().clone(),
            request.input().len(),
            request.max_output_bytes(),
            output,
            finish,
        )
    }

    pub(crate) fn new_checked(
        model: ModelIdentity,
        input_bytes: usize,
        max_output_bytes: usize,
        output: impl Into<String>,
        finish: FinishReason,
    ) -> Result<Self, LlmError> {
        if input_bytes > MAX_TEXT_INPUT_BYTES {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidResponse,
                "response input accounting exceeds byte limit",
            ));
        }
        if !(1..=MAX_TEXT_OUTPUT_BYTES).contains(&max_output_bytes) {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidResponse,
                "response output bound is invalid",
            ));
        }

        let output = output.into();
        if output.is_empty() {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidResponse,
                "provider response output is empty",
            ));
        }
        if output.len() > max_output_bytes {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidResponse,
                "provider response output exceeds request bound",
            ));
        }

        let usage = TextUsage::new_checked(input_bytes, output.len())?;

        Ok(Self {
            model,
            output: output.into_boxed_str(),
            usage,
            finish,
        })
    }

    /// Returns the exact provider-scoped request model.
    #[must_use]
    pub const fn model(&self) -> &ModelIdentity {
        &self.model
    }

    /// Returns the explicitly accessed terminal output text.
    #[must_use]
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Returns exact local UTF-8 byte accounting.
    #[must_use]
    pub const fn usage(&self) -> TextUsage {
        self.usage
    }

    /// Returns the mapped provider-neutral terminal reason.
    #[must_use]
    pub const fn finish(&self) -> FinishReason {
        self.finish
    }
}

impl Debug for TextGenerationResponse {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TextGenerationResponse")
            .field("model", &self.model)
            .field("output_bytes", &self.usage.output_bytes())
            .field("usage", &self.usage)
            .field("finish", &self.finish)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FinishReason, MAX_TEXT_INPUT_BYTES, MAX_TEXT_OUTPUT_BYTES, TextGenerationResponse,
    };
    use crate::{
        LlmErrorKind, ModelCapability, ModelDescriptor, ModelId, ModelIdentity, ProviderId,
        TextGenerationRequest,
    };

    fn model() -> ModelIdentity {
        ModelIdentity::new(
            ProviderId::new("provider").expect("provider ID must pass"),
            ModelId::new("model").expect("model ID must pass"),
        )
    }

    fn request(max_output_bytes: usize) -> TextGenerationRequest {
        let descriptor = ModelDescriptor::new(model(), [ModelCapability::TextGeneration]);
        TextGenerationRequest::new(&descriptor, "вход", max_output_bytes)
            .expect("request must pass")
    }

    #[test]
    fn public_response_is_bound_to_the_originating_request() {
        let request = request("ответ".len());
        let response = TextGenerationResponse::new(&request, "ответ", FinishReason::Completed)
            .expect("response at the request bound must pass");

        assert_eq!(response.model(), request.model());
        assert_eq!(response.output(), "ответ");
        assert_eq!(response.usage().input_bytes(), request.input().len());
        assert_eq!(response.usage().output_bytes(), "ответ".len());

        let error = TextGenerationResponse::new(&request, "ответ!", FinishReason::Completed)
            .expect_err("response over the request bound must fail");
        assert_eq!(error.kind(), LlmErrorKind::InvalidResponse);
    }

    #[test]
    fn checked_response_preserves_output_and_exact_byte_usage() {
        let response = TextGenerationResponse::new_checked(
            model(),
            "вход".len(),
            MAX_TEXT_OUTPUT_BYTES,
            "ответ",
            FinishReason::Completed,
        )
        .expect("response must pass");

        assert_eq!(response.output(), "ответ");
        assert_eq!(response.usage().input_bytes(), "вход".len());
        assert_eq!(response.usage().output_bytes(), "ответ".len());
        assert_eq!(response.finish(), FinishReason::Completed);
    }

    #[test]
    fn response_bounds_are_atomic_and_debug_omits_sensitive_output() {
        let sentinel = "synthetic-response-sentinel";
        let response = TextGenerationResponse::new_checked(
            model(),
            MAX_TEXT_INPUT_BYTES,
            sentinel.len(),
            sentinel,
            FinishReason::OutputLimit,
        )
        .expect("exact response bound must pass");
        assert!(!format!("{response:?}").contains(sentinel));

        let empty = TextGenerationResponse::new_checked(model(), 1, 1, "", FinishReason::Completed)
            .expect_err("empty response must fail");
        assert_eq!(empty.kind(), LlmErrorKind::InvalidResponse);

        assert!(
            TextGenerationResponse::new_checked(model(), 1, 1, "xx", FinishReason::Completed)
                .is_err()
        );
        assert!(
            TextGenerationResponse::new_checked(
                model(),
                MAX_TEXT_INPUT_BYTES + 1,
                1,
                "x",
                FinishReason::Completed
            )
            .is_err()
        );
    }
}
