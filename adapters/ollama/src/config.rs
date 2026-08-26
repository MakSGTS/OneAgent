use oneagent_llm::{LlmError, LlmErrorKind, ProviderConfiguration, ProviderDiagnostic, ProviderId};
use reqwest::{Client, Url, redirect::Policy};

use crate::{
    GENERATE_PATH, LOCAL_BASE_URL, MAX_BASE_URL_BYTES, PROVIDER_ID, SHOW_PATH, TAGS_PATH,
    USER_AGENT,
};

/// A concrete client foundation for the bounded local Ollama contract.
///
/// Construction is deterministic and performs no network I/O. Model discovery
/// and text generation execute only through `LlmProvider`.
pub struct OllamaProvider {
    id: ProviderId,
    client: Client,
    tags_url: Url,
    show_url: Url,
    generate_url: Url,
}

impl OllamaProvider {
    /// Constructs a provider from explicit provider-neutral configuration and
    /// one numeric-loopback Ollama server-origin root URL.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`LlmErrorKind::InvalidConfiguration`] when the
    /// provider ID, credential policy, URL, endpoint, or HTTP client policy is
    /// invalid.
    pub fn new(configuration: ProviderConfiguration, base_url: &str) -> Result<Self, LlmError> {
        let (id, credential) = configuration.into_parts();
        if id.as_str() != PROVIDER_ID {
            return Err(configuration_error("provider identifier is not ollama"));
        }
        if credential.is_some() {
            return Err(configuration_error(
                "provider credential is unsupported for local ollama",
            ));
        }

        let root = validate_base_url(base_url)?;
        let tags_url = join_endpoint(&root, TAGS_PATH)?;
        let show_url = join_endpoint(&root, SHOW_PATH)?;
        let generate_url = join_endpoint(&root, GENERATE_PATH)?;
        let client = Client::builder()
            .redirect(Policy::none())
            .no_proxy()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|_| configuration_error("HTTP client construction failed"))?;

