use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum GoalStatus {
    Pending,
    Active,
    Completed,
}

#[account]
#[derive(InitSpace)]
pub struct Goal {
    pub goal_slot_id: u64,
    pub owner: Pubkey,
    pub agent: Pubkey,
    pub task_data: Pubkey,
    pub status: GoalStatus,
    #[max_len(200)]
    pub description_cid: String,
    pub max_iterations: u64,
    pub current_iteration: u64,
    pub task_index_at_goal_start: u64,
    pub task_index_at_goal_end: u64,
    pub bump: u8,
}
