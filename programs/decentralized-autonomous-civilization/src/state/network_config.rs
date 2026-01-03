use anchor_lang::prelude::*;
use crate::utils::SemanticVersion;


#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CodeMeasurement {
    pub measurement: [u8; 32],
    pub version: SemanticVersion,
}

#[account]
#[derive(InitSpace)]
pub struct NetworkConfig {
    #[max_len(128)]
    pub cid_config: String,
    pub agent_count: u64,
    pub goal_count: u64, 
    pub task_count: u64, 
    pub validator_node_count: u64,
    pub compute_node_count: u64,
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
    
    pub fn get_measurement_by_version(&self, version: &SemanticVersion) -> Option<&CodeMeasurement> {
        self.approved_code_measurements
            .iter()
            .find(|m| &m.version == version)
    }
}