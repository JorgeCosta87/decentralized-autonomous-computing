use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{NetworkConfig, NodeInfo, NodeStatus, NodeType};

#[derive(Accounts)]
pub struct ClaimComputeNode<'info> {
    #[account(mut)]
    pub compute_node: Signer<'info>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    #[account(
        mut,
        seeds = [b"node_info", compute_node.key().as_ref()],
        bump = node_info.bump,
    )]
    pub node_info: Account<'info, NodeInfo>,
}

impl<'info> ClaimComputeNode<'info> {
    pub fn claim_compute_node(&mut self, node_info_cid: String) -> Result<()> {
        require!(
            self.node_info.node_type == NodeType::Compute,
            ErrorCode::InvalidNodeType
        );
        require!(
            self.node_info.status == NodeStatus::PendingClaim,
            ErrorCode::InvalidNodeStatus
        );

        self.node_info.node_info_cid = Some(node_info_cid);
        self.node_info.status = NodeStatus::AwaitingValidation;

        Ok(())
    }
}
