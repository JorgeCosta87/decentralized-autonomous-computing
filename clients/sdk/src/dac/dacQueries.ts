import type { Address, Rpc } from '@solana/kit';
import {
  deriveNetworkConfigAddress,
  deriveAgentAddress,
  deriveGoalAddress,
  deriveTaskAddress,
  deriveContributionAddress,
  deriveGoalVaultAddress,
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
  decodeContribution,
  CONTRIBUTION_DISCRIMINATOR,
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
  const { rpc, programAddress, getAuthority } = deps;

  return {
    async getNetworkConfig(authority?: Address): Promise<NetworkConfig | null> {
      const auth = authority || getAuthority();
      if (!auth) {
        throw new Error('Authority is required. Either set it with setAuthority() or pass it as parameter.');
      }
      const networkConfigAddress = await deriveNetworkConfigAddress(programAddress, auth);
      const account = await fetchMaybeNetworkConfig(rpc, networkConfigAddress);
      return account.exists ? account.data : null;
    },

    async getAgent(agentAddress: Address): Promise<Agent | null> {
      const account = await fetchMaybeAgent(rpc, agentAddress);
      return account.exists ? account.data : null;
    },

    async getAgentBySlot(networkConfig: Address, agentSlotId: bigint): Promise<Agent | null> {
      const agentAddress = await deriveAgentAddress(programAddress, networkConfig, agentSlotId);
      const account = await fetchMaybeAgent(rpc, agentAddress);
      return account.exists ? account.data : null;
    },

    async getGoal(networkConfig: Address, goalSlotId: bigint): Promise<Goal | null> {
      const goalAddress = await deriveGoalAddress(programAddress, networkConfig, goalSlotId);
      const account = await fetchMaybeGoal(rpc, goalAddress);
      return account.exists ? account.data : null;
    },

    async getTask(networkConfig: Address, taskSlotId: bigint): Promise<Task | null> {
      const taskAddress = await deriveTaskAddress(programAddress, networkConfig, taskSlotId);
      const account = await fetchMaybeTask(rpc, taskAddress);
      return account.exists ? account.data : null;
    },

    async getContribution(goal: Address, contributor: Address): Promise<Contribution | null> {
      const contributionAddress = await deriveContributionAddress(programAddress, goal, contributor);
      const account = await fetchMaybeContribution(rpc, contributionAddress);
      return account.exists ? account.data : null;
    },

    async getNodeInfo(nodePubkey: Address): Promise<NodeInfo | null> {
      const nodeInfoAddress = await deriveNodeInfoAddress(programAddress, nodePubkey);
      const account = await fetchMaybeNodeInfo(rpc, nodeInfoAddress);
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

      const response = await (rpc as any).getProgramAccounts(programAddress, {
          encoding: 'base64',
        filters: filters as any,
      }).send();

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

      const response = await (rpc as any).getProgramAccounts(programAddress, {
          encoding: 'base64',
        filters: filters as any,
      }).send();

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

      const response = await (rpc as any).getProgramAccounts(programAddress, {
          encoding: 'base64',
        filters: filters as any,
      }).send();

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

      const response = await (rpc as any).getProgramAccounts(programAddress, {
          encoding: 'base64',
        filters: filters as any,
      }).send();

      return decodeAccountsFromResponse(response, decodeGoal);
    },

    async batchGetContributionsForGoals(
      networkConfig: Address,
      goalSlotIds: bigint[],
      contributorAddress: Address
    ): Promise<Map<bigint, Contribution | null>> {
      if (goalSlotIds.length === 0) {
        return new Map();
      }

      // Derive all contribution addresses upfront
      const contributionAddresses: Address[] = [];
      const goalSlotIdToAddress = new Map<bigint, Address>();
      
      for (const goalSlotId of goalSlotIds) {
        const goalAddress = await deriveGoalAddress(programAddress, networkConfig, goalSlotId);
        const contributionAddress = await deriveContributionAddress(programAddress, goalAddress, contributorAddress);
        contributionAddresses.push(contributionAddress);
        goalSlotIdToAddress.set(goalSlotId, contributionAddress);
      }

      // Batch fetch all contribution accounts using getMultipleAccounts
      try {
        const response = await (rpc as any).getMultipleAccounts(
          contributionAddresses
        ).send();

        const result = new Map<bigint, Contribution | null>();
        
        if (response.value) {
          let idx = 0;
          for (const goalSlotId of goalSlotIds) {
            const accountInfo = response.value[idx];
            if (accountInfo && accountInfo.data) {
              try {
                const decoded = decodeContribution({
                  address: contributionAddresses[idx],
                  data: new Uint8Array(Buffer.from(accountInfo.data[0], 'base64')),
                  executable: accountInfo.executable || false,
                  lamports: accountInfo.lamports as any,
                  programAddress: accountInfo.owner,
                  space: (accountInfo.space ?? 0n) as any,
                });
                result.set(goalSlotId, decoded.data);
              } catch (e) {
                result.set(goalSlotId, null);
              }
            } else {
              result.set(goalSlotId, null);
            }
            idx++;
          }
        } else {
          goalSlotIds.forEach(slotId => result.set(slotId, null));
        }

        return result;
      } catch (error) {
        // Fallback to individual calls if batch fails
        console.warn('Batch getMultipleAccounts failed, falling back to individual calls:', error);
        const results = await Promise.all(
          goalSlotIds.map(async (goalSlotId) => {
            const contributionAddress = goalSlotIdToAddress.get(goalSlotId);
            if (!contributionAddress) return { goalSlotId, contribution: null };
            try {
              const contribution = await this.getContribution(
                await deriveGoalAddress(programAddress, networkConfig, goalSlotId),
                contributorAddress
              );
              return { goalSlotId, contribution };
            } catch (e) {
              return { goalSlotId, contribution: null };
            }
          })
        );

        const result = new Map<bigint, Contribution | null>();
        results.forEach(({ goalSlotId, contribution }) => result.set(goalSlotId, contribution));
        return result;
      }
    },

    async batchGetVaultBalances(
      networkConfig: Address,
      goalSlotIds: bigint[]
    ): Promise<Map<bigint, { balance: bigint; rentExempt: bigint }>> {
      if (goalSlotIds.length === 0) {
        return new Map();
      }

      // Derive all vault addresses
      const vaultAddresses: Address[] = [];
      const goalSlotIdToVault = new Map<bigint, Address>();
      
      for (const goalSlotId of goalSlotIds) {
        const goalAddress = await deriveGoalAddress(programAddress, networkConfig, goalSlotId);
        const vaultAddress = await deriveGoalVaultAddress(programAddress, goalAddress);
        vaultAddresses.push(vaultAddress);
        goalSlotIdToVault.set(goalSlotId, vaultAddress);
      }

      // Get rent exempt minimum (cached)
      let rentExempt: bigint;
      try {
        const rent = await (rpc as any).getMinimumBalanceForRentExemption(0).send();
        rentExempt = BigInt(rent || 0);
      } catch (e) {
        rentExempt = 0n;
      }

      // Batch fetch all vault balances
      try {
        const response = await (rpc as any).getMultipleAccounts(
          vaultAddresses
        ).send();

        const result = new Map<bigint, { balance: bigint; rentExempt: bigint }>();
        
        if (response.value) {
          let idx = 0;
          for (const goalSlotId of goalSlotIds) {
            const accountInfo = response.value[idx];
            if (accountInfo) {
              result.set(goalSlotId, {
                balance: BigInt(accountInfo.lamports || 0),
                rentExempt,
              });
            } else {
              result.set(goalSlotId, { balance: 0n, rentExempt });
            }
            idx++;
          }
        } else {
          goalSlotIds.forEach(slotId => result.set(slotId, { balance: 0n, rentExempt }));
        }

        return result;
      } catch (error) {
        // Fallback to individual calls
        console.warn('Batch getMultipleAccounts failed for vaults, falling back:', error);
        const results = await Promise.all(
          goalSlotIds.map(async (goalSlotId) => {
            const vaultAddress = goalSlotIdToVault.get(goalSlotId);
            if (!vaultAddress) return { goalSlotId, balance: 0n, rentExempt };
            try {
              const accountInfo = await (rpc as any).getAccountInfo(vaultAddress).send();
              return {
                goalSlotId,
                balance: accountInfo?.value ? BigInt(accountInfo.value.lamports || 0) : 0n,
                rentExempt,
              };
            } catch (e) {
              return { goalSlotId, balance: 0n, rentExempt };
            }
          })
        );

        const result = new Map<bigint, { balance: bigint; rentExempt: bigint }>();
        results.forEach(({ goalSlotId, balance, rentExempt }) => 
          result.set(goalSlotId, { balance, rentExempt })
        );
        return result;
      }
    },

    async getContributorsForGoals(
      networkConfig: Address,
      goalSlotIds: bigint[]
    ): Promise<Map<bigint, { count: number; contributors: Array<{ address: Address; shares: bigint }> }>> {
      if (goalSlotIds.length === 0) {
        return new Map();
      }

      // Derive all goal addresses
      const goalAddresses = await Promise.all(
        goalSlotIds.map(slotId => deriveGoalAddress(programAddress, networkConfig, slotId))
      );
      const goalSlotIdToAddress = new Map<bigint, Address>();
      goalSlotIds.forEach((slotId, idx) => goalSlotIdToAddress.set(slotId, goalAddresses[idx]));

      // Single getProgramAccounts call to get ALL contributions for ALL goals
      const response = await (rpc as any).getProgramAccounts(programAddress, {
        encoding: 'base64',
        filters: [
          {
            memcmp: {
              offset: 0,
              bytes: Array.from(CONTRIBUTION_DISCRIMINATOR),
            },
          },
        ],
      }).send();

      // Decode all contributions
      const allContributions = decodeAccountsFromResponse(response, decodeContribution);

      // Group contributions by goal address
      const contributionsByGoal = new Map<Address, Array<{ contribution: Contribution; contributor: Address }>>();
      for (const account of response) {
        try {
          const contribution = decodeContribution({
            address: account.pubkey,
            data: new Uint8Array(Buffer.from(String(account.account.data), 'base64')),
            executable: account.account.executable || false,
            lamports: account.account.lamports as any,
            programAddress: account.account.owner,
            space: (account.account.space ?? 0n) as any,
          });
          
          const goalAddress = contribution.data.goal;
          if (!contributionsByGoal.has(goalAddress)) {
            contributionsByGoal.set(goalAddress, []);
          }
          contributionsByGoal.get(goalAddress)!.push({
            contribution: contribution.data,
            contributor: account.pubkey,
          });
        } catch (e) {
          // Skip invalid contributions
        }
      }

      // Map to goalSlotId and return contributors with shares
      const result = new Map<bigint, { count: number; contributors: Array<{ address: Address; shares: bigint }> }>();
      for (const goalSlotId of goalSlotIds) {
        const goalAddress = goalSlotIdToAddress.get(goalSlotId);
        if (!goalAddress) {
          result.set(goalSlotId, { count: 0, contributors: [] });
          continue;
        }

        const contributions = contributionsByGoal.get(goalAddress) || [];
        const validContributors = contributions
          .filter(c => c.contribution.shares > 0n)
          .map(c => ({
            address: c.contributor,
            shares: c.contribution.shares,
          }));
        
        result.set(goalSlotId, {
          count: validContributors.length,
          contributors: validContributors,
        });
      }

      return result;
    },
  };
}
