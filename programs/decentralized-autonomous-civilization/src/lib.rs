use anchor_lang::prelude::*;

declare_id!("ANgS5PVGWJ72rBbidEgpp1CKYjDs1sXNNyDEwW4cfSjx");

pub mod state;
pub mod utils;

#[program]
pub mod decentralized_autonomous_civilization {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
