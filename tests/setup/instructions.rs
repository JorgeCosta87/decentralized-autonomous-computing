use dac_client::instructions::{
    ActivateNodeBuilder, ClaimConfidentialNodeBuilder, ClaimPublicNodeBuilder, ClaimTaskBuilder,
    ContributeToSessionBuilder, CreateAgentBuilder, CreateSessionBuilder, InitializeNetworkBuilder,
    RegisterNodeBuilder, SetSessionBuilder, SubmitConfidentialTaskValidationBuilder,
    SubmitPublicTaskValidationBuilder, SubmitTaskResultBuilder, UpdateNetworkConfigBuilder,
    ValidateAgentBuilder, ValidatePublicNodeBuilder, WithdrawFromSessionBuilder,
};
use dac_client::types::TaskType;
use dac_client::types::{CodeMeasurement, NodeType};
use litesvm::types::TransactionResult;
use solana_sdk::message::Instruction;
use solana_sdk::{
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer as SolanaSigner},
};
use std::str::FromStr;

use crate::setup::{Accounts, TestFixture};
use utils::Utils;

pub trait Instructions {
    fn initialize_network(
        &mut self,
        authority: &Keypair,
        network_config: &Pubkey,
        cid_config: String,
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
        required_validations: u32,
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

    fn claim_confidential_node(
        &mut self,
        confidential_node: &Keypair,
        code_measurement: [u8; 32],
        tee_signing_pubkey: Pubkey,
    ) -> TransactionResult;

    fn validate_public_node(
        &mut self,
        node: &Keypair,
        node_to_validate_pubkey: &Pubkey,
        approved: bool,
    ) -> TransactionResult;

    fn activate_node(&mut self, authority: &Keypair, node_pubkey: &Pubkey) -> TransactionResult;

    fn validate_agent(
        &mut self,
        node_validating: &Keypair,
        agent_slot_id: u64,
    ) -> TransactionResult;

    fn create_agent(
        &mut self,
        agent_owner: &Keypair,
        agent_config_cid: String,
    ) -> TransactionResult;

    fn create_session(
        &mut self,
        payer: &Keypair,
        is_owned: bool,
        is_confidential: bool,
    ) -> TransactionResult;

    fn set_session(
        &mut self,
        session_owner: &Keypair,
        session_slot_id: u64,
        specification_cid: String,
        max_iterations: u64,
        agent_slot_id: u64,
        task_slot_id: u64,
        initial_deposit: u64,
        compute_node: Pubkey,
        task_type: TaskType,
    ) -> TransactionResult;

    fn contribute_to_session(
        &mut self,
        contributor: &Keypair,
        session_slot_id: u64,
        deposit_amount: u64,
    ) -> TransactionResult;

    fn withdraw_from_session(
        &mut self,
        contributor: &Keypair,
        session_slot_id: u64,
        shares_to_burn: u64,
    ) -> TransactionResult;

    fn claim_task(
        &mut self,
        compute_node: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        max_task_cost: u64,
        max_call_count: u64,
    ) -> TransactionResult;

    fn submit_task_result(
        &mut self,
        compute_node: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        input_cid: String,
        output_cid: String,
        state_cid: Option<String>,
        call_count: u64,
    ) -> TransactionResult;

    fn submit_confidential_task_validation(
        &mut self,
        node_validating: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        compute_node_pubkey: &Pubkey,
        ed25519_ix: &Instruction,
    ) -> TransactionResult;

    fn submit_public_task_validation(
        &mut self,
        node_validating: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        compute_node_pubkey: &Pubkey,
        payment_amount: u64,
        approved: bool,
        session_completed: bool,
    ) -> TransactionResult;

    fn update_network_config(
        &mut self,
        authority: &Keypair,
        cid_config: Option<String>,
        new_code_measurement: Option<CodeMeasurement>,
    ) -> TransactionResult;
}

impl Instructions for TestFixture {
    fn initialize_network(
        &mut self,
        authority: &Keypair,
        network_config: &Pubkey,
        cid_config: String,
        allocate_tasks: u64,
        approved_code_measurements: Vec<CodeMeasurement>,
        required_validations: u32,
        remaining_accounts: &[AccountMeta],
    ) -> TransactionResult {
        let authority_pubkey = authority.pubkey();

        let mut builder = InitializeNetworkBuilder::new();
        builder
            .authority(authority_pubkey)
            .network_config(*network_config)
            .cid_config(cid_config)
            .allocate_tasks(allocate_tasks)
            .approved_code_measurements(approved_code_measurements)
            .required_validations(required_validations);

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
        let network_config_pda = self.find_network_config_pda().0;
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
        let network_config_pda = self.find_network_config_pda().0;
        let (node_info_pda, _) = self.find_node_info_pda(&compute_node_pubkey);

        let mut builder = ClaimPublicNodeBuilder::new();
        builder
            .node(compute_node_pubkey)
            .network_config(network_config_pda)
            .node_info(node_info_pda)
            .node_info_cid(node_info_cid);

        self.svm.send_tx(
            &[builder.instruction()],
            &compute_node_pubkey,
            &[compute_node],
        )
    }

