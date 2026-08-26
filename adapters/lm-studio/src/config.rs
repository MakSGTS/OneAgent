use oneagent_llm::{LlmError, LlmErrorKind, ProviderConfiguration, ProviderDiagnostic, ProviderId};
use oneagent_openai_compatible::OpenAiCompatibleProvider;
use reqwest::{Client, Url, header::HeaderValue, redirect::Policy};

use crate::{
    LOCAL_BASE_URL, MAX_BASE_URL_BYTES, NATIVE_MODELS_PATH, OPENAI_COMPATIBLE_PROVIDER_ID,
    PROVIDER_ID, USER_AGENT,
};

/// A concrete client foundation for the bounded LM Studio provider contract.
///
/// Construction is deterministic and performs no network I/O. Native model
/// discovery and composed text generation are introduced by later Sprint 25
/// tasks.
#[allow(
    dead_code,
    reason = "Task 3 retains transport and composition state consumed by Tasks 4 and 5"
)]
pub struct LmStudioProvider {
    id: ProviderId,
    native_client: Client,
    native_models_url: Url,
    native_authorization: Option<HeaderValue>,
    generation_provider: OpenAiCompatibleProvider,
}

impl LmStudioProvider {
    /// Constructs a provider from explicit provider-neutral configuration and
    /// one LM Studio server-origin root URL.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`LlmErrorKind::InvalidConfiguration`] when the
    /// provider ID, URL, credential header, or HTTP client policy is invalid.
    /// An impossible static internal provider-identity failure returns
    /// [`LlmErrorKind::Internal`].
    pub fn new(configuration: ProviderConfiguration, base_url: &str) -> Result<Self, LlmError> {
        let (id, credential) = configuration.into_parts();
        if id.as_str() != PROVIDER_ID {
            return Err(configuration_error("provider identifier is not lm-studio"));
        }

        let root = validate_base_url(base_url)?;
        let native_models_url = join_endpoint(&root, NATIVE_MODELS_PATH)?;
        let native_authorization = credential
            .as_ref()
            .map(|secret| authorization_header(secret.expose()))
            .transpose()?;
        let native_client = Client::builder()
            .redirect(Policy::none())
            .no_proxy()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|_| configuration_error("HTTP client construction failed"))?;

        let generation_id = ProviderId::new(OPENAI_COMPATIBLE_PROVIDER_ID)
            .map_err(|_| internal_error("internal provider identifier construction failed"))?;
        let generation_configuration = ProviderConfiguration::new(generation_id, credential);
        let generation_provider =
            OpenAiCompatibleProvider::new(generation_configuration, base_url)?;

        Ok(Self {
            id,
            native_client,
            native_models_url,
            native_authorization,
            generation_provider,
        })
    }

    /// Constructs a provider for the deterministic numeric-loopback LM Studio
    /// root `http://127.0.0.1:1234`.
    ///
    /// # Errors
    ///
    /// Returns the same redacted construction failures as [`Self::new`].
    pub fn new_local(configuration: ProviderConfiguration) -> Result<Self, LlmError> {
        Self::new(configuration, LOCAL_BASE_URL)
    }

    /// Returns the stable provider identity used by the LM Studio adapter.
    #[must_use]
    pub const fn id(&self) -> &ProviderId {
        &self.id
    }
}

