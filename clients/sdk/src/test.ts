import { createSolanaClient, generateKeyPairSigner, address } from 'gill';
import { DacFrontendClient, NodeType, DAC_PROGRAM_ID } from './index.js';
import { deriveNetworkConfigAddress } from './dacPdas.js';
/**
 * Test file for DAC SDK functions
 * 
 * Usage:
 *   npm run dev src/test.ts
 *   or
 *   tsx src/test.ts
 */

// Configuration
const RPC_URL = process.env.RPC_URL || 'https://api.devnet.solana.com';

async function main() {
  console.log('🚀 Starting DAC SDK Tests\n');
  console.log(`RPC URL: ${RPC_URL}`);
  console.log(`Program ID: ${DAC_PROGRAM_ID}\n`);

  // Initialize Solana client
  const solanaClient = createSolanaClient({ urlOrMoniker: RPC_URL });
  const dacClient = new DacFrontendClient(solanaClient);

  // Generate test keypairs (these return promises)
  const authority = await generateKeyPairSigner();
  const nodeOwner = await generateKeyPairSigner();
  const nodeKeypair = await generateKeyPairSigner(); // Node's own keypair (generated on server)

  console.log('📝 Generated Test Keypairs:');
  console.log(`  Authority: ${authority.address}`);
  console.log(`  Node Owner: ${nodeOwner.address}`);
  console.log(`  Node Pubkey: ${nodeKeypair.address}\n`);

  // ============================================================================
  // Test 1: Initialize Network
  // ============================================================================
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('Test 1: Initialize Network');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  try {
    const networkConfigCid = 'QmTestNetworkConfig123456789'; // Example IPFS CID
    const allocateGoals = 10n;
    const allocateTasks = 20n;
    const approvedCodeMeasurements = [
      {
        measurement: new Uint8Array(32).fill(1), // Example measurement
        version: { major: 1, minor: 0, patch: 0 },
      },
    ];

    console.log('Parameters:');
    console.log(`  Network Config CID: ${networkConfigCid}`);
    console.log(`  Allocate Goals: ${allocateGoals}`);
    console.log(`  Allocate Tasks: ${allocateTasks}`);
    console.log(`  Approved Code Measurements: ${approvedCodeMeasurements.length}\n`);

    const { signature: initSignature, networkConfigAddress } =
      await dacClient.initializeNetwork({
        authority,
        cidConfig: networkConfigCid,
        allocateGoals,
        allocateTasks,
        approvedCodeMeasurements,
      });

    console.log('✅ Network initialized successfully!');
    console.log(`  Transaction Signature: ${initSignature}`);
    console.log(`  Network Config Address: ${networkConfigAddress}\n`);

    // Verify network config was created
    const networkConfig = await dacClient.getNetworkConfig(authority.address);
    if (networkConfig) {
      console.log('✅ Network config fetched:');
      console.log(`  Agent Count: ${networkConfig.agentCount}`);
      console.log(`  Goal Count: ${networkConfig.goalCount}`);
      console.log(`  Task Count: ${networkConfig.taskCount}`);
      console.log(`  Validator Node Count: ${networkConfig.validatorNodeCount}`);
      console.log(`  Compute Node Count: ${networkConfig.computeNodeCount}\n`);
    } else {
      console.log('⚠️  Network config not found (may need to wait for confirmation)\n');
    }
  } catch (error) {
    console.error('❌ Failed to initialize network:');
    console.error(error);
    process.exit(1);
  }

  // ============================================================================
  // Test 2: Register Node
  // ============================================================================
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('Test 2: Register Node');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  try {
    const networkConfigAddress = await dacClient.getNetworkConfig(authority.address);
    if (!networkConfigAddress) {
      throw new Error('Network config not found. Run initializeNetwork first.');
    }


    const networkConfigAddr = await deriveNetworkConfigAddress(DAC_PROGRAM_ID, authority.address);

    // Test registering a Compute node
    console.log('Registering Compute Node...');
    const { signature: registerSignature, nodeInfoAddress, nodeTreasuryAddress } =
      await dacClient.registerNode({
        owner: nodeOwner,
        networkConfig: networkConfigAddr,
        nodePubkey: nodeKeypair.address,
        nodeType: NodeType.Compute,
      });

    console.log('✅ Node registered successfully!');
    console.log(`  Transaction Signature: ${registerSignature}`);
    console.log(`  Node Info Address: ${nodeInfoAddress}`);
    console.log(`  Node Treasury Address: ${nodeTreasuryAddress}\n`);

    // Test registering a Validator node (with a different node keypair)
    const validatorNodeKeypair = await generateKeyPairSigner();
    console.log('Registering Validator Node...');
    console.log(`  Validator Node Pubkey: ${validatorNodeKeypair.address}`);

    const { signature: validatorSignature, nodeInfoAddress: validatorNodeInfoAddress } =
      await dacClient.registerNode({
        owner: nodeOwner,
        networkConfig: networkConfigAddr,
        nodePubkey: validatorNodeKeypair.address,
        nodeType: NodeType.Validator,
      });

    console.log('✅ Validator node registered successfully!');
    console.log(`  Transaction Signature: ${validatorSignature}`);
    console.log(`  Node Info Address: ${validatorNodeInfoAddress}\n`);
  } catch (error) {
    console.error('❌ Failed to register node:');
    console.error(error);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
