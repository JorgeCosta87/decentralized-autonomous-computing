use anchor_lang::prelude::*;

declare_id!("C821M111okror4TN2gjdUBJK58Mbd2s94pZc8xH6S5BQ");

pub mod state;
pub mod utils;
pub mod errors;
pub mod instructions;

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
}

