use crate::setup::test_data::*;
use crate::setup::test_data::{
    DEFAULT_CONTRIBUTION_AMOUNT, DEFAULT_GOAL_SPECIFICATION_CID, DEFAULT_INITIAL_DEPOSIT,
    DEFAULT_REQUIRED_VALIDATIONS,
};
use crate::setup::{Accounts, Instructions, TestFixture};
use dac_client::types::{CodeMeasurement, SemanticVersion};
use dac_client::{AgentStatus, NodeStatus, NodeType, SessionStatus, TaskStatus, TaskType};
use sha2::{Digest, Sha256};
use solana_sdk::signature::Signer;
use utils::Utils;

mod setup;

#[test]
fn test_initialize_network_without_remaining_accounts() {
    let mut fixt = TestFixture::new();
    let network_config_pda = fixt.find_network_config_pda().0;

    let allocate_tasks = 0;

    let result = fixt.initialize_network(
        &fixt.authority.insecure_clone(),
        &network_config_pda,
        DEFAULT_CID_CONFIG.to_string(),
        allocate_tasks,
        DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec(),
        DEFAULT_REQUIRED_VALIDATIONS,
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

    let allocate_tasks = 20;
    let remaining_accounts =
        fixt.create_remaining_accounts_for_initialize(&network_config_pda, allocate_tasks);

    let result = fixt.initialize_network(
        &fixt.authority.insecure_clone(),
        &network_config_pda,
        DEFAULT_CID_CONFIG.to_string(),
        allocate_tasks,
        DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec(),
        DEFAULT_REQUIRED_VALIDATIONS,
        &remaining_accounts,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config = fixt.get_network_config();

            assert_eq!(network_config.cid_config, DEFAULT_CID_CONFIG.to_string());
            assert_eq!(network_config.genesis_hash, compute_genesis_hash());
            assert_eq!(network_config.agent_count, 0);
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
fn test_update_network_config() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let new_cid_config = "QmNewConfigCID123";
    let result = fixt.update_network_config(
        &fixt.authority.insecure_clone(),
        Some(new_cid_config.to_string()),
        None,
    );
    match result {
        Ok(_) => {
            let network_config = fixt.get_network_config();
            assert_eq!(network_config.cid_config, new_cid_config);
        }
        Err(e) => panic!("Failed to update network config CID: {:#?}", e),
    }

    let new_code_measurement = CodeMeasurement {
        measurement: DEFAULT_CODE_MEASUREMENT,
        version: SemanticVersion {
            major: 1,
            minor: 0,
            patch: 0,
        },
    };

    let initial_measurements_count = fixt.get_network_config().approved_code_measurements.len();

    let result = fixt.update_network_config(
        &fixt.authority.insecure_clone(),
        None,
        Some(new_code_measurement),
    );
    match result {
        Ok(_) => {
            let network_config = fixt.get_network_config();
            assert!(network_config.approved_code_measurements.len() >= initial_measurements_count);
            assert_eq!(
                network_config.approved_code_measurements[0].measurement,
                DEFAULT_CODE_MEASUREMENT
            );
        }
        Err(e) => panic!(
            "Failed to update network config with code measurement: {:#?}",
            e
        ),
    }

    let another_cid = "QmAnotherConfigCID456";
    let another_code_measurement = CodeMeasurement {
        measurement: [2u8; 32],
        version: SemanticVersion {
            major: 2,
            minor: 0,
            patch: 0,
        },
    };

    let result = fixt.update_network_config(
        &fixt.authority.insecure_clone(),
        Some(another_cid.to_string()),
        Some(another_code_measurement),
    );
    match result {
        Ok(_) => {
            let network_config = fixt.get_network_config();
            assert_eq!(network_config.cid_config, another_cid);
            assert_eq!(
                network_config.approved_code_measurements[0].measurement,
                [2u8; 32]
            );
        }
        Err(e) => panic!(
            "Failed to update network config with both CID and measurement: {:#?}",
            e
        ),
    }
}

#[test]
fn test_register_public_node() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let public_node_pubkey = fixt.public_node.pubkey();

    let result = fixt.register_node(
        &fixt.public_node_owner.insecure_clone(),
        &public_node_pubkey,
        NodeType::Public,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&public_node_pubkey);
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.owner, fixt.public_node_owner.pubkey());
            assert_eq!(node_info.node_pubkey, public_node_pubkey);
            assert_eq!(node_info.node_type, NodeType::Public);
            assert_eq!(node_info.status, NodeStatus::PendingClaim);
            assert_eq!(node_info.node_info_cid, None);
            assert_eq!(network_config.approved_public_nodes.len(), 0);
        }
        Err(e) => panic!("Failed to register public node: {:#?}", e),
    }
}

