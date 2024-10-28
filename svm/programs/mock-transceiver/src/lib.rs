use anchor_lang::prelude::*;
use router::cpi::accounts::PickUpMessage;
use router::program::Router;
use router::{self};

// Declare the program ID for the mock transceiver
declare_id!("4ZiURKmq17KrwY3K2KxxzqttzytxQsqAcMy374QUi7tx");

#[program]
pub mod mock_transceiver {
    use super::*;

    /// Invokes the pick_up_message instruction on the router program via CPI
    ///
    /// This function demonstrates how a transceiver program would interact
    /// with the router program to pick up a message from the outbox.
    pub fn invoke_pick_up_message(ctx: Context<InvokePickUpMessage>) -> Result<()> {
        // Prepare the seeds for PDA signing
        let bump_seed = &[ctx.bumps.transceiver_pda][..];
        let signer_seeds: &[&[&[u8]]] = &[&[b"transceiver_pda", bump_seed]];

        // Perform the CPI call to the router program's pick_up_message instruction
        router::cpi::pick_up_message(
            ctx.accounts
                .invoke_pick_up_message()
                .with_signer(signer_seeds),
        )?;

        Ok(())
    }
}

/// Accounts struct for the invoke_pick_up_message instruction
#[derive(Accounts)]
pub struct InvokePickUpMessage<'info> {
    #[account(mut)]
    /// The outbox message account to be picked up
    /// CHECK: This account is checked by the router program
    pub outbox_message: UncheckedAccount<'info>,

    /// The transceiver info account
    /// CHECK: This account is checked by the router program
    pub transceiver_info: UncheckedAccount<'info>,

    /// The transceiver PDA account, used for signing
    #[account(
        seeds = [b"transceiver_pda"],
        bump,
    )]
    pub transceiver_pda: SystemAccount<'info>,

    #[account(mut)]
    /// CHECK: this is a refund recipient that will be passed in by integrator
    pub refund_recipient: AccountInfo<'info>,

    /// The system program
    pub system_program: Program<'info, System>,

    /// The router program
    pub router_program: Program<'info, Router>,
}

impl<'info> InvokePickUpMessage<'info> {
    /// Helper function to create the CpiContext for the pick_up_message instruction
    pub fn invoke_pick_up_message(&self) -> CpiContext<'_, '_, '_, 'info, PickUpMessage<'info>> {
        let cpi_program = self.router_program.to_account_info();
        let cpi_accounts = PickUpMessage {
            outbox_message: self.outbox_message.to_account_info(),
            transceiver_info: self.transceiver_info.to_account_info(),
            transceiver_pda: self.transceiver_pda.to_account_info(),
            refund_recipient: self.refund_recipient.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
