use litesvm::LiteSVM;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use utils::Utils;

use crate::setup::test_data::*;
use crate::setup::Instructions;
use crate::setup::{Accounts, Helpers};

pub struct TestFixture {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub authority: Keypair,

    // Node keypairs for testing
    pub compute_node_owner: Keypair,
    pub compute_node: Keypair,
    pub validator_node_owner: Keypair,
    pub validator_node: Keypair,
    pub tee_signing_keypair: Keypair,
    pub agent_owner: Keypair,
}

impl TestFixture {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new().with_precompiles().with_sysvars();

        let payer = Keypair::new();

        let program_id = svm.deploy_program_from_keypair(DAC_KEYPAIR_PATH, DAC_SO_PATH);

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund payer");

        let authority = Keypair::new();
        svm.airdrop(&authority.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund authority");

        // Create and fund node keypairs
        let compute_node_owner = Keypair::new();
        svm.airdrop(&compute_node_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund compute_node_owner");

        let compute_node = Keypair::new();
        svm.airdrop(&compute_node.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund compute_node");

        let validator_node_owner = Keypair::new();
        svm.airdrop(&validator_node_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund validator_node_owner");

        let validator_node = Keypair::new();
        svm.airdrop(&validator_node.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund validator_node");

        let tee_signing_keypair = Keypair::new();

        let agent_owner = Keypair::new();
        svm.airdrop(&agent_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund agent_owner");

        Self {
            svm,
            program_id,
            payer,
            authority,
            compute_node_owner,
            compute_node,
            validator_node_owner,
            validator_node,
            tee_signing_keypair,
            agent_owner,
        }
    }

    pub fn create_keypair(&mut self) -> Keypair {
        let keypair = Keypair::new();
        self.svm
            .airdrop(&keypair.pubkey(), LAMPORTS_PER_SOL)
            .expect("Failed to fund keypair");
        keypair
    }

    pub fn with_initialize_network(mut self) -> Self {
        let network_config_pda = self.find_network_config_pda(&self.authority.pubkey()).0;

        let remaining_accounts = self.create_remaining_accounts_for_initialize(
            &network_config_pda,
            DEFAULT_ALLOCATE_GOALS,
            DEFAULT_ALLOCATE_TASKS,
        );

        let result = self.initialize_network(
            &self.authority.insecure_clone(),
            &network_config_pda,
            DEFAULT_CID_CONFIG.to_string(),
            DEFAULT_ALLOCATE_GOALS,
            DEFAULT_ALLOCATE_TASKS,
            DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec(),
            &remaining_accounts,
        );
        assert!(result.is_ok(), "Failed to initialize network");
        self
    }

    pub fn with_register_compute_node(mut self) -> Self {
        let result = self.register_node(
            &self.compute_node_owner.insecure_clone(),
            &self.compute_node.pubkey(),
            dac_client::dac::types::NodeType::Compute,
        );
        assert!(result.is_ok(), "Failed to register compute node");
        self
    }

    pub fn with_register_validator_node(mut self) -> Self {
        let validator_node_owner = self.validator_node_owner.insecure_clone();
        let validator_node_pubkey = self.validator_node.pubkey();
        let result = self.register_node(
            &validator_node_owner,
            &validator_node_pubkey,
            dac_client::dac::types::NodeType::Validator,
        );
        assert!(result.is_ok(), "Failed to register validator node");
        self
    }

    pub fn with_claim_compute_node(mut self) -> Self {
        let compute_node = self.compute_node.insecure_clone();
        let result = self.claim_compute_node(&compute_node, DEFAULT_NODE_INFO_CID.to_string());
        assert!(result.is_ok(), "Failed to claim compute node");
        self
    }

    pub fn with_claim_validator_node(mut self) -> Self {
        let validator_node = self.validator_node.insecure_clone();
        let tee_signing_pubkey = self.tee_signing_keypair.pubkey();
        let result = self.claim_validator_node(
            &validator_node,
            DEFAULT_CODE_MEASUREMENT,
            tee_signing_pubkey,
        );
        assert!(result.is_ok(), "Failed to claim validator node");
        self
    }

    pub fn with_validate_compute_node(mut self, approved: bool) -> Self {
        let validator_node = self.validator_node.insecure_clone();
        let compute_node_pubkey = self.compute_node.pubkey();
        let tee_signing_keypair = self.tee_signing_keypair.insecure_clone();

        let ed25519_ix = Helpers::create_ed25519_instruction_to_validate_compute_node(
            &compute_node_pubkey,
            approved,
            &tee_signing_keypair,
        );

        let result = self.validate_compute_node(&validator_node, &compute_node_pubkey, &ed25519_ix);
        assert!(result.is_ok(), "Failed to validate compute node");
        self
    }

    pub fn with_create_agent(mut self) -> Self {
        let agent_owner = self.agent_owner.insecure_clone();
        let result = self.create_agent(&agent_owner, crate::setup::test_data::DEFAULT_AGENT_CONFIG_CID.to_string());
        assert!(result.is_ok(), "Failed to create agent");
        self
    }
}
