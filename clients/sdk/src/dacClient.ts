import {
  type Address,
  type TransactionSigner,
  type SolanaClient,
  type Instruction,
  AccountRole,
} from 'gill';
import { sendTransaction } from './utils.js';
import {
  DAC_PROGRAM_ID,
  deriveNetworkConfigAddress,
  deriveAgentAddress,
  deriveGoalAddress,
  deriveTaskAddress,
  deriveGoalVaultAddress,
  deriveContributionAddress,
  deriveNodeInfoAddress,
} from './dacPdas.js';
import {
  getActivateNodeInstruction,
  getInitializeNetworkInstruction,
  getCreateAgentInstruction,
  getCreateGoalInstruction,
  getSetGoalInstructionAsync,
  getContributeToGoalInstructionAsync,
  getWithdrawFromGoalInstructionAsync,
  getRegisterNodeInstructionAsync,
  getUpdateNetworkConfigInstructionAsync,
  type InitializeNetworkInput,
  type CreateAgentInput,
  type CreateGoalInput,
  type SetGoalAsyncInput,
  type ContributeToGoalAsyncInput,
  type WithdrawFromGoalAsyncInput,
  type RegisterNodeAsyncInput,
  type UpdateNetworkConfigAsyncInput,
} from './generated/dac/instructions/index.js';
import type { NodeType } from './generated/dac/types/index.js';
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
} from './generated/dac/accounts/index.js';
import type { CodeMeasurementArgs, NodeStatus, AgentStatus, TaskStatus, GoalStatus } from './generated/dac/types/index.js';
import { base64 } from '@coral-xyz/anchor/dist/cjs/utils/bytes/index.js';

/**
 * Wait mode for waitForNodesStatus and waitForAgentsStatus
 */
export enum WaitMode {
  /** Wait for all nodes/agents to reach the target status */
  All = 'all',
  /** Return as soon as the first node/agent reaches the target status */
  First = 'first',
}

/**
 * Client for interacting with the DAC (Decentralized Autonomous Computing) program.
 * 
 * This client provides methods for frontend/UI operations only. Node operations (like
 * claimTask, submitTaskResult, etc.) are handled by separate node clients.
 * 
 * @example
 * ```typescript
 * import { createSolanaClient } from 'gill';
 * import { DacSDK } from './dacClient';
 * 
 * const client = createSolanaClient('https://api.mainnet-beta.solana.com');
 * const dacClient = new DacSDK(client);
 * 
 * // Initialize network
 * const { signature, networkConfigAddress } = await dacClient.initializeNetwork({
 *   authority: myKeypair,
 *   cidConfig: 'QmNetworkConfig...',
 *   allocateGoals: 10n,
 *   allocateTasks: 10n,
 *   approvedCodeMeasurements: [...],
 *   requiredValidations: 1
 * });
 * 
 * // Create an agent
 * const { signature, agentAddress } = await dacClient.createAgent({
 *   agentOwner: myKeypair,
 *   networkConfig: networkConfigAddress,
 *   agentConfigCid: 'QmXXX...'
 * });
 * 
 * // Create a goal (public or confidential)
 * const { signature, goalAddress } = await dacClient.createGoal({
 *   payer: myKeypair,
 *   owner: myKeypair,
 *   networkConfig: networkConfigAddress,
 *   isOwned: true,
 *   isConfidential: false
 * });
 * ```
 */
export class DacSDK {
  private authority: Address | null = null;

  constructor(
    private readonly client: SolanaClient,
    private readonly programAddress: Address = DAC_PROGRAM_ID
  ) {}

  setAuthority(authority: Address): void {
    this.authority = authority;
  }

  /**
   * Generic helper to decode accounts from getProgramAccounts response.
   * Uses the working pattern: base64.decode(account.data.toString())
   */
  private decodeAccountsFromResponse<T>(
    response: Array<{ 
      pubkey: Address; 
      account: { 
        data: unknown; 
        executable: boolean; 
        owner: Address; 
        lamports: bigint; 
        space?: bigint;
      };
    }>,
    decodeFn: (encodedAccount: any) => { data: T }
  ): T[] {
    try {
      const decoded = response.map(({ pubkey, account }) =>
        decodeFn({
          address: pubkey,
          data: base64.decode(String(account.data)),
          executable: account.executable,
          lamports: account.lamports,
          programAddress: account.owner,
          space: account.space ?? 0n,
        }),
      );

      return decoded.map((item) => item.data);
    } catch (error: any) {
      console.error('Error decoding accounts', error);
      return [];
    }
  }

