use anchor_lang::{InstructionData, ToAccountMetas};
use mock_integrator::accounts::InvokeRegister;
use router::instructions::RegisterArgs;
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
    let (integrator_program_pda, integrator_program_pda_bump) =
        Pubkey::find_program_address(&[b"router_integrator"], &integrator_program_id);

    let (_, integrator_config_bump) = Pubkey::find_program_address(
        &[
            router::state::IntegratorConfig::SEED_PREFIX,
            integrator_program_id.as_ref(),
        ],
        &router::id(),
    );

    let accounts = InvokeRegister {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        integrator_program_pda,
        system_program: solana_sdk::system_program::id(),
        router_program: router::id(),
    };

    let args = RegisterArgs {
        integrator_program_id,
        integrator_config_bump,
        integrator_program_pda_bump,
    };

    let ix = Instruction {
        program_id: mock_integrator::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_integrator::instruction::InvokeRegister { args }.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
