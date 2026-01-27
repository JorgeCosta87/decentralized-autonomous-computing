use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{CodeMeasurement, NetworkConfig, Task, TaskStatus};
use crate::utils::init_dynamic_pda;
use crate::TaskType;

#[derive(Accounts)]
#[instruction(allocate_tasks: u64)]
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
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
        required_validations: u32,
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

        let genesis_hash = self.network_config.compute_genesis_hash()?;

        self.network_config.set_inner(NetworkConfig {
            authority: self.authority.key(),
            cid_config: cid_config,
            genesis_hash: genesis_hash,
            task_count: allocate_tasks,
            required_validations: required_validations,
            allowed_models: vec![],
            approved_confidential_nodes: vec![],
            approved_public_nodes: vec![],
            agent_count: 0,
            session_count: 0,
            approved_code_measurements: approved_code_measurements,
            bump: bumps.network_config,
        });

        Self::pre_allocate_tasks(
            &remaining_accounts,
            &self.authority,
            self.network_config.key(),
            genesis_hash,
            allocate_tasks,
            &self.system_program,
        )?;

        Ok(())
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

            let seeds = &[b"task", network_config_key.as_ref(), &task_id.to_le_bytes()];

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
                session_slot_id: None,
                status: TaskStatus::Ready,
                compute_node: None,
                task_type: TaskType::Completion(0),
                chain_proof: genesis_hash,
                task_index: 0,
                max_task_cost: 0,
                max_call_count: 0,
                call_count: 0,
                input_cid: None,
                output_cid: None,
                pending_input_cid: None,
                pending_output_cid: None,
                validations: Vec::new(),
                bump,
            };

            task_data.try_serialize(&mut *task_account_info.try_borrow_mut_data()?)?;
        }

        Ok(())
    }
}
