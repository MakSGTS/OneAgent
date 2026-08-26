use std::future::Future;

use oneagent_llm::{LlmError, LlmErrorKind, ProviderDiagnostic, ProviderExecutionContext};
use reqwest::{Response, StatusCode};

pub(crate) async fn run_with_context<T>(
    context: ProviderExecutionContext<'_>,
    operation: impl Future<Output = Result<T, LlmError>>,
) -> Result<T, LlmError> {
    tokio::pin!(operation);

    if let Some(timeout) = context.policy().timeout() {
        let timer = tokio::time::sleep(timeout);
        tokio::pin!(timer);
        tokio::select! {
            biased;
            () = context.cancellation().cancelled() => {
                Err(adapter_error(LlmErrorKind::Cancelled, "provider operation was cancelled"))
            }
            result = &mut operation => result,
            () = &mut timer => {
                Err(adapter_error(LlmErrorKind::Timeout, "provider operation timed out"))
            }
        }
    } else {
        tokio::select! {
            biased;
            () = context.cancellation().cancelled() => {
                Err(adapter_error(LlmErrorKind::Cancelled, "provider operation was cancelled"))
            }
            result = &mut operation => result,
        }
    }
}

pub(crate) async fn bounded_success_body(
    mut response: Response,
    maximum: usize,
) -> Result<Vec<u8>, LlmError> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum as u64)
    {
        return Err(adapter_error(
            LlmErrorKind::Protocol,
            "provider response body exceeds byte limit",
        ));
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| {
        adapter_error(
            LlmErrorKind::Transport,
            "provider response body transport failed",
        )
    })? {
        let next = body.len().checked_add(chunk.len()).ok_or_else(|| {
            adapter_error(
                LlmErrorKind::Protocol,
                "provider response body size overflowed",
            )
        })?;
        if next > maximum {
            return Err(adapter_error(
                LlmErrorKind::Protocol,
                "provider response body exceeds byte limit",
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub(crate) fn status_error(status: StatusCode) -> LlmError {
    let kind = if status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
    {
        LlmErrorKind::ProviderUnavailable
    } else {
        LlmErrorKind::ProviderRejected
    };
    let diagnostic = ProviderDiagnostic::new(format!("provider returned HTTP {}", status.as_u16()))
        .expect("HTTP status diagnostic must satisfy the shared bound");
    LlmError::new(kind).with_diagnostic(diagnostic)
}

pub(crate) fn adapter_error(kind: LlmErrorKind, diagnostic: &'static str) -> LlmError {
    let diagnostic = ProviderDiagnostic::new(diagnostic)
        .expect("static adapter diagnostics must satisfy the shared bound");
    LlmError::new(kind).with_diagnostic(diagnostic)
}

#[cfg(test)]
mod tests {
    use oneagent_llm::{
        CancellationSignal, LlmErrorKind, ProviderExecutionContext, ProviderExecutionPolicy,
        ProviderFuture,
    };

    use super::run_with_context;

    struct ReadyCancellation;

    impl CancellationSignal for ReadyCancellation {
        fn is_cancelled(&self) -> bool {
            false
        }

        fn cancelled(&self) -> ProviderFuture<'_, ()> {
            Box::pin(std::future::ready(()))
        }
    }

    #[tokio::test]
    async fn simultaneous_ready_cancellation_precedes_operation() {
        let policy = ProviderExecutionPolicy::default();
        let cancellation = ReadyCancellation;
        let context = ProviderExecutionContext::new(&policy, &cancellation);
        let result = run_with_context(context, async { Ok::<_, oneagent_llm::LlmError>(()) }).await;

        assert_eq!(
            result.expect_err("cancellation must win").kind(),
            LlmErrorKind::Cancelled
        );
    }
}
