import type { Address, TransactionSigner, Instruction } from 'gill';
import { AccountRole } from 'gill';
import { sendTransaction } from './utils.js';
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
import { fetchMaybeNetworkConfig } from '../generated/dac/accounts/index.js';
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
  const { client, programAddress } = deps;

  return {
    async initializeNetwork(params: InitializeNetworkParams): Promise<{ signature: string; networkConfigAddress: Address }> {
      const networkConfigAddress = await deriveNetworkConfigAddress(
        programAddress,
        params.authority.address
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
        authority: params.authority,
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

      const signature = await sendTransaction(client, params.authority, [instructionWithRemainingAccounts]);

      return { signature, networkConfigAddress };
    },

    async registerNode(params: RegisterNodeParams): Promise<{ signature: string; nodeInfoAddress: Address; nodeTreasuryAddress: Address }> {
      const input: RegisterNodeAsyncInput = {
        owner: params.owner,
        networkConfig: params.networkConfig,
        nodePubkey: params.nodePubkey,
        nodeType: params.nodeType,
      };

      const instruction = await getRegisterNodeInstructionAsync(input, {
        programAddress,
      });

      const nodeInfoAddress = instruction.accounts[2].address;
      const nodeTreasuryAddress = instruction.accounts[3].address;

      const signature = await sendTransaction(client, params.owner, [instruction]);

      return { signature, nodeInfoAddress, nodeTreasuryAddress };
    },

    async createAgent(params: CreateAgentParams): Promise<{ signature: string; agentAddress: Address; agentSlotId: bigint }> {
      const networkConfigAccount = await fetchMaybeNetworkConfig(client.rpc, params.networkConfig);
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
        agentOwner: params.agentOwner,
        networkConfig: params.networkConfig,
        agent: agentAddress,
        agentConfigCid: params.agentConfigCid,
      };

      const instruction = getCreateAgentInstruction(input, {
        programAddress,
      });

      const signature = await sendTransaction(client, params.agentOwner, [instruction]);

      return { signature, agentAddress, agentSlotId };
    },

    async createGoal(params: CreateGoalParams): Promise<{ signature: string; goalAddress: Address; goalSlotId: bigint; taskAddress: Address; taskSlotId: bigint }> {
      const account = await fetchMaybeNetworkConfig(client.rpc, params.networkConfig);
      if (!account.exists || !account.data) {
        throw new Error('Network config not found');
      }
      const networkConfigData = account.data;

      const goalSlotId = networkConfigData.goalCount;
      const taskSlotId = networkConfigData.taskCount;
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, goalSlotId);
      const taskAddress = await deriveTaskAddress(programAddress, params.networkConfig, taskSlotId);

      const input: CreateGoalInput = {
        payer: params.payer,
        owner: params.owner,
        networkConfig: params.networkConfig,
        goal: goalAddress,
        task: taskAddress,
        isOwned: params.isOwned,
        isConfidential: params.isConfidential,
      };

      const instruction = getCreateGoalInstruction(input, {
        programAddress,
      });

      const signature = await sendTransaction(client, params.payer, [instruction]);

      return { signature, goalAddress, goalSlotId, taskAddress, taskSlotId };
    },

    async setGoal(params: SetGoalParams): Promise<string> {
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, params.goalSlotId);
      const agentAddress = await deriveAgentAddress(programAddress, params.networkConfig, params.agentSlotId);
      const taskAddress = await deriveTaskAddress(programAddress, params.networkConfig, params.taskSlotId);

      const input: SetGoalAsyncInput = {
        owner: params.owner,
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

      return await sendTransaction(client, params.owner, [instruction]);
    },

    async contributeToGoal(params: ContributeToGoalParams): Promise<string> {
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, params.goalSlotId);

      const input: ContributeToGoalAsyncInput = {
        contributor: params.contributor,
        goal: goalAddress,
        networkConfig: params.networkConfig,
        depositAmount: params.depositAmount,
      };

      const instruction = await getContributeToGoalInstructionAsync(input, {
        programAddress,
      });

      return await sendTransaction(client, params.contributor, [instruction]);
    },

    async withdrawFromGoal(params: WithdrawFromGoalParams): Promise<string> {
      const goalAddress = await deriveGoalAddress(programAddress, params.networkConfig, params.goalSlotId);

      const input: WithdrawFromGoalAsyncInput = {
        contributor: params.contributor,
        goal: goalAddress,
        networkConfig: params.networkConfig,
        sharesToBurn: params.sharesToBurn,
      };

      const instruction = await getWithdrawFromGoalInstructionAsync(input, {
        programAddress,
      });

      return await sendTransaction(client, params.contributor, [instruction]);
    },

    async updateNetworkConfig(params: UpdateNetworkConfigParams): Promise<string> {
      const networkConfigAddress = await deriveNetworkConfigAddress(
        programAddress,
        params.authority.address
      );

      const input: UpdateNetworkConfigAsyncInput = {
        authority: params.authority,
        networkConfig: networkConfigAddress,
        cidConfig: params.cidConfig ?? null,
        newCodeMeasurement: params.newCodeMeasurement ?? null,
      };

      const instruction = await getUpdateNetworkConfigInstructionAsync(input, {
        programAddress,
      });

      return await sendTransaction(client, params.authority, [instruction]);
    },

    async activateNode(params: ActivateNodeParams): Promise<string> {
      const networkConfigAddress = await deriveNetworkConfigAddress(
        programAddress,
        params.authority.address
      );
      const nodeInfoAddress = await deriveNodeInfoAddress(
        programAddress,
        params.nodePubkey
      );

      const instruction = getActivateNodeInstruction(
        {
          authority: params.authority,
          networkConfig: networkConfigAddress,
          nodeInfo: nodeInfoAddress,
        },
        {
          programAddress,
        }
      );

      return await sendTransaction(client, params.authority, [instruction]);
    },
  };
}
