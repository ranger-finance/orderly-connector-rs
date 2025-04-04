# Orderly Connector RS

Rust SDK for interacting with the Orderly Network API (v1/v2).

This library provides clients for both REST API interactions and WebSocket stream subscriptions.

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

## Getting Started

### Adding Dependency

```toml
[dependencies]
orderly-connector-rs = { git = "<your_repo_url>" } # Or path, or crates.io once published
# Add other necessary dependencies like tokio, serde, etc.
```

### Configuration

The library (specifically the examples and tests) expects configuration via environment variables. Create a `.env` file in the root of your project (`orderly-connector-rs/`) with the following structure:

```.env
ORDERLY_API_KEY=your_api_key_or_orderly_key
ORDERLY_SECRET=your_api_secret_or_orderly_secret
ORDERLY_ACCOUNT_ID=your_orderly_account_id
# Set to true for testnet, false for mainnet
ORDERLY_TESTNET=true

# Optional, if needed for specific auth/signing
# SOLANA_PRIVATE_KEY_BS58=your_solana_private_key
```

**Note:** Ensure `ORDERLY_TESTNET` is lowercase (`true` or `false`).

### Examples

The `examples/` directory contains usage examples:

- `rest_public.rs`: Demonstrates public REST API calls.
- `rest_private.rs`: Demonstrates private REST API calls (order placement).
- `ws_public.rs`: Demonstrates connecting and subscribing to public WebSocket streams.
- `ws_private.rs`: Demonstrates connecting and subscribing to private WebSocket streams.

To run an example (e.g., `ws_public`):

```bash
cargo run --example ws_public
```

### Testing

Integration tests are located in the `tests/` directory.

1.  Ensure your `.env` file is configured correctly in the `orderly-connector-rs/` directory, with `ORDERLY_TESTNET=true` and corresponding testnet credentials.
2.  Run the tests:

    ```bash
    # Run only ignored tests (which require network/credentials)
    cargo test -- --ignored

    # Run all tests (including ignored)
    # cargo test -- --include-ignored
    ```

## Contributing

(Add contribution guidelines here if applicable)

## License

(Specify your license, e.g., MIT, Apache-2.0)
