use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{NetworkConfig, NodeInfo, NodeStatus, NodeType};

#[derive(Accounts)]
pub struct ClaimConfidentialNode<'info> {
    #[account(mut)]
    pub confidential_node: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config", network_config.authority.as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    #[account(
        mut,
        seeds = [b"node_info", confidential_node.key().as_ref()],
        bump = node_info.bump,
    )]
    pub node_info: Account<'info, NodeInfo>,
}

impl<'info> ClaimConfidentialNode<'info> {
    pub fn claim_confidential_node(
        &mut self,
        code_measurement: [u8; 32],
        tee_signing_pubkey: Pubkey,
    ) -> Result<()> {
        require!(
            self.node_info.node_type == NodeType::Confidential,
            ErrorCode::InvalidNodeType
        );
        require!(
            self.node_info.status == NodeStatus::PendingClaim,
            ErrorCode::InvalidNodeStatus
        );
        require!(
            self.network_config
                .is_measurement_approved(&code_measurement),
            ErrorCode::CodeMeasurementNotApproved
        );

        self.node_info.code_measurement = Some(code_measurement);
        self.node_info.tee_signing_pubkey = Some(tee_signing_pubkey);
        self.node_info.status = NodeStatus::Active;

        self.network_config.increment_validator_node_count()?;

        Ok(())
    }
}

// TODO: Full SGX attestation verification should be implemented:
// 1. Verify certificate chain (Intel Root CA → PCK → QE → Quote)
// 2. Extract MRENCLAVE from quote
// 3. Verify report_data[0..32] == node_pubkey
// 4. Extract tee_signing_pubkey from report_data[32..64]
// This requires additional libraries for SGX quote parsing and certificate chain verification.
