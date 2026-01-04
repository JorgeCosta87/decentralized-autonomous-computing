use crate::setup::test_data::*;
use crate::setup::Accounts;
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
