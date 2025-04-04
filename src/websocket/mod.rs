//! WebSocket Client module

pub mod client;

// Re-export the client structs for easier access
pub use client::{WebsocketPrivateClient, WebsocketPublicClient};
