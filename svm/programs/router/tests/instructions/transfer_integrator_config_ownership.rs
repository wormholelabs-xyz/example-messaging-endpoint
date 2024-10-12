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
    current_owner: &Keypair,
    new_owner: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_program: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = TransferIntegratorConfigOwnership {
        owner: current_owner.pubkey(),
        new_owner: new_owner.pubkey(),
        integrator_config,
        integrator_program,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::TransferIntegratorConfigOwnership {}.data(),
    };

    execute_transaction(context, ix, &[current_owner, new_owner, payer], payer).await
}
