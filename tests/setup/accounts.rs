use dac_client::accounts::{Agent, Contribution, Goal, NetworkConfig, NodeInfo, Task};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signature::Signer};

use crate::setup::TestFixture;

pub trait Accounts {
    fn find_network_config_pda(&self) -> (Pubkey, u8);
    fn get_network_config(&self) -> NetworkConfig;
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
    fn get_node_info(&self, node_pubkey: &Pubkey) -> NodeInfo;
    fn find_agent_pda(&self, network_config: &Pubkey, agent_slot_id: u64) -> (Pubkey, u8);
    fn get_agent(&self, network_config: &Pubkey, agent_slot_id: u64) -> Agent;
    fn find_goal_vault_pda(&self, goal: &Pubkey) -> (Pubkey, u8);
    fn get_goal(&self, network_config: &Pubkey, goal_slot_id: u64) -> Goal;
    fn find_contribution_pda(&self, goal: &Pubkey, contributor: &Pubkey) -> (Pubkey, u8);
    fn get_contribution(&self, goal: &Pubkey, contributor: &Pubkey) -> Contribution;
    fn get_task(&self, network_config: &Pubkey, task_slot_id: u64) -> Task;
}

impl Accounts for TestFixture {
    fn find_network_config_pda(&self) -> (Pubkey, u8) {
        let (pda, bump) = Pubkey::find_program_address(
            &[b"dac_network_config", self.authority.pubkey().as_ref()],
            &self.program_id,
        );
        (pda, bump)
    }

    fn get_network_config(&self) -> NetworkConfig {
        let addr = self.find_network_config_pda().0;

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

    fn get_node_info(&self, node_pubkey: &Pubkey) -> NodeInfo {
        let addr = self.find_node_info_pda(node_pubkey).0;

        let account = self
            .svm
            .get_account(&addr)
            .expect("NodeInfo account not found");

        NodeInfo::from_bytes(&account.data).expect("Failed to deserialize NodeInfo account")
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

    fn find_goal_vault_pda(&self, goal: &Pubkey) -> (Pubkey, u8) {
        let seeds = &[b"goal_vault", goal.as_ref()];
        Pubkey::find_program_address(seeds, &self.program_id)
    }

    fn get_goal(&self, network_config: &Pubkey, goal_slot_id: u64) -> Goal {
        let addr = self.find_goal_pda(network_config, goal_slot_id).0;

        let account = self.svm.get_account(&addr).expect("Goal account not found");

        Goal::from_bytes(&account.data).expect("Failed to deserialize Goal account")
    }

    fn find_contribution_pda(&self, goal: &Pubkey, contributor: &Pubkey) -> (Pubkey, u8) {
        let seeds = &[b"contribution", goal.as_ref(), contributor.as_ref()];
        Pubkey::find_program_address(seeds, &self.program_id)
    }

    fn get_contribution(&self, goal: &Pubkey, contributor: &Pubkey) -> Contribution {
        let addr = self.find_contribution_pda(goal, contributor).0;

        let account = self
            .svm
            .get_account(&addr)
            .expect("Contribution account not found");

        Contribution::from_bytes(&account.data).expect("Failed to deserialize Contribution account")
    }

    fn get_task(&self, network_config: &Pubkey, task_slot_id: u64) -> Task {
        let addr = self.find_task_pda(network_config, task_slot_id).0;

        let account = self.svm.get_account(&addr).expect("Task account not found");

        Task::from_bytes(&account.data).expect("Failed to deserialize Task account")
    }
}
