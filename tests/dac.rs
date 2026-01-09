use crate::setup::test_data::*;
use crate::setup::test_data::{
    DEFAULT_CONTRIBUTION_AMOUNT, DEFAULT_GOAL_SPECIFICATION_CID, DEFAULT_INITIAL_DEPOSIT,
};
use crate::setup::Accounts;
use crate::setup::Helpers;
use crate::setup::Instructions;
use crate::setup::TestFixture;
use dac_client::{ActionType, AgentStatus, GoalStatus, NodeStatus, NodeType, TaskStatus};
use sha2::{Digest, Sha256};
use solana_sdk::signature::Signer;
use utils::Utils;

mod setup;

#[test]
fn test_initialize_network_without_remaining_accounts() {
    let mut fixt = TestFixture::new();
    let network_config_pda = fixt.find_network_config_pda().0;

    let allocate_goals = 0;
    let allocate_tasks = 0;

    let result = fixt.initialize_network(
        &fixt.authority.insecure_clone(),
        &network_config_pda,
        DEFAULT_CID_CONFIG.to_string(),
        allocate_goals,
        allocate_tasks,
        DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec(),
        &[],
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config = fixt.get_network_config();

            assert_eq!(network_config.authority, fixt.authority.pubkey());
            assert_eq!(network_config.cid_config, DEFAULT_CID_CONFIG.to_string());
            assert_eq!(network_config.genesis_hash, compute_genesis_hash());
            assert_eq!(network_config.agent_count, 0);
            assert_eq!(network_config.goal_count, allocate_goals);
            assert_eq!(network_config.task_count, allocate_tasks);
            assert_eq!(
                network_config.approved_code_measurements,
                DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec()
            );
        }
        Err(e) => panic!("Failed to initialize network: {:#?}", e),
    }
}

#[test]
fn test_initialize_network_with_remaining_accounts() {
    let mut fixt = TestFixture::new();
    let network_config_pda = fixt.find_network_config_pda().0;

    let allocate_goals = 2;
    let allocate_tasks = 20;
    let remaining_accounts = fixt.create_remaining_accounts_for_initialize(
        &network_config_pda,
        allocate_goals,
        allocate_tasks,
    );

    let result = fixt.initialize_network(
        &fixt.authority.insecure_clone(),
        &network_config_pda,
        DEFAULT_CID_CONFIG.to_string(),
        allocate_goals,
        allocate_tasks,
        DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec(),
        &remaining_accounts,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config = fixt.get_network_config();

            assert_eq!(network_config.cid_config, DEFAULT_CID_CONFIG.to_string());
            assert_eq!(network_config.genesis_hash, compute_genesis_hash());
            assert_eq!(network_config.agent_count, 0);
            assert_eq!(network_config.goal_count, allocate_goals);
            assert_eq!(network_config.task_count, allocate_tasks);
            assert_eq!(
                network_config.approved_code_measurements,
                DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec()
            );
        }
        Err(e) => panic!("Failed to initialize network: {:#?}", e),
    }
}

#[test]
fn test_register_compute_node() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let compute_node_pubkey = fixt.compute_node.pubkey();

    let result = fixt.register_node(
        &fixt.compute_node_owner.insecure_clone(),
        &compute_node_pubkey,
        NodeType::Compute,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&compute_node_pubkey);
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.owner, fixt.compute_node_owner.pubkey());
            assert_eq!(node_info.node_pubkey, compute_node_pubkey);
            assert_eq!(node_info.node_type, NodeType::Compute);
            assert_eq!(node_info.status, NodeStatus::PendingClaim);
            assert_eq!(node_info.node_info_cid, None);
            assert_eq!(network_config.compute_node_count, 0);
        }
        Err(e) => panic!("Failed to register compute node: {:#?}", e),
    }
}

