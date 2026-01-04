use dac_client::dac::types::{CodeMeasurement, SemanticVersion};
use sha2::{Digest, Sha256};

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
