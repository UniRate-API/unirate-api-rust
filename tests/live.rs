//! Live integration tests — hit `api.unirateapi.com`.
//!
//! These are gated in two ways:
//!
//! 1. Every test carries `#[ignore]`, so they are skipped by a plain `cargo test`.
//!    Run explicitly with `cargo test --test live -- --ignored`.
//! 2. Each test also short-circuits if `UNIRATE_API_KEY` is unset, so that
//!    even an `--ignored` run on CI without a key does not fail.
//!
//! Only **free-tier** endpoints are exercised (`/api/rates`, `/api/convert`,
//! `/api/currencies`, `/api/vat/rates`). Historical / timeseries / limits
//! return 403 on free keys and belong in the mock suite.

use unirate_api::Client;

fn client() -> Option<Client> {
    let key = std::env::var("UNIRATE_API_KEY").ok()?;
    if key.is_empty() {
        return None;
    }
    Some(Client::new(key))
}

#[tokio::test]
#[ignore = "hits the live UniRate API; set UNIRATE_API_KEY and pass --ignored"]
async fn live_get_rate() {
    let Some(c) = client() else { return };
    let rate = c.get_rate("USD", "EUR").await.expect("live rate");
    assert!(rate > 0.0 && rate < 10.0, "unexpected rate: {rate}");
}

#[tokio::test]
#[ignore = "hits the live UniRate API; set UNIRATE_API_KEY and pass --ignored"]
async fn live_get_all_rates() {
    let Some(c) = client() else { return };
    let rates = c.get_all_rates("USD").await.expect("live all rates");
    assert!(rates.contains_key("EUR"));
    assert!(
        rates.len() > 100,
        "expected >100 rates, got {}",
        rates.len()
    );
}

#[tokio::test]
#[ignore = "hits the live UniRate API; set UNIRATE_API_KEY and pass --ignored"]
async fn live_convert() {
    let Some(c) = client() else { return };
    let result = c.convert(100.0, "USD", "EUR").await.expect("live convert");
    assert!(result > 0.0 && result < 1000.0);
}

#[tokio::test]
#[ignore = "hits the live UniRate API; set UNIRATE_API_KEY and pass --ignored"]
async fn live_supported_currencies() {
    let Some(c) = client() else { return };
    let list = c.get_supported_currencies().await.expect("live currencies");
    assert!(list.contains(&"USD".to_string()));
    assert!(list.contains(&"EUR".to_string()));
    assert!(list.len() > 100);
}

#[tokio::test]
#[ignore = "hits the live UniRate API; set UNIRATE_API_KEY and pass --ignored"]
async fn live_vat_country() {
    let Some(c) = client() else { return };
    let resp = c.get_vat_rate("DE").await.expect("live vat DE");
    assert_eq!(resp.vat_data.country_code.as_deref(), Some("DE"));
    assert_eq!(resp.vat_data.country_name.as_deref(), Some("Germany"));
    assert_eq!(resp.vat_data.vat_rate, 19.0);
}

#[tokio::test]
#[ignore = "hits the live UniRate API; set UNIRATE_API_KEY and pass --ignored"]
async fn live_vat_all() {
    let Some(c) = client() else { return };
    let resp = c.get_vat_rates().await.expect("live vat all");
    assert!(resp.total_countries > 20);
    assert_eq!(
        resp.vat_rates
            .get("DE")
            .and_then(|v| v.country_name.as_deref()),
        Some("Germany")
    );
}
