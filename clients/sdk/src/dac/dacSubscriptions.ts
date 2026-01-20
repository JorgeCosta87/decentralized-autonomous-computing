import type { Address, RpcSubscriptions } from '@solana/kit';
import { address, isSome, unwrapOption } from '@solana/kit';
import type { DacServiceDeps } from './dacService.js';
import { deriveGoalAddress, DAC_PROGRAM_ID } from './dacPdas.js';
import {
  getTaskClaimedDecoder,
  getTaskResultSubmittedDecoder,
  getTaskValidationSubmittedDecoder,
  getGoalSetDecoder,
  getContributionMadeDecoder,
  getGoalCompletedDecoder,
  getAgentCreatedDecoder,
  getNodeValidatedDecoder,
  getNodeRejectedDecoder,
} from '../generated/dac/types/index.js';

export interface GoalEvent {
  type: 'TaskClaimed' | 'TaskResultSubmitted' | 'TaskValidationSubmitted' | 'GoalSet' | 'ContributionMade' | 'GoalCompleted' | 'NodeValidated' | 'NodeRejected' | 'AgentCreated';
  goalSlotId?: bigint;
  taskSlotId?: bigint;
  timestamp: Date;
  signature?: string;
  data: TaskClaimedEvent | TaskResultSubmittedEvent | TaskValidationSubmittedEvent | GoalSetEvent | ContributionMadeEvent | GoalCompletedEvent | NodeValidatedEvent | NodeRejectedEvent | AgentCreatedEvent;
}

export interface TaskClaimedEvent {
  goalSlotId: bigint;
  taskSlotId: bigint;
  computeNode: Address;
  maxTaskCost: bigint;
}

export interface TaskResultSubmittedEvent {
  goalSlotId: bigint;
  taskSlotId: bigint;
  inputCid: string;
  outputCid: string;
  nextInputCid: string;
}

export interface TaskValidationSubmittedEvent {
  goalSlotId: bigint;
  taskSlotId: bigint;
  validator: Address;
  paymentAmount: bigint;
  approved: boolean;
  goalCompleted: boolean;
  currentIteration: bigint;
  vaultBalance: bigint;
  lockedForTasks: bigint;
}

export interface GoalSetEvent {
  goalSlotId: bigint;
  owner: Address;
  agentSlotId: bigint;
  taskSlotId: bigint;
  specificationCid: string;
  maxIterations: bigint;
  initialDeposit: bigint;
}

export interface ContributionMadeEvent {
  goalSlotId: bigint;
  contributor: Address;
  depositAmount: bigint;
  sharesMinted: bigint;
  totalShares: bigint;
}

export interface GoalCompletedEvent {
  goalSlotId: bigint;
  finalIteration: bigint;
  vaultBalance: bigint;
}

export interface NodeValidatedEvent {
  node: Address;
  validator: Address;
  goalSlotId?: bigint;
  taskSlotId?: bigint;
}

export interface NodeRejectedEvent {
  node: Address;
  validator: Address;
  goalSlotId?: bigint;
  taskSlotId?: bigint;
}

export interface AgentCreatedEvent {
  agentSlotId: bigint;
  owner: Address;
  agentConfigCid: string;
}

/**
 * Options for fetching historical events
 */
export interface FetchHistoricalEventsOptions {
  /** Optional task slot ID to filter events for a specific task */
  taskSlotId?: bigint;
  /** Maximum number of transactions to fetch (default: 100) */
  limit?: number;
  /** Optional signature to start fetching from (for pagination) */
  before?: string;
}

/**
 * Subscription service interface
 */
export interface ISubscriptionService {
  subscribeToGoalEvents(
    networkConfig: Address,
    goalSlotId: bigint,
    callback: (event: GoalEvent) => void
  ): Promise<() => void>;
  
  /**
   * Subscribe to network-wide events (all events, not filtered by goal)
   */
  subscribeToNetworkEvents(
    callback: (event: GoalEvent) => void
  ): Promise<() => void>;
  
