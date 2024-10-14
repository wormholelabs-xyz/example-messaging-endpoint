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

async fn execute_set_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    integrator_program: Pubkey,
    registered_transceiver: Pubkey,
    transceiver: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let accounts = SetTransceiver {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
        registered_transceiver,
        transceiver,
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
    owner: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    integrator_program: Pubkey,
    registered_transceiver: Pubkey,
    transceiver: Pubkey,
    chain_id: u16,
) -> Result<(), BanksClientError> {
    let args = SetTransceiverArgs { chain_id };
    let instruction_data = router::instruction::SetRecvTransceiver { args }.data();
    execute_set_transceiver(
        context,
        owner,
        payer,
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
        registered_transceiver,
        transceiver,
        instruction_data,
    )
    .await
}

pub async fn disable_recv_transceiver(
    context: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    integrator_program: Pubkey,
    registered_transceiver: Pubkey,
    transceiver: Pubkey,
    chain_id: u16,
) -> Result<(), BanksClientError> {
    let args = SetTransceiverArgs { chain_id };
    let instruction_data = router::instruction::DisableRecvTransceiver { args }.data();
    execute_set_transceiver(
        context,
        owner,
        payer,
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
        registered_transceiver,
        transceiver,
        instruction_data,
    )
    .await
}

pub async fn set_send_transceiver(
    context: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    integrator_program: Pubkey,
    registered_transceiver: Pubkey,
    transceiver: Pubkey,
    chain_id: u16,
) -> Result<(), BanksClientError> {
    let args = SetTransceiverArgs { chain_id };
    let instruction_data = router::instruction::SetSendTransceiver { args }.data();
    execute_set_transceiver(
        context,
        owner,
        payer,
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
        registered_transceiver,
        transceiver,
        instruction_data,
    )
    .await
}

pub async fn disable_send_transceiver(
    context: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_chain_transceivers: Pubkey,
    integrator_program: Pubkey,
    registered_transceiver: Pubkey,
    transceiver: Pubkey,
    chain_id: u16,
) -> Result<(), BanksClientError> {
    let args = SetTransceiverArgs { chain_id };
    let instruction_data = router::instruction::DisableSendTransceiver { args }.data();
    execute_set_transceiver(
        context,
        owner,
        payer,
        integrator_config,
        integrator_chain_transceivers,
        integrator_program,
        registered_transceiver,
        transceiver,
        instruction_data,
    )
    .await
}
