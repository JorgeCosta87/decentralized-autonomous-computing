use crate::errors::ErrorCode;
use crate::utils::SemanticVersion;
use anchor_lang::prelude::*;
use sha2::{Digest, Sha256};

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CodeMeasurement {
    pub measurement: [u8; 32],
    pub version: SemanticVersion,
}

#[account]
#[derive(InitSpace)]
pub struct NetworkConfig {
    pub authority: Pubkey,
    #[max_len(128)]
    pub cid_config: String,
    pub genesis_hash: [u8; 32],
    pub task_count: u64,
    pub required_validations: u32,
    //TODO: This needs to be a separate account
    #[max_len(32)]
    pub allowed_models: Vec<u64>, // this needs to match the models in config
    //TODO: Nodes registery should be another account
    #[max_len(32)]
    pub approved_confidential_nodes: Vec<Pubkey>,
    #[max_len(32)]
    pub approved_public_nodes: Vec<Pubkey>,
    //TODO: This should be on another smart contract
    pub agent_count: u64,
    pub session_count: u64,

    #[max_len(10)]
    pub approved_code_measurements: Vec<CodeMeasurement>,

    pub bump: u8,
}

impl NetworkConfig {
    pub fn add_code_measurement(&mut self, measurement: [u8; 32], version: SemanticVersion) {
        let new_measurement = CodeMeasurement {
            measurement,
            version,
        };

        self.approved_code_measurements.insert(0, new_measurement);

        if self.approved_code_measurements.len() > 10 {
            self.approved_code_measurements.pop();
        }
    }

    pub fn is_measurement_approved(&self, measurement: &[u8; 32]) -> bool {
        self.approved_code_measurements
            .iter()
            .any(|m| &m.measurement == measurement)
    }

    pub fn get_latest_measurement(&self) -> Option<&CodeMeasurement> {
        self.approved_code_measurements.first()
    }

    pub fn compute_genesis_hash(&self) -> Result<[u8; 32]> {
        let mut hasher = Sha256::new();
        hasher.update(b"DAC_GENESIS");
        Ok(hasher.finalize().into())
    }

    pub fn increment_agent_count(&mut self) -> Result<()> {
        self.agent_count = self.agent_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
        Ok(())
    }

    pub fn next_agent_slot_id(&self) -> u64 {
        self.agent_count
    }

    pub fn next_session_slot_id(&self) -> u64 {
        self.session_count
    }

    pub fn next_task_slot_id(&self) -> u64 {
        self.task_count
    }

    pub fn increment_session_count(&mut self) -> Result<()> {
        self.session_count = self.session_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
        Ok(())
    }

    pub fn increment_task_count(&mut self) -> Result<()> {
        self.task_count = self.task_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
        Ok(())
    }

    pub fn add_confidential_node(&mut self, node_pubkey: Pubkey) -> Result<()> {
        self.approved_confidential_nodes.push(node_pubkey);
        if self.approved_confidential_nodes.len() > 10 {
            self.approved_confidential_nodes.pop();
        }
        Ok(())
    }

    pub fn add_public_node(&mut self, node_pubkey: Pubkey) -> Result<()> {
        self.approved_public_nodes.push(node_pubkey);
        if self.approved_public_nodes.len() > 10 {
            self.approved_public_nodes.pop();
        }
        Ok(())
    }
}
