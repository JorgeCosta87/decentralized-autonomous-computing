import type { Address, Rpc, RpcSubscriptions } from '@solana/kit';
import { createSolanaRpc, createSolanaRpcSubscriptions } from '@solana/kit';
import { DAC_PROGRAM_ID } from './dac/dacPdas.js';
import { createQueryService } from './dac/dacQueries.js';
import { createTransactionService } from './dac/dacTransactions.js';
import { createMonitoringService, WaitMode } from './dac/dacMonitoring.js';
import { createSubscriptionService } from './dac/dacSubscriptions.js';
import { safeStringify } from './dac/utils.js';
import type {
  IQueryService,
  ITransactionService,
  IMonitoringService,
  ISubscriptionService,
  DacServiceDeps,
  InitializeNetworkParams,
  RegisterNodeParams,
  CreateAgentParams,
  CreateSessionParams,
  SetSessionParams,
  ContributeToSessionParams,
  WithdrawFromSessionParams,
  UpdateNetworkConfigParams,
  ActivateNodeParams,
  SubmitTaskParams,
} from './dac/dacService.js';
import type { SessionEvent, FetchHistoricalEventsOptions } from './dac/dacSubscriptions.js';
import type { NodeStatus, AgentStatus, TaskStatus, SessionStatus, NodeType } from './generated/dac/types/index.js';
import { ConfigService } from './dac/configService.js';
import type {
  NetworkConfig,
  NodeConfig,
  ToolsConfig,
  AgentConfig,
  GoalSpecification,
  ConfigSchema,
} from './dac/configTypes.js';

export { WaitMode };