#[test]
fn test_register_validator_node() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let validator_node_pubkey = fixt.validator_node.pubkey();

    let result = fixt.register_node(
        &fixt.validator_node_owner.insecure_clone(),
        &validator_node_pubkey,
        NodeType::Validator,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&validator_node_pubkey);
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.owner, fixt.validator_node_owner.pubkey());
            assert_eq!(node_info.node_pubkey, validator_node_pubkey);
            assert_eq!(node_info.node_type, NodeType::Validator);
            assert_eq!(node_info.status, NodeStatus::PendingClaim);
            assert_eq!(node_info.code_measurement, None);
            assert_eq!(network_config.validator_node_count, 0);
        }
        Err(e) => panic!("Failed to register validator node: {:#?}", e),
    }
}

#[test]
fn test_claim_compute_node() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_compute_node();

    let result = fixt.claim_compute_node(
        &fixt.compute_node.insecure_clone(),
        DEFAULT_NODE_INFO_CID.to_string(),
    );
    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&fixt.compute_node.pubkey());
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.status, NodeStatus::AwaitingValidation);
            assert_eq!(
                node_info.node_info_cid,
                Some(DEFAULT_NODE_INFO_CID.to_string())
            );
            assert_eq!(network_config.compute_node_count, 0);
        }
        Err(e) => panic!("Failed to claim compute node: {:#?}", e),
    }
}

#[test]
fn test_claim_validator_node() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node();

    let result = fixt.claim_validator_node(
        &fixt.validator_node.insecure_clone(),
        DEFAULT_CODE_MEASUREMENT,
        fixt.tee_signing_keypair.pubkey(),
    );
    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&fixt.validator_node.pubkey());
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.status, NodeStatus::Active);
            assert_eq!(node_info.code_measurement, Some(DEFAULT_CODE_MEASUREMENT));
            assert_eq!(
                node_info.tee_signing_pubkey,
                Some(fixt.tee_signing_keypair.pubkey())
            );
            assert_eq!(network_config.validator_node_count, 1);
        }
        Err(e) => panic!("Failed to claim validator node: {:#?}", e),
    }
}

#[test]
fn test_validate_compute_node() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_register_compute_node()
        .with_claim_compute_node();

    let ed25519_ix = Helpers::create_ed25519_instruction_to_validate_compute_node(
        &fixt.compute_node.pubkey(),
        true,
        &fixt.tee_signing_keypair.insecure_clone(),
    );

    let result = fixt.validate_compute_node(
        &fixt.validator_node.insecure_clone(),
        &fixt.compute_node.pubkey(),
        &ed25519_ix,
    );
    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&fixt.compute_node.pubkey());
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.status, NodeStatus::Active);
            assert_eq!(node_info.total_tasks_completed, 0);
            assert_eq!(node_info.total_earned, 0);
            assert_eq!(network_config.compute_node_count, 1);
        }
        Err(e) => panic!("Failed to validate compute node: {:#?}", e),
    }
}

#[test]
fn test_validate_compute_node_not_approved() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_register_compute_node()
        .with_claim_compute_node();

    let ed25519_ix = Helpers::create_ed25519_instruction_to_validate_compute_node(
        &fixt.compute_node.pubkey(),
        false,
        &fixt.tee_signing_keypair.insecure_clone(),
    );

    let result = fixt.validate_compute_node(
        &fixt.validator_node.insecure_clone(),
        &fixt.compute_node.pubkey(),
        &ed25519_ix,
    );
    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&fixt.compute_node.pubkey());

            assert_eq!(node_info.status, NodeStatus::Rejected);
        }
        Err(e) => panic!("Failed to validate compute node not approved: {:#?}", e),
    }
}

#[test]
fn test_wrong_tee_signing_pubkey() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_register_compute_node()
        .with_claim_compute_node();

    let attacker_tee_keypair = fixt.create_keypair();

    let ed25519_ix = Helpers::create_ed25519_instruction_to_validate_compute_node(
        &fixt.compute_node.pubkey(),
        true,
        &attacker_tee_keypair,
    );

    let result = fixt.validate_compute_node(
        &fixt.validator_node.insecure_clone(),
        &fixt.compute_node.pubkey(),
        &ed25519_ix,
    );

    assert!(
        result.is_err(),
        "Should fail because of wrong TEE signing pubkey"
    );
}

