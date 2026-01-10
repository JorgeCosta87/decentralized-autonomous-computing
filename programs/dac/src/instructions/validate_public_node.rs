use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{NetworkConfig, NodeInfo, NodeStatus, NodeType};
use crate::utils::check_validation_threshold;

#[derive(Accounts)]
pub struct ValidatePublicNode<'info> {
    #[account(mut)]
    pub node_validating: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    #[account(
        mut,
        seeds = [b"node_info", node_validating.key().as_ref()],
        bump = node_validating_info.bump,
    )]
    pub node_validating_info: Account<'info, NodeInfo>,

    #[account(
        mut,
        seeds = [b"node_info", node_info.node_pubkey.key().as_ref()],
        bump = node_info.bump,
    )]
    pub node_info: Account<'info, NodeInfo>,
}

impl<'info> ValidatePublicNode<'info> {
    pub fn validate_public_node(&mut self, approved: bool) -> Result<()> {
        require!(
            self.node_validating_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.node_info.status == NodeStatus::AwaitingValidation,
            ErrorCode::InvalidNodeStatus
        );

        require!(
            self.node_validating_info.node_type == NodeType::Public
                || self.node_validating_info.node_type == NodeType::Confidential,
            ErrorCode::InvalidNodeType
        );

        require!(
            self.node_info.node_type == NodeType::Public,
            ErrorCode::InvalidNodeType
        );

        require!(
            !self
                .node_info
                .approved_validators
                .contains(&self.node_validating.key())
                && !self
                    .node_info
                    .rejected_validators
                    .contains(&self.node_validating.key()),
            ErrorCode::DuplicateValidation
        );

        if approved {
            self.node_info
                .approved_validators
                .push(self.node_validating.key());
            let approved_count = self.node_info.approved_validators.len() as u32;
            let threshold_reached = check_validation_threshold(
                approved_count,
                self.network_config.required_validations,
            )?;
            if threshold_reached {
                self.node_info.status = NodeStatus::Active;
                self.network_config.increment_public_node_count()?;
            }
        } else {
            self.node_info
                .rejected_validators
                .push(self.node_validating.key());
            self.node_info.status = NodeStatus::Rejected;
        }

        Ok(())
    }
}
