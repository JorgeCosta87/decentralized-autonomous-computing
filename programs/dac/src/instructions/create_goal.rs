use anchor_lang::prelude::*;

use crate::state::{Goal, GoalStatus, NetworkConfig, Task, TaskStatus};
use crate::ActionType;

#[derive(Accounts)]
pub struct CreateGoal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + Goal::INIT_SPACE,
        seeds = [
            b"goal",
            network_config.key().as_ref(),
            network_config.next_goal_slot_id().to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub goal: Account<'info, Goal>,

    #[account(
        init,
        payer = payer,
        space = 8 + Task::INIT_SPACE,
        seeds = [
            b"task",
            network_config.key().as_ref(),
            network_config.next_task_slot_id().to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub task: Account<'info, Task>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateGoal<'info> {
    pub fn create_goal(
        &mut self,
        is_owned: bool,
        is_confidential: bool,
        bumps: &CreateGoalBumps,
    ) -> Result<()> {
        let goal_slot_id = self.network_config.next_goal_slot_id();
        let task_slot_id = self.network_config.next_task_slot_id();
        let genesis_hash = self.network_config.genesis_hash;

        let owner = if is_owned {
            self.owner.key()
        } else {
            Pubkey::default()
        };

        // Initialize task
        self.task.set_inner(Task {
            task_slot_id,
            status: TaskStatus::Ready,
            compute_node: None,
            action_type: ActionType::Llm,
            chain_proof: genesis_hash,
            execution_count: 0,
            max_task_cost: 0,
            max_call_count: 0,
            call_count: 0,
            input_cid: None,
            output_cid: None,
            pending_input_cid: None,
            pending_output_cid: None,
            approved_validators: Vec::new(),
            rejected_validators: Vec::new(),
            bump: bumps.task,
        });

        // Initialize goal and link to task
        self.goal.set_inner(Goal {
            goal_slot_id,
            owner,
            agent: Pubkey::default(),
            task: self.task.key(),
            status: GoalStatus::Ready,
            is_confidential,
            max_iterations: 0,
            current_iteration: 0,
            task_index_at_goal_start: 0,
            task_index_at_goal_end: 0,
            total_shares: 0,
            locked_for_tasks: 0,
            chain_proof: genesis_hash,
            specification_cid: "".to_string(),
            state_cid: None,
            vault_bump: 0,
            bump: bumps.goal,
        });

        self.network_config.increment_goal_count()?;
        self.network_config.increment_task_count()?;

        Ok(())
    }
}