#[test]
fn test_wrong_compute_node_pubkey() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_register_compute_node()
        .with_claim_compute_node();

    let diferent_compute_node = fixt.create_keypair();

    let ed25519_ix = Helpers::create_ed25519_instruction_to_validate_compute_node(
        &diferent_compute_node.pubkey(),
        true,
        &fixt.tee_signing_keypair.insecure_clone(),
    );

    let result = fixt.validate_compute_node(
        &fixt.validator_node.insecure_clone(),
        &fixt.compute_node.pubkey(),
        &ed25519_ix,
    );

    assert!(
        result.is_err(),
        "Should fail because of wrong compute node pubkey in message"
    );
}

#[test]
fn test_inactive_validator() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node();

    let ed25519_ix = Helpers::create_ed25519_instruction_to_validate_compute_node(
        &fixt.compute_node.pubkey(),
        true,
        &fixt.tee_signing_keypair.insecure_clone(),
    );

    let result = fixt.validate_compute_node(
        &fixt.validator_node.insecure_clone(),
        &fixt.compute_node.pubkey(),
        &ed25519_ix,
    );

    assert!(
        result.is_err(),
        "should fail because validator node is not active"
    );
}

#[test]
fn test_create_agent() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let agent_owner = fixt.agent_owner.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda().0;

    let result = fixt.create_agent(&agent_owner, DEFAULT_AGENT_CONFIG_CID.to_string());

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config = fixt.get_network_config();
            let agent = fixt.get_agent(&network_config_pda, 0);

            assert_eq!(agent.agent_slot_id, 0);
            assert_eq!(agent.owner, agent_owner.pubkey());
            assert_eq!(agent.agent_config_cid, DEFAULT_AGENT_CONFIG_CID.to_string());
            assert_eq!(agent.agent_memory_cid, None);
            assert_eq!(agent.status, AgentStatus::Pending);
            assert_eq!(network_config.agent_count, 1);
        }
        Err(e) => panic!("Failed to create agent: {:#?}", e),
    }
}

#[test]
fn test_create_multiple_agents() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let agent_owner = fixt.agent_owner.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda().0;

    let result1 = fixt.create_agent(&agent_owner, DEFAULT_AGENT_CONFIG_CID.to_string());
    assert!(result1.is_ok(), "Failed to create first agent");

    let result2 = fixt.create_agent(&agent_owner, "QmSecondAgentConfigCID".to_string());
    assert!(result2.is_ok(), "Failed to create second agent");

    let network_config = fixt.get_network_config();
    assert_eq!(network_config.agent_count, 2);

    let agent0 = fixt.get_agent(&network_config_pda, 0);
    assert_eq!(agent0.agent_slot_id, 0);
    assert_eq!(
        agent0.agent_config_cid,
        DEFAULT_AGENT_CONFIG_CID.to_string()
    );

    let agent1 = fixt.get_agent(&network_config_pda, 1);
    assert_eq!(agent1.agent_slot_id, 1);
    assert_eq!(
        agent1.agent_config_cid,
        "QmSecondAgentConfigCID".to_string()
    );
}

