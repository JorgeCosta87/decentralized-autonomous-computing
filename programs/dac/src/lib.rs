use anchor_lang::prelude::*;

declare_id!("BaY9vp3RXAQugzAoBojkBEZs9fJKS4dNManN7vwDZSFh");

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

pub use events::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[program]
pub mod dac {
    use super::*;

    pub fn initialize_network<'info>(
        ctx: Context<'_, '_, '_, 'info, InitializeNetwork<'info>>,
        cid_config: String,
        allocate_goals: u64,
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
        required_validations: u32,
    ) -> Result<()> {
        ctx.accounts.initialize_network(
            cid_config,
            allocate_goals,
            allocate_tasks,
            approved_code_measurements,
            required_validations,
            &ctx.remaining_accounts,
            &ctx.bumps,
        )
    }

    pub fn update_network_config(
        ctx: Context<UpdateNetworkConfig>,
        cid_config: Option<String>,
        new_code_measurement: Option<CodeMeasurement>,
    ) -> Result<()> {
        ctx.accounts
            .update_network_config(cid_config, new_code_measurement)
    }

    pub fn register_node(
        ctx: Context<RegisterNode>,
        node_pubkey: Pubkey,
        node_type: NodeType,
    ) -> Result<()> {
        ctx.accounts
            .register_node(node_pubkey, node_type, &ctx.bumps)
    }

    pub fn claim_public_node(ctx: Context<ClaimPublicNode>, node_info_cid: String) -> Result<()> {
        ctx.accounts.claim_public_node(node_info_cid)
    }

    pub fn claim_confidential_node<'info>(
        ctx: Context<ClaimConfidentialNode>,
        code_measurement: [u8; 32],
        tee_signing_pubkey: Pubkey,
    ) -> Result<()> {
        ctx.accounts
            .claim_confidential_node(code_measurement, tee_signing_pubkey)
    }

    pub fn validate_public_node(ctx: Context<ValidatePublicNode>, approved: bool) -> Result<()> {
        ctx.accounts.validate_public_node(approved)
    }

    pub fn activate_node(ctx: Context<ActivateNode>) -> Result<()> {
        ctx.accounts.activate_node()
    }

    pub fn create_agent(ctx: Context<CreateAgent>, agent_config_cid: String) -> Result<()> {
        ctx.accounts.create_agent(agent_config_cid, &ctx.bumps)
    }

    pub fn validate_agent(ctx: Context<ValidateAgent>) -> Result<()> {
        ctx.accounts.validate_agent()
    }

    pub fn create_goal(
        ctx: Context<CreateGoal>,
        is_owned: bool,
        is_confidential: bool,
    ) -> Result<()> {
        ctx.accounts
            .create_goal(is_owned, is_confidential, &ctx.bumps)
    }

    pub fn set_goal(
        ctx: Context<SetGoal>,
        specification_cid: String,
        max_iterations: u64,
        initial_deposit: u64,
    ) -> Result<()> {
        ctx.accounts.set_goal(
            specification_cid,
            max_iterations,
            initial_deposit,
            &ctx.bumps,
        )
    }

    pub fn contribute_to_goal(ctx: Context<ContributeToGoal>, deposit_amount: u64) -> Result<()> {
        ctx.accounts.contribute_to_goal(deposit_amount, &ctx.bumps)
    }

    pub fn withdraw_from_goal(ctx: Context<WithdrawFromGoal>, shares_to_burn: u64) -> Result<()> {
        ctx.accounts.withdraw_from_goal(shares_to_burn)
    }

    pub fn claim_task(ctx: Context<ClaimTask>, max_task_cost: u64) -> Result<()> {
        ctx.accounts.claim_task(max_task_cost)
    }

    pub fn submit_task_result(
        ctx: Context<SubmitTaskResult>,
        input_cid: String,
        output_cid: String,
    ) -> Result<()> {
        ctx.accounts.submit_task_result(input_cid, output_cid)
    }

    // Note: submit_confidential_task_validation handles TEE-based validation (requires Ed25519 instruction)
    pub fn submit_confidential_task_validation(ctx: Context<SubmitTaskValidation>) -> Result<()> {
        ctx.accounts.submit_confidential_task_validation()
    }

    // Note: submit_public_task_validation handles common validation (validators provide parameters directly)
    pub fn submit_public_task_validation(
        ctx: Context<SubmitTaskValidation>,
        payment_amount: u64,
        approved: bool,
        goal_completed: bool,
    ) -> Result<()> {
        ctx.accounts
            .submit_public_task_validation(payment_amount, approved, goal_completed)
    }
}
