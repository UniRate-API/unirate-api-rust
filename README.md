# UniRate Rust Client

Official Rust client for the [UniRate API](https://unirateapi.com) — free, real-time and historical currency exchange rates plus VAT rates.

- Real-time exchange rates between 170+ currencies (fiat + crypto)
- Historical rates back to 1999
- Time-series ranges up to 5 years
- Currency conversion (current and historical)
- VAT rates for countries worldwide
- Free tier, no credit card required
- Modern Rust: `async`/`await`, `Send + Sync`, `serde`-derived models
- Pulls in only `reqwest` (rustls) + `serde` + `thiserror` — no OpenSSL dependency

## Requirements

- Rust 1.74+ (stable)
- An async runtime (examples and tests use [`tokio`])

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unirate-api = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Or via the CLI:

```bash
cargo add unirate-api
```

## Quick start

```rust
use unirate_api::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("your-api-key");

    // Current rate
    let rate = client.get_rate("USD", "EUR").await?;
    println!("USD -> EUR: {rate}");

    // Convert
    let euros = client.convert(100.0, "USD", "EUR").await?;
    println!("100 USD = {euros} EUR");

    // All supported currencies
    let currencies = client.get_supported_currencies().await?;
    println!("{} currencies supported", currencies.len());
    Ok(())
}
```

Get a free API key at [https://unirateapi.com](https://unirateapi.com).

## API

### Current rates

```rust
// Single pair
let rate: f64 = client.get_rate("USD", "EUR").await?;

// All rates for a base
let rates: std::collections::HashMap<String, f64> =
    client.get_all_rates("USD").await?;

// Convert an amount
let result: f64 = client.convert(100.0, "USD", "EUR").await?;

// Supported currency list
let codes: Vec<String> = client.get_supported_currencies().await?;
```

### Historical data (Pro)

These endpoints return `403` on the free tier and surface as
`UniRateError::Api { status: 403, .. }`.

```rust
// Rate on a specific date
let rate = client.get_historical_rate("2024-01-01", "USD", "EUR").await?;

// All rates on a date
let rates = client.get_historical_rates("2024-01-01", "USD").await?;

// Convert using historical rate
let amount = client
    .convert_historical(100.0, "USD", "EUR", "2024-01-01")
    .await?;

// Time series
let series = client
    .get_time_series(
        "2024-01-01",
        "2024-01-07",
        "USD",
        Some(&["EUR", "GBP"]),
        1.0,
    )
    .await?;

// Available historical coverage per currency
let limits = client.get_historical_limits().await?;
```

### VAT rates

```rust
// All countries
let vat = client.get_vat_rates().await?;

// Single country (ISO-3166 alpha-2)
let germany = client.get_vat_rate("DE").await?;
println!("Germany VAT: {}%", germany.vat_data.vat_rate);
```

## Error handling

All methods return `Result<T, UniRateError>`:

```rust
use unirate_api::UniRateError;

match client.get_rate("USD", "ZZZ").await {
    Ok(rate) => println!("{rate}"),
    Err(UniRateError::Authentication) => eprintln!("bad api key"),
    Err(UniRateError::InvalidCurrency) => eprintln!("unknown currency code"),
    Err(UniRateError::RateLimit) => eprintln!("back off and retry"),
    Err(UniRateError::InvalidDate) => eprintln!("bad date format"),
    Err(UniRateError::Api { status, body }) => eprintln!("HTTP {status}: {body}"),
    Err(e) => eprintln!("transport error: {e}"),
}
```

## Advanced — custom `reqwest::Client`

Use `Client::builder` to swap in a pre-configured HTTP client — handy for
tests against a local `wiremock::MockServer`, or to share a connection pool
with the rest of your app:

```rust
use std::time::Duration;
use unirate_api::Client;

let http = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?;

let client = Client::builder("your-api-key")
    .http_client(http)
    .build()?;
```

## Rate limits

- **Currency endpoints:** standard rate limits apply
- **Historical endpoints:** 50 requests/hour on the free tier
- **VAT endpoints:** 1800 requests/hour on the free tier

## Related clients

- [unirate-api-python](https://github.com/UniRate-API/unirate-api-python) (PyPI: `unirate-api`)
- [unirate-api-nodejs](https://github.com/UniRate-API/unirate-api-nodejs) (npm: `unirate-api`)
- [unirate-api-swift](https://github.com/UniRate-API/unirate-api-swift) (Swift Package Manager)

## License

MIT — see [LICENSE](LICENSE).
