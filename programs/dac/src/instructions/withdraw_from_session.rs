use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::ErrorCode;
use crate::state::{Contribution, Session, SessionStatus};
use crate::NetworkConfig;

#[derive(Accounts)]
pub struct WithdrawFromSession<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"session", network_config.key().as_ref(), session.session_slot_id.to_le_bytes().as_ref()],
        bump = session.bump,
    )]
    pub session: Account<'info, Session>,

    #[account(
        mut,
        seeds = [b"session_vault", session.key().as_ref()],
        bump = session.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"contribution", session.key().as_ref(), contributor.key().as_ref()],
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

impl<'info> WithdrawFromSession<'info> {
    pub fn withdraw_from_session(&mut self, shares_to_burn: u64) -> Result<()> {
        require!(
            self.session.status == SessionStatus::Active,
            ErrorCode::InvalidSessionStatus
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
            .checked_sub(self.session.locked_for_tasks)
            .ok_or(ErrorCode::Underflow)?
            .checked_sub(rent_exempt_minimum)
            .ok_or(ErrorCode::Underflow)?;
        let share_price = (available_balance as f64) / (self.session.total_shares as f64);

        let withdraw_amount = (shares_to_burn as f64 * share_price) as u64;
        // available_balance already excludes rent and locked_for_tasks
        require!(
            withdraw_amount <= available_balance,
            ErrorCode::InsufficientBalance
        );

        let session_key = self.session.key();
        let vault_seeds = &[b"session_vault", session_key.as_ref(), &[self.session.vault_bump]];
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
        self.session.total_shares = self
            .session
            .total_shares
            .checked_sub(shares_to_burn)
            .ok_or(ErrorCode::Underflow)?;

        Ok(())
    }
}