#[test]
fn test_set_goal() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_create_agent()
        .with_validated_agent(0);

    let goal_owner = fixt.create_keypair();
    let network_config_pda = fixt.find_network_config_pda().0;
    // Set goal
    let result2 = fixt.set_goal(
        &goal_owner,
        0,
        DEFAULT_GOAL_SPECIFICATION_CID.to_string(),
        10,
        0, // agent_slot_id
        0, // task_slot_id
        DEFAULT_INITIAL_DEPOSIT,
    );

    match result2 {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result2.unwrap());
            let goal = fixt.get_goal(&network_config_pda, 0);
            let (goal_pda, _) = fixt.find_goal_pda(&network_config_pda, 0);
            let owner_contribution = fixt.get_contribution(&goal_pda, &goal_owner.pubkey());
            let (task_pda, _) = fixt.find_task_pda(&network_config_pda, 0);
            let task = fixt.get_task(&network_config_pda, 0);
            let (agent_pda, _) = fixt.find_agent_pda(&network_config_pda, 0);

            // Verify goal was set
            assert_eq!(goal.owner, goal_owner.pubkey());
            assert_eq!(goal.agent, agent_pda);
            assert_eq!(
                goal.specification_cid,
                DEFAULT_GOAL_SPECIFICATION_CID.to_string()
            );
            assert_eq!(goal.max_iterations, 10);
            assert_eq!(goal.status, GoalStatus::Active);
            assert_eq!(goal.total_shares, DEFAULT_INITIAL_DEPOSIT);
            assert_eq!(goal.task, task_pda);

            // Verify owner's contribution was created
            assert_eq!(owner_contribution.goal, goal_pda);
            assert_eq!(owner_contribution.contributor, goal_owner.pubkey());
            assert_eq!(owner_contribution.shares, DEFAULT_INITIAL_DEPOSIT);
            assert_eq!(owner_contribution.refund_amount, 0);

            // Verify task was updated correctly
            let (agent_pda, _) = fixt.find_agent_pda(&network_config_pda, 0);
            assert_eq!(task.status, TaskStatus::Pending);
            assert_eq!(task.agent, agent_pda);
            assert_eq!(task.action_type, ActionType::Llm);
        }
        Err(e) => panic!("Failed to set goal: {:#?}", e),
    }
}

#[test]
fn test_contribute_to_goal() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_create_agent()
        .with_validated_agent(0);

    let contributor = fixt.create_keypair();
    let network_config_pda = fixt.find_network_config_pda().0;

    let mut fixt = fixt.with_set_goal(0, 0);

    let result = fixt.contribute_to_goal(&contributor, 0, DEFAULT_CONTRIBUTION_AMOUNT);

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let goal = fixt.get_goal(&network_config_pda, 0);
            let (goal_pda, _) = fixt.find_goal_pda(&network_config_pda, 0);
            let contribution = fixt.get_contribution(&goal_pda, &contributor.pubkey());

            assert_eq!(contribution.goal, goal_pda);
            assert_eq!(contribution.contributor, contributor.pubkey());
            assert!(contribution.shares > 0, "Contributor should have shares");
            assert_eq!(contribution.refund_amount, 0);

            assert!(
                goal.total_shares > DEFAULT_INITIAL_DEPOSIT,
                "Total shares should include contributor's shares"
            );
        }
        Err(e) => panic!("Failed to contribute to goal: {:#?}", e),
    }
}

#[test]
fn test_contribute_to_goal_multiple_contributors() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_create_agent()
        .with_validated_agent(0);

    let goal_owner = fixt.create_keypair();
    let contributor1 = fixt.create_keypair();
    let contributor2 = fixt.create_keypair();
    let network_config_pda = fixt.find_network_config_pda().0;

    let result = fixt.set_goal(
        &goal_owner,
        0,
        DEFAULT_GOAL_SPECIFICATION_CID.to_string(),
        10,
        0,
        0,
        DEFAULT_INITIAL_DEPOSIT,
    );
    assert!(result.is_ok(), "Failed to set goal: {:#?}", result.err());

    // First contribution
    let result1 = fixt.contribute_to_goal(&contributor1, 0, DEFAULT_CONTRIBUTION_AMOUNT);
    assert!(result1.is_ok(), "Failed first contribution");

    // Second contribution
    let result2 = fixt.contribute_to_goal(&contributor2, 0, DEFAULT_CONTRIBUTION_AMOUNT);
    assert!(result2.is_ok(), "Failed second contribution");

    let goal = fixt.get_goal(&network_config_pda, 0);
    let (goal_pda, _) = fixt.find_goal_pda(&network_config_pda, 0);
    let contribution1 = fixt.get_contribution(&goal_pda, &contributor1.pubkey());
    let contribution2 = fixt.get_contribution(&goal_pda, &contributor2.pubkey());

    assert!(contribution1.shares > 0, "Contributor1 should have shares");
    assert!(contribution2.shares > 0, "Contributor2 should have shares");

    println!("Total shares: {}", goal.total_shares);
    println!("Contribution1 shares: {}", contribution1.shares);
    println!("Contribution2 shares: {}", contribution2.shares);
    println!("Initial deposit: {}", DEFAULT_INITIAL_DEPOSIT);
    println!("Contribution amount: {}", DEFAULT_CONTRIBUTION_AMOUNT);
    println!("Total contributions: {}", DEFAULT_CONTRIBUTION_AMOUNT * 2);
    println!(
        "Total shares should be sum of all contributions: {}",
        DEFAULT_INITIAL_DEPOSIT + DEFAULT_CONTRIBUTION_AMOUNT * 2
    );

    assert!(
        goal.total_shares >= DEFAULT_INITIAL_DEPOSIT + DEFAULT_CONTRIBUTION_AMOUNT * 2,
        "Total shares should include all contributions"
    );
}

