use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::InitIntegratorConfig;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn initialize_integrator_config(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    owner: Pubkey,
    integrator_config: Pubkey,
    integrator_program: &Keypair,
) -> Result<(), BanksClientError> {
    let accounts = InitIntegratorConfig {
        payer: payer.pubkey(),
        owner,
        integrator_config,
        integrator_program: integrator_program.pubkey(),
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::InitIntegratorConfig {}.data(),
    };

    execute_transaction(context, ix, &[integrator_program, payer], payer).await
}
