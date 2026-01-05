use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{message::Instruction, pubkey::Pubkey, signature::Keypair};
use utils::create_ed25519_instruction_with_signature;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidateComputeNodeMessage {
    pub compute_node_pubkey: Pubkey,
    pub approved: bool,
}

pub struct Helpers;

impl Helpers {
    pub fn create_ed25519_instruction_for_validate_compute_node(
        compute_node_pubkey: &Pubkey,
        approved: bool,
        signing_keypair: &Keypair,
    ) -> Instruction {
        let message = ValidateComputeNodeMessage {
            compute_node_pubkey: *compute_node_pubkey,
            approved: approved,
        };
        let mut message_data = Vec::new();
        message
            .serialize(&mut message_data)
            .expect("Failed to serialize message");

        create_ed25519_instruction_with_signature(&message_data, signing_keypair)
    }
}
