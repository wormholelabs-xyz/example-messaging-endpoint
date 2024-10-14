use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::RegisterTransceiver;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn register_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    registered_transceiver: Pubkey,
    integrator_program: Pubkey,
    transceiver_address: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = RegisterTransceiver {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        registered_transceiver,
        integrator_program,
        transceiver_address,
        system_program: solana_sdk::system_program::id(),
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::RegisterTransceiver {}.data(),
    };

    execute_transaction(context, ix, &[admin, payer], payer).await
}
