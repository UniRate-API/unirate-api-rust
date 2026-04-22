//! Endpoint implementations for [`Client`].

use std::collections::HashMap;

use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use serde_json::Value;

use crate::{
    client::Client,
    error::UniRateError,
    models::{HistoricalLimitsResponse, VatRateResponse, VatRatesResponse},
    VERSION,
};

// ---------------------------------------------------------------------------
// Public endpoint surface
// ---------------------------------------------------------------------------

impl Client {
    /// Fetch the current exchange rate between two currencies.
    ///
    /// Corresponds to `GET /api/rates?from=<FROM>&to=<TO>`.
    pub async fn get_rate(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> Result<f64, UniRateError> {
        let query = vec![("from", upper(from)), ("to", upper(to))];
        let body: RateResponse = self.request("/api/rates", &query).await?;
        Ok(body.rate)
    }

    /// Fetch **all** current exchange rates for a given base currency.
    pub async fn get_all_rates(
        &self,
        from: impl AsRef<str>,
    ) -> Result<HashMap<String, f64>, UniRateError> {
        let query = vec![("from", upper(from))];
        let body: RatesResponse = self.request("/api/rates", &query).await?;
        Ok(body.rates)
    }

    /// Convert an amount from one currency to another using the current rate.
    pub async fn convert(
        &self,
        amount: f64,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> Result<f64, UniRateError> {
        let query = vec![
            ("from", upper(from)),
            ("to", upper(to)),
            ("amount", amount.to_string()),
        ];
        let body: ConvertResponse = self.request("/api/convert", &query).await?;
        Ok(body.result)
    }

    /// Fetch the full list of supported currency codes.
    pub async fn get_supported_currencies(&self) -> Result<Vec<String>, UniRateError> {
        let body: CurrenciesResponse = self.request("/api/currencies", &[]).await?;
        Ok(body.currencies)
    }

    /// Fetch a historical exchange rate for a specific date (`YYYY-MM-DD`).
    ///
    /// **Pro-gated** — returns [`UniRateError::Api`] with `status = 403` on free-tier keys.
    pub async fn get_historical_rate(
        &self,
        date: impl Into<String>,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> Result<f64, UniRateError> {
        let query = vec![
            ("date", date.into()),
            ("amount", "1".to_string()),
            ("from", upper(from)),
            ("to", upper(to)),
        ];
        let body: RateResponse = self.request("/api/historical/rates", &query).await?;
        Ok(body.rate)
    }

    /// Fetch **all** historical exchange rates for a base currency on a given date.
    ///
    /// **Pro-gated** — returns [`UniRateError::Api`] with `status = 403` on free-tier keys.
    pub async fn get_historical_rates(
        &self,
        date: impl Into<String>,
        from: impl AsRef<str>,
    ) -> Result<HashMap<String, f64>, UniRateError> {
        let query = vec![
            ("date", date.into()),
            ("amount", "1".to_string()),
            ("from", upper(from)),
        ];
        let body: RatesResponse = self.request("/api/historical/rates", &query).await?;
        Ok(body.rates)
    }

    /// Convert an amount using a historical exchange rate.
    ///
    /// **Pro-gated** — returns [`UniRateError::Api`] with `status = 403` on free-tier keys.
    pub async fn convert_historical(
        &self,
        amount: f64,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
        date: impl Into<String>,
    ) -> Result<f64, UniRateError> {
        let query = vec![
            ("date", date.into()),
            ("amount", amount.to_string()),
            ("from", upper(from)),
            ("to", upper(to)),
        ];
        let body: ConvertResponse = self.request("/api/historical/rates", &query).await?;
        Ok(body.result)
    }

    /// Fetch a time series of exchange rates between two dates (max 5-year span).
    ///
    /// **Pro-gated** — returns [`UniRateError::Api`] with `status = 403` on free-tier keys.
    pub async fn get_time_series(
        &self,
        start_date: impl Into<String>,
        end_date: impl Into<String>,
        base: impl AsRef<str>,
        currencies: Option<&[&str]>,
        amount: f64,
    ) -> Result<HashMap<String, HashMap<String, f64>>, UniRateError> {
        let mut query = vec![
            ("start_date", start_date.into()),
            ("end_date", end_date.into()),
            ("amount", amount.to_string()),
            ("base", upper(base)),
        ];
        if let Some(list) = currencies {
            if !list.is_empty() {
                let joined = list
                    .iter()
                    .map(|c| c.to_ascii_uppercase())
                    .collect::<Vec<_>>()
                    .join(",");
                query.push(("currencies", joined));
            }
        }
        let body: TimeSeriesResponse = self.request("/api/historical/timeseries", &query).await?;
        Ok(body.data)
    }

    /// Fetch the available historical-data coverage per currency.
    ///
    /// **Pro-gated** — returns [`UniRateError::Api`] with `status = 403` on free-tier keys.
    pub async fn get_historical_limits(&self) -> Result<HistoricalLimitsResponse, UniRateError> {
        self.request("/api/historical/limits", &[]).await
    }

    /// Fetch VAT rates for all supported countries.
    pub async fn get_vat_rates(&self) -> Result<VatRatesResponse, UniRateError> {
        self.request("/api/vat/rates", &[]).await
    }

    /// Fetch the VAT rate for a specific country (ISO-3166 alpha-2 code, e.g. `"DE"`).
    pub async fn get_vat_rate(
        &self,
        country: impl AsRef<str>,
    ) -> Result<VatRateResponse, UniRateError> {
        let query = vec![("country", upper(country))];
        self.request("/api/vat/rates", &query).await
    }
}

// ---------------------------------------------------------------------------
// Internal transport
// ---------------------------------------------------------------------------

impl Client {
    async fn request<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, UniRateError> {
        let url = format!("{}{}", self.base_url, path);
        // api_key is appended as the final query parameter — the Swift client
        // does the same so it shows up last in the URL.
        let mut params: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        params.push(("api_key", self.api_key.as_str()));

        let request = self
            .http
            .get(&url)
            // These headers are set as defaults on the built-in client, but a
            // user-supplied `reqwest::Client` (e.g. in tests) may not carry them
            // — set them per-request to guarantee spec compliance.
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, format!("unirate-rust/{VERSION}"))
            .query(&params);

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            if body.is_empty() {
                return Err(UniRateError::Api {
                    status: status.as_u16(),
                    body: "empty response body".into(),
                });
            }
            return serde_json::from_str::<T>(&body).map_err(UniRateError::from);
        }

        Err(match status.as_u16() {
            400 => UniRateError::InvalidDate,
            401 => UniRateError::Authentication,
            404 => UniRateError::InvalidCurrency,
            429 => UniRateError::RateLimit,
            other => UniRateError::Api {
                status: other,
                body,
            },
        })
    }
}

// ---------------------------------------------------------------------------
// Private response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RateResponse {
    #[serde(deserialize_with = "deserialize_number")]
    rate: f64,
}

