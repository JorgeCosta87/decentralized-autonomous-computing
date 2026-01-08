use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{message::Instruction, pubkey::Pubkey, signature::Keypair};
use utils::create_ed25519_instruction_with_signature;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidateComputeNodeMessage {
    pub compute_node_pubkey: Pubkey,
    pub approved: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct SubmitTaskValidationMessage {
    pub goal_id: u64,
    pub task_slot_id: u64,
    pub payment_amount: u64,
    pub validation_proof: [u8; 32],
    pub approved: bool,
    pub goal_completed: bool,
}

pub struct Helpers;

impl Helpers {
    pub fn create_ed25519_instruction_to_validate_compute_node(
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

    pub fn create_ed25519_instruction_to_submit_task_validation(
        goal_id: u64,
        task_slot_id: u64,
        payment_amount: u64,
        validation_proof: [u8; 32],
        approved: bool,
        goal_completed: bool,
        signing_keypair: &Keypair,
    ) -> Instruction {
        let message = SubmitTaskValidationMessage {
            goal_id,
            task_slot_id,
            payment_amount,
            validation_proof,
            approved,
            goal_completed,
        };
        let mut message_data = Vec::new();
        message
            .serialize(&mut message_data)
            .expect("Failed to serialize message");

        create_ed25519_instruction_with_signature(&message_data, signing_keypair)
    }
}
