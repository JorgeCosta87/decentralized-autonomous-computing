use dac_client::types::NodeStatus;
use dac_client::NodeType;
use litesvm::LiteSVM;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use utils::Utils;

use crate::setup::test_data::*;
use crate::setup::Accounts;
use crate::setup::Instructions;

pub struct TestFixture {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub authority: Keypair,

    // Keypairs for testing
    pub public_node_owner: Keypair,
    pub public_node: Keypair,
    pub confidential_node_owner: Keypair,
    pub confidential_node: Keypair,
    pub tee_signing_keypair: Keypair,
    pub agent_owner: Keypair,
    pub contributor: Keypair,
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

        let public_node_owner = Keypair::new();
        svm.airdrop(&public_node_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund public_node_owner");

        let public_node = Keypair::new();
        svm.airdrop(&public_node.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund public_node");

        let confidential_node_owner = Keypair::new();
        svm.airdrop(&confidential_node_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund confidential_node_owner");

        let confidential_node = Keypair::new();
        svm.airdrop(&confidential_node.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund confidential_node");

        let public_node_owner = Keypair::new();
        svm.airdrop(&public_node_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund public_node_owner");

        let public_node = Keypair::new();
        svm.airdrop(&public_node.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund public_node");

        let tee_signing_keypair = Keypair::new();

        let agent_owner = Keypair::new();
        svm.airdrop(&agent_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund agent_owner");

        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund contributor");

        Self {
            svm,
            program_id,
            payer,
            authority,
            public_node_owner,
            public_node,
            confidential_node_owner,
            confidential_node,
            tee_signing_keypair,
            agent_owner,
            contributor,
        }
    }

    pub fn create_keypair(&mut self) -> Keypair {
        let keypair = Keypair::new();
        self.svm
            .airdrop(&keypair.pubkey(), LAMPORTS_PER_SOL * 10)
            .expect("Failed to fund keypair");
        keypair
    }

    pub fn with_initialize_network(mut self) -> Self {
        let network_config_pda = self.find_network_config_pda().0;

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
            crate::setup::test_data::DEFAULT_REQUIRED_VALIDATIONS,
            &remaining_accounts,
        );
        assert!(result.is_ok(), "Failed to initialize network");
        self
    }

    pub fn with_register_public_node(mut self) -> Self {
        let result = self.register_node(
            &self.public_node_owner.insecure_clone(),
            &self.public_node.pubkey(),
            NodeType::Public,
        );
        assert!(result.is_ok(), "Failed to register public node");
        self
    }

    pub fn with_register_confidential_node(mut self) -> Self {
        let confidential_node_owner = self.confidential_node_owner.insecure_clone();
        let confidential_node_pubkey = self.confidential_node.pubkey();
        let result = self.register_node(
            &confidential_node_owner,
            &confidential_node_pubkey,
            NodeType::Confidential,
        );
        assert!(result.is_ok(), "Failed to register confidential node");
        self
    }

    pub fn with_claim_public_node(mut self) -> Self {
        let public_node = self.public_node.insecure_clone();
        let result = self.claim_compute_node(&public_node, DEFAULT_NODE_INFO_CID.to_string());
        assert!(result.is_ok(), "Failed to claim public node");
        self
    }

    pub fn with_claim_confidential_node(mut self) -> Self {
        let confidential_node = self.confidential_node.insecure_clone();
        let tee_signing_pubkey = self.tee_signing_keypair.pubkey();
        let result = self.claim_confidential_node(
            &confidential_node,
            DEFAULT_CODE_MEASUREMENT,
            tee_signing_pubkey,
        );
        assert!(result.is_ok(), "Failed to claim confidential node");
        self
    }

    pub fn with_register_public_validator_node(mut self) -> Self {
        let result = self.register_node(
            &self.public_node_owner.insecure_clone(),
            &self.public_node.pubkey(),
            NodeType::Public,
        );
        assert!(result.is_ok(), "Failed to register public validator node");
        self
    }

    pub fn with_claim_public_validator_node(mut self) -> Self {
        let public_node = self.public_node.insecure_clone();
        let result = self.claim_compute_node(&public_node, DEFAULT_NODE_INFO_CID.to_string());
        assert!(result.is_ok(), "Failed to claim public validator node");
        self
    }

    pub fn with_create_agent(mut self) -> Self {
        let agent_owner = self.agent_owner.insecure_clone();
        let result = self.create_agent(
            &agent_owner,
            crate::setup::test_data::DEFAULT_AGENT_CONFIG_CID.to_string(),
        );
        assert!(result.is_ok(), "Failed to create agent");
        self
    }

    pub fn with_validated_agent(mut self, agent_slot_id: u64) -> Self {
        let (node_info_pda, _) = self.find_node_info_pda(&self.confidential_node.pubkey());
        let account = self.svm.get_account(&node_info_pda);

        let needs_registration = account.is_none();
        let needs_claim = if let Some(acc) = account {
            use dac_client::accounts::NodeInfo;
            let node_info =
                NodeInfo::from_bytes(&acc.data).expect("Failed to deserialize NodeInfo");
            node_info.status != NodeStatus::Active
        } else {
            true
        };

        if needs_registration {
            let confidential_node_owner = self.confidential_node_owner.insecure_clone();
            let confidential_node_pubkey = self.confidential_node.pubkey();
            let result = self.register_node(
                &confidential_node_owner,
                &confidential_node_pubkey,
                dac_client::NodeType::Confidential,
            );
            assert!(
                result.is_ok(),
                "Failed to register confidential node for agent validation"
            );
        }

        if needs_claim {
            let confidential_node = self.confidential_node.insecure_clone();
            let tee_signing_pubkey = self.tee_signing_keypair.pubkey();
            use crate::setup::test_data::DEFAULT_CODE_MEASUREMENT;
            let result = self.claim_confidential_node(
                &confidential_node,
                DEFAULT_CODE_MEASUREMENT,
                tee_signing_pubkey,
            );
            assert!(
                result.is_ok(),
                "Failed to claim confidential node for agent validation"
            );
        }

        let result = self.validate_agent(&self.confidential_node.insecure_clone(), agent_slot_id);
        assert!(result.is_ok(), "Failed to validate agent");
        self
    }

    pub fn with_create_goal(mut self, is_confidential: bool) -> Self {
        let goal_owner = self.agent_owner.insecure_clone();
        let result = self.create_goal(&goal_owner, true, is_confidential);
        assert!(result.is_ok(), "Failed to create goal");
        self
    }

    pub fn with_set_goal(mut self, goal_slot_id: u64, agent_slot_id: u64) -> Self {
        let goal_owner = self.agent_owner.insecure_clone();
        let result = self.set_goal(
            &goal_owner,
            goal_slot_id,
            crate::setup::test_data::DEFAULT_GOAL_SPECIFICATION_CID.to_string(),
            10,
            agent_slot_id,
            0,
            DEFAULT_INITIAL_DEPOSIT,
        );
        assert!(result.is_ok(), "Failed to set goal");
        self
    }

    pub fn with_contribute_to_goal(mut self, goal_slot_id: u64, deposit_amount: u64) -> Self {
        let contributor = self.contributor.insecure_clone();
        let result = self.contribute_to_goal(&contributor, goal_slot_id, deposit_amount);
        assert!(result.is_ok(), "Failed to contribute to goal");
        self
    }

    pub fn with_withdraw_from_goal(
        mut self,
        goal_slot_id: u64,
        contributor: &Keypair,
        shares_to_burn: u64,
    ) -> Self {
        let result = self.withdraw_from_goal(contributor, goal_slot_id, shares_to_burn);
        assert!(result.is_ok(), "Failed to withdraw from goal");
        self
    }
}