#[derive(Debug, Deserialize)]
struct RatesResponse {
    #[serde(deserialize_with = "deserialize_number_map")]
    rates: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct ConvertResponse {
    #[serde(deserialize_with = "deserialize_number")]
    result: f64,
}

#[derive(Debug, Deserialize)]
struct CurrenciesResponse {
    currencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TimeSeriesResponse {
    #[serde(deserialize_with = "deserialize_nested_number_map")]
    data: HashMap<String, HashMap<String, f64>>,
}

// ---------------------------------------------------------------------------
// Numeric coercion helpers — the UniRate API sometimes returns rates as JSON
// numbers and sometimes as numeric strings (e.g. `"0.92"`). Accept both.
// ---------------------------------------------------------------------------

/// Deserialize a `f64` that may arrive as a JSON number **or** numeric string.
pub(crate) fn deserialize_number<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    value_to_f64(&value).map_err(serde::de::Error::custom)
}

fn deserialize_number_map<'de, D>(deserializer: D) -> Result<HashMap<String, f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, Value> = HashMap::deserialize(deserializer)?;
    let mut out = HashMap::with_capacity(raw.len());
    for (k, v) in raw {
        out.insert(k, value_to_f64(&v).map_err(serde::de::Error::custom)?);
    }
    Ok(out)
}

fn deserialize_nested_number_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, HashMap<String, f64>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, HashMap<String, Value>> = HashMap::deserialize(deserializer)?;
    let mut out = HashMap::with_capacity(raw.len());
    for (date, inner) in raw {
        let mut converted = HashMap::with_capacity(inner.len());
        for (code, value) in inner {
            converted.insert(
                code,
                value_to_f64(&value).map_err(serde::de::Error::custom)?,
            );
        }
        out.insert(date, converted);
    }
    Ok(out)
}

fn value_to_f64(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| format!("cannot represent {n} as f64")),
        Value::String(s) => s
            .parse::<f64>()
            .map_err(|e| format!("cannot parse {s:?} as f64: {e}")),
        other => Err(format!("expected number or numeric string, got {other:?}")),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn upper(s: impl AsRef<str>) -> String {
    s.as_ref().to_ascii_uppercase()
}
