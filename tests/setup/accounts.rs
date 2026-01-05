use dac_client::dac::accounts::{Agent, NetworkConfig};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use crate::setup::TestFixture;

pub trait Accounts {
    fn find_network_config_pda(&self, authority: &Pubkey) -> (Pubkey, u8);
    fn get_network_config(&self, authority: &Pubkey) -> NetworkConfig;
    fn find_goal_pda(&self, network_config: &Pubkey, goal_id: u64) -> (Pubkey, u8);
    fn find_task_pda(&self, network_config: &Pubkey, task_id: u64) -> (Pubkey, u8);
    fn create_goal_pdas(&self, network_config: &Pubkey, count: u64) -> Vec<AccountMeta>;
    fn create_task_pdas(&self, network_config: &Pubkey, count: u64) -> Vec<AccountMeta>;
    fn create_remaining_accounts_for_initialize(
        &self,
        network_config: &Pubkey,
        allocate_goals: u64,
        allocate_tasks: u64,
    ) -> Vec<AccountMeta>;
    fn find_node_info_pda(&self, node_pubkey: &Pubkey) -> (Pubkey, u8);
    fn find_node_treasury_pda(&self, node_info: &Pubkey) -> (Pubkey, u8);
    fn get_node_info(&self, node_pubkey: &Pubkey) -> dac_client::dac::accounts::NodeInfo;
    fn find_agent_pda(&self, network_config: &Pubkey, agent_slot_id: u64) -> (Pubkey, u8);
    fn get_agent(&self, network_config: &Pubkey, agent_slot_id: u64) -> Agent;
}

impl Accounts for TestFixture {
    fn find_network_config_pda(&self, authority: &Pubkey) -> (Pubkey, u8) {
        let (pda, bump) = Pubkey::find_program_address(
            &[b"dac_network_config", authority.as_ref()],
            &self.program_id,
        );
        (pda, bump)
    }

    fn get_network_config(&self, authority: &Pubkey) -> NetworkConfig {
        let addr = self.find_network_config_pda(authority).0;

        let account = self
            .svm
            .get_account(&addr)
            .expect("Network config account not found");

        NetworkConfig::from_bytes(&account.data)
            .expect("Failed to deserialize network config account")
    }

    fn find_goal_pda(&self, network_config: &Pubkey, goal_id: u64) -> (Pubkey, u8) {
        let seeds = &[b"goal", network_config.as_ref(), &goal_id.to_le_bytes()];
        Pubkey::find_program_address(seeds, &self.program_id)
    }

    fn find_task_pda(&self, network_config: &Pubkey, task_id: u64) -> (Pubkey, u8) {
        let seeds = &[b"task", network_config.as_ref(), &task_id.to_le_bytes()];
        Pubkey::find_program_address(seeds, &self.program_id)
    }

    fn create_goal_pdas(&self, network_config: &Pubkey, count: u64) -> Vec<AccountMeta> {
        (0..count)
            .map(|goal_id| {
                let (pda, _bump) = self.find_goal_pda(network_config, goal_id);
                AccountMeta {
                    pubkey: pda,
                    is_signer: false,
                    is_writable: true,
                }
            })
            .collect()
    }

    fn create_task_pdas(&self, network_config: &Pubkey, count: u64) -> Vec<AccountMeta> {
        (0..count)
            .map(|task_id| {
                let (pda, _bump) = self.find_task_pda(network_config, task_id);
                AccountMeta {
                    pubkey: pda,
                    is_signer: false,
                    is_writable: true,
                }
            })
            .collect()
    }

    fn create_remaining_accounts_for_initialize(
        &self,
        network_config: &Pubkey,
        allocate_goals: u64,
        allocate_tasks: u64,
    ) -> Vec<AccountMeta> {
        let mut accounts = Vec::new();

        accounts.extend(self.create_goal_pdas(network_config, allocate_goals));
        accounts.extend(self.create_task_pdas(network_config, allocate_tasks));

        accounts
    }

    fn find_node_info_pda(&self, node_pubkey: &Pubkey) -> (Pubkey, u8) {
        let seeds = &[b"node_info", node_pubkey.as_ref()];
        Pubkey::find_program_address(seeds, &self.program_id)
    }

    fn find_node_treasury_pda(&self, node_info: &Pubkey) -> (Pubkey, u8) {
        let seeds = &[b"node_treasury", node_info.as_ref()];
        Pubkey::find_program_address(seeds, &self.program_id)
    }

    fn get_node_info(&self, node_pubkey: &Pubkey) -> dac_client::dac::accounts::NodeInfo {
        let addr = self.find_node_info_pda(node_pubkey).0;

        let account = self
            .svm
            .get_account(&addr)
            .expect("NodeInfo account not found");

        dac_client::dac::accounts::NodeInfo::from_bytes(&account.data)
            .expect("Failed to deserialize NodeInfo account")
    }

    fn find_agent_pda(&self, network_config: &Pubkey, agent_slot_id: u64) -> (Pubkey, u8) {
        let seeds = &[
            b"agent",
            network_config.as_ref(),
            &agent_slot_id.to_le_bytes(),
        ];
        Pubkey::find_program_address(seeds, &self.program_id)
    }

    fn get_agent(&self, network_config: &Pubkey, agent_slot_id: u64) -> Agent {
        let addr = self.find_agent_pda(network_config, agent_slot_id).0;

        let account = self
            .svm
            .get_account(&addr)
            .expect("Agent account not found");

        Agent::from_bytes(&account.data).expect("Failed to deserialize Agent account")
    }
}
