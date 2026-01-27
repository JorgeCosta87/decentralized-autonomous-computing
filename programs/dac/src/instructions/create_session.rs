use anchor_lang::prelude::*;

use crate::state::{NetworkConfig, Session, SessionStatus, Task, TaskStatus};
use crate::TaskType;

#[derive(Accounts)]
pub struct CreateSession<'info> {
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
        space = 8 + Session::INIT_SPACE,
        seeds = [
            b"session",
            network_config.key().as_ref(),
            network_config.next_session_slot_id().to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub session: Account<'info, Session>,

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

impl<'info> CreateSession<'info> {
    pub fn create_session(
        &mut self,
        is_owned: bool,
        is_confidential: bool,
        bumps: &CreateSessionBumps,
    ) -> Result<()> {
        let session_slot_id = self.network_config.next_session_slot_id();
        let task_slot_id = self.network_config.next_task_slot_id();

        let owner = if is_owned {
            self.owner.key()
        } else {
            Pubkey::default()
        };

        self.session.set_inner(Session {
            session_slot_id,
            owner,
            task: self.task.key(),
            status: SessionStatus::Pending,
            is_confidential,
            max_iterations: 0,
            current_iteration: 0,
            task_index_start: 0,
            task_index_end: 0,
            total_shares: 0,
            locked_for_tasks: 0,
            specification_cid: "".to_string(),
            state_cid: None,
            vault_bump: 0,
            bump: bumps.session,
        });

        self.task.set_inner(Task {
            task_slot_id,
            session_slot_id: Some(session_slot_id),
            status: TaskStatus::Ready,
            compute_node: None,
            task_type: TaskType::Completion(0),
            chain_proof: [0u8; 32],
            task_index: 0,
            max_task_cost: 0,
            max_call_count: 0,
            call_count: 0,
            input_cid: None,
            output_cid: None,
            pending_input_cid: None,
            pending_output_cid: None,
            validations: Vec::new(),
            bump: bumps.task,
        });

        self.network_config.increment_session_count()?;
        self.network_config.increment_task_count()?;

        Ok(())
    }
}
