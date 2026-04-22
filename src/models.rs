//! Response models for the UniRate API.
//!
//! These are typed deserialization targets for the structured endpoints
//! ([`get_historical_limits`](crate::Client::get_historical_limits),
//! [`get_vat_rates`](crate::Client::get_vat_rates),
//! [`get_vat_rate`](crate::Client::get_vat_rate)).

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

use crate::endpoints::deserialize_number;

/// Historical-data coverage window for a single currency.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HistoricalLimit {
    /// Earliest date (`YYYY-MM-DD`) for which data is available.
    pub earliest_date: String,
    /// Latest date (`YYYY-MM-DD`) for which data is available.
    pub latest_date: String,
}

/// Response for [`Client::get_historical_limits`](crate::Client::get_historical_limits).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HistoricalLimitsResponse {
    /// Total number of currencies with historical coverage.
    pub total_currencies: u32,
    /// Per-currency coverage details, keyed by ISO-4217 code.
    pub currencies: HashMap<String, HistoricalLimit>,
}

/// VAT rate data for a single country.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VatRate {
    /// ISO-3166 alpha-2 country code (may be omitted in nested responses).
    #[serde(default)]
    pub country_code: Option<String>,
    /// Human-readable country name.
    #[serde(default)]
    pub country_name: Option<String>,
    /// VAT rate as a percentage (e.g. `19.0` for 19%). Accepts numbers or numeric strings.
    #[serde(deserialize_with = "deserialize_number")]
    pub vat_rate: f64,
}

/// Response for [`Client::get_vat_rate`](crate::Client::get_vat_rate).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VatRateResponse {
    /// ISO-3166 alpha-2 country code echoed back by the API.
    #[serde(default)]
    pub country: Option<String>,
    /// Nested VAT rate detail.
    pub vat_data: VatRate,
}

/// Response for [`Client::get_vat_rates`](crate::Client::get_vat_rates).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VatRatesResponse {
    /// Snapshot date (`YYYY-MM-DD`), when provided by the API.
    #[serde(default)]
    pub date: Option<String>,
    /// Total number of countries in the response.
    pub total_countries: u32,
    /// Per-country VAT rates, keyed by ISO-3166 alpha-2 code.
    pub vat_rates: HashMap<String, VatRate>,
}

impl fmt::Display for VatRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.country_code.as_deref().unwrap_or("??");
        write!(f, "{}: {}%", code, self.vat_rate)
    }
}