/**
 * Client for interacting with the DAC (Decentralized Autonomous Computing) program.
 * 
 * This client provides methods for frontend/UI operations only. Node operations (like
 * claimTask, submitTaskResult, etc.) are handled by separate node clients.
 * 
 * @example
 * ```typescript
 * import { createSolanaRpc } from '@solana/kit';
 * import { DacSDK } from './dacSDK';
 * 
 * const rpc = createSolanaRpc('https://api.mainnet-beta.solana.com');
 * const dacClient = new DacSDK(rpc);
 * 
 * // Initialize network (only tasks are pre-allocated; no sessions)
 * const { signature, networkConfigAddress } = await dacClient.initializeNetwork({
 *   authority: myKeypair,
 *   cidConfig: 'QmNetworkConfig...',
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
 * // Create a session (public or confidential)
 * const { signature, sessionAddress } = await dacClient.createSession({
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
  private readonly subscriptionService: ISubscriptionService | null = null;
  private readonly rpc: Rpc<any>;
  private readonly rpcSubscriptions: RpcSubscriptions<any> | null = null;

  constructor(
    rpcUrlOrRpc: string | Rpc<any>,
    private readonly programAddress: Address = DAC_PROGRAM_ID,
    rpcSubscriptionsUrl?: string
  ) {
    const rpc = typeof rpcUrlOrRpc === 'string' 
      ? createSolanaRpc(rpcUrlOrRpc)
      : rpcUrlOrRpc;
    
    this.rpc = rpc;
    
    // Create RPC subscriptions if WebSocket URL provided
    if (rpcSubscriptionsUrl) {
      this.rpcSubscriptions = createSolanaRpcSubscriptions(rpcSubscriptionsUrl);
    }
    
    const deps: DacServiceDeps = {
      rpc,
      programAddress: this.programAddress,
      getAuthority: () => this.authority,
    };

    // Note: createQueryService is now async, but we'll handle it synchronously
    // by creating the RPC client lazily in the service
    this.queryService = createQueryService(deps) as any;
    this.transactionService = createTransactionService(deps);
    this.monitoringService = createMonitoringService(deps, this.queryService);
    
    // Create subscription service if RPC subscriptions available
    if (this.rpcSubscriptions) {
      this.subscriptionService = createSubscriptionService({
        ...deps,
        rpcSubscriptions: this.rpcSubscriptions,
      });
    }
  }

  setAuthority(authority: Address): void {
    this.authority = authority;
  }

  // Query methods
  getNetworkConfig = (authority?: Address) => this.queryService.getNetworkConfig(authority);
  getAgent = (agentAddress: Address) => this.queryService.getAgent(agentAddress);
  getAgentBySlot = (networkConfig: Address, agentSlotId: bigint) => 
    this.queryService.getAgentBySlot(networkConfig, agentSlotId);
  getSession = (networkConfig: Address, sessionSlotId: bigint) => 
    this.queryService.getSession(networkConfig, sessionSlotId);
  getTask = (networkConfig: Address, taskSlotId: bigint) => 
    this.queryService.getTask(networkConfig, taskSlotId);
  getContribution = (session: Address, contributor: Address) => 
    this.queryService.getContribution(session, contributor);
  getNodeInfo = (nodePubkey: Address) => this.queryService.getNodeInfo(nodePubkey);
  getNodesByStatus = (params?: { status?: NodeStatus; nodeType?: NodeType }) => 
    this.queryService.getNodesByStatus(params);
  getAgentsByStatus = (status?: AgentStatus) => 
    this.queryService.getAgentsByStatus(status);
  getTasksByStatus = (status?: TaskStatus) => 
    this.queryService.getTasksByStatus(status);
  getSessionsByStatus = (status?: SessionStatus) =>
    this.queryService.getSessionsByStatus(status);

  // Batch methods for efficient bulk operations
  batchGetContributionsForSessions = (
    networkConfig: Address,
    sessionSlotIds: bigint[],
    contributorAddress: Address
  ) => this.queryService.batchGetContributionsForSessions(networkConfig, sessionSlotIds, contributorAddress);

  batchGetVaultBalances = (
    networkConfig: Address,
    sessionSlotIds: bigint[]
  ) => this.queryService.batchGetVaultBalances(networkConfig, sessionSlotIds);

  getContributorsForSessions = (
    networkConfig: Address,
    sessionSlotIds: bigint[]
  ) => this.queryService.getContributorsForSessions(networkConfig, sessionSlotIds);

  // Transaction methods
  // These methods build the transaction, sign it, and send it
  // They maintain backward compatibility by returning signatures
  async initializeNetwork(params: InitializeNetworkParams) {
    const { transactionMessage, networkConfigAddress } = await this.transactionService.initializeNetwork(params);
    const signature = await this.signAndSendTransaction(transactionMessage);
    return { signature, networkConfigAddress };
  }

  async registerNode(params: RegisterNodeParams) {
    const { transactionMessage, nodeInfoAddress, nodeTreasuryAddress } = await this.transactionService.registerNode(params);
    const signature = await this.signAndSendTransaction(transactionMessage);
    return { signature, nodeInfoAddress, nodeTreasuryAddress };
  }

  async createAgent(params: CreateAgentParams) {
    const { transactionMessage, agentAddress, agentSlotId } = await this.transactionService.createAgent(params);
    const signature = await this.signAndSendTransaction(transactionMessage);
    return { signature, agentAddress, agentSlotId };
  }

  async createSession(params: CreateSessionParams) {
    const { transactionMessage, sessionAddress, sessionSlotId, taskAddress, taskSlotId } = await this.transactionService.createSession(params);
    const signature = await this.signAndSendTransaction(transactionMessage);
    return { signature, sessionAddress, sessionSlotId, taskAddress, taskSlotId };
  }

  async setSession(params: SetSessionParams) {
    const transactionMessage = await this.transactionService.setSession(params);
    return await this.signAndSendTransaction(transactionMessage);
  }

  async contributeToSession(params: ContributeToSessionParams) {
    const transactionMessage = await this.transactionService.contributeToSession(params);
    return await this.signAndSendTransaction(transactionMessage);
  }

  async withdrawFromSession(params: WithdrawFromSessionParams) {
    const transactionMessage = await this.transactionService.withdrawFromSession(params);
    return await this.signAndSendTransaction(transactionMessage);
  }

  async updateNetworkConfig(params: UpdateNetworkConfigParams) {
    const transactionMessage = await this.transactionService.updateNetworkConfig(params);
    return await this.signAndSendTransaction(transactionMessage);
  }

  async activateNode(params: ActivateNodeParams) {
    const transactionMessage = await this.transactionService.activateNode(params);
    return await this.signAndSendTransaction(transactionMessage);
  }

  async submitTask(params: SubmitTaskParams) {
    const transactionMessage = await this.transactionService.submitTask(params);
    return await this.signAndSendTransaction(transactionMessage);
  }

  /**
   * Sign and encode a transaction message
   */
  private async signAndEncodeTransaction(transactionMessage: any): Promise<{
    signature: string;
    base64Encoded: string;
  }> {
    const { signTransactionMessageWithSigners, getTransactionEncoder, getSignatureFromTransaction } = await import('@solana/kit');

    const signedTransaction = await signTransactionMessageWithSigners(transactionMessage);
    const signature = getSignatureFromTransaction(signedTransaction);
    
    if (!signature) {
      throw new Error('Failed to extract signature from signed transaction');
    }

    const transactionEncoder = getTransactionEncoder();
    const wireBytes = transactionEncoder.encode(signedTransaction as any);
    const base64Encoded = Buffer.from(wireBytes).toString('base64');

    return { signature, base64Encoded };
  }


  /**
   * Send transaction to the network
   */
  private async sendTransaction(base64Encoded: string, fallbackSignature: string): Promise<string> {
    try {
      const rpcResponse = await (this.rpc as any).sendTransaction(base64Encoded, {
        encoding: 'base64',
        skipPreflight: false,
        maxRetries: 3,
      }).send();

      const rpcSignature = rpcResponse?.value ?? rpcResponse;
      return (rpcSignature && typeof rpcSignature === 'string' && rpcSignature.length > 0) 
        ? rpcSignature 
        : fallbackSignature;
    } catch (error: any) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      const errorData = error?.data || error?.cause;
      
      console.error('[signAndSendTransaction] Failed to send transaction:', error);
      console.error('[signAndSendTransaction] Error data:', safeStringify(errorData));
      throw new Error(`Failed to send transaction: ${errorMessage}`);
    }
  }

  /**
   * Sign and send a transaction
   */
  private async signAndSendTransaction(transactionMessage: any): Promise<string> {
    const { signature, base64Encoded } = await this.signAndEncodeTransaction(transactionMessage);
    return await this.sendTransaction(base64Encoded, signature);
  }

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
  
  waitForSessionsStatus = (
    sessionAddresses: Address[],
    targetStatus: SessionStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ) => this.monitoringService.waitForSessionsStatus(sessionAddresses, targetStatus, options);

  waitForTasksStatus = (
    taskAddresses: Address[],
    targetStatus: TaskStatus,
    options?: { timeoutMs?: number; waitMode?: WaitMode }
  ) => this.monitoringService.waitForTasksStatus(taskAddresses, targetStatus, options);

  // Config methods - provide config schema and parsing utilities
  /**
   * Get default config schema with field type information
   */
  getConfigSchema = (): ConfigSchema => ConfigService.getDefaultSchema();

  /**
   * Parse network config from YAML content
   */
  parseNetworkConfig = (yamlContent: string): NetworkConfig =>
    ConfigService.parseNetworkConfig(yamlContent);

  /**
   * Parse node config from YAML content
   */
  parseNodeConfig = (yamlContent: string): NodeConfig =>
    ConfigService.parseNodeConfig(yamlContent);

  /**
   * Parse tools config from YAML content
   */
  parseToolsConfig = (yamlContent: string): ToolsConfig =>
    ConfigService.parseToolsConfig(yamlContent);

  /**
   * Parse agent config from YAML content
   */
  parseAgentConfig = (yamlContent: string): AgentConfig =>
    ConfigService.parseAgentConfig(yamlContent);

  /**
   * Parse goal specification from YAML content
   */
  parseGoalSpecification = (yamlContent: string): GoalSpecification =>
    ConfigService.parseGoalSpecification(yamlContent);

  /**
   * Check if a field can reference a file/IPFS CID
   */
  canFieldBeFile = (
    configType: 'network_config' | 'node_config' | 'tools_config' | 'agent_config' | 'goal_specification',
    fieldPath: string
  ): boolean => ConfigService.canFieldBeFile(configType, fieldPath);

  /**
   * Get allowed file types for a field
   */
  getAllowedFileTypes = (
    configType: 'agent_config' | 'goal_specification',
    fieldPath: string
  ): string[] => ConfigService.getAllowedFileTypes(configType, fieldPath);

  // Subscription methods
  /**
   * Subscribe to session events (requires RPC Subscriptions WebSocket URL in constructor)
   * Note: Local Solana validators don't support WebSocket subscriptions.
   * For localhost, use fetchHistoricalEvents with polling instead.
   */
  subscribeToSessionEvents = (
    networkConfig: Address,
    sessionSlotId: bigint,
    callback: (event: SessionEvent) => void
  ): Promise<() => void> => {
    if (!this.subscriptionService) {
      throw new Error(
        'RPC Subscriptions not available. WebSocket subscriptions are not supported by local Solana validators. ' +
        'Use fetchHistoricalEvents() with polling for localhost, or connect to a remote RPC that supports WebSocket.'
      );
    }
    return this.subscriptionService.subscribeToSessionEvents(networkConfig, sessionSlotId, callback);
  };

  /**
   * Subscribe to network-wide events (SessionSet, ContributionMade, SessionCompleted, AgentCreated)
   */
  subscribeToNetworkEvents = (
    callback: (event: SessionEvent) => void
  ): Promise<() => void> => {
    if (!this.subscriptionService) {
      throw new Error('Subscription service not available');
    }
    return this.subscriptionService.subscribeToNetworkEvents(callback);
  };

  /**
   * Fetch network-wide historical events
   */
  fetchNetworkHistoricalEvents = (
    options?: import('./dac/dacSubscriptions.js').FetchHistoricalEventsOptions
  ): Promise<SessionEvent[]> => {
    if (!this.subscriptionService) {
      throw new Error('Subscription service not available');
    }
    return this.subscriptionService.fetchNetworkHistoricalEvents(options);
  };

  /**
   * Fetch historical events for a session (and optionally a specific task)
   * This works even without WebSocket subscriptions - only requires RPC
   */
  fetchHistoricalEvents = async (
    networkConfig: Address,
    sessionSlotId: bigint,
    options?: FetchHistoricalEventsOptions
  ): Promise<SessionEvent[]> => {
    if (this.subscriptionService) {
      return this.subscriptionService.fetchHistoricalEvents(networkConfig, sessionSlotId, options);
    }
    
    const { createSubscriptionService } = await import('./dac/dacSubscriptions.js');
    const tempService = createSubscriptionService({
      rpc: this.rpc,
      programAddress: this.programAddress,
      getAuthority: () => this.authority,
      rpcSubscriptions: null,
    });
    return tempService.fetchHistoricalEvents(networkConfig, sessionSlotId, options);
  };
}
