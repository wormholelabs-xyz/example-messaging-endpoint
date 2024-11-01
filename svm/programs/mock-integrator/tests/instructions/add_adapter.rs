use anchor_lang::{InstructionData, ToAccountMetas};
use endpoint::accounts::AddAdapter;
use endpoint::instructions::AddAdapterArgs;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

use crate::common::execute_transaction::execute_transaction;

pub async fn add_adapter(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    integrator_config: Pubkey,
    adapter_info: Pubkey,
    integrator_program_id: Pubkey,
    adapter_program_id: Pubkey,
) -> Result<(), BanksClientError> {
    let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &endpoint::id());

    let accounts = AddAdapter {
        payer: payer.pubkey(),
        admin: admin.pubkey(),
        integrator_config,
        adapter_info,
        system_program: solana_sdk::system_program::id(),
        program: endpoint::id(),
        event_authority,
    };

    let args = AddAdapterArgs {
        integrator_program_id,
        adapter_program_id,
    };

    let ix = Instruction {
        program_id: endpoint::id(),
        accounts: accounts.to_account_metas(None),
        data: endpoint::instruction::AddAdapter { args }.data(),
    };

    execute_transaction(context, ix, &[admin, payer], payer).await
}
