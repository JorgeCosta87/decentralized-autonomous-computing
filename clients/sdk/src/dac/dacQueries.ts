import type { Address, Rpc } from '@solana/kit';
import {
  deriveNetworkConfigAddress,
  deriveAgentAddress,
  deriveSessionAddress,
  deriveTaskAddress,
  deriveContributionAddress,
  deriveSessionVaultAddress,
  deriveNodeInfoAddress,
} from './dacPdas.js';
import {
  fetchMaybeNetworkConfig,
  fetchMaybeAgent,
  fetchMaybeSession,
  fetchMaybeTask,
  fetchMaybeContribution,
  fetchMaybeNodeInfo,
  decodeNodeInfo,
  decodeAgent,
  decodeTask,
  decodeSession,
  decodeContribution,
  CONTRIBUTION_DISCRIMINATOR,
  NODE_INFO_DISCRIMINATOR,
  AGENT_DISCRIMINATOR,
  TASK_DISCRIMINATOR,
  SESSION_DISCRIMINATOR,
  type NetworkConfig,
  type Agent,
  type Session,
  type Contribution,
  type NodeInfo,
  type Task,
} from '../generated/dac/accounts/index.js';
import type { NodeStatus, AgentStatus, TaskStatus, SessionStatus, NodeType } from '../generated/dac/types/index.js';
import type { IQueryService, DacServiceDeps } from './dacService.js';
import { decodeAccountsFromResponse } from './dacUtils.js';

/**
 * Create query service factory
 */