  /**
   * Fetch historical events for a goal (and optionally a specific task)
   */
  fetchHistoricalEvents(
    networkConfig: Address,
    goalSlotId: bigint,
    options?: FetchHistoricalEventsOptions
  ): Promise<GoalEvent[]>;
  
  /**
   * Fetch network-wide historical events
   */
  fetchNetworkHistoricalEvents(
    options?: FetchHistoricalEventsOptions
  ): Promise<GoalEvent[]>;
}

/**
 * Create subscription service
 */
export function createSubscriptionService(
  deps: DacServiceDeps & { rpcSubscriptions?: RpcSubscriptions<any> | null }
): ISubscriptionService {
  const { rpcSubscriptions, programAddress } = deps;

  return {
    async subscribeToGoalEvents(
      networkConfig: Address,
      goalSlotId: bigint,
      callback: (event: GoalEvent) => void
    ): Promise<() => void> {
      if (!rpcSubscriptions) {
        throw new Error('RPC Subscriptions not available');
      }

      const abortController = new AbortController();

      try {
        // IMPORTANT: mentions only supports ONE pubkey per subscription.
        // For goal-specific realtime, we subscribe to the PROGRAM (not goal address),
        // then filter by decoded.goalSlotId in parseEventsFromLogs (which already does this).
        // This scales better than one subscription per goal.
        const rpcSubs = rpcSubscriptions as any;
        const logsIterable = await rpcSubs
          .logsNotifications({ mentions: [programAddress] })
          .subscribe({ abortSignal: abortController.signal });

        (async () => {
          try {
            for await (const notification of logsIterable) {
              if (abortController.signal.aborted) break;

              // Extract notification data (handle different notification shapes)
              const value = notification?.value ?? notification?.params?.result?.value ?? notification?.result?.value;
              const signature: string | undefined = value?.signature;
              const logs: string[] | undefined = value?.logs;
              const err = value?.err;

              if (!signature || !logs) continue;
              if (err) continue; // Skip failed transactions

              // Goal filter is applied inside parseEventsFromLogs via goalSlotId check
              // parseEventsFromLogs will only return events matching the goalSlotId
              const events = parseEventsFromLogs(logs, goalSlotId, signature);
              
              // Filter to only include task events (exclude NodeValidated, NodeRejected, etc.)
              for (const event of events) {
                if (
                  event.type === 'TaskClaimed' ||
                  event.type === 'TaskResultSubmitted' ||
                  event.type === 'TaskValidationSubmitted'
                ) {
                  callback(event);
                }
              }
            }
          } catch (e) {
            if (!abortController.signal.aborted) {
              console.error('[subscribeToGoalEvents] logsNotifications error:', e);
            }
          }
        })();

        return async () => {
          abortController.abort();
        };
      } catch (error) {
        throw new Error(`Failed to subscribe to goal events: ${error}`);
      }
    },
    
    async fetchHistoricalEvents(
      networkConfig: Address,
      goalSlotId: bigint,
      options: FetchHistoricalEventsOptions = {}
    ): Promise<GoalEvent[]> {
      const { taskSlotId, limit = 1000, before } = options; // Increased default limit for tracking all iterations
      const goalAddress = await deriveGoalAddress(programAddress, networkConfig, goalSlotId);
      const events: GoalEvent[] = [];

      try {
        
        // Fetch transaction signatures for the goal account
        // Programs don't have signatures directly - transactions are signed by users
        // So we need to get transactions that involve the goal account
        const signatureOptions: any = {
          limit: Math.min(limit * 2, 500), // Increased cap to fetch more signatures for many iterations
        };
        if (before) {
          signatureOptions.before = before;
        }

        // Get signatures for the goal account (this should work on local validators)
        let signatures: any[] = [];
        try {
          signatures = await (deps.rpc as any).getSignaturesForAddress(goalAddress, signatureOptions).send();
        } catch (error) {
          console.error('[fetchHistoricalEvents] Failed to get goal account transactions:', error);
          // Also try the goal vault address as a fallback
          try {
            const { deriveGoalVaultAddress } = await import('./dacPdas.js');
            const vaultAddress = await deriveGoalVaultAddress(programAddress, goalAddress);
            signatures = await (deps.rpc as any).getSignaturesForAddress(vaultAddress, signatureOptions).send();
          } catch (vaultError) {
            console.error('[fetchHistoricalEvents] Failed to get vault transactions:', vaultError);
          }
        }

        if (!signatures || signatures.length === 0) {
          console.log('[fetchHistoricalEvents] No signatures found');
          return events;
        }
        
        console.log(`[fetchHistoricalEvents] Found ${signatures.length} signatures, processing ${Math.min(limit * 2, 100)}`);

        // Batch fetch transactions in parallel instead of sequentially
        const processedSignatures = new Set<string>();
        const signatureList = (signatures || [])
          .filter((sig: any) => sig.signature && !processedSignatures.has(sig.signature))
          .slice(0, Math.min(limit * 2, 100)) // Limit to reasonable number, cap at 100
          .map((sig: any) => {
            processedSignatures.add(sig.signature!);
            return sig.signature!;
          });

        // Fetch all transactions in parallel (batched)
        const transactionPromises = signatureList.map(async (signature: string) => {
          try {
            const tx = await (deps.rpc as any).getTransaction(signature, {
              commitment: 'confirmed',
              maxSupportedTransactionVersion: 0,
            }).send();

            if (!tx?.meta?.logMessages) return [];

            // Parse events from transactions that have Program data
            const logsStr = tx.meta.logMessages.join('\n');
            if (!logsStr.includes('Program data:')) return [];

            // Get transaction timestamp from blockTime (Unix timestamp in seconds)
            // blockTime might be BigInt, so convert to number first
            const blockTime = tx.blockTime 
              ? new Date(Number(tx.blockTime) * 1000) 
              : new Date();

            const parsedEvents = parseEventsFromLogs(
              tx.meta.logMessages,
              goalSlotId,
              signature,
              blockTime
            );

            // Filter to only include task events (exclude NodeValidated, NodeRejected, etc.)
            // and filter by taskSlotId if specified
            return parsedEvents.filter(event => {
              // Only include task events
              if (event.type !== 'TaskClaimed' && 
                  event.type !== 'TaskResultSubmitted' && 
                  event.type !== 'TaskValidationSubmitted') {
                return false;
              }
              // Filter by taskSlotId if specified
              if (taskSlotId !== undefined && event.taskSlotId !== taskSlotId) {
                return false;
              }
              return true;
            });
          } catch (error) {
            // Skip errors for individual transactions
            console.error('[fetchHistoricalEvents] Error processing transaction:', error);
            return [];
          }
        });

        // Wait for all transactions in parallel
        const eventArrays = await Promise.all(transactionPromises);
        console.log(`[fetchHistoricalEvents] Processed ${eventArrays.length} transactions, got ${eventArrays.reduce((sum, arr) => sum + arr.length, 0)} events before filtering`);
        for (const eventArray of eventArrays) {
          // Filter to only include task events (exclude NodeValidated, NodeRejected, etc.)
          const taskEvents = eventArray.filter(event =>
            event.type === 'TaskClaimed' ||
            event.type === 'TaskResultSubmitted' ||
            event.type === 'TaskValidationSubmitted'
          );
          events.push(...taskEvents);
          // Early stop if we have enough events
          if (events.length >= limit) break;
        }

        // Sort events by timestamp (newest first)
        events.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
      } catch (error) {
        console.error('[fetchHistoricalEvents] Failed to fetch historical events:', error);
        throw error;
      }
      console.log('[fetchHistoricalEvents] Events:', events.length);
      return events;
    },
    
    async subscribeToNetworkEvents(
      callback: (event: GoalEvent) => void
    ): Promise<() => void> {
      if (!rpcSubscriptions) {
        throw new Error('RPC Subscriptions not available');
      }

      const abortController = new AbortController();

      try {
        // Use logsNotifications with mentions (program-wide realtime)
        // This gives us signature + logs directly, eliminating getTransaction calls
        const rpcSubs = rpcSubscriptions as any;
        const logsIterable = await rpcSubs
          .logsNotifications({ mentions: [programAddress] })
          .subscribe({ abortSignal: abortController.signal });

        (async () => {
          try {
            for await (const notification of logsIterable) {
              if (abortController.signal.aborted) break;

              // Extract notification data (handle different notification shapes)
              const value = notification?.value ?? notification?.params?.result?.value ?? notification?.result?.value;
              const signature: string | undefined = value?.signature;
              const logs: string[] | undefined = value?.logs;
              const err = value?.err;

              if (!signature || !logs) continue;
              if (err) continue; // Skip failed transactions

              // Parse events directly from logs (no getTransaction call needed)
              const parsedEvents = parseEventsFromLogs(logs, null, signature);

              // Emit only network-wide event types
              for (const event of parsedEvents) {
                if (
                  event.type === 'GoalSet' ||
                  event.type === 'ContributionMade' ||
                  event.type === 'GoalCompleted' ||
                  event.type === 'AgentCreated'
                ) {
                  callback(event);
                }
              }
            }
          } catch (e) {
            if (!abortController.signal.aborted) {
              console.error('[subscribeToNetworkEvents] logsNotifications error:', e);
            }
          }
        })();

        return async () => {
          abortController.abort();
        };
      } catch (error) {
        throw new Error(`Failed to subscribe to network events: ${error}`);
      }
    },
    
    async fetchNetworkHistoricalEvents(
      options: FetchHistoricalEventsOptions = {}
    ): Promise<GoalEvent[]> {
      const { limit = 100, before } = options;
      const events: GoalEvent[] = [];

      try {
        const signatureOptions: any = { limit: Math.min(limit * 2, 100) }; // Cap at 100 signatures max
        if (before) {
          signatureOptions.before = before;
        }

        const signatures = await (deps.rpc as any).getSignaturesForAddress(programAddress, signatureOptions).send();

        if (!signatures || signatures.length === 0) {
          return events;
        }

        // Batch fetch transactions in parallel instead of sequentially
        const processedSignatures = new Set<string>();
        const signatureList = (signatures || [])
          .filter((sig: any) => sig.signature && !processedSignatures.has(sig.signature))
          .slice(0, Math.min(limit * 2, 100)) // Limit to reasonable number, cap at 100
          .map((sig: any) => {
            processedSignatures.add(sig.signature!);
            return sig.signature!;
          });

        // Fetch all transactions in parallel (batched)
        const transactionPromises = signatureList.map(async (signature: string) => {
          try {
            const tx = await (deps.rpc as any).getTransaction(signature, {
              commitment: 'confirmed',
              maxSupportedTransactionVersion: 0,
            }).send();

            if (!tx?.meta?.logMessages) return [];

            // Get transaction timestamp from blockTime (Unix timestamp in seconds)
            // blockTime might be BigInt, so convert to number first
            const blockTime = tx.blockTime 
              ? new Date(Number(tx.blockTime) * 1000) 
              : new Date();

            const parsedEvents = parseEventsFromLogs(
              tx.meta.logMessages,
              null, // No goal filter
              signature,
              blockTime
            );

            // Only include network-wide events
            return parsedEvents.filter(event => 
              event.type === 'GoalSet' || event.type === 'ContributionMade' || 
              event.type === 'GoalCompleted' || event.type === 'AgentCreated'
            );
          } catch (error) {
            // Skip individual transaction errors
            return [];
          }
        });

        // Wait for all transactions in parallel
        const eventArrays = await Promise.all(transactionPromises);
        for (const eventArray of eventArrays) {
          events.push(...eventArray);
          // Early stop if we have enough events
          if (events.length >= limit) break;
        }

        events.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
      } catch (error) {
        console.error('[fetchNetworkHistoricalEvents] Failed:', error);
        throw error;
      }

      return events;
    },
  };
}

