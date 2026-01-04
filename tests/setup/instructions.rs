use dac_client::dac::instructions::InitializeNetworkBuilder;
use dac_client::dac::types::CodeMeasurement;
use litesvm::types::TransactionResult;
use solana_sdk::{
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use utils::Utils;

use crate::setup::test_data::*;
use crate::setup::Accounts;
use crate::setup::TestFixture;

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
    fn initialize_network_with_defaults(&mut self) -> TransactionResult;
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
    fn initialize_network_with_defaults(&mut self) -> TransactionResult {
        let network_config_pda = self.find_network_config_pda(&self.authority.pubkey()).0;

        let remaining_accounts = self.create_remaining_accounts_for_initialize(
            &network_config_pda,
            DEFAULT_ALLOCATE_GOALS,
            DEFAULT_ALLOCATE_TASKS,
        );

        self.initialize_network(
            &self.authority.insecure_clone(),
            &network_config_pda,
            DEFAULT_CID_CONFIG.to_string(),
            DEFAULT_ALLOCATE_GOALS,
            DEFAULT_ALLOCATE_TASKS,
            DEFAULT_APPROVED_CODE_MEASUREMENTS.to_vec(),
            &remaining_accounts,
        )
    }
}
