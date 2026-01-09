import {
  address,
  getAddressEncoder,
  getU64Encoder,
  getProgramDerivedAddress,
  type Address,
} from 'gill';

export const DAC_PROGRAM_ID = address('BaY9vp3RXAQugzAoBojkBEZs9fJKS4dNManN7vwDZSFh');

export async function deriveNetworkConfigAddress(
  programAddress: Address
): Promise<Address> {
  const [networkConfigAddress] = await getProgramDerivedAddress({
    programAddress,
    seeds: [Buffer.from('dac_network_config')],
  });
  return networkConfigAddress;
}

export async function deriveAgentAddress(
  programAddress: Address,
  networkConfig: Address,
  agentSlotId: bigint
): Promise<Address> {
  const [agentAddress] = await getProgramDerivedAddress({
    programAddress,
    seeds: [
      Buffer.from('agent'),
      getAddressEncoder().encode(networkConfig),
      Buffer.from(getU64Encoder().encode(agentSlotId)),
    ],
  });
  return agentAddress;
}

export async function deriveGoalAddress(
  programAddress: Address,
  networkConfig: Address,
  goalSlotId: bigint
): Promise<Address> {
  const [goalAddress] = await getProgramDerivedAddress({
    programAddress,
    seeds: [
      Buffer.from('goal'),
      getAddressEncoder().encode(networkConfig),
      Buffer.from(getU64Encoder().encode(goalSlotId)),
    ],
  });
  return goalAddress;
}

export async function deriveTaskAddress(
  programAddress: Address,
  networkConfig: Address,
  taskSlotId: bigint
): Promise<Address> {
  const [taskAddress] = await getProgramDerivedAddress({
    programAddress,
    seeds: [
      Buffer.from('task'),
      getAddressEncoder().encode(networkConfig),
      Buffer.from(getU64Encoder().encode(taskSlotId)),
    ],
  });
  return taskAddress;
}

export async function deriveGoalVaultAddress(
  programAddress: Address,
  goal: Address
): Promise<Address> {
  const [vaultAddress] = await getProgramDerivedAddress({
    programAddress,
    seeds: [Buffer.from('goal_vault'), getAddressEncoder().encode(goal)],
  });
  return vaultAddress;
}

export async function deriveContributionAddress(
  programAddress: Address,
  goal: Address,
  contributor: Address
): Promise<Address> {
  const [contributionAddress] = await getProgramDerivedAddress({
    programAddress,
    seeds: [
      Buffer.from('contribution'),
      getAddressEncoder().encode(goal),
      getAddressEncoder().encode(contributor),
    ],
  });
  return contributionAddress;
}

export async function deriveNodeInfoAddress(
  programAddress: Address,
  nodePubkey: Address
): Promise<Address> {
  const [nodeInfoAddress] = await getProgramDerivedAddress({
    programAddress,
    seeds: [Buffer.from('node_info'), getAddressEncoder().encode(nodePubkey)],
  });
  return nodeInfoAddress;
}
