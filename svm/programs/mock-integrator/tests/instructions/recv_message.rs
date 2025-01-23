use anchor_lang::{InstructionData, ToAccountMetas};
use endpoint::{instructions::recv_message::RecvMessageArgs, state::IntegratorChainConfig};
use mock_integrator::accounts::InvokeRecvMessage;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn recv_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    attestation_info: Pubkey,
    src_chain: u16,
    src_addr: [u8; 32],
    sequence: u64,
    dst_chain: u16,
    payload_hash: [u8; 32],
) -> Result<(), BanksClientError> {
    let (integrator_program_pda, integrator_program_pda_bump) =
        Pubkey::find_program_address(&[b"endpoint_integrator"], &mock_integrator::id());
    let (event_authority, _) =
        Pubkey::find_program_address(&[b"__event_authority"], &endpoint::id());
    let integrator_program_id = mock_integrator::id();
    let (integrator_chain_config, _) =
        IntegratorChainConfig::pda(&integrator_program_id, src_chain);

    let accounts = InvokeRecvMessage {
        payer: payer.pubkey(),
        integrator_program_pda,
        attestation_info,
        system_program: solana_sdk::system_program::id(),
        integrator_chain_config,
        endpoint_program: endpoint::id(),
        program: endpoint::id(),
        event_authority,
    };

    let args = RecvMessageArgs {
        integrator_program_pda_bump,
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        integrator_program_id: mock_integrator::id(),
        payload_hash,
    };

    let ix = Instruction {
        program_id: mock_integrator::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_integrator::instruction::InvokeRecvMessage { args }.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
