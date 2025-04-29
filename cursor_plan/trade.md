# Solana Deposit, Withdrawal, and Key Registration Implementation Plan

This document outlines the steps to implement Solana deposit, withdrawal, and key registration functionality within the `orderly-connector-rs` SDK.

## 1. Project Setup & Dependencies

- **Dependencies (`Cargo.toml`):**
  - Ensure the following crates are added:

````toml
    solana-sdk = "..."
    solana-client = { version = "...", features = ["nonblocking"] }
    solana-program = "..."
    spl-token = "..."
    spl-associated-token-account = "..."
    anchor-client = "..."
    anchor-lang = "..." # Optional, for direct IDL struct use
    bs58 = "..."
    base64 = "..."
    serde = { version = "...", features = ["derive"] }
    serde_json = "..."
    reqwest = { version = "...", features = ["json"] }
    tokio = { version = "...", features = ["full"] }
    sha3 = "..." # For Keccak256
    hex = "..."
    anyhow = "..." # Or define custom Error enum
    thiserror = "..." # If using custom Error enum
    bincode = "..." # For transaction serialization
    ```
- **Configuration:**
  - Define a `SolanaConfig` struct or similar to hold:
    - `rpc_url: String`
    - `api_base_url: String`
    - `vault_program_id: Pubkey` **(Verified: `ErBmAD61mGFKvrFNaTJuxoPwqrS8GgtwtqJTJVjFWx9Q`)**
    - `usdc_mint: Pubkey`
    - `broker_id: String`
    - Relevant LayerZero/Endpoint Program IDs (`ENDPOINT_PROGRAM_ID`, etc.) **(Needs Verification)**
- **Error Handling:**
  - Define a custom `Error` enum using `thiserror` or use `anyhow::Error` for simplicity. Propagate errors using `Result<T, Error>`.

## 2. Core Solana Utilities

- **Hashing:**
  - Implement `fn hash_broker_id(broker_id: &str) -> [u8; 32]` using `sha3::Keccak256`.
  - Implement `fn hash_token_id(token_id: &str) -> [u8; 32]` using `sha3::Keccak256`.
- **PDA Derivation:**
  - Create a module (e.g., `solana_pdas.rs`) with functions for deriving addresses:
    - `find_vault_authority_pda(vault_program_id: &Pubkey) -> Pubkey` **(Seeds Need Verification)**
    - `find_user_token_account(user_wallet: &Pubkey, mint: &Pubkey) -> Pubkey` (using `spl_associated_token_account::get_associated_token_address`)
    - `find_vault_token_account(vault_authority_pda: &Pubkey, mint: &Pubkey) -> Pubkey` (using `spl_associated_token_account::get_associated_token_address`)
    - `find_allowed_broker_pda(vault_program_id: &Pubkey, broker_hash: &[u8; 32]) -> Pubkey` **(Seeds Need Verification)**
    - `find_allowed_token_pda(vault_program_id: &Pubkey, token_hash: &[u8; 32]) -> Pubkey` **(Seeds Need Verification)**
    - Functions for all LayerZero/Endpoint PDAs (`oappConfigPDA`, `peerPDA`, `endorcedOptionsPDA`, `sendLibPDA`, `noncePDA`, etc.) based on JS code. **(Seeds Need Verification)**
  - **Note:** Document clearly that the seeds used initially are based on JS code interpretation and require verification against the `solana-vault` Rust source.

## 3. API Client Enhancements

- In the existing API client module:
  - Add `async fn get_withdrawal_nonce(&self) -> Result<u64, Error>`
    - Perform a `GET` request to `{api_base_url}/v1/withdraw_nonce`.
    - Parse the JSON response to extract the nonce.
    - Handle potential API errors.

## 4. Solana Deposit Implementation (`prepare_solana_deposit_tx`)

- **Function Signature:**
  ```rust
  async fn prepare_solana_deposit_tx(
      rpc_client: &solana_client::nonblocking::rpc_client::RpcClient,
      anchor_program_client: // Type depends on anchor_client setup, e.g., &anchor_client::Program
      user_wallet: Pubkey,
      amount_lamports: u64, // Amount in lamports (USDC has 6 decimals usually)
      config: &SolanaConfig,
  ) -> Result<String, Error> // Base64 encoded VersionedTransaction string
