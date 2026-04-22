//! Mock tests for the UniRate Rust client.
//!
//! These run against a local `wiremock::MockServer` — no network access
//! required. Cover all 9 endpoints plus the error-mapping paths.

use std::time::Duration;

use unirate_api::{Client, UniRateError};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Build a client pointed at `server.uri()`. Uses a short timeout so hung
/// stubs fail fast.
async fn make_client(server: &MockServer) -> Client {
    Client::builder("test-key")
        .base_url(server.uri())
        .timeout(Duration::from_secs(5))
        .build()
        .expect("client builds")
}

// ---------------------------------------------------------------------------
// Happy-path tests — one per endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_rate_returns_float_and_uppercases_codes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/rates"))
        .and(query_param("from", "USD"))
        .and(query_param("to", "EUR"))
        .and(query_param("api_key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"rate": 0.9321}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let rate = client.get_rate("usd", "eur").await.expect("rate");
    assert!((rate - 0.9321).abs() < 1e-6);
}

#[tokio::test]
async fn get_rate_accepts_string_encoded_number() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/rates"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"rate": "0.8412"}"#))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let rate = client.get_rate("USD", "EUR").await.expect("rate");
    assert!((rate - 0.8412).abs() < 1e-6);
}

#[tokio::test]
async fn get_all_rates_returns_map() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/rates"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"rates": {"EUR": 0.9, "GBP": "0.8"}}"#),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let rates = client.get_all_rates("USD").await.expect("rates");
    assert_eq!(rates.get("EUR").copied(), Some(0.9));
    assert_eq!(rates.get("GBP").copied(), Some(0.8));
}

#[tokio::test]
async fn convert_returns_float() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/convert"))
        .and(query_param("from", "USD"))
        .and(query_param("to", "EUR"))
        .and(query_param("amount", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"result": 93.21}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let amount = client.convert(100.0, "USD", "EUR").await.expect("convert");
    assert!((amount - 93.21).abs() < 0.01);
}

#[tokio::test]
async fn get_supported_currencies_returns_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/currencies"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"currencies": ["USD", "EUR", "GBP", "BTC"]}"#),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let codes = client.get_supported_currencies().await.expect("currencies");
    assert_eq!(codes, vec!["USD", "EUR", "GBP", "BTC"]);
}

#[tokio::test]
async fn get_historical_rate_sends_date_and_returns_rate() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/historical/rates"))
        .and(query_param("date", "2024-01-01"))
        .and(query_param("amount", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"rate": 0.8412}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let rate = client
        .get_historical_rate("2024-01-01", "USD", "EUR")
        .await
        .expect("historical rate");
    assert!((rate - 0.8412).abs() < 1e-6);
}

#[tokio::test]
async fn get_historical_rates_returns_map() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/historical/rates"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"rates": {"EUR": 0.9, "GBP": 0.8}}"#),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let rates = client
        .get_historical_rates("2024-01-01", "USD")
        .await
        .expect("historical rates");
    assert_eq!(rates.get("EUR").copied(), Some(0.9));
}

#[tokio::test]
async fn convert_historical_returns_result() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/historical/rates"))
        .and(query_param("amount", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"result": 84.12}"#))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let result = client
        .convert_historical(100.0, "USD", "EUR", "2024-01-01")
        .await
        .expect("convert historical");
    assert!((result - 84.12).abs() < 0.01);
}

#[tokio::test]
async fn get_time_series_returns_nested_map_and_joins_currencies() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/historical/timeseries"))
        .and(query_param("start_date", "2024-01-01"))
        .and(query_param("end_date", "2024-01-02"))
        .and(query_param("base", "USD"))
        .and(query_param("currencies", "EUR,GBP"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"data": {"2024-01-01": {"EUR": 0.9}, "2024-01-02": {"EUR": 0.91}}}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let series = client
        .get_time_series(
            "2024-01-01",
            "2024-01-02",
            "USD",
            Some(&["eur", "gbp"]),
            1.0,
        )
        .await
        .expect("series");
    assert_eq!(
        series.get("2024-01-01").and_then(|m| m.get("EUR")).copied(),
        Some(0.9)
    );
    assert_eq!(
        series.get("2024-01-02").and_then(|m| m.get("EUR")).copied(),
        Some(0.91)
    );
}

