use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemanticVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => match self.minor.cmp(&other.minor) {
                std::cmp::Ordering::Equal => self.patch.cmp(&other.patch),
                ord => ord,
            },
            ord => ord,
        }
    }
}
