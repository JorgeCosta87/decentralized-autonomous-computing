use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum SessionStatus {
    Pending,
    Active,
    Completed,
    //TODO: Add refund status in the future
}

#[account]
#[derive(InitSpace)]
pub struct Session {
    pub session_slot_id: u64,
    pub owner: Pubkey,
    pub task: Pubkey,
    pub status: SessionStatus,
    pub is_confidential: bool,
    pub max_iterations: u64, // 0 is infinite
    pub current_iteration: u64,
    pub task_index_start: u64,
    pub task_index_end: u64,
    pub total_shares: u64,
    pub locked_for_tasks: u64,
    #[max_len(128)]
    pub specification_cid: String, // IPFS CID of session specification
    #[max_len(128)]
    pub state_cid: Option<String>, // IPFS CID of session state
    pub vault_bump: u8,
    pub bump: u8,
}
