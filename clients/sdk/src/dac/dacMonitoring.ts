import type { Address } from 'gill';
import { base64 } from '@coral-xyz/anchor/dist/cjs/utils/bytes/index.js';
import {
  decodeNodeInfo,
  decodeAgent,
  decodeGoal,
  decodeTask,
  NODE_INFO_DISCRIMINATOR,
  AGENT_DISCRIMINATOR,
  GOAL_DISCRIMINATOR,
  TASK_DISCRIMINATOR,
  type NodeInfo,
  type Agent,
  type Goal,
  type Task,
} from '../generated/dac/accounts/index.js';
import type { NodeStatus, AgentStatus, GoalStatus, TaskStatus } from '../generated/dac/types/index.js';
import type { IMonitoringService, DacServiceDeps, IQueryService } from './dacService.js';

/**
 * Wait mode for status monitoring methods
 */
export enum WaitMode {
  /** Wait for all entities to reach the target status */
  All = 'all',
  /** Return as soon as the first entity reaches the target status */
  First = 'first',
}

/**
 * Generic helper to wait for accounts to reach a specific status using WebSocket subscriptions
 */
async function waitForStatus<T, TStatus>(
  deps: DacServiceDeps,
  queryService: IQueryService,
  targetKeys: Address[],
  targetStatus: TStatus,
  options: { timeoutMs?: number; waitMode?: WaitMode } | undefined,
  config: {
    getByStatus: (status: TStatus) => Promise<T[]>;
    discriminator: Uint8Array;
    statusOffset: number;
    decode: (encodedAccount: any) => { data: T };
    getKey: (item: T) => Address;
    getAccountKey: (notification: any) => Address;
    entityName: string;
    initialFoundItems?: Map<Address, T>;
  }
): Promise<T[]> {
  const { client, programAddress } = deps;
  const waitMode = options?.waitMode ?? WaitMode.All;
  const foundItems = config.initialFoundItems ? new Map(config.initialFoundItems) : new Map<Address, T>();
  
  if (!config.initialFoundItems) {
    const initialItems = await config.getByStatus(targetStatus);
    
    for (const item of initialItems) {
      const key = config.getKey(item);
      if (key && targetKeys.includes(key)) {
        foundItems.set(key, item);
      }
    }
    
    if (waitMode === WaitMode.First && foundItems.size > 0) {
      return Array.from(foundItems.values());
    }

    if (waitMode === WaitMode.All && foundItems.size === targetKeys.length) {
      return Array.from(foundItems.values());
    }
  } else {
    if (waitMode === WaitMode.First && foundItems.size > 0) {
      return Array.from(foundItems.values());
    }
    if (waitMode === WaitMode.All && foundItems.size === targetKeys.length) {
      return Array.from(foundItems.values());
    }
  }

  const abortController = new AbortController();
  let timeoutId: NodeJS.Timeout | null = null;
  
  if (options?.timeoutMs !== undefined) {
    timeoutId = setTimeout(() => {
      abortController.abort();
    }, options.timeoutMs);
  }

  try {
    const filters: any[] = [
      {
        memcmp: {
          offset: 0,
          bytes: Array.from(config.discriminator),
        },
      },
      {
        memcmp: {
          offset: config.statusOffset,
          bytes: [targetStatus as number],
        },
      },
    ];

    const notifications = await client.rpcSubscriptions
      .programNotifications(programAddress, {
        encoding: 'base64',
        filters,
        commitment: 'confirmed',
      })
      .subscribe({ abortSignal: abortController.signal });

    for await (const notification of notifications) {
      try {
        const accountInfo = notification.value.account;
        const encodedAccount = {
          address: notification.value.pubkey,
          data: base64.decode(String(accountInfo.data)),
          executable: accountInfo.executable,
          owner: accountInfo.owner,
          lamports: accountInfo.lamports,
          programAddress: accountInfo.owner,
          space: accountInfo.space ?? 0n,
        };
        const decoded = config.decode(encodedAccount);
        const item = 'exists' in decoded && decoded.exists ? decoded.data : decoded.data;
        if (!item) {
          continue;
        }

        const accountKey = config.getAccountKey(notification);
        const itemKey = config.getKey(item);
        const key = itemKey || accountKey;
        const matches = targetKeys.includes(key);
        
        if (matches) {
          foundItems.set(key, item);

          if (waitMode === WaitMode.First) {
            if (timeoutId) {
              clearTimeout(timeoutId);
            }
            abortController.abort();
            return Array.from(foundItems.values());
          }

          if (waitMode === WaitMode.All && foundItems.size === targetKeys.length) {
            if (timeoutId) {
              clearTimeout(timeoutId);
            }
            abortController.abort();
            return Array.from(foundItems.values());
          }
        }
      } catch (error) {
        console.error('Error in waitForStatus', error);
        continue;
      }
    }

    if (options?.timeoutMs !== undefined) {
      const expectedCount = waitMode === WaitMode.First ? 1 : targetKeys.length;
      throw new Error(
        `Timeout waiting for ${expectedCount} ${config.entityName}(s) to reach status ${targetStatus}`
      );
    }
    
    throw new Error(
      `Subscription ended unexpectedly while waiting for ${config.entityName}s to reach status ${targetStatus}`
    );
  } catch (error: any) {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    
    if (error.name === 'AbortError' || abortController.signal.aborted) {
      if (options?.timeoutMs !== undefined) {
        const expectedCount = waitMode === WaitMode.First ? 1 : targetKeys.length;
        throw new Error(
          `Timeout waiting for ${expectedCount} ${config.entityName}(s) to reach status ${targetStatus}`
        );
      }
      throw new Error(
        `Subscription aborted while waiting for ${config.entityName}s to reach status ${targetStatus}`
      );
    }
    
    throw error;
  }
}