        Ok(Self {
            id,
            client,
            tags_url,
            show_url,
            generate_url,
        })
    }

    /// Constructs a provider for the deterministic numeric-loopback Ollama
    /// root `http://127.0.0.1:11434`.
    ///
    /// # Errors
    ///
    /// Returns the same redacted construction failures as [`Self::new`].
    pub fn new_local(configuration: ProviderConfiguration) -> Result<Self, LlmError> {
        Self::new(configuration, LOCAL_BASE_URL)
    }

    /// Returns the stable provider identity used by the Ollama adapter.
    #[must_use]
    pub const fn id(&self) -> &ProviderId {
        &self.id
    }

    pub(crate) const fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) const fn tags_url(&self) -> &Url {
        &self.tags_url
    }

    pub(crate) const fn show_url(&self) -> &Url {
        &self.show_url
    }

    pub(crate) const fn generate_url(&self) -> &Url {
        &self.generate_url
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
    let authority_and_path = value
        .strip_prefix("http://")
        .ok_or_else(|| configuration_error("base URL scheme is unsupported"))?;

    let mut url = Url::parse(value).map_err(|_| configuration_error("base URL is invalid"))?;
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

    let authority = authority_and_path
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let valid_authority = authority == "127.0.0.1"
        || authority
            .strip_prefix("127.0.0.1:")
            .is_some_and(|port| !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()));
    if !valid_authority || url.host_str() != Some("127.0.0.1") {
        return Err(configuration_error("base URL host is not numeric loopback"));
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

fn configuration_error(diagnostic: &'static str) -> LlmError {
    let diagnostic = ProviderDiagnostic::new(diagnostic)
        .expect("static adapter diagnostics must satisfy the shared bound");
    LlmError::new(LlmErrorKind::InvalidConfiguration).with_diagnostic(diagnostic)
}

#[cfg(test)]
mod tests {
    use oneagent_llm::{
        LlmError, LlmErrorKind, ProviderConfiguration, ProviderDiagnostic, ProviderId,
        ProviderSecret,
    };

    use super::OllamaProvider;

    fn configuration(provider: &str, secret: Option<&str>) -> ProviderConfiguration {
        ProviderConfiguration::new(
            ProviderId::new(provider).expect("provider ID must pass"),
            secret.map(|value| ProviderSecret::new(value).expect("secret must pass")),
        )
    }

    fn construction_error(result: Result<OllamaProvider, LlmError>, context: &str) -> LlmError {
        match result {
            Ok(_) => panic!("{context}"),
            Err(error) => error,
        }
    }

    #[test]
    fn construction_builds_exact_private_endpoints_without_io() {
        for base in ["http://127.0.0.1", "http://127.0.0.1:1/"] {
            let provider = OllamaProvider::new(configuration("ollama", None), base)
                .expect("numeric-loopback root must construct without I/O");

            assert_eq!(provider.id().as_str(), "ollama");
            assert_eq!(provider.tags_url.path(), "/api/tags");
            assert_eq!(provider.show_url.path(), "/api/show");
            assert_eq!(provider.generate_url.path(), "/api/generate");
            assert_eq!(provider.tags_url.scheme(), "http");
            assert_eq!(provider.tags_url.host_str(), Some("127.0.0.1"));
            let _ = &provider.client;
        }
    }

    #[test]
    fn local_construction_uses_the_exact_numeric_loopback_default() {
        let provider = OllamaProvider::new_local(configuration("ollama", None))
            .expect("local default must construct without I/O");

        assert_eq!(provider.tags_url.scheme(), "http");
        assert_eq!(provider.tags_url.host_str(), Some("127.0.0.1"));
        assert_eq!(provider.tags_url.port(), Some(11_434));
        assert_eq!(provider.tags_url.path(), "/api/tags");
    }

    #[test]
    fn provider_and_secret_precedence_is_deterministic_and_redacted() {
        let sentinel = "synthetic-secret-sentinel";
        let mismatch = construction_error(
            OllamaProvider::new(configuration("other", Some(sentinel)), "not a URL"),
            "provider mismatch must fail",
        );
        assert_eq!(mismatch.kind(), LlmErrorKind::InvalidConfiguration);
        assert_eq!(
            mismatch.diagnostic().map(ProviderDiagnostic::as_str),
            Some("provider identifier is not ollama")
        );

        let credential = construction_error(
            OllamaProvider::new(configuration("ollama", Some(sentinel)), "not a URL"),
            "credential presence must fail",
        );
        assert_eq!(credential.kind(), LlmErrorKind::InvalidConfiguration);
        assert_eq!(
            credential.diagnostic().map(ProviderDiagnostic::as_str),
            Some("provider credential is unsupported for local ollama")
        );
        for rendered in [format!("{credential}"), format!("{credential:?}")] {
            assert!(!rendered.contains(sentinel));
        }
    }

    #[test]
    fn construction_rejects_every_nonlocal_or_ambiguous_root_without_echoing_it() {
        let sentinel = "synthetic-url-sentinel";
        let invalid = [
            "",
            " http://127.0.0.1",
            "http://127.0.0.1 ",
            "HTTP://127.0.0.1",
            "https://127.0.0.1",
            "http://localhost:11434",
            "http://0.0.0.0:11434",
            "http://127.1:11434",
            "http://[::1]:11434",
            "http://user:password@127.0.0.1:11434",
            "http://127.0.0.1:11434/api",
            "http://127.0.0.1:11434/?query=synthetic-url-sentinel",
            "http://127.0.0.1:11434/#synthetic-url-sentinel",
            "http://127.0.0.1:11434/\n",
            "http://synthetic-url-sentinel.invalid:11434",
        ];

        for value in invalid {
            let error = construction_error(
                OllamaProvider::new(configuration("ollama", None), value),
                "invalid root must fail",
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

        let over_limit = format!("http://127.0.0.1/{}", "x".repeat(2_048));
        let error = construction_error(
            OllamaProvider::new(configuration("ollama", None), &over_limit),
            "over-limit root must fail",
        );
        assert_eq!(error.kind(), LlmErrorKind::InvalidConfiguration);
        assert!(!format!("{error:?}").contains(&over_limit));
    }

    #[test]
    fn provider_foundation_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<OllamaProvider>();
    }
}
