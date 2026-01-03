use anchor_lang::prelude::*;


use crate::ActionType;
use crate::utils::init_dynamic_pda;
use crate::errors::ErrorCode;
use crate::state::{NetworkConfig, Goal, GoalStatus, Task, TaskStatus, CodeMeasurement};

#[derive(Accounts)]
#[instruction(allocate_goals: u64, allocate_tasks: u64)]
pub struct InitializeNetwork<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + NetworkConfig::INIT_SPACE,
        seeds = [b"dac_network_config", authority.key().as_ref()],
        bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeNetwork<'info> {
    pub fn initialize_network(
        &mut self,
        cid_config: String,
        allocate_goals: u64,
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
        remaining_accounts: &[AccountInfo<'info>],
        bumps: &InitializeNetworkBumps,
    ) -> Result<()> {
        require!(
            !approved_code_measurements.is_empty(),
            ErrorCode::NeedAtLeastOneCodeMeasurement
        );
        
        require!(
            approved_code_measurements.len() <= 10,
            ErrorCode::TooManyCodeMeasurements
        );

        let genesis_hash = self.network_config.compute_genesis_hash(&self.authority.key())?;
        
        self.network_config.set_inner(NetworkConfig {
            authority: self.authority.key(),
            cid_config: cid_config,
            genesis_hash: genesis_hash,
            agent_count: 0,
            goal_count: allocate_goals,
            task_count: allocate_tasks,
            validator_node_count: 0,
            compute_node_count: 0,
            approved_code_measurements: approved_code_measurements,
            bump: bumps.network_config,
        });
        
        let mut account_offset = 0;
        
        account_offset += Self::pre_allocate_goals(
            &remaining_accounts[account_offset..],
            &self.authority,
            self.network_config.key(),
            genesis_hash,
            allocate_goals,
            &self.system_program,
        )?;
        
        Self::pre_allocate_tasks(
            &remaining_accounts[account_offset..],
            &self.authority,
            self.network_config.key(),
            genesis_hash,
            allocate_tasks,
            &self.system_program,
        )?;
        
        Ok(())
    }
    
    fn pre_allocate_goals(
        remaining_accounts: &[AccountInfo<'info>],
        authority: &Signer<'info>,
        network_config_key: Pubkey,
        genesis_hash: [u8; 32],
        allocate_goals: u64,
        system_program: &Program<'info, System>,
    ) -> Result<usize> {
        let mut account_index = 0;
        
        for goal_id in 0..allocate_goals {
            let goal_account_info = remaining_accounts
                .get(account_index)
                .ok_or(ErrorCode::MissingAccount)?;
            account_index += 1;

            let seeds = &[
                b"goal",
                network_config_key.as_ref(),
                &goal_id.to_le_bytes(),
            ];

            let bump = init_dynamic_pda(
                authority,
                goal_account_info,
                seeds,
                8 + Goal::INIT_SPACE,
                &crate::ID,
                system_program,
            )?;

            let goal_data = Goal {
                goal_slot_id: goal_id,
                owner: authority.key(),
                agent: Pubkey::default(),
                task: Pubkey::default(),
                specification_cid: "".to_string(),
                status: GoalStatus::Ready,
                max_iterations: 0,
                current_iteration: 0,
                task_index_at_goal_start: 0,
                task_index_at_goal_end: 0,
                chain_proof: genesis_hash,
                bump,
            };
            
            goal_data.try_serialize(&mut *goal_account_info.try_borrow_mut_data()?)?;
        }
        
        Ok(account_index)
    }
    
    fn pre_allocate_tasks(
        remaining_accounts: &[AccountInfo<'info>],
        authority: &Signer<'info>,
        network_config_key: Pubkey,
        genesis_hash: [u8; 32],
        allocate_tasks: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        for task_id in 0..allocate_tasks {
            let task_account_info = remaining_accounts
                .get(task_id as usize)
                .ok_or(ErrorCode::MissingAccount)?;

            let seeds = &[
                b"task",
                network_config_key.as_ref(),
                &task_id.to_le_bytes(),
            ];

            let bump = init_dynamic_pda(
                authority,
                task_account_info,
                seeds,
                8 + Task::INIT_SPACE,
                &crate::ID,
                system_program,
            )?;

            let task_data = Task {
                task_slot_id: task_id,
                action_type: ActionType::Llm,
                status: TaskStatus::Ready,
                input_cid: None,
                output_cid: None,
                pending_input_cid: None,
                pending_output_cid: None,
                chain_proof: genesis_hash,
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

