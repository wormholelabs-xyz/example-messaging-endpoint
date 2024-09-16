use anchor_lang::prelude::*;

declare_id!("Gfo1Jn4zHvc8BBGWNPQpNDZob5DsG2bhmS4wEA2GKFx6");

#[program]
pub mod svm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
