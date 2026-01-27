use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::events::TaskClaimed;
use crate::state::{
    NetworkConfig, Session, SessionStatus, Task, TaskStatus, ValidationStatus, Validator,
};

#[derive(Accounts)]
pub struct ClaimTask<'info> {
    #[account(mut)]
    pub compute_node: Signer<'info>,

    #[account(
        mut,
        seeds = [b"task", network_config.key().as_ref(), task.task_slot_id.to_le_bytes().as_ref()],
        bump = task.bump,
    )]
    pub task: Account<'info, Task>,

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
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,
}

impl<'info> ClaimTask<'info> {
    pub fn claim_task(&mut self, max_task_cost: u64, max_call_count: u64) -> Result<()> {
        require!(
            self.task.status == TaskStatus::Pending,
            ErrorCode::InvalidTaskStatus
        );
        require!(
            self.session.status == SessionStatus::Active,
            ErrorCode::InvalidSessionStatus
        );
        require!(
            self.task.compute_node == Some(self.compute_node.key()),
            ErrorCode::InvalidComputeNodePubkey
        );
        require!(self.session.total_shares > 0, ErrorCode::Overflow);

        let pool = if self.session.is_confidential {
            &self.network_config.approved_confidential_nodes
        } else {
            &self.network_config.approved_public_nodes
        };
        let compute_pubkey = self.compute_node.key();
        let candidates: Vec<Pubkey> = pool
            .iter()
            .copied()
            .filter(|p| *p != compute_pubkey)
            .collect();
        let required = self.network_config.required_validations;
        require!(
            candidates.len() >= required as usize,
            ErrorCode::NotEnoughValidators
        );

        let clock = Clock::get()?;
        let start_idx = (clock.slot as usize) % candidates.len();
        self.task.validations.clear();
        for i in 0..required {
            let idx = (start_idx + i as usize) % candidates.len();
            self.task.validations.push(Validator {
                pubkey: candidates[idx],
                status: ValidationStatus::Pending,
            });
        }

        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(0);
        let available_balance = self
            .vault
            .lamports()
            .checked_sub(self.session.locked_for_tasks)
            .ok_or(ErrorCode::Underflow)?
            .checked_sub(rent_exempt_minimum)
            .ok_or(ErrorCode::Underflow)?;

        require!(
            available_balance >= max_task_cost,
            ErrorCode::InsufficientBalance
        );

        self.session.locked_for_tasks = self
            .session
            .locked_for_tasks
            .checked_add(max_task_cost)
            .ok_or(ErrorCode::Overflow)?;

        self.task.max_task_cost = max_task_cost;
        self.task.max_call_count = max_call_count;
        self.task.status = TaskStatus::Processing;
        self.task.task_index = self
            .task
            .task_index
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;

        emit!(TaskClaimed {
            session_slot_id: self.session.session_slot_id,
            task_slot_id: self.task.task_slot_id,
            compute_node: compute_pubkey,
            max_task_cost,
        });

        Ok(())
    }
}