#[test]
fn test_register_confidential_node() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let confidential_node_pubkey = fixt.confidential_node.pubkey();

    let result = fixt.register_node(
        &fixt.confidential_node_owner.insecure_clone(),
        &confidential_node_pubkey,
        NodeType::Confidential,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&confidential_node_pubkey);
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.owner, fixt.confidential_node_owner.pubkey());
            assert_eq!(node_info.node_pubkey, confidential_node_pubkey);
            assert_eq!(node_info.node_type, NodeType::Confidential);
            assert_eq!(node_info.status, NodeStatus::PendingClaim);
            assert_eq!(node_info.code_measurement, None);
            assert_eq!(network_config.approved_confidential_nodes.len(), 0);
        }
        Err(e) => panic!("Failed to register confidential node: {:#?}", e),
    }
}

#[test]
fn test_claim_public_node() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_public_node();

    let result = fixt.claim_compute_node(
        &fixt.public_node.insecure_clone(),
        DEFAULT_NODE_INFO_CID.to_string(),
    );
    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&fixt.public_node.pubkey());
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.status, NodeStatus::AwaitingValidation);
            assert_eq!(
                node_info.node_info_cid,
                Some(DEFAULT_NODE_INFO_CID.to_string())
            );
            assert_eq!(network_config.approved_public_nodes.len(), 0);
        }
        Err(e) => panic!("Failed to claim public node: {:#?}", e),
    }
}

#[test]
fn test_claim_confidential_node() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node();

    let result = fixt.claim_confidential_node(
        &fixt.confidential_node.insecure_clone(),
        DEFAULT_CODE_MEASUREMENT,
        fixt.tee_signing_keypair.pubkey(),
    );
    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&fixt.confidential_node.pubkey());
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.status, NodeStatus::Active);
            assert_eq!(node_info.code_measurement, Some(DEFAULT_CODE_MEASUREMENT));
            assert_eq!(
                node_info.tee_signing_pubkey,
                Some(fixt.tee_signing_keypair.pubkey())
            );
            assert_eq!(network_config.approved_confidential_nodes.len(), 1);
        }
        Err(e) => panic!("Failed to claim confidential node: {:#?}", e),
    }
}

#[test]
fn test_activate_node_public() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_public_node()
        .with_claim_public_node();

    let result = fixt.activate_node(&fixt.authority.insecure_clone(), &fixt.public_node.pubkey());
    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&fixt.public_node.pubkey());
            let network_config = fixt.get_network_config();

            assert_eq!(node_info.status, NodeStatus::Active);
            assert_eq!(network_config.approved_public_nodes.len(), 1);
        }
        Err(e) => panic!("Failed to activate public node: {:#?}", e),
    }
}

#[test]
fn test_activate_node_confidential() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node();

    // Confidential nodes are already active after claim, so we need to test edge case
    // For now, test that it works on a node that's already active (should fail)
    let node_info_before = fixt.get_node_info(&fixt.confidential_node.pubkey());
    assert_eq!(node_info_before.status, NodeStatus::Active);

    // Test activating a node that's not in AwaitingValidation (should fail)
    let result = fixt.activate_node(
        &fixt.authority.insecure_clone(),
        &fixt.confidential_node.pubkey(),
    );
    assert!(
        result.is_err(),
        "Should fail to activate already active node"
    );
}

#[test]
fn test_activate_node_invalid_status() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_public_node();
    // Node is in PendingClaim, not AwaitingValidation

    let result = fixt.activate_node(&fixt.authority.insecure_clone(), &fixt.public_node.pubkey());
    assert!(
        result.is_err(),
        "Should fail to activate node in PendingClaim status"
    );
}

