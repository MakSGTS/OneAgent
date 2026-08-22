//! Validated provider-neutral text generation requests.

use std::fmt::{Debug, Formatter};

use crate::{
    LlmError, LlmErrorKind, MAX_TEXT_INPUT_BYTES, MAX_TEXT_OUTPUT_BYTES, ModelCapability,
    ModelDescriptor, ModelIdentity,
};

/// One owned text-generation request validated against a selected model.
#[derive(Clone, PartialEq, Eq)]
pub struct TextGenerationRequest {
    model: ModelIdentity,
    input: Box<str>,
    max_output_bytes: usize,
}

impl TextGenerationRequest {
    /// Creates a request for one provider-scoped model descriptor.
    ///
    /// Accepted input is preserved exactly. Validation rejects empty input,
    /// input and output bounds, then model incompatibility in that order.
    ///
    /// # Errors
    ///
    /// Returns [`LlmErrorKind::InvalidRequest`] for invalid text or bounds and
    /// [`LlmErrorKind::IncompatibleModel`] when the selected model does not
    /// advertise [`ModelCapability::TextGeneration`].
    pub fn new(
        model: &ModelDescriptor,
        input: impl Into<String>,
        max_output_bytes: usize,
    ) -> Result<Self, LlmError> {
        let input = input.into();
        if input.trim().is_empty() {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidRequest,
                "request input is empty",
            ));
        }
        if input.len() > MAX_TEXT_INPUT_BYTES {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidRequest,
                "request input exceeds byte limit",
            ));
        }
        if max_output_bytes == 0 {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidRequest,
                "request output bound is zero",
            ));
        }
        if max_output_bytes > MAX_TEXT_OUTPUT_BYTES {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::InvalidRequest,
                "request output bound exceeds byte limit",
            ));
        }
        if !model.supports(ModelCapability::TextGeneration) {
            return Err(LlmError::with_static_diagnostic(
                LlmErrorKind::IncompatibleModel,
                "selected model does not support text generation",
            ));
        }

        Ok(Self {
            model: model.identity().clone(),
            input: input.into_boxed_str(),
            max_output_bytes,
        })
    }

    /// Returns the selected provider-scoped model identity.
    #[must_use]
    pub const fn model(&self) -> &ModelIdentity {
        &self.model
    }

    /// Returns the explicitly accessed input text exactly as supplied.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Returns the required maximum response size in UTF-8 bytes.
    #[must_use]
    pub const fn max_output_bytes(&self) -> usize {
        self.max_output_bytes
    }
}

impl Debug for TextGenerationRequest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TextGenerationRequest")
            .field("model", &self.model)
            .field("input_bytes", &self.input.len())
            .field("max_output_bytes", &self.max_output_bytes)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::TextGenerationRequest;
    use crate::{
        LlmErrorKind, MAX_TEXT_INPUT_BYTES, MAX_TEXT_OUTPUT_BYTES, ModelCapability,
        ModelDescriptor, ModelId, ModelIdentity, ProviderDiagnostic, ProviderId,
    };

    fn descriptor(capabilities: impl IntoIterator<Item = ModelCapability>) -> ModelDescriptor {
        ModelDescriptor::new(
            ModelIdentity::new(
                ProviderId::new("provider").expect("provider ID must pass"),
                ModelId::new("model").expect("model ID must pass"),
            ),
            capabilities,
        )
    }

    fn text_model() -> ModelDescriptor {
        descriptor([ModelCapability::TextGeneration])
    }

    #[test]
    fn request_preserves_sensitive_input_and_debug_reports_only_bytes() {
        let sentinel = "  synthetic-request-sentinel\r\nПривет  ";
        let request = TextGenerationRequest::new(&text_model(), sentinel, 17)
            .expect("bounded text request must pass");

        assert_eq!(request.input(), sentinel);
        assert_eq!(request.max_output_bytes(), 17);
        assert_eq!(request.model().model().as_str(), "model");
        assert!(!format!("{request:?}").contains(sentinel));
    }

    #[test]
    fn input_and_output_bounds_are_exact_utf8_byte_limits() {
        let exact = "x".repeat(MAX_TEXT_INPUT_BYTES);
        assert!(TextGenerationRequest::new(&text_model(), exact, MAX_TEXT_OUTPUT_BYTES).is_ok());

        let unicode_exact = "я".repeat(MAX_TEXT_INPUT_BYTES / "я".len());
        assert_eq!(unicode_exact.len(), MAX_TEXT_INPUT_BYTES);
        assert!(TextGenerationRequest::new(&text_model(), unicode_exact, 1).is_ok());

        let over = "x".repeat(MAX_TEXT_INPUT_BYTES + 1);
        assert_eq!(
            TextGenerationRequest::new(&text_model(), over, 1)
                .expect_err("over-limit input must fail")
                .kind(),
            LlmErrorKind::InvalidRequest
        );
        assert_eq!(
            TextGenerationRequest::new(&text_model(), "x", MAX_TEXT_OUTPUT_BYTES + 1)
                .expect_err("over-limit output bound must fail")
                .kind(),
            LlmErrorKind::InvalidRequest
        );
    }

    #[test]
    fn validation_precedence_is_input_then_bounds_then_capability() {
        let incompatible = descriptor([]);
        let over_input = "x".repeat(MAX_TEXT_INPUT_BYTES + 1);

        let cases = [
            (" \n\t".to_owned(), 0, "request input is empty"),
            (over_input, 0, "request input exceeds byte limit"),
            ("x".to_owned(), 0, "request output bound is zero"),
            (
                "x".to_owned(),
                MAX_TEXT_OUTPUT_BYTES + 1,
                "request output bound exceeds byte limit",
            ),
        ];

        for (input, max_output_bytes, expected) in cases {
            let error = TextGenerationRequest::new(&incompatible, input, max_output_bytes)
                .expect_err("the earlier validation failure must win");
            assert_eq!(error.kind(), LlmErrorKind::InvalidRequest);
            assert_eq!(
                error.diagnostic().map(ProviderDiagnostic::as_str),
                Some(expected)
            );
        }

        let error = TextGenerationRequest::new(&incompatible, "x", 1)
            .expect_err("missing capability must fail after valid bounds");
        assert_eq!(error.kind(), LlmErrorKind::IncompatibleModel);
    }

    #[test]
    fn equivalent_capability_input_produces_equal_repeated_requests() {
        let first = descriptor([
            ModelCapability::TextGeneration,
            ModelCapability::TextGeneration,
        ]);
        let second = descriptor([ModelCapability::TextGeneration]);

        let left = TextGenerationRequest::new(&first, "same", 32).expect("request must pass");
        let right = TextGenerationRequest::new(&second, "same", 32).expect("request must pass");

        assert_eq!(left, right);
        assert_eq!(
            left,
            TextGenerationRequest::new(&first, "same", 32).expect("repeat must pass")
        );
    }
}
