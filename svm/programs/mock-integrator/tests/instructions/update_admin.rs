use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::UpdateAdmin;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn update_admin(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    new_admin: &Pubkey,
    payer: &Keypair,
    integrator_config: Pubkey,
    integrator_program: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = UpdateAdmin {
        admin: admin.pubkey(),
        new_admin: *new_admin,
        integrator_config,
        integrator_program,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::UpdateAdmin {}.data(),
    };

    execute_transaction(context, ix, &[admin, payer], payer).await
}
