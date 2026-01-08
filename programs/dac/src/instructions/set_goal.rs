use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::ErrorCode;
use crate::state::{Agent, AgentStatus, Contribution, Goal, GoalStatus, Task, TaskStatus};
use crate::ActionType;
use crate::NetworkConfig;

#[derive(Accounts)]
pub struct SetGoal<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"goal", network_config.key().as_ref(), goal.goal_slot_id.to_le_bytes().as_ref()],
        bump = goal.bump,
    )]
    pub goal: Account<'info, Goal>,

    #[account(
        mut,
        seeds = [b"goal_vault", goal.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [b"contribution", goal.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub owner_contribution: Account<'info, Contribution>,

    #[account(
        mut,
        seeds = [b"task", network_config.key().as_ref(), task.task_slot_id.to_le_bytes().as_ref()],
        bump = task.bump,
    )]
    pub task: Account<'info, Task>,

    #[account(
        seeds = [b"agent", network_config.key().as_ref(), agent.agent_slot_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        seeds = [b"dac_network_config", network_config.authority.key().as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> SetGoal<'info> {
    pub fn set_goal(
        &mut self,
        specification_cid: String,
        max_iterations: u64,
        initial_deposit: u64,
        bumps: &SetGoalBumps,
    ) -> Result<()> {
        require!(
            self.goal.status == GoalStatus::Ready,
            ErrorCode::InvalidGoalStatus
        );
        require!(
            self.goal.owner == Pubkey::default() || self.goal.owner == self.owner.key(),
            ErrorCode::InvalidGoalOwner
        );
        require!(
            self.task.status == TaskStatus::Ready,
            ErrorCode::InvalidTaskStatus
        );
        require!(
            self.agent.status == AgentStatus::Active,
            ErrorCode::InvalidAgentStatus
        );
        require!(initial_deposit > 0, ErrorCode::DepositTooSmall);

        // Check if vault only has rent lamports (no leftover SOL from previous goal)
        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(0);
        let vault_balance = self.vault.lamports();
        require!(
            vault_balance == rent_exempt_minimum || vault_balance == 0,
            ErrorCode::VaultHasLeftoverFunds
        );

        // If reusing goal (current_iteration > 0), reset execution and payment state
        if self.goal.current_iteration > 0 {
            self.goal.current_iteration = 0;
            self.goal.task_index_at_goal_start = self.goal.task_index_at_goal_end;
            self.goal.task_index_at_goal_end = 0;
            self.goal.total_shares = 0;
            self.goal.locked_for_tasks = 0;
        }

        let goal_key = self.goal.key();
        let vault_seeds = &[b"goal_vault", goal_key.as_ref(), &[bumps.vault]];
        let vault_signer = &[&vault_seeds[..]];

        if vault_balance == 0 {
            // Vault doesn't exist
            let required_lamports = rent_exempt_minimum;
            let transfer_amount = initial_deposit
                .checked_add(required_lamports)
                .ok_or(ErrorCode::Overflow)?;

            let cpi_accounts = system_program::CreateAccount {
                from: self.owner.to_account_info(),
                to: self.vault.to_account_info(),
            };
            let cpi_context = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                cpi_accounts,
                vault_signer,
            );

            system_program::create_account(cpi_context, transfer_amount, 0, &system_program::ID)?;
        } else {
            // Vault exists with only rent
            let cpi_accounts = system_program::Transfer {
                from: self.owner.to_account_info(),
                to: self.vault.to_account_info(),
            };
            let cpi_context = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
            system_program::transfer(cpi_context, initial_deposit)?;
        }

        // Mint shares for owner's initial deposit
        // First deposit always uses share_price = 1.0
        let share_price = 1.0_f64;
        let shares = (initial_deposit as f64 / share_price) as u64;
        require!(shares > 0, ErrorCode::Overflow);

        self.owner_contribution.set_inner(Contribution {
            goal: self.goal.key(),
            contributor: self.owner.key(),
            shares,
            refund_amount: 0,
            bump: bumps.owner_contribution,
        });

        self.goal.owner = self.owner.key();
        self.goal.agent = self.agent.key();
        self.goal.task = self.task.key();
        self.goal.specification_cid = specification_cid;
        self.goal.max_iterations = max_iterations;
        self.goal.total_shares = shares;
        self.goal.status = GoalStatus::Active;
        self.goal.vault_bump = bumps.vault;
        self.goal.task_index_at_goal_start = self.task.execution_count;

        self.task.status = TaskStatus::Pending; // Task is pending for execution
        self.task.agent = self.agent.key();
        self.task.action_type = ActionType::Llm;

        Ok(())
    }
}
