# Orderly Connector RS

<div align="center">
  </br>
  <p>
    <img height="300" src="https://pbs.twimg.com/profile_banners/1764920763360899072/1711621031/1500x500" />
  </p>
  <p>
    <strong>Orderly Connector Rust</strong>
  </p>
  <p>
    <a href="https://ranger.finance">ranger.finance</a>
  </p>
  
  [![Rust](https://github.com/ranger-finance/orderly-connector-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/ranger-finance/orderly-connector-rs/actions/workflows/rust.yml)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Crates.io](https://img.shields.io/crates/v/orderly-connector-rs.svg)](https://crates.io/crates/orderly-connector-rs)
  [![Documentation](https://docs.rs/orderly-connector-rs/badge.svg)](https://docs.rs/orderly-connector-rs)
</div>

Rust SDK for interacting with the Orderly Network API (v1/v2). For more information about Orderly Network, visit [Orderly Documentation](https://orderly.network/docs/home). This is based on the python connector [orderly-evm-connector-python](https://github.com/OrderlyNetwork/orderly-evm-connector-python).

## Features

- **REST Client:** (`rest::Client`)
  - Access public endpoints (system status, exchange info, market data).
  - Access private, authenticated endpoints (account info, order management, positions).
  - Automatic request signing for private endpoints.
- **WebSocket Clients:**
  - `websocket::WebsocketPublicClient`: Subscribe to public data streams (tickers, orderbooks, klines, etc.) for a specific account ID.
  - `websocket::WebsocketPrivateClient`: Subscribe to private data streams (execution reports, balance updates, position updates) using API key authentication.
  - Automatic WebSocket authentication for private streams.
  - Built-in reconnection logic with configurable retries.
- **Core Modules:**
  - `types`: Request/response structures and enums.
  - `auth`: Authentication helpers (signing).
  - `error`: Custom error types.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
orderly-connector-rs = "0.1.0"
```

Or install using cargo:

```bash
cargo add orderly-connector-rs
```

## Quick Start

### REST API Client

```rust
use orderly_connector_rs::rest::Client;
use orderly_connector_rs::types::OrderType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with your API credentials
    let client = Client::new(
        "https://testnet-api-evm.orderly.org",  // API base URL
        Some("your_orderly_key"),               // Optional: API key
        Some("your_orderly_secret"),            // Optional: API secret
        Some("your_account_id"),                // Optional: Account ID
    )?;

    // Get system status
    let status = client.get_system_status().await?;
    println!("System status: {:?}", status);

    // Get exchange info
    let info = client.get_exchange_info(None).await?;
    println!("Exchange info: {:?}", info);

    Ok(())
}
```

### WebSocket Client

```rust
use orderly_connector_rs::websocket::{WebsocketPublicClient, WebsocketClientConfig};
use orderly_connector_rs::types::OrderType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create WebSocket configuration
    let config = WebsocketClientConfig {
        base_url: "wss://testnet-ws-evm.orderly.org".to_string(),
        orderly_key: Some("your_orderly_key".to_string()),
        orderly_secret: Some("your_orderly_secret".to_string()),
        orderly_account_id: Some("your_account_id".to_string()),
        wss_id: None,
    };

    // Create public WebSocket client
    let mut public_client = WebsocketPublicClient::new(config.clone())?;

    // Connect to WebSocket
    public_client.connect().await?;

    // Subscribe to orderbook updates
    public_client.subscribe_orderbook("PERP_ETH_USDC").await?;

    // Handle incoming messages
    while let Some(msg) = public_client.next().await {
        match msg {
            Ok(message) => println!("Received: {:?}", message),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}
```

## API Documentation

For detailed API documentation, visit [docs.rs/orderly-connector-rs](https://docs.rs/orderly-connector-rs).

### REST Client

The REST client provides methods for interacting with the Orderly Network API:

#### Public Endpoints

- `get_system_status()`: Get the current system status
- `get_exchange_info(symbol: Option<String>)`: Get exchange information
- `get_market_trades(symbol: String, limit: Option<u32>)`: Get recent market trades
- `get_klines(symbol: String, interval: String, start_time: Option<u64>, end_time: Option<u64>, limit: Option<u32>)`: Get kline/candlestick data

#### Private Endpoints

- `get_account_info()`: Get account information
- `get_positions(symbol: Option<String>)`: Get current positions
- `create_order(request: CreateOrderRequest)`: Create a new order
- `get_orders(params: GetOrdersParams)`: Get order history
- `cancel_order(symbol: String, order_id: Option<String>, client_order_id: Option<String>)`: Cancel an order

### WebSocket Client

The WebSocket client provides real-time data streaming:

#### Public WebSocket

- `subscribe_ticker(symbol: String)`: Subscribe to ticker updates
- `subscribe_orderbook(symbol: String)`: Subscribe to orderbook updates
- `subscribe_trades(symbol: String)`: Subscribe to trade updates
- `subscribe_klines(symbol: String, interval: String)`: Subscribe to kline updates

#### Private WebSocket

- `subscribe_execution()`: Subscribe to execution reports
- `subscribe_position()`: Subscribe to position updates
- `subscribe_balance()`: Subscribe to balance updates

## Error Handling

The library uses a custom error type `OrderlyError` for error handling. All operations return a `Result<T, OrderlyError>` where `T` is the success type.

Example error handling:

```rust
match client.get_system_status().await {
    Ok(status) => println!("System status: {:?}", status),
    Err(OrderlyError::ClientError { status, code, message, .. }) => {
        eprintln!("Client error: {} (code: {})", message, code);
    },
    Err(e) => eprintln!("Error: {:?}", e),
}
```

## Configuration

### Environment Variables

The library supports configuration through environment variables:

```bash
ORDERLY_API_BASE_URL=https://testnet-api-evm.orderly.org
ORDERLY_KEY=your_orderly_key
ORDERLY_SECRET=your_orderly_secret
ORDERLY_ACCOUNT_ID=your_account_id
```

### WebSocket Configuration

The `WebsocketClientConfig` struct allows you to configure:

- `base_url`: WebSocket server URL
- `orderly_key`: API key for authentication
- `orderly_secret`: API secret for authentication
- `orderly_account_id`: Account ID for private streams
- `wss_id`: Optional WebSocket session ID

## Examples

See the `examples` directory for more complete examples:

- `rest_client.rs`: REST API usage examples
- `websocket_client.rs`: WebSocket streaming examples
- `order_management.rs`: Order creation and management examples

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.

## Links

- [Crates.io Package](https://crates.io/crates/orderly-connector-rs)
- [API Documentation](https://docs.rs/orderly-connector-rs)
- [GitHub Repository](https://github.com/ranger-finance/orderly-connector-rs)
- [Orderly Network Documentation](https://orderly.network/docs/home)
