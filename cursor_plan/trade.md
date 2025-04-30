# Solana Deposit, Withdrawal, and Key Registration Implementation Plan

**Current Blockers & TODOs (Summary):**

- ~~**Analyze JS (`helper.ts`):**~~ **(DONE)**
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
  - **Empty PDA Seeds:** Confirm the unusual empty seeds (`[]`) for some LayerZero PDAs are correct and handled appropriately. **(TODO)**
  - **`dst_eid` Flexibility:** Confirm if `LAYERZERO_SOLANA_MAINNET_EID` (30109) needs to be configurable for testnets. **(TODO)**
  - **Orderly Account ID Format:** Confirm hex string is the standard input format. **(TODO)**
- **Implementation TODOs:**
  - Implement all PDA derivation functions in Rust (`pdas.rs`), noting empty seeds.
  - Implement `prepare_solana_deposit_tx` using derived info (including loading IDL via `include_str!`).
  - Implement EIP 712 preparation functions (`prepare_withdrawal_message`) based on **ASSUMPTIONS** if official specs remain unavailable (document assumptions clearly).
- Implement account registration functionality (`register_solana_account`) following the provided example. (COMPLETED)

---

This document outlines the steps to implement Solana deposit, withdrawal, and key registration functionality within the `orderly-connector-rs` SDK.

## 1. Project Setup & Dependencies (COMPLETED)

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

## 2. `solabi` ABI Implementation (`src/eth/abi.rs`) (COMPLETED)

- **Message Encoding Types:**

  ```rust
  use solabi::ethprim::{Address, B256, Bytes, U256};
  use solabi::keccak::v256;  // Use the canonical Keccak-256 implementation
  use solabi::{encode, decode, DecodeError, Decoder, Encode, Encoder, Size};
  use crate::error::OrderlyError;
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
      let broker_id_hash = B256::from(v256(broker_id.as_bytes())); // Use solabi's canonical Keccak-256
      let token_hash = B256::from(v256(token.as_bytes()));       // Use solabi's canonical Keccak-256
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
      let broker_id_hash = B256::from(v256(broker_id.as_bytes())); // Use solabi's canonical Keccak-256

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
  use solabi::keccak::v256; // Use the canonical Keccak-256 implementation

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
  let message_hash = v256(&encoded_message); // Returns [u8; 32]
  let signature = sign_solana_message(&message_hash, keypair)?;

  // In register_solana_account:
  let message = create_registration_message(
      &config.broker_id,
      config.orderly_solana_chain_id,
      timestamp,
      registration_nonce,
  )?;
  let encoded_message = solabi::encode(&message);
  let message_hash = v256(&encoded_message); // Returns [u8; 32]
  let signature = sign_solana_message(&message_hash, keypair)?;
  ```

- **Key Points:**
  - Uses `solabi::ethprim` types like `B256`, `U256`.
  - Uses `solabi::keccak::v256` for all Keccak-256 hashing operations.
  - Implements `solabi::Encode` trait for `WithdrawalMessage` and `RegistrationMessage` to define the exact encoding sequence.
  - Uses `solabi::encode(&message)` to perform the ABI encoding.
  - Provides type-safe message creation functions.
  - Converts the Solana receiver `Pubkey` to `B256` for ABI encoding within the `create_withdrawal_message` function.
  - The `v256` function returns a `[u8; 32]` which can be directly used for signing or converted to `B256` for ABI encoding.

## 3. Core Solana Utilities (`src/solana/utils.rs`, `src/solana/pdas.rs`)

## 4. API Client Enhancements (`src/rest/client.rs` or similar) (COMPLETED)

## 5. Solana Deposit Implementation (`src/solana/client.rs`)

## 6. Off-Chain Message Preparation (Withdrawal & Account Registration) (`src/models.rs`, `src/solana/signing.rs`)

- **Serializable Structs:**
  - **(Structs previously defined are suitable) - DONE**
- **Implement Solana Signing Utility (`src/solana/signing.rs`):**
  - Create a helper function `sign_solana_message(message_bytes: &[u8], keypair: &Keypair) -> Result<String, OrderlyError>` - **DONE**
- **Implement `prepare_withdrawal_message` (likely within `src/rest/client.rs` or called from it):**
  - **(Implementation TODO - Assumptions need verification)**
- Implement `register_solana_account` (within `src/rest/client.rs`): (COMPLETED)
  - Check Registration Status: (COMPLETED)
  - Get Registration Nonce: (COMPLETED)
  - Message Fields (Confirmed by API Docs): (COMPLETED)
  - Hashing/Encoding (Based on Example): (COMPLETED)
  - Signing (Based on Example): (COMPLETED)
  - Submit Registration: (COMPLETED)
  - Return the account ID from the response. (COMPLETED)
  - Documentation: Add clear instructions that users need to create their API keys through their broker's website after registration. (COMPLETED - via code comments)