  async getNetworkConfig(authority?: Address): Promise<NetworkConfig | null> {
    const auth = authority || this.authority;
    if (!auth) {
      throw new Error('Authority is required. Either set it with setAuthority() or pass it as parameter.');
    }
    const networkConfigAddress = await deriveNetworkConfigAddress(this.programAddress, auth);
    const account = await fetchMaybeNetworkConfig(this.client.rpc, networkConfigAddress);
    return account.exists ? account.data : null;
  }

  async getAgent(agentAddress: Address): Promise<Agent | null> {
    const account = await fetchMaybeAgent(this.client.rpc, agentAddress);
    return account.exists ? account.data : null;
  }

  async getAgentBySlot(networkConfig: Address, agentSlotId: bigint): Promise<Agent | null> {
    const agentAddress = await deriveAgentAddress(this.programAddress, networkConfig, agentSlotId);
    return this.getAgent(agentAddress);
  }

  async getGoal(networkConfig: Address, goalSlotId: bigint): Promise<Goal | null> {
    const goalAddress = await deriveGoalAddress(this.programAddress, networkConfig, goalSlotId);
    const account = await fetchMaybeGoal(this.client.rpc, goalAddress);
    return account.exists ? account.data : null;
  }

  async getTask(networkConfig: Address, taskSlotId: bigint): Promise<Task | null> {
    const taskAddress = await deriveTaskAddress(this.programAddress, networkConfig, taskSlotId);
    const account = await fetchMaybeTask(this.client.rpc, taskAddress);
    return account.exists ? account.data : null;
  }

  async getContribution(goal: Address, contributor: Address): Promise<Contribution | null> {
    const contributionAddress = await deriveContributionAddress(this.programAddress, goal, contributor);
    const account = await fetchMaybeContribution(this.client.rpc, contributionAddress);
    return account.exists ? account.data : null;
  }

  async getNodeInfo(nodePubkey: Address): Promise<NodeInfo | null> {
    const nodeInfoAddress = await deriveNodeInfoAddress(this.programAddress, nodePubkey);
    const account = await fetchMaybeNodeInfo(this.client.rpc, nodeInfoAddress);
    return account.exists ? account.data : null;
  }

  async initializeNetwork(params: {
    authority: TransactionSigner;
    cidConfig: string;
    allocateGoals: bigint;
    allocateTasks: bigint;
    approvedCodeMeasurements: CodeMeasurementArgs[];
    requiredValidations: number;
  }): Promise<{ signature: string; networkConfigAddress: Address }> {
    const networkConfigAddress = await deriveNetworkConfigAddress(
      this.programAddress,
      params.authority.address
    );

    const remainingAccounts: Address[] = [];

    for (let goalId = 0; goalId < params.allocateGoals; goalId++) {
      const goalAddress = await deriveGoalAddress(this.programAddress, networkConfigAddress, BigInt(goalId));
      remainingAccounts.push(goalAddress);
    }

    for (let taskId = 0; taskId < params.allocateTasks; taskId++) {
      const taskAddress = await deriveTaskAddress(this.programAddress, networkConfigAddress, BigInt(taskId));
      remainingAccounts.push(taskAddress);
    }

    const input: InitializeNetworkInput = {
      authority: params.authority,
      networkConfig: networkConfigAddress,
      cidConfig: params.cidConfig,
      allocateGoals: params.allocateGoals,
      allocateTasks: params.allocateTasks,
      approvedCodeMeasurements: params.approvedCodeMeasurements,
      requiredValidations: params.requiredValidations,
    };

    const instruction = getInitializeNetworkInstruction(input, {
      programAddress: this.programAddress,
    });

    // Add remaining accounts (goals and tasks) to the instruction
    const allAccounts = [
      ...instruction.accounts,
      ...remainingAccounts.map((address) => ({
        address,
        role: AccountRole.WRITABLE,
      })),
    ];

    const instructionWithRemainingAccounts: Instruction = {
      ...instruction,
      accounts: allAccounts as any,
    };

    const signature = await sendTransaction(this.client, params.authority, [instructionWithRemainingAccounts]);

    return { signature, networkConfigAddress };
  }

