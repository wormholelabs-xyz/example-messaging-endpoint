use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::SetTransceiver;
use router::instructions::SetTransceiverArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn execute_set_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_config: Pubkey,
    registered_transceiver: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let accounts = SetTransceiver {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        integrator_chain_config,
        registered_transceiver,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: instruction_data,
    };
    execute_transaction(context, ix, &[admin, payer], payer).await
}

pub async fn set_recv_transceiver(
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
    let args = SetTransceiverArgs {
        chain_id,
        transceiver,
        integrator_program,
    };
    let instruction_data = router::instruction::SetRecvTransceiver { args }.data();
    execute_set_transceiver(
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

pub async fn set_send_transceiver(
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
    let args = SetTransceiverArgs {
        chain_id,
        transceiver,
        integrator_program,
    };
    let instruction_data = router::instruction::SetSendTransceiver { args }.data();
    execute_set_transceiver(
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
