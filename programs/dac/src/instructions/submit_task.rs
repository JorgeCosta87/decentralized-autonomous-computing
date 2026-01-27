use anchor_lang::prelude::*;

use crate::SessionStatus;
use crate::errors::ErrorCode;
use crate::state::{Task, TaskStatus};
use crate::state::Session;

#[derive(Accounts)]
pub struct SubmitTask<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"task", network_config.key().as_ref(), task.task_slot_id.to_le_bytes().as_ref()],
        bump = task.bump,
    )]
    pub task: Account<'info, Task>,
    #[account(
        mut,
        has_one = owner,
        seeds = [b"session", network_config.key().as_ref(), session.session_slot_id.to_le_bytes().as_ref()],
        bump = session.bump,
    )]
    pub session: Account<'info, Session>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, crate::NetworkConfig>,
}

impl<'info> SubmitTask<'info> {
    pub fn submit_task(
        &mut self,
        input_cid: String,
    ) -> Result<()> {
        require!(
            self.task.status == TaskStatus::Ready,
            ErrorCode::InvalidTaskStatus
        );
        require!(self.session.status == SessionStatus::Active, ErrorCode::InvalidSessionStatus);
        require!(self.task.session_slot_id == Some(self.session.session_slot_id), ErrorCode::InvalidSession);

        self.task.pending_input_cid = Some(input_cid.clone());
        self.task.status = TaskStatus::Pending;

        Ok(())
    }
}
