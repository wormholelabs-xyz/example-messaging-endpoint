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
    pub fn invoke_register(ctx: Context<InvokeRegister>, args: RegisterArgs) -> Result<()> {
        let bump_seed = &[args.integrator_program_pda_bump][..];
        let signer_seeds: &[&[&[u8]]] = &[&[b"router_integrator", bump_seed]];

        router::cpi::register(
            ctx.accounts.invoke_register().with_signer(signer_seeds),
            args,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(args: RegisterArgs)]
pub struct InvokeRegister<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: This account is not checked for safety because it is assumed to be a trusted admin account.
    pub admin: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: This account is to be checked and initialized by the router program
    pub integrator_config: UncheckedAccount<'info>,

    /// The integrator program's PDA
    #[account(
        seeds = [b"router_integrator"],
        bump = args.integrator_program_pda_bump,
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
            admin: self.admin.to_account_info(),
            integrator_config: self.integrator_config.to_account_info(),
            integrator_program_pda: self.integrator_program_pda.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