/**
 * Parse events from transaction logs
 * Anchor events are emitted as "Program data: <base64>" in logs
 */
function parseEventsFromLogs(
  logs: string[],
  goalSlotId: bigint | null, // Make optional for network-wide events
  signature: string,
  timestamp: Date = new Date() // Transaction timestamp
): GoalEvent[] {
  const events: GoalEvent[] = [];
  
  
  for (const log of logs) {
    // Anchor events appear as "Program data: <base64>"
    if (log.includes('Program data:')) {
      try {
        const dataMatch = log.match(/Program data: (.+)/);
        if (dataMatch) {
          const base64Data = dataMatch[1].trim();
          const event = parseAnchorEvent(base64Data, goalSlotId, signature, timestamp);
          if (event) {
            events.push(event);
          }
        }
      } catch (e) {
        console.error('[parseEventsFromLogs] Error parsing log:', e);
      }
    }
  }
  
  return events;
}

/**
 * Calculate Anchor event discriminator
 * Anchor uses: sha256("event:<EventName>")[0..8]
 * Cached to avoid recalculating on every event parse
 */
const DISCRIMINATOR_CACHE = new Map<string, Buffer>();

function calculateEventDiscriminator(eventName: string): Buffer {
  if (DISCRIMINATOR_CACHE.has(eventName)) {
    return DISCRIMINATOR_CACHE.get(eventName)!;
  }
  
  const crypto = require('crypto');
  const discriminator = Buffer.from(
    crypto
      .createHash('sha256')
      .update(`event:${eventName}`)
      .digest()
      .slice(0, 8)
  );
  
  DISCRIMINATOR_CACHE.set(eventName, discriminator);
  return discriminator;
}