fn validate_base_url(value: &str) -> Result<Url, LlmError> {
    if value.is_empty() {
        return Err(configuration_error("base URL is empty"));
    }
    if value.len() > MAX_BASE_URL_BYTES {
        return Err(configuration_error("base URL exceeds byte limit"));
    }
    if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
        return Err(configuration_error("base URL has boundary whitespace"));
    }
    if value.chars().any(char::is_control) {
        return Err(configuration_error("base URL contains a control character"));
    }

    let (_, authority_and_path) = value
        .split_once("://")
        .ok_or_else(|| configuration_error("base URL is invalid"))?;
    if authority_and_path.starts_with(['/', '\\']) {
        return Err(configuration_error("base URL has no host"));
    }

    let mut url = Url::parse(value).map_err(|_| configuration_error("base URL is invalid"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(configuration_error("base URL scheme is unsupported"));
    }
    if !url.has_host() {
        return Err(configuration_error("base URL has no host"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(configuration_error("base URL contains user information"));
    }
    if url.query().is_some() {
        return Err(configuration_error("base URL contains a query"));
    }
    if url.fragment().is_some() {
        return Err(configuration_error("base URL contains a fragment"));
    }
    if !matches!(url.path(), "" | "/") {
        return Err(configuration_error("base URL path is not the server root"));
    }

    url.set_path("/");
    Ok(url)
}

fn join_endpoint(root: &Url, path: &'static str) -> Result<Url, LlmError> {
    let endpoint = root
        .join(path)
        .map_err(|_| configuration_error("provider endpoint construction failed"))?;
    if endpoint.scheme() != root.scheme()
        || endpoint.host_str() != root.host_str()
        || endpoint.port_or_known_default() != root.port_or_known_default()
    {
        return Err(configuration_error(
            "provider endpoint changed server origin",
        ));
    }
    Ok(endpoint)
}

fn authorization_header(secret: &str) -> Result<HeaderValue, LlmError> {
    let value = format!("Bearer {secret}");
    let mut header = HeaderValue::from_str(&value)
        .map_err(|_| configuration_error("provider credential is not a valid bearer header"))?;
    header.set_sensitive(true);
    Ok(header)
}

fn configuration_error(diagnostic: &'static str) -> LlmError {
    adapter_error(LlmErrorKind::InvalidConfiguration, diagnostic)
}

fn internal_error(diagnostic: &'static str) -> LlmError {
    adapter_error(LlmErrorKind::Internal, diagnostic)
}

fn adapter_error(kind: LlmErrorKind, diagnostic: &'static str) -> LlmError {
    let diagnostic = ProviderDiagnostic::new(diagnostic)
        .expect("static adapter diagnostics must satisfy the shared bound");
    LlmError::new(kind).with_diagnostic(diagnostic)
}

#[cfg(test)]
mod tests {
    use oneagent_llm::{
        LlmError, LlmErrorKind, ProviderConfiguration, ProviderDiagnostic, ProviderId,
        ProviderSecret,
    };

    use super::LmStudioProvider;

    fn configuration(secret: Option<&str>) -> ProviderConfiguration {
        ProviderConfiguration::new(
            ProviderId::new("lm-studio").expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
        )
    }

    fn construction_error(result: Result<LmStudioProvider, LlmError>, context: &str) -> LlmError {
        match result {
            Ok(_) => panic!("{context}"),
            Err(error) => error,
        }
    }

    #[test]
    fn construction_builds_exact_native_and_composed_foundation_without_io() {
        for base in [
            "http://127.0.0.1:1",
            "https://example.invalid/",
            "http://[::1]:1234",
        ] {
            let provider = LmStudioProvider::new(configuration(None), base)
                .expect("explicit root must construct without I/O");

            assert_eq!(provider.id().as_str(), "lm-studio");
            assert_eq!(provider.native_models_url.path(), "/api/v1/models");
            assert_eq!(
                provider.generation_provider.id().as_str(),
                "openai-compatible"
            );
            assert_eq!(
                provider.native_models_url.scheme(),
                if base.starts_with("https") {
                    "https"
                } else {
                    "http"
                }
            );
            assert!(provider.native_authorization.is_none());
            let _ = &provider.native_client;
        }
    }

    #[test]
    fn local_construction_uses_only_numeric_loopback_default() {
        let provider = LmStudioProvider::new_local(configuration(None))
            .expect("local default must construct without I/O");

        assert_eq!(provider.native_models_url.scheme(), "http");
        assert_eq!(provider.native_models_url.host_str(), Some("127.0.0.1"));
        assert_eq!(provider.native_models_url.port(), Some(1234));
        assert_eq!(provider.native_models_url.path(), "/api/v1/models");
    }

    #[test]
    fn construction_rejects_disallowed_url_components_without_echoing_them() {
        let sentinel = "synthetic-url-sentinel";
        let invalid = [
            "",
            " ftp://example.invalid",
            "ftp://example.invalid",
            "http:///missing-host",
            "http://user:password@example.invalid",
            "http://example.invalid/v1",
            "http://example.invalid/?query=synthetic-url-sentinel",
            "http://example.invalid/#synthetic-url-sentinel",
            "http://example.invalid/\n",
        ];

        for value in invalid {
            let error = construction_error(
                LmStudioProvider::new(configuration(None), value),
                "invalid URL must fail",
            );
            assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
            assert!(!format!("{error}").contains(sentinel));
            assert!(!format!("{error:?}").contains(sentinel));
            assert!(
                !error
                    .diagnostic()
                    .map(ProviderDiagnostic::as_str)
                    .is_some_and(|value| value.contains(sentinel))
            );
        }

        let over_limit = format!("http://{}.invalid", "a".repeat(2_048));
        let error = construction_error(
            LmStudioProvider::new(configuration(None), &over_limit),
            "over-limit URL must fail",
        );
        assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
        assert!(!format!("{error:?}").contains(&over_limit));
    }

    #[test]
    fn provider_mismatch_precedes_url_and_client_construction() {
        let configuration = ProviderConfiguration::new(
            ProviderId::new("other-provider").expect("provider ID must pass"),
            None,
        );
        let error = construction_error(
            LmStudioProvider::new(configuration, "not a URL"),
            "provider mismatch must fail",
        );

        assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
        assert_eq!(
            error.diagnostic().map(ProviderDiagnostic::as_str),
            Some("provider identifier is not lm-studio")
        );
    }

    #[test]
    fn optional_bearer_header_is_sensitive_and_invalid_content_is_redacted() {
        let sentinel = "synthetic-secret-sentinel";
        let provider =
            LmStudioProvider::new(configuration(Some(sentinel)), "http://example.invalid")
                .expect("synthetic bearer must construct");
        let header = provider
            .native_authorization
            .as_ref()
            .expect("authorization must be present");

        assert_eq!(
            header.to_str().expect("header must be visible explicitly"),
            format!("Bearer {sentinel}")
        );
        assert!(header.is_sensitive());
        assert!(!format!("{header:?}").contains(sentinel));

        let invalid_secret = format!("{sentinel}\ninvalid");
        let error = construction_error(
            LmStudioProvider::new(
                configuration(Some(&invalid_secret)),
                "http://example.invalid",
            ),
            "invalid bearer must fail",
        );
        assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
        assert!(!format!("{error}").contains(sentinel));
        assert!(!format!("{error:?}").contains(sentinel));
        assert!(
            !error
                .diagnostic()
                .map(ProviderDiagnostic::as_str)
                .is_some_and(|value| value.contains(sentinel))
        );
    }

    #[test]
    fn provider_foundation_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<LmStudioProvider>();
    }
}