#[test]
fn test_withdraw_from_goal() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_create_agent()
        .with_validated_agent(0)
        .with_set_goal(0, 0)
        .with_contribute_to_goal(0, DEFAULT_CONTRIBUTION_AMOUNT);

    let contributor = fixt.contributor.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda().0;

    let (goal_pda, _) = fixt.find_goal_pda(&network_config_pda, 0);
    let contribution_before = fixt.get_contribution(&goal_pda, &contributor.pubkey());
    let goal_before = fixt.get_goal(&network_config_pda, 0);
    let shares_to_burn = contribution_before.shares / 2;

    let result = fixt.withdraw_from_goal(&contributor, 0, shares_to_burn);

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let goal_after = fixt.get_goal(&network_config_pda, 0);
            let contribution_after = fixt.get_contribution(&goal_pda, &contributor.pubkey());

            assert_eq!(
                contribution_after.shares,
                contribution_before.shares - shares_to_burn,
                "Contribution shares should decrease"
            );

            assert_eq!(
                goal_after.total_shares,
                goal_before.total_shares - shares_to_burn,
                "Goal total shares should decrease"
            );

            assert_eq!(contribution_after.goal, goal_pda);
            assert_eq!(contribution_after.contributor, contributor.pubkey());
        }
        Err(e) => panic!("Failed to withdraw from goal: {:#?}", e),
    }
}

#[test]
fn test_claim_task() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_register_compute_node()
        .with_claim_compute_node()
        .with_validate_compute_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_goal()
        .with_set_goal(0, 0);

    let goal_slot_id = 0;
    let task_slot_id = 0;
    let max_task_cost = 1_000_000_000; // 1 SOL

    let result = fixt.claim_task(
        &fixt.compute_node.insecure_clone(),
        goal_slot_id,
        task_slot_id,
        max_task_cost,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config_pda = fixt.find_network_config_pda().0;
            let goal = fixt.get_goal(&network_config_pda, goal_slot_id);
            let task = fixt.get_task(&network_config_pda, task_slot_id);

            assert_eq!(task.status, TaskStatus::Processing);
            assert_eq!(task.compute_node, Some(fixt.compute_node.pubkey()));
            assert_eq!(task.max_task_cost, max_task_cost);
            assert_eq!(task.execution_count, 1);
            assert_eq!(goal.locked_for_tasks, max_task_cost);
        }
        Err(e) => panic!("Failed to claim task: {:#?}", e),
    }
}

