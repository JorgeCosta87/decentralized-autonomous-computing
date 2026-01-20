import { PublicKey } from '@solana/web3.js';
import { address, type Address } from '@solana/kit';

export const DAC_PROGRAM_ID = address('BaY9vp3RXAQugzAoBojkBEZs9fJKS4dNManN7vwDZSFh');

/**
 * Convert bigint to 8-byte little-endian buffer (browser-compatible)
 */
function bigintToBufferLE(value: bigint): Uint8Array {
  const buffer = new Uint8Array(8);
  // Manual conversion - always use this for browser compatibility
  let n = value;
  for (let i = 0; i < 8; i++) {
    buffer[i] = Number(n & 0xffn);
    n = n >> 8n;
  }
  return buffer;
}

export async function deriveNetworkConfigAddress(
  programAddress: Address,
  authority: Address
): Promise<Address> {
  const [networkConfigAddress] = PublicKey.findProgramAddressSync(
    [
      new TextEncoder().encode('dac_network_config'),
      new PublicKey(authority).toBuffer(),
    ],
    new PublicKey(programAddress)
  );
  return address(networkConfigAddress.toBase58());
}

export async function deriveAgentAddress(
  programAddress: Address,
  networkConfig: Address,
  agentSlotId: bigint
): Promise<Address> {
  const slotIdBuffer = bigintToBufferLE(agentSlotId);
  const [agentAddress] = PublicKey.findProgramAddressSync(
    [
      new TextEncoder().encode('agent'),
      new PublicKey(networkConfig).toBuffer(),
      slotIdBuffer,
    ],
    new PublicKey(programAddress)
  );
  return address(agentAddress.toBase58());
}

export async function deriveGoalAddress(
  programAddress: Address,
  networkConfig: Address,
  goalSlotId: bigint
): Promise<Address> {
  const slotIdBuffer = bigintToBufferLE(goalSlotId);
  const [goalAddress] = PublicKey.findProgramAddressSync(
    [
      new TextEncoder().encode('goal'),
      new PublicKey(networkConfig).toBuffer(),
      slotIdBuffer,
    ],
    new PublicKey(programAddress)
  );
  return address(goalAddress.toBase58());
}

export async function deriveTaskAddress(
  programAddress: Address,
  networkConfig: Address,
  taskSlotId: bigint
): Promise<Address> {
  const slotIdBuffer = bigintToBufferLE(taskSlotId);
  const [taskAddress] = PublicKey.findProgramAddressSync(
    [
      new TextEncoder().encode('task'),
      new PublicKey(networkConfig).toBuffer(),
      slotIdBuffer,
    ],
    new PublicKey(programAddress)
  );
  return address(taskAddress.toBase58());
}

export async function deriveGoalVaultAddress(
  programAddress: Address,
  goal: Address
): Promise<Address> {
  const [vaultAddress] = PublicKey.findProgramAddressSync(
    [
      new TextEncoder().encode('goal_vault'),
      new PublicKey(goal).toBuffer(),
    ],
    new PublicKey(programAddress)
  );
  return address(vaultAddress.toBase58());
}

export async function deriveContributionAddress(
  programAddress: Address,
  goal: Address,
  contributor: Address
): Promise<Address> {
  const [contributionAddress] = PublicKey.findProgramAddressSync(
    [
      new TextEncoder().encode('contribution'),
      new PublicKey(goal).toBuffer(),
      new PublicKey(contributor).toBuffer(),
    ],
    new PublicKey(programAddress)
  );
  return address(contributionAddress.toBase58());
}

export async function deriveNodeInfoAddress(
  programAddress: Address,
  nodePubkey: Address
): Promise<Address> {
  const [nodeInfoAddress] = PublicKey.findProgramAddressSync(
    [
      new TextEncoder().encode('node_info'),
      new PublicKey(nodePubkey).toBuffer(),
    ],
    new PublicKey(programAddress)
  );
  return address(nodeInfoAddress.toBase58());
}
