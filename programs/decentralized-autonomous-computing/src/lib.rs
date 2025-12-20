use anchor_lang::prelude::*;

declare_id!("ANgS5PVGWJ72rBbidEgpp1CKYjDs1sXNNyDEwW4cfSjx");

#[program]
pub mod decentralized_autonomous_computing {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
