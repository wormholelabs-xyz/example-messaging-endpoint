use anchor_lang::prelude::*;
use router::cpi::accounts::{AttestMessage, PickUpMessage};
use router::instructions::{AttestMessageArgs, PickUpMessageArgs};
use router::program::Router;
use router::{self};
use universal_address::UniversalAddress;

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
            PickUpMessageArgs {
                transceiver_program_id: crate::id(),
                transceiver_pda_bump: ctx.bumps.transceiver_pda,
            },
        )?;

        Ok(())
    }

    /// Invokes the attest_message instruction on the router program via CPI
    pub fn invoke_attest_message(
        ctx: Context<InvokeAttestMessage>,
        args: InvokeAttestMessageArgs,
    ) -> Result<()> {
        // Prepare the seeds for PDA signing
        let bump_seed = &[ctx.bumps.transceiver_pda][..];
        let signer_seeds: &[&[&[u8]]] = &[&[b"transceiver_pda", bump_seed]];

        // Perform the CPI call to the router program's attest_message instruction
        router::cpi::attest_message(
            ctx.accounts
                .invoke_attest_message()
                .with_signer(signer_seeds),
            AttestMessageArgs {
                transceiver_program_id: crate::id(),
                transceiver_pda_bump: ctx.bumps.transceiver_pda,
                src_chain: args.src_chain,
                src_addr: args.src_addr,
                sequence: args.sequence,
                dst_chain: args.dst_chain,
                integrator_program_id: args.integrator_program_id,
                payload_hash: args.payload_hash,
                message_hash: args.message_hash,
            },
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

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InvokeAttestMessageArgs {
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub integrator_program_id: Pubkey,
    pub payload_hash: [u8; 32],
    pub message_hash: [u8; 32],
}

/// Accounts struct for the invoke_attest_message instruction
#[derive(Accounts)]
pub struct InvokeAttestMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The transceiver info account
    /// CHECK: This account is checked by the router program
    pub transceiver_info: UncheckedAccount<'info>,

    /// The transceiver PDA account, used for signing
    #[account(
        seeds = [b"transceiver_pda"],
        bump,
    )]
    pub transceiver_pda: SystemAccount<'info>,

    /// The integrator chain config account
    /// CHECK: This account is checked by the router program
    pub integrator_chain_config: UncheckedAccount<'info>,

    /// The attestation info account
    /// CHECK: This account is checked by the router program
    #[account(mut)]
    pub attestation_info: UncheckedAccount<'info>,

    /// The system program
    pub system_program: Program<'info, System>,

    /// The router program
    pub router_program: Program<'info, Router>,
}

impl<'info> InvokeAttestMessage<'info> {
    /// Helper function to create the CpiContext for the attest_message instruction
    pub fn invoke_attest_message(&self) -> CpiContext<'_, '_, '_, 'info, AttestMessage<'info>> {
        let cpi_program = self.router_program.to_account_info();
        let cpi_accounts = AttestMessage {
            payer: self.payer.to_account_info(),
            transceiver_info: self.transceiver_info.to_account_info(),
            transceiver_pda: self.transceiver_pda.to_account_info(),
            integrator_chain_config: self.integrator_chain_config.to_account_info(),
            attestation_info: self.attestation_info.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
