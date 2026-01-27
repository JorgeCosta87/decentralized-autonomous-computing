use anchor_lang::prelude::borsh::{BorshDeserialize, BorshSerialize};
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use sha2::{Digest, Sha256};

use crate::errors::ErrorCode;
use crate::events::{SessionCompleted, TaskValidationSubmitted};
use crate::state::{
    NetworkConfig, NodeInfo, NodeStatus, NodeType, Session, SessionStatus, Task, TaskStatus,
    ValidationStatus,
};
use crate::utils::{check_validation_threshold, verify_tee_signature};

#[derive(InitSpace, BorshSerialize, BorshDeserialize)]
pub struct SubmitTaskValidationMessage {
    pub goal_id: u64,
    pub task_slot_id: u64,
    pub payment_amount: u64,
    pub validation_proof: [u8; 32],
    pub approved: bool,
    pub session_completed: bool,
}

#[derive(Accounts)]
pub struct SubmitTaskValidation<'info> {
    #[account(mut)]
    pub node_validating: Signer<'info>,

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
        mut,
        seeds = [b"task", network_config.key().as_ref(), task.task_slot_id.to_le_bytes().as_ref()],
        bump = task.bump,
    )]
    pub task: Account<'info, Task>,

    #[account(
        mut,
        seeds = [b"node_info", node_info.node_pubkey.key().as_ref()],
        bump = node_info.bump,
    )]
    pub node_info: Account<'info, NodeInfo>,

    #[account(
        mut,
        seeds = [b"node_treasury", node_info.key().as_ref()],
        bump,
    )]
    pub node_treasury: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"node_info", node_validating.key().as_ref()],
        bump = validator_node_info.bump,
    )]
    pub validator_node_info: Account<'info, NodeInfo>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    /// CHECK: Check if the instruction is from the Ed25519 program (only for confidential)
    #[account(address = anchor_lang::solana_program::sysvar::instructions::id())]
    pub instruction_sysvar: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> SubmitTaskValidation<'info> {
    pub fn submit_confidential_task_validation(&mut self) -> Result<()> {
        self.validate_common_requirements()?;

        require!(self.session.is_confidential, ErrorCode::InvalidSessionStatus);

        let message = self.verify_confidential_validation()?;

        if message.approved {
            self.process_approved_validation(&message)?;
        } else {
            self.process_rejected_validation()?;
        }

        Ok(())
    }

    pub fn submit_public_task_validation(
        &mut self,
        payment_amount: u64,
        approved: bool,
        goal_completed: bool,
    ) -> Result<()> {
        self.validate_common_requirements()?;

        require!(!self.session.is_confidential, ErrorCode::InvalidSessionStatus);

        require!(
            self.validator_node_info.node_type == NodeType::Public
                || self.validator_node_info.node_type == NodeType::Confidential,
            ErrorCode::InvalidNodeType
        );
        let validator_pubkey = self.node_validating.key();
        let validator_entry = self
            .task
            .validations
            .iter()
            .find(|v| v.pubkey == validator_pubkey)
            .ok_or(ErrorCode::ValidatorNotAssigned)?;
        require!(
            validator_entry.status == ValidationStatus::Pending,
            ErrorCode::DuplicateValidation
        );

        if approved {
            let message = SubmitTaskValidationMessage {
                goal_id: self.session.session_slot_id,
                task_slot_id: self.task.task_slot_id,
                payment_amount,
                validation_proof: [0; 32],
                approved,
                session_completed: goal_completed,
            };
            self.process_approved_validation(&message)?;
        } else {
            self.process_rejected_validation()?;
        }

        Ok(())
    }

    fn validate_common_requirements(&self) -> Result<()> {
        require!(
            self.validator_node_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.node_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.session.status == SessionStatus::Active,
            ErrorCode::InvalidSessionStatus
        );
        require!(
            self.task.status == TaskStatus::AwaitingValidation,
            ErrorCode::InvalidTaskStatus
        );
        require!(
            self.task.compute_node == Some(self.node_info.node_pubkey),
            ErrorCode::InvalidComputeNodePubkey
        );

        Ok(())
    }

    fn verify_confidential_validation(&mut self) -> Result<SubmitTaskValidationMessage> {
        require!(
            self.validator_node_info.node_type == NodeType::Confidential,
            ErrorCode::InvalidNodeType
        );

        // Get TEE signing pubkey
        let validator_tee_signing_pubkey = self
            .validator_node_info
            .tee_signing_pubkey
            .ok_or(ErrorCode::InvalidTeeSignature)?;

        // Verify TEE signature and extract message
        let message: SubmitTaskValidationMessage =
            verify_tee_signature(&self.instruction_sysvar, &validator_tee_signing_pubkey)?;

        require!(
            message.goal_id == self.session.session_slot_id,
            ErrorCode::InvalidValidatorMessage
        );
        require!(
            message.task_slot_id == self.task.task_slot_id,
            ErrorCode::InvalidValidatorMessage
        );
        require!(message.payment_amount > 0, ErrorCode::Overflow);

        // Verify validation_proof matches expected proof
        self.verify_validation_proof(&message)?;

        let validator_pubkey = self.node_validating.key();
        let validator_entry = self
            .task
            .validations
            .iter()
            .find(|v| v.pubkey == validator_pubkey)
            .ok_or(ErrorCode::ValidatorNotAssigned)?;
        require!(
            validator_entry.status == ValidationStatus::Pending,
            ErrorCode::DuplicateValidation
        );

        Ok(message)
    }

    /// Verify validation proof matches expected hash
    fn verify_validation_proof(&self, message: &SubmitTaskValidationMessage) -> Result<()> {
        let pending_input_cid = self
            .task
            .pending_input_cid
            .as_ref()
            .ok_or(ErrorCode::InvalidPDAAccount)?;
        let pending_output_cid = self
            .task
            .pending_output_cid
            .as_ref()
            .ok_or(ErrorCode::InvalidPDAAccount)?;

        let mut hasher = Sha256::new();
        hasher.update(pending_input_cid.as_bytes());
        hasher.update(pending_output_cid.as_bytes());
        let expected_proof: [u8; 32] = hasher.finalize().into();

        require!(
            message.validation_proof == expected_proof,
            ErrorCode::InvalidTeeSignature
        );

        Ok(())
    }

    fn process_approved_validation(&mut self, message: &SubmitTaskValidationMessage) -> Result<()> {
        let validator_pubkey = self.node_validating.key();
        if let Some(v) = self
            .task
            .validations
            .iter_mut()
            .find(|v| v.pubkey == validator_pubkey)
        {
            v.status = ValidationStatus::Approved;
        }
        let approved_count = self
            .task
            .validations
            .iter()
            .filter(|v| v.status == ValidationStatus::Approved)
            .count() as u32;
        let threshold_reached =
            check_validation_threshold(approved_count, self.network_config.required_validations)?;

        if !threshold_reached {
            return Ok(());
        }

        // Update task chain_proof
        let old_input_cid = self
            .task
            .input_cid
            .as_ref()
            .map(|s| s.as_bytes())
            .unwrap_or(&[]);
        let old_output_cid = self
            .task
            .output_cid
            .as_ref()
            .map(|s| s.as_bytes())
            .unwrap_or(&[]);

        let mut hasher = Sha256::new();
        hasher.update(&self.task.chain_proof);
        hasher.update(old_input_cid);
        hasher.update(old_output_cid);
        hasher.update(&self.task.task_index.to_le_bytes());
        self.task.chain_proof = hasher.finalize().into();

        // Move pending to validated (these become the historical record)
        self.task.input_cid = self.task.pending_input_cid.take();
        self.task.output_cid = self.task.pending_output_cid.take();

        // Release locked funds
        self.session.locked_for_tasks = self
            .session
            .locked_for_tasks
            .checked_sub(self.task.max_task_cost)
            .ok_or(ErrorCode::Underflow)?;

        // Pay compute node
        require!(
            self.vault.lamports() >= message.payment_amount,
            ErrorCode::InsufficientBalance
        );

        let session_key = self.session.key();
        let vault_seeds = &[b"session_vault", session_key.as_ref(), &[self.session.vault_bump]];
        let vault_signer = &[&vault_seeds[..]];

        let cpi_accounts = system_program::Transfer {
            from: self.vault.to_account_info(),
            to: self.node_treasury.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            vault_signer,
        );

        system_program::transfer(cpi_context, message.payment_amount)?;

        self.node_info.total_earned = self
            .node_info
            .total_earned
            .checked_add(message.payment_amount)
            .ok_or(ErrorCode::Overflow)?;
        self.node_info.total_tasks_completed = self
            .node_info
            .total_tasks_completed
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;

        self.session.current_iteration = self
            .session
            .current_iteration
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;

        if message.session_completed
            || (self.session.max_iterations != 0
                && self.session.current_iteration >= self.session.max_iterations)
        {
            self.session.status = SessionStatus::Completed;
            self.task.status = TaskStatus::Ready;

            emit!(SessionCompleted {
                session_slot_id: self.session.session_slot_id,
                final_iteration: self.session.current_iteration,
                vault_balance: self.vault.lamports(),
            });
        } else {
            self.task.status = TaskStatus::Pending;
        }

        self.task.validations.clear();

        // Emit task validation submitted event
        emit!(TaskValidationSubmitted {
            session_slot_id: self.session.session_slot_id,
            task_slot_id: self.task.task_slot_id,
            validator: self.node_validating.key(),
            payment_amount: message.payment_amount,
            approved: message.approved,
            session_completed: message.session_completed,
            current_iteration: self.session.current_iteration,
            vault_balance: self.vault.lamports(),
            locked_for_tasks: self.session.locked_for_tasks,
        });

        Ok(())
    }

    fn process_rejected_validation(&mut self) -> Result<()> {
        let validator_pubkey = self.node_validating.key();
        if let Some(v) = self
            .task
            .validations
            .iter_mut()
            .find(|v| v.pubkey == validator_pubkey)
        {
            v.status = ValidationStatus::Rejected;
        }
        let rejected_count = self
            .task
            .validations
            .iter()
            .filter(|v| v.status == ValidationStatus::Rejected)
            .count() as u32;
        let threshold_reached =
            check_validation_threshold(rejected_count, self.network_config.required_validations)?;

        if !threshold_reached {
            return Ok(());
        }

        // Release task lock
        self.session.locked_for_tasks = self
            .session
            .locked_for_tasks
            .checked_sub(self.task.max_task_cost)
            .ok_or(ErrorCode::Underflow)?;

        // Clear pending fields (task will be reset for next claim)
        self.task.pending_input_cid = None;
        self.task.pending_output_cid = None;
        self.task.status = TaskStatus::Ready;

        self.task.validations.clear();

        Ok(())
    }
}
