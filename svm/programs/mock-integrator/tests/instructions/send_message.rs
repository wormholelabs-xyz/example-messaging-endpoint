use anchor_lang::{InstructionData, ToAccountMetas};
use mock_integrator::{accounts::InvokeSendMessage, InvokeSendMessageArgs};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use universal_address::UniversalAddress;

use crate::common::execute_transaction::execute_transaction;

async fn execute_send_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    integrator_program_pda: Pubkey,
    integrator_chain_config: Pubkey,
    outbox_message: Pubkey,
    outbox_message_key: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<(), BanksClientError> {
    let accounts = InvokeSendMessage {
        payer: payer.pubkey(),
        integrator_program_pda,
        integrator_chain_config,
        outbox_message,
        outbox_message_key,
        system_program: solana_sdk::system_program::id(),
        router_program: router::id(),
    };

    let ix = Instruction {
        program_id: mock_integrator::id(),
        accounts: accounts.to_account_metas(None),
        data: instruction_data,
    };

    execute_transaction(context, ix, &[payer], payer).await
}

pub async fn send_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    integrator_program_pda: Pubkey,
    integrator_chain_config: Pubkey,
    outbox_message: Pubkey,
    outbox_message_key: Pubkey,
    integrator_program_id: Pubkey,
    integrator_program_pda_bump: u8,
    dst_chain: u16,
    dst_addr: UniversalAddress,
    payload_hash: [u8; 32],
) -> Result<(), BanksClientError> {
    let args = InvokeSendMessageArgs {
        dst_chain,
        dst_addr,
        payload_hash,
    };

    let instruction_data = mock_integrator::instruction::InvokeSendMessage { args }.data();

    execute_send_message(
        context,
        payer,
        integrator_program_pda,
        integrator_chain_config,
        outbox_message,
        outbox_message_key,
        instruction_data,
    )
    .await
}