  async registerNode(params: {
    owner: TransactionSigner;
    networkConfig: Address;
    nodePubkey: Address;
    nodeType: NodeType;
  }): Promise<{ signature: string; nodeInfoAddress: Address; nodeTreasuryAddress: Address }> {
    const input: RegisterNodeAsyncInput = {
      owner: params.owner,
      networkConfig: params.networkConfig,
      nodePubkey: params.nodePubkey,
      nodeType: params.nodeType,
    };

    const instruction = await getRegisterNodeInstructionAsync(input, {
      programAddress: this.programAddress,
    });

    const nodeInfoAddress = instruction.accounts[2].address;
    const nodeTreasuryAddress = instruction.accounts[3].address;

    const signature = await sendTransaction(this.client, params.owner, [instruction]);

    return { signature, nodeInfoAddress, nodeTreasuryAddress };
  }

  async createAgent(params: {
    agentOwner: TransactionSigner;
    networkConfig: Address;
    agentConfigCid: string;
  }): Promise<{ signature: string; agentAddress: Address; agentSlotId: bigint }> {
    // Fetch network config using the provided networkConfig address
    const networkConfigAccount = await fetchMaybeNetworkConfig(this.client.rpc, params.networkConfig);
    if (!networkConfigAccount.exists || !networkConfigAccount.data) {
      throw new Error('Network config not found');
    }

    const agentSlotId = networkConfigAccount.data.agentCount;
    const agentAddress = await deriveAgentAddress(
      this.programAddress,
      params.networkConfig,
      agentSlotId
    );

    const input: CreateAgentInput = {
      agentOwner: params.agentOwner,
      networkConfig: params.networkConfig,
      agent: agentAddress,
      agentConfigCid: params.agentConfigCid,
    };

    const instruction = getCreateAgentInstruction(input, {
      programAddress: this.programAddress,
    });

    const signature = await sendTransaction(this.client, params.agentOwner, [instruction]);

    return { signature, agentAddress, agentSlotId };
  }

  async createGoal(params: {
    payer: TransactionSigner;
    owner: TransactionSigner;
    networkConfig: Address;
    isOwned: boolean;
    isConfidential: boolean;
  }): Promise<{ signature: string; goalAddress: Address; goalSlotId: bigint; taskAddress: Address; taskSlotId: bigint }> {
    const account = await fetchMaybeNetworkConfig(this.client.rpc, params.networkConfig);
    if (!account.exists || !account.data) {
      throw new Error('Network config not found');
    }
    const networkConfigData = account.data;

    const goalSlotId = networkConfigData.goalCount;
    const taskSlotId = networkConfigData.taskCount;
    const goalAddress = await deriveGoalAddress(this.programAddress, params.networkConfig, goalSlotId);
    const taskAddress = await deriveTaskAddress(this.programAddress, params.networkConfig, taskSlotId);

    const input: CreateGoalInput = {
      payer: params.payer,
      owner: params.owner,
      networkConfig: params.networkConfig,
      goal: goalAddress,
      task: taskAddress,
      isOwned: params.isOwned,
      isConfidential: params.isConfidential,
    };

    const instruction = getCreateGoalInstruction(input, {
      programAddress: this.programAddress,
    });

    const signature = await sendTransaction(this.client, params.payer, [instruction]);

    return { signature, goalAddress, goalSlotId, taskAddress, taskSlotId };
  }

  async setGoal(params: {
    owner: TransactionSigner;
    networkConfig: Address;
    goalSlotId: bigint;
    agentSlotId: bigint;
    taskSlotId: bigint;
    specificationCid: string;
    maxIterations: bigint;
    initialDeposit: bigint;
  }): Promise<string> {
    const goalAddress = await deriveGoalAddress(this.programAddress, params.networkConfig, params.goalSlotId);
    const agentAddress = await deriveAgentAddress(this.programAddress, params.networkConfig, params.agentSlotId);
    const taskAddress = await deriveTaskAddress(this.programAddress, params.networkConfig, params.taskSlotId);

    const input: SetGoalAsyncInput = {
      owner: params.owner,
      goal: goalAddress,
      task: taskAddress,
      agent: agentAddress,
      networkConfig: params.networkConfig,
      specificationCid: params.specificationCid,
      maxIterations: params.maxIterations,
      initialDeposit: params.initialDeposit,
    };

    const instruction = await getSetGoalInstructionAsync(input, {
      programAddress: this.programAddress,
    });

    return await sendTransaction(this.client, params.owner, [instruction]);
  }

