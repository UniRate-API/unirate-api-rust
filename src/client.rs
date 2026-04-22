//! HTTP client and builder.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};

use crate::VERSION;

/// Default request timeout (30 seconds) — matches the other UniRate clients.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Async client for the UniRate API.
///
/// Construct via [`Client::new`] for the common case, or [`Client::builder`]
/// when you need to customize the timeout, base URL, or underlying
/// [`reqwest::Client`] (the customization path used by the mock test suite).
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) http: reqwest::Client,
}

impl Client {
    /// Create a new client with default settings.
    ///
    /// Points at `https://api.unirateapi.com` with a 30-second timeout.
    pub fn new(api_key: impl Into<String>) -> Self {
        ClientBuilder::new(api_key)
            .build()
            .expect("default reqwest::Client should always build")
    }

    /// Begin a fluent [`ClientBuilder`].
    pub fn builder(api_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(api_key)
    }

    /// Borrow the API key currently in use.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Borrow the base URL currently in use.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Builder for [`Client`] — set a custom timeout, base URL, or HTTP client.
#[derive(Debug)]
pub struct ClientBuilder {
    api_key: String,
    base_url: String,
    timeout: Duration,
    http: Option<reqwest::Client>,
}

impl ClientBuilder {
    /// Start a new builder with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: crate::DEFAULT_BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
            http: None,
        }
    }

    /// Override the base URL (useful for tests pointing at a `wiremock::MockServer`).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Override the request timeout. Ignored when a fully-configured
    /// [`reqwest::Client`] is supplied via [`ClientBuilder::http_client`].
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Supply a pre-configured [`reqwest::Client`]. When set, [`ClientBuilder::timeout`]
    /// is ignored — configure the timeout on the supplied client instead.
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    /// Finalize the builder and produce a [`Client`].
    pub fn build(self) -> Result<Client, reqwest::Error> {
        let http = match self.http {
            Some(h) => h,
            None => {
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                headers.insert(
                    USER_AGENT,
                    HeaderValue::from_str(&format!("unirate-rust/{VERSION}"))
                        .expect("user agent must be ASCII"),
                );
                reqwest::Client::builder()
                    .timeout(self.timeout)
                    .default_headers(headers)
                    .build()?
            }
        };

        // Normalize base_url — strip trailing slash so path joins remain stable.
        let base_url = self.base_url.trim_end_matches('/').to_string();

        Ok(Client {
            api_key: self.api_key,
            base_url,
            http,
        })
    }
}