    fn claim_confidential_node(
        &mut self,
        confidential_node: &Keypair,
        code_measurement: [u8; 32],
        tee_signing_pubkey: Pubkey,
    ) -> TransactionResult {
        let confidential_node_pubkey = confidential_node.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (node_info_pda, _) = self.find_node_info_pda(&confidential_node_pubkey);

        let mut builder = ClaimConfidentialNodeBuilder::new();
        builder
            .confidential_node(confidential_node_pubkey)
            .network_config(network_config_pda)
            .node_info(node_info_pda)
            .code_measurement(code_measurement)
            .tee_signing_pubkey(tee_signing_pubkey);

        self.svm.send_tx(
            &[builder.instruction()],
            &confidential_node_pubkey,
            &[confidential_node],
        )
    }

    fn validate_public_node(
        &mut self,
        node: &Keypair,
        node_to_validate_pubkey: &Pubkey,
        approved: bool,
    ) -> TransactionResult {
        let node_pubkey = node.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (node_info_pda, _) = self.find_node_info_pda(&node_pubkey);
        let (node_to_validate_info_pda, _) = self.find_node_info_pda(node_to_validate_pubkey);

        let mut builder = ValidatePublicNodeBuilder::new();
        builder
            .node_validating(node_pubkey)
            .network_config(network_config_pda)
            .node_validating_info(node_info_pda)
            .node_info(node_to_validate_info_pda)
            .approved(approved);

        let validate_ix = builder.instruction();

        self.svm.send_tx(&[validate_ix], &node_pubkey, &[node])
    }

    fn validate_agent(&mut self, node: &Keypair, agent_slot_id: u64) -> TransactionResult {
        let node_pubkey = node.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (agent_pda, _) = self.find_agent_pda(&network_config_pda, agent_slot_id);
        let (node_info_pda, _) = self.find_node_info_pda(&node_pubkey);

        let mut builder = ValidateAgentBuilder::new();
        builder
            .node(node_pubkey)
            .agent(agent_pda)
            .node_info(node_info_pda)
            .network_config(network_config_pda);

        self.svm
            .send_tx(&[builder.instruction()], &node_pubkey, &[node])
    }

    fn activate_node(&mut self, authority: &Keypair, node_pubkey: &Pubkey) -> TransactionResult {
        let authority_pubkey = authority.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (node_info_pda, _) = self.find_node_info_pda(node_pubkey);

        let mut builder = ActivateNodeBuilder::new();
        builder
            .authority(authority_pubkey)
            .network_config(network_config_pda)
            .node_info(node_info_pda);

        self.svm
            .send_tx(&[builder.instruction()], &authority_pubkey, &[authority])
    }

    fn create_agent(
        &mut self,
        agent_owner: &Keypair,
        agent_config_cid: String,
    ) -> TransactionResult {
        let agent_owner_pubkey = agent_owner.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let network_config = self.get_network_config();
        let agent_slot_id = network_config.agent_count;
        let (agent_pda, _) = self.find_agent_pda(&network_config_pda, agent_slot_id);

        let mut builder = CreateAgentBuilder::new();
        builder
            .agent_owner(agent_owner_pubkey)
            .network_config(network_config_pda)
            .agent(agent_pda)
            .agent_config_cid(agent_config_cid);

        self.svm.send_tx(
            &[builder.instruction()],
            &agent_owner_pubkey,
            &[agent_owner],
        )
    }

