use crate::errors::ErrorCode;
use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum NodeType {
    Validator,
    Compute,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum NodeStatus {
    PendingClaim,
    AwaitingValidation,
    Active,
    Disabled,
    Rejected,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RewardEntry {
    pub amount: u64,
    pub slot: u64,
}

#[account]
#[derive(InitSpace)]
pub struct NodeInfo {
    pub owner: Pubkey,
    pub node_pubkey: Pubkey,
    pub node_type: NodeType,
    pub status: NodeStatus,
    #[max_len(128)]
    pub node_info_cid: Option<String>,
    pub code_measurement: Option<[u8; 32]>,
    pub tee_signing_pubkey: Option<Pubkey>,
    pub node_treasury: Pubkey,
    #[max_len(64)]
    pub recent_rewards: Vec<RewardEntry>,
    pub total_earned: u64,
    pub max_entries_before_transfer: u64,
    pub last_transfer_slot: u64,
    pub total_tasks_completed: u64,
    pub bump: u8,
}

impl NodeInfo {
    pub fn add_reward(&mut self, amount: u64, slot: u64) -> Result<()> {
        require!(self.recent_rewards.len() < 64, ErrorCode::RewardVectorFull);

        self.recent_rewards.push(RewardEntry { amount, slot });
        Ok(())
    }

    pub fn clear_rewards(&mut self) {
        self.recent_rewards.clear();
    }

    pub fn total_pending_rewards(&self) -> u64 {
        self.recent_rewards.iter().map(|r| r.amount).sum()
    }
}
