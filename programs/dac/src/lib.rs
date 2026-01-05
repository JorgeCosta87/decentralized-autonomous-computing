use anchor_lang::prelude::*;

declare_id!("GtupmyvcYoz9DXZ1qpYS4FPMtz3EHHHTzyRGWBJYevKQ");

pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::*;
pub use state::*;

#[program]
pub mod dac {
    use super::*;

    pub fn initialize_network<'info>(
        ctx: Context<'_, '_, '_, 'info, InitializeNetwork<'info>>,
        cid_config: String,
        allocate_goals: u64,
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
    ) -> Result<()> {
        ctx.accounts.initialize_network(
            cid_config,
            allocate_goals,
            allocate_tasks,
            approved_code_measurements,
            &ctx.remaining_accounts,
            &ctx.bumps,
        )
    }

    pub fn register_node(
        ctx: Context<RegisterNode>,
        node_pubkey: Pubkey,
        node_type: NodeType,
    ) -> Result<()> {
        ctx.accounts
            .register_node(node_pubkey, node_type, &ctx.bumps)
    }

    pub fn claim_compute_node(ctx: Context<ClaimComputeNode>, node_info_cid: String) -> Result<()> {
        ctx.accounts.claim_compute_node(node_info_cid)
    }

    pub fn claim_validator_node<'info>(
        ctx: Context<ClaimValidatorNode>,
        code_measurement: [u8; 32],
        tee_signing_pubkey: Pubkey,
    ) -> Result<()> {
        ctx.accounts
            .claim_validator_node(code_measurement, tee_signing_pubkey)
    }

    pub fn validate_compute_node(ctx: Context<ValidateComputeNode>) -> Result<()> {
        ctx.accounts.validate_compute_node()
    }
}