    fn create_session(
        &mut self,
        owner: &Keypair,
        is_owned: bool,
        is_confidential: bool,
    ) -> TransactionResult {
        let owner_pubkey = owner.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let network_config = self.get_network_config();
        let session_slot_id = network_config.session_count;
        let task_slot_id = network_config.task_count;
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);

        let mut builder = CreateSessionBuilder::new();
        builder
            .payer(owner_pubkey)
            .owner(owner_pubkey)
            .network_config(network_config_pda)
            .session(session_pda)
            .task(task_pda)
            .is_owned(is_owned)
            .is_confidential(is_confidential);

        self.svm
            .send_tx(&[builder.instruction()], &owner_pubkey, &[owner])
    }

    fn set_session(
        &mut self,
        session_owner: &Keypair,
        session_slot_id: u64,
        specification_cid: String,
        max_iterations: u64,
        agent_slot_id: u64,
        task_slot_id: u64,
        initial_deposit: u64,
        compute_node: Pubkey,
        task_type: TaskType,
    ) -> TransactionResult {
        let session_owner_pubkey = session_owner.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);
        let (vault_pda, _) = self.find_session_vault_pda(&session_pda);
        let (owner_contribution_pda, _) =
            self.find_contribution_pda(&session_pda, &session_owner_pubkey);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (agent_pda, _) = self.find_agent_pda(&network_config_pda, agent_slot_id);

        let mut builder = SetSessionBuilder::new();
        builder
            .owner(session_owner_pubkey)
            .session(session_pda)
            .vault(vault_pda)
            .owner_contribution(owner_contribution_pda)
            .task(task_pda)
            .agent(agent_pda)
            .network_config(network_config_pda)
            .specification_cid(specification_cid)
            .max_iterations(max_iterations)
            .initial_deposit(initial_deposit)
            .compute_node(compute_node)
            .task_type(task_type);

