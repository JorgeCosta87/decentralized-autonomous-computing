use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum AgentStatus {
    Pending,
    Active,
    Inactive,
}

#[account]
#[derive(InitSpace)]
pub struct Agent {
    pub agent_slot_id: u64,
    pub owner: Pubkey,
    pub status: AgentStatus,
    #[max_len(128)]
    pub agent_config_cid: String,
    #[max_len(128)]
    pub agent_memory_cid: Option<String>,
    #[max_len(10)]
    pub approved_validators: Vec<Pubkey>,
    #[max_len(10)]
    pub rejected_validators: Vec<Pubkey>,
    pub bump: u8,
}
