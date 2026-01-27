use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::ErrorCode;
use crate::events::SessionSet;
use crate::state::{Agent, AgentStatus, Contribution, Session, SessionStatus, Task, TaskStatus};
use crate::NetworkConfig;
use crate::TaskType;

#[derive(Accounts)]
pub struct SetSession<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"session", network_config.key().as_ref(), session.session_slot_id.to_le_bytes().as_ref()],
        bump = session.bump,
    )]
    pub session: Account<'info, Session>,

    #[account(
        mut,
        seeds = [b"session_vault", session.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [b"contribution", session.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub owner_contribution: Account<'info, Contribution>,

    #[account(
        mut,
        seeds = [b"task", network_config.key().as_ref(), task.task_slot_id.to_le_bytes().as_ref()],
        bump = task.bump,
    )]
    pub task: Account<'info, Task>,

    #[account(
        seeds = [b"agent", network_config.key().as_ref(), agent.agent_slot_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> SetSession<'info> {
    pub fn set_session(
        &mut self,
        specification_cid: String,
        max_iterations: u64,
        initial_deposit: u64,
        compute_node: Pubkey,
        task_type: TaskType,
        bumps: &SetSessionBumps,
    ) -> Result<()> {
        require!(
            self.session.status == SessionStatus::Pending,
            ErrorCode::InvalidSessionStatus
        );
        require!(
            self.session.owner == Pubkey::default() || self.session.owner == self.owner.key(),
            ErrorCode::InvalidSessionOwner
        );
        require!(
            self.task.status == TaskStatus::Ready,
            ErrorCode::InvalidTaskStatus
        );
        require!(
            self.agent.status == AgentStatus::Active,
            ErrorCode::InvalidAgentStatus
        );
        require!(initial_deposit > 0, ErrorCode::DepositTooSmall);

        let approved = if self.session.is_confidential {
            &self.network_config.approved_confidential_nodes
        } else {
            &self.network_config.approved_public_nodes
        };
        require!(
            approved.contains(&compute_node),
            ErrorCode::InvalidComputeNodePubkey
        );

        // Check if vault only has rent lamports (no leftover SOL from previous goal)
        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(0);
        let vault_balance = self.vault.lamports();
        require!(
            vault_balance == rent_exempt_minimum || vault_balance == 0,
            ErrorCode::VaultHasLeftoverFunds
        );

        if self.session.current_iteration > 0 {
            self.session.current_iteration = 0;
            self.session.task_index_start = self.session.task_index_end;
            self.session.task_index_end = 0;
            self.session.total_shares = 0;
            self.session.locked_for_tasks = 0;
        }

        let session_key = self.session.key();
        let vault_seeds = &[b"session_vault", session_key.as_ref(), &[bumps.vault]];
        let vault_signer = &[&vault_seeds[..]];

        if vault_balance == 0 {
            // Vault doesn't exist
            let required_lamports = rent_exempt_minimum;
            let transfer_amount = initial_deposit
                .checked_add(required_lamports)
                .ok_or(ErrorCode::Overflow)?;

            let cpi_accounts = system_program::CreateAccount {
                from: self.owner.to_account_info(),
                to: self.vault.to_account_info(),
            };
            let cpi_context = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                cpi_accounts,
                vault_signer,
            );

            system_program::create_account(cpi_context, transfer_amount, 0, &system_program::ID)?;
        } else {
            // Vault exists with only rent
            let cpi_accounts = system_program::Transfer {
                from: self.owner.to_account_info(),
                to: self.vault.to_account_info(),
            };
            let cpi_context = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
            system_program::transfer(cpi_context, initial_deposit)?;
        }

        // Mint shares for owner's initial deposit
        // First deposit always uses share_price = 1.0
        let share_price = 1.0_f64;
        let shares = (initial_deposit as f64 / share_price) as u64;
        require!(shares > 0, ErrorCode::Overflow);

        self.owner_contribution.set_inner(Contribution {
            session: self.session.key(),
            contributor: self.owner.key(),
            shares,
            refund_amount: 0,
            bump: bumps.owner_contribution,
        });

        self.session.owner = self.owner.key();
        self.session.task = self.task.key();
        self.session.specification_cid = specification_cid;
        self.session.max_iterations = max_iterations;
        self.session.total_shares = shares;
        self.session.status = SessionStatus::Active;
        self.session.vault_bump = bumps.vault;
        self.session.task_index_start = self.task.task_index;

        self.task.compute_node = Some(compute_node);
        self.task.status = TaskStatus::Ready;
        self.task.task_type = task_type;

        emit!(SessionSet {
            session_slot_id: self.session.session_slot_id,
            owner: self.owner.key(),
            task_slot_id: self.task.task_slot_id,
            specification_cid: self.session.specification_cid.clone(),
            max_iterations: self.session.max_iterations,
            initial_deposit,
        });

        Ok(())
    }
}
