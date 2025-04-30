# Solana Deposit & Withdrawal Implementation Plan

This plan details the step-by-step implementation of Solana deposit and withdrawal functionality for the `orderly-connector-rs` SDK, based on analysis of `trade.md` and the reference JS SDK (`helper.ts`).

**Scope:** This plan covers the creation of PDA derivation utilities, the Solana deposit transaction preparation, and the off-chain message preparation and API interaction for withdrawals. Account registration is considered complete and out of scope for this specific plan.

---

## Phase 1: Prerequisites & Setup

**Goal:** Ensure foundational components, configuration, and utilities are in place.

1.  **Configuration (`src/solana/types.rs`):**

    - Verify the `SolanaConfig` struct contains necessary fields: `rpc_url`, `usdc_mint`, `broker_id`, `orderly_solana_chain_id` (for signing).
    - Confirm the `get_program_id(name: &str)` function provides access to all required program IDs (Vault, LayerZero Endpoint, SendLib, Treasury, Executor, PriceFeed, DVN).
    - Confirm the `LAYERZERO_SOLANA_MAINNET_EID` constant (`30109`) is defined. _(Self-Correction: Place EID in `SolanaConfig` if testnet flexibility is desired later, otherwise constant is fine for now)_.

2.  **Core Solana Utilities:**

    - Ensure a basic Solana signing utility exists, e.g., `sign_solana_message(message_bytes: &[u8], keypair: &Keypair) -> Result<String, OrderlyError>` (likely in `src/solana/signing.rs`).
    - Ensure basic error types (`OrderlyError`) are defined (`src/error.rs`).

3.  **ABI Definitions (`src/eth/abi.rs`):**
    - Confirm `WithdrawalMessage` and `RegistrationMessage` structs align with the latest EIP-712 definitions. **(DONE)**
    - Confirm `create_withdrawal_message` and `create_registration_message` functions correctly populate these structs. **(DONE)**
    - Confirm `Encode` implementations for both messages correctly reflect the EIP-712 data hashing structure (hashing strings, etc.). **(DONE)**
    - _(Self-Correction: Resolve linter errors related to `U64` and `Encoder` if they cause build issues later)_.

---

## Phase 2: PDA Derivation Utilities

**Goal:** Implement functions to derive all necessary Program Derived Addresses (PDAs) required for the deposit transaction.

