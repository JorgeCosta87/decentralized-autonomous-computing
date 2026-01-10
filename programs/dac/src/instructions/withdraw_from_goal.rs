use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::ErrorCode;
use crate::state::{Contribution, Goal, GoalStatus};
use crate::NetworkConfig;

#[derive(Accounts)]
pub struct WithdrawFromGoal<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"goal", network_config.key().as_ref(), goal.goal_slot_id.to_le_bytes().as_ref()],
        bump = goal.bump,
    )]
    pub goal: Account<'info, Goal>,

    #[account(
        mut,
        seeds = [b"goal_vault", goal.key().as_ref()],
        bump = goal.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"contribution", goal.key().as_ref(), contributor.key().as_ref()],
        bump = contribution.bump,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawFromGoal<'info> {
    pub fn withdraw_from_goal(&mut self, shares_to_burn: u64) -> Result<()> {
        require!(
            self.goal.status == GoalStatus::Active,
            ErrorCode::InvalidGoalStatus
        );
        require!(shares_to_burn > 0, ErrorCode::Overflow);
        require!(
            self.contribution.shares >= shares_to_burn,
            ErrorCode::Underflow
        );

        // Exclude rent lamports from share price calculation
        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(0);
        let available_balance = self
            .vault
            .lamports()
            .checked_sub(self.goal.locked_for_tasks)
            .ok_or(ErrorCode::Underflow)?
            .checked_sub(rent_exempt_minimum)
            .ok_or(ErrorCode::Underflow)?;
        let share_price = (available_balance as f64) / (self.goal.total_shares as f64);

        let withdraw_amount = (shares_to_burn as f64 * share_price) as u64;
        // available_balance already excludes rent and locked_for_tasks
        require!(
            withdraw_amount <= available_balance,
            ErrorCode::InsufficientBalance
        );

        let goal_key = self.goal.key();
        let vault_seeds = &[b"goal_vault", goal_key.as_ref(), &[self.goal.vault_bump]];
        let vault_signer = &[&vault_seeds[..]];

        let cpi_accounts = system_program::Transfer {
            from: self.vault.to_account_info(),
            to: self.contributor.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            vault_signer,
        );
        system_program::transfer(cpi_context, withdraw_amount)?;

        // Update contribution shares
        self.contribution.shares = self
            .contribution
            .shares
            .checked_sub(shares_to_burn)
            .ok_or(ErrorCode::Underflow)?;

        // Update goal total shares
        self.goal.total_shares = self
            .goal
            .total_shares
            .checked_sub(shares_to_burn)
            .ok_or(ErrorCode::Underflow)?;

        Ok(())
    }
}
