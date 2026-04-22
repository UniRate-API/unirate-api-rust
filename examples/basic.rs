//! Runnable example for the UniRate Rust client.
//!
//! Usage:
//!
//! ```bash
//! UNIRATE_API_KEY=your-key cargo run --example basic
//! ```

use unirate_api::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("UNIRATE_API_KEY")
        .map_err(|_| "UNIRATE_API_KEY must be set to run this example")?;
    let client = Client::new(key);

    // Current rate
    let rate = client.get_rate("USD", "EUR").await?;
    println!("USD -> EUR: {rate}");

    // Conversion
    let converted = client.convert(100.0, "USD", "EUR").await?;
    println!("100 USD = {converted} EUR");

    // Supported currencies
    let currencies = client.get_supported_currencies().await?;
    println!("{} supported currencies", currencies.len());

    Ok(())
}
