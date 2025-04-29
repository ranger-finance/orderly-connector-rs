//! Ethereum ABI Encoding for Orderly Network Messages
//!
//! This module provides Ethereum ABI encoding functionality for Solana-based messages that need to be
//! signed and verified by the Orderly Network. It implements the encoding of withdrawal and registration
//! messages according to the EIP-712 standard, which is required for off-chain message signing.
//!
//! # Overview
//!
//! The module handles two main types of messages:
//! 1. Withdrawal messages - Used when withdrawing funds from Orderly Network
//! 2. Registration messages - Used when registering a new Solana account with Orderly Network
//!
//! Each message type is encoded according to specific ABI rules and then hashed using Keccak-256
//! for signing purposes.
//!
//! # Implementation Details
//!
//! The implementation follows the specifications outlined in `trade.md`:
//! - Uses `solabi` crate for ABI encoding
//! - Implements proper message structure for both withdrawal and registration
//! - Handles Solana-specific address conversions
//! - Provides type-safe message creation functions
//!
//! # Usage Example
//!
//! ```rust
//! use orderly_connector_rs::eth::abi::{create_withdrawal_message, create_registration_message};
//!
//! // Create a withdrawal message
//! let withdrawal = create_withdrawal_message(
//!     "woofi_pro",
//!     900900900,
//!     "9aNfiFoNmbPaP6kA7FbAFJq8voNu813HGraPj9e8z7N7",
//!     "USDC",
//!     100_000_000,
//!     12345,
//!     1678886400000,
//! )?;
//!
//! // Create a registration message
//! let registration = create_registration_message(
//!     "woofi_pro",
//!     900900900,
//!     1678886400000,
//!     12345,
//! )?;
//! ```

use crate::error::OrderlyError;
use solabi::encode::{Encode, Encoder, Size};
use solabi::ethprim::U256;
use solabi::keccak::v256; // Use the canonical Keccak-256 implementation
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Represents a withdrawal message for the Orderly Network.
///
/// This struct follows the EIP-712 standard for message encoding and is used when
/// withdrawing funds from the Orderly Network. All fields are encoded as 32-byte
/// values in the ABI encoding process.
///
/// # Fields
///
/// * `broker_id_hash` - Keccak-256 hash of the broker ID string
/// * `chain_id` - The Solana chain ID (e.g., 900900900 for mainnet)
/// * `receiver` - The Solana public key of the withdrawal recipient
/// * `token_hash` - Keccak-256 hash of the token symbol (e.g., "USDC")
/// * `amount` - The withdrawal amount
/// * `withdraw_nonce` - Unique nonce for the withdrawal request
/// * `timestamp` - Unix timestamp in milliseconds
#[derive(Debug, Clone, PartialEq)]
pub struct WithdrawalMessage {
    pub broker_id_hash: [u8; 32], // bytes32
    pub chain_id: U256,           // uint256
    pub receiver: [u8; 32],       // bytes32 // Store as [u8; 32] for direct ABI encoding
    pub token_hash: [u8; 32],     // bytes32
    pub amount: U256,             // uint256
    pub withdraw_nonce: U256,     // uint256
    pub timestamp: U256,          // uint256
}

/// Represents a registration message for the Orderly Network.
///
/// This struct follows the EIP-712 standard for message encoding and is used when
/// registering a new Solana account with the Orderly Network. All fields are encoded
/// as 32-byte values in the ABI encoding process.
///
/// # Fields
///
/// * `broker_id_hash` - Keccak-256 hash of the broker ID string
/// * `chain_id` - The Solana chain ID (e.g., 900900900 for mainnet)
/// * `timestamp` - Unix timestamp in milliseconds
/// * `registration_nonce` - Unique nonce for the registration request
#[derive(Debug, Clone, PartialEq)]
pub struct RegistrationMessage {
    pub broker_id_hash: [u8; 32], // bytes32
    pub chain_id: U256,           // uint256
    pub timestamp: U256,          // uint256
    pub registration_nonce: U256, // uint256
}

// Implement solabi::Encode for WithdrawalMessage
impl Encode for WithdrawalMessage {
    fn size(&self) -> Size {
        Size::Static(7)
    }
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_word(self.broker_id_hash);
        self.chain_id.encode(encoder);
        encoder.write_word(self.receiver);
        encoder.write_word(self.token_hash);
        self.amount.encode(encoder);
        self.withdraw_nonce.encode(encoder);
        self.timestamp.encode(encoder);
    }
}

// Implement solabi::Encode for RegistrationMessage
impl Encode for RegistrationMessage {
    fn size(&self) -> Size {
        Size::Static(4)
    }
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_word(self.broker_id_hash);
        self.chain_id.encode(encoder);
        self.timestamp.encode(encoder);
        self.registration_nonce.encode(encoder);
    }
}

