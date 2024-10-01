use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::RegisterIntegrator;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::common::setup::TestContext;

pub async fn register_integrator(
    context: &mut TestContext,
    owner: &Keypair,
    config_pda: Pubkey,
    integrator_pda: Pubkey,
    integrator_authority: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = RegisterIntegrator {
        config: config_pda,
        owner: owner.pubkey(),
        payer: context.payer.pubkey(),
        integrator: integrator_pda,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::RegisterIntegrator {
            authority: integrator_authority,
        }
        .data(),
    };

    let recent_blockhash = context.banks_client.get_latest_blockhash().await?;

    let transaction = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        recent_blockhash,
    );

    context.banks_client.process_transaction(transaction).await
}

pub fn get_integrator_pda(integrator_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            router::state::Integrator::SEED_PREFIX,
            &integrator_id.to_le_bytes(),
        ],
        &router::id(),
    )
}
