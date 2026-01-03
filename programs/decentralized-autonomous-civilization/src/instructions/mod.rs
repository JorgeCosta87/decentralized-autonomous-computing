use anchor_lang::prelude::*;

use crate::ActionType;
use crate::utils::init_dynamic_pda;
use crate::errors::ErrorCode;
use crate::state::{NetworkConfig, Goal, GoalStatus, TaskData, TaskStatus};

#[derive(Accounts)]
#[instruction(allocate_goals: u64, allocate_tasks: u64)]
pub struct InitializeNetwork<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + NetworkConfig::INIT_SPACE,
        seeds = [b"network_config"],
        bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeNetwork<'info> {
    pub fn initialize_network(
        &mut self,
        allocate_goals: u64,
        allocate_tasks: u64,
        approved_code_measurements: Vec<[u8; 32]>,
        remaining_accounts: &[AccountInfo<'info>],
        bumps: &InitializeNetworkBumps,
    ) -> Result<()> {
        require!(
            approved_code_measurements.len() > 0,
            ErrorCode::InvalidTeeProof
        );
        
        require!(
            approved_code_measurements.len() <= 10,
            ErrorCode::InvalidTeeProof
        );
        
        
        // Initialize network config
        self.network_config.agent_count = 0;
        self.network_config.goal_count = allocate_goals; // Set to allocated amount
        self.network_config.task_count = allocate_tasks; // Set to allocated amount
        self.network_config.validator_node_count = 0;
        self.network_config.compute_node_count = 0;
        self.network_config.approved_code_measurements = approved_code_measurements;
        self.network_config.bump = bumps.network_config;
        
        // Pre-allocate network-level goals
        let mut remaining_accounts_iter = remaining_accounts.iter();
        let network_config_key = self.network_config.key();
        
        for goal_id in 0..allocate_goals {
            let goal_account_info = remaining_accounts_iter.next()
                .ok_or(ErrorCode::MissingAccount)?;

            let seeds = &[
                b"goal",
                network_config_key.as_ref(),
                &goal_id.to_le_bytes(),
            ];

            let bump = init_dynamic_pda(
                &self.authority,
                goal_account_info,
                seeds,
                8 + Goal::INIT_SPACE,
                &crate::ID,
                &self.system_program,
            )?;

            let goal_data = Goal {
                goal_slot_id: goal_id,
                owner: self.authority.key(),
                agent: Pubkey::default(), // Will be assigned when agent uses this goal
                task_data: Pubkey::default(),
                description: "".to_string(),
                status: GoalStatus::Pending,
                max_iterations: 0,
                current_iteration: 0,
                task_index_at_goal_start: 0,
                task_index_at_goal_end: 0,
                bump: bump,
            };
            
            goal_data.try_serialize(&mut *goal_account_info.try_borrow_mut_data()?)?;
        }

        // Pre-allocate network-level tasks
        for task_id in 0..allocate_tasks {
            let task_account_info = remaining_accounts_iter.next()
                .ok_or(ErrorCode::MissingAccount)?;

            let seeds = &[
                b"task_data",
                network_config_key.as_ref(),
                &task_id.to_le_bytes(),
            ];

            let bump = init_dynamic_pda(
                &self.authority,
                task_account_info,
                seeds,
                8 + TaskData::INIT_SPACE,
                &crate::ID,
                &self.system_program,
            )?;

            let task_data = TaskData {
                task_slot_id: task_id,
                action_type: ActionType::Llm,
                status: TaskStatus::Ready,
                input_cid: None,
                output_cid: None,
                chain_proof: [0; 32],
                agent: Pubkey::default(),
                compute_node: None,
                execution_count: 0,
                bump,
            };

            task_data.try_serialize(&mut *task_account_info.try_borrow_mut_data()?)?;
        }
        
        Ok(())
    }
}

