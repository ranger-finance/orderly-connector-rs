// examples/rest_public.rs
use orderly_connector_rs::rest::OrderlyService;
use std::env;

#[tokio::main]
async fn main() {
    // Optional: Load .env file if you have one
    dotenv::dotenv().ok();

    let is_testnet: bool = env::var("ORDERLY_TESTNET")
        .unwrap_or("true".to_string())
        .parse()
        .expect("ORDERLY_TESTNET must be true or false");

    // Initialize the client
    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    println!("Using Testnet: {}", is_testnet);

    // --- Get System Status ---
    println!("\nFetching system status...");
    match client.get_system_status().await {
        Ok(status) => println!("System Status: {:#?}", status),
        Err(e) => eprintln!("Error fetching system status: {}", e),
    }

    // --- Get Exchange Info (All Symbols) ---
    println!("\nFetching exchange info (all symbols)...");
    match client.get_exchange_info(None).await {
        Ok(info) => println!("Exchange Info (All): {:#?}", info),
        Err(e) => eprintln!("Error fetching exchange info: {}", e),
    }

    // --- Get Exchange Info (Specific Symbol) ---
    let symbol = "PERP_ETH_USDC"; // Example symbol, works on testnet
    println!("\nFetching exchange info for {}...", symbol);
    match client.get_exchange_info(Some(symbol)).await {
        Ok(info) => println!("Exchange Info ({}): {:#?}", symbol, info),
        Err(e) => eprintln!("Error fetching exchange info for {}: {}", symbol, e),
    }

    // --- Get Futures Info (All Symbols) ---
    println!("\nFetching futures info (all symbols)...");
    match client.get_futures_info(None).await {
        Ok(info) => println!("Futures Info (All): {:#?}", info),
        Err(e) => eprintln!("Error fetching futures info: {}", e),
    }

    // --- Get Futures Info (Specific Symbol) ---
    println!("\nFetching futures info for {}...", symbol);
    match client.get_futures_info(Some(symbol)).await {
        Ok(info) => println!("Futures Info ({}): {:#?}", symbol, info),
        Err(e) => eprintln!("Error fetching futures info for {}: {}", symbol, e),
    }
}