#[test]
fn test_submit_task_result() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_register_compute_node()
        .with_claim_compute_node()
        .with_validate_compute_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_goal()
        .with_set_goal(0, 0);

    let goal_slot_id = 0;
    let task_slot_id = 0;
    let max_task_cost = 1_000_000_000;

    let result = fixt.claim_task(
        &fixt.compute_node.insecure_clone(),
        goal_slot_id,
        task_slot_id,
        max_task_cost,
    );
    assert!(result.is_ok(), "Failed to claim task");

    let input_cid = "QmTestInput123456789".to_string();
    let output_cid = "QmTestOutput123456789".to_string();

    let result = fixt.submit_task_result(
        &fixt.compute_node.insecure_clone(),
        task_slot_id,
        input_cid.clone(),
        output_cid.clone(),
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config_pda = fixt.find_network_config_pda().0;
            let task = fixt.get_task(&network_config_pda, task_slot_id);

            assert_eq!(task.status, TaskStatus::AwaitingValidation);
            assert_eq!(task.pending_input_cid, Some(input_cid));
            assert_eq!(task.pending_output_cid, Some(output_cid));
        }
        Err(e) => panic!("Failed to submit task result: {:#?}", e),
    }
}

#[test]
fn test_submit_task_validation_approved() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_register_compute_node()
        .with_claim_compute_node()
        .with_validate_compute_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_goal()
        .with_set_goal(0, 0);

    let goal_slot_id = 0;
    let task_slot_id = 0;
    let max_task_cost = 1_000_000_000;
    let payment_amount = 500_000_000;

    let result = fixt.claim_task(
        &fixt.compute_node.insecure_clone(),
        goal_slot_id,
        task_slot_id,
        max_task_cost,
    );
    assert!(result.is_ok(), "Failed to claim task");

    // Submit task result
    let input_cid = "QmTestInput123456789".to_string();
    let output_cid = "QmTestOutput123456789".to_string();
    let result = fixt.submit_task_result(
        &fixt.compute_node.insecure_clone(),
        task_slot_id,
        input_cid.clone(),
        output_cid.clone(),
    );
    assert!(result.is_ok(), "Failed to submit task result");

    // Compute validation proof
    let mut hasher = Sha256::new();
    hasher.update(input_cid.as_bytes());
    hasher.update(output_cid.as_bytes());
    let validation_proof: [u8; 32] = hasher.finalize().into();

    // Create Ed25519 instruction
    let ed25519_ix = Helpers::create_ed25519_instruction_to_submit_task_validation(
        goal_slot_id,
        task_slot_id,
        payment_amount,
        validation_proof,
        true,
        false,
        &fixt.tee_signing_keypair.insecure_clone(),
    );

    // Submit validation
    let result = fixt.submit_task_validation(
        &fixt.validator_node.insecure_clone(),
        goal_slot_id,
        task_slot_id,
        &fixt.compute_node.pubkey(),
        &ed25519_ix,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config_pda = fixt.find_network_config_pda().0;
            let goal = fixt.get_goal(&network_config_pda, goal_slot_id);
            let task = fixt.get_task(&network_config_pda, task_slot_id);
            let compute_node_info = fixt.get_node_info(&fixt.compute_node.pubkey());
            let (compute_node_info_pda, _) = fixt.find_node_info_pda(&fixt.compute_node.pubkey());

            assert_eq!(task.status, TaskStatus::Pending);
            assert_eq!(task.input_cid, Some(input_cid));
            assert_eq!(task.output_cid, Some(output_cid));
            assert_eq!(task.pending_input_cid, None);
            assert_eq!(task.pending_output_cid, None);
            assert_eq!(goal.locked_for_tasks, 0);
            assert_eq!(goal.current_iteration, 1);
            assert_eq!(compute_node_info.total_tasks_completed, 1);
            assert_eq!(compute_node_info.total_earned, payment_amount);

            let (node_treasury_pda, _) = fixt.find_node_treasury_pda(&compute_node_info_pda);
            let node_treasury_lamports = fixt.svm.get_lamports(&node_treasury_pda);

            assert!(
                node_treasury_lamports >= payment_amount,
                "Node treasury should have at least payment_amount. Got: {}, Expected: {}",
                node_treasury_lamports,
                payment_amount
            );
        }
        Err(e) => panic!("Failed to submit task validation: {:#?}", e),
    }
}
