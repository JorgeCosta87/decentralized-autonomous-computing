import { address, type Address } from '@solana/kit';
import type { Instruction, TransactionMessage, TransactionMessageWithFeePayer, TransactionMessageWithBlockhashLifetime } from '@solana/kit';
import { AccountRole } from '@solana/kit';
import { buildTransaction, type TransactionSigner } from './utils.js';

type TransactionMessageType = TransactionMessage & TransactionMessageWithFeePayer<string> & TransactionMessageWithBlockhashLifetime;
import {
  deriveNetworkConfigAddress,
  deriveAgentAddress,
  deriveSessionAddress,
  deriveTaskAddress,
  deriveNodeInfoAddress,
} from './dacPdas.js';
import {
  getActivateNodeInstruction,
  getInitializeNetworkInstruction,
  getCreateAgentInstruction,
  getCreateSessionInstruction,
  getSetSessionInstructionAsync,
  getContributeToSessionInstructionAsync,
  getWithdrawFromSessionInstructionAsync,
  getRegisterNodeInstructionAsync,
  getUpdateNetworkConfigInstructionAsync,
  getSubmitTaskInstruction,
  type InitializeNetworkInput,
  type CreateAgentInput,
  type CreateSessionInput,
  type SetSessionAsyncInput,
  type ContributeToSessionAsyncInput,
  type WithdrawFromSessionAsyncInput,
  type RegisterNodeAsyncInput,
  type UpdateNetworkConfigAsyncInput,
} from '../generated/dac/instructions/index.js';
import { AgentStatus } from '../generated/dac/types/index.js';
import {
  fetchMaybeNetworkConfig,
  fetchMaybeSession,
  fetchMaybeTask,
  fetchMaybeAgent,
} from '../generated/dac/accounts/index.js';
import type {
  ITransactionService,
  DacServiceDeps,
  InitializeNetworkParams,
  RegisterNodeParams,
  CreateAgentParams,
  CreateSessionParams,
  SetSessionParams,
  ContributeToSessionParams,
  WithdrawFromSessionParams,
  SubmitTaskParams,
  UpdateNetworkConfigParams,
  ActivateNodeParams,
} from './dacService.js';

/**
 * Create transaction service factory
 */
