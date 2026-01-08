use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::state::{NetworkConfig, NodeInfo, NodeStatus, NodeType};

#[derive(Accounts)]
#[instruction(node_pubkey: Pubkey)]
pub struct RegisterNode<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dac_network_config", network_config.authority.key().as_ref()],
        bump = network_config.bump,
    )]
    pub network_config: Account<'info, NetworkConfig>,
    #[account(
        init,
        payer = owner,
        space = 8 + NodeInfo::INIT_SPACE,
        seeds = [b"node_info", node_pubkey.key().as_ref()],
        bump,
    )]
    pub node_info: Account<'info, NodeInfo>,
    #[account(
        mut,
        seeds = [b"node_treasury", node_info.key().as_ref()],
        bump,
    )]
    pub node_treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> RegisterNode<'info> {
    pub fn register_node(
        &mut self,
        node_pubkey: Pubkey,
        node_type: NodeType,
        bumps: &RegisterNodeBumps,
    ) -> Result<()> {
        self.node_info.set_inner(NodeInfo {
            owner: self.owner.key(),
            node_pubkey: node_pubkey,
            node_type,
            status: NodeStatus::PendingClaim,
            node_info_cid: None,
            code_measurement: None,
            tee_signing_pubkey: None,
            node_treasury: self.node_treasury.key(),
            total_earned: 0,
            total_tasks_completed: 0,
            bump: bumps.node_info,
        });

        let node_info_key = self.node_info.key();
        let treasury_seeds = &[
            b"node_treasury",
            node_info_key.as_ref(),
            &[bumps.node_treasury],
        ];
        let treasury_signer = &[&treasury_seeds[..]];

        let cpi_accounts = system_program::CreateAccount {
            from: self.owner.to_account_info(),
            to: self.node_treasury.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            treasury_signer,
        );

        system_program::create_account(
            cpi_context,
            Rent::get()?.minimum_balance(0),
            0,
            &system_program::ID,
        )?;

        Ok(())
    }
}
