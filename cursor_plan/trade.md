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
  - Find exact EIP 712 domain & type definitions for `AddOrderlyKey` message signing. **(TODO - BLOCKER for Key Reg)**
- **Points Requiring Caution/Confirmation During Implementation:**
  - **Empty PDA Seeds:** Confirm the unusual empty seeds (`[]`) for some LayerZero PDAs are correct and handled appropriately.
  - **`dst_eid` Flexibility:** Confirm if `LAYERZERO_SOLANA_MAINNET_EID` (30109) needs to be configurable for testnets.
  - **Orderly Account ID Format:** Confirm hex string is the standard input format.
- **Implementation TODOs:**
  - Implement all PDA derivation functions in Rust (`pdas.rs`), noting empty seeds.
  - Implement `prepare_solana_deposit_tx` using derived info (including loading IDL via `include_str!`).
  - Implement EIP 712 preparation functions (`prepare_withdrawal_message`, `prepare_register_orderly_key_message`) based on **ASSUMPTIONS** if official specs remain unavailable (document assumptions clearly).

---

This document outlines the steps to implement Solana deposit, withdrawal, and key registration functionality within the `orderly-connector-rs` SDK.

## 1. Project Setup & Dependencies

- **File Structure:** (As previously defined)
- **Dependencies (`Cargo.toml`):** (As previously defined)
- **Configuration (`src/solana/types.rs`):**

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

## 2. Core Solana Utilities (`src/solana/utils.rs`, `src/solana/pdas.rs`)

- **Hashing (`src/solana/utils.rs`):**

  ```rust
  use ethers_core::utils::keccak256;

  /// Calculates the Keccak256 hash, matching common JS libraries.
  pub fn keccak256_hash(data: &[u8]) -> [u8; 32] {
      keccak256(data)
  }
  // ... hash_broker_id, hash_token_id ...
  ```

  - **Note:** Using `ethers-core` `keccak256` is preferred for exact JS compatibility.

- **PDA Derivation (`src/solana/pdas.rs`):**
  - **(Implementation TODO)** Implement all functions previously listed.
  - **Caution:** For functions like `find_endpoint_setting_pda`, `find_uln_setting_pda`, `find_price_feed_pda` which use empty seeds (`[]`), ensure the implementation correctly handles `Pubkey::find_program_address(&[], &program_id)`. Add comments noting the unusual nature and reliance on JS analysis.

## 3. API Client Enhancements (`src/rest/client.rs` or similar)

- **(DONE - Code outlined previously is sufficient)**

## 4. Solana Deposit Implementation (`src/solana/client.rs`)

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

## 5. Off-Chain Message Preparation (Withdrawal & Key Reg) (`src/models.rs`, `src/solana/signing.rs`)

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
    - Hash `brokerId` using `solidityPackedKeccak256` -> `bytes32`.
    - Hash `token` string using `solidityPackedKeccak256` -> `bytes32`.
    - Convert `receiver` pubkey string to its 32 bytes -> `bytes32`.
    - **Assumed ABI Types:** `["bytes32", "uint256", "bytes32", "bytes32", "uint256", "uint256", "uint256"]`
    - **Assumed ABI Order:** `[hash(brokerId), chainId, bytes32(receiver_pubkey), hash(token), amount, withdrawNonce, timestamp]`
    - ABI-encode these values using `ethers::abi::encode` (or equivalent Rust implementation).
    - Calculate the Keccak256 hash of the encoded bytes.
  - **Signing (Assumption based on Registration Gist):**
    - Convert the final hash bytes to a hex string.
    - Encode this _hex string_ using a `TextEncoder` equivalent (e.g., `hash_hex.as_bytes()`).
    - Call the `sign_solana_message` utility (from `src/solana/signing.rs`) with these TextEncoder-bytes and the user's keypair.
  - **Return:** The original message components (as needed by the `POST /v1/withdraw_request` API body) and the hex-encoded signature.
  - **Documentation:** Clearly document the assumptions made about ABI encoding order/types and the `TextEncoder` step.
- **Implement `prepare_register_orderly_key_message`:**
  - **(Implementation TODO - BLOCKED by missing message structure)**
  - Need API documentation or an example for the specific fields, types, order, and nonce requirements for key registration on Solana.
  - Follow the same hashing/encoding/signing pattern once the structure is known.

## 6. Verification and Refinement

- **(Status updated, Withdrawal structure largely known, KeyReg structure pending)**
- **Need to confirm:** The **exact ABI encoding types and order** assumed for the Solana withdrawal message (hash(brokerId), chainId, bytes32(receiver), hash(token), amount, nonce, timestamp).
- **Need to confirm:** Whether the final hash's hex representation needs to be **`TextEncoder`-encoded** before signing via the Memo transaction for Solana withdrawals (currently assumed YES based on registration Gist).
- **Need to find:** The **exact message structure** (fields, types, order, nonce source) for the `AddOrderlyKey` operation on Solana.

## 7. Testing

- **(Plan remains the same)**