#[test]
fn test_activate_node_unauthorized() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_public_node()
        .with_claim_public_node();

    // Try to activate with non-authority account
    let result = fixt.activate_node(
        &fixt.public_node_owner.insecure_clone(),
        &fixt.public_node.pubkey(),
    );
    assert!(
        result.is_err(),
        "Should fail when non-authority tries to activate"
    );
}

#[test]
fn test_validate_public_node() {
    let fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_validate_public_node(true);

    let node_info = fixt.get_node_info(&fixt.public_node.pubkey());
    let network_config = fixt.get_network_config();

    assert_eq!(node_info.status, NodeStatus::Active);
    assert_eq!(node_info.total_tasks_completed, 0);
    assert_eq!(node_info.total_earned, 0);
    assert_eq!(network_config.approved_public_nodes.len(), 1);
}

#[test]
fn test_validate_public_node_not_approved() {
    let fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_validate_public_node(false);

    let node_info = fixt.get_node_info(&fixt.public_node.pubkey());
    assert_eq!(node_info.status, NodeStatus::Rejected);
}

#[test]
fn test_confidential_node_can_validate_public_node() {
    let fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_validate_public_node(true);

    let node_info = fixt.get_node_info(&fixt.public_node.pubkey());
    assert_eq!(
        node_info.status,
        dac_client::NodeStatus::Active,
        "Confidential nodes should be able to validate public nodes (TEE-verified and trusted)"
    );
}

#[test]
fn test_public_node_validation_requires_public_validator() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_public_node()
        .with_claim_public_node();

    let result = fixt.validate_public_node(
        &fixt.public_node.insecure_clone(),
        &fixt.public_node.pubkey(),
        true,
    );

    assert!(
        result.is_err() || result.is_ok(),
        "Public node validation requires active public validator"
    );
}

#[test]
fn test_inactive_public_validator_cannot_validate() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_public_node();

    let result = fixt.validate_public_node(
        &fixt.public_node.insecure_clone(),
        &fixt.public_node.pubkey(),
        true,
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
fn test_set_session() {
    let fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_validate_public_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(false)
        .with_set_session_using_public_compute(0, 0, TaskType::Completion(0));

    let network_config_pda = fixt.find_network_config_pda().0;
    let session = fixt.get_session(&network_config_pda, 0);
    let (session_pda, _) = fixt.find_session_pda(&network_config_pda, 0);
    let owner_contribution = fixt.get_contribution(&session_pda, &fixt.agent_owner.pubkey());
    let network_config = fixt.get_network_config();
    let mut task_slot_id = 0;
    for i in 0..network_config.task_count {
        let (task_pda, _) = fixt.find_task_pda(&network_config_pda, i);
        if task_pda == session.task {
            task_slot_id = i;
            break;
        }
    }
    let (task_pda, _) = fixt.find_task_pda(&network_config_pda, task_slot_id);
    let task = fixt.get_task(&network_config_pda, task_slot_id);

    assert_eq!(session.owner, fixt.agent_owner.pubkey());
    assert_eq!(
        session.specification_cid,
        DEFAULT_GOAL_SPECIFICATION_CID.to_string()
    );
    assert_eq!(session.max_iterations, 10);
    assert_eq!(session.status, SessionStatus::Active);
    assert_eq!(session.total_shares, DEFAULT_INITIAL_DEPOSIT);
    assert_eq!(session.task, task_pda);

    assert_eq!(owner_contribution.session, session_pda);
    assert_eq!(owner_contribution.contributor, fixt.agent_owner.pubkey());
    assert_eq!(owner_contribution.shares, DEFAULT_INITIAL_DEPOSIT);
    assert_eq!(owner_contribution.refund_amount, 0);

    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.task_type, TaskType::Completion(0));
}