  async contributeToGoal(params: {
    contributor: TransactionSigner;
    networkConfig: Address;
    goalSlotId: bigint;
    depositAmount: bigint;
  }): Promise<string> {
    const goalAddress = await deriveGoalAddress(this.programAddress, params.networkConfig, params.goalSlotId);

    const input: ContributeToGoalAsyncInput = {
      contributor: params.contributor,
      goal: goalAddress,
      networkConfig: params.networkConfig,
      depositAmount: params.depositAmount,
    };

    const instruction = await getContributeToGoalInstructionAsync(input, {
      programAddress: this.programAddress,
    });

    return await sendTransaction(this.client, params.contributor, [instruction]);
  }

  async withdrawFromGoal(params: {
    contributor: TransactionSigner;
    networkConfig: Address;
    goalSlotId: bigint;
    sharesToBurn: bigint;
  }): Promise<string> {
    const goalAddress = await deriveGoalAddress(this.programAddress, params.networkConfig, params.goalSlotId);

    const input: WithdrawFromGoalAsyncInput = {
      contributor: params.contributor,
      goal: goalAddress,
      networkConfig: params.networkConfig,
      sharesToBurn: params.sharesToBurn,
    };

    const instruction = await getWithdrawFromGoalInstructionAsync(input, {
      programAddress: this.programAddress,
    });

    return await sendTransaction(this.client, params.contributor, [instruction]);
  }

  async updateNetworkConfig(params: {
    authority: TransactionSigner;
    cidConfig?: string | null;
    newCodeMeasurement?: CodeMeasurementArgs | null;
  }): Promise<string> {
    const networkConfigAddress = await deriveNetworkConfigAddress(
      this.programAddress,
      params.authority.address
    );

    const input: UpdateNetworkConfigAsyncInput = {
      authority: params.authority,
      networkConfig: networkConfigAddress,
      cidConfig: params.cidConfig ?? null,
      newCodeMeasurement: params.newCodeMeasurement ?? null,
    };

    const instruction = await getUpdateNetworkConfigInstructionAsync(input, {
      programAddress: this.programAddress,
    });

    return await sendTransaction(this.client, params.authority, [instruction]);
  }

  async activateNode(params: {
    authority: TransactionSigner;
    nodePubkey: Address;
  }): Promise<string> {
    const networkConfigAddress = await deriveNetworkConfigAddress(
      this.programAddress,
      params.authority.address
    );
    const nodeInfoAddress = await deriveNodeInfoAddress(
      this.programAddress,
      params.nodePubkey
    );

    const instruction = getActivateNodeInstruction(
      {
        authority: params.authority,
        networkConfig: networkConfigAddress,
        nodeInfo: nodeInfoAddress,
      },
      {
        programAddress: this.programAddress,
      }
    );

    return await sendTransaction(this.client, params.authority, [instruction]);
  }