export function createQueryService(deps: DacServiceDeps): IQueryService {
  const { rpc, programAddress, getAuthority } = deps;

  return {
    async getNetworkConfig(authority?: Address): Promise<NetworkConfig | null> {
      const auth = authority || getAuthority();
      if (!auth) {
        throw new Error('Authority is required. Either set it with setAuthority() or pass it as parameter.');
      }
      const networkConfigAddress = await deriveNetworkConfigAddress(programAddress, auth);
      const account = await fetchMaybeNetworkConfig(rpc, networkConfigAddress);
      return account.exists ? account.data : null;
    },

    async getAgent(agentAddress: Address): Promise<Agent | null> {
      const account = await fetchMaybeAgent(rpc, agentAddress);
      return account.exists ? account.data : null;
    },

    async getAgentBySlot(networkConfig: Address, agentSlotId: bigint): Promise<Agent | null> {
      const agentAddress = await deriveAgentAddress(programAddress, networkConfig, agentSlotId);
      const account = await fetchMaybeAgent(rpc, agentAddress);
      return account.exists ? account.data : null;
    },

    async getSession(networkConfig: Address, sessionSlotId: bigint): Promise<Session | null> {
      const sessionAddress = await deriveSessionAddress(programAddress, networkConfig, sessionSlotId);
      const account = await fetchMaybeSession(rpc, sessionAddress);
      return account.exists ? account.data : null;
    },

    async getTask(networkConfig: Address, taskSlotId: bigint): Promise<Task | null> {
      const taskAddress = await deriveTaskAddress(programAddress, networkConfig, taskSlotId);
      const account = await fetchMaybeTask(rpc, taskAddress);
      return account.exists ? account.data : null;
    },

    async getContribution(session: Address, contributor: Address): Promise<Contribution | null> {
      const contributionAddress = await deriveContributionAddress(programAddress, session, contributor);
      const account = await fetchMaybeContribution(rpc, contributionAddress);
      return account.exists ? account.data : null;
    },

    async getNodeInfo(nodePubkey: Address): Promise<NodeInfo | null> {
      const nodeInfoAddress = await deriveNodeInfoAddress(programAddress, nodePubkey);
      const account = await fetchMaybeNodeInfo(rpc, nodeInfoAddress);
      return account.exists ? account.data : null;
    },

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
            offset: 72,
            bytes: [params.nodeType],
          },
        });
      }

      if (params?.status !== undefined) {
        filters.push({
          memcmp: {
            offset: 73,
            bytes: [params.status],
          },
        });
      }

      const response = await (rpc as any).getProgramAccounts(programAddress, {
          encoding: 'base64',
        filters: filters as any,
      }).send();

      return decodeAccountsFromResponse(response, decodeNodeInfo);
    },

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
            offset: 48,
            bytes: [status],
          },
        });
      }

      const response = await (rpc as any).getProgramAccounts(programAddress, {
          encoding: 'base64',
        filters: filters as any,
      }).send();

      return decodeAccountsFromResponse(response, decodeAgent);
    },

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
            offset: 49,
            bytes: [status],
          },
        });
      }

      const response = await (rpc as any).getProgramAccounts(programAddress, {
          encoding: 'base64',
        filters: filters as any,
      }).send();

      return decodeAccountsFromResponse(response, decodeTask);
    },

    async getSessionsByStatus(status?: SessionStatus): Promise<Session[]> {
      const filters: any[] = [
        { memcmp: { offset: 0, bytes: Array.from(SESSION_DISCRIMINATOR) } },
      ];
      if (status !== undefined) {
        filters.push({ memcmp: { offset: 112, bytes: [status as number] } });
      }
      const response = await (rpc as any).getProgramAccounts(programAddress, { encoding: 'base64', filters: filters as any }).send();
      return decodeAccountsFromResponse(response, decodeSession);
    },

    async batchGetContributionsForSessions(
      networkConfig: Address,
      sessionSlotIds: bigint[],
      contributorAddress: Address
    ): Promise<Map<bigint, Contribution | null>> {
      if (sessionSlotIds.length === 0) return new Map();
      const contributionAddresses: Address[] = [];
      const slotToAddress = new Map<bigint, Address>();
      for (const sessionSlotId of sessionSlotIds) {
        const sessionAddress = await deriveSessionAddress(programAddress, networkConfig, sessionSlotId);
        const contributionAddress = await deriveContributionAddress(programAddress, sessionAddress, contributorAddress);
        contributionAddresses.push(contributionAddress);
        slotToAddress.set(sessionSlotId, contributionAddress);
      }
      try {
        const response = await (rpc as any).getMultipleAccounts(contributionAddresses).send();
        const result = new Map<bigint, Contribution | null>();
        if (response.value) {
          let idx = 0;
          for (const sessionSlotId of sessionSlotIds) {
            const accountInfo = response.value[idx];
            if (accountInfo?.data) {
              try {
                const decoded = decodeContribution({
                  address: contributionAddresses[idx],
                  data: new Uint8Array(Buffer.from(accountInfo.data[0], 'base64')),
                  executable: accountInfo.executable || false,
                  lamports: accountInfo.lamports as any,
                  programAddress: accountInfo.owner,
                  space: (accountInfo.space ?? 0n) as any,
                });
                result.set(sessionSlotId, decoded.data);
              } catch {
                result.set(sessionSlotId, null);
              }
            } else {
              result.set(sessionSlotId, null);
            }
            idx++;
          }
        } else {
          sessionSlotIds.forEach(id => result.set(id, null));
        }
        return result;
      } catch (error) {
        console.warn('Batch getMultipleAccounts failed, falling back to individual calls:', error);
        const result = new Map<bigint, Contribution | null>();
        for (const sessionSlotId of sessionSlotIds) {
          const sessionAddress = await deriveSessionAddress(programAddress, networkConfig, sessionSlotId);
          const c = await (this as IQueryService).getContribution(sessionAddress, contributorAddress);
          result.set(sessionSlotId, c);
        }
        return result;
      }
    },

    async batchGetVaultBalances(
      networkConfig: Address,
      sessionSlotIds: bigint[]
    ): Promise<Map<bigint, { balance: bigint; rentExempt: bigint }>> {
      if (sessionSlotIds.length === 0) return new Map();
      const vaultAddresses: Address[] = [];
      const slotToVault = new Map<bigint, Address>();
      for (const sessionSlotId of sessionSlotIds) {
        const sessionAddress = await deriveSessionAddress(programAddress, networkConfig, sessionSlotId);
        const vaultAddress = await deriveSessionVaultAddress(programAddress, sessionAddress);
        vaultAddresses.push(vaultAddress);
        slotToVault.set(sessionSlotId, vaultAddress);
      }
      let rentExempt: bigint;
      try {
        const rent = await (rpc as any).getMinimumBalanceForRentExemption(0).send();
        rentExempt = BigInt(rent || 0);
      } catch {
        rentExempt = 0n;
      }
      try {
        const response = await (rpc as any).getMultipleAccounts(vaultAddresses).send();
        const result = new Map<bigint, { balance: bigint; rentExempt: bigint }>();
        if (response.value) {
          let idx = 0;
          for (const sessionSlotId of sessionSlotIds) {
            const acc = response.value[idx];
            const balance = acc?.lamports != null ? BigInt(acc.lamports) : 0n;
            result.set(sessionSlotId, { balance, rentExempt });
            idx++;
          }
        } else {
          sessionSlotIds.forEach(id => result.set(id, { balance: 0n, rentExempt }));
        }
        return result;
      } catch (error) {
        console.warn('Batch vault balance failed:', error);
        const result = new Map<bigint, { balance: bigint; rentExempt: bigint }>();
        for (const sessionSlotId of sessionSlotIds) {
          result.set(sessionSlotId, { balance: 0n, rentExempt });
        }
        return result;
      }
    },

    async getContributorsForSessions(
      networkConfig: Address,
      sessionSlotIds: bigint[]
    ): Promise<Map<bigint, { count: number; contributors: Array<{ address: Address; shares: bigint }> }>> {
      if (sessionSlotIds.length === 0) return new Map();
      const sessionAddresses = await Promise.all(
        sessionSlotIds.map(id => deriveSessionAddress(programAddress, networkConfig, id))
      );
      const slotToAddress = new Map<bigint, Address>();
      sessionSlotIds.forEach((id, i) => slotToAddress.set(id, sessionAddresses[i]));

      const response = await (rpc as any).getProgramAccounts(programAddress, {
        encoding: 'base64',
        filters: [{ memcmp: { offset: 0, bytes: Array.from(CONTRIBUTION_DISCRIMINATOR) } }],
      }).send();

      const contributionsBySession = new Map<Address, Array<{ contribution: Contribution; contributor: Address }>>();
      const accounts = (response as { value?: Array<{ pubkey: Address; account: { data: string; executable: boolean; lamports: unknown; owner: Address; space?: number } }> }).value ?? [];
      for (const account of accounts) {
        try {
          const decoded = decodeContribution({
            address: account.pubkey,
            data: new Uint8Array(Buffer.from(account.account.data, 'base64')),
            executable: account.account.executable || false,
            lamports: account.account.lamports as any,
            programAddress: account.account.owner,
            space: (account.account.space ?? 0n) as any,
          });
          const sessionAddress = (decoded.data as { goal?: Address; session?: Address }).goal ?? (decoded.data as { goal?: Address; session?: Address }).session!;
          if (!contributionsBySession.has(sessionAddress)) {
            contributionsBySession.set(sessionAddress, []);
          }
          contributionsBySession.get(sessionAddress)!.push({ contribution: decoded.data, contributor: account.pubkey });
        } catch {
          // skip invalid
        }
      }

      const result = new Map<bigint, { count: number; contributors: Array<{ address: Address; shares: bigint }> }>();
      for (const sessionSlotId of sessionSlotIds) {
        const sessionAddress = slotToAddress.get(sessionSlotId);
        const contributions = sessionAddress ? (contributionsBySession.get(sessionAddress) ?? []) : [];
        const contributors = contributions
          .filter(c => c.contribution.shares > 0n)
          .map(c => ({ address: c.contributor, shares: c.contribution.shares }));
        result.set(sessionSlotId, { count: contributors.length, contributors });
      }
      return result;
    },
  };
}
