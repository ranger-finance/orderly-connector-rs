use crate::error::OrderlyError;
use solabi::encode::{encode, Encode, Encoder, Size};
use solabi::ethprim::U256;
use solabi::keccak::v256; // Use the canonical Keccak-256 implementation
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// Define ABI types for withdrawal message
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

// Define ABI types for registration message
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
        Size::Static(7) // 7 fields, each 32 bytes
    }
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write(&self.broker_id_hash);
        self.chain_id.encode(encoder);
        encoder.write(&self.receiver);
        encoder.write(&self.token_hash);
        self.amount.encode(encoder);
        self.withdraw_nonce.encode(encoder);
        self.timestamp.encode(encoder);
    }
}

// Implement solabi::Encode for RegistrationMessage
impl Encode for RegistrationMessage {
    fn size(&self) -> Size {
        Size::Static(4) // 4 fields, each 32 bytes
    }
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write(&self.broker_id_hash);
        self.chain_id.encode(encoder);
        self.timestamp.encode(encoder);
        self.registration_nonce.encode(encoder);
    }
}

// Helper functions for creating messages
pub fn create_withdrawal_message(
    broker_id: &str,
    chain_id: u64,
    receiver_address_str: &str, // Solana address as string
    token: &str,
    amount: u64,
    withdraw_nonce: u64,
    timestamp: u64, // Unix ms
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

pub fn create_registration_message(
    broker_id: &str,
    chain_id: u64,
    timestamp: u64, // Unix ms
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
    use hex_literal::hex;

    // Helper to decode hex string to [u8; 32]
    fn hex_to_arr32(hex_str: &str) -> [u8; 32] {
        let bytes = hex::decode(hex_str).expect("Failed to decode hex");
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    }

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
            hex_to_arr32("9c94e16c6592a880a57f8aa3036b25a3a86772810e6a12c617c5b821420b7d69");
        let expected_token_hash =
            hex_to_arr32("5553444300000000000000000000000000000000000000000000000000000000"); // keccak256("USDC") padded
        let receiver_pubkey = Pubkey::from_str(receiver_address).unwrap();
        let expected_receiver = receiver_pubkey.to_bytes();

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
            hex_to_arr32("1c7d19220b0418d803065c586cf7e715371732b623e60923158e7c8a3084d08c");

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
