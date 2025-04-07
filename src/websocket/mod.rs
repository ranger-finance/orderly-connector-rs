//! WebSocket API client implementation for the Orderly Network.
//!
//! This module provides WebSocket clients for connecting to Orderly Network's real-time data streams.
//! It supports both public and private data streams through two main client types:
//!
//! - [`WebsocketPublicClient`]: For public market data streams (tickers, orderbook, trades)
//! - [`WebsocketPrivateClient`]: For private, authenticated streams (orders, positions, balance)
//!
//! # Architecture
//!
//! The WebSocket implementation uses a robust connection management system that provides:
//!
//! - Automatic connection management and recovery
//! - Subscription state persistence
//! - Asynchronous message handling
//! - Automatic ping/pong handling
//! - Clean shutdown capabilities
//!
//! # Usage
//!
//! ## Public Data Streams
//!
//! ```no_run
//! use orderly_connector_rs::websocket::WebsocketPublicClient;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create message handlers
//!     let message_handler = Arc::new(|msg: String| {
//!         println!("Received: {}", msg);
//!     });
//!     let close_handler = Arc::new(|| {
//!         println!("Connection closed");
//!     });
//!
//!     // Connect to public streams
//!     let client = WebsocketPublicClient::connect(
//!         "your_account_id".to_string(),
//!         true, // is_testnet
//!         message_handler,
//!         close_handler,
//!     ).await.expect("Failed to connect");
//!
//!     // Subscribe to streams
//!     client.subscribe_tickers().await.expect("Failed to subscribe");
//!     client.subscribe_orderbook("PERP_ETH_USDC").await.expect("Failed to subscribe");
//!
//!     // Keep alive until Ctrl+C
//!     tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
//!     client.stop().await;
//! }
//! ```
//!
//! ## Private Data Streams
//!
//! ```no_run
//! use orderly_connector_rs::websocket::WebsocketPrivateClient;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create message handlers
//!     let message_handler = Arc::new(|msg: String| {
//!         println!("Received: {}", msg);
//!     });
//!     let close_handler = Arc::new(|| {
//!         println!("Connection closed");
//!     });
//!
//!     // Connect to private streams with authentication
//!     let client = WebsocketPrivateClient::connect(
//!         "your_api_key".to_string(),
//!         "your_secret".to_string(),
//!         "your_account_id".to_string(),
//!         true, // is_testnet
//!         message_handler,
//!         close_handler,
//!     ).await.expect("Failed to connect");
//!
//!     // Subscribe to private streams
//!     client.subscribe_balance().await.expect("Failed to subscribe");
//!     client.subscribe_positions().await.expect("Failed to subscribe");
//!
//!     // Keep alive until Ctrl+C
//!     tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
//!     client.stop().await;
//! }
//! ```
//!
//! # Error Handling
//!
//! All WebSocket operations return a `Result` type that can be handled for proper error management:
//!
//! ```no_run
//! use orderly_connector_rs::websocket::WebsocketPublicClient;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = WebsocketPublicClient::connect(
//!         "your_account_id".to_string(),
//!         true,
//!         Arc::new(|msg| println!("Received: {}", msg)),
//!         Arc::new(|| println!("Closed")),
//!     ).await;
//!
//!     match client {
//!         Ok(client) => {
//!             if let Err(e) = client.subscribe_tickers().await {
//!                 eprintln!("Failed to subscribe: {}", e);
//!             }
//!         }
//!         Err(e) => eprintln!("Connection failed: {}", e),
//!     }
//! }
//! ```
//!
//! # Reconnection Behavior
//!
//! Both client types implement automatic reconnection with the following behavior:
//!
//! - Maximum retries: 30 attempts
//! - Delay between retries: 5 seconds
//! - Automatic resubscription to previous topics after reconnection
//! - Authentication renewal for private streams
//!
//! # Message Handling
//!
//! Messages are handled asynchronously through callback functions:
//!
//! - `on_message`: Called for each received message
//! - `on_close`: Called when the connection is closed
//!
//! These callbacks should be thread-safe and quick to execute to avoid blocking the WebSocket loop.

pub mod client;

// Re-export the client structs for easier access
pub use client::{WebsocketClientConfig, WebsocketPrivateClient, WebsocketPublicClient};
