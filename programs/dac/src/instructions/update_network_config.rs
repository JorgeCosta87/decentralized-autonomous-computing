use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{CodeMeasurement, NetworkConfig};

#[derive(Accounts)]
pub struct UpdateNetworkConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config", authority.key().as_ref()],
        bump = network_config.bump,
        constraint = network_config.authority == authority.key() @ ErrorCode::InvalidAuthority
    )]
    pub network_config: Account<'info, NetworkConfig>,
}

impl<'info> UpdateNetworkConfig<'info> {
    pub fn update_network_config(
        &mut self,
        cid_config: Option<String>,
        new_code_measurement: Option<CodeMeasurement>,
    ) -> Result<()> {
        if let Some(new_cid_config) = cid_config {
            require!(new_cid_config.len() <= 128, ErrorCode::InvalidCID);
            self.network_config.cid_config = new_cid_config;
        }

        if let Some(measurement) = new_code_measurement {
            if !self.network_config.approved_code_measurements
                .iter()
                .any(|m| m.measurement == measurement.measurement)
            {
                self.network_config.add_code_measurement(
                    measurement.measurement,
                    measurement.version,
                );
            }
        }

        Ok(())
    }
}
