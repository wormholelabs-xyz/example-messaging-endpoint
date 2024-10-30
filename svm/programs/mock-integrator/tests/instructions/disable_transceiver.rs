use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::DisableTransceiver;
use router::instructions::TransceiverInfoArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

async fn execute_disable_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    transceiver_info: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let accounts = DisableTransceiver {
        admin: admin.pubkey(),
        integrator_config,
        integrator_chain_config,
        transceiver_info,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: instruction_data,
    };
    execute_transaction(context, ix, &[admin, payer], payer).await
}

pub async fn disable_recv_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    transceiver_info: Pubkey,
    chain_id: u16,
    transceiver_program_id: Pubkey,
    integrator_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let args = TransceiverInfoArgs {
        chain_id,
        transceiver_program_id,
        integrator_program_id,
    };
    let instruction_data = router::instruction::DisableRecvTransceiver { args }.data();
    execute_disable_transceiver(
        context,
        admin,
        payer,
        integrator_config,
        integrator_chain_config,
        transceiver_info,
        instruction_data,
    )
    .await
}

pub async fn disable_send_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    transceiver_info: Pubkey,
    chain_id: u16,
    transceiver_program_id: Pubkey,
    integrator_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let args = TransceiverInfoArgs {
        chain_id,
        transceiver_program_id,
        integrator_program_id,
    };
    let instruction_data = router::instruction::DisableSendTransceiver { args }.data();
    execute_disable_transceiver(
        context,
        admin,
        payer,
        integrator_config,
        integrator_chain_config,
        transceiver_info,
        instruction_data,
    )
    .await
}