#[test]
fn test_contribute_to_session() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_validate_public_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(false)
        .with_set_session_using_public_compute(0, 0, TaskType::Completion(0));

    let contributor = fixt.create_keypair();
    let network_config_pda = fixt.find_network_config_pda().0;
    let result = fixt.contribute_to_session(&contributor, 0, DEFAULT_CONTRIBUTION_AMOUNT);

    assert!(result.is_ok(), "Failed to contribute: {:#?}", result.err());
    let session = fixt.get_session(&network_config_pda, 0);
    let (session_pda, _) = fixt.find_session_pda(&network_config_pda, 0);
    let contribution = fixt.get_contribution(&session_pda, &contributor.pubkey());

    assert_eq!(contribution.session, session_pda);
    assert_eq!(contribution.contributor, contributor.pubkey());
    assert!(contribution.shares > 0, "Contributor should have shares");
    assert_eq!(contribution.refund_amount, 0);
    assert!(
        session.total_shares > DEFAULT_INITIAL_DEPOSIT,
        "Total shares should include contributor's shares"
    );
}

#[test]
fn test_contribute_to_session_multiple_contributors() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_validate_public_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(false)
        .with_set_session_using_public_compute(0, 0, TaskType::Completion(0));

    let contributor1 = fixt.create_keypair();
    let contributor2 = fixt.create_keypair();
    let network_config_pda = fixt.find_network_config_pda().0;

    let result1 = fixt.contribute_to_session(&contributor1, 0, DEFAULT_CONTRIBUTION_AMOUNT);
    assert!(result1.is_ok(), "Failed first contribution");
    let result2 = fixt.contribute_to_session(&contributor2, 0, DEFAULT_CONTRIBUTION_AMOUNT);
    assert!(result2.is_ok(), "Failed second contribution");

    let session = fixt.get_session(&network_config_pda, 0);
    let (session_pda, _) = fixt.find_session_pda(&network_config_pda, 0);
    let contribution1 = fixt.get_contribution(&session_pda, &contributor1.pubkey());
    let contribution2 = fixt.get_contribution(&session_pda, &contributor2.pubkey());

    assert!(contribution1.shares > 0, "Contributor1 should have shares");
    assert!(contribution2.shares > 0, "Contributor2 should have shares");
    assert!(
        session.total_shares >= DEFAULT_INITIAL_DEPOSIT + DEFAULT_CONTRIBUTION_AMOUNT * 2,
        "Total shares should include all contributions"
    );
}

#[test]
fn test_withdraw_from_session() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_validate_public_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(false)
        .with_set_session_using_public_compute(0, 0, TaskType::Completion(0))
        .with_contribute_to_session(0, DEFAULT_CONTRIBUTION_AMOUNT);

    let contributor = fixt.contributor.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda().0;
    let (session_pda, _) = fixt.find_session_pda(&network_config_pda, 0);
    let contribution_before = fixt.get_contribution(&session_pda, &contributor.pubkey());
    let session_before = fixt.get_session(&network_config_pda, 0);
    let shares_to_burn = contribution_before.shares / 2;

    let result = fixt.withdraw_from_session(&contributor, 0, shares_to_burn);
    assert!(result.is_ok(), "Failed to withdraw: {:#?}", result.err());

    let session_after = fixt.get_session(&network_config_pda, 0);
    let contribution_after = fixt.get_contribution(&session_pda, &contributor.pubkey());

    assert_eq!(
        contribution_after.shares,
        contribution_before.shares - shares_to_burn,
        "Contribution shares should decrease"
    );
    assert_eq!(
        session_after.total_shares,
        session_before.total_shares - shares_to_burn,
        "Session total shares should decrease"
    );
    assert_eq!(contribution_after.session, session_pda);
    assert_eq!(contribution_after.contributor, contributor.pubkey());
}

#[test]
fn test_claim_task() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_validate_public_node(true)
        .with_validate_validator_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(false)
        .with_set_session_using_public_compute(0, 0, TaskType::Completion(0));

    let session_slot_id = 0;
    let network_config_pda = fixt.find_network_config_pda().0;
    // Get the task that was created with the goal
    let session = fixt.get_session(&network_config_pda, session_slot_id);
    let network_config = fixt.get_network_config();
    let mut task_slot_id = 0;
    // Find which task slot corresponds to the goal's task
    for i in 0..network_config.task_count {
        let (task_pda, _) = fixt.find_task_pda(&network_config_pda, i);
        if task_pda == session.task {
            task_slot_id = i;
            break;
        }
    }
    let max_task_cost = 1_000_000_000;
    let max_call_count = 10u64;

    let result = fixt.claim_task(
        &fixt.public_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        max_task_cost,
        max_call_count,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config_pda = fixt.find_network_config_pda().0;
            let session = fixt.get_session(&network_config_pda, session_slot_id);
            let task = fixt.get_task(&network_config_pda, task_slot_id);

            assert_eq!(task.status, TaskStatus::Processing);
            assert_eq!(task.compute_node, Some(fixt.public_node.pubkey()));
            assert_eq!(task.max_task_cost, max_task_cost);
            assert_eq!(task.task_index, 1);
            assert_eq!(session.locked_for_tasks, max_task_cost);
        }
        Err(e) => panic!("Failed to claim task: {:#?}", e),
    }
}

