use dac_client::types::{CodeMeasurement, SemanticVersion};
use sha2::Digest;

// Program paths and IDs
pub const DAC_KEYPAIR_PATH: &str = "target/deploy/dac-keypair.json";
pub const DAC_SO_PATH: &str = "target/deploy/dac.so";

//test data
pub const DEFAULT_CID_CONFIG: &str = "QmDefaultConfig";
pub const DEFAULT_ALLOCATE_GOALS: u64 = 2;
pub const DEFAULT_ALLOCATE_TASKS: u64 = 3;
pub const DEFAULT_GENESIS_HASH: [u8; 32] = [0u8; 32];
pub const DEFAULT_APPROVED_CODE_MEASUREMENTS: [CodeMeasurement; 1] = [CodeMeasurement {
    measurement: [1u8; 32],
    version: SemanticVersion {
        major: 0,
        minor: 0,
        patch: 0,
    },
}];
pub fn compute_genesis_hash() -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(b"DAC_GENESIS");
    hasher.finalize().into()
}

// Node test data
pub const DEFAULT_NODE_INFO_CID: &str = "QmNodeInfoCID";
pub const DEFAULT_CODE_MEASUREMENT: [u8; 32] = [1u8; 32];

// Agent test data
pub const DEFAULT_AGENT_CONFIG_CID: &str = "QmAgentConfigCID";

// Goal test data
pub const DEFAULT_GOAL_SPECIFICATION_CID: &str = "QmGoalSpecificationCID";
pub const DEFAULT_INITIAL_DEPOSIT: u64 = 1_000_000_000; // 1 SOL
pub const DEFAULT_CONTRIBUTION_AMOUNT: u64 = 500_000_000; // 0.5 SOL
