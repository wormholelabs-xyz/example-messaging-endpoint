use anchor_lang::{InstructionData, ToAccountMetas};
use endpoint::state::SequenceTracker;
use mock_integrator::{accounts::InvokeRegister, InvokeRegisterArgs};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn register(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    admin: &Keypair,
    integrator_config: Pubkey,
    integrator_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let (integrator_program_pda, _) =
        Pubkey::find_program_address(&[b"endpoint_integrator"], &integrator_program_id);
    let (event_authority, _) =
        Pubkey::find_program_address(&[b"__event_authority"], &endpoint::id());
    let (sequence_tracker, _) = SequenceTracker::pda(&integrator_program_id);

    let accounts = InvokeRegister {
        payer: payer.pubkey(),
        integrator_config,
        sequence_tracker,
        integrator_program_pda,
        event_authority,
        program: endpoint::id(),
        system_program: solana_sdk::system_program::id(),
        endpoint_program: endpoint::id(),
    };

    let args = InvokeRegisterArgs {
        admin: admin.pubkey(),
    };

    let ix = Instruction {
        program_id: mock_integrator::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_integrator::instruction::InvokeRegister { args }.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
