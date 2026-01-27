use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Pending,
    Processing,
    AwaitingValidation,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum TaskType {
    Completion(u64), // model id
    Custom(u64),     //module identifier
    HumanInLoop,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ValidationStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub struct Validator {
    pub pubkey: Pubkey,
    pub status: ValidationStatus,
}

#[account]
#[derive(InitSpace)]
pub struct Task {
    pub task_slot_id: u64,
    pub session_slot_id: Option<u64>,
    pub status: TaskStatus,
    pub compute_node: Option<Pubkey>, // Further on there will be a scheduler to assign tasks to nodes
    pub task_type: TaskType,
    pub chain_proof: [u8; 32],
    pub task_index: u64,
    pub max_task_cost: u64,
    pub max_call_count: u64,
    pub call_count: u64, // Each task execution can have multiple calls
    #[max_len(128)]
    pub input_cid: Option<String>,
    #[max_len(128)]
    pub output_cid: Option<String>,
    #[max_len(128)]
    pub pending_input_cid: Option<String>,
    #[max_len(128)]
    pub pending_output_cid: Option<String>,
    #[max_len(10)]
    pub validations: Vec<Validator>,
    pub bump: u8,
}
