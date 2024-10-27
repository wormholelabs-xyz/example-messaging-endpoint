use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::SendMessage;
use router::instructions::SendMessageArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use universal_address::UniversalAddress;

use crate::common::execute_transaction::execute_transaction;

pub async fn send_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    integrator_program_pda: &Keypair,
    integrator_chain_config: Pubkey,
    outbox_message: Pubkey,
    outbox_message_key: Pubkey,
    integrator_program_id: Pubkey,
    integrator_program_pda_bump: u8,
    dst_chain: u16,
    dst_addr: UniversalAddress,
    payload_hash: [u8; 32],
) -> Result<(), BanksClientError> {
    let accounts = SendMessage {
        integrator_program_pda: integrator_program_pda.pubkey(),
        payer: payer.pubkey(),
        integrator_chain_config,
        outbox_message,
        outbox_message_key,
        system_program: solana_sdk::system_program::id(),
    };

    let args = SendMessageArgs {
        integrator_program_id,
        integrator_program_pda_bump,
        dst_chain,
        dst_addr,
        payload_hash,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::SendMessage { args }.data(),
    };

    execute_transaction(context, ix, &[integrator_program_pda, payer], payer).await
}