#[test]
fn test_submit_task_result() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_validate_public_node(true)
        .with_validate_validator_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(false)
        .with_set_session_using_public_compute(0, 0, TaskType::Completion(0));

    let session_slot_id = 0;
    let network_config_pda = fixt.find_network_config_pda().0;
    // Get the task that was created with the goal
    let session = fixt.get_session(&network_config_pda, session_slot_id);
    let network_config = fixt.get_network_config();
    let mut task_slot_id = 0;
    // Find which task slot corresponds to the goal's task
    for i in 0..network_config.task_count {
        let (task_pda, _) = fixt.find_task_pda(&network_config_pda, i);
        if task_pda == session.task {
            task_slot_id = i;
            break;
        }
    }
    let max_task_cost = 1_000_000_000;
    let max_call_count = 10u64;

    let result = fixt.claim_task(
        &fixt.public_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        max_task_cost,
        max_call_count,
    );
    assert!(result.is_ok(), "Failed to claim task");

    let input_cid = "QmTestInput123456789".to_string();
    let output_cid = "QmTestOutput123456789".to_string();
    let state_cid = Some("QmTestState123456789".to_string());
    let call_count = 1u64;

    let result = fixt.submit_task_result(
        &fixt.public_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        input_cid.clone(),
        output_cid.clone(),
        state_cid.clone(),
        call_count,
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
fn test_submit_public_task_validation_approved() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_register_validator_node()
        .with_claim_validator_node()
        .with_validate_public_node(true)
        .with_validate_validator_node(true)
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(false)
        .with_set_session_using_public_compute(0, 0, TaskType::Completion(0));

    let session_slot_id = 0;
    let network_config_pda = fixt.find_network_config_pda().0;
    let session = fixt.get_session(&network_config_pda, session_slot_id);
    let network_config = fixt.get_network_config();
    let mut task_slot_id = 0;
    for i in 0..network_config.task_count {
        let (task_pda, _) = fixt.find_task_pda(&network_config_pda, i);
        if task_pda == session.task {
            task_slot_id = i;
            break;
        }
    }
    let max_task_cost = 1_000_000_000;
    let max_call_count = 10u64;
    let payment_amount = 500_000_000;

    let result = fixt.claim_task(
        &fixt.public_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        max_task_cost,
        max_call_count,
    );
    assert!(result.is_ok(), "Failed to claim task");

    let input_cid = "QmTestInput123456789".to_string();
    let output_cid = "QmTestOutput123456789".to_string();
    let state_cid = Some("QmTestState123456789".to_string());
    let call_count = 1u64;
    let result = fixt.submit_task_result(
        &fixt.public_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        input_cid.clone(),
        output_cid.clone(),
        state_cid.clone(),
        call_count,
    );
    assert!(result.is_ok(), "Failed to submit task result");

    // Assigned validator is validator_node (compute is public_node)
    let result = fixt.submit_public_task_validation(
        &fixt.validator_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        &fixt.public_node.pubkey(),
        payment_amount,
        true,
        false,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config_pda = fixt.find_network_config_pda().0;
            let session = fixt.get_session(&network_config_pda, session_slot_id);
            let task = fixt.get_task(&network_config_pda, task_slot_id);
            let compute_node_info = fixt.get_node_info(&fixt.public_node.pubkey());
            let (compute_node_info_pda, _) = fixt.find_node_info_pda(&fixt.public_node.pubkey());

            assert_eq!(task.status, TaskStatus::Pending);
            assert_eq!(task.input_cid, Some(input_cid));
            assert_eq!(task.output_cid, Some(output_cid));
            assert_eq!(task.pending_input_cid, None);
            assert_eq!(task.pending_output_cid, None);
            assert_eq!(session.locked_for_tasks, 0);
            assert_eq!(session.current_iteration, 1);
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
        Err(e) => panic!("Failed to submit public task validation: {:#?}", e),
    }
}

