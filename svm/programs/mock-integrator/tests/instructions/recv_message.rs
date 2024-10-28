use anchor_lang::{InstructionData, ToAccountMetas};
use mock_integrator::accounts::InvokeRecvMessage;
use router::instructions::recv_message::RecvMessageArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use universal_address::UniversalAddress;

use crate::common::execute_transaction::execute_transaction;

pub async fn recv_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    integrator_chain_config: Pubkey,
    attestation_info: Pubkey,
    src_chain: u16,
    src_addr: UniversalAddress,
    sequence: u64,
    dst_chain: u16,
    dst_addr: UniversalAddress,
    payload_hash: [u8; 32],
) -> Result<(), BanksClientError> {
    let (integrator_program_pda, integrator_program_pda_bump) =
        Pubkey::find_program_address(&[b"router_integrator"], &mock_integrator::id());

    let accounts = InvokeRecvMessage {
        payer: payer.pubkey(),
        integrator_program_pda,
        integrator_chain_config,
        attestation_info,
        system_program: solana_sdk::system_program::id(),
        router_program: router::id(),
    };

    let args = RecvMessageArgs {
        integrator_program_pda_bump,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    };

    let ix = Instruction {
        program_id: mock_integrator::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_integrator::instruction::InvokeRecvMessage { args }.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
