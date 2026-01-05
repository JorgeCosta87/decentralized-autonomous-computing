use anchor_lang::prelude::*;

use crate::state::{Agent, AgentStatus, NetworkConfig};

#[derive(Accounts)]
pub struct CreateAgent<'info> {
    #[account(mut)]
    pub agent_owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config", network_config.authority.key().as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    #[account(
        init,
        payer = agent_owner,
        space = 8 + Agent::INIT_SPACE,
        seeds = [
            b"agent",
            network_config.key().as_ref(),
            network_config.next_agent_slot_id().to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub agent: Account<'info, Agent>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateAgent<'info> {
    pub fn create_agent(
        &mut self,
        agent_config_cid: String,
        bumps: &CreateAgentBumps,
    ) -> Result<()> {
        let agent_slot_id = self.network_config.next_agent_slot_id();

        self.agent.set_inner(Agent {
            agent_slot_id,
            owner: self.agent_owner.key(),
            agent_config_cid,
            agent_memory_cid: None,
            status: AgentStatus::Pending,
            bump: bumps.agent,
        });

        self.network_config.increment_agent_count()?;

        Ok(())
    }
}