````

- **Argument Structs (Mirroring IDL):** **(IDL Found)**
  - Define `DepositParams { account_id: [u8; 32], broker_hash: [u8; 32], token_hash: [u8; 32], user_address: [u8; 32], token_amount: u64 }` (Based on IDL - Verify exact types/sizes, e.g., `u64` vs `u128` if needed).
  - Define `OAppSendParams { native_fee: u64, lz_token_fee: u64 }` (Based on IDL).
- **PDA Derivation:**
  - Call the helper functions from Step 2 to get all required PDA `Pubkey`s. Use hashed `broker_id` and `token_id` ("USDC").
- **Fetch Deposit Fee (`get_deposit_quote_fee` Logic):** **(IDL Found - Instruction: `oappQuote`)**
  - Use the `anchor_program_client` to make a view call to the `oappQuote` instruction.
  - Construct the `DepositParams` argument required by `oappQuote`.
  - Parse the result (IDL defines `MessagingFee { native_fee: u64, lz_token_fee: u64 }`) to get the `native_fee`.
- **Build Instruction:** **(IDL Found & Seed Verification Needed)**
  - Instantiate the `DepositParams` and `OAppSendParams` structs for the `deposit` instruction.
  - Use `anchor_client`'s request builder:
    ```rust
    let instruction = anchor_program_client
        .request()
        // Accounts based on IDL 'deposit' instruction:
        .accounts(solana_vault::accounts::DepositAccounts { // Example struct name
            user: user_wallet,
            user_token_account: user_usdc_ata_pda,
            vault_authority: vault_authority_pda,
            vault_token_account: vault_usdc_ata_pda,
            deposit_token: config.usdc_mint, // Check if this is the correct account name in IDL
            peer: peer_pda, // Derived
            enforced_options: enforced_options_pda, // Derived
            oapp_config: oapp_config_pda, // Derived
            allowed_broker: allowed_broker_pda, // Derived
            allowed_token: allowed_token_pda, // Derived
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: system_program::ID,
        })
        .args(solana_vault::instruction::Deposit { // Example struct name
             deposit_params: deposit_params_instance,
             oapp_params: oapp_send_params_instance,
        })
        .remaining_accounts(...) // Add all remaining accounts for LZ/Endpoint calls in the EXACT order from JS/IDL **(Needs careful verification)**
        .instructions()?;
    ```
  - Ensure account pubkeys and `remaining_accounts` match the JS example and underlying program logic precisely.
- **Build Transaction:**
  - Create `ComputeBudgetInstruction::set_compute_unit_limit(400_000)`.
  - Derive the Address Lookup Table (ALT) address using the logic from `getLookupTableAddress(appProgramId)` in JS. **(Needs Verification)**
  - Fetch the `lookup_table_account` using `rpc_client.get_account(&alt_address).await?`. Deserialize it into `AddressLookupTableAccount`.
  - Fetch `latest_blockhash = rpc_client.get_latest_blockhash().await?`.
  - Create `message = solana_sdk::message::v0::Message::try_compile(&user_wallet, &[deposit_instruction, compute_budget_instruction], &[lookup_table_account], latest_blockhash)?`.
  - Create `tx = solana_sdk::transaction::VersionedTransaction::try_new(message, &[] /* signers */)?`.
- **Serialize & Encode:**
  - `serialized_tx = bincode::serialize(&tx)?` or use `tx.serialize()?`.
  - `base64_encoded = base64::engine::general_purpose::STANDARD.encode(&serialized_tx)`
- **Return:** `Ok(base64_encoded)`

## 5. Off-Chain Message Preparation (Withdrawal & Key Reg)

- **Serializable Structs:**
  - Define `WithdrawMessage { domain: Eip712Domain, types: Eip712Types, primary_type: String, message: WithdrawMessageData }` (or similar structure suitable for frontend EIP-712 signing).
  - Define `WithdrawMessageData { brokerId: String, chainId: u64, receiver: String, token: String, amount: String, withdrawNonce: u64, timestamp: u64 }`. Use `String` for receiver address and amount for easy JSON serialization.
  - Define `RegisterKeyMessage { ... }` and `RegisterKeyMessageData { orderlyKey: String, scope: String, expiration: u64, timestamp: u64 }` similarly.
