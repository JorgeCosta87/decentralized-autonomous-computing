import type { Address, Rpc, TransactionMessage, TransactionMessageWithFeePayer, TransactionMessageWithBlockhashLifetime } from '@solana/kit';
import type {
  NetworkConfig,
  Agent,
  Goal,
  Contribution,
  NodeInfo,
  Task,
} from '../generated/dac/accounts/index.js';
import type { NodeStatus, AgentStatus, TaskStatus, GoalStatus, NodeType, CodeMeasurementArgs } from '../generated/dac/types/index.js';
import type { WaitMode } from './dacMonitoring.js';
import type { TransactionSigner } from './utils.js';

/**
 * Dependency container for DAC services
 */
export interface DacServiceDeps {
  rpc: Rpc<any>;
  programAddress: Address;
  getAuthority: () => Address | null;
}

/**
 * Query service interface for read operations
 */
export interface IQueryService {
  getNetworkConfig(authority?: Address): Promise<NetworkConfig | null>;
  getAgent(agentAddress: Address): Promise<Agent | null>;
  getAgentBySlot(networkConfig: Address, agentSlotId: bigint): Promise<Agent | null>;
  getGoal(networkConfig: Address, goalSlotId: bigint): Promise<Goal | null>;
  getTask(networkConfig: Address, taskSlotId: bigint): Promise<Task | null>;
  getContribution(goal: Address, contributor: Address): Promise<Contribution | null>;
  getNodeInfo(nodePubkey: Address): Promise<NodeInfo | null>;
  getNodesByStatus(params?: { status?: NodeStatus; nodeType?: NodeType }): Promise<NodeInfo[]>;
  getAgentsByStatus(status?: AgentStatus): Promise<Agent[]>;
  getTasksByStatus(status?: TaskStatus): Promise<Task[]>;
  getGoalsByStatus(status?: GoalStatus): Promise<Goal[]>;
  
  // Batch methods for efficient bulk operations
  batchGetContributionsForGoals(
    networkConfig: Address,
    goalSlotIds: bigint[],
    contributorAddress: Address
  ): Promise<Map<bigint, Contribution | null>>;
  
  batchGetVaultBalances(
    networkConfig: Address,
    goalSlotIds: bigint[]
  ): Promise<Map<bigint, { balance: bigint; rentExempt: bigint }>>;
  
  getContributorsForGoals(
    networkConfig: Address,
    goalSlotIds: bigint[]
  ): Promise<Map<bigint, { count: number; contributors: Array<{ address: Address; shares: bigint }> }>>;
}

/**
 * Transaction parameter types
 */
export type InitializeNetworkParams = {
  authority: TransactionSigner;
  cidConfig: string;
  allocateGoals: bigint;
  allocateTasks: bigint;
  approvedCodeMeasurements: CodeMeasurementArgs[];
  requiredValidations: number;
};

export type RegisterNodeParams = {
  owner: TransactionSigner;
  networkConfig: Address;
  nodePubkey: Address;
  nodeType: NodeType;
};

export type CreateAgentParams = {
  agentOwner: TransactionSigner;
  networkConfig: Address;
  agentConfigCid: string;
};

export type CreateGoalParams = {
  payer: TransactionSigner;
  owner: TransactionSigner;
  networkConfig: Address;
  isOwned: boolean;
  isConfidential: boolean;
};

export type SetGoalParams = {
  owner: TransactionSigner;
  networkConfig: Address;
  goalSlotId: bigint;
  agentSlotId: bigint;
  taskSlotId: bigint;
  specificationCid: string;
  maxIterations: bigint;
  initialDeposit: bigint;
};

export type ContributeToGoalParams = {
  contributor: TransactionSigner;
  networkConfig: Address;
  goalSlotId: bigint;
  depositAmount: bigint;
};

export type WithdrawFromGoalParams = {
  contributor: TransactionSigner;
  networkConfig: Address;
  goalSlotId: bigint;
  sharesToBurn: bigint;
};

export type UpdateNetworkConfigParams = {
  authority: TransactionSigner;
  cidConfig?: string | null;
  newCodeMeasurement?: CodeMeasurementArgs | null;
};

export type ActivateNodeParams = {
  authority: TransactionSigner;
  nodePubkey: Address;
};

/**
 * Transaction service interface for write operations
 */
type TransactionMessageType = TransactionMessage & TransactionMessageWithFeePayer<string> & TransactionMessageWithBlockhashLifetime;

export interface ITransactionService {
  initializeNetwork(params: InitializeNetworkParams): Promise<{ transactionMessage: TransactionMessageType; networkConfigAddress: Address }>;
  registerNode(params: RegisterNodeParams): Promise<{ transactionMessage: TransactionMessageType; nodeInfoAddress: Address; nodeTreasuryAddress: Address }>;
  createAgent(params: CreateAgentParams): Promise<{ transactionMessage: TransactionMessageType; agentAddress: Address; agentSlotId: bigint }>;
  createGoal(params: CreateGoalParams): Promise<{ transactionMessage: TransactionMessageType; goalAddress: Address; goalSlotId: bigint; taskAddress: Address; taskSlotId: bigint }>;
  setGoal(params: SetGoalParams): Promise<TransactionMessageType>;
  contributeToGoal(params: ContributeToGoalParams): Promise<TransactionMessageType>;
  withdrawFromGoal(params: WithdrawFromGoalParams): Promise<TransactionMessageType>;
  updateNetworkConfig(params: UpdateNetworkConfigParams): Promise<TransactionMessageType>;
  activateNode(params: ActivateNodeParams): Promise<TransactionMessageType>;
}

/**
 * Monitoring service interface for status monitoring
 */
export interface IMonitoringService {
  waitForNodesStatus(
    nodePubkeys: Address[],
    targetStatus: NodeStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ): Promise<NodeInfo[]>;
  
  waitForAgentsStatus(
    agentAddresses: Address[],
    targetStatus: AgentStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ): Promise<Agent[]>;
  
  waitForGoalsStatus(
    goalAddresses: Address[],
    targetStatus: GoalStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ): Promise<Goal[]>;
  
  waitForTasksStatus(
    taskAddresses: Address[],
    targetStatus: TaskStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ): Promise<Task[]>;
}

/**
 * Subscription service interface for real-time event subscriptions
 */
export interface ISubscriptionService {
  subscribeToGoalEvents(
    networkConfig: Address,
    goalSlotId: bigint,
    callback: (event: import('./dacSubscriptions.js').GoalEvent) => void
  ): Promise<() => void>;
  
  /**
   * Subscribe to network-wide events (all events, not filtered by goal)
   */
  subscribeToNetworkEvents(
    callback: (event: import('./dacSubscriptions.js').GoalEvent) => void
  ): Promise<() => void>;
  
  /**
   * Fetch historical events for a goal (and optionally a specific task)
   */
  fetchHistoricalEvents(
    networkConfig: Address,
    goalSlotId: bigint,
    options?: import('./dacSubscriptions.js').FetchHistoricalEventsOptions
  ): Promise<import('./dacSubscriptions.js').GoalEvent[]>;
  
  /**
   * Fetch network-wide historical events
   */
  fetchNetworkHistoricalEvents(
    options?: import('./dacSubscriptions.js').FetchHistoricalEventsOptions
  ): Promise<import('./dacSubscriptions.js').GoalEvent[]>;
}
