use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{Agent, AgentStatus, NetworkConfig, NodeInfo, NodeStatus};
use crate::utils::check_validation_threshold;

#[derive(Accounts)]
pub struct ValidateAgent<'info> {
    #[account(mut)]
    pub node: Signer<'info>,

    #[account(
        mut,
        seeds = [b"agent", network_config.key().as_ref(), agent.agent_slot_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        mut,
        seeds = [b"node_info", node.key().as_ref()],
        bump = node_info.bump,
    )]
    pub node_info: Account<'info, NodeInfo>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,
}

impl<'info> ValidateAgent<'info> {
    pub fn validate_agent(&mut self) -> Result<()> {
        require!(
            self.agent.status == AgentStatus::Pending,
            ErrorCode::InvalidAgentStatus
        );
        require!(
            self.node_info.status == NodeStatus::Active,
            ErrorCode::InvalidNodeStatus
        );

        require!(
            !self.agent.approved_validators.contains(&self.node.key())
                && !self.agent.rejected_validators.contains(&self.node.key()),
            ErrorCode::DuplicateValidation
        );

        self.agent.approved_validators.push(self.node.key());
        let approved_count = self.agent.approved_validators.len() as u32;

        if check_validation_threshold(approved_count, self.network_config.required_validations)? {
            self.agent.status = AgentStatus::Active;
        }

        Ok(())
    }
}
