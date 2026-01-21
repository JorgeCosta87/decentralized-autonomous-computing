use anchor_lang::prelude::*;

#[event]
pub struct TaskClaimed {
    pub goal_slot_id: u64,
    pub task_slot_id: u64,
    pub compute_node: Pubkey,
    pub max_task_cost: u64,
}

#[event]
pub struct TaskResultSubmitted {
    pub goal_slot_id: u64,
    pub task_slot_id: u64,
    pub input_cid: String,
    pub output_cid: String,
}

#[event]
pub struct TaskValidationSubmitted {
    pub goal_slot_id: u64,
    pub task_slot_id: u64,
    pub validator: Pubkey,
    pub payment_amount: u64,
    pub approved: bool,
    pub goal_completed: bool,
    pub current_iteration: u64,
    pub vault_balance: u64,
    pub locked_for_tasks: u64,
}

#[event]
pub struct GoalSet {
    pub goal_slot_id: u64,
    pub owner: Pubkey,
    pub agent_slot_id: u64,
    pub task_slot_id: u64,
    pub specification_cid: String,
    pub max_iterations: u64,
    pub initial_deposit: u64,
}

#[event]
pub struct ContributionMade {
    pub goal_slot_id: u64,
    pub contributor: Pubkey,
    pub deposit_amount: u64,
    pub shares_minted: u64,
    pub total_shares: u64,
}

#[event]
pub struct GoalCompleted {
    pub goal_slot_id: u64,
    pub final_iteration: u64,
    pub vault_balance: u64,
}

#[event]
pub struct NodeValidated {
    pub node: Pubkey,
    pub validator: Pubkey,
    pub goal_slot_id: Option<u64>,
    pub task_slot_id: Option<u64>,
}

#[event]
pub struct NodeRejected {
    pub node: Pubkey,
    pub validator: Pubkey,
    pub goal_slot_id: Option<u64>,
    pub task_slot_id: Option<u64>,
}

#[event]
pub struct AgentCreated {
    pub agent_slot_id: u64,
    pub owner: Pubkey,
    pub agent_config_cid: String,
}
