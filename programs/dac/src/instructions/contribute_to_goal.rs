use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::ErrorCode;
use crate::state::{Contribution, Goal, GoalStatus};
use crate::NetworkConfig;

#[derive(Accounts)]
pub struct ContributeToGoal<'info> {
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
        init_if_needed,
        payer = contributor,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [b"contribution", goal.key().as_ref(), contributor.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        seeds = [b"dac_network_config"],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> ContributeToGoal<'info> {
    pub fn contribute_to_goal(
        &mut self,
        deposit_amount: u64,
        bumps: &ContributeToGoalBumps,
    ) -> Result<()> {
        require!(
            self.goal.status == GoalStatus::Active,
            ErrorCode::InvalidGoalStatus
        );
        require!(deposit_amount > 0, ErrorCode::Overflow);

        // Handle case where total_shares == 0 (goal was set but all funds withdrawn)
        // In this case, treat it like first deposit (share_price = 1.0)
        let share_price = if self.goal.total_shares == 0 {
            1.0_f64
        } else {
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
            (available_balance as f64) / (self.goal.total_shares as f64)
        };

        let shares_to_mint = (deposit_amount as f64 / share_price) as u64;
        require!(shares_to_mint > 0, ErrorCode::Overflow);

        let cpi_accounts = system_program::Transfer {
            from: self.contributor.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_context = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
        system_program::transfer(cpi_context, deposit_amount)?;

        let goal_key = self.goal.key();
        let contributor_key = self.contributor.key();

        if self.contribution.goal == Pubkey::default() {
            self.contribution.goal = goal_key;
            self.contribution.contributor = contributor_key;
            self.contribution.shares = shares_to_mint;
            self.contribution.refund_amount = 0;
            self.contribution.bump = bumps.contribution;
        } else {
            require_keys_eq!(
                self.contribution.goal,
                goal_key,
                ErrorCode::InvalidPDAAccount
            );
            require_keys_eq!(
                self.contribution.contributor,
                contributor_key,
                ErrorCode::InvalidPDAAccount
            );

            // Add additional shares
            self.contribution.shares = self
                .contribution
                .shares
                .checked_add(shares_to_mint)
                .ok_or(ErrorCode::Overflow)?;
        }

        // Update goal total shares
        self.goal.total_shares = self
            .goal
            .total_shares
            .checked_add(shares_to_mint)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }
}