- **Implement `prepare_withdrawal_message`:**
  ```rust
  fn prepare_withdrawal_message(
      broker_id: &str,
      destination_chain_id: u64, // Solana Chain ID
      receiver_address: &str,    // User's address on Solana
      token_symbol: &str,        // e.g., "USDC"
      amount_str: &str,          // Amount as string
      nonce: u64,
      timestamp_ms: u64,
  ) -> Result<WithdrawMessage, Error> // Return the serializable struct
  ```
  - Construct the `WithdrawMessageData`.
  - Define the EIP-712 `domain`, `types`, and `primary_type` according to Orderly specs.
  - Return the combined `WithdrawMessage` struct.
- **Implement `prepare_register_orderly_key_message`:**
  ```rust
   fn prepare_register_orderly_key_message(
       orderly_key: &str, // The public key being registered
       scope: &str,       // e.g., "read,trading"
       expiration_ms: u64,
       timestamp_ms: u64,
   ) -> Result<RegisterKeyMessage, Error> // Return the serializable struct
  ```
  - Construct the `RegisterKeyMessageData`.
  - Define the EIP-712 `domain`, `types`, `primary_type` for key registration.
  - Return the combined `RegisterKeyMessage` struct.

## 6. Verification and Refinement

- **Action:** Clone `OrderlyNetwork/solana-vault`. **(DONE - Info Used)**
- **Action:** Locate `vault.json` (or similar) IDL file. **(DONE - IDL Found at [JS SDK Link](https://github.com/OrderlyNetwork/js-sdk/blob/main/packages/default-solana-adapter/src/idl/solana_vault.json))** Copy it into the `orderly-connector-rs` project.
- **Action:** Find Vault Program ID. **(DONE - `ErBmAD61mGFKvrFNaTJuxoPwqrS8GgtwtqJTJVjFWx9Q`)** Update `SolanaConfig` (Step 1).
- **Action:** Analyze the **JavaScript implementation** (`helper.ts`) provided at [JS SDK Link](https://github.com/OrderlyNetwork/js-sdk/blob/main/packages/default-solana-adapter/src/helper.ts#L493) to determine the exact seeds used in `findProgramAddressSync` for all PDAs required by the `deposit` instruction (Vault PDAs and associated LayerZero/Endpoint PDAs). Update PDA helper functions (Step 2). **(TODO - Relying on JS)**
- **Action:** If using `anchor_client`, ensure the IDL is correctly parsed and types match. If building instructions manually, ensure accounts and data layout match the IDL. Update Step 4 based on IDL. **(Partially DONE - IDL available)**
- **Action:** Analyze the **JavaScript implementation** (`helper.ts`) to determine the logic for deriving the Address Lookup Table address (`getLookupTableAddress` function). Implement this logic in Rust. **(TODO - Relying on JS)**
- **Action:** Analyze the **JavaScript implementation** (`helper.ts`) to determine the exact order and composition of the `.remainingAccounts([...])` for the `deposit` instruction. Ensure the Rust implementation matches precisely. **(TODO - Relying on JS)**

## 7. Testing

- **Unit Tests:**
  - Test Keccak256 hashing functions.
  - Test PDA derivation functions with known, verified seeds and inputs.
  - Test construction of `WithdrawMessage` and `RegisterKeyMessage` structs.
- **Integration Tests:**
  - Test `get_withdrawal_nonce` against the Orderly API (testnet endpoint).
  - Mock `RpcClient` responses for `prepare_solana_deposit_tx` to test:
    - Correct fee fetching (mock view call response).
    - Correct ALT fetching and usage.
    - Correct blockhash usage.
    - Verify the structure, accounts, and data of the generated `deposit` instruction against expected values based on the IDL.
    - Verify the final base64 encoding/decoding.
  - **Full End-to-End:** Requires deploying/using the `solana-vault` program on Solana Devnet/Testnet, funding a wallet, and sending the transaction prepared by the SDK function.
