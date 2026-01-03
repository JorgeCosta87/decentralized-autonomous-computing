use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Pending,
    Processing,
    AwaitingValidation,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ActionType {
    Llm,
    Tool(Pubkey),
    Agent2Agent(Pubkey),
}

#[account]
#[derive(InitSpace)]
pub struct Task {
    pub task_slot_id: u64,
    pub action_type: ActionType,
    pub agent: Pubkey,
    pub status: TaskStatus,
    pub compute_node: Option<Pubkey>,
    #[max_len(128)]
    pub input_cid: Option<String>,
    #[max_len(128)]
    pub output_cid: Option<String>,
    pub chain_proof: [u8; 32],
    pub execution_count: u64,
    pub bump: u8,
}
