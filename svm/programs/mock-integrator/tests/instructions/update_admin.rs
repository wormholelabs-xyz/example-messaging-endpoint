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
    integrator_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = UpdateAdmin {
        admin: admin.pubkey(),
        integrator_config,
    };

    let args = router::instructions::UpdateAdminArgs {
        integrator_program_id,
        new_admin: *new_admin,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::UpdateAdmin { args }.data(),
    };

    execute_transaction(context, ix, &[admin, payer], payer).await
}
