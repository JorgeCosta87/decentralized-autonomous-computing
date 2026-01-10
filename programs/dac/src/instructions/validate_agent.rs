use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{Agent, AgentStatus, NetworkConfig};

#[derive(Accounts)]
pub struct ValidateAgent<'info> {
    #[account(mut)]
    pub validator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"agent", network_config.key().as_ref(), agent.agent_slot_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

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

        // For now, just set agent to Active
        // TODO: Add TEE signature verification and agent config validation
        self.agent.status = AgentStatus::Active;

        Ok(())
    }
}
