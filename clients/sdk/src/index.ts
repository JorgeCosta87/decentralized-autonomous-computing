// Main SDK exports
export { DacSDK, WaitMode } from './dacSDK.js';
export { IpfsClient } from './ipfsClient.js';
export type { IpfsClientOptions } from './ipfsClient.js';

// Types
// Export Address - use @solana/kit's Address type for compatibility with generated code
export type { Address } from '@solana/kit';
export type {
  NetworkConfig,
  Agent,
  Goal,
  Contribution,
  NodeInfo,
  Task,
} from './generated/dac/accounts/index.js';
export type { CodeMeasurement, CodeMeasurementArgs } from './generated/dac/types/codeMeasurement.js';
export { NodeType, NodeStatus, AgentStatus, TaskStatus, GoalStatus } from './generated/dac/types/index.js';
export { DAC_PROGRAM_ID, deriveNetworkConfigAddress, deriveAgentAddress, deriveGoalAddress, deriveTaskAddress, deriveContributionAddress, deriveGoalVaultAddress } from './dac/dacPdas.js';
export { getNodeStatusName, getAgentStatusName, getTaskStatusName, getGoalStatusName } from './dac/statusUtils.js';

// Instruction builders (for useWalletUiSignAndSend)
export { getInitializeNetworkInstruction } from './generated/dac/instructions/index.js';
export type { InitializeNetworkInput } from './generated/dac/instructions/index.js';

// Config types and utilities
export type {
  NetworkConfig as ConfigNetworkConfig,
  NodeConfig,
  ToolsConfig,
  AgentConfig,
  GoalSpecification,
  ConfigSchema,
  ToolConfig,
  ToolArg,
} from './dac/configTypes.js';
export { ConfigService } from './dac/configService.js';

// Transaction signer interface
export type { TransactionSigner } from './dac/utils.js';

// Subscription types
export type {
  GoalEvent,
  TaskClaimedEvent,
  TaskResultSubmittedEvent,
  TaskValidationSubmittedEvent,
  FetchHistoricalEventsOptions,
} from './dac/dacSubscriptions.js';