1.  **Create File:** Create `src/solana/pdas.rs`.
2.  **Add Imports:** Include `solana_program::pubkey::{find_program_address, Pubkey}` and necessary constants/types.
3.  **Implement PDA Functions:** Create public functions for each required PDA. Follow seeds identified in `helper.ts` (lines ~152-207 and deposit instruction context lines ~514-566). Ensure correct program IDs are used for derivation (mostly Vault Program ID per JS source).
    - `get_vault_authority_pda(vault_program_id: &Pubkey) -> (Pubkey, u8)`
      - Seed: `[b"vault_authority"]`
    - `get_broker_pda(vault_program_id: &Pubkey, broker_hash: &[u8; 32]) -> (Pubkey, u8)`
      - Seed: `[b"allowed_broker", broker_hash]`
    - `get_token_pda(vault_program_id: &Pubkey, token_hash: &[u8; 32]) -> (Pubkey, u8)`
      - Seed: `[b"allowed_token", token_hash]`
    - `get_oapp_config_pda(vault_program_id: &Pubkey) -> (Pubkey, u8)`
      - Seed: `[b"OAppConfig"]`
    - `get_peer_pda(vault_program_id: &Pubkey, dst_eid: u32) -> (Pubkey, u8)`
      - Seed: `[b"Peer", &dst_eid.to_be_bytes()]`
    - `get_enforced_options_pda(vault_program_id: &Pubkey, dst_eid: u32) -> (Pubkey, u8)`
      - Seed: `[b"Options", &dst_eid.to_be_bytes(), b""]` (Using `b""` for empty options buffer per JS)
    - `get_send_lib_pda() -> Pubkey`
      - _(Returns static LayerZero Program ID)_
    - `get_send_lib_config_pda(oapp_config_pda: &Pubkey, dst_eid: u32) -> (Pubkey, u8)`
      - Seed: `[b"SendLibConfig", &dst_eid.to_be_bytes()]`, derived using `oapp_config_pda` (Needs verification - LayerZero PDAs usually derived using LZ program IDs, but follow JS if specific to Orderly's setup). _(Correction: JS uses `oappConfigPDA` as base for derivation, potentially implying it's the program ID used)_
    - `get_default_send_lib_pda(dst_eid: u32) -> (Pubkey, u8)`
      - Seed: `[b"DefaultSendLib", &dst_eid.to_be_bytes()]`, derived using LZ SendLib Program ID.
    - `get_send_lib_info_pda(send_lib_pda: &Pubkey) -> (Pubkey, u8)`
      - Seed: `[b"SendLibInfo"]`, derived using `send_lib_pda`.
    - `get_endpoint_setting_pda() -> (Pubkey, u8)`
      - Seed: `[b"EndpointSettings"]`, derived using LZ Endpoint Program ID.
    - `get_nonce_pda(vault_program_id: &Pubkey, dst_eid: u32) -> (Pubkey, u8)`
      - Seed: `[b"Nonce", &dst_eid.to_be_bytes()]`, derived using `vault_program_id` (per JS).
    - `get_event_authority_pda() -> Pubkey`
      - _(Returns static LayerZero Endpoint Program ID's authority)_
    - `get_uln_setting_pda() -> (Pubkey, u8)`
      - Seed: `[b"UlnSettings"]`, derived using LZ Endpoint Program ID.
    - `get_send_config_pda(oapp_config_pda: &Pubkey, dst_eid: u32) -> (Pubkey, u8)`
      - Seed: `[b"SendConfig", &dst_eid.to_be_bytes()]`, derived using `oapp_config_pda` (per JS).
    - `get_default_send_config_pda(dst_eid: u32) -> (Pubkey, u8)`
      - Seed: `[b"DefaultSendConfig", &dst_eid.to_be_bytes()]`, derived using LZ Endpoint Program ID.
    - `get_uln_event_authority_pda() -> Pubkey`
      - _(Returns static LayerZero ULN authority)_
    - `get_executor_config_pda() -> (Pubkey, u8)`
      - Seed: `[b"ExecutorConfig"]`, derived using LZ Executor Program ID.
    - `get_price_feed_pda() -> (Pubkey, u8)`
      - Seed: `[b"PriceFeed"]`, derived using LZ Price Feed Program ID.
    - `get_dvn_config_pda() -> (Pubkey, u8)`
      - Seed: `[b"DVNConfig"]`, derived using DVN Program ID.
    - _(Add any other PDAs identified during deposit instruction analysis)_
4.  **Expose Module:** Add `pub mod pdas;` to `src/solana/mod.rs`.
5.  **Unit Tests:** Create `src/solana/pdas.rs` tests to verify PDA derivations against known values from the JS SDK or manual calculation.

---

## Phase 3: Solana Deposit Transaction Preparation

**Goal:** Implement the function that constructs and partially signs the Solana deposit transaction.

1.  **Define Structs (`src/solana/types.rs` or `src/types.rs`):**

    - Ensure `DepositParams` struct matching the Anchor IDL's definition exists.
    - Ensure `SendParam` struct matching the Anchor IDL's definition exists (`nativeFee: u64`, `lzTokenFee: u64`).

2.  **Implement `prepare_solana_deposit_tx` (in `src/solana/client.rs`):**
    - **Function Signature:**
      ```rust
      pub fn prepare_solana_deposit_tx(
          rpc_client: &RpcClient, // Or SolanaClient wrapper
          config: &SolanaConfig,
          user_keypair: &Keypair,
          amount: u64,
          orderly_account_id_hex: &str,
          vault_program_id: &Pubkey, // Pass derived program ID
          // Include other necessary program IDs if not in config
      ) -> Result<VersionedTransaction, OrderlyError>
      ```
    - **Step 1: Input Validation:** Check `amount > 0`. Validate `orderly_account_id_hex` format (32 bytes hex).
    - **Step 2: Prepare Inputs:**
      - Get user pubkey: `user_keypair.pubkey()`.
      - Convert `orderly_account_id_hex` to `[u8; 32]`. Handle potential errors.
      - Calculate `broker_hash = v256(config.broker_id.as_bytes())`.
      - Calculate `token_hash = v256("USDC".as_bytes())` (assuming USDC for now).
      - Define `dst_eid = LAYERZERO_SOLANA_MAINNET_EID`.
    - **Step 3: Derive PDAs:** Call functions from `pdas.rs` to get all required PDAs for the instruction accounts and remaining accounts. List needed: `vault_authority`, `allowed_broker`, `allowed_token`, `oapp_config`, `peer`, `enforced_options`, `send_lib_config`, `default_send_lib`, `send_lib_info`, `endpoint_setting`, `nonce`, `uln_setting`, `send_config`, `default_send_config`, `executor_config`, `price_feed`, `dvn_config`.
    - **Step 4: Get Token Accounts:** Derive user and vault USDC associated token accounts (`get_associated_token_address`).
    - **Step 5: Load IDL & Create Program:**
      - `let idl: Idl = serde_json::from_str(include_str!("../../idl/solana_vault.json"))?;`
      - `let program = Client::new_with_options(..., program_id, ...).program(idl);` (Adapt based on Anchor client setup)
    - **Step 6: Construct Instruction Data:**
      - Create `DepositParams` instance.
      - Create `SendParam` instance. **TODO:** Implement fee calculation (e.g., `getDepositQuoteFee` equivalent) or use `0` initially.
    - **Step 7: Build Account Metas:**
      - Create `Vec<AccountMeta>` for `.accounts({...})`. Ensure order matches IDL (`userTokenAccount`, `vaultAuthority`, `vaultTokenAccount`, `depositToken`, `user`, `peer`, `enforcedOptions`, `oappConfig`, `allowedBroker`, `allowedToken`). Set `is_signer` and `is_writable` correctly.
      - Create `Vec<AccountMeta>` for `.remainingAccounts([...])`. **Crucially, ensure the exact order and pubkeys match the JS `helper.ts` implementation (lines ~514-566).** Map derived PDAs and program IDs carefully. Set `is_signer` and `is_writable` correctly.
    - **Step 8: Build Instruction:**
      - Use `program.request()` fluent builder:
      ```rust
      let ix = program.request()
          .accounts(accounts_struct) // Use defined Account metas
          .args(instruction_data) // deposit_params, send_param
          .instruction()?;
      ```
      - _(Self-Correction: The `.remainingAccounts` need to be added to the instruction builder correctly, potentially after `.accounts()` or via direct `Instruction` construction if the builder doesn't support it easily)._
    - **Step 9: Add Compute Budget:** `ComputeBudgetInstruction::set_compute_unit_limit(400_000)`.
    - **Step 10: Fetch Blockhash:** `rpc_client.get_latest_blockhash()?`.
    - **Step 11: Create Transaction:**
      - Assemble instructions: `[ix, compute_budget_ix]`.
      - Create `Message::new_with_blockhash(...)` or `MessageV0::new_with_blockhash(...)` (if using ALT).
      - Create `VersionedTransaction::try_new(...)`.
    - **Step 12: Partial Sign:** `tx.sign(&[user_keypair], blockhash)?;`.
    - **Step 13: Return Transaction:** `Ok(tx)`.
3.  **Integration Tests:** Test `prepare_solana_deposit_tx`:
    - Verify instruction data serialization.
    - Verify account keys and meta properties match expected values.
    - Simulate sending against a local validator if possible.

---

## Phase 4: Withdrawal Flow

**Goal:** Implement the off-chain message signing and API interaction for withdrawals.

1.  **Define API Types (`src/rest/models.rs` or `src/types.rs`):**

    - Struct for `GET /v1/withdraw_nonce` response (e.g., `WithdrawNonceResponse { success: bool, data: WithdrawNonceData { withdraw_nonce: u64 } }`).
    - Struct for `POST /v1/withdraw_request` request body (e.g., `WithdrawRequest { message: WithdrawalMessageForApi, signature: String }`). Note `WithdrawalMessageForApi` needs fields matching API spec (raw strings, nonce, timestamp etc.).
    - Struct for `POST /v1/withdraw_request` response (e.g., `WithdrawResponse { success: bool, ... }`).

2.  **Implement `prepare_withdrawal_message` (in `src/solana/signing.rs` or `src/rest/client.rs`):**

    - **Function Signature:**
      ```rust
      pub async fn prepare_withdrawal_message(
          // HTTP client, base URL, etc. needed to fetch nonce
          http_client: &reqwest::Client,
          base_api_url: &str,
          orderly_account_id: &str, // Needed for nonce endpoint
          creds: &Credentials,       // For authenticated nonce endpoint
          solana_config: &SolanaConfig,
          user_keypair: &Keypair,
          receiver_addr: &str,
          token: &str,
          amount: u64,
      ) -> Result<(WithdrawalMessageForApi, String), OrderlyError> // Returns API message + signature
      ```
    - **Step 1: Fetch Nonce:**
      - Make authenticated `GET` request to `/v1/withdraw_nonce`. Use `orderly_account_id`. Handle HTTP errors.
      - Extract `withdraw_nonce` from the response.
    - **Step 2: Get Timestamp:** `SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64`.
    - **Step 3: Create `WithdrawalMessage`:** Use `create_withdrawal_message` from `abi.rs`. Pass the correct `chain_id` from `solana_config.orderly_solana_chain_id`.
    - **Step 4: Encode & Hash:**
      - `let encoded_message = solabi::encode(&message);`
      - `let message_hash = solabi::keccak::v256(&encoded_message);`
    - **Step 5: Sign Hash:** `let signature = sign_solana_message(&message_hash, user_keypair)?;`
    - **Step 6: Prepare API Message:** Create the `WithdrawalMessageForApi` struct instance using the _original_ (unhashed) values from the `WithdrawalMessage` struct (brokerId, chainId, receiver, token, amount, timestamp, withdrawNonce).
    - **Step 7: Return:** `Ok((api_message, signature))`

3.  **Implement `request_withdrawal` (in `src/rest/client.rs`):**

    - **Function Signature:**
      ```rust
      pub async fn request_withdrawal(
          &self, // Assuming part of OrderlyService
          creds: &Credentials,
          api_message: WithdrawalMessageForApi,
          signature: String,
      ) -> Result<WithdrawResponse, OrderlyError>
      ```
    - **Step 1: Create Request Body:** Construct `WithdrawRequest { message: api_message, signature }`.
    - **Step 2: Send Request:** Make authenticated `POST` request to `/v1/withdraw_request` with the JSON body.
    - **Step 3: Handle Response:** Deserialize response into `WithdrawResponse`, handle API errors (`success: false`) and HTTP errors.
    - **Step 4: Return:** `Ok(response)`.

4.  **Unit/Integration Tests:**
    - Unit test `prepare_withdrawal_message` (mock nonce API call). Verify encoding, hashing, signing.
    - Integration test `request_withdrawal` (mock API or use testnet).

---

## Phase 5: Finalization & Documentation

1.  **Code Cleanup:** Ensure consistent formatting, remove unused code, add necessary comments.
2.  **Error Handling:** Review error propagation and ensure meaningful errors are returned.
3.  **Examples:** Add examples for deposit preparation and the full withdrawal flow to `README.md` or an examples directory.
4.  **Documentation:** Update `lib.rs` documentation, module-level docs, and any relevant struct/function docs. Update main `README.md`.

---
