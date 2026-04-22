//! # UniRate API — Rust client
//!
//! Official Rust client for the [UniRate API](https://unirateapi.com) — free,
//! real-time and historical currency exchange rates plus VAT rates.
//!
//! ## Quick start
//!
//! ```no_run
//! use unirate_api::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("your-api-key");
//!
//!     let rate = client.get_rate("USD", "EUR").await?;
//!     println!("USD → EUR: {}", rate);
//!
//!     let converted = client.convert(100.0, "USD", "EUR").await?;
//!     println!("100 USD = {} EUR", converted);
//!
//!     Ok(())
//! }
//! ```
//!
//! Get a free API key at <https://unirateapi.com>.

#![deny(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

pub mod client;
pub mod endpoints;
pub mod error;
pub mod models;

pub use client::{Client, ClientBuilder};
pub use error::UniRateError;
pub use models::{
    HistoricalLimit, HistoricalLimitsResponse, VatRate, VatRateResponse, VatRatesResponse,
};

/// Default UniRate API base URL.
pub const DEFAULT_BASE_URL: &str = "https://api.unirateapi.com";

/// Crate version, exposed as the `User-Agent` suffix.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
