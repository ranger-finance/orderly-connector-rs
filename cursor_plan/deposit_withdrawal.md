# Solana Deposit & Withdrawal Implementation Plan

This plan details the step-by-step implementation of Solana deposit and withdrawal functionality for the `orderly-connector-rs` SDK, based on analysis of `trade.md` and the reference JS SDK (`helper.ts`).

**Scope:** This plan covers the creation of PDA derivation utilities, the Solana deposit transaction preparation, and the off-chain message preparation and API interaction for withdrawals. Account registration is considered complete and out of scope for this specific plan.

---

## Phase 1: Prerequisites & Setup

**Goal:** Ensure foundational components, configuration, and utilities are in place.

- [x] **Configuration (`src/solana/types.rs`):**

  - [x] `SolanaConfig` struct contains necessary fields: `rpc_url`, `usdc_mint`, `broker_id`, `orderly_solana_chain_id` (for signing).
  - [x] `get_program_id(name: &str)` function provides access to all required program IDs (Vault, LayerZero Endpoint, SendLib, Treasury, Executor, PriceFeed, DVN).
  - [x] `LAYERZERO_SOLANA_MAINNET_EID` constant (`30109`) is defined.

- [x] **Core Solana Utilities:**

  - [x] Basic Solana signing utility exists, e.g., `sign_solana_message(message_bytes: &[u8], keypair: &Keypair) -> Result<String, OrderlyError>`.
  - [x] Basic error types (`OrderlyError`) are defined.

- [x] **ABI Definitions (`src/eth/abi.rs`):**
  - [x] `WithdrawalMessage` and `RegistrationMessage` structs align with the latest EIP-712 definitions.
  - [x] `create_withdrawal_message` and `create_registration_message` functions correctly populate these structs.
  - [x] `Encode` implementations for both messages correctly reflect the EIP-712 data hashing structure.

---

## Phase 2: PDA Derivation Utilities

**Goal:** Implement functions to derive all necessary Program Derived Addresses (PDAs) required for the deposit transaction.

- [x] **Create File:** `src/solana/pdas.rs` exists.
- [x] **Add Imports:** Required imports are present.
- [x] **Implement PDA Functions:** All required PDA derivation functions are implemented.
- [x] **Expose Module:** Module is exposed in `src/solana/mod.rs`.
- [x] **Unit Tests:** PDA derivation tests are present.

---

## Phase 3: Solana Deposit Transaction Preparation

**Goal:** Implement the function that constructs and partially signs the Solana deposit transaction.

- [x] **Define Structs:** `DepositParams` and `SendParam` (as `OAppSendParams`) are present via Anchor CPI and IDL.
- [x] **Implement `prepare_solana_deposit_tx` (in `src/solana/client.rs`):** Function is implemented and matches the plan.
- [ ] **Integration Tests:** Main logic is implemented; additional integration/unit tests are recommended for full coverage.

---

## Phase 4: Withdrawal Flow

**Goal:** Implement the off-chain message signing and API interaction for withdrawals.

- [x] **Define API Types:** Types for withdrawal nonce, request, and response are present in `src/types.rs`.
- [x] **Implement `prepare_withdrawal_message`:** Implemented in `src/solana/signing.rs`.
- [x] **Implement `request_withdrawal`:** Implemented in `src/rest/client.rs`.
- [ ] **Unit/Integration Tests:** Main logic is implemented; additional tests are recommended for robustness.

---

## Phase 5: Finalization & Documentation

- [ ] **Code Cleanup:** Review for final polish.
- [ ] **Error Handling:** Review for comprehensive error propagation and context.
- [ ] **Examples:** Add usage examples for deposit and withdrawal flows.
- [ ] **Documentation:** Update module-level and README documentation as needed.

---

**Legend:**

- [x] Complete
- [ ] Incomplete/Recommended for further work

**Summary:**

- All core logic for Solana deposit and withdrawal is implemented and matches the plan.
- Remaining work is primarily in testing, documentation, and final polish for production readiness.

---
