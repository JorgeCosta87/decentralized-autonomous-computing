import type { Address } from 'gill';
import {
  deriveNetworkConfigAddress,
  deriveAgentAddress,
  deriveGoalAddress,
  deriveTaskAddress,
  deriveContributionAddress,
  deriveNodeInfoAddress,
} from './dacPdas.js';
import {
  fetchMaybeNetworkConfig,
  fetchMaybeAgent,
  fetchMaybeGoal,
  fetchMaybeTask,
  fetchMaybeContribution,
  fetchMaybeNodeInfo,
  decodeNodeInfo,
  decodeAgent,
  decodeTask,
  decodeGoal,
  NODE_INFO_DISCRIMINATOR,
  AGENT_DISCRIMINATOR,
  TASK_DISCRIMINATOR,
  GOAL_DISCRIMINATOR,
  type NetworkConfig,
  type Agent,
  type Goal,
  type Contribution,
  type NodeInfo,
  type Task,
} from '../generated/dac/accounts/index.js';
import type { NodeStatus, AgentStatus, TaskStatus, GoalStatus, NodeType } from '../generated/dac/types/index.js';
import type { IQueryService, DacServiceDeps } from './dacService.js';
import { decodeAccountsFromResponse } from './dacUtils.js';

/**
 * Create query service factory
 */
export function createQueryService(deps: DacServiceDeps): IQueryService {
  const { client, programAddress, getAuthority } = deps;

  return {
    async getNetworkConfig(authority?: Address): Promise<NetworkConfig | null> {
      const auth = authority || getAuthority();
      if (!auth) {
        throw new Error('Authority is required. Either set it with setAuthority() or pass it as parameter.');
      }
      const networkConfigAddress = await deriveNetworkConfigAddress(programAddress, auth);
      const account = await fetchMaybeNetworkConfig(client.rpc, networkConfigAddress);
      return account.exists ? account.data : null;
    },

    async getAgent(agentAddress: Address): Promise<Agent | null> {
      const account = await fetchMaybeAgent(client.rpc, agentAddress);
      return account.exists ? account.data : null;
    },

    async getAgentBySlot(networkConfig: Address, agentSlotId: bigint): Promise<Agent | null> {
      const agentAddress = await deriveAgentAddress(programAddress, networkConfig, agentSlotId);
      const account = await fetchMaybeAgent(client.rpc, agentAddress);
      return account.exists ? account.data : null;
    },

    async getGoal(networkConfig: Address, goalSlotId: bigint): Promise<Goal | null> {
      const goalAddress = await deriveGoalAddress(programAddress, networkConfig, goalSlotId);
      const account = await fetchMaybeGoal(client.rpc, goalAddress);
      return account.exists ? account.data : null;
    },

    async getTask(networkConfig: Address, taskSlotId: bigint): Promise<Task | null> {
      const taskAddress = await deriveTaskAddress(programAddress, networkConfig, taskSlotId);
      const account = await fetchMaybeTask(client.rpc, taskAddress);
      return account.exists ? account.data : null;
    },

    async getContribution(goal: Address, contributor: Address): Promise<Contribution | null> {
      const contributionAddress = await deriveContributionAddress(programAddress, goal, contributor);
      const account = await fetchMaybeContribution(client.rpc, contributionAddress);
      return account.exists ? account.data : null;
    },

    async getNodeInfo(nodePubkey: Address): Promise<NodeInfo | null> {
      const nodeInfoAddress = await deriveNodeInfoAddress(programAddress, nodePubkey);
      const account = await fetchMaybeNodeInfo(client.rpc, nodeInfoAddress);
      return account.exists ? account.data : null;
    },

    async getNodesByStatus(params?: {
      status?: NodeStatus;
      nodeType?: NodeType;
    }): Promise<NodeInfo[]> {
      const filters: any[] = [
        {
          memcmp: {
            offset: 0,
            bytes: Array.from(NODE_INFO_DISCRIMINATOR),
          },
        },
      ];

      if (params?.nodeType !== undefined) {
        filters.push({
          memcmp: {
            offset: 72,
            bytes: [params.nodeType],
          },
        });
      }

      if (params?.status !== undefined) {
        filters.push({
          memcmp: {
            offset: 73,
            bytes: [params.status],
          },
        });
      }

      const response = await client.rpc
        .getProgramAccounts(programAddress, {
          encoding: 'base64',
          filters,
        })
        .send();

      return decodeAccountsFromResponse(response, decodeNodeInfo);
    },

    async getAgentsByStatus(status?: AgentStatus): Promise<Agent[]> {
      const filters: any[] = [
        {
          memcmp: {
            offset: 0,
            bytes: Array.from(AGENT_DISCRIMINATOR),
          },
        },
      ];

      if (status !== undefined) {
        filters.push({
          memcmp: {
            offset: 48,
            bytes: [status],
          },
        });
      }

      const response = await client.rpc
        .getProgramAccounts(programAddress, {
          encoding: 'base64',
          filters,
        })
        .send();

      return decodeAccountsFromResponse(response, decodeAgent);
    },

    async getTasksByStatus(status?: TaskStatus): Promise<Task[]> {
      const filters: any[] = [
        {
          memcmp: {
            offset: 0,
            bytes: Array.from(TASK_DISCRIMINATOR),
          },
        },
      ];

      if (status !== undefined) {
        filters.push({
          memcmp: {
            offset: 49,
            bytes: [status],
          },
        });
      }

      const response = await client.rpc
        .getProgramAccounts(programAddress, {
          encoding: 'base64',
          filters,
        })
        .send();

      return decodeAccountsFromResponse(response, decodeTask);
    },

    async getGoalsByStatus(status?: GoalStatus): Promise<Goal[]> {
      const filters: any[] = [
        {
          memcmp: {
            offset: 0,
            bytes: Array.from(GOAL_DISCRIMINATOR),
          },
        },
      ];

      if (status !== undefined) {
        filters.push({
          memcmp: {
            offset: 112,
            bytes: [status],
          },
        });
      }

      const response = await client.rpc
        .getProgramAccounts(programAddress, {
          encoding: 'base64',
          filters,
        })
        .send();

      return decodeAccountsFromResponse(response, decodeGoal);
    },
  };
}
