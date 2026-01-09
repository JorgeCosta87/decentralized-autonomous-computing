use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{Goal, GoalStatus, NetworkConfig, NodeInfo, NodeStatus, NodeType, Task, TaskStatus};

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
        seeds = [b"node_info", compute_node.key().as_ref()],
        bump = compute_node_info.bump,
    )]
    pub compute_node_info: Account<'info, NodeInfo>,

    #[account(
        seeds = [b"dac_network_config"],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,
}

impl<'info> ClaimTask<'info> {
    pub fn claim_task(&mut self, max_task_cost: u64) -> Result<()> {
        require!(
            self.task.status == TaskStatus::Pending,
            ErrorCode::InvalidTaskStatus
        );
        require!(
            self.goal.status == GoalStatus::Active,
            ErrorCode::InvalidGoalStatus
        );
        require!(
            self.compute_node_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.compute_node_info.node_type == NodeType::Compute,
            ErrorCode::InvalidNodeType
        );
        require!(max_task_cost > 0, ErrorCode::Overflow);
        require!(
            self.goal.total_shares > 0,
            ErrorCode::Overflow
        );

        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(0);
        let available_balance = self
            .vault
            .lamports()
            .checked_sub(self.goal.locked_for_tasks)
            .ok_or(ErrorCode::Underflow)?
            .checked_sub(rent_exempt_minimum)
            .ok_or(ErrorCode::Underflow)?;

        require!(
            available_balance >= max_task_cost,
            ErrorCode::InsufficientBalance
        );

        self.goal.locked_for_tasks = self
            .goal
            .locked_for_tasks
            .checked_add(max_task_cost)
            .ok_or(ErrorCode::Overflow)?;

        self.task.compute_node = Some(self.compute_node.key());
        self.task.max_task_cost = max_task_cost;
        self.task.status = TaskStatus::Processing;
        self.task.execution_count = self
            .task
            .execution_count
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }
}
