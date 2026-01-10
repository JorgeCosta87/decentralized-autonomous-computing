use dac_client::instructions::{
    ClaimComputeNodeBuilder, ClaimTaskBuilder, ClaimValidatorNodeBuilder, ContributeToGoalBuilder,
    CreateAgentBuilder, CreateGoalBuilder, InitializeNetworkBuilder, RegisterNodeBuilder,
    SetGoalBuilder, SubmitTaskResultBuilder, SubmitTaskValidationBuilder, UpdateNetworkConfigBuilder,
    ValidateAgentBuilder, ValidateComputeNodeBuilder, WithdrawFromGoalBuilder,
};
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

    fn validate_agent(&mut self, validator: &Keypair, agent_slot_id: u64) -> TransactionResult;

    fn create_agent(
        &mut self,
        agent_owner: &Keypair,
        agent_config_cid: String,
    ) -> TransactionResult;

    fn create_goal(&mut self, payer: &Keypair, is_public: bool) -> TransactionResult;

    fn set_goal(
        &mut self,
        goal_owner: &Keypair,
        goal_slot_id: u64,
        specification_cid: String,
        max_iterations: u64,
        agent_slot_id: u64,
        task_slot_id: u64,
        initial_deposit: u64,
    ) -> TransactionResult;

    fn contribute_to_goal(
        &mut self,
        contributor: &Keypair,
        goal_slot_id: u64,
        deposit_amount: u64,
    ) -> TransactionResult;

    fn withdraw_from_goal(
        &mut self,
        contributor: &Keypair,
        goal_slot_id: u64,
        shares_to_burn: u64,
    ) -> TransactionResult;

    fn claim_task(
        &mut self,
        compute_node: &Keypair,
        goal_slot_id: u64,
        task_slot_id: u64,
        max_task_cost: u64,
    ) -> TransactionResult;

    fn submit_task_result(
        &mut self,
        compute_node: &Keypair,
        task_slot_id: u64,
        input_cid: String,
        output_cid: String,
    ) -> TransactionResult;

    fn submit_task_validation(
        &mut self,
        validator: &Keypair,
        goal_slot_id: u64,
        task_slot_id: u64,
        compute_node_pubkey: &Pubkey,
        ed25519_ix: &Instruction,
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
        let network_config_pda = self.find_network_config_pda().0;
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
        let network_config_pda = self.find_network_config_pda().0;
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

    fn validate_agent(&mut self, validator: &Keypair, agent_slot_id: u64) -> TransactionResult {
        let validator_pubkey = validator.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (agent_pda, _) = self.find_agent_pda(&network_config_pda, agent_slot_id);

        let mut builder = ValidateAgentBuilder::new();
        builder
            .validator(validator_pubkey)
            .agent(agent_pda)
            .network_config(network_config_pda);

        self.svm
            .send_tx(&[builder.instruction()], &validator_pubkey, &[validator])
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

    fn create_goal(&mut self, owner: &Keypair, is_public: bool) -> TransactionResult {
        let owner_pubkey = owner.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let network_config = self.get_network_config();
        let goal_slot_id = network_config.goal_count;
        let (goal_pda, _) = self.find_goal_pda(&network_config_pda, goal_slot_id);

        let mut builder = CreateGoalBuilder::new();
        builder
            .payer(owner_pubkey)
            .owner(owner_pubkey)
            .network_config(network_config_pda)
            .goal(goal_pda)
            .is_public(is_public);

        self.svm
            .send_tx(&[builder.instruction()], &owner_pubkey, &[owner])
    }

    fn set_goal(
        &mut self,
        goal_owner: &Keypair,
        goal_slot_id: u64,
        specification_cid: String,
        max_iterations: u64,
        agent_slot_id: u64,
        task_slot_id: u64,
        initial_deposit: u64,
    ) -> TransactionResult {
        let goal_owner_pubkey = goal_owner.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (goal_pda, _) = self.find_goal_pda(&network_config_pda, goal_slot_id);
        let (vault_pda, _) = self.find_goal_vault_pda(&goal_pda);
        let (owner_contribution_pda, _) = self.find_contribution_pda(&goal_pda, &goal_owner_pubkey);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (agent_pda, _) = self.find_agent_pda(&network_config_pda, agent_slot_id);

        let mut builder = SetGoalBuilder::new();
        builder
            .owner(goal_owner_pubkey)
            .goal(goal_pda)
            .vault(vault_pda)
            .owner_contribution(owner_contribution_pda)
            .task(task_pda)
            .agent(agent_pda)
            .network_config(network_config_pda)
            .specification_cid(specification_cid)
            .max_iterations(max_iterations)
            .initial_deposit(initial_deposit);

        self.svm
            .send_tx(&[builder.instruction()], &goal_owner_pubkey, &[goal_owner])
    }

    fn contribute_to_goal(
        &mut self,
        contributor: &Keypair,
        goal_slot_id: u64,
        deposit_amount: u64,
    ) -> TransactionResult {
        let contributor_pubkey = contributor.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (goal_pda, _) = self.find_goal_pda(&network_config_pda, goal_slot_id);
        let (vault_pda, _) = self.find_goal_vault_pda(&goal_pda);
        let (contribution_pda, _) = self.find_contribution_pda(&goal_pda, &contributor_pubkey);

        let mut builder = ContributeToGoalBuilder::new();
        builder
            .contributor(contributor_pubkey)
            .goal(goal_pda)
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

    fn withdraw_from_goal(
        &mut self,
        contributor: &Keypair,
        goal_slot_id: u64,
        shares_to_burn: u64,
    ) -> TransactionResult {
        let contributor_pubkey = contributor.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (goal_pda, _) = self.find_goal_pda(&network_config_pda, goal_slot_id);
        let (vault_pda, _) = self.find_goal_vault_pda(&goal_pda);
        let (contribution_pda, _) = self.find_contribution_pda(&goal_pda, &contributor_pubkey);

        let mut builder = WithdrawFromGoalBuilder::new();
        builder
            .contributor(contributor_pubkey)
            .goal(goal_pda)
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
        goal_slot_id: u64,
        task_slot_id: u64,
        max_task_cost: u64,
    ) -> TransactionResult {
        let compute_node_pubkey = compute_node.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (goal_pda, _) = self.find_goal_pda(&network_config_pda, goal_slot_id);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (vault_pda, _) = self.find_goal_vault_pda(&goal_pda);
        let (compute_node_info_pda, _) = self.find_node_info_pda(&compute_node_pubkey);

        let mut builder = ClaimTaskBuilder::new();
        builder
            .compute_node(compute_node_pubkey)
            .task(task_pda)
            .goal(goal_pda)
            .vault(vault_pda)
            .compute_node_info(compute_node_info_pda)
            .network_config(network_config_pda)
            .max_task_cost(max_task_cost);

        self.svm.send_tx(
            &[builder.instruction()],
            &compute_node_pubkey,
            &[compute_node],
        )
    }

    fn submit_task_result(
        &mut self,
        compute_node: &Keypair,
        task_slot_id: u64,
        input_cid: String,
        output_cid: String,
    ) -> TransactionResult {
        let compute_node_pubkey = compute_node.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);

        let mut builder = SubmitTaskResultBuilder::new();
        builder
            .compute_node(compute_node_pubkey)
            .task(task_pda)
            .network_config(network_config_pda)
            .input_cid(input_cid)
            .output_cid(output_cid);

        self.svm.send_tx(
            &[builder.instruction()],
            &compute_node_pubkey,
            &[compute_node],
        )
    }

    fn submit_task_validation(
        &mut self,
        validator: &Keypair,
        goal_slot_id: u64,
        task_slot_id: u64,
        compute_node_pubkey: &Pubkey,
        ed25519_ix: &Instruction,
    ) -> TransactionResult {
        let validator_pubkey = validator.pubkey();
        let network_config_pda = self.find_network_config_pda().0;
        let (goal_pda, _) = self.find_goal_pda(&network_config_pda, goal_slot_id);
        let (task_pda, _) = self.find_task_pda(&network_config_pda, task_slot_id);
        let (vault_pda, _) = self.find_goal_vault_pda(&goal_pda);
        let (compute_node_info_pda, _) = self.find_node_info_pda(compute_node_pubkey);
        let (node_treasury_pda, _) = self.find_node_treasury_pda(&compute_node_info_pda);
        let (validator_node_info_pda, _) = self.find_node_info_pda(&validator_pubkey);

        let mut builder = SubmitTaskValidationBuilder::new();
        builder
            .validator(validator_pubkey)
            .goal(goal_pda)
            .vault(vault_pda)
            .task(task_pda)
            .compute_node_info(compute_node_info_pda)
            .node_treasury(node_treasury_pda)
            .validator_node_info(validator_node_info_pda)
            .network_config(network_config_pda)
            .instruction_sysvar(solana_sdk::sysvar::instructions::id());

        let validate_ix = builder.instruction();

        self.svm.send_tx(
            &[ed25519_ix.clone(), validate_ix],
            &validator_pubkey,
            &[validator],
        )
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
