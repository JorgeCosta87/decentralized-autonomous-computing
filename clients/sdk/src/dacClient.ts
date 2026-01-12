import type { Address, SolanaClient } from 'gill';
import { DAC_PROGRAM_ID } from './dac/dacPdas.js';
import { createQueryService } from './dac/dacQueries.js';
import { createTransactionService } from './dac/dacTransactions.js';
import { createMonitoringService, WaitMode } from './dac/dacMonitoring.js';
import type {
  IQueryService,
  ITransactionService,
  IMonitoringService,
  DacServiceDeps,
  InitializeNetworkParams,
  RegisterNodeParams,
  CreateAgentParams,
  CreateGoalParams,
  SetGoalParams,
  ContributeToGoalParams,
  WithdrawFromGoalParams,
  UpdateNetworkConfigParams,
  ActivateNodeParams,
} from './dac/dacService.js';
import type { NodeStatus, AgentStatus, TaskStatus, GoalStatus, NodeType } from './generated/dac/types/index.js';

export { WaitMode };

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
  private readonly queryService: IQueryService;
  private readonly transactionService: ITransactionService;
  private readonly monitoringService: IMonitoringService;

  constructor(
    private readonly client: SolanaClient,
    private readonly programAddress: Address = DAC_PROGRAM_ID
  ) {
    const deps: DacServiceDeps = {
      client: this.client,
      programAddress: this.programAddress,
      getAuthority: () => this.authority,
    };

    this.queryService = createQueryService(deps);
    this.transactionService = createTransactionService(deps);
    this.monitoringService = createMonitoringService(deps, this.queryService);
  }

  setAuthority(authority: Address): void {
    this.authority = authority;
  }

  // Query methods
  getNetworkConfig = (authority?: Address) => this.queryService.getNetworkConfig(authority);
  getAgent = (agentAddress: Address) => this.queryService.getAgent(agentAddress);
  getAgentBySlot = (networkConfig: Address, agentSlotId: bigint) => 
    this.queryService.getAgentBySlot(networkConfig, agentSlotId);
  getGoal = (networkConfig: Address, goalSlotId: bigint) => 
    this.queryService.getGoal(networkConfig, goalSlotId);
  getTask = (networkConfig: Address, taskSlotId: bigint) => 
    this.queryService.getTask(networkConfig, taskSlotId);
  getContribution = (goal: Address, contributor: Address) => 
    this.queryService.getContribution(goal, contributor);
  getNodeInfo = (nodePubkey: Address) => this.queryService.getNodeInfo(nodePubkey);
  getNodesByStatus = (params?: { status?: NodeStatus; nodeType?: NodeType }) => 
    this.queryService.getNodesByStatus(params);
  getAgentsByStatus = (status?: AgentStatus) => 
    this.queryService.getAgentsByStatus(status);
  getTasksByStatus = (status?: TaskStatus) => 
    this.queryService.getTasksByStatus(status);
  getGoalsByStatus = (status?: GoalStatus) => 
    this.queryService.getGoalsByStatus(status);

  // Transaction methods
  initializeNetwork = (params: InitializeNetworkParams) => 
    this.transactionService.initializeNetwork(params);
  registerNode = (params: RegisterNodeParams) => 
    this.transactionService.registerNode(params);
  createAgent = (params: CreateAgentParams) => 
    this.transactionService.createAgent(params);
  createGoal = (params: CreateGoalParams) => 
    this.transactionService.createGoal(params);
  setGoal = (params: SetGoalParams) => 
    this.transactionService.setGoal(params);
  contributeToGoal = (params: ContributeToGoalParams) => 
    this.transactionService.contributeToGoal(params);
  withdrawFromGoal = (params: WithdrawFromGoalParams) => 
    this.transactionService.withdrawFromGoal(params);
  updateNetworkConfig = (params: UpdateNetworkConfigParams) => 
    this.transactionService.updateNetworkConfig(params);
  activateNode = (params: ActivateNodeParams) => 
    this.transactionService.activateNode(params);

  // Monitoring methods
  waitForNodesStatus = (
    nodePubkeys: Address[],
    targetStatus: NodeStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ) => this.monitoringService.waitForNodesStatus(nodePubkeys, targetStatus, options);
  
  waitForAgentsStatus = (
    agentAddresses: Address[],
    targetStatus: AgentStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ) => this.monitoringService.waitForAgentsStatus(agentAddresses, targetStatus, options);
  
  waitForGoalsStatus = (
    goalAddresses: Address[],
    targetStatus: GoalStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ) => this.monitoringService.waitForGoalsStatus(goalAddresses, targetStatus, options);
  
  waitForTasksStatus = (
    taskAddresses: Address[],
    targetStatus: TaskStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ) => this.monitoringService.waitForTasksStatus(taskAddresses, targetStatus, options);
}
