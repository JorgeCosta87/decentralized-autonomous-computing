use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum NodeType {
    Public,
    Confidential,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum NodeStatus {
    PendingClaim,
    AwaitingValidation,
    Active,
    Disabled,
    Rejected,
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
    pub total_earned: u64,
    pub total_tasks_completed: u64,
    #[max_len(10)]
    pub approved_validators: Vec<Pubkey>,
    #[max_len(10)]
    pub rejected_validators: Vec<Pubkey>,
    pub bump: u8,
}
