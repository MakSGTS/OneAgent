use oneagent_llm::{LlmError, LlmErrorKind, ProviderConfiguration, ProviderDiagnostic, ProviderId};
use reqwest::{
    Client, Url,
    header::{AUTHORIZATION, HeaderValue},
    redirect::Policy,
};

use crate::{COMPLETIONS_PATH, MAX_BASE_URL_BYTES, MODELS_PATH, PROVIDER_ID, USER_AGENT};

/// A concrete client for the bounded OpenAI-compatible provider contract.
///
/// Construction is deterministic and performs no network I/O. The value owns
/// no implicit configuration source and intentionally exposes neither its
/// endpoint URLs nor its optional authorization header.
pub struct OpenAiCompatibleProvider {
    id: ProviderId,
    client: Client,
    models_url: Url,
    completions_url: Url,
    authorization: Option<HeaderValue>,
}

impl OpenAiCompatibleProvider {
    /// Constructs a provider from explicit provider-neutral configuration and
    /// one server-origin root URL.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`LlmErrorKind::InvalidConfiguration`] when the
    /// provider ID, URL, credential header, or HTTP client policy is invalid.
    pub fn new(configuration: ProviderConfiguration, base_url: &str) -> Result<Self, LlmError> {
        let (id, credential) = configuration.into_parts();
        if id.as_str() != PROVIDER_ID {
            return Err(configuration_error(
                "provider identifier is not openai-compatible",
            ));
        }

        let root = validate_base_url(base_url)?;
        let models_url = join_endpoint(&root, MODELS_PATH)?;
        let completions_url = join_endpoint(&root, COMPLETIONS_PATH)?;
        let authorization = credential
            .as_ref()
            .map(|secret| authorization_header(secret.expose()))
            .transpose()?;
        let client = Client::builder()
            .redirect(Policy::none())
            .no_proxy()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|_| configuration_error("HTTP client construction failed"))?;

        Ok(Self {
            id,
            client,
            models_url,
            completions_url,
            authorization,
        })
    }

    /// Returns the stable provider identity used by every adapter operation.
    #[must_use]
    pub const fn id(&self) -> &ProviderId {
        &self.id
    }

    pub(crate) const fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) const fn models_url(&self) -> &Url {
        &self.models_url
    }

    #[allow(dead_code)]
    pub(crate) const fn completions_url(&self) -> &Url {
        &self.completions_url
    }

    pub(crate) const fn authorization(&self) -> Option<&HeaderValue> {
        self.authorization.as_ref()
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

pub(crate) fn apply_authorization(
    request: reqwest::RequestBuilder,
    authorization: Option<&HeaderValue>,
) -> reqwest::RequestBuilder {
    match authorization {
        Some(value) => request.header(AUTHORIZATION, value.clone()),
        None => request,
    }
}

pub(crate) fn configuration_error(diagnostic: &'static str) -> LlmError {
    let diagnostic = ProviderDiagnostic::new(diagnostic)
        .expect("static adapter diagnostics must satisfy the shared bound");
    LlmError::new(LlmErrorKind::InvalidConfiguration).with_diagnostic(diagnostic)
}

#[cfg(test)]
mod tests {
    use oneagent_llm::{
        LlmErrorKind, ProviderConfiguration, ProviderDiagnostic, ProviderId, ProviderSecret,
    };

    use super::OpenAiCompatibleProvider;

    fn configuration(secret: Option<&str>) -> ProviderConfiguration {
        ProviderConfiguration::new(
            ProviderId::new("openai-compatible").expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
        )
    }

    fn construction_error(
        result: Result<OpenAiCompatibleProvider, oneagent_llm::LlmError>,
        context: &str,
    ) -> oneagent_llm::LlmError {
        match result {
            Ok(_) => panic!("{context}"),
            Err(error) => error,
        }
    }

    #[test]
    fn construction_normalizes_root_and_builds_exact_endpoints_without_io() {
        for base in ["http://127.0.0.1:8080", "https://example.invalid/"] {
            let provider = OpenAiCompatibleProvider::new(configuration(None), base)
                .expect("explicit root must pass");

            assert_eq!(provider.id().as_str(), "openai-compatible");
            assert_eq!(provider.models_url().path(), "/v1/models");
            assert_eq!(provider.completions_url().path(), "/v1/completions");
            assert_eq!(
                provider.models_url().scheme(),
                provider.completions_url().scheme()
            );
            assert_eq!(
                provider.models_url().host_str(),
                provider.completions_url().host_str()
            );
            let _ = provider.client();
        }
    }

    #[test]
    fn construction_rejects_every_disallowed_url_component_without_echoing_it() {
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
                OpenAiCompatibleProvider::new(configuration(None), value),
                "invalid URL must fail",
            );
            assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
            assert!(!format!("{error}").contains(sentinel));
            assert!(!format!("{error:?}").contains(sentinel));
            assert!(
                !error
                    .diagnostic()
                    .map(ProviderDiagnostic::as_str)
                    .unwrap_or_default()
                    .contains(sentinel)
            );
        }
    }

    #[test]
    fn construction_enforces_url_byte_bound_and_provider_identity() {
        let over_limit = format!("http://{}.invalid", "x".repeat(2_048));
        assert!(OpenAiCompatibleProvider::new(configuration(None), &over_limit).is_err());

        let wrong = ProviderConfiguration::new(
            ProviderId::new("other").expect("provider ID must pass"),
            None,
        );
        let error = construction_error(
            OpenAiCompatibleProvider::new(wrong, "http://example.invalid"),
            "provider mismatch must fail",
        );
        assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
    }

    #[test]
    fn optional_bearer_header_is_sensitive_and_invalid_content_is_redacted() {
        let sentinel = "synthetic-secret-sentinel";
        let provider =
            OpenAiCompatibleProvider::new(configuration(Some(sentinel)), "http://example.invalid")
                .expect("valid bearer configuration must pass");
        let header = provider.authorization().expect("header must be present");
        assert!(header.is_sensitive());
        assert_eq!(
            header.to_str().expect("header must be text"),
            format!("Bearer {sentinel}")
        );

        let invalid = construction_error(
            OpenAiCompatibleProvider::new(
                configuration(Some("synthetic-secret-sentinel\ninvalid")),
                "http://example.invalid",
            ),
            "invalid bearer header must fail",
        );
        assert_eq!(invalid.kind(), LlmErrorKind::InvalidConfiguration);
        assert!(!format!("{invalid}").contains(sentinel));
        assert!(!format!("{invalid:?}").contains(sentinel));
        assert!(
            !invalid
                .diagnostic()
                .map(ProviderDiagnostic::as_str)
                .unwrap_or_default()
                .contains(sentinel)
        );

        let absent = OpenAiCompatibleProvider::new(configuration(None), "http://example.invalid")
            .expect("credential-free configuration must pass");
        assert!(absent.authorization().is_none());
    }
}
