use solana_program::declare_id;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// Define Program ID constants (Verified from JS helper.ts)
pub mod program_ids {
    use super::*;
    declare_id!("ErBmAD61mGFKvrFNaTJuxoPwqrS8GgtwtqJTJVjFWx9Q");
    pub const VAULT_PROGRAM_ID: Pubkey = Pubkey::new_from_array(*id().as_ref());

    declare_id!("LzV2EndpointV211111111111111111111111111111");
    pub const ENDPOINT_PROGRAM_ID: Pubkey = Pubkey::new_from_array(*id().as_ref());

    declare_id!("LzV2SendLib11111111111111111111111111111111");
    pub const SEND_LIB_PROGRAM_ID: Pubkey = Pubkey::new_from_array(*id().as_ref());

    declare_id!("LzV2Treasury1111111111111111111111111111111");
    pub const TREASURY_PROGRAM_ID: Pubkey = Pubkey::new_from_array(*id().as_ref());

    declare_id!("LzV2Executor1111111111111111111111111111111");
    pub const EXECUTOR_PROGRAM_ID: Pubkey = Pubkey::new_from_array(*id().as_ref());

    declare_id!("LzV2PriceFeed111111111111111111111111111111");
    pub const PRICE_FEED_PROGRAM_ID: Pubkey = Pubkey::new_from_array(*id().as_ref());

    declare_id!("LzV2DVN111111111111111111111111111111111111");
    pub const DVN_PROGRAM_ID: Pubkey = Pubkey::new_from_array(*id().as_ref());
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
