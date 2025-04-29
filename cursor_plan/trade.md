# Solana Deposit, Withdrawal, and Key Registration Implementation Plan

**Current Blockers & TODOs (Summary):**

- ~~**Analyze JS (`helper.ts`):**~~
  - ~~Verify exact seeds & args for ALL PDA derivations (Vault, LZ/Endpoint related).~~ **(DONE)**
  - ~~Verify exact structure, order, and keys for `.remainingAccounts` in the `deposit` instruction.~~ **(DONE)**
  - ~~Verify Address Lookup Table (ALT) derivation logic (`getLookupTableAddress`).~~ **(DONE)**
  - ~~Extract all external Program IDs (Endpoint, SendLib, Treasury, etc.).~~ **(DONE)**
  - ~~Determine source/value of LayerZero Destination EID (`dst_eid`).~~ **(DONE - Hardcoded 30109)**
- **Clarify with Orderly Specs/Docs:**
  - ~~Is Orderly Account ID (`[u8; 32]`) needed for on-chain `DepositParams`, and how to get it?~~ **(CONFIRMED - Needed. SDK function requires it as input.)**
  - Find exact EIP 712 domain & type definitions for `Withdraw` message signing. **(TODO - BLOCKER for Withdrawal)**
  - ~~Find exact EIP 712 domain & type definitions for `AddOrderlyKey` message signing.~~ **(REMOVED - Not needed. API keys are created through broker's website)**
- **Points Requiring Caution/Confirmation During Implementation:**
  - **Empty PDA Seeds:** Confirm the unusual empty seeds (`[]`) for some LayerZero PDAs are correct and handled appropriately.
  - **`dst_eid` Flexibility:** Confirm if `LAYERZERO_SOLANA_MAINNET_EID` (30109) needs to be configurable for testnets.
  - **Orderly Account ID Format:** Confirm hex string is the standard input format.
- **Implementation TODOs:**
  - Implement all PDA derivation functions in Rust (`pdas.rs`), noting empty seeds.
  - Implement `prepare_solana_deposit_tx` using derived info (including loading IDL via `include_str!`).
  - Implement EIP 712 preparation functions (`prepare_withdrawal_message`) based on **ASSUMPTIONS** if official specs remain unavailable (document assumptions clearly).
  - Implement account registration functionality (`register_solana_account`) following the provided example.

---

This document outlines the steps to implement Solana deposit, withdrawal, and key registration functionality within the `orderly-connector-rs` SDK.

## 1. Project Setup & Dependencies

- **File Structure:** (As previously defined)
- **Dependencies (`Cargo.toml`):** (As previously defined)
- **Configuration (`src/solana/types.rs`):**

  - **Program ID Access:**

    - Program IDs are now accessed at runtime using a function, not as `const` values or via `declare_id!` macros. This avoids macro conflicts and Rust's `const` restrictions for `Pubkey::from_str`.
    - Use the following function to get a program ID by name:

    ```rust
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
    ```

    - **Usage Example:**
      ```rust
      let vault_id = get_program_id("VAULT").unwrap();
      ```
    - This approach is necessary because `Pubkey::from_str` is not a `const fn` and cannot be used in `const` or `static` initializers.
    - Remove all references to `declare_id!` and static `const` program ID values from the codebase and documentation.

  - Define `SolanaConfig` struct and constants:

    ```rust
    use solana_sdk::pubkey::Pubkey;
    use solana_program::declare_id;
    use std::str::FromStr;

    // Define Program ID constants (Verified from JS helper.ts)
    pub mod program_ids {
        use super::*;
        declare_id!("ErBmAD61mGFKvrFNaTJuxoPwqrS8GgtwtqJTJVjFWx9Q");
        pub const VAULT_PROGRAM_ID: Pubkey = PUBKEY;

        declare_id!("LzV2EndpointV211111111111111111111111111111");
        pub const ENDPOINT_PROGRAM_ID: Pubkey = PUBKEY;

        declare_id!("LzV2SendLib11111111111111111111111111111111");
        pub const SEND_LIB_PROGRAM_ID: Pubkey = PUBKEY;

        declare_id!("LzV2Treasury1111111111111111111111111111111");
        pub const TREASURY_PROGRAM_ID: Pubkey = PUBKEY;

        declare_id!("LzV2Executor1111111111111111111111111111111");
        pub const EXECUTOR_PROGRAM_ID: Pubkey = PUBKEY;

        declare_id!("LzV2PriceFeed111111111111111111111111111111");
        pub const PRICE_FEED_PROGRAM_ID: Pubkey = PUBKEY;

        declare_id!("LzV2DVN111111111111111111111111111111111111");
        pub const DVN_PROGRAM_ID: Pubkey = PUBKEY;
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
    ```

- **Error Handling (`src/error.rs`):** (As previously defined)

## 2. `solabi` ABI Implementation (`src/solana/abi.rs`)

- **Message Encoding Types:**

  ```rust
  use solabi::ethprim::{Address, B256, Bytes, U256, keccak256};
  use solabi::{encode, decode, DecodeError, Decoder, Encode, Encoder, Size};
  use crate::error::OrderlyError;
  // No longer need a separate keccak helper, use solabi::ethprim::keccak256 directly
  // use crate::solana::utils::keccak256_hash;
  use solana_sdk::pubkey::Pubkey;
  use std::str::FromStr;

  // Define ABI types for withdrawal message
  #[derive(Debug, Clone, PartialEq)]
  pub struct WithdrawalMessage {
      pub broker_id_hash: B256,      // bytes32
      pub chain_id: U256,            // uint256
      pub receiver: B256,            // bytes32 // Store as B256 for direct ABI encoding
      pub token_hash: B256,          // bytes32
      pub amount: U256,              // uint256
      pub withdraw_nonce: U256,      // uint256
      pub timestamp: U256,           // uint256
  }

  // Define ABI types for registration message
  #[derive(Debug, Clone, PartialEq)]
  pub struct RegistrationMessage {
      pub broker_id_hash: B256,      // bytes32
      pub chain_id: U256,            // uint256
      pub timestamp: U256,           // uint256
      pub registration_nonce: U256,  // uint256
  }

  // Implement solabi::Encode for WithdrawalMessage
  // Encoding order: broker_id_hash, chain_id, receiver, token_hash, amount, withdraw_nonce, timestamp
  impl Encode for WithdrawalMessage {
      fn size(&self) -> Size {
          Size::Fixed(32 * 4 + 32 + 32 + 32) // 4x bytes32, 3x uint256 (encoded as 32 bytes)
      }

      fn encode(&self, encoder: &mut Encoder) {
          self.broker_id_hash.encode(encoder);
          self.chain_id.encode(encoder);
          self.receiver.encode(encoder);
          self.token_hash.encode(encoder);
          self.amount.encode(encoder);
          self.withdraw_nonce.encode(encoder);
          self.timestamp.encode(encoder);
      }
  }

  // Implement solabi::Encode for RegistrationMessage
  // Encoding order: broker_id_hash, chain_id, timestamp, registration_nonce
  impl Encode for RegistrationMessage {
      fn size(&self) -> Size {
          Size::Fixed(32 * 1 + 32 + 32 + 32) // 1x bytes32, 3x uint256 (encoded as 32 bytes)
      }

      fn encode(&self, encoder: &mut Encoder) {
          self.broker_id_hash.encode(encoder);
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
      let broker_id_hash = keccak256(broker_id.as_bytes()); // Use solabi's keccak
      let token_hash = keccak256(token.as_bytes());       // Use solabi's keccak
      let receiver_pubkey = Pubkey::from_str(receiver_address_str)
          .map_err(|e| OrderlyError::ValidationError(format!("Invalid receiver pubkey string: {}", e)))?;
      let receiver_bytes = receiver_pubkey.to_bytes();
      let receiver = B256::from_slice(&receiver_bytes); // Convert Solana pubkey bytes to B256

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
      let broker_id_hash = keccak256(broker_id.as_bytes()); // Use solabi's keccak

      Ok(RegistrationMessage {
          broker_id_hash,
          chain_id: U256::from(chain_id),
          timestamp: U256::from(timestamp),
          registration_nonce: U256::from(registration_nonce),
      })
  }
  ```

- **Usage in Message Preparation:**

  ```rust
  use solabi::ethprim::keccak256; // Make sure keccak256 is in scope

  // In prepare_withdrawal_message:
  let message = create_withdrawal_message(
      &config.broker_id,
      config.orderly_solana_chain_id,
      &receiver, // Solana address string
      &token,
      amount,
      withdraw_nonce,
      timestamp,
  )?;
  let encoded_message = solabi::encode(&message);
  let message_hash = keccak256(&encoded_message); // Use solabi::ethprim::keccak256
  let signature = sign_solana_message(&message_hash.0, keypair)?;

  // In register_solana_account:
  let message = create_registration_message(
      &config.broker_id,
      config.orderly_solana_chain_id,
      timestamp,
      registration_nonce,
  )?;
  let encoded_message = solabi::encode(&message);
  let message_hash = keccak256(&encoded_message); // Use solabi::ethprim::keccak256
  let signature = sign_solana_message(&message_hash.0, keypair)?;
  ```

- **Key Points:**
  - Uses `solabi::ethprim` types like `B256`, `U256`.
  - Implements `solabi::Encode` trait for `WithdrawalMessage` and `RegistrationMessage` to define the exact encoding sequence.
  - Uses `solabi::encode(&message)` to perform the ABI encoding.
  - Provides type-safe message creation functions.
  - Uses `solabi::ethprim::keccak256` for all necessary hashing (broker ID, token ID, final encoded message). **Crucially relies on this implementation matching the standard expected by Orderly's backend.**
  - Converts the Solana receiver `Pubkey` to `B256` for ABI encoding within the `create_withdrawal_message` function.

## 3. Core Solana Utilities (`src/solana/utils.rs`, `src/solana/pdas.rs`)

- **Hashing (`src/solana/utils.rs`):**
  - _Remove the `keccak256_hash` helper function as it's no longer needed. Use `solabi::ethprim::keccak256` directly where hashing is required._

## 4. API Client Enhancements (`src/rest/client.rs` or similar)

- **(DONE - Code outlined previously is sufficient)**

## 5. Solana Deposit Implementation (`src/solana/client.rs`)

- **Argument/Account Structs (`src/solana/types.rs`):**
  - **(DONE - Structs previously defined based on IDL are correct)**
- **Main Function (`src/solana/client.rs`):**
  - **(Implementation TODO)** Implement the function as outlined previously.
  - **Assumption:** `orderly_account_id` is provided as a valid hex string argument.
  - **IDL Loading:** Load the IDL JSON string using `include_str!("idl/solana_vault.json")` (assuming the IDL file is placed there) when initializing the `anchor_client::Program`.
  ```rust
  // Inside prepare_solana_deposit_tx
  const IDL_JSON: &str = include_str!("idl/solana_vault.json");
  let idl: anchor_client::idl::Idl = serde_json::from_str(IDL_JSON)
      .map_err(|e| OrderlyError::InvalidConfiguration(format!("Failed to parse IDL: {}", e)))?;
  let anchor_program = anchor_client.program_with_idl(program_ids::VAULT_PROGRAM_ID, idl)?;
  // ... rest of function ...
  ```

## 6. Off-Chain Message Preparation (Withdrawal & Account Registration) (`src/models.rs`, `src/solana/signing.rs`)

- **Serializable Structs:**
  - **(Structs previously defined are suitable)**
- **Implement Solana Signing Utility (`src/solana/signing.rs`):**
  - Create a helper function `sign_solana_message(message_bytes: &[u8], keypair: &Keypair) -> Result<String, OrderlyError>` that:
    - Takes the final message bytes to be signed (e.g., the Keccak256 hash, potentially TextEncoded as per the Gist).
    - Creates a Solana transaction with a Memo instruction containing these bytes.
    - Sets fee payer and a dummy blockhash.
    - Signs the transaction using the provided keypair.
    - Returns the hex-encoded transaction signature.
    - This encapsulates the logic from the Gist's `signMessage` function.
- **Implement `prepare_withdrawal_message` (likely within `src/rest/client.rs` or called from it):**
  - **(Implementation TODO - Assumptions need verification)**
  - **Fetch Nonce:** First, call the `GET /v1/withdraw_nonce` endpoint (implemented in `src/rest/client.rs`) to get the `withdrawNonce`.
  - **Message Fields (Confirmed by API Docs):**
    - `brokerId`: string (from config)
    - `chainId`: integer (Solana chain ID, e.g., 900900900, from config)
    - `receiver`: string (Recipient Solana address)
    - `token`: string (e.g., "USDC")
    - `amount`: number (Withdrawal amount)
    - `withdrawNonce`: string (From API)
    - `timestamp`: string (Unix ms timestamp)
  - **Hashing/Encoding (Assumptions based on Registration Gist & API fields):**
    - Hash `brokerId` using `solabi::ethprim::keccak256` -> `B256`.
    - Hash `token` string using `solabi::ethprim::keccak256` -> `B256`.
    - Convert `receiver` pubkey string to its 32 bytes -> `B256`.
    - Create `WithdrawalMessage` struct using `create_withdrawal_message`.
    - ABI-encode the struct using `solabi::encode`.
    - Calculate the Keccak256 hash of the encoded bytes using `solabi::ethprim::keccak256` -> `B256`.
  - **Signing (Assumption based on Registration Gist):**
    - Get the byte slice from the final hash: `message_hash.0`.
    - Encode this _byte slice_ using a `TextEncoder` equivalent (e.g., `hash_hex.as_bytes()`).
    - Call the `sign_solana_message` utility (from `src/solana/signing.rs`) with these TextEncoder-bytes and the user's keypair.
  - **Return:** The original message components (as needed by the `POST /v1/withdraw_request` API body) and the hex-encoded signature.
  - **Documentation:** Clearly document the assumptions made about ABI encoding order/types and the `TextEncoder` step.
- **Implement `register_solana_account` (within `src/rest/client.rs`):**
  - **(Implementation TODO - Based on provided example)**
  - **Check Registration Status:**
    - Call `GET /v1/public/wallet_registered` to check if the wallet is already registered.
  - **Get Registration Nonce:**
    - Call `GET /v1/registration_nonce` to get a unique nonce for registration.
  - **Message Fields (Confirmed by API Docs):**
    - `brokerId`: string (from config)
    - `chainId`: integer (Solana chain ID, e.g., 900900900, from config)
    - `chainType`: string ("SOL")
    - `timestamp`: string (Unix ms timestamp)
    - `registrationNonce`: string (From API)
  - **Hashing/Encoding (Based on Example):**
    - Hash `brokerId` using `solabi::ethprim::keccak256` -> `B256`.
    - Create `RegistrationMessage` struct using `create_registration_message`.
    - ABI-encode the struct using `solabi::encode`.
    - Calculate the Keccak256 hash of the encoded bytes using `solabi::ethprim::keccak256` -> `B256`.
  - **Signing (Based on Example):**
    - Get the byte slice from the final hash: `message_hash.0`.
    - Encode this _byte slice_ using UTF-8 bytes.
    - Call the `sign_solana_message` utility with these bytes and the user's keypair.
  - **Submit Registration:**
    - Call `POST /v1/register_account` with the message, signature, and user address.
    - Return the account ID from the response.
  - **Documentation:** Add clear instructions that users need to create their API keys through their broker's website after registration.

## 7. Testing

- **(Plan remains the same)**
