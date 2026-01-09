use anchor_lang::prelude::borsh::{BorshDeserialize, BorshSerialize};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{sysvar::instructions as ix_sysvar, sysvar::SysvarId};
use bytemuck::from_bytes;
use solana_ed25519_program::{Ed25519SignatureOffsets, PUBKEY_SERIALIZED_SIZE};
use solana_sdk_ids::ed25519_program;

use crate::errors::ErrorCode;
use crate::state::{NetworkConfig, NodeInfo, NodeStatus, NodeType};

#[derive(InitSpace, BorshSerialize, BorshDeserialize)]
pub struct ValidateComputeNodeMessage {
    pub compute_node_pubkey: Pubkey,
    pub approved: bool,
}

#[derive(Accounts)]
pub struct ValidateComputeNode<'info> {
    #[account(mut)]
    pub validator_node_pubkey: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config"],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    #[account(
        seeds = [b"node_info", validator_node_pubkey.key().as_ref()],
        bump = validator_node_info.bump,
    )]
    pub validator_node_info: Account<'info, NodeInfo>,
    #[account(
        mut,
        seeds = [b"node_info", compute_node_info.node_pubkey.key().as_ref()],
        bump = compute_node_info.bump,
    )]
    pub compute_node_info: Account<'info, NodeInfo>,

    /// CHECK: Check if the instruction is from the Ed25519 program
    #[account(address = ix_sysvar::Instructions::id())]
    pub instruction_sysvar: AccountInfo<'info>,
}

// https://rareskills.io/post/solana-signature-verification
// https://github.com/solana-foundation/anchor/blob/master/lang/src/signature_verification/ed25519.rs
impl<'info> ValidateComputeNode<'info> {
    pub fn validate_compute_node(&mut self) -> Result<()> {
        // Verify validator node is Active and is a Validator
        require!(
            self.validator_node_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.validator_node_info.node_type == NodeType::Validator,
            ErrorCode::InvalidNodeType
        );
        require!(
            self.compute_node_info.status == NodeStatus::AwaitingValidation,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.compute_node_info.node_type == NodeType::Compute,
            ErrorCode::InvalidNodeType
        );

        // Get TEE signing pubkey from validator node info (stored during claim_validator_node)
        let validator_tee_signing_pubkey = self
            .validator_node_info
            .tee_signing_pubkey
            .ok_or(ErrorCode::InvalidTeeSignature)?;

        msg!(
            "validator_tee_signing_pubkey: {:?}",
            validator_tee_signing_pubkey
        );

        let ix_sysvar_account = self.instruction_sysvar.to_account_info();
        let current_ix_index = ix_sysvar::load_current_index_checked(&ix_sysvar_account)
            .map_err(|_| error!(ErrorCode::InvalidInstructionSysvar))?;

        msg!("current_ix_index: {}", current_ix_index);

        require!(current_ix_index > 0, ErrorCode::InvalidInstructionSysvar);

        let ed_ix = ix_sysvar::load_instruction_at_checked(
            (current_ix_index - 1) as usize,
            &ix_sysvar_account,
        )
        .map_err(|_| error!(ErrorCode::InvalidInstructionSysvar))?;
        msg!("ed_ix: {:?}", ed_ix);
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

        msg!(
            "pubkey_offset: {}, msg_offset: {}, msg_len: {}",
            pubkey_offset,
            msg_offset,
            msg_len
        );

        let validator_pubkey_slice =
            &ed_data[pubkey_offset..(pubkey_offset + PUBKEY_SERIALIZED_SIZE)];
        let msg_bytes = &mut &ed_data[msg_offset..(msg_offset + msg_len)];

        require!(
            validator_pubkey_slice == validator_tee_signing_pubkey.as_ref(),
            ErrorCode::InvalidValidatorTeeSigningPubkey
        );

        let validated_message = ValidateComputeNodeMessage::deserialize(msg_bytes)?;

        require!(
            validated_message.compute_node_pubkey == self.compute_node_info.node_pubkey,
            ErrorCode::InvalidComputeNodePubkey
        );

        if validated_message.approved {
            self.compute_node_info.status = NodeStatus::Active;
            self.network_config.increment_compute_node_count()?;
        } else {
            self.compute_node_info.status = NodeStatus::Rejected;
        }

        Ok(())
    }
}
