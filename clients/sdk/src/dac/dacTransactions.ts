import { address, type Address } from '@solana/kit';
import type { Instruction, TransactionMessage, TransactionMessageWithFeePayer, TransactionMessageWithBlockhashLifetime } from '@solana/kit';
import { AccountRole } from '@solana/kit';
import { buildTransaction, type TransactionSigner } from './utils.js';

type TransactionMessageType = TransactionMessage & TransactionMessageWithFeePayer<string> & TransactionMessageWithBlockhashLifetime;
import {
  deriveNetworkConfigAddress,
  deriveAgentAddress,
  deriveGoalAddress,
  deriveTaskAddress,
  deriveNodeInfoAddress,
} from './dacPdas.js';
import {
  getActivateNodeInstruction,
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
} from '../generated/dac/instructions/index.js';
import type { NodeType } from '../generated/dac/types/index.js';
import { AgentStatus } from '../generated/dac/types/index.js';
import { 
  fetchMaybeNetworkConfig,
  fetchMaybeGoal,
  fetchMaybeTask,
  fetchMaybeAgent,
} from '../generated/dac/accounts/index.js';
import type {
  ITransactionService,
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

      for (let goalId = 0; goalId < params.allocateGoals; goalId++) {
        const goalAddress = await deriveGoalAddress(programAddress, networkConfigAddress, BigInt(goalId));
        remainingAccounts.push(goalAddress);
      }

      for (let taskId = 0; taskId < params.allocateTasks; taskId++) {
        const taskAddress = await deriveTaskAddress(programAddress, networkConfigAddress, BigInt(taskId));
        remainingAccounts.push(taskAddress);
      }

      const input: InitializeNetworkInput = {
        authority: address(params.authority.address) as any,
        networkConfig: networkConfigAddress,
        cidConfig: params.cidConfig,
        allocateGoals: params.allocateGoals,
        allocateTasks: params.allocateTasks,
        approvedCodeMeasurements: params.approvedCodeMeasurements,
        requiredValidations: params.requiredValidations,
      };

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

    async createGoal(params: CreateGoalParams): Promise<{ transactionMessage: TransactionMessageType; goalAddress: Address; goalSlotId: bigint; taskAddress: Address; taskSlotId: bigint }> {
      const account = await fetchMaybeNetworkConfig(rpc, params.networkConfig);
      if (!account.exists || !account.data) {
        throw new Error('Network config not found');
      }
      const networkConfigData = account.data;

      const goalSlotId = networkConfigData.goalCount;
      const taskSlotId = networkConfigData.taskCount;
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, goalSlotId);
      const taskAddress = await deriveTaskAddress(programAddress, params.networkConfig, taskSlotId);

      const input: CreateGoalInput = {
        payer: address(params.payer.address) as any,
        owner: address(params.owner.address) as any,
        networkConfig: params.networkConfig,
        goal: goalAddress,
        task: taskAddress,
        isOwned: params.isOwned,
        isConfidential: params.isConfidential,
      };

      const instruction = getCreateGoalInstruction(input, {
        programAddress,
      });

      const { transactionMessage } = await buildTransactionWithRpc(params.payer, [instruction]);

      return { transactionMessage, goalAddress, goalSlotId, taskAddress, taskSlotId };
    },

    async setGoal(params: SetGoalParams): Promise<TransactionMessageType> {
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, params.goalSlotId);
      const agentAddress = await deriveAgentAddress(programAddress, params.networkConfig, params.agentSlotId);
      const taskAddress = await deriveTaskAddress(programAddress, params.networkConfig, params.taskSlotId);


      // Check if accounts exist using the proper fetch functions
      try {
        const goalAccount = await fetchMaybeGoal(rpc, goalAddress);
        if (!goalAccount.exists) {
          throw new Error(`Goal account does not exist for slotId ${params.goalSlotId.toString()}. Make sure the goal was created first.`);
        }
      } catch (error: any) {
        console.error('[setGoal] Error checking goal account:', error);
        throw new Error(`Failed to check goal account: ${error.message}`);
      }

      try {
        const taskAccount = await fetchMaybeTask(rpc, taskAddress);
        if (!taskAccount.exists) {
          throw new Error(`Task account does not exist for slotId ${params.taskSlotId.toString()}. Make sure the task was created first.`);
        }
      } catch (error: any) {
        console.error('[setGoal] Error checking task account:', error);
        throw new Error(`Failed to check task account: ${error.message}`);
      }

      try {
        const agentAccount = await fetchMaybeAgent(rpc, agentAddress);
        if (!agentAccount.exists) {
          throw new Error(`Agent account does not exist for slotId ${params.agentSlotId.toString()}. Make sure the agent was created and activated first.`);
        }
        if (agentAccount.data && agentAccount.data.status !== AgentStatus.Active) {
          throw new Error(`Agent is not active. Current status: ${agentAccount.data.status}. The agent must be activated before setting a goal.`);
        }
      } catch (error: any) {
        console.error('[setGoal] Error checking agent account:', error);
        throw new Error(`Failed to check agent account: ${error.message}`);
      }

      const input: SetGoalAsyncInput = {
        owner: address(params.owner.address) as any,
        goal: goalAddress,
        task: taskAddress,
        agent: agentAddress,
        networkConfig: params.networkConfig,
        specificationCid: params.specificationCid,
        maxIterations: params.maxIterations,
        initialDeposit: params.initialDeposit,
      };

      const instruction = await getSetGoalInstructionAsync(input, {
        programAddress,
      });

      const { transactionMessage } = await buildTransactionWithRpc(params.owner, [instruction]);
      return transactionMessage;
    },

    async contributeToGoal(params: ContributeToGoalParams): Promise<TransactionMessageType> {
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, params.goalSlotId);

      const input: ContributeToGoalAsyncInput = {
        contributor: address(params.contributor.address) as any,
        goal: goalAddress,
        networkConfig: params.networkConfig,
        depositAmount: params.depositAmount,
      };

      const instruction = await getContributeToGoalInstructionAsync(input, {
        programAddress,
      });

      const { transactionMessage } = await buildTransactionWithRpc(params.contributor, [instruction]);
      return transactionMessage;
    },

    async withdrawFromGoal(params: WithdrawFromGoalParams): Promise<TransactionMessageType> {
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, params.goalSlotId);

      const input: WithdrawFromGoalAsyncInput = {
        contributor: address(params.contributor.address) as any,
        goal: goalAddress,
        networkConfig: params.networkConfig,
        sharesToBurn: params.sharesToBurn,
      };

      const instruction = await getWithdrawFromGoalInstructionAsync(input, {
        programAddress,
      });

      const { transactionMessage } = await buildTransactionWithRpc(params.contributor, [instruction]);
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
