use litesvm::{
    types::{TransactionMetadata, TransactionResult},
    LiteSVM,
};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
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
        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(payer),
            signing_keypairs,
            blockhash,
        );
        let result = self.send_transaction(tx);

        result
    }

    fn get_lamports(&self, address: &Pubkey) -> u64 {
        self.get_account(address)
            .unwrap_or_else(|| panic!("Account not found: {}", address))
            .lamports
    }
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
