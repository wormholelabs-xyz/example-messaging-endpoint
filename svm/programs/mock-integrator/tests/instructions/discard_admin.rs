use anchor_lang::{InstructionData, ToAccountMetas};
use endpoint::accounts::DiscardAdmin;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn discard_admin(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = DiscardAdmin {
        admin: admin.pubkey(),
        integrator_config,
    };

    let ix = Instruction {
        program_id: endpoint::id(),
        accounts: accounts.to_account_metas(None),
        data: endpoint::instruction::DiscardAdmin {}.data(),
    };

    execute_transaction(context, ix, &[admin, payer], payer).await
}
