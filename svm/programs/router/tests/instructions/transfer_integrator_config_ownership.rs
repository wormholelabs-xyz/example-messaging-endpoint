use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::TransferIntegratorConfigOwnership;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn transfer_integrator_config_ownership(
    context: &mut ProgramTestContext,
    current_admin: &Keypair,
    new_admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_program: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = TransferIntegratorConfigOwnership {
        admin: current_admin.pubkey(),
        new_admin: new_admin.pubkey(),
        integrator_config,
        integrator_program,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::TransferIntegratorConfigOwnership {}.data(),
    };

    execute_transaction(context, ix, &[current_admin, new_admin, payer], payer).await
}
