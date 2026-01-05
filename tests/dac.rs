use crate::setup::test_data::*;
use crate::setup::Accounts;
use crate::setup::Helpers;
use crate::setup::Instructions;
use crate::setup::TestFixture;
use solana_sdk::signature::Signer;
use utils::Utils;

mod setup;

#[test]
fn test_initialize_network_without_remaining_accounts() {
    let mut fixt = TestFixture::new();
    let authority = fixt.authority.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda(&authority.pubkey()).0;

    let allocate_goals = 0;
    let allocate_tasks = 0;

    let result = fixt.initialize_network(
        &authority,
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
            let network_config = fixt.get_network_config(&authority.pubkey());

            assert_eq!(network_config.authority, authority.pubkey());
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
    let authority = fixt.authority.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda(&authority.pubkey()).0;

    let allocate_goals = 2;
    let allocate_tasks = 20;
    let remaining_accounts = fixt.create_remaining_accounts_for_initialize(
        &network_config_pda,
        allocate_goals,
        allocate_tasks,
    );

    let result = fixt.initialize_network(
        &authority,
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
            let network_config = fixt.get_network_config(&authority.pubkey());

            assert_eq!(network_config.authority, authority.pubkey());
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

    let authority = fixt.authority.insecure_clone();
    let compute_node_pubkey = fixt.compute_node.pubkey();

    let result = fixt.register_node(
        &fixt.compute_node_owner.insecure_clone(),
        &compute_node_pubkey,
        dac_client::dac::types::NodeType::Compute,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&compute_node_pubkey);
            let network_config = fixt.get_network_config(&authority.pubkey());

            assert_eq!(node_info.owner, fixt.compute_node_owner.pubkey());
            assert_eq!(node_info.node_pubkey, compute_node_pubkey);
            assert_eq!(
                node_info.node_type,
                dac_client::dac::types::NodeType::Compute
            );
            assert_eq!(
                node_info.status,
                dac_client::dac::types::NodeStatus::PendingClaim
            );
            assert_eq!(node_info.node_info_cid, None);
            assert_eq!(node_info.max_entries_before_transfer, 64);
            assert_eq!(network_config.compute_node_count, 0);
        }
        Err(e) => panic!("Failed to register compute node: {:#?}", e),
    }
}

#[test]
fn test_register_validator_node() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let authority = fixt.authority.insecure_clone();
    let validator_node_pubkey = fixt.validator_node.pubkey();

    let result = fixt.register_node(
        &fixt.validator_node_owner.insecure_clone(),
        &validator_node_pubkey,
        dac_client::dac::types::NodeType::Validator,
    );

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let node_info = fixt.get_node_info(&validator_node_pubkey);
            let network_config = fixt.get_network_config(&authority.pubkey());

            assert_eq!(node_info.owner, fixt.validator_node_owner.pubkey());
            assert_eq!(node_info.node_pubkey, validator_node_pubkey);
            assert_eq!(
                node_info.node_type,
                dac_client::dac::types::NodeType::Validator
            );
            assert_eq!(
                node_info.status,
                dac_client::dac::types::NodeStatus::PendingClaim
            );
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
            let network_config = fixt.get_network_config(&fixt.authority.pubkey());

            assert_eq!(
                node_info.status,
                dac_client::dac::types::NodeStatus::AwaitingValidation
            );
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
            let network_config = fixt.get_network_config(&fixt.authority.pubkey());

            assert_eq!(node_info.status, dac_client::dac::types::NodeStatus::Active);
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
            let network_config = fixt.get_network_config(&fixt.authority.pubkey());

            assert_eq!(node_info.status, dac_client::dac::types::NodeStatus::Active);
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

            assert_eq!(
                node_info.status,
                dac_client::dac::types::NodeStatus::Rejected
            );
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

    let authority = fixt.authority.insecure_clone();
    let agent_owner = fixt.agent_owner.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda(&authority.pubkey()).0;

    let result = fixt.create_agent(&agent_owner, DEFAULT_AGENT_CONFIG_CID.to_string());

    match result {
        Ok(_) => {
            fixt.svm.print_transaction_logs(&result.unwrap());
            let network_config = fixt.get_network_config(&authority.pubkey());
            let agent = fixt.get_agent(&network_config_pda, 0);

            assert_eq!(agent.agent_slot_id, 0);
            assert_eq!(agent.owner, agent_owner.pubkey());
            assert_eq!(agent.agent_config_cid, DEFAULT_AGENT_CONFIG_CID.to_string());
            assert_eq!(agent.agent_memory_cid, None);
            assert_eq!(agent.status, dac_client::dac::types::AgentStatus::Pending);
            assert_eq!(network_config.agent_count, 1);
        }
        Err(e) => panic!("Failed to create agent: {:#?}", e),
    }
}

#[test]
fn test_create_multiple_agents() {
    let mut fixt = TestFixture::new().with_initialize_network();

    let authority = fixt.authority.insecure_clone();
    let agent_owner = fixt.agent_owner.insecure_clone();
    let network_config_pda = fixt.find_network_config_pda(&authority.pubkey()).0;

    let result1 = fixt.create_agent(&agent_owner, DEFAULT_AGENT_CONFIG_CID.to_string());
    assert!(result1.is_ok(), "Failed to create first agent");

    let result2 = fixt.create_agent(&agent_owner, "QmSecondAgentConfigCID".to_string());
    assert!(result2.is_ok(), "Failed to create second agent");

    let network_config = fixt.get_network_config(&authority.pubkey());
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