/**
 * Create monitoring service factory
 */
export function createMonitoringService(
  deps: DacServiceDeps,
  queryService: IQueryService
): IMonitoringService {
  return {
    async waitForNodesStatus(
      nodePubkeys: Address[],
      targetStatus: NodeStatus,
      options?: { timeoutMs?: number; waitMode?: WaitMode }
    ): Promise<NodeInfo[]> {
      return waitForStatus(
        deps,
        queryService,
        nodePubkeys,
        targetStatus,
        options,
        {
          getByStatus: (status) => queryService.getNodesByStatus({ status }),
          discriminator: NODE_INFO_DISCRIMINATOR,
          statusOffset: 73,
          decode: decodeNodeInfo,
          getKey: (node) => node.nodePubkey,
          getAccountKey: (notification) => notification.value.pubkey,
          entityName: 'node',
        }
      );
    },

    async waitForAgentsStatus(
      agentAddresses: Address[],
      targetStatus: AgentStatus,
      options?: { timeoutMs?: number; waitMode?: WaitMode }
    ): Promise<Agent[]> {
      const waitMode = options?.waitMode ?? WaitMode.All;
      
      const foundAgents = new Map<Address, Agent>();
      for (const agentAddress of agentAddresses) {
        const agent = await queryService.getAgent(agentAddress);
        if (agent && agent.status === targetStatus) {
          foundAgents.set(agentAddress, agent);
        }
      }
      
      if (waitMode === WaitMode.First && foundAgents.size > 0) {
        return Array.from(foundAgents.values());
      }

      if (waitMode === WaitMode.All && foundAgents.size === agentAddresses.length) {
        return Array.from(foundAgents.values());
      }

      return waitForStatus(
        deps,
        queryService,
        agentAddresses,
        targetStatus,
        options,
        {
          getByStatus: (status) => queryService.getAgentsByStatus(status),
          discriminator: AGENT_DISCRIMINATOR,
          statusOffset: 48,
          decode: decodeAgent,
          getKey: (_agent) => '' as Address,
          getAccountKey: (notification) => notification.value.pubkey,
          entityName: 'agent',
          initialFoundItems: foundAgents,
        }
      );
    },

    async waitForGoalsStatus(
      goalAddresses: Address[],
      targetStatus: GoalStatus,
      options?: { timeoutMs?: number; waitMode?: WaitMode }
    ): Promise<Goal[]> {
      return waitForStatus(
        deps,
        queryService,
        goalAddresses,
        targetStatus,
        options,
        {
          getByStatus: (status) => queryService.getGoalsByStatus(status),
          discriminator: GOAL_DISCRIMINATOR,
          statusOffset: 8 + 8 + 32 + 32 + 32,
          decode: decodeGoal,
          getKey: (_goal) => '' as Address,
          getAccountKey: (notification) => notification.value.pubkey,
          entityName: 'goal',
        }
      );
    },

    async waitForTasksStatus(
      taskAddresses: Address[],
      targetStatus: TaskStatus,
      options?: { timeoutMs?: number; waitMode?: WaitMode }
    ): Promise<Task[]> {
      return waitForStatus(
        deps,
        queryService,
        taskAddresses,
        targetStatus,
        options,
        {
          getByStatus: (status) => queryService.getTasksByStatus(status),
          discriminator: TASK_DISCRIMINATOR,
          statusOffset: 49,
          decode: decodeTask,
          getKey: (_task) => '' as Address,
          getAccountKey: (notification) => notification.value.pubkey,
          entityName: 'task',
        }
      );
    },
  };
}
