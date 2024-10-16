use anchor_lang::prelude::*;
use router::program::Router;
use router::{self};
use router::{cpi::accounts::Register, instructions::RegisterArgs};

declare_id!("B86KSKnHBRiJeDcP7vwaXuxfkqfChZmYKBqh4dkLYEpj");

#[program]
pub mod mock_integrator {
    use super::*;

    pub fn invoke_register(ctx: Context<InvokeRegister>, args: RegisterArgs) -> Result<()> {
        msg!("Greetings from: {:?} invoke_register", ctx.program_id);
        msg!("Integrator program id: {:?}", args.integrator_program_id);
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
        msg!(
            "Integrator program pda: {:?}",
            self.integrator_program_pda.key()
        );

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
