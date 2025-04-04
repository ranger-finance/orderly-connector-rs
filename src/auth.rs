use crate::error::{OrderlyError, Result};
use base64::Engine;
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};

/// Gets the current UTC timestamp in milliseconds since the Unix epoch.
///
/// This function is used for generating timestamps for API requests and signatures.
///
/// # Returns
///
/// A `Result` containing the current timestamp in milliseconds or an error if
/// the system time cannot be retrieved.
///
/// # Examples
///
/// ```no_run
/// use orderly_connector_rs::auth::get_timestamp_ms;
///
/// let timestamp = get_timestamp_ms().expect("Failed to get timestamp");
/// println!("Current timestamp: {}", timestamp);
/// ```
pub fn get_timestamp_ms() -> Result<u64> {
    let now = Utc::now();
    Ok(now.timestamp_millis() as u64)
}

/// Parses the Orderly secret key string into its raw byte representation.
///
/// The secret key string should be in the format "ed25519:<base58_private_key>".
///
/// # Arguments
///
/// * `secret_key_str` - The secret key string to parse
///
/// # Returns
///
/// A `Result` containing the 32-byte private key or an error if parsing fails.
///
/// # Errors
///
/// Returns an error if:
/// * The secret key doesn't start with "ed25519:"
/// * The base58 decoding fails
/// * The decoded key is not exactly 32 bytes
///
/// # Examples
///
/// ```no_run
/// use orderly_connector_rs::auth::parse_secret_key;
///
/// let secret_key = "ed25519:your_base58_private_key";
/// let key_bytes = parse_secret_key(secret_key).expect("Failed to parse secret key");
/// ```
fn parse_secret_key(secret_key_str: &str) -> Result<[u8; 32]> {
    if !secret_key_str.starts_with("ed25519:") {
        return Err(OrderlyError::AuthenticationError(
            "Invalid secret key format: Missing 'ed25519:' prefix".to_string(),
        ));
    }
    let base58_key = &secret_key_str[8..];
    let decoded_bytes = bs58::decode(base58_key).into_vec().map_err(|e| {
        OrderlyError::AuthenticationError(format!("Failed to decode base58 secret key: {}", e))
    })?;

    if decoded_bytes.len() != 32 {
        return Err(OrderlyError::AuthenticationError(format!(
            "Invalid secret key length: Expected 32 bytes, got {}",
            decoded_bytes.len()
        )));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&decoded_bytes);
    Ok(key_bytes)
}

/// Generates an Ed25519 signature for a given message using the Orderly secret key.
///
/// This function is used to sign API requests for authentication.
///
/// # Arguments
///
/// * `orderly_secret` - The Orderly secret key string (e.g., "ed25519:...")
/// * `message` - The message string to sign (typically timestamp + method + path + body)
///
/// # Returns
///
/// A `Result` containing the Base64 encoded signature string or an error if signing fails.
///
/// # Examples
///
/// ```no_run
/// use orderly_connector_rs::auth::generate_signature;
///
/// let secret = "ed25519:your_base58_private_key";
/// let message = "your_message_to_sign";
/// let signature = generate_signature(secret, message).expect("Failed to generate signature");
/// println!("Signature: {}", signature);
/// ```
pub fn generate_signature(orderly_secret: &str, message: &str) -> Result<String> {
    let key_bytes = parse_secret_key(orderly_secret)?;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let signature = signing_key.sign(message.as_bytes());
    Ok(base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*; // Import functions from outer module

    // Note: These tests require a valid (but not necessarily real) key pair for testing.
    // Replace with actual test vectors if available, or generate a throwaway pair for testing.
    const TEST_SECRET_KEY_STR: &str = "ed25519:2wyRcTuEuip6nLoNmfKvmkxMgC7zLbW8DH4PPQT5hWKd"; // Example, NOT a real secret
                                                                                              // Corresponding public key (not needed for signing test, but good for context):
                                                                                              // const TEST_PUBLIC_KEY_STR: &str = "ed25519:FyWdUuNubF8LhDbV34qzV7uXRU51jBLN8YsnK9Z8pZbF";

    #[test]
    fn test_get_timestamp_ms_works() {
        let ts = get_timestamp_ms().expect("Failed to get timestamp");
        assert!(ts > 1600000000000); // Ensure it's a reasonable timestamp (post ~Sept 2020)
        println!("Current Timestamp (ms): {}", ts);
    }

    #[test]
    fn test_parse_secret_key_valid() {
        let key_bytes = parse_secret_key(TEST_SECRET_KEY_STR).expect("Failed to parse valid key");
        assert_eq!(key_bytes.len(), 32);
    }

    #[test]
    fn test_parse_secret_key_invalid_prefix() {
        let result = parse_secret_key("invalid:key");
        assert!(matches!(result, Err(OrderlyError::AuthenticationError(_))));
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing 'ed25519:' prefix"));
    }

    #[test]
    fn test_parse_secret_key_invalid_base58() {
        let result = parse_secret_key("ed25519:invalid-base58~");
        assert!(matches!(result, Err(OrderlyError::AuthenticationError(_))));
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to decode base58"));
    }

    #[test]
    fn test_parse_secret_key_invalid_length() {
        // Generate a valid base58 string but with wrong length
        let short_key = bs58::encode(vec![0u8; 31]).into_string();
        let result = parse_secret_key(&format!("ed25519:{}", short_key));
        assert!(matches!(result, Err(OrderlyError::AuthenticationError(_))));
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid secret key length"));
    }

    #[test]
    fn test_generate_signature_works() {
        let timestamp = get_timestamp_ms().unwrap();
        let method = "POST";
        let path = "/v1/order";
        let body = r#"{"symbol": "SPOT_NEAR_USDC", "order_type": "LIMIT", "order_price": 15.23, "order_quantity": 23.11, "side": "BUY"}"#;
        let message_to_sign = format!("{}{}{}{}", timestamp, method, path, body);

        let signature = generate_signature(TEST_SECRET_KEY_STR, &message_to_sign)
            .expect("Failed to generate signature");

        // Basic check: signature should be non-empty base64 string
        assert!(!signature.is_empty());
        // Try decoding base64 to ensure it's valid
        base64::engine::general_purpose::STANDARD
            .decode(&signature)
            .expect("Signature is not valid base64");

        println!("Message: {}", message_to_sign);
        println!("Generated Signature: {}", signature);
        // We can't easily verify the signature without the public key and known test vectors,
        // but we ensure the process runs without errors.
    }
}