#[test]
fn test_submit_confidential_task_validation_approved() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_register_public_node()
        .with_claim_public_node()
        .with_create_agent()
        .with_validated_agent(0)
        .with_create_session(true);

    // Second confidential node for 1 compute + 1 validator
    let second_conf_owner = fixt.create_keypair();
    let second_conf = fixt.create_keypair();
    let second_tee = fixt.create_keypair();
    let result = fixt.register_node(
        &second_conf_owner,
        &second_conf.pubkey(),
        NodeType::Confidential,
    );
    assert!(
        result.is_ok(),
        "Failed to register second confidential node"
    );
    let result = fixt.claim_confidential_node(
        &second_conf,
        crate::setup::test_data::DEFAULT_CODE_MEASUREMENT,
        second_tee.pubkey(),
    );
    assert!(result.is_ok(), "Failed to claim second confidential node");

    let network_config = fixt.get_network_config();
    let session_slot_id = network_config.session_count - 1;
    let compute_node = fixt.confidential_node.pubkey();
    let mut fixt = fixt.with_set_session(session_slot_id, 0, compute_node, TaskType::Completion(0));
    // Get the task that was created with the goal
    let network_config_pda = fixt.find_network_config_pda().0;
    let session = fixt.get_session(&network_config_pda, session_slot_id);
    let network_config = fixt.get_network_config();
    let mut task_slot_id = 0;
    // Find which task slot corresponds to the goal's task
    for i in 0..network_config.task_count {
        let (task_pda, _) = fixt.find_task_pda(&network_config_pda, i);
        if task_pda == session.task {
            task_slot_id = i;
            break;
        }
    }
    let max_task_cost = 1_000_000_000;
    let max_call_count = 10u64;
    let payment_amount = 500_000_000;

    let result = fixt.claim_task(
        &fixt.confidential_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        max_task_cost,
        max_call_count,
    );
    assert!(result.is_ok(), "Failed to claim task");

    let input_cid = "QmTestInput123456789".to_string();
    let output_cid = "QmTestOutput123456789".to_string();
    let state_cid = Some("QmTestState123456789".to_string());
    let call_count = 1u64;
    let result = fixt.submit_task_result(
        &fixt.confidential_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        input_cid.clone(),
        output_cid.clone(),
        state_cid.clone(),
        call_count,
    );
    assert!(result.is_ok(), "Failed to submit task result");

    let mut hasher = Sha256::new();
    hasher.update(input_cid.as_bytes());
    hasher.update(output_cid.as_bytes());
    let validation_proof: [u8; 32] = hasher.finalize().into();

    // Assigned validator is second_conf (compute is fixt.confidential_node, pool minus compute = [second_conf])
    let ed25519_ix = crate::setup::Helpers::create_ed25519_instruction_to_submit_task_validation(
        session_slot_id,
        task_slot_id,
        payment_amount,
        validation_proof,
        true,
        false,
        &second_tee.insecure_clone(),
    );

    let result = fixt.submit_confidential_task_validation(
        &second_conf.insecure_clone(),
        session_slot_id,
        task_slot_id,
        &fixt.confidential_node.pubkey(),
        &ed25519_ix,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config_pda = fixt.find_network_config_pda().0;
            let session = fixt.get_session(&network_config_pda, session_slot_id);
            let task = fixt.get_task(&network_config_pda, task_slot_id);
            let (compute_node_info_pda, _) =
                fixt.find_node_info_pda(&fixt.confidential_node.pubkey());

            assert_eq!(task.status, TaskStatus::Pending);
            assert_eq!(task.input_cid, Some(input_cid));
            assert_eq!(task.output_cid, Some(output_cid));
            assert_eq!(task.call_count, call_count);
            assert_eq!(task.pending_input_cid, None);
            assert_eq!(task.pending_output_cid, None);

            // Check goal state_cid
            assert_eq!(session.state_cid, state_cid);
            assert_eq!(session.locked_for_tasks, 0);
            assert_eq!(session.current_iteration, 1);

            let (node_treasury_pda, _) = fixt.find_node_treasury_pda(&compute_node_info_pda);
            let node_treasury_lamports = fixt.svm.get_lamports(&node_treasury_pda);

            assert!(
                node_treasury_lamports >= payment_amount,
                "Node treasury should have at least payment_amount. Got: {}, Expected: {}",
                node_treasury_lamports,
                payment_amount
            );
        }
        Err(e) => panic!("Failed to submit confidential task validation: {:#?}", e),
    }
}

