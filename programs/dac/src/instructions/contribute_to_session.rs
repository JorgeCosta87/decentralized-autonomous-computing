use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::ErrorCode;
use crate::events::ContributionMade;
use crate::state::{Contribution, Session, SessionStatus};
use crate::NetworkConfig;

#[derive(Accounts)]
pub struct ContributeToSession<'info> {
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
        init_if_needed,
        payer = contributor,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [b"contribution", session.key().as_ref(), contributor.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> ContributeToSession<'info> {
    pub fn contribute_to_session(
        &mut self,
        deposit_amount: u64,
        bumps: &ContributeToSessionBumps,
    ) -> Result<()> {
        require!(
            self.session.status == SessionStatus::Active,
            ErrorCode::InvalidSessionStatus
        );
        require!(deposit_amount > 0, ErrorCode::Overflow);

        let share_price = if self.session.total_shares == 0 {
            1.0_f64
        } else {
            let rent = Rent::get()?;
            let rent_exempt_minimum = rent.minimum_balance(0);
            let available_balance = self
                .vault
                .lamports()
                .checked_sub(self.session.locked_for_tasks)
                .ok_or(ErrorCode::Underflow)?
                .checked_sub(rent_exempt_minimum)
                .ok_or(ErrorCode::Underflow)?;
            (available_balance as f64) / (self.session.total_shares as f64)
        };

        let shares_to_mint = (deposit_amount as f64 / share_price) as u64;
        require!(shares_to_mint > 0, ErrorCode::Overflow);

        let cpi_accounts = system_program::Transfer {
            from: self.contributor.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_context = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
        system_program::transfer(cpi_context, deposit_amount)?;

        let session_key = self.session.key();
        let contributor_key = self.contributor.key();

        if self.contribution.session == Pubkey::default() {
            self.contribution.session = session_key;
            self.contribution.contributor = contributor_key;
            self.contribution.shares = shares_to_mint;
            self.contribution.refund_amount = 0;
            self.contribution.bump = bumps.contribution;
        } else {
            require_keys_eq!(
                self.contribution.session,
                session_key,
                ErrorCode::InvalidPDAAccount
            );
            require_keys_eq!(
                self.contribution.contributor,
                contributor_key,
                ErrorCode::InvalidPDAAccount
            );

            self.contribution.shares = self
                .contribution
                .shares
                .checked_add(shares_to_mint)
                .ok_or(ErrorCode::Overflow)?;
        }

        self.session.total_shares = self
            .session
            .total_shares
            .checked_add(shares_to_mint)
            .ok_or(ErrorCode::Overflow)?;

        emit!(ContributionMade {
            session_slot_id: self.session.session_slot_id,
            contributor: self.contributor.key(),
            deposit_amount,
            shares_minted: shares_to_mint,
            total_shares: self.session.total_shares,
        });

        Ok(())
    }
}