#[tokio::test]
async fn get_historical_limits_decodes_struct() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/historical/limits"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"total_currencies": 2, "currencies": {"USD": {"earliest_date": "1999-01-01", "latest_date": "2026-04-20"}, "EUR": {"earliest_date": "1999-01-01", "latest_date": "2026-04-20"}}}"#,
        ))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let limits = client.get_historical_limits().await.expect("limits");
    assert_eq!(limits.total_currencies, 2);
    assert_eq!(
        limits.currencies.get("USD").unwrap().earliest_date,
        "1999-01-01"
    );
}

#[tokio::test]
async fn get_vat_rate_decodes_struct() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/vat/rates"))
        .and(query_param("country", "DE"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"country": "DE", "vat_data": {"country_code": "DE", "country_name": "Germany", "vat_rate": 19.0}}"#,
        ))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client.get_vat_rate("de").await.expect("vat");
    assert_eq!(resp.country.as_deref(), Some("DE"));
    assert_eq!(resp.vat_data.vat_rate, 19.0);
    assert_eq!(resp.vat_data.country_code.as_deref(), Some("DE"));
    assert_eq!(resp.vat_data.country_name.as_deref(), Some("Germany"));
}

#[tokio::test]
async fn get_vat_rates_decodes_struct() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/vat/rates"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"date": "2026-01-22", "total_countries": 2, "vat_rates": {"DE": {"country_code": "DE", "country_name": "Germany", "vat_rate": 19.0}, "FR": {"country_code": "FR", "country_name": "France", "vat_rate": 20.0}}}"#,
        ))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client.get_vat_rates().await.expect("vat all");
    assert_eq!(resp.total_countries, 2);
    assert_eq!(resp.date.as_deref(), Some("2026-01-22"));
    assert_eq!(resp.vat_rates.get("DE").unwrap().vat_rate, 19.0);
    assert_eq!(
        resp.vat_rates.get("FR").unwrap().country_name.as_deref(),
        Some("France")
    );
}

// ---------------------------------------------------------------------------
// Error-path tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn http_401_maps_to_authentication_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let client = make_client(&server).await;
    match client.get_rate("USD", "EUR").await {
        Err(UniRateError::Authentication) => {}
        other => panic!("expected Authentication, got {other:?}"),
    }
}

#[tokio::test]
async fn http_429_maps_to_rate_limit_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;
    let client = make_client(&server).await;
    match client.get_rate("USD", "EUR").await {
        Err(UniRateError::RateLimit) => {}
        other => panic!("expected RateLimit, got {other:?}"),
    }
}

#[tokio::test]
async fn http_404_maps_to_invalid_currency_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let client = make_client(&server).await;
    match client.get_rate("USD", "ZZZ").await {
        Err(UniRateError::InvalidCurrency) => {}
        other => panic!("expected InvalidCurrency, got {other:?}"),
    }
}

#[tokio::test]
async fn http_400_maps_to_invalid_date_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&server)
        .await;
    let client = make_client(&server).await;
    match client.get_historical_rate("not-a-date", "USD", "EUR").await {
        Err(UniRateError::InvalidDate) => {}
        other => panic!("expected InvalidDate, got {other:?}"),
    }
}

#[tokio::test]
async fn http_403_surfaces_as_api_error_with_body() {
    // Free-tier keys receive 403 for historical endpoints — clients must
    // surface this as a generic Api error so callers can detect Pro gating.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(403).set_body_string(
                r#"{"error": "Historical data access requires a Pro subscription"}"#,
            ),
        )
        .mount(&server)
        .await;
    let client = make_client(&server).await;
    match client.get_historical_rate("2024-01-01", "USD", "EUR").await {
        Err(UniRateError::Api { status, body }) => {
            assert_eq!(status, 403);
            assert!(body.contains("Pro subscription"), "body was {body:?}");
        }
        other => panic!("expected Api {{ status: 403, .. }}, got {other:?}"),
    }
}

#[tokio::test]
async fn api_key_is_appended_to_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/currencies"))
        .and(query_param("api_key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"currencies": []}"#))
        .expect(1)
        .mount(&server)
        .await;
    let client = make_client(&server).await;
    let _ = client.get_supported_currencies().await.expect("currencies");
}

#[tokio::test]
async fn accept_header_is_json() {
    use wiremock::matchers::header;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/currencies"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"currencies": []}"#))
        .expect(1)
        .mount(&server)
        .await;
    let client = make_client(&server).await;
    let _ = client.get_supported_currencies().await.expect("currencies");
}
