use anchor_lang::prelude::*;
use endpoint::cpi::accounts::{AttestMessage, PickUpMessage};
use endpoint::instructions::{AttestMessageArgs, PickUpMessageArgs};
use endpoint::program::Endpoint;
use endpoint::{self};

// Declare the program ID for the mock adapter
declare_id!("5k8XySmYJ6nQTF8ZFZtRoevjCx9Y9PS5MT9oJDLNA162");

#[program]
pub mod mock_adapter {

    use super::*;

    /// Invokes the pick_up_message instruction on the endpoint program via CPI
    ///
    /// This function demonstrates how an adapter program would interact
    /// with the endpoint program to pick up a message from the outbox.
    pub fn invoke_pick_up_message(ctx: Context<InvokePickUpMessage>) -> Result<()> {
        // Prepare the seeds for PDA signing
        let bump_seed = &[ctx.bumps.adapter_pda][..];
        let signer_seeds: &[&[&[u8]]] = &[&[b"adapter_pda", bump_seed]];

        // Perform the CPI call to the endpoint program's pick_up_message instruction
        endpoint::cpi::pick_up_message(
            ctx.accounts
                .invoke_pick_up_message()
                .with_signer(signer_seeds),
            PickUpMessageArgs {
                adapter_program_id: crate::id(),
                adapter_pda_bump: ctx.bumps.adapter_pda,
            },
        )?;

        Ok(())
    }

    /// Invokes the attest_message instruction on the endpoint program via CPI
    pub fn invoke_attest_message(
        ctx: Context<InvokeAttestMessage>,
        args: InvokeAttestMessageArgs,
    ) -> Result<()> {
        // Prepare the seeds for PDA signing
        let bump_seed = &[ctx.bumps.adapter_pda][..];
        let signer_seeds: &[&[&[u8]]] = &[&[b"adapter_pda", bump_seed]];

        // Perform the CPI call to the endpoint program's attest_message instruction
        endpoint::cpi::attest_message(
            ctx.accounts
                .invoke_attest_message()
                .with_signer(signer_seeds),
            AttestMessageArgs {
                adapter_program_id: crate::id(),
                adapter_pda_bump: ctx.bumps.adapter_pda,
                src_chain: args.src_chain,
                src_addr: args.src_addr,
                sequence: args.sequence,
                dst_chain: args.dst_chain,
                integrator_program_id: args.integrator_program_id,
                payload_hash: args.payload_hash,
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
    /// CHECK: This account is checked by the endpoint program
    pub outbox_message: UncheckedAccount<'info>,

    /// The adapter info account
    /// CHECK: This account is checked by the endpoint program
    pub adapter_info: UncheckedAccount<'info>,

    /// The adapter PDA account, used for signing
    #[account(
        seeds = [b"adapter_pda"],
        bump,
    )]
    pub adapter_pda: SystemAccount<'info>,

    /// CHECK: This should be seeded with `__event_authority`
    #[account(
            seeds = [b"__event_authority"],
            bump,
            seeds::program = endpoint::id(),
        )]
    pub event_authority: AccountInfo<'info>,

    /// CHECK: Self-CPI will fail if the program is n
    pub program: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: this is a refund recipient that will be passed in by integrator
    pub refund_recipient: AccountInfo<'info>,

    /// The system program
    pub system_program: Program<'info, System>,

    /// The endpoint program
    pub endpoint_program: Program<'info, Endpoint>,
}

impl<'info> InvokePickUpMessage<'info> {
    /// Helper function to create the CpiContext for the pick_up_message instruction
    pub fn invoke_pick_up_message(&self) -> CpiContext<'_, '_, '_, 'info, PickUpMessage<'info>> {
        let cpi_program = self.endpoint_program.to_account_info();
        let cpi_accounts = PickUpMessage {
            outbox_message: self.outbox_message.to_account_info(),
            adapter_info: self.adapter_info.to_account_info(),
            adapter_pda: self.adapter_pda.to_account_info(),
            event_authority: self.event_authority.to_account_info(),
            program: self.program.to_account_info(),
            refund_recipient: self.refund_recipient.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InvokeAttestMessageArgs {
    pub src_chain: u16,
    pub src_addr: [u8; 32],
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

    /// The adapter info account
    /// CHECK: This account is checked by the endpoint program
    pub adapter_info: UncheckedAccount<'info>,

    /// The adapter PDA account, used for signing
    #[account(
        seeds = [b"adapter_pda"],
        bump,
    )]
    pub adapter_pda: SystemAccount<'info>,

    /// The integrator chain config account
    /// CHECK: This account is checked by the endpoint program
    pub integrator_chain_config: UncheckedAccount<'info>,

    /// The attestation info account
    /// CHECK: This account is checked by the endpoint program
    #[account(mut)]
    pub attestation_info: UncheckedAccount<'info>,

    /// The event authority PDA
    /// CHECK: This should be seeded with `__event_authority`
    #[account(
        seeds = [b"__event_authority"],
        bump,
        seeds::program = endpoint::id(),
    )]
    pub event_authority: AccountInfo<'info>,

    /// CHECK: Self-CPI will fail if the program is n
    pub program: AccountInfo<'info>,

    /// The system program
    pub system_program: Program<'info, System>,

    /// The endpoint program
    pub endpoint_program: Program<'info, Endpoint>,
}

impl<'info> InvokeAttestMessage<'info> {
    /// Helper function to create the CpiContext for the attest_message instruction
    pub fn invoke_attest_message(&self) -> CpiContext<'_, '_, '_, 'info, AttestMessage<'info>> {
        let cpi_program = self.endpoint_program.to_account_info();
        let cpi_accounts = AttestMessage {
            payer: self.payer.to_account_info(),
            adapter_info: self.adapter_info.to_account_info(),
            adapter_pda: self.adapter_pda.to_account_info(),
            integrator_chain_config: self.integrator_chain_config.to_account_info(),
            attestation_info: self.attestation_info.to_account_info(),
            event_authority: self.event_authority.to_account_info(),
            program: self.program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
