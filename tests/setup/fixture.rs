use litesvm::LiteSVM;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use utils::Utils;

use crate::setup::test_data::*;

pub struct TestFixture {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub authority: Keypair,
}

impl TestFixture {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new().with_builtins();
        let payer = Keypair::new();

        let program_id = svm.deploy_program_from_keypair(DAC_KEYPAIR_PATH, DAC_SO_PATH);

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund payer");

        let authority = Keypair::new();
        svm.airdrop(&authority.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund authority");

        Self {
            svm,
            program_id,
            payer,
            authority,
        }
    }

    pub fn create_keypair(&mut self) -> Keypair {
        let keypair = Keypair::new();
        self.svm
            .airdrop(&keypair.pubkey(), LAMPORTS_PER_SOL)
            .expect("Failed to fund keypair");
        keypair
    }
}
