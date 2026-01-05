use dac_client::dac::instructions::{
    ClaimComputeNodeBuilder, ClaimValidatorNodeBuilder, CreateAgentBuilder,
    InitializeNetworkBuilder, RegisterNodeBuilder, ValidateComputeNodeBuilder,
};
use dac_client::dac::types::{CodeMeasurement, NodeType};
use litesvm::types::TransactionResult;
use solana_sdk::message::Instruction;
use solana_sdk::{
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer as SolanaSigner},
};
use std::str::FromStr;

use crate::setup::{TestFixture, Accounts};
use utils::Utils;

pub trait Instructions {
    fn initialize_network(
        &mut self,
        authority: &Keypair,
        network_config: &Pubkey,
        cid_config: String,
        allocate_goals: u64,
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
        remaining_accounts: &[AccountMeta],
    ) -> TransactionResult;
    fn register_node(
        &mut self,
        owner: &Keypair,
        node_pubkey: &Pubkey,
        node_type: NodeType,
    ) -> TransactionResult;

    fn claim_compute_node(
        &mut self,
        compute_node: &Keypair,
        node_info_cid: String,
    ) -> TransactionResult;

    fn claim_validator_node(
        &mut self,
        validator_node: &Keypair,
        code_measurement: [u8; 32],
        tee_signing_pubkey: Pubkey,
    ) -> TransactionResult;

    fn validate_compute_node(
        &mut self,
        validator_node: &Keypair,
        compute_node_pubkey: &Pubkey,
        ed25519_ix: &Instruction,
    ) -> TransactionResult;

    fn create_agent(
        &mut self,
        agent_owner: &Keypair,
        agent_config_cid: String,
    ) -> TransactionResult;
}

impl Instructions for TestFixture {
    fn initialize_network(
        &mut self,
        authority: &Keypair,
        network_config: &Pubkey,
        cid_config: String,
        allocate_goals: u64,
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
        remaining_accounts: &[AccountMeta],
    ) -> TransactionResult {
        let authority_pubkey = authority.pubkey();

        let mut builder = InitializeNetworkBuilder::new();
        builder
            .authority(authority_pubkey)
            .network_config(*network_config)
            .cid_config(cid_config)
            .allocate_goals(allocate_goals)
            .allocate_tasks(allocate_tasks)
            .approved_code_measurements(approved_code_measurements);

        if !remaining_accounts.is_empty() {
            builder.add_remaining_accounts(remaining_accounts);
        }

        self.svm
            .send_tx(&[builder.instruction()], &authority_pubkey, &[authority])
    }

    fn register_node(
        &mut self,
        owner: &Keypair,
        node_pubkey: &Pubkey,
        node_type: NodeType,
    ) -> TransactionResult {
        let owner_pubkey = owner.pubkey();
        let network_config_pda = self.find_network_config_pda(&self.authority.pubkey()).0;
        let (node_info_pda, _) = self.find_node_info_pda(node_pubkey);
        let (node_treasury_pda, _) = self.find_node_treasury_pda(&node_info_pda);

        let mut builder = RegisterNodeBuilder::new();
        builder
            .owner(owner_pubkey)
            .network_config(network_config_pda)
            .node_info(node_info_pda)
            .node_treasury(node_treasury_pda)
            .system_program(
                solana_sdk::pubkey::Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            )
            .node_pubkey(*node_pubkey)
            .node_type(node_type);

        self.svm
            .send_tx(&[builder.instruction()], &owner_pubkey, &[owner])
    }

    fn claim_compute_node(
        &mut self,
        compute_node: &Keypair,
        node_info_cid: String,
    ) -> TransactionResult {
        let compute_node_pubkey = compute_node.pubkey();
        let network_config_pda = self.find_network_config_pda(&self.authority.pubkey()).0;
        let (node_info_pda, _) = self.find_node_info_pda(&compute_node_pubkey);

        let mut builder = ClaimComputeNodeBuilder::new();
        builder
            .compute_node(compute_node_pubkey)
            .network_config(network_config_pda)
            .node_info(node_info_pda)
            .node_info_cid(node_info_cid);

        self.svm.send_tx(
            &[builder.instruction()],
            &compute_node_pubkey,
            &[compute_node],
        )
    }

    fn claim_validator_node(
        &mut self,
        validator_node: &Keypair,
        code_measurement: [u8; 32],
        tee_signing_pubkey: Pubkey,
    ) -> TransactionResult {
        let validator_node_pubkey = validator_node.pubkey();
        let network_config_pda = self.find_network_config_pda(&self.authority.pubkey()).0;
        let (node_info_pda, _) = self.find_node_info_pda(&validator_node_pubkey);

        let mut builder = ClaimValidatorNodeBuilder::new();
        builder
            .validator_node(validator_node_pubkey)
            .network_config(network_config_pda)
            .node_info(node_info_pda)
            .code_measurement(code_measurement)
            .tee_signing_pubkey(tee_signing_pubkey);

        self.svm.send_tx(
            &[builder.instruction()],
            &validator_node_pubkey,
            &[validator_node],
        )
    }

    fn validate_compute_node(
        &mut self,
        validator_node: &Keypair,
        compute_node_pubkey: &Pubkey,
        ed25519_ix: &Instruction,
    ) -> TransactionResult {
        let validator_node_pubkey = validator_node.pubkey();
        let network_config_pda = self.find_network_config_pda(&self.authority.pubkey()).0;
        let (validator_node_info_pda, _) = self.find_node_info_pda(&validator_node_pubkey);
        let (compute_node_info_pda, _) = self.find_node_info_pda(compute_node_pubkey);

        let mut builder = ValidateComputeNodeBuilder::new();
        builder
            .validator_node_pubkey(validator_node_pubkey)
            .network_config(network_config_pda)
            .validator_node_info(validator_node_info_pda)
            .compute_node_info(compute_node_info_pda)
            .instruction_sysvar(solana_sdk::sysvar::instructions::id());

        let validate_ix = builder.instruction();

        self.svm.send_tx(
            &[ed25519_ix.clone(), validate_ix],
            &validator_node_pubkey,
            &[validator_node],
        )
    }

    fn create_agent(
        &mut self,
        agent_owner: &Keypair,
        agent_config_cid: String,
    ) -> TransactionResult {
        let agent_owner_pubkey = agent_owner.pubkey();
        let network_config_pda = self.find_network_config_pda(&self.authority.pubkey()).0;
        let network_config = self.get_network_config(&self.authority.pubkey());
        let agent_slot_id = network_config.agent_count;
        let (agent_pda, _) = self.find_agent_pda(&network_config_pda, agent_slot_id);

        let mut builder = CreateAgentBuilder::new();
        builder
            .agent_owner(agent_owner_pubkey)
            .network_config(network_config_pda)
            .agent(agent_pda)
            .agent_config_cid(agent_config_cid);

        self.svm
            .send_tx(&[builder.instruction()], &agent_owner_pubkey, &[agent_owner])
    }
}
