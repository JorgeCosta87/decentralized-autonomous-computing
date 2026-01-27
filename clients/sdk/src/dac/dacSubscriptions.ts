import type { Address, RpcSubscriptions } from '@solana/kit';
import { address, isSome, unwrapOption } from '@solana/kit';
import type { DacServiceDeps } from './dacService.js';
import { deriveSessionAddress, DAC_PROGRAM_ID } from './dacPdas.js';
import {
  getTaskClaimedDecoder,
  getTaskResultSubmittedDecoder,
  getTaskValidationSubmittedDecoder,
  getSessionSetDecoder,
  getContributionMadeDecoder,
  getSessionCompletedDecoder,
  getAgentCreatedDecoder,
  getNodeValidatedDecoder,
  getNodeRejectedDecoder,
} from '../generated/dac/types/index.js';

export interface SessionEvent {
  type: 'TaskClaimed' | 'TaskResultSubmitted' | 'TaskValidationSubmitted' | 'SessionSet' | 'ContributionMade' | 'SessionCompleted' | 'NodeValidated' | 'NodeRejected' | 'AgentCreated';
  sessionSlotId?: bigint;
  taskSlotId?: bigint;
  timestamp: Date;
  signature?: string;
  data: TaskClaimedEvent | TaskResultSubmittedEvent | TaskValidationSubmittedEvent | SessionSetEvent | ContributionMadeEvent | SessionCompletedEvent | NodeValidatedEvent | NodeRejectedEvent | AgentCreatedEvent;
}

export interface TaskClaimedEvent {
  sessionSlotId: bigint;
  taskSlotId: bigint;
  computeNode: Address;
  maxTaskCost: bigint;
}

export interface TaskResultSubmittedEvent {
  sessionSlotId: bigint;
  taskSlotId: bigint;
  inputCid: string;
  outputCid: string;
}

export interface TaskValidationSubmittedEvent {
  sessionSlotId: bigint;
  taskSlotId: bigint;
  validator: Address;
  paymentAmount: bigint;
  approved: boolean;
  sessionCompleted: boolean;
  currentIteration: bigint;
  vaultBalance: bigint;
  lockedForTasks: bigint;
}

export interface SessionSetEvent {
  sessionSlotId: bigint;
  owner: Address;
  taskSlotId: bigint;
  specificationCid: string;
  maxIterations: bigint;
  initialDeposit: bigint;
}

export interface ContributionMadeEvent {
  sessionSlotId: bigint;
  contributor: Address;
  depositAmount: bigint;
  sharesMinted: bigint;
  totalShares: bigint;
}

export interface SessionCompletedEvent {
  sessionSlotId: bigint;
  finalIteration: bigint;
  vaultBalance: bigint;
}

export interface NodeValidatedEvent {
  node: Address;
  validator: Address;
  sessionSlotId?: bigint;
  taskSlotId?: bigint;
}

export interface NodeRejectedEvent {
  node: Address;
  validator: Address;
  sessionSlotId?: bigint;
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
  subscribeToSessionEvents(
    networkConfig: Address,
    sessionSlotId: bigint,
    callback: (event: SessionEvent) => void
  ): Promise<() => void>;

  /**
   * Subscribe to network-wide events (SessionSet, ContributionMade, SessionCompleted, AgentCreated)
   */
  subscribeToNetworkEvents(
    callback: (event: SessionEvent) => void
  ): Promise<() => void>;
  
  /**
   * Fetch historical events for a session (and optionally a specific task)
   */
  fetchHistoricalEvents(
    networkConfig: Address,
    sessionSlotId: bigint,
    options?: FetchHistoricalEventsOptions
  ): Promise<SessionEvent[]>;
  
  /**
   * Fetch network-wide historical events
   */
  fetchNetworkHistoricalEvents(
    options?: FetchHistoricalEventsOptions
  ): Promise<SessionEvent[]>;
}

/**
 * Create subscription service
 */
