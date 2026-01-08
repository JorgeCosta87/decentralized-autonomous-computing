use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Contribution {
    pub goal: Pubkey,
    pub contributor: Pubkey,
    pub shares: u64,
    pub refund_amount: u64,
    pub bump: u8,
}
