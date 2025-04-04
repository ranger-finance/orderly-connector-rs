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
