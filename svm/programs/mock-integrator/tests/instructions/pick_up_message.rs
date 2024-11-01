use anchor_lang::{InstructionData, ToAccountMetas};
use mock_adapter::accounts::InvokePickUpMessage;
use solana_program_test::*;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signer::keypair::Keypair};

use crate::common::execute_transaction::execute_transaction;

pub async fn pick_up_message(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    outbox_message: Pubkey,
    adapter_info: Pubkey,
    adapter_pda: Pubkey,
    refund_recipient: Pubkey,
) -> Result<(), BanksClientError> {
    let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &endpoint::id());

    let accounts = InvokePickUpMessage {
        outbox_message,
        adapter_info,
        adapter_pda,
        system_program: solana_sdk::system_program::id(),
        endpoint_program: endpoint::id(),
        refund_recipient,
        program: endpoint::id(),
        event_authority,
    };

    let ix = Instruction {
        program_id: mock_adapter::id(),
        accounts: accounts.to_account_metas(None),
        data: mock_adapter::instruction::InvokePickUpMessage {}.data(),
    };

    execute_transaction(context, ix, &[payer], payer).await
}
