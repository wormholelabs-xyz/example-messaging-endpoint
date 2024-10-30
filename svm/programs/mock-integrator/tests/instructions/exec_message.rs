use anchor_lang::{InstructionData, ToAccountMetas};
use mock_integrator::accounts::InvokeExecMessage;
use router::{instructions::ExecMessageArgs, state::AttestationInfo};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use universal_address::UniversalAddress;

use crate::common::execute_transaction::execute_transaction;

pub async fn exec_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    src_chain: u16,
    src_addr: UniversalAddress,
    sequence: u64,
    dst_chain: u16,
    dst_addr: UniversalAddress,
    payload_hash: [u8; 32],
) -> Result<(), BanksClientError> {
    let (integrator_program_pda, integrator_program_pda_bump) =
        Pubkey::find_program_address(&[b"router_integrator"], &mock_integrator::id());

    let message_hash = AttestationInfo::compute_message_hash(
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    );
    let (attestation_info, _) = AttestationInfo::pda(message_hash);

    let accounts = InvokeExecMessage {
        payer: payer.pubkey(),
        integrator_program_pda,
        attestation_info,
        system_program: solana_sdk::system_program::id(),
        router_program: router::id(),
    };

    let args = ExecMessageArgs {
        integrator_program_pda_bump,
        src_chain,
        src_addr,
        sequence,
        integrator_program_id: mock_integrator::id(),
        dst_chain,
        payload_hash,
        message_hash,
    };

    let ix = Instruction {
        program_id: mock_integrator::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_integrator::instruction::InvokeExecMessage { args }.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
