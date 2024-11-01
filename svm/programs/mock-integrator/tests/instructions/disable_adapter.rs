use anchor_lang::{InstructionData, ToAccountMetas};
use endpoint::accounts::DisableAdapter;
use endpoint::instructions::AdapterInfoArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

async fn execute_disable_adapter(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    adapter_info: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &endpoint::id());

    let accounts = DisableAdapter {
        admin: admin.pubkey(),
        integrator_config,
        integrator_chain_config,
        adapter_info,
        event_authority,
        program: endpoint::id(),
    };

    let ix = Instruction {
        program_id: endpoint::id(),
        accounts: accounts.to_account_metas(None),
        data: instruction_data,
    };
    execute_transaction(context, ix, &[admin, payer], payer).await
}

pub async fn disable_recv_adapter(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    adapter_info: Pubkey,
    chain_id: u16,
    adapter_program_id: Pubkey,
    integrator_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let args = AdapterInfoArgs {
        chain_id,
        adapter_program_id,
        integrator_program_id,
    };
    let instruction_data = endpoint::instruction::DisableRecvAdapter { args }.data();
    execute_disable_adapter(
        context,
        admin,
        payer,
        integrator_config,
        integrator_chain_config,
        adapter_info,
        instruction_data,
    )
    .await
}

pub async fn disable_send_adapter(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    adapter_info: Pubkey,
    chain_id: u16,
    adapter_program_id: Pubkey,
    integrator_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let args = AdapterInfoArgs {
        chain_id,
        adapter_program_id,
        integrator_program_id,
    };
    let instruction_data = endpoint::instruction::DisableSendAdapter { args }.data();
    execute_disable_adapter(
        context,
        admin,
        payer,
        integrator_config,
        integrator_chain_config,
        adapter_info,
        instruction_data,
    )
    .await
}
