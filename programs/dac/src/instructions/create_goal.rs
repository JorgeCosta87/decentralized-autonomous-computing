use anchor_lang::prelude::*;

use crate::state::{Goal, GoalStatus, NetworkConfig};

#[derive(Accounts)]
pub struct CreateGoal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config"],
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

    pub system_program: Program<'info, System>,
}

impl<'info> CreateGoal<'info> {
    pub fn create_goal(&mut self, is_public: bool, bumps: &CreateGoalBumps) -> Result<()> {
        let goal_slot_id = self.network_config.next_goal_slot_id();
        let genesis_hash = self.network_config.genesis_hash;

        let owner = if is_public {
            Pubkey::default()
        } else {
            self.owner.key()
        };

        self.goal.set_inner(Goal {
            goal_slot_id,
            owner,
            agent: Pubkey::default(),
            task: Pubkey::default(),
            status: GoalStatus::Ready,
            specification_cid: "".to_string(),
            max_iterations: 0,
            current_iteration: 0,
            task_index_at_goal_start: 0,
            task_index_at_goal_end: 0,
            chain_proof: genesis_hash,
            total_shares: 0,
            locked_for_tasks: 0,
            vault_bump: 0,
            bump: bumps.goal,
        });

        self.network_config.increment_goal_count()?;

        Ok(())
    }
}