/**
 * Pre-calculate all event discriminators once
 */
const EVENT_DISCRIMINATORS = {
  taskClaimed: calculateEventDiscriminator('TaskClaimed'),
  taskResultSubmitted: calculateEventDiscriminator('TaskResultSubmitted'),
  taskValidationSubmitted: calculateEventDiscriminator('TaskValidationSubmitted'),
  goalSet: calculateEventDiscriminator('GoalSet'),
  contributionMade: calculateEventDiscriminator('ContributionMade'),
  goalCompleted: calculateEventDiscriminator('GoalCompleted'),
  nodeValidated: calculateEventDiscriminator('NodeValidated'),
  nodeRejected: calculateEventDiscriminator('NodeRejected'),
  agentCreated: calculateEventDiscriminator('AgentCreated'),
} as const;

/**
 * Parse Anchor event from base64 data using generated decoders
 * Anchor events have an 8-byte discriminator followed by the event data
 */
function parseAnchorEvent(
  base64Data: string,
  goalSlotId: bigint | null,
  signature: string,
  timestamp: Date = new Date() // Transaction timestamp
): GoalEvent | null {
  try {
    const buffer = Buffer.from(base64Data, 'base64');
    if (buffer.length < 8) return null;

    const discriminator = buffer.slice(0, 8);
    const eventData = new Uint8Array(buffer.slice(8));

    // Decode events using generated decoders
    try {
      if (discriminator.equals(EVENT_DISCRIMINATORS.goalSet)) {
        const decoder = getGoalSetDecoder();
        const decoded = decoder.decode(eventData);
        
        if (goalSlotId !== null && decoded.goalSlotId !== goalSlotId) {
          return null;
        }
        
        return {
          type: 'GoalSet',
          goalSlotId: decoded.goalSlotId,
          taskSlotId: decoded.taskSlotId,
          timestamp,
          signature,
          data: decoded,
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.contributionMade)) {
        const decoder = getContributionMadeDecoder();
        const decoded = decoder.decode(eventData);
        
        if (goalSlotId !== null && decoded.goalSlotId !== goalSlotId) {
          return null;
        }
        
        return {
          type: 'ContributionMade',
          goalSlotId: decoded.goalSlotId,
          timestamp,
          signature,
          data: decoded,
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.goalCompleted)) {
        const decoder = getGoalCompletedDecoder();
        const decoded = decoder.decode(eventData);
        
        if (goalSlotId !== null && decoded.goalSlotId !== goalSlotId) {
          return null;
        }
        
        return {
          type: 'GoalCompleted',
          goalSlotId: decoded.goalSlotId,
          timestamp,
          signature,
          data: decoded,
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.nodeValidated)) {
        const decoder = getNodeValidatedDecoder();
        const decoded = decoder.decode(eventData);
        
        const goalSlotIdValue = isSome(decoded.goalSlotId) ? (unwrapOption(decoded.goalSlotId) ?? undefined) : undefined;
        const taskSlotIdValue = isSome(decoded.taskSlotId) ? (unwrapOption(decoded.taskSlotId) ?? undefined) : undefined;
        
        if (goalSlotId !== null && goalSlotIdValue !== undefined && goalSlotIdValue !== goalSlotId) {
          return null;
        }
        
        return {
          type: 'NodeValidated',
          goalSlotId: goalSlotIdValue,
          taskSlotId: taskSlotIdValue,
          timestamp,
          signature,
          data: {
            node: decoded.node,
            validator: decoded.validator,
            goalSlotId: goalSlotIdValue,
            taskSlotId: taskSlotIdValue,
          },
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.nodeRejected)) {
        const decoder = getNodeRejectedDecoder();
        const decoded = decoder.decode(eventData);
        
        const goalSlotIdValue = isSome(decoded.goalSlotId) ? (unwrapOption(decoded.goalSlotId) ?? undefined) : undefined;
        const taskSlotIdValue = isSome(decoded.taskSlotId) ? (unwrapOption(decoded.taskSlotId) ?? undefined) : undefined;
        
        if (goalSlotId !== null && goalSlotIdValue !== undefined && goalSlotIdValue !== goalSlotId) {
          return null;
        }
        
        return {
          type: 'NodeRejected',
          goalSlotId: goalSlotIdValue,
          taskSlotId: taskSlotIdValue,
          timestamp,
          signature,
          data: {
            node: decoded.node,
            validator: decoded.validator,
            goalSlotId: goalSlotIdValue,
            taskSlotId: taskSlotIdValue,
          },
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.agentCreated)) {
        const decoder = getAgentCreatedDecoder();
        const decoded = decoder.decode(eventData);
        
        return {
          type: 'AgentCreated',
          timestamp,
          signature,
          data: decoded,
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.taskClaimed)) {
        const decoder = getTaskClaimedDecoder();
        const decoded = decoder.decode(eventData);
        
        if (goalSlotId !== null && decoded.goalSlotId !== goalSlotId) {
          return null;
        }

        return {
          type: 'TaskClaimed',
          goalSlotId: decoded.goalSlotId,
          taskSlotId: decoded.taskSlotId,
          timestamp,
          signature,
          data: decoded,
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.taskResultSubmitted)) {
        const decoder = getTaskResultSubmittedDecoder();
        const decoded = decoder.decode(eventData);

        if (goalSlotId !== null && decoded.goalSlotId !== goalSlotId) {
          return null;
        }

        return {
          type: 'TaskResultSubmitted',
          goalSlotId: decoded.goalSlotId,
          taskSlotId: decoded.taskSlotId,
          timestamp,
          signature,
          data: decoded,
        };
      } else if (discriminator.equals(EVENT_DISCRIMINATORS.taskValidationSubmitted)) {
        const decoder = getTaskValidationSubmittedDecoder();
        const decoded = decoder.decode(eventData);

        if (goalSlotId !== null && decoded.goalSlotId !== goalSlotId) {
          return null;
        }

        return {
          type: 'TaskValidationSubmitted',
          goalSlotId: decoded.goalSlotId,
          taskSlotId: decoded.taskSlotId,
          timestamp,
          signature,
          data: decoded,
        };
      }
    } catch (decodeError) {
      console.error('[parseAnchorEvent] Error decoding event data:', decodeError);
      return null;
    }

    return null;
  } catch (e) {
    console.error('[parseAnchorEvent] Failed to parse event:', e);
    return null;
  }
}
