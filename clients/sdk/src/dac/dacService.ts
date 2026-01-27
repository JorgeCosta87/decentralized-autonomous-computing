import type { Address, Rpc, TransactionMessage, TransactionMessageWithFeePayer, TransactionMessageWithBlockhashLifetime } from '@solana/kit';
import type {
  NetworkConfig,
  Agent,
  Session,
  Contribution,
  NodeInfo,
  Task,
} from '../generated/dac/accounts/index.js';
import type { NodeStatus, AgentStatus, TaskStatus, SessionStatus, NodeType, CodeMeasurementArgs } from '../generated/dac/types/index.js';

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
  getSession(networkConfig: Address, sessionSlotId: bigint): Promise<Session | null>;
  getTask(networkConfig: Address, taskSlotId: bigint): Promise<Task | null>;
  getContribution(session: Address, contributor: Address): Promise<Contribution | null>;
  getNodeInfo(nodePubkey: Address): Promise<NodeInfo | null>;
  getNodesByStatus(params?: { status?: NodeStatus; nodeType?: NodeType }): Promise<NodeInfo[]>;
  getAgentsByStatus(status?: AgentStatus): Promise<Agent[]>;
  getTasksByStatus(status?: TaskStatus): Promise<Task[]>;
  getSessionsByStatus(status?: SessionStatus): Promise<Session[]>;

  batchGetContributionsForSessions(
    networkConfig: Address,
    sessionSlotIds: bigint[],
    contributorAddress: Address
  ): Promise<Map<bigint, Contribution | null>>;
  batchGetVaultBalances(
    networkConfig: Address,
    sessionSlotIds: bigint[]
  ): Promise<Map<bigint, { balance: bigint; rentExempt: bigint }>>;
  getContributorsForSessions(
    networkConfig: Address,
    sessionSlotIds: bigint[]
  ): Promise<Map<bigint, { count: number; contributors: Array<{ address: Address; shares: bigint }> }>>;
}

/**
 * Transaction parameter types
 */
export type InitializeNetworkParams = {
  authority: TransactionSigner;
  cidConfig: string;
  /** Number of task PDAs to pre-allocate; no sessions/goals are pre-allocated on init. */
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

export type CreateSessionParams = {
  payer: TransactionSigner;
  owner: TransactionSigner;
  networkConfig: Address;
  isOwned: boolean;
  isConfidential: boolean;
};

export type SetSessionParams = {
  owner: TransactionSigner;
  networkConfig: Address;
  sessionSlotId: bigint;
  taskSlotId: bigint;
  agentSlotId: bigint;
  specificationCid: string;
  maxIterations: bigint;
  initialDeposit: bigint;
  /** Compute node pubkey to assign to the task. */
  computeNode: Address;
  /** Task type (e.g. Completion(model_id), Custom(module_id), HumanInLoop). */
  taskType: { type: 'Completion'; modelId: bigint } | { type: 'Custom'; moduleId: bigint } | { type: 'HumanInLoop' };
};

export type ContributeToSessionParams = {
  contributor: TransactionSigner;
  networkConfig: Address;
  sessionSlotId: bigint;
  depositAmount: bigint;
};

export type WithdrawFromSessionParams = {
  contributor: TransactionSigner;
  networkConfig: Address;
  sessionSlotId: bigint;
  sharesToBurn: bigint;
};

export type SubmitTaskParams = {
  owner: TransactionSigner;
  networkConfig: Address;
  sessionSlotId: bigint;
  taskSlotId: bigint;
  inputCid: string;
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
  createSession(params: CreateSessionParams): Promise<{ transactionMessage: TransactionMessageType; sessionAddress: Address; sessionSlotId: bigint; taskAddress: Address; taskSlotId: bigint }>;
  setSession(params: SetSessionParams): Promise<TransactionMessageType>;
  contributeToSession(params: ContributeToSessionParams): Promise<TransactionMessageType>;
  withdrawFromSession(params: WithdrawFromSessionParams): Promise<TransactionMessageType>;
  submitTask(params: SubmitTaskParams): Promise<TransactionMessageType>;
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
  
  waitForSessionsStatus(
    sessionAddresses: Address[],
    targetStatus: SessionStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ): Promise<Session[]>;

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
  subscribeToSessionEvents(
    networkConfig: Address,
    sessionSlotId: bigint,
    callback: (event: import('./dacSubscriptions.js').SessionEvent) => void
  ): Promise<() => void>;
  subscribeToNetworkEvents(
    callback: (event: import('./dacSubscriptions.js').SessionEvent) => void
  ): Promise<() => void>;
  fetchHistoricalEvents(
    networkConfig: Address,
    sessionSlotId: bigint,
    options?: import('./dacSubscriptions.js').FetchHistoricalEventsOptions
  ): Promise<import('./dacSubscriptions.js').SessionEvent[]>;
  fetchNetworkHistoricalEvents(
    options?: import('./dacSubscriptions.js').FetchHistoricalEventsOptions
  ): Promise<import('./dacSubscriptions.js').SessionEvent[]>;

}
