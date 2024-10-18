use anchor_lang::{InstructionData, ToAccountMetas};
use router::accounts::AddTransceiver;
use router::instructions::AddTransceiverArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn add_transceiver(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    transceiver_info: Pubkey,
    integrator_program_id: Pubkey,
    transceiver_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let accounts = AddTransceiver {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        transceiver_info,
        system_program: solana_sdk::system_program::id(),
    };

    let args = AddTransceiverArgs {
        integrator_program_id,
        transceiver_program_id,
    };

    let ix = Instruction {
        program_id: router::id(),
        accounts: accounts.to_account_metas(None),
        data: router::instruction::AddTransceiver { args }.data(),
    };

    execute_transaction(context, ix, &[admin, payer], payer).await
}
