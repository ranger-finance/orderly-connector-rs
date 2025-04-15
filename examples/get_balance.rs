use orderly_connector_rs::rest::client::Credentials;
use orderly_connector_rs::rest::OrderlyService;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Get credentials from environment variables
    let api_key = env::var("ORDERLY_API_KEY").expect("ORDERLY_API_KEY must be set");
    let secret = env::var("ORDERLY_SECRET").expect("ORDERLY_SECRET must be set");
    let account_id = env::var("ORDERLY_ACCOUNT_ID").expect("ORDERLY_ACCOUNT_ID must be set");
    let is_testnet = env::var("ORDERLY_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .expect("ORDERLY_TESTNET must be true or false");

    println!(
        "Using network: {}",
        if is_testnet { "testnet" } else { "mainnet" }
    );
    println!("Account ID: {}", account_id);

    // Initialize the client
    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    // Create credentials
    let creds = Credentials {
        orderly_key: Box::leak(api_key.into_boxed_str()),
        orderly_secret: Box::leak(secret.into_boxed_str()),
        orderly_account_id: Box::leak(account_id.into_boxed_str()),
    };

    // Get account information
    println!("\nAccount Information:");
    match client.get_account_info(&creds).await {
        Ok(info) => {
            println!("  Account ID: {}", info.data.account_id);
            println!("  Account Mode: {}", info.data.account_mode);
            println!("  Maker Fee Rate: {}%", info.data.maker_fee_rate);
            println!("  Taker Fee Rate: {}%", info.data.taker_fee_rate);
            println!(
                "  Futures Maker Fee Rate: {}%",
                info.data.futures_maker_fee_rate
            );
            println!(
                "  Futures Taker Fee Rate: {}%",
                info.data.futures_taker_fee_rate
            );
            println!("  Max Leverage: {}x", info.data.max_leverage);
        }
        Err(e) => eprintln!("Error fetching account info: {}", e),
    }

    // Get holdings (balances)
    println!("\nToken Balances:");
    match client.get_holding(&creds).await {
        Ok(holdings) => {
            for holding in holdings.data.holding {
                println!("  {} Balance:", holding.token);
                println!("    Total: {}", holding.holding);
                println!("    Available: {}", holding.available_balance);
                println!("    Frozen: {}", holding.frozen);
                if let Some(pending_short) = holding.pending_short_qty {
                    println!("    Pending Short: {}", pending_short);
                }
                println!();
            }
        }
        Err(e) => eprintln!("Error fetching holdings: {}", e),
    }

    Ok(())
}