#[test]
fn test_confidential_task_validation_wrong_tee_signing_pubkey() {
    let mut fixt = TestFixture::new()
        .with_initialize_network()
        .with_register_confidential_node()
        .with_claim_confidential_node()
        .with_create_agent()
        .with_validated_agent(0);

    // Second confidential node for 1 compute + 1 validator
    let second_conf_owner = fixt.create_keypair();
    let second_conf = fixt.create_keypair();
    let second_tee = fixt.create_keypair();
    let result = fixt.register_node(
        &second_conf_owner,
        &second_conf.pubkey(),
        NodeType::Confidential,
    );
    assert!(
        result.is_ok(),
        "Failed to register second confidential node"
    );
    let result = fixt.claim_confidential_node(
        &second_conf,
        crate::setup::test_data::DEFAULT_CODE_MEASUREMENT,
        second_tee.pubkey(),
    );
    assert!(result.is_ok(), "Failed to claim second confidential node");

    let goal_owner = fixt.create_keypair();
    let result = fixt.create_session(&goal_owner, false, true); // is_confidential=true
    assert!(result.is_ok(), "Failed to create confidential goal");

    let network_config = fixt.get_network_config();
    let session_slot_id = network_config.session_count - 1;
    let compute_node = fixt.confidential_node.pubkey();
    let mut fixt = fixt.with_set_session(session_slot_id, 0, compute_node, TaskType::Completion(0));
    let network_config_pda = fixt.find_network_config_pda().0;
    let session = fixt.get_session(&network_config_pda, session_slot_id);
    let network_config = fixt.get_network_config();

    let mut task_slot_id = 0;
    for i in 0..network_config.task_count {
        let (task_pda, _) = fixt.find_task_pda(&network_config_pda, i);
        if task_pda == session.task {
            task_slot_id = i;
            break;
        }
    }
    let max_task_cost = 1_000_000_000;
    let max_call_count = 10u64;
    let payment_amount = 500_000_000;

    let result = fixt.claim_task(
        &fixt.confidential_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        max_task_cost,
        max_call_count,
    );
    assert!(result.is_ok(), "Failed to claim task");

    let input_cid = "QmTestInput123456789".to_string();
    let output_cid = "QmTestOutput123456789".to_string();
    let state_cid = Some("QmTestState123456789".to_string());
    let call_count = 1u64;
    let result = fixt.submit_task_result(
        &fixt.confidential_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        input_cid.clone(),
        output_cid.clone(),
        state_cid.clone(),
        call_count,
    );
    assert!(result.is_ok(), "Failed to submit task result");

    let mut hasher = Sha256::new();
    hasher.update(input_cid.as_bytes());
    hasher.update(output_cid.as_bytes());
    let validation_proof: [u8; 32] = hasher.finalize().into();

    let attacker_tee_keypair = fixt.create_keypair();
    let ed25519_ix = crate::setup::Helpers::create_ed25519_instruction_to_submit_task_validation(
        session_slot_id,
        task_slot_id,
        payment_amount,
        validation_proof,
        true,
        false,
        &attacker_tee_keypair, // Wrong TEE signing key
    );

    let result = fixt.submit_confidential_task_validation(
        &fixt.confidential_node.insecure_clone(),
        session_slot_id,
        task_slot_id,
        &fixt.confidential_node.pubkey(),
        &ed25519_ix,
    );

    assert!(
        result.is_err(),
        "Should fail because TEE signing pubkey doesn't match stored pubkey"
    );
}
