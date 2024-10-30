use anchor_lang::{InstructionData, ToAccountMetas};
use mock_transceiver::{accounts::InvokeAttestMessage, InvokeAttestMessageArgs};
use router::state::AttestationInfo;
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
    transceiver_info: Pubkey,
    transceiver_pda: Pubkey,
    integrator_chain_config: Pubkey,
    src_chain: u16,
    src_addr: UniversalAddress,
    sequence: u64,
    dst_chain: u16,
    dst_addr: UniversalAddress,
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
    let accounts = InvokeAttestMessage {
        payer: payer.pubkey(),
        transceiver_info,
        transceiver_pda,
        integrator_chain_config,
        attestation_info,
        system_program: solana_sdk::system_program::id(),
        router_program: router::id(),
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
        program_id: mock_transceiver::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_transceiver::instruction::InvokeAttestMessage { args }.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
