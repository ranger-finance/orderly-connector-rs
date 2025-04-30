use crate::error::{OrderlyError, Result};
use crate::eth::abi::create_withdrawal_message;
use crate::rest::client::{Credentials, OrderlyService};
use crate::solana::types::SolanaConfig;
use solabi::encode::encode;
use solabi::keccak::v256;
use solana_sdk::{
    hash::Hash, // For Hash::new_unique()
    instruction::{AccountMeta, Instruction},
    message::Message as SolanaMessage, // Rename to avoid conflict
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

/// Signs a byte slice message using a Solana keypair.
///
/// This function simulates the signing process expected by some off-chain systems
/// (like Orderly's registration/withdrawal) where a message needs to be signed
/// as if it were part of a Solana transaction.
///
/// # Arguments
///
/// * `message_bytes` - The raw bytes of the message to sign.
/// * `keypair` - The Solana keypair to sign with.
///
/// # Returns
///
/// A `Result` containing the Base58 encoded signature string or an `OrderlyError`.
pub fn sign_solana_message(message_bytes: &[u8], keypair: &Keypair) -> Result<String> {
    // Create a simple Memo instruction containing the message bytes.
    // The program ID for the Memo program is fixed.
    // We only need one account meta: the signer's pubkey.
    let memo_instruction = Instruction {
        program_id: spl_memo::id(), // Use the official spl_memo program ID
        accounts: vec![AccountMeta::new(keypair.pubkey(), true)], // Signer is writable and signer
        data: message_bytes.to_vec(),
    };

    // Create a Solana message containing just this instruction.
    // Use the keypair's pubkey as the fee payer and a dummy blockhash.
    let message = SolanaMessage::new(
        &[memo_instruction],
        Some(&keypair.pubkey()), // Fee payer is the signer
    );

    // Create a transaction from the message.
    // We need a recent blockhash for a real transaction, but for just signing
    // the message content as required by Orderly, a dummy hash might suffice.
    // However, using a real (but potentially old/invalid) hash is safer.
    // Hash::new_unique() provides a unique hash each time.
    let blockhash = Hash::new_unique(); // Use a unique dummy hash
    let mut transaction = Transaction::new_unsigned(message);

    // Sign the transaction with the keypair.
    // This will produce the signature over the transaction message content.
    transaction.try_sign(&[keypair], blockhash).map_err(|e| {
        OrderlyError::SigningError(format!("Failed to sign Solana transaction: {}", e))
    })?;

    // Get the first signature (since we only signed with one keypair).
    let signature: Signature = transaction.signatures.get(0).cloned().ok_or_else(|| {
        OrderlyError::SigningError("Transaction signing produced no signature".to_string())
    })?;

    // Return the signature encoded as a Base58 string.
    Ok(signature.to_string())
}

/// Prepares and signs a withdrawal message for Orderly Network (Solana).
///
/// This fetches the withdrawal nonce, builds the message, hashes and signs it.
/// Returns the WithdrawalMessage and the base58 signature string.
pub async fn prepare_withdrawal_message(
    service: &OrderlyService,
    creds: &Credentials<'_>,
    solana_config: &SolanaConfig,
    user_keypair: &Keypair,
    receiver_addr: &str,
    token: &str,
    amount: u64,
) -> crate::error::Result<(crate::eth::abi::WithdrawalMessage, String)> {
    // 1. Fetch nonce
    let withdraw_nonce = service.get_withdraw_nonce(creds).await?;
    // 2. Get timestamp
    let timestamp = crate::auth::get_timestamp_ms()?;
    // 3. Create message
    let message = create_withdrawal_message(
        &solana_config.broker_id,
        solana_config.orderly_solana_chain_id,
        receiver_addr,
        token,
        amount,
        withdraw_nonce,
        timestamp,
    )?;
    // 4. Encode and hash
    let encoded = encode(&message);
    let hash = v256(&encoded);
    // 5. Sign
    let signature = sign_solana_message(&hash, user_keypair)?;
    Ok((message, signature))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signer::keypair::Keypair;

    #[test]
    fn test_sign_solana_message_basic() {
        let keypair = Keypair::new();
        let message = b"Hello, Orderly!";

        let result = sign_solana_message(message, &keypair);

        assert!(result.is_ok());
        let signature_str = result.unwrap();

        // Basic validation: check if it's a valid base58 string of expected length
        assert!(bs58::decode(&signature_str).into_vec().is_ok());
        // Solana signatures are typically 64 bytes, resulting in Base58 strings
        // of varying lengths, usually around 86-88 chars.
        assert!(signature_str.len() > 80 && signature_str.len() < 90);

        println!("Message: {:?}", message);
        println!("Public Key: {}", keypair.pubkey());
        println!("Signature: {}", signature_str);
    }

    #[test]
    fn test_sign_solana_message_empty() {
        let keypair = Keypair::new();
        let message = b""; // Empty message

        let result = sign_solana_message(message, &keypair);
        assert!(result.is_ok());
        let signature_str = result.unwrap();
        assert!(bs58::decode(&signature_str).into_vec().is_ok());
        assert!(signature_str.len() > 80 && signature_str.len() < 90);
    }

    #[test]
    fn test_sign_solana_message_different_keys() {
        let keypair1 = Keypair::new();
        let keypair2 = Keypair::new();
        let message = b"Sign this message";

        let sig1 = sign_solana_message(message, &keypair1).unwrap();
        let sig2 = sign_solana_message(message, &keypair2).unwrap();

        assert_ne!(sig1, sig2); // Signatures from different keys should be different
    }
    #[test]
    fn test_sign_solana_message_different_messages() {
        let keypair = Keypair::new();
        let message1 = b"Message One";
        let message2 = b"Message Two";

        let sig1 = sign_solana_message(message1, &keypair).unwrap();
        let sig2 = sign_solana_message(message2, &keypair).unwrap();

        assert_ne!(sig1, sig2); // Signatures for different messages should be different
    }
}
