pub mod dac;

// Re-export types to match generated code's expected paths (crate::generated::types)
pub mod types {
    pub use super::dac::types::*;
}
