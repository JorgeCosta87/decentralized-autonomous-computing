use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{NetworkConfig, NodeInfo, NodeStatus, NodeType};

#[derive(Accounts)]
pub struct ActivateNode<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ ErrorCode::InvalidAuthority,
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    #[account(
        mut,
        seeds = [b"node_info", node_info.node_pubkey.as_ref()],
        bump = node_info.bump,
    )]
    pub node_info: Account<'info, NodeInfo>,
}

impl<'info> ActivateNode<'info> {
    pub fn activate_node(&mut self) -> Result<()> {
        require!(
            self.node_info.status == NodeStatus::AwaitingValidation,
            ErrorCode::InvalidNodeStatus
        );

        match self.node_info.node_type {
            NodeType::Public => {
                require!(
                    self.node_info.node_info_cid.is_some(),
                    ErrorCode::InvalidNodeStatus
                );
            }
            NodeType::Confidential => {
                require!(
                    self.node_info.code_measurement.is_some()
                        && self.node_info.tee_signing_pubkey.is_some(),
                    ErrorCode::InvalidNodeStatus
                );
            }
        }

        self.node_info.status = NodeStatus::Active;

        match self.node_info.node_type {
            NodeType::Public => {
                self.network_config
                    .add_public_node(self.node_info.node_pubkey)?;
            }
            NodeType::Confidential => {
                self.network_config
                    .add_confidential_node(self.node_info.node_pubkey)?;
            }
        }

        Ok(())
    }
}