export function createTransactionService(deps: DacServiceDeps): ITransactionService {
  const { rpc, programAddress } = deps;
  
  /**
   * Helper function to build transactions using the RPC from the service deps
   * This avoids passing rpc to every transaction method
   */
  async function buildTransactionWithRpc(
    payer: TransactionSigner,
    instructions: Instruction[]
  ) {
    return buildTransaction(rpc, payer, instructions);
  }

  return {
    async initializeNetwork(params: InitializeNetworkParams): Promise<{ transactionMessage: TransactionMessageType; networkConfigAddress: Address }> {
      const networkConfigAddress = await deriveNetworkConfigAddress(
        programAddress,
        address(params.authority.address)
      );

      const remainingAccounts: Address[] = [];
      for (let taskId = 0; taskId < params.allocateTasks; taskId++) {
        const taskAddress = await deriveTaskAddress(programAddress, networkConfigAddress, BigInt(taskId));
        remainingAccounts.push(taskAddress);
      }

      const input: InitializeNetworkInput = {
        authority: address(params.authority.address) as any,
        networkConfig: networkConfigAddress,
        cidConfig: params.cidConfig,
        allocateTasks: params.allocateTasks,
        approvedCodeMeasurements: params.approvedCodeMeasurements,
        requiredValidations: params.requiredValidations,
      } as InitializeNetworkInput;

      const instruction = getInitializeNetworkInstruction(input, {
        programAddress,
      });

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

      const { transactionMessage } = await buildTransactionWithRpc(params.authority, [instructionWithRemainingAccounts]);

      return { transactionMessage, networkConfigAddress };
    },

    async registerNode(params: RegisterNodeParams): Promise<{ transactionMessage: TransactionMessageType; nodeInfoAddress: Address; nodeTreasuryAddress: Address }> {
      const input: RegisterNodeAsyncInput = {
        owner: address(params.owner.address) as any,
        networkConfig: params.networkConfig,
        nodePubkey: params.nodePubkey,
        nodeType: params.nodeType,
      };

      const instruction = await getRegisterNodeInstructionAsync(input, {
        programAddress,
      });

      const nodeInfoAddress = address(instruction.accounts[2].address);
      const nodeTreasuryAddress = address(instruction.accounts[3].address);

      const { transactionMessage } = await buildTransactionWithRpc(params.owner, [instruction]);

      return { transactionMessage, nodeInfoAddress, nodeTreasuryAddress };
    },

    async createAgent(params: CreateAgentParams): Promise<{ transactionMessage: TransactionMessageType; agentAddress: Address; agentSlotId: bigint }> {
      const networkConfigAccount = await fetchMaybeNetworkConfig(rpc, params.networkConfig);
      if (!networkConfigAccount.exists || !networkConfigAccount.data) {
        throw new Error('Network config not found');
      }

      const agentSlotId = networkConfigAccount.data.agentCount;
      const agentAddress = await deriveAgentAddress(
        programAddress,
        params.networkConfig,
        agentSlotId
      );

      const input: CreateAgentInput = {
        agentOwner: address(params.agentOwner.address) as any,
        networkConfig: params.networkConfig,
        agent: agentAddress,
        agentConfigCid: params.agentConfigCid,
      };

      const instruction = getCreateAgentInstruction(input, {
        programAddress,
      });

      const { transactionMessage } = await buildTransactionWithRpc(params.agentOwner, [instruction]);

      return { transactionMessage, agentAddress, agentSlotId };
    },

    async createSession(params: CreateSessionParams): Promise<{ transactionMessage: TransactionMessageType; sessionAddress: Address; sessionSlotId: bigint; taskAddress: Address; taskSlotId: bigint }> {
      const account = await fetchMaybeNetworkConfig(rpc, params.networkConfig);
      if (!account.exists || !account.data) {
        throw new Error('Network config not found');
      }
      const networkConfigData = account.data as { sessionCount?: bigint; taskCount?: bigint };
      const sessionSlotId = networkConfigData.sessionCount!;
      const taskSlotId = networkConfigData.taskCount!;
      const sessionAddress = await deriveSessionAddress(programAddress, params.networkConfig, sessionSlotId);
      const taskAddress = await deriveTaskAddress(programAddress, params.networkConfig, taskSlotId);

      const input: CreateSessionInput = {
        payer: address(params.payer.address) as any,
        owner: address(params.owner.address) as any,
        networkConfig: params.networkConfig,
        session: sessionAddress,
        task: taskAddress,
        isOwned: params.isOwned,
        isConfidential: params.isConfidential,
      };

      const instruction = getCreateSessionInstruction(input, { programAddress });
      const { transactionMessage } = await buildTransactionWithRpc(params.payer, [instruction]);
      return { transactionMessage, sessionAddress, sessionSlotId, taskAddress, taskSlotId };
    },

    async setSession(params: SetSessionParams): Promise<TransactionMessageType> {
      const sessionAddress = await deriveSessionAddress(programAddress, params.networkConfig, params.sessionSlotId);
      const taskAddress = await deriveTaskAddress(programAddress, params.networkConfig, params.taskSlotId);
      const agentAddress = await deriveAgentAddress(programAddress, params.networkConfig, params.agentSlotId);

      const sessionAccount = await fetchMaybeSession(rpc, sessionAddress);
      if (!sessionAccount.exists) {
        throw new Error(`Session account does not exist for slotId ${params.sessionSlotId.toString()}. Create the session first.`);
      }
      const taskAccount = await fetchMaybeTask(rpc, taskAddress);
      if (!taskAccount.exists) {
        throw new Error(`Task account does not exist for slotId ${params.taskSlotId.toString()}.`);
      }

      const taskType = (params.taskType.type === 'Completion')
        ? { completion: params.taskType.modelId }
        : (params.taskType.type === 'Custom')
          ? { custom: params.taskType.moduleId }
          : { humanInLoop: true };
      const input: SetSessionAsyncInput = {
        owner: address(params.owner.address) as any,
        session: sessionAddress,
        task: taskAddress,
        agent: agentAddress,
        networkConfig: params.networkConfig,
        specificationCid: params.specificationCid,
        maxIterations: params.maxIterations,
        initialDeposit: params.initialDeposit,
        computeNode: params.computeNode,
        taskType: taskType as any,
      };

      const instruction = await getSetSessionInstructionAsync(input, { programAddress });
      const { transactionMessage } = await buildTransactionWithRpc(params.owner, [instruction]);
      return transactionMessage;
    },

    async contributeToSession(params: ContributeToSessionParams): Promise<TransactionMessageType> {
      const sessionAddress = await deriveSessionAddress(programAddress, params.networkConfig, params.sessionSlotId);
      const input: ContributeToSessionAsyncInput = {
        contributor: address(params.contributor.address) as any,
        session: sessionAddress,
        networkConfig: params.networkConfig,
        depositAmount: params.depositAmount,
      };
      const instruction = await getContributeToSessionInstructionAsync(input, { programAddress });
      const { transactionMessage } = await buildTransactionWithRpc(params.contributor, [instruction]);
      return transactionMessage;
    },

    async withdrawFromSession(params: WithdrawFromSessionParams): Promise<TransactionMessageType> {
      const sessionAddress = await deriveSessionAddress(programAddress, params.networkConfig, params.sessionSlotId);
      const input: WithdrawFromSessionAsyncInput = {
        contributor: address(params.contributor.address) as any,
        session: sessionAddress,
        networkConfig: params.networkConfig,
        sharesToBurn: params.sharesToBurn,
      };
      const instruction = await getWithdrawFromSessionInstructionAsync(input, { programAddress });
      const { transactionMessage } = await buildTransactionWithRpc(params.contributor, [instruction]);
      return transactionMessage;
    },

    async submitTask(params: SubmitTaskParams): Promise<TransactionMessageType> {
      const sessionAddress = await deriveSessionAddress(programAddress, params.networkConfig, params.sessionSlotId);
      const taskAddress = await deriveTaskAddress(programAddress, params.networkConfig, params.taskSlotId);
      const instruction = getSubmitTaskInstruction(
        {
          owner: address(params.owner.address) as any,
          task: taskAddress,
          session: sessionAddress,
          networkConfig: params.networkConfig,
          inputCid: params.inputCid,
        } as any,
        { programAddress }
      );
      const { transactionMessage } = await buildTransactionWithRpc(params.owner, [instruction]);
      return transactionMessage;
    },

    async updateNetworkConfig(params: UpdateNetworkConfigParams): Promise<TransactionMessageType> {
      const networkConfigAddress = await deriveNetworkConfigAddress(
        programAddress,
        address(params.authority.address)
      );

      const input: UpdateNetworkConfigAsyncInput = {
        authority: address(params.authority.address) as any,
        networkConfig: networkConfigAddress,
        cidConfig: params.cidConfig ?? null,
        newCodeMeasurement: params.newCodeMeasurement ?? null,
      };

      const instruction = await getUpdateNetworkConfigInstructionAsync(input, {
        programAddress,
      });

      const { transactionMessage } = await buildTransactionWithRpc(params.authority, [instruction]);
      return transactionMessage;
    },

    async activateNode(params: ActivateNodeParams): Promise<TransactionMessageType> {
      const networkConfigAddress = await deriveNetworkConfigAddress(
        programAddress,
        address(params.authority.address)
      );
      const nodeInfoAddress = await deriveNodeInfoAddress(
        programAddress,
        params.nodePubkey
      );

      const instruction = getActivateNodeInstruction(
        {
          authority: address(params.authority.address) as any,
          networkConfig: networkConfigAddress,
          nodeInfo: nodeInfoAddress,
        },
        {
          programAddress,
        }
      );

      const { transactionMessage } = await buildTransactionWithRpc(params.authority, [instruction]);
      return transactionMessage;
    },
  };
}
