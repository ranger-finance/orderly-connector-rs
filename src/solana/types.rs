use anchor_lang::declare_id;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn get_program_id(name: &str) -> Option<Pubkey> {
    match name {
        "VAULT" => Pubkey::from_str("ErBmAD61mGFKvrFNaTJuxoPwqrS8GgtwtqJTJVjFWx9Q").ok(),
        "ENDPOINT" => Pubkey::from_str("LzV2EndpointV211111111111111111111111111111").ok(),
        "SEND_LIB" => Pubkey::from_str("LzV2SendLib11111111111111111111111111111111").ok(),
        "TREASURY" => Pubkey::from_str("LzV2Treasury1111111111111111111111111111111").ok(),
        "EXECUTOR" => Pubkey::from_str("LzV2Executor1111111111111111111111111111111").ok(),
        "PRICE_FEED" => Pubkey::from_str("LzV2PriceFeed111111111111111111111111111111").ok(),
        "DVN" => Pubkey::from_str("LzV2DVN111111111111111111111111111111111111").ok(),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub api_base_url: String, // For nonce retrieval
    pub usdc_mint: Pubkey,
    pub broker_id: String,
    // pub layerzero_dst_eid: Option<u32>, // Optional: Make configurable? Defaults to mainnet const?
    // Include details needed for EIP 712 if fetched from config
    // pub orderly_chain_id: u64, // e.g., 421614 for Arb Sepolia
    // pub eip712_verifying_contract: String, // Address as hex string
    // Solana Chain ID used for off-chain message signing (e.g., 900900900 from gist)
    pub orderly_solana_chain_id: u64,
}

// LayerZero Endpoint ID for Solana Mainnet (Verified from JS: getDstEID)
pub const LAYERZERO_SOLANA_MAINNET_EID: u32 = 30109;

pub const SOLANA_MAINNET_CHAIN_ID: u64 = 900900900;
pub const SOLANA_DEVNET_CHAIN_ID: u64 = 901901901;
