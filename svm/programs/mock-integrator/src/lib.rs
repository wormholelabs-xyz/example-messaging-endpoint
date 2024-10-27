use anchor_lang::prelude::*;
use router::program::Router;
use router::{self};
use router::{cpi::accounts::Register, instructions::RegisterArgs};

declare_id!("B86KSKnHBRiJeDcP7vwaXuxfkqfChZmYKBqh4dkLYEpj");

/// This module serves as a mock integrator to demonstrate how to call the register function
/// in the router program. It's designed to simulate the process of registering an integrator,
/// which requires a Cross-Program Invocation (CPI) call with a Program Derived Address (PDA) signer.

#[program]
pub mod mock_integrator {
    use super::*;

    /// Invokes the register function in the router program via a CPI call.
    /// This function demonstrates how to properly set up the accounts and sign the transaction
    /// using a PDA, which is required for the registration process.
    pub fn invoke_register(ctx: Context<InvokeRegister>, args: InvokeRegisterArgs) -> Result<()> {
        let bump_seed = &[ctx.bumps.integrator_program_pda][..];
        let signer_seeds: &[&[&[u8]]] = &[&[b"router_integrator", bump_seed]];

        router::cpi::register(
            ctx.accounts.invoke_register().with_signer(signer_seeds),
            RegisterArgs {
                integrator_program_pda_bump: ctx.bumps.integrator_program_pda,
                integrator_program_id: crate::ID,
                admin: args.admin,
            },
        )?;
        Ok(())
    }
}
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InvokeRegisterArgs {
    pub admin: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: InvokeRegisterArgs)]
pub struct InvokeRegister<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    /// CHECK: This account is to be checked and initialized by the router program
    pub integrator_config: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: This account is to be checked and initialized by the router program
    pub outbox_message_key: UncheckedAccount<'info>,

    /// The integrator program's PDA
    #[account(
        seeds = [b"router_integrator"],
        bump,
    )]
    pub integrator_program_pda: SystemAccount<'info>,

    /// The System Program
    pub system_program: Program<'info, System>,

    pub router_program: Program<'info, Router>,
}

impl<'info> InvokeRegister<'info> {
    pub fn invoke_register(&self) -> CpiContext<'_, '_, '_, 'info, Register<'info>> {
        let cpi_program = self.router_program.to_account_info();
        let cpi_accounts = Register {
            payer: self.payer.to_account_info(),
            integrator_config: self.integrator_config.to_account_info(),
            integrator_program_pda: self.integrator_program_pda.to_account_info(),
            outbox_message_key: self.outbox_message_key.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
