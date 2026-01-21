use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Pending,
    Processing,
    AwaitingValidation,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ActionType {
    Llm,
    Agent(Pubkey),
    Tool(Pubkey),
    Agent2Agent(Pubkey, Pubkey), // agent_pubkey, target_agent_pubkey
}

#[account]
#[derive(InitSpace)]
pub struct Task {
    pub task_slot_id: u64,
    pub status: TaskStatus,
    pub compute_node: Option<Pubkey>, // Further on there will be a scheduler to assign tasks to nodes
    pub action_type: ActionType,
    pub chain_proof: [u8; 32],
    pub execution_count: u64,
    pub max_task_cost: u64,
    pub max_call_count: u64,
    pub call_count: u64, // Each task interaction can have multiple llm calls
    #[max_len(128)]
    pub input_cid: Option<String>,
    #[max_len(128)]
    pub output_cid: Option<String>,
    #[max_len(128)]
    pub pending_input_cid: Option<String>,
    #[max_len(128)]
    pub pending_output_cid: Option<String>,
    #[max_len(10)]
    pub approved_validators: Vec<Pubkey>,
    #[max_len(10)]
    pub rejected_validators: Vec<Pubkey>,
    pub bump: u8,
}
