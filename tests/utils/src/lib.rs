use ed25519_dalek::Signer;
use litesvm::{
    types::{TransactionMetadata, TransactionResult},
    LiteSVM,
};
use solana_ed25519_program::new_ed25519_instruction_with_signature;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer as SolanaSigner},
    transaction::Transaction,
};

pub trait Utils {
    fn deploy_program_from_keypair(&mut self, keypair_path: &str, so_path: &str) -> Pubkey;
    fn deploy_program_from_id(&mut self, program_id: Pubkey, so_path: &str) -> Pubkey;
    fn print_transaction_logs(&self, result: &TransactionMetadata);
    fn send_tx(
        &mut self,
        instructions: &[Instruction],
        payer: &Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult;
    fn get_lamports(&self, address: &Pubkey) -> u64;
}

impl Utils for LiteSVM {
    fn deploy_program_from_keypair(&mut self, keypair_path: &str, so_path: &str) -> Pubkey {
        let program_keypair = read_keypair_file(keypair_path).expect("Failed to read keypair file");
        let program_id = program_keypair.pubkey();
        println!("Deploying program from keypair: {}", program_id);

        deploy_program_internal(self, program_id, so_path)
    }

    fn deploy_program_from_id(&mut self, program_id: Pubkey, so_path: &str) -> Pubkey {
        deploy_program_internal(self, program_id, so_path)
    }

    fn print_transaction_logs(&self, result: &TransactionMetadata) {
        println!("\nTransaction logs:");
        for log in &result.logs {
            println!("  {}", log);
        }
    }

    fn send_tx(
        &mut self,
        instructions: &[Instruction],
        payer: &Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let blockhash = self.latest_blockhash();
        let message = Message::new(instructions, Some(payer));
        let mut tx = Transaction::new_unsigned(message);
        tx.sign(signing_keypairs, blockhash);
        let result = self.send_transaction(tx);

        result
    }

    fn get_lamports(&self, address: &Pubkey) -> u64 {
        self.get_account(address)
            .unwrap_or_else(|| panic!("Account not found: {}", address))
            .lamports
    }
}

pub fn create_ed25519_instruction_with_signature(
    message: &[u8],
    key_pair: &Keypair,
) -> Instruction {
    // message is already serialized bytes, use directly
    let message_data = message.to_vec();

    let tee_keypair_bytes = key_pair.to_bytes();
    let mut tee_secret_bytes = [0u8; 32];
    tee_secret_bytes.copy_from_slice(&tee_keypair_bytes[..32]);
    let tee_secret_key = ed25519_dalek::SigningKey::from_bytes(&tee_secret_bytes);
    let signature = tee_secret_key.sign(&message_data);

    let tee_pubkey = key_pair.pubkey();
    let mut tee_pubkey_bytes = [0u8; 32];
    tee_pubkey_bytes.copy_from_slice(tee_pubkey.as_ref());
    let signature_bytes: [u8; 64] = signature.to_bytes();

    new_ed25519_instruction_with_signature(&message_data, &signature_bytes, &tee_pubkey_bytes)
}

fn deploy_program_internal(svm: &mut LiteSVM, program_id: Pubkey, so_path: &str) -> Pubkey {
    svm.add_program_from_file(program_id, so_path)
        .expect("Failed to deploy program from file");

    assert!(
        svm.get_account(&program_id).is_some(),
        "Program account not created"
    );
    assert!(
        svm.get_account(&program_id).unwrap().executable,
        "Program not executable"
    );

    program_id
}
