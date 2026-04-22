//! Error types for the UniRate client.

use thiserror::Error;

/// Errors returned by [`Client`](crate::Client) methods.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum UniRateError {
    /// HTTP 401 — missing or invalid API key.
    #[error("Authentication failed: missing or invalid API key")]
    Authentication,

    /// HTTP 429 — rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimit,

    /// HTTP 404 — currency not found or no data available for the requested parameters.
    #[error("Invalid currency: currency not found or no data available")]
    InvalidCurrency,

    /// HTTP 400 — invalid request parameters (commonly bad date format).
    #[error("Invalid date or request parameters")]
    InvalidDate,

    /// Any other non-2xx HTTP response. `403` for Pro-gated endpoints on free tier,
    /// `503` for service unavailable, etc.
    #[error("API error (status {status}): {body}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Raw response body.
        body: String,
    },

    /// Transport-level / network error from `reqwest`.
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON deserialization failure.
    #[error("JSON decoding error: {0}")]
    Json(#[from] serde_json::Error),
}
