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
  fetchMaybeContribution,
  fetchMaybeNodeInfo,
  type NetworkConfig,
  type Agent,
  type Goal,
  type Contribution,
  type NodeInfo,
} from './generated/dac/accounts/index.js';
import type { CodeMeasurementArgs } from './generated/dac/types/index.js';

/**
 * Client for interacting with the DAC (Decentralized Autonomous Computing) program.
 * 
 * This client provides methods for frontend/UI operations only. Node operations (like
 * claimTask, submitTaskResult, etc.) are handled by separate node clients.
 * 
 * @example
 * ```typescript
 * import { createSolanaClient } from 'gill';
 * import { DacFrontendClient } from './dacFrontendClient';
 * 
 * const client = createSolanaClient('https://api.mainnet-beta.solana.com');
 * const dacClient = new DacFrontendClient(client);
 * 
 * // Create an agent
 * const { signature, agentAddress } = await dacClient.createAgent({
 *   agentOwner: myKeypair,
 *   networkConfig: networkConfigAddress,
 *   agentConfigCid: 'QmXXX...'
 * });
 * ```
 */
export class DacFrontendClient {
  private authority: Address | null = null;

  constructor(
    private readonly client: SolanaClient,
    private readonly programAddress: Address = DAC_PROGRAM_ID
  ) {}

  setAuthority(authority: Address): void {
    this.authority = authority;
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

  async getAgent(networkConfig: Address, agentSlotId: bigint): Promise<Agent | null> {
    const agentAddress = await deriveAgentAddress(this.programAddress, networkConfig, agentSlotId);
    const account = await fetchMaybeAgent(this.client.rpc, agentAddress);
    return account.exists ? account.data : null;
  }

  async getGoal(networkConfig: Address, goalSlotId: bigint): Promise<Goal | null> {
    const goalAddress = await deriveGoalAddress(this.programAddress, networkConfig, goalSlotId);
    const account = await fetchMaybeGoal(this.client.rpc, goalAddress);
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
    isPublic: boolean;
  }): Promise<{ signature: string; goalAddress: Address; goalSlotId: bigint }> {
    const networkConfigData = await this.getNetworkConfig();
    if (!networkConfigData) {
      throw new Error('Network config not found');
    }

    const goalSlotId = networkConfigData.goalCount;
    const goalAddress = await deriveGoalAddress(this.programAddress, params.networkConfig, goalSlotId);

    const input: CreateGoalInput = {
      payer: params.payer,
      owner: params.owner,
      networkConfig: params.networkConfig,
      goal: goalAddress,
      isPublic: params.isPublic,
    };

    const instruction = getCreateGoalInstruction(input, {
      programAddress: this.programAddress,
    });

    const signature = await sendTransaction(this.client, params.payer, [instruction]);

    return { signature, goalAddress, goalSlotId };
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
}
