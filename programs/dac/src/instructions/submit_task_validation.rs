use anchor_lang::prelude::borsh::{BorshDeserialize, BorshSerialize};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{sysvar::instructions as ix_sysvar, sysvar::SysvarId};
use anchor_lang::system_program;
use bytemuck::from_bytes;
use sha2::{Digest, Sha256};
use solana_ed25519_program::{Ed25519SignatureOffsets, PUBKEY_SERIALIZED_SIZE};
use solana_sdk_ids::ed25519_program;

use crate::errors::ErrorCode;
use crate::state::{
    Goal, GoalStatus, NetworkConfig, NodeInfo, NodeStatus, NodeType, Task, TaskStatus,
};

#[derive(InitSpace, BorshSerialize, BorshDeserialize)]
pub struct SubmitTaskValidationMessage {
    pub goal_id: u64,
    pub task_slot_id: u64,
    pub payment_amount: u64,
    pub validation_proof: [u8; 32],
    pub approved: bool,
    pub goal_completed: bool,
}

#[derive(Accounts)]
pub struct SubmitTaskValidation<'info> {
    #[account(mut)]
    pub validator: Signer<'info>,

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
        mut,
        seeds = [b"task", network_config.key().as_ref(), task.task_slot_id.to_le_bytes().as_ref()],
        bump = task.bump,
    )]
    pub task: Account<'info, Task>,

    #[account(
        mut,
        seeds = [b"node_info", compute_node_info.node_pubkey.key().as_ref()],
        bump = compute_node_info.bump,
    )]
    pub compute_node_info: Account<'info, NodeInfo>,

    #[account(
        mut,
        seeds = [b"node_treasury", compute_node_info.key().as_ref()],
        bump,
    )]
    pub node_treasury: SystemAccount<'info>,

    #[account(
        seeds = [b"node_info", validator.key().as_ref()],
        bump = validator_node_info.bump,
    )]
    pub validator_node_info: Account<'info, NodeInfo>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    /// CHECK: Check if the instruction is from the Ed25519 program
    #[account(address = ix_sysvar::Instructions::id())]
    pub instruction_sysvar: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> SubmitTaskValidation<'info> {
    pub fn submit_task_validation(&mut self) -> Result<()> {
        require!(
            self.validator_node_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.validator_node_info.node_type == NodeType::Validator,
            ErrorCode::InvalidNodeType
        );
        require!(
            self.compute_node_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.compute_node_info.node_type == NodeType::Compute,
            ErrorCode::InvalidNodeType
        );
        require!(
            self.goal.status == GoalStatus::Active,
            ErrorCode::InvalidGoalStatus
        );
        require!(
            self.task.status == TaskStatus::AwaitingValidation,
            ErrorCode::InvalidTaskStatus
        );

        // Verify TEE signature
        let validator_tee_signing_pubkey = self
            .validator_node_info
            .tee_signing_pubkey
            .ok_or(ErrorCode::InvalidTeeSignature)?;

        let ix_sysvar_account = self.instruction_sysvar.to_account_info();
        let current_ix_index = ix_sysvar::load_current_index_checked(&ix_sysvar_account)
            .map_err(|_| error!(ErrorCode::InvalidInstructionSysvar))?;

        require!(current_ix_index > 0, ErrorCode::InvalidInstructionSysvar);

        let ed_ix = ix_sysvar::load_instruction_at_checked(
            (current_ix_index - 1) as usize,
            &ix_sysvar_account,
        )
        .map_err(|_| error!(ErrorCode::InvalidInstructionSysvar))?;

        require!(
            ed_ix.program_id.as_ref() == ed25519_program::ID.as_ref(),
            ErrorCode::BadEd25519Program
        );
        require!(ed_ix.accounts.is_empty(), ErrorCode::BadEd25519Accounts);

        let ed_data = &ed_ix.data;
        require!(ed_data.len() >= 16, ErrorCode::InvalidInstructionSysvar);

        let offsets: &Ed25519SignatureOffsets = from_bytes(&ed_data[2..16]);

        let pubkey_offset = offsets.public_key_offset as usize;
        let msg_offset = offsets.message_data_offset as usize;
        let msg_len = offsets.message_data_size as usize;

        let validator_pubkey_slice =
            &ed_data[pubkey_offset..(pubkey_offset + PUBKEY_SERIALIZED_SIZE)];
        let msg_bytes = &mut &ed_data[msg_offset..(msg_offset + msg_len)];

        require!(
            validator_pubkey_slice == validator_tee_signing_pubkey.as_ref(),
            ErrorCode::InvalidValidatorTeeSigningPubkey
        );

        let message = SubmitTaskValidationMessage::deserialize(msg_bytes)?;

        // Verify message matches
        require!(
            message.goal_id == self.goal.goal_slot_id,
            ErrorCode::InvalidValidatorMessage
        );
        require!(
            message.task_slot_id == self.task.task_slot_id,
            ErrorCode::InvalidValidatorMessage
        );
        require!(message.payment_amount > 0, ErrorCode::Overflow);

        // Verify validation_proof matches expected proof
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

        if message.approved {
            // Update task chain_proof: SHA256(old_chain_proof + input_cid + output_cid + execution_count)
            // Uses previous validated input_cid/output_cid
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
            hasher.update(&self.task.execution_count.to_le_bytes());
            self.task.chain_proof = hasher.finalize().into();

            self.task.input_cid = self.task.pending_input_cid.take();
            self.task.output_cid = self.task.pending_output_cid.take();

            // Update goal chain_proof: SHA256(old_goal_proof + task_chain_proof + task_id + iteration)
            let mut hasher = Sha256::new();
            hasher.update(&self.goal.chain_proof);
            hasher.update(&self.task.chain_proof);
            hasher.update(&self.task.task_slot_id.to_le_bytes());
            hasher.update(&self.goal.current_iteration.to_le_bytes());
            self.goal.chain_proof = hasher.finalize().into();

            // Release amout locked for tasks
            self.goal.locked_for_tasks = self
                .goal
                .locked_for_tasks
                .checked_sub(self.task.max_task_cost)
                .ok_or(ErrorCode::Underflow)?;

            // Pay compute node from goal vault to node treasury
            require!(
                self.vault.lamports() >= message.payment_amount,
                ErrorCode::InsufficientBalance
            );

            let goal_key = self.goal.key();
            let vault_seeds = &[b"goal_vault", goal_key.as_ref(), &[self.goal.vault_bump]];
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

            // Update compute node stats
            self.compute_node_info.total_earned = self
                .compute_node_info
                .total_earned
                .checked_add(message.payment_amount)
                .ok_or(ErrorCode::Overflow)?;
            self.compute_node_info.total_tasks_completed = self
                .compute_node_info
                .total_tasks_completed
                .checked_add(1)
                .ok_or(ErrorCode::Overflow)?;

            // Update goal progress
            self.goal.current_iteration = self
                .goal
                .current_iteration
                .checked_add(1)
                .ok_or(ErrorCode::Overflow)?;

            if message.goal_completed {
                self.goal.status = GoalStatus::Ready;
            } else {
                self.task.status = TaskStatus::Pending;
            }
        } else {
            // Validation rejected
            // Release lock

            self.goal.locked_for_tasks = self
                .goal
                .locked_for_tasks
                .checked_sub(self.task.max_task_cost)
                .ok_or(ErrorCode::Underflow)?;

            // Clear pending input and output cids and set task back to Ready
            self.task.pending_input_cid = None;
            self.task.pending_output_cid = None;
            self.task.status = TaskStatus::Ready;
        }

        Ok(())
    }
}