/// Creates a withdrawal message for the Orderly Network.
///
/// This function constructs a `WithdrawalMessage` with all necessary fields properly
/// hashed and encoded according to the EIP-712 standard.
///
/// # Arguments
///
/// * `broker_id` - The broker ID string (e.g., "woofi_pro")
/// * `chain_id` - The Solana chain ID (e.g., 900900900 for mainnet)
/// * `receiver_address_str` - The Solana address as a base58 string
/// * `token` - The token symbol (e.g., "USDC")
/// * `amount` - The withdrawal amount
/// * `withdraw_nonce` - Unique nonce for the withdrawal request
/// * `timestamp` - Unix timestamp in milliseconds
///
/// # Returns
///
/// Returns a `Result` containing the constructed `WithdrawalMessage` or an `OrderlyError`
/// if the receiver address is invalid.
///
/// # Examples
///
/// ```rust
/// use orderly_connector_rs::eth::abi::create_withdrawal_message;
///
/// let message = create_withdrawal_message(
///     "woofi_pro",
///     900900900,
///     "9aNfiFoNmbPaP6kA7FbAFJq8voNu813HGraPj9e8z7N7",
///     "USDC",
///     100_000_000,
///     12345,
///     1678886400000,
/// )?;
/// ```
pub fn create_withdrawal_message(
    broker_id: &str,
    chain_id: u64,
    receiver_address_str: &str,
    token: &str,
    amount: u64,
    withdraw_nonce: u64,
    timestamp: u64,
) -> Result<WithdrawalMessage, OrderlyError> {
    let broker_id_hash = v256(broker_id.as_bytes()); // [u8; 32]
    let token_hash = v256(token.as_bytes()); // [u8; 32]
    let receiver_pubkey = Pubkey::from_str(receiver_address_str).map_err(|e| {
        OrderlyError::ValidationError(format!("Invalid receiver pubkey string: {}", e))
    })?;
    let receiver = receiver_pubkey.to_bytes();

    Ok(WithdrawalMessage {
        broker_id_hash,
        chain_id: U256::from(chain_id),
        receiver,
        token_hash,
        amount: U256::from(amount),
        withdraw_nonce: U256::from(withdraw_nonce),
        timestamp: U256::from(timestamp),
    })
}

