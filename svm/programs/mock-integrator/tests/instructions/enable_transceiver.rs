use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::EnableTransceiver;
use router::instructions::TransceiverInfoArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn execute_enable_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    transceiver_info: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let accounts = EnableTransceiver {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        integrator_chain_config,
        transceiver_info,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: instruction_data,
    };
    execute_transaction(context, ix, &[admin, payer], payer).await
}

pub async fn enable_recv_transceiver(
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
    let instruction_data = router::instruction::EnableRecvTransceiver { args }.data();
    execute_enable_transceiver(
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

pub async fn enable_send_transceiver(
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
    let instruction_data = router::instruction::EnableSendTransceiver { args }.data();
    execute_enable_transceiver(
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