export function createSubscriptionService(
  deps: DacServiceDeps & { rpcSubscriptions?: RpcSubscriptions<any> | null }
): ISubscriptionService {
  const { rpcSubscriptions, programAddress } = deps;

  return {
    async subscribeToSessionEvents(
      networkConfig: Address,
      sessionSlotId: bigint,
      callback: (event: SessionEvent) => void
    ): Promise<() => void> {
      if (!rpcSubscriptions) {
        throw new Error('RPC Subscriptions not available');
      }

      const abortController = new AbortController();

      try {
        const rpcSubs = rpcSubscriptions as any;
        const logsIterable = await rpcSubs
          .logsNotifications({ mentions: [programAddress] })
          .subscribe({ abortSignal: abortController.signal });

        (async () => {
          try {
            for await (const notification of logsIterable) {
              if (abortController.signal.aborted) break;

              const value = notification?.value ?? notification?.params?.result?.value ?? notification?.result?.value;
              const signature: string | undefined = value?.signature;
              const logs: string[] | undefined = value?.logs;
              const err = value?.err;

              if (!signature || !logs) continue;
              if (err) continue;

              const events = parseEventsFromLogs(logs, sessionSlotId, signature);
              for (const event of events) {
                callback(event);
              }
            }
          } catch (e) {
            if (!abortController.signal.aborted) {
              console.error('[subscribeToSessionEvents] logsNotifications error:', e);
            }
          }
        })();

        return async () => {
          abortController.abort();
        };
      } catch (error) {
        throw new Error(`Failed to subscribe to session events: ${error}`);
      }
    },

    async fetchHistoricalEvents(
      networkConfig: Address,
      sessionSlotId: bigint,
      options: FetchHistoricalEventsOptions = {}
    ): Promise<SessionEvent[]> {
      const { taskSlotId, limit = 1000, before } = options;
      const sessionAddress = await deriveSessionAddress(programAddress, networkConfig, sessionSlotId);
      const events: SessionEvent[] = [];

      try {
        
        // Fetch transaction signatures for the session account
        const signatureOptions: any = {
          limit: Math.min(limit * 2, 500),
        };
        if (before) {
          signatureOptions.before = before;
        }

        let signatures: any[] = [];
        try {
          signatures = await (deps.rpc as any).getSignaturesForAddress(sessionAddress, signatureOptions).send();
        } catch (error) {
          console.error('[fetchHistoricalEvents] Failed to get session account transactions:', error);
          try {
            const { deriveSessionVaultAddress } = await import('./dacPdas.js');
            const vaultAddress = await deriveSessionVaultAddress(programAddress, sessionAddress);
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
              sessionSlotId,
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
      callback: (event: SessionEvent) => void
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

              for (const event of parsedEvents) {
                if (
                  event.type === 'SessionSet' ||
                  event.type === 'ContributionMade' ||
                  event.type === 'SessionCompleted' ||
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
    ): Promise<SessionEvent[]> {
      const { limit = 100, before } = options;
      const events: SessionEvent[] = [];

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

            return parsedEvents.filter(event =>
              event.type === 'SessionSet' || event.type === 'ContributionMade' ||
              event.type === 'SessionCompleted' || event.type === 'AgentCreated'
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
  sessionSlotId: bigint | null,
  signature: string,
  timestamp: Date = new Date()
): SessionEvent[] {
  const events: SessionEvent[] = [];

  for (const log of logs) {
    if (log.includes('Program data:')) {
      try {
        const dataMatch = log.match(/Program data: (.+)/);
        if (dataMatch) {
          const base64Data = dataMatch[1].trim();
          const event = parseAnchorEvent(base64Data, sessionSlotId, signature, timestamp);
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
  sessionSet: calculateEventDiscriminator('SessionSet'),
  contributionMade: calculateEventDiscriminator('ContributionMade'),
  sessionCompleted: calculateEventDiscriminator('SessionCompleted'),
  nodeValidated: calculateEventDiscriminator('NodeValidated'),
  nodeRejected: calculateEventDiscriminator('NodeRejected'),
  agentCreated: calculateEventDiscriminator('AgentCreated'),
} as const;

/** Decoded event fields: generated types use camelCase; IDL can use snake_case. */
type Decoded = Record<string, unknown>;

function optSlot(v: unknown): bigint | undefined {
  if (v == null) return undefined;
  try {
    if (!isSome(v as Parameters<typeof isSome>[0])) return undefined;
    const raw = unwrapOption(v as Parameters<typeof unwrapOption>[0]);
    return typeof raw === 'bigint' ? raw : undefined;
  } catch {
    return undefined;
  }
}

function str(d: Decoded, snake: string, camel: string): string {
  return String((d[snake] ?? d[camel]) ?? '');
}
function bn(d: Decoded, snake: string, camel: string): bigint {
  const v = d[snake] ?? d[camel];
  if (typeof v === 'bigint') return v;
  return BigInt(Number(v ?? 0));
}

/**
 * Parse Anchor event from base64 data using generated decoders
 * Anchor events have an 8-byte discriminator followed by the event data
 */
function parseAnchorEvent(
  base64Data: string,
  sessionSlotId: bigint | null,
  signature: string,
  timestamp: Date = new Date()
): SessionEvent | null {
  try {
    const buffer = Buffer.from(base64Data, 'base64');
    if (buffer.length < 8) return null;

    const discriminator = buffer.slice(0, 8);
    const eventData = new Uint8Array(buffer.slice(8));

    try {
      if (discriminator.equals(EVENT_DISCRIMINATORS.sessionSet)) {
        const decoder = getSessionSetDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = (d.session_slot_id ?? d.sessionSlotId) as bigint;
        if (sessionSlotId !== null && sid !== sessionSlotId) return null;
        const tid = (d.task_slot_id ?? d.taskSlotId) as bigint;
        return {
          type: 'SessionSet',
          sessionSlotId: sid,
          taskSlotId: tid,
          timestamp,
          signature,
          data: {
            sessionSlotId: sid,
            owner: d.owner as Address,
            taskSlotId: tid,
            specificationCid: str(d, 'specification_cid', 'specificationCid'),
            maxIterations: bn(d, 'max_iterations', 'maxIterations'),
            initialDeposit: bn(d, 'initial_deposit', 'initialDeposit'),
          },
        };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.contributionMade)) {
        const decoder = getContributionMadeDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = (d.session_slot_id ?? d.sessionSlotId) as bigint;
        if (sessionSlotId !== null && sid !== sessionSlotId) return null;
        return {
          type: 'ContributionMade',
          sessionSlotId: sid,
          timestamp,
          signature,
          data: {
            sessionSlotId: sid,
            contributor: d.contributor as Address,
            depositAmount: bn(d, 'deposit_amount', 'depositAmount'),
            sharesMinted: bn(d, 'shares_minted', 'sharesMinted'),
            totalShares: bn(d, 'total_shares', 'totalShares'),
          },
        };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.sessionCompleted)) {
        const decoder = getSessionCompletedDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = (d.session_slot_id ?? d.sessionSlotId) as bigint;
        if (sessionSlotId !== null && sid !== sessionSlotId) return null;
        return {
          type: 'SessionCompleted',
          sessionSlotId: sid,
          timestamp,
          signature,
          data: {
            sessionSlotId: sid,
            finalIteration: bn(d, 'final_iteration', 'finalIteration'),
            vaultBalance: bn(d, 'vault_balance', 'vaultBalance'),
          },
        };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.nodeValidated)) {
        const decoder = getNodeValidatedDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = optSlot(d.goal_slot_id ?? d.goalSlotId);
        const tid = optSlot(d.task_slot_id ?? d.taskSlotId);
        if (sessionSlotId !== null && sid !== undefined && sid !== sessionSlotId) return null;
        return {
          type: 'NodeValidated',
          sessionSlotId: sid,
          taskSlotId: tid,
          timestamp,
          signature,
          data: { node: d.node as Address, validator: d.validator as Address, sessionSlotId: sid, taskSlotId: tid },
        };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.nodeRejected)) {
        const decoder = getNodeRejectedDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = optSlot(d.goal_slot_id ?? d.goalSlotId);
        const tid = optSlot(d.task_slot_id ?? d.taskSlotId);
        if (sessionSlotId !== null && sid !== undefined && sid !== sessionSlotId) return null;
        return {
          type: 'NodeRejected',
          sessionSlotId: sid,
          taskSlotId: tid,
          timestamp,
          signature,
          data: { node: d.node as Address, validator: d.validator as Address, sessionSlotId: sid, taskSlotId: tid },
        };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.agentCreated)) {
        const decoder = getAgentCreatedDecoder();
        const decoded = decoder.decode(eventData);
        return { type: 'AgentCreated', timestamp, signature, data: decoded as AgentCreatedEvent };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.taskClaimed)) {
        const decoder = getTaskClaimedDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = (d.session_slot_id ?? d.sessionSlotId) as bigint;
        if (sessionSlotId !== null && sid !== sessionSlotId) return null;
        const tid = (d.task_slot_id ?? d.taskSlotId) as bigint;
        return {
          type: 'TaskClaimed',
          sessionSlotId: sid,
          taskSlotId: tid,
          timestamp,
          signature,
          data: {
            sessionSlotId: sid,
            taskSlotId: tid,
            computeNode: (d.compute_node ?? d.computeNode) as Address,
            maxTaskCost: bn(d, 'max_task_cost', 'maxTaskCost'),
          },
        };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.taskResultSubmitted)) {
        const decoder = getTaskResultSubmittedDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = (d.session_slot_id ?? d.sessionSlotId) as bigint;
        if (sessionSlotId !== null && sid !== sessionSlotId) return null;
        const tid = (d.task_slot_id ?? d.taskSlotId) as bigint;
        return {
          type: 'TaskResultSubmitted',
          sessionSlotId: sid,
          taskSlotId: tid,
          timestamp,
          signature,
          data: {
            sessionSlotId: sid,
            taskSlotId: tid,
            inputCid: str(d, 'input_cid', 'inputCid'),
            outputCid: str(d, 'output_cid', 'outputCid'),
          },
        };
      }
      if (discriminator.equals(EVENT_DISCRIMINATORS.taskValidationSubmitted)) {
        const decoder = getTaskValidationSubmittedDecoder();
        const d = decoder.decode(eventData) as unknown as Decoded;
        const sid = (d.session_slot_id ?? d.sessionSlotId) as bigint;
        if (sessionSlotId !== null && sid !== sessionSlotId) return null;
        const tid = (d.task_slot_id ?? d.taskSlotId) as bigint;
        return {
          type: 'TaskValidationSubmitted',
          sessionSlotId: sid,
          taskSlotId: tid,
          timestamp,
          signature,
          data: {
            sessionSlotId: sid,
            taskSlotId: tid,
            validator: d.validator as Address,
            paymentAmount: bn(d, 'payment_amount', 'paymentAmount'),
            approved: Boolean(d.approved),
            sessionCompleted: Boolean(d.session_completed ?? d.sessionCompleted),
            currentIteration: bn(d, 'current_iteration', 'currentIteration'),
            vaultBalance: bn(d, 'vault_balance', 'vaultBalance'),
            lockedForTasks: bn(d, 'locked_for_tasks', 'lockedForTasks'),
          },
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