        self.svm
            .send_tx(&[builder.instruction()], &session_owner_pubkey, &[session_owner])
    }

    fn contribute_to_session(
        &mut self,
        contributor: &Keypair,
        session_slot_id: u64,
        deposit_amount: u64,
    ) -> TransactionResult {
        let contributor_pubkey = contributor.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);
        let (vault_pda, _) = self.find_session_vault_pda(&session_pda);
        let (contribution_pda, _) = self.find_contribution_pda(&session_pda, &contributor_pubkey);

        let mut builder = ContributeToSessionBuilder::new();
        builder
            .contributor(contributor_pubkey)
            .session(session_pda)
            .vault(vault_pda)
            .contribution(contribution_pda)
            .network_config(network_config_pda)
            .deposit_amount(deposit_amount);

        self.svm.send_tx(
            &[builder.instruction()],
            &contributor_pubkey,
            &[contributor],
        )
    }

    fn withdraw_from_session(
        &mut self,
        contributor: &Keypair,
        session_slot_id: u64,
        shares_to_burn: u64,
    ) -> TransactionResult {
        let contributor_pubkey = contributor.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);
        let (vault_pda, _) = self.find_session_vault_pda(&session_pda);
        let (contribution_pda, _) = self.find_contribution_pda(&session_pda, &contributor_pubkey);

        let mut builder = WithdrawFromSessionBuilder::new();
        builder
            .contributor(contributor_pubkey)
            .session(session_pda)
            .vault(vault_pda)
            .contribution(contribution_pda)
            .network_config(network_config_pda)
            .shares_to_burn(shares_to_burn);

        self.svm.send_tx(
            &[builder.instruction()],
            &contributor_pubkey,
            &[contributor],
        )
    }

    fn claim_task(
        &mut self,
        compute_node: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        max_task_cost: u64,
        max_call_count: u64,
    ) -> TransactionResult {
        let compute_node_pubkey = compute_node.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (vault_pda, _) = self.find_session_vault_pda(&session_pda);

        let mut builder = ClaimTaskBuilder::new();
        builder
            .compute_node(compute_node_pubkey)
            .task(task_pda)
            .session(session_pda)
            .vault(vault_pda)
            .network_config(network_config_pda)
            .max_task_cost(max_task_cost)
            .max_call_count(max_call_count);

        self.svm.send_tx(
            &[builder.instruction()],
            &compute_node_pubkey,
            &[compute_node],
        )
    }

    fn submit_task_result(
        &mut self,
        compute_node: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        input_cid: String,
        output_cid: String,
        state_cid: Option<String>,
        call_count: u64,
    ) -> TransactionResult {
        let compute_node_pubkey = compute_node.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);

        let mut builder = SubmitTaskResultBuilder::new();
        builder
            .compute_node(compute_node_pubkey)
            .task(task_pda)
            .session(session_pda)
            .network_config(network_config_pda)
            .input_cid(input_cid)
            .output_cid(output_cid);

        if let Some(state_cid_str) = state_cid {
            builder.state_cid(state_cid_str);
        }

        builder.call_count(call_count);

        self.svm.send_tx(
            &[builder.instruction()],
            &compute_node_pubkey,
            &[compute_node],
        )
    }

    fn submit_confidential_task_validation(
        &mut self,
        node_validating: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        compute_node_pubkey: &Pubkey,
        ed25519_ix: &Instruction,
    ) -> TransactionResult {
        let validator_pubkey = node_validating.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (vault_pda, _) = self.find_session_vault_pda(&session_pda);
        let (compute_node_info_pda, _) = self.find_node_info_pda(compute_node_pubkey);
        let (node_treasury_pda, _) = self.find_node_treasury_pda(&compute_node_info_pda);
        let (validator_node_info_pda, _) = self.find_node_info_pda(&validator_pubkey);

        let mut builder = SubmitConfidentialTaskValidationBuilder::new();
        builder
            .node_validating(validator_pubkey)
            .session(session_pda)
            .vault(vault_pda)
            .task(task_pda)
            .node_info(compute_node_info_pda)
            .node_treasury(node_treasury_pda)
            .validator_node_info(validator_node_info_pda)
            .network_config(network_config_pda)
            .instruction_sysvar(solana_sdk::sysvar::instructions::id());

        let validate_ix = builder.instruction();

        self.svm.send_tx(
            &[ed25519_ix.clone(), validate_ix],
            &validator_pubkey,
            &[node_validating],
        )
    }

    fn submit_public_task_validation(
        &mut self,
        node_validating: &Keypair,
        session_slot_id: u64,
        task_slot_id: u64,
        compute_node_pubkey: &Pubkey,
        payment_amount: u64,
        approved: bool,
        session_completed: bool,
    ) -> TransactionResult {
        let node_validating_pubkey = node_validating.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (session_pda, _) = self.find_session_pda(&network_config_pda, session_slot_id);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (vault_pda, _) = self.find_session_vault_pda(&session_pda);
        let (compute_node_info_pda, _) = self.find_node_info_pda(compute_node_pubkey);
        let (node_treasury_pda, _) = self.find_node_treasury_pda(&compute_node_info_pda);
        let (node_validating_info_pda, _) = self.find_node_info_pda(&node_validating_pubkey);

        let mut builder = SubmitPublicTaskValidationBuilder::new();
        builder
            .node_validating(node_validating_pubkey)
            .session(session_pda)
            .vault(vault_pda)
            .task(task_pda)
            .node_info(compute_node_info_pda)
            .node_treasury(node_treasury_pda)
            .validator_node_info(node_validating_info_pda)
            .network_config(network_config_pda)
            .instruction_sysvar(solana_sdk::sysvar::instructions::id())
            .payment_amount(payment_amount)
            .approved(approved)
            .goal_completed(session_completed);

        let validate_ix = builder.instruction();

        self.svm
            .send_tx(&[validate_ix], &node_validating_pubkey, &[node_validating])
    }

    fn update_network_config(
        &mut self,
        authority: &Keypair,
        cid_config: Option<String>,
        new_code_measurement: Option<CodeMeasurement>,
    ) -> TransactionResult {
        let authority_pubkey = authority.pubkey();
        let network_config_pda = self.find_network_config_pda().0;

        let mut builder = UpdateNetworkConfigBuilder::new();
        builder
            .authority(authority_pubkey)
            .network_config(network_config_pda);

        if let Some(cid) = cid_config {
            builder.cid_config(cid);
        }

        if let Some(measurement) = new_code_measurement {
            builder.new_code_measurement(measurement);
        }

        self.svm
            .send_tx(&[builder.instruction()], &authority_pubkey, &[authority])
    }
}