  /**
   * Get all nodes filtered by status and/or type
   */
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
          offset: 72, // discriminator 8 + owner 32 + nodePubkey 32
          bytes: [params.nodeType],
        },
      });
    }

    if (params?.status !== undefined) {
      filters.push({
        memcmp: {
          offset: 73, // discriminator 8 + owner 32 + nodePubkey 32 + nodeType 1
          bytes: [params.status],
        },
      });
    }

    const response = await this.client.rpc
      .getProgramAccounts(this.programAddress, {
        encoding: 'base64',
        filters,
      })
      .send();

    return this.decodeAccountsFromResponse(response, decodeNodeInfo);
  }

  /**
   * Get all agents filtered by status
   */
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
          offset: 48, // discriminator 8 + agentSlotId 8 + owner 32
          bytes: [status],
        },
      });
    }

    const response = await this.client.rpc
      .getProgramAccounts(this.programAddress, {
        encoding: 'base64',
        filters,
      })
      .send();

    return this.decodeAccountsFromResponse(response, decodeAgent);
  }

  /**
   * Get all tasks filtered by status
   */
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
          offset: 49, // discriminator 8 + taskSlotId 8 + actionType 1 + agent 32
          bytes: [status],
        },
      });
    }

    const response = await this.client.rpc
      .getProgramAccounts(this.programAddress, {
        encoding: 'base64',
        filters,
      })
      .send();

    return this.decodeAccountsFromResponse(response, decodeTask);
  }

  /**
   * Get all goals filtered by status
   */
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
          offset: 112, // discriminator 8 + goalSlotId 8 + owner 32 + agent 32 + task 32
          bytes: [status],
        },
      });
    }

    const response = await this.client.rpc
      .getProgramAccounts(this.programAddress, {
        encoding: 'base64',
        filters,
      })
      .send();

    return this.decodeAccountsFromResponse(response, decodeGoal);
  }

  /**
   * Generic helper to wait for accounts to reach a specific status using WebSocket subscriptions
   */
  private async waitForStatus<T, TStatus>(
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
      initialFoundItems?: Map<Address, T>; // Pre-populated found items (e.g., from manual initial check)
    }
  ): Promise<T[]> {
    const waitMode = options?.waitMode ?? WaitMode.All;
    const foundItems = config.initialFoundItems ? new Map(config.initialFoundItems) : new Map<Address, T>();
    
    // First, check current state immediately (skip if initialFoundItems provided)
    if (!config.initialFoundItems) {
      const initialItems = await config.getByStatus(targetStatus);
      
      for (const item of initialItems) {
        const key = config.getKey(item);
        if (key && targetKeys.includes(key)) {
          foundItems.set(key, item);
        }
      }
      
      // If waitMode is 'first' and we found at least one item, return immediately
      if (waitMode === WaitMode.First && foundItems.size > 0) {
        return Array.from(foundItems.values());
      }

      // If waitMode is 'all' and all items are already in target status, return immediately
      if (waitMode === WaitMode.All && foundItems.size === targetKeys.length) {
        return Array.from(foundItems.values());
      }
    } else {
      // If initialFoundItems provided, check if we can return early
      if (waitMode === WaitMode.First && foundItems.size > 0) {
        return Array.from(foundItems.values());
      }
      if (waitMode === WaitMode.All && foundItems.size === targetKeys.length) {
        return Array.from(foundItems.values());
      }
    }

    // Set up abort signal (only if timeout is provided)
    const abortController = new AbortController();
    let timeoutId: NodeJS.Timeout | null = null;
    
    if (options?.timeoutMs !== undefined) {
      timeoutId = setTimeout(() => {
        abortController.abort();
      }, options.timeoutMs);
    }

    try {
      // Build filters for the subscription
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

      // Subscribe to program account changes
      const notifications = await this.client.rpcSubscriptions
        .programNotifications(this.programAddress, {
          encoding: 'base64',
          filters,
          commitment: 'confirmed',
        })
        .subscribe({ abortSignal: abortController.signal });

      // Listen for notifications
      for await (const notification of notifications) {
        try {
          const accountInfo = notification.value.account;
          const encodedAccount = {
            address: notification.value.pubkey,
            data: accountInfo.data,
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
          
          // Check if this is one of the items we're waiting for
          // For nodes: match by nodePubkey field
          // For agents: match by account address (itemKey will be empty string, use accountKey)
          const key = itemKey || accountKey;
          const matches = targetKeys.includes(key);
          
          if (matches) {
            foundItems.set(key, item);

            // If waitMode is 'first', return as soon as we find one item
            if (waitMode === WaitMode.First) {
              if (timeoutId) {
                clearTimeout(timeoutId);
              }
              abortController.abort(); // Unsubscribe
              return Array.from(foundItems.values());
            }

            // If waitMode is 'all', check if we've found all items
            if (waitMode === WaitMode.All && foundItems.size === targetKeys.length) {
              if (timeoutId) {
                clearTimeout(timeoutId);
              }
              abortController.abort(); // Unsubscribe
              return Array.from(foundItems.values());
            }
          }
        } catch (error) {
          // Skip invalid accounts
        }
      }

      // If we exit the loop without finding items, throw timeout (only if timeout was set)
      if (options?.timeoutMs !== undefined) {
        const expectedCount = waitMode === WaitMode.First ? 1 : targetKeys.length;
        throw new Error(
          `Timeout waiting for ${expectedCount} ${config.entityName}(s) to reach status ${targetStatus}`
        );
      }
      
      // If no timeout, this shouldn't happen, but handle gracefully
      throw new Error(
        `Subscription ended unexpectedly while waiting for ${config.entityName}s to reach status ${targetStatus}`
      );
    } catch (error: any) {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      
      // If aborted due to timeout, throw timeout error
      if (error.name === 'AbortError' || abortController.signal.aborted) {
        if (options?.timeoutMs !== undefined) {
          const expectedCount = waitMode === WaitMode.First ? 1 : targetKeys.length;
          throw new Error(
            `Timeout waiting for ${expectedCount} ${config.entityName}(s) to reach status ${targetStatus}`
          );
        }
        // If no timeout was set but we got aborted, it might be a manual abort
        throw new Error(
          `Subscription aborted while waiting for ${config.entityName}s to reach status ${targetStatus}`
        );
      }
      
      // Otherwise, rethrow the original error
      throw error;
    }
  }

  /**
   * Wait for nodes to reach a specific status using WebSocket subscriptions
   * This is event-driven and more efficient than polling
   * 
   * @param nodePubkeys - Array of node public keys to wait for
   * @param targetStatus - The status to wait for
   * @param options - Optional configuration
   * @param options.timeoutMs - Optional timeout in milliseconds. If not provided, will wait indefinitely
   * @param options.waitMode - Wait mode: 'all' to wait for all nodes, 'first' to return on first node. Default: 'all'
   * @returns Promise that resolves with node(s) that reached the target status
   *   - If waitMode is 'all': returns array of all nodes
   *   - If waitMode is 'first': returns array with single node (the first one that reached the status)
   */
  async waitForNodesStatus(
    nodePubkeys: Address[],
    targetStatus: NodeStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ): Promise<NodeInfo[]> {
    return this.waitForStatus(
      nodePubkeys,
      targetStatus,
      options,
      {
        getByStatus: (status) => this.getNodesByStatus({ status }),
        discriminator: NODE_INFO_DISCRIMINATOR,
        statusOffset: 73, // discriminator 8 + owner 32 + nodePubkey 32 + nodeType 1
        decode: decodeNodeInfo,
        getKey: (node) => node.nodePubkey,
        getAccountKey: (notification) => notification.value.pubkey,
        entityName: 'node',
      }
    );
  }

  /**
   * Wait for agents to reach a specific status using WebSocket subscriptions
   * This is event-driven and more efficient than polling
   * 
   * @param agentAddresses - Array of agent addresses to wait for
   * @param targetStatus - The status to wait for
   * @param options - Optional configuration
   * @param options.timeoutMs - Optional timeout in milliseconds. If not provided, will wait indefinitely
   * @param options.waitMode - Wait mode: 'all' to wait for all agents, 'first' to return on first agent. Default: 'all'
   * @returns Promise that resolves with agent(s) that reached the target status
   *   - If waitMode is 'all': returns array of all agents
   *   - If waitMode is 'first': returns array with single agent (the first one that reached the status)
   */
  async waitForAgentsStatus(
    agentAddresses: Address[],
    targetStatus: AgentStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ): Promise<Agent[]> {
    const waitMode = options?.waitMode ?? WaitMode.All;
    
    // For agents, check each address individually since we match by address, not by a field
    const foundAgents = new Map<Address, Agent>();
    for (const agentAddress of agentAddresses) {
      const agent = await this.getAgent(agentAddress);
      if (agent && agent.status === targetStatus) {
        foundAgents.set(agentAddress, agent);
      }
    }
    
    // If waitMode is 'first' and we found at least one agent, return immediately
    if (waitMode === WaitMode.First && foundAgents.size > 0) {
      return Array.from(foundAgents.values());
    }

    // If waitMode is 'all' and all agents are already in target status, return immediately
    if (waitMode === WaitMode.All && foundAgents.size === agentAddresses.length) {
      return Array.from(foundAgents.values());
    }

    // Use the generic helper for subscription-based waiting (pass initial found items)
    return this.waitForStatus(
      agentAddresses,
      targetStatus,
      options,
      {
        getByStatus: (status) => this.getAgentsByStatus(status),
        discriminator: AGENT_DISCRIMINATOR,
        statusOffset: 48, // discriminator 8 + agentSlotId 8 + owner 32
        decode: decodeAgent,
        getKey: (_agent) => '' as Address, // Agents are matched by account address, not a field
        getAccountKey: (notification) => notification.value.pubkey,
        entityName: 'agent',
        initialFoundItems: foundAgents, // Pass the manually found agents
      }
    );
  }
}
