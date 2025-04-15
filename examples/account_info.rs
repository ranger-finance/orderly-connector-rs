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
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .expect("ORDERLY_TESTNET must be true or false");

    println!("Using Testnet: {}", is_testnet);

    // Initialize the client
    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    // Create credentials
    let creds = Credentials {
        orderly_key: Box::leak(api_key.into_boxed_str()),
        orderly_secret: Box::leak(secret.into_boxed_str()),
        orderly_account_id: Box::leak(account_id.into_boxed_str()),
    };

    // Get account information
    println!("\nFetching account information...");
    match client.get_account_info(&creds).await {
        Ok(info) => {
            println!("Account Information:");
            println!("  Account ID: {}", info.data.account_id);
            println!("  Email: {}", info.data.email);
            println!("  Market Type: {:?}", info.data.market_type);
            println!("  Leverage: {}", info.data.leverage);
            println!("  Max Leverage: {}", info.data.max_leverage);
            println!("  Free Collateral: {}", info.data.free_collateral);
            println!("  Total Collateral: {}", info.data.total_collateral);
            if let Some(total_pnl) = info.data.total_pnl {
                println!("  Total PnL: {}", total_pnl);
            }
        }
        Err(e) => eprintln!("Error fetching account info: {}", e),
    }

    // Get holdings (balances)
    println!("\nFetching holdings...");
    match client.get_holding(&creds).await {
        Ok(holdings) => {
            println!("Holdings:");
            for holding in holdings.data.holding {
                println!("  Token: {}", holding.token);
                println!("    Total: {}", holding.holding);
                println!("    Free: {}", holding.available_balance);
                println!("    Frozen: {}", holding.frozen);
                if let Some(pending_short) = holding.pending_short_qty {
                    println!("    Pending Short: {}", pending_short);
                }
                println!("");
            }
        }
        Err(e) => eprintln!("Error fetching holdings: {}", e),
    }

    // Get positions
    println!("\nFetching positions...");
    match client.get_positions(&creds).await {
        Ok(positions) => {
            println!("Positions:");
            for position in positions.data.rows {
                println!("  Symbol: {}", position.symbol);
                println!("    Position Qty: {}", position.position_qty);
                println!("    Cost Position: {}", position.cost_position);
                println!("    Mark Price: {}", position.mark_price);
                println!("    Unrealized PnL: {}", position.unrealized_pnl);
                println!("    Average Open Price: {}", position.average_open_price);
                println!("");
            }
        }
        Err(e) => eprintln!("Error fetching positions: {}", e),
    }

    Ok(())
}
