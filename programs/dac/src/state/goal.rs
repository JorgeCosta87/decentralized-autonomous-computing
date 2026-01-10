use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum GoalStatus {
    Ready,
    Active,
}

#[account]
#[derive(InitSpace)]
pub struct Goal {
    pub goal_slot_id: u64,
    pub owner: Pubkey,
    pub agent: Pubkey,
    pub task: Pubkey,
    pub status: GoalStatus,
    #[max_len(128)]
    pub specification_cid: String, // IPFS CID of goal specification (Goal description, expected output, etc)
    pub max_iterations: u64,
    pub current_iteration: u64,
    pub task_index_at_goal_start: u64,
    pub task_index_at_goal_end: u64,
    pub total_shares: u64,
    pub locked_for_tasks: u64,
    pub chain_proof: [u8; 32],
    pub is_confidential: bool,
    pub vault_bump: u8,
    pub bump: u8,
}
