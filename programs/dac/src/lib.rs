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

    pub fn create_agent(ctx: Context<CreateAgent>, agent_config_cid: String) -> Result<()> {
        ctx.accounts.create_agent(agent_config_cid, &ctx.bumps)
    }

    pub fn validate_agent(ctx: Context<ValidateAgent>) -> Result<()> {
        ctx.accounts.validate_agent()
    }

    pub fn create_goal(ctx: Context<CreateGoal>, is_public: bool) -> Result<()> {
        ctx.accounts.create_goal(is_public, &ctx.bumps)
    }

    pub fn set_goal(
        ctx: Context<SetGoal>,
        specification_cid: String,
        max_iterations: u64,
        initial_deposit: u64,
    ) -> Result<()> {
        ctx.accounts
            .set_goal(specification_cid, max_iterations, initial_deposit, &ctx.bumps)
    }

    pub fn contribute_to_goal(
        ctx: Context<ContributeToGoal>,
        deposit_amount: u64,
    ) -> Result<()> {
        ctx.accounts.contribute_to_goal(deposit_amount, &ctx.bumps)
    }

    pub fn withdraw_from_goal(
        ctx: Context<WithdrawFromGoal>,
        shares_to_burn: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw_from_goal(shares_to_burn)
    }
}
