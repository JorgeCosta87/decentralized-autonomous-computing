use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{Task, TaskStatus};
use crate::Goal;

#[derive(Accounts)]
pub struct SubmitTaskResult<'info> {
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
        seeds = [b"goal", network_config.key().as_ref(), goal.goal_slot_id.to_le_bytes().as_ref()],
        bump = goal.bump,
    )]
    pub goal: Account<'info, Goal>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, crate::NetworkConfig>,
}

impl<'info> SubmitTaskResult<'info> {
    pub fn submit_task_result(
        &mut self,
        input_cid: String,
        output_cid: String,
        next_input_cid: String,
    ) -> Result<()> {
        require!(
            self.task.status == TaskStatus::Processing,
            ErrorCode::InvalidTaskStatus
        );
        require!(
            self.task.compute_node == Some(self.compute_node.key()),
            ErrorCode::InvalidComputeNodePubkey
        );
        require!(input_cid.len() <= 128, ErrorCode::InvalidCID);
        require!(output_cid.len() <= 128, ErrorCode::InvalidCID);

        //TODO: after the first interaction the peding_input will be the the current next_input_cid
        self.task.pending_input_cid = Some(input_cid);
        self.task.pending_output_cid = Some(output_cid);
        self.task.next_input_cid = Some(next_input_cid);
        self.task.status = TaskStatus::AwaitingValidation;

        Ok(())
    }
}
