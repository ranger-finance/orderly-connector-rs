//! Solana client logic for Orderly deposit/withdrawal flows.

use crate::error::OrderlyError;
use crate::solana::pdas::*;
use crate::solana::types::SolanaConfig;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    message::{v0::Message as MessageV0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::VersionedTransaction,
};
use solana_vault_cpi::{
    accounts::Deposit, instruction::Deposit as DepositIx, DepositParams, OAppSendParams,
};
use spl_associated_token_account::get_associated_token_address;

/// Prepares a Solana deposit transaction for Orderly, matching the JS SDK logic.
pub fn prepare_solana_deposit_tx(
    rpc_client: &RpcClient,
    config: &SolanaConfig,
    user_keypair: &Keypair,
    amount: u64,
    orderly_account_id_hex: &str,
    // vault_program_id: &Pubkey, // Not needed, use get_program_id
) -> Result<VersionedTransaction, OrderlyError> {
    if amount == 0 {
        return Err(OrderlyError::ValidationError("Amount must be > 0".into()));
    }
    let orderly_account_id = hex::decode(orderly_account_id_hex)
        .map_err(|_| OrderlyError::ValidationError("Invalid orderly_account_id_hex".into()))?;
    if orderly_account_id.len() != 32 {
        return Err(OrderlyError::ValidationError(
            "orderly_account_id_hex must be 32 bytes".into(),
        ));
    }
    let orderly_account_id_bytes: [u8; 32] = orderly_account_id.try_into().unwrap();

    let broker_hash = v256(config.broker_id.as_bytes());
    let token_hash = v256(b"USDC");
    let dst_eid = crate::solana::types::LAYERZERO_SOLANA_MAINNET_EID;
    let user_pubkey = user_keypair.pubkey();
    let usdc_mint = config.usdc_mint;

    // PDAs
    let (vault_authority, _) = get_vault_authority_pda();
    let (allowed_broker, _) = get_broker_pda(&broker_hash);
    let (allowed_token, _) = get_token_pda(&token_hash);
    let (oapp_config, _) = get_oapp_config_pda();
    let (peer, _) = get_peer_pda(dst_eid);
    let (enforced_options, _) = get_enforced_options_pda(dst_eid);
    let (send_lib_config, _) = get_send_lib_config_pda(dst_eid);
    let (default_send_lib, _) = get_default_send_lib_pda(dst_eid);
    let (send_lib_info, _) = get_send_lib_info_pda();
    let (endpoint_setting, _) = get_endpoint_setting_pda();
    let (nonce, _) = get_nonce_pda(dst_eid);
    let (uln_setting, _) = get_uln_setting_pda();
    let (send_config, _) = get_send_config_pda(dst_eid);
    let (default_send_config, _) = get_default_send_config_pda(dst_eid);
    let (executor_config, _) = get_executor_config_pda();
    let (price_feed, _) = get_price_feed_pda();
    let (dvn_config, _) = get_dvn_config_pda();
    // For event_authority and uln_event_authority, use get_program_id or hardcoded if needed
    let event_authority = get_program_id("ENDPOINT").expect("ENDPOINT program ID not found");
    let uln_event_authority = get_program_id("ENDPOINT").expect("ENDPOINT program ID not found");

    // Token accounts
    let user_usdc_account = get_associated_token_address(&user_pubkey, &usdc_mint);
    let vault_usdc_account = get_associated_token_address(&vault_authority, &usdc_mint);

    // Build DepositParams and OAppSendParams
    let deposit_params = DepositParams {
        account_id: orderly_account_id_bytes,
        broker_hash,
        token_hash,
        user_address: user_pubkey.to_bytes(),
        token_amount: amount,
    };
    let oapp_params = OAppSendParams {
        native_fee: 0, // TODO: implement fee calculation
        lz_token_fee: 0,
    };

    let accounts = Deposit {
        user_token_account: user_usdc_account,
        vault_authority,
        vault_token_account: vault_usdc_account,
        deposit_token: usdc_mint,
        user: user_pubkey,
        peer,
        enforced_options,
        oapp_config,
        allowed_broker,
        allowed_token,
        associated_token_program: spl_associated_token_account::id(),
        system_program: system_program::id(),
        token_program: spl_token::id(),
    };

    let remaining_accounts = vec![
        AccountMeta::new_readonly(get_program_id("ENDPOINT").unwrap(), false),
        AccountMeta::new_readonly(oapp_config, false),
        AccountMeta::new_readonly(get_program_id("SEND_LIB").unwrap(), false),
        AccountMeta::new_readonly(send_lib_config, false),
        AccountMeta::new_readonly(default_send_lib, false),
        AccountMeta::new_readonly(send_lib_info, false),
        AccountMeta::new_readonly(endpoint_setting, false),
        AccountMeta::new(nonce, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(get_program_id("ENDPOINT").unwrap(), false),
        AccountMeta::new_readonly(uln_setting, false),
        AccountMeta::new_readonly(send_config, false),
        AccountMeta::new_readonly(default_send_config, false),
        AccountMeta::new_readonly(user_pubkey, true),
        AccountMeta::new_readonly(get_program_id("TREASURY").unwrap(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(uln_event_authority, false),
        AccountMeta::new_readonly(get_program_id("SEND_LIB").unwrap(), false),
        AccountMeta::new_readonly(get_program_id("EXECUTOR").unwrap(), false),
        AccountMeta::new(executor_config, false),
        AccountMeta::new_readonly(get_program_id("PRICE_FEED").unwrap(), false),
        AccountMeta::new_readonly(price_feed, false),
        AccountMeta::new_readonly(get_program_id("DVN").unwrap(), false),
        AccountMeta::new(dvn_config, false),
        AccountMeta::new_readonly(get_program_id("PRICE_FEED").unwrap(), false),
        AccountMeta::new_readonly(price_feed, false),
    ];

    // Construct Instruction manually for CPI
    let ix_data = DepositIx {
        _deposit_params: deposit_params,
        _oapp_params: oapp_params,
    };

    // Convert the accounts struct + remaining accounts into AccountMeta list
    let mut account_metas = accounts.to_account_metas(None);
    account_metas.extend(remaining_accounts); // Add the manually specified remaining accounts

    // Construct the final Instruction
    let ix = Instruction {
        program_id: solana_vault_cpi::ID, // Use the Program ID from the CPI crate
        accounts: account_metas,
        data: ix_data.data(), // Serialize the instruction data
    };

    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    let blockhash = rpc_client
        .get_latest_blockhash()
        .map_err(|e| OrderlyError::NetworkError(e.to_string()))?;
    let instructions = vec![ix, compute_budget_ix];
    let payer = user_pubkey;
    let message = MessageV0::try_compile(
        &payer,
        &instructions,
        &[], // No address lookup tables for now
        blockhash,
    )
    .map_err(|e| OrderlyError::NetworkError(e.to_string()))?;
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(message), &[user_keypair])
        .map_err(|e| OrderlyError::NetworkError(e.to_string()))?;

    Ok(tx)
}

// Helper for keccak256 hash (v256)
fn v256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}