## 7. Detailed Implementation & Testing (NEXT STEPS)

### 7.1 Account Registration (`src/rest/client.rs`) (COMPLETED)

- Objective: Implement the full flow for registering a Solana account with Orderly via the REST API. (COMPLETED)
- Steps: (COMPLETED)
  1.  Add API Types: (COMPLETED)
  2.  Implement `register_solana_account` method in `OrderlyService`: (COMPLETED)
      - Check Registration (COMPLETED)
      - Get Nonce (COMPLETED)
      - Prepare Message (COMPLETED)
      - Sign Message (COMPLETED)
      - Submit Registration (COMPLETED)
      - Return (COMPLETED)
- Testing: (COMPLETED)
  - Unit tests for registration message creation and signing logic (COMPLETED)
  - Integration tests (using mock server) (COMPLETED)

### 7.2 Solana Deposit (`src/solana/client.rs`)

- **Objective:** Finalize and test the function that prepares the Solana transaction for depositing USDC into the Orderly Vault.
- **Steps:**
  1.  **Review/Refine `prepare_solana_deposit_tx`:** Ensure it correctly:
      - Takes necessary parameters (`SolanaConfig`, user `Keypair`, deposit `amount`, `orderly_account_id` as hex string).
      - Derives all required PDAs using functions from `src/solana/pdas.rs`.
      - Loads the Vault IDL using `include_str!`.
      - Constructs the `DepositParams` struct.
      - Assembles the `remaining_accounts` correctly based on LayerZero requirements.
      - Builds the Anchor instruction using `program.request()`.
      - Creates and partially signs the Solana `Transaction`.
  2.  **Handle `orderly_account_id`:** Ensure the hex string input is correctly converted to `[u8; 32]`.
- **Testing:**
  - Add unit tests verifying the construction of the `DepositParams` and `remaining_accounts`.
  - Add tests simulating the transaction creation process (mocking RPC calls if necessary).
  - Add integration tests that create the transaction, sign it fully, and potentially submit it to a local validator or testnet.

### 7.3 Withdrawal (`src/rest/client.rs`)

- **Objective:** Implement the full flow for requesting a withdrawal from Orderly via the REST API.
- **Steps:**
  1.  **Add API Types:** Define necessary request/response structs for:
      - `GET /v1/withdraw_nonce` (Get nonce)
      - `POST /v1/withdraw_request` (Submit withdrawal request)
  2.  **Implement `prepare_withdrawal_message` (or similar name) method in `OrderlyService`:**
      - Takes necessary parameters (e.g., `SolanaConfig`, `Keypair`, `receiver` address, `token`, `amount`).
      - **Fetch Nonce:** Call `GET /v1/withdraw_nonce` to obtain the `withdrawNonce`.
      - **Prepare Message:**
        - Get current `timestamp`.
        - Use `create_withdrawal_message` from `src/eth/abi.rs` to construct the `WithdrawalMessage`.
        - ABI-encode the message using `solabi::encode`.
        - Hash the encoded message using `solabi::keccak::v256`.
      - **Sign Message:**
        - Use the `sign_solana_message` utility (from `src/solana/signing.rs`) with the message hash and the user's `Keypair` to get the signature.
      - **Return:** A struct containing the original message components (required for the API call) and the calculated signature.
  3.  **Implement `request_withdrawal` method in `OrderlyService` (if not already suitable):**
      - Takes the prepared message components and signature.
      - Calls `POST /v1/withdraw_request` with the correct body structure.
      - Handles the API response (success or error).
- **Testing:**
  - Add unit tests for the withdrawal message creation and signing logic (if not already covered by `abi.rs` tests).
  - Add integration tests (potentially using a mock server or against the testnet) to verify the full API flow (`get_nonce` -> prepare -> `request_withdrawal`).

## 8. Final Steps

- **Address Remaining TODOs:** Confirm details about empty PDA seeds, EID configurability, and account ID format.
- **Refine Error Handling:** Ensure robust error handling throughout all flows.
- **Add Examples:** Create clear examples demonstrating how to use the registration, deposit, and withdrawal functions.
- **Update Documentation:** Finalize all documentation for the SDK.

---

## **(Original Sections 3-6 details omitted for brevity)**

## 9. Testing (Overall Strategy)

- **Unit Tests:** Focus on isolated logic like ABI encoding, PDA derivation, message creation, and signing utilities.
- **Integration Tests:**
  - Test interactions with the Orderly REST API (use mock server or testnet).
  - Test Solana transaction building and signing.
  - Potentially test transaction submission against a local validator or testnet.
- **End-to-End Tests:** (Optional but recommended) Simulate a full user flow: Register -> Deposit -> Withdraw.
