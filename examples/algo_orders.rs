use orderly_connector_rs::rest::client::{Credentials, OrderlyService};
use orderly_connector_rs::types::GetAlgoOrdersParams;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Optional: Load .env file if present
    dotenv::dotenv().ok();

    // Load credentials from environment variables
    let api_key = env::var("ORDERLY_API_KEY").expect("ORDERLY_API_KEY not set");
    let secret = env::var("ORDERLY_SECRET").expect("ORDERLY_SECRET not set");
    let account_id = env::var("ORDERLY_ACCOUNT_ID").expect("ORDERLY_ACCOUNT_ID not set");
    let is_testnet: bool = false;

    // Initialize the OrderlyService client
    let client = OrderlyService::new(is_testnet, None)?;

    // Prepare credentials struct
    let creds = Credentials {
        orderly_key: &api_key,
        orderly_secret: &secret,
        orderly_account_id: &account_id,
    };

    // 1. Fetch all algo orders (first page, default size)
    println!("Fetching all algo orders...");
    let params = GetAlgoOrdersParams::default();
    let all_orders_resp = client.get_algo_orders(&creds, params).await?;
    println!("All algo orders: {:#?}", all_orders_resp.data.rows);

    // 2. If any exist, fetch details for the first algo_order_id
    if let Some(first_order) = all_orders_resp.data.rows.first() {
        let order_id = &first_order.algo_order_id;
        println!("\nFetching details for algo_order_id: {}", order_id);
        let order_detail_resp = client
            .get_algo_order_by_id(&creds, &order_id.to_string())
            .await?;
        println!("Algo order details: {:#?}", order_detail_resp.data);
    } else {
        println!("No algo orders found for this account.");
    }

    Ok(())
}