/// Creates a registration message for the Orderly Network.
///
/// This function constructs a `RegistrationMessage` with all necessary fields properly
/// hashed and encoded according to the EIP-712 standard.
///
/// # Arguments
///
/// * `broker_id` - The broker ID string (e.g., "woofi_pro")
/// * `chain_id` - The Solana chain ID (e.g., 900900900 for mainnet)
/// * `timestamp` - Unix timestamp in milliseconds
/// * `registration_nonce` - Unique nonce for the registration request
///
/// # Returns
///
/// Returns a `Result` containing the constructed `RegistrationMessage` or an `OrderlyError`
/// if any error occurs during message creation.
///
/// # Examples
///
/// ```rust
/// use orderly_connector_rs::eth::abi::create_registration_message;
///
/// let message = create_registration_message(
///     "woofi_pro",
///     900900900,
///     1678886400000,
///     12345,
/// )?;
/// ```
pub fn create_registration_message(
    broker_id: &str,
    chain_id: u64,
    timestamp: u64,
    registration_nonce: u64,
) -> Result<RegistrationMessage, OrderlyError> {
    let broker_id_hash = v256(broker_id.as_bytes()); // [u8; 32]

    Ok(RegistrationMessage {
        broker_id_hash,
        chain_id: U256::from(chain_id),
        timestamp: U256::from(timestamp),
        registration_nonce: U256::from(registration_nonce),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solabi::encode::encode;

    /// Helper function to convert a hex string to a 32-byte array.
    ///
    /// This is used in tests to create expected hash values for comparison.
    fn hex_to_arr32(hex_str: &str) -> [u8; 32] {
        let bytes = hex::decode(hex_str).expect("Failed to decode hex");
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    }

    /// Tests the creation and encoding of a withdrawal message.
    ///
    /// This test verifies that:
    /// 1. The message is created with correct field values
    /// 2. All hashes are computed correctly
    /// 3. The ABI encoding produces the expected byte sequence
    ///
    /// The test uses known values and pre-calculated hashes to ensure
    /// consistency with the JavaScript implementation.
    #[test]
    fn test_create_and_encode_withdrawal_message() {
        let broker_id = "woofi_pro";
        let chain_id = 900900900u64;
        let receiver_address = "9aNfiFoNmbPaP6kA7FbAFJq8voNu813HGraPj9e8z7N7";
        let token = "USDC";
        let amount = 100_000_000u64; // Example amount (assuming 6 decimals for USDC)
        let withdraw_nonce = 12345u64;
        let timestamp = 1678886400000u64; // Example timestamp

        let withdrawal_message = create_withdrawal_message(
            broker_id,
            chain_id,
            receiver_address,
            token,
            amount,
            withdraw_nonce,
            timestamp,
        )
        .unwrap();

        // Pre-calculate expected hashes/values
        let expected_broker_id_hash =
            hex_to_arr32("6ca2f644ef7bd6d75953318c7f2580014941e753b3c6d54da56b3bf75dd14dfc");
        let expected_token_hash =
            hex_to_arr32("d6aca1be9729c13d677335161321649cccae6a591554772516700f986f942eaa");
        let expected_receiver = [
            0x7f, 0x6a, 0x29, 0x5b, 0xe4, 0xdc, 0xb9, 0x50, 0x84, 0x4e, 0x74, 0xb3, 0x76, 0xab,
            0x75, 0xd3, 0xd8, 0xc9, 0xb5, 0x56, 0xff, 0xc1, 0x02, 0x2a, 0x3a, 0x2d, 0x5b, 0x2c,
            0x59, 0x09, 0xb9, 0x18,
        ];

        assert_eq!(withdrawal_message.broker_id_hash, expected_broker_id_hash);
        assert_eq!(withdrawal_message.chain_id, U256::from(chain_id));
        assert_eq!(withdrawal_message.receiver, expected_receiver);
        assert_eq!(withdrawal_message.token_hash, expected_token_hash);
        assert_eq!(withdrawal_message.amount, U256::from(amount));
        assert_eq!(
            withdrawal_message.withdraw_nonce,
            U256::from(withdraw_nonce)
        );
        assert_eq!(withdrawal_message.timestamp, U256::from(timestamp));

        // Encode the message
        let encoded_bytes = encode(&withdrawal_message);

        // Construct expected encoded bytes manually (order matters!)
        let mut expected_bytes = Vec::new();
        expected_bytes.extend_from_slice(&expected_broker_id_hash);
        expected_bytes.extend_from_slice(&U256::from(chain_id).to_be_bytes());
        expected_bytes.extend_from_slice(&expected_receiver);
        expected_bytes.extend_from_slice(&expected_token_hash);
        expected_bytes.extend_from_slice(&U256::from(amount).to_be_bytes());
        expected_bytes.extend_from_slice(&U256::from(withdraw_nonce).to_be_bytes());
        expected_bytes.extend_from_slice(&U256::from(timestamp).to_be_bytes());

        assert_eq!(encoded_bytes, expected_bytes);
        assert_eq!(encoded_bytes.len(), 32 * 7);
    }

    /// Tests the creation and encoding of a registration message.
    ///
    /// This test verifies that:
    /// 1. The message is created with correct field values
    /// 2. All hashes are computed correctly
    /// 3. The ABI encoding produces the expected byte sequence
    ///
    /// The test uses known values and pre-calculated hashes to ensure
    /// consistency with the JavaScript implementation.
    #[test]
    fn test_create_and_encode_registration_message() {
        let broker_id = "another_broker";
        let chain_id = 901901901u64; // Devnet example
        let timestamp = 1700000000000u64;
        let registration_nonce = 98765u64;

        let registration_message =
            create_registration_message(broker_id, chain_id, timestamp, registration_nonce)
                .unwrap();

        // Pre-calculate expected hash
        let expected_broker_id_hash =
            hex_to_arr32("63fd74a9e62627565a687605d912f8bcbe55a1677e417919bf24e9e301f79e87");

        assert_eq!(registration_message.broker_id_hash, expected_broker_id_hash);
        assert_eq!(registration_message.chain_id, U256::from(chain_id));
        assert_eq!(registration_message.timestamp, U256::from(timestamp));
        assert_eq!(
            registration_message.registration_nonce,
            U256::from(registration_nonce)
        );

        // Encode the message
        let encoded_bytes = encode(&registration_message);

        // Construct expected encoded bytes manually
        let mut expected_bytes = Vec::new();
        expected_bytes.extend_from_slice(&expected_broker_id_hash);
        expected_bytes.extend_from_slice(&U256::from(chain_id).to_be_bytes());
        expected_bytes.extend_from_slice(&U256::from(timestamp).to_be_bytes());
        expected_bytes.extend_from_slice(&U256::from(registration_nonce).to_be_bytes());

        assert_eq!(encoded_bytes, expected_bytes);
        assert_eq!(encoded_bytes.len(), 32 * 4);
    }

    /// Tests error handling for invalid receiver addresses.
    ///
    /// This test verifies that the `create_withdrawal_message` function
    /// properly handles invalid Solana public key strings by returning
    /// an appropriate error.
    #[test]
    fn test_create_withdrawal_message_invalid_receiver() {
        let result = create_withdrawal_message(
            "test_broker",
            900900900,
            "InvalidPublicKeyString", // Not a valid base58 pubkey
            "USDC",
            1000,
            1,
            1678886400000,
        );

        assert!(result.is_err());
        match result.err().unwrap() {
            OrderlyError::ValidationError(msg) => {
                assert!(msg.contains("Invalid receiver pubkey string"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
