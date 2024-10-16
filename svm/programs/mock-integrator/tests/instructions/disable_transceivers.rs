use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::DisableTransceiver;
use router::instructions::DisableTransceiverArgs;
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
    registered_transceiver: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let accounts = DisableTransceiver {
        admin: admin.pubkey(),
        integrator_config,
        integrator_chain_config,
        registered_transceiver,
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
    registered_transceiver: Pubkey,
    chain_id: u16,
    transceiver: Pubkey,
    integrator_program: Pubkey,
) -> Result<(), BanksClientError> {
    let args = DisableTransceiverArgs {
        chain_id,
        transceiver,
        integrator_program,
    };
    let instruction_data = router::instruction::DisableRecvTransceiver { args }.data();
    execute_disable_transceiver(
        context,
        admin,
        payer,
        integrator_config,
        integrator_chain_config,
        registered_transceiver,
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
    registered_transceiver: Pubkey,
    chain_id: u16,
    transceiver: Pubkey,
    integrator_program: Pubkey,
) -> Result<(), BanksClientError> {
    let args = DisableTransceiverArgs {
        chain_id,
        transceiver,
        integrator_program,
    };
    let instruction_data = router::instruction::DisableSendTransceiver { args }.data();
    execute_disable_transceiver(
        context,
        admin,
        payer,
        integrator_config,
        integrator_chain_config,
        registered_transceiver,
        instruction_data,
    )
    .await
}
