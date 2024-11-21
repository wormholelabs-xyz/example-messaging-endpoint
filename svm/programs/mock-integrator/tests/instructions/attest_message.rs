use anchor_lang::{InstructionData, ToAccountMetas};
use endpoint::state::AttestationInfo;
use mock_adapter::{accounts::InvokeAttestMessage, InvokeAttestMessageArgs};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use universal_address::UniversalAddress;

use crate::common::execute_transaction::execute_transaction;

pub async fn attest_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    adapter_info: Pubkey,
    adapter_pda: Pubkey,
    integrator_chain_config: Pubkey,
    src_chain: u16,
    src_addr: [u8; 32],
    sequence: u64,
    dst_chain: u16,
    dst_addr: [u8; 32],
    payload_hash: [u8; 32],
) -> Result<(), BanksClientError> {
    let message_hash = AttestationInfo::compute_message_hash(
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        dst_addr,
        payload_hash,
    );
    let (attestation_info, _) = AttestationInfo::pda(message_hash);
    let (event_authority, _) =
        Pubkey::find_program_address(&[b"__event_authority"], &endpoint::id());

    let accounts = InvokeAttestMessage {
        payer: payer.pubkey(),
        adapter_info,
        adapter_pda,
        integrator_chain_config,
        attestation_info,
        system_program: solana_sdk::system_program::id(),
        endpoint_program: endpoint::id(),
        program: endpoint::id(),
        event_authority,
    };

    let args = InvokeAttestMessageArgs {
        src_chain,
        src_addr,
        sequence,
        dst_chain,
        integrator_program_id: mock_integrator::id(),
        payload_hash,
        message_hash,
    };

    let ix = Instruction {
        program_id: mock_adapter::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_adapter::instruction::InvokeAttestMessage { args }.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
