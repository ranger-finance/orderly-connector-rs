use orderly_connector_rs::rest::client::Credentials;
// examples/rest_private.rs
use orderly_connector_rs::rest::OrderlyService;
use orderly_connector_rs::types::{CreateOrderRequest, GetOrdersParams, OrderType, Side};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Optional: Load .env file if you have one
    dotenv::dotenv().ok();

    // Load configuration from environment variables
    let api_key = env::var("ORDERLY_API_KEY").expect("ORDERLY_API_KEY not set");
    let secret = env::var("ORDERLY_SECRET").expect("ORDERLY_SECRET not set");
    let account_id = env::var("ORDERLY_ACCOUNT_ID").expect("ORDERLY_ACCOUNT_ID not set");
    let is_testnet: bool = env::var("ORDERLY_TESTNET")
        .unwrap_or("true".to_string())
        .parse()
        .expect("ORDERLY_TESTNET must be true or false");

    // IMPORTANT: Private endpoint examples modify state (create/cancel orders).
    // Ensure you are running against TESTNET unless you intend to affect mainnet.
    if !is_testnet {
        panic!("Private examples should only be run against testnet (set ORDERLY_TESTNET=true)");
    }
    println!("Using Testnet: {}", is_testnet);

    // Initialize the client
    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let symbol = "PERP_ETH_USDC"; // Testnet symbol

    // --- Create Limit Buy Order ---
    println!("\nAttempting to create a limit buy order for {}...", symbol);
    let order_req = CreateOrderRequest {
        symbol: symbol.to_string(),
        order_type: OrderType::Limit,
        side: Side::Buy,
        order_price: Some(3000.0),
        order_quantity: Some(0.01),
        order_amount: None,
        client_order_id: Some("test_order_001".to_string()),
        visible_quantity: None,
        reduce_only: None, // Not a reduce-only order
    };

    let creds = Credentials {
        orderly_key: &api_key,
        orderly_secret: &secret,
        orderly_account_id: &account_id,
    };

    let order_id = match client.create_order(&creds, order_req).await {
        Ok(resp) => {
            println!("Create Order Response: {:#?}", resp);
            if resp.success {
                Some(resp.data.order_id)
            } else {
                eprintln!("Order creation failed (API success=false)");
                None
            }
        }
        Err(e) => {
            eprintln!("Error creating order: {}", e);
            None
        }
    };

    // Give order time to appear if created
    sleep(Duration::from_secs(2)).await;

    // --- Get Specific Order (if created) ---
    if let Some(id) = order_id {
        println!("\nFetching order {}...", id);
        match client.get_order(&creds, id).await {
            Ok(resp) => println!("Get Order Response: {:#?}", resp),
            Err(e) => eprintln!("Error fetching order {}: {}", id, e),
        }
    }

    // --- Get Orders (Filtered by symbol) ---
    println!("\nFetching orders for {}...", symbol);
    let params = GetOrdersParams {
        symbol: Some(symbol.to_string()),
        ..Default::default()
    };
    match client.get_orders(&creds, Some(params)).await {
        Ok(resp) => println!("Get Orders Response: {:#?}", resp),
        Err(e) => eprintln!("Error fetching orders for {}: {}", symbol, e),
    }

    // --- Cancel Order (if created) ---
    if let Some(id) = order_id {
        println!("\nAttempting to cancel order {}...", id);
        match client.cancel_order(&creds, id, symbol).await {
            Ok(resp) => println!("Cancel Order Response: {:#?}", resp),
            Err(e) => eprintln!("Error cancelling order {}: {}", id, e),
        }
    }
}
