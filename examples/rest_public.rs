// examples/rest_public.rs
use orderly_connector_rs::rest::Client;
use std::env;

#[tokio::main]
async fn main() {
    // Optional: Load .env file if you have one
    dotenv::dotenv().ok();

    // Load configuration from environment variables
    // NOTE: Even public endpoints require these for client initialization in this example.
    let api_key = env::var("ORDERLY_API_KEY").expect("ORDERLY_API_KEY not set");
    let secret = env::var("ORDERLY_SECRET").expect("ORDERLY_SECRET not set");
    let account_id = env::var("ORDERLY_ACCOUNT_ID").expect("ORDERLY_ACCOUNT_ID not set");
    let is_testnet: bool = env::var("ORDERLY_TESTNET").unwrap_or("true".to_string()).parse().expect("ORDERLY_TESTNET must be true or false");

    // Initialize the client
    let client = Client::new(api_key, secret, account_id, is_testnet, None)
        .expect("Failed to create REST client");

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