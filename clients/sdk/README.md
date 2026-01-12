# Decentralized Autonomous Civilization (DAC) TypeScript SDK

TypeScript SDK for interacting with the Decentralized Autonomous Civilization (DAC) smart contract, IPFS, and node operations.

## Installation

```bash
npm install
```

## Components

### DacSDK
Interact with the DAC smart contract on Solana. This client provides methods for frontend/UI operations only. Node operations (like claimTask, submitTaskResult, etc.) are handled by separate node clients.

```typescript
import { DacSDK } from './dacClient';
import { createSolanaClient } from 'gill';

const client = createSolanaClient('https://api.mainnet-beta.solana.com');
const dacClient = new DacSDK(client);

// Initialize network
const { signature, networkConfigAddress } = await dacClient.initializeNetwork({
  authority: myKeypair,
  cidConfig: 'QmNetworkConfig...',
  allocateGoals: 10n,
  allocateTasks: 10n,
  approvedCodeMeasurements: [...],
  requiredValidations: 1
});

// Create an agent
const { signature, agentAddress, agentSlotId } = await dacClient.createAgent({
  agentOwner: myKeypair,
  networkConfig: networkConfigAddress,
  agentConfigCid: 'QmXXX...'
});

// Create a goal (public or confidential)
const { signature, goalAddress, goalSlotId, taskAddress, taskSlotId } = await dacClient.createGoal({
  payer: myKeypair,
  owner: myKeypair,
  networkConfig: networkConfigAddress,
  isOwned: true,
  isConfidential: false
});
```

### Status Waiting Methods

Wait for nodes or agents to reach a specific status using WebSocket subscriptions.

```typescript
import { DacSDK, WaitMode } from './dacClient';

// Wait for nodes to reach a specific status
const nodes = await dacClient.waitForNodesStatus(
  [nodePubkey1, nodePubkey2],
  NodeStatus.Active,
  {
    timeoutMs: 30000,
    waitMode: WaitMode.All
  }
);

// Wait for agents to reach a specific status
const agents = await dacClient.waitForAgentsStatus(
  [agentAddress1, agentAddress2],
  AgentStatus.Validated,
  {
    timeoutMs: 30000,
    waitMode: WaitMode.First
  }
);
```

**WaitMode Options:**
- `WaitMode.All`: Wait for all specified nodes/agents to reach the target status. Default.
- `WaitMode.First`: Return when the first node/agent reaches the target status.

### IPFSClient
Upload and download data from IPFS. Files are automatically added to MFS (Mutable File System).

```typescript
import { IpfsClient } from './ipfsClient';

const ipfsClient = new IpfsClient({
  apiUrl: 'http://localhost:5001',
});

// Upload data
const cid = await ipfsClient.upload('Hello World', 'hello.txt');

// Download data
const data = await ipfsClient.download(cid);

// List all pinned files with access URLs
const files = await ipfsClient.listPinnedWithDetails();
files.forEach(file => {
  console.log(file.gatewayUrl);
});

// List files in MFS (visible in WebUI)
const mfsFiles = await ipfsClient.listMfsFiles('/dac-uploads');
```

**File Tracking:**
- Files uploaded via API are added to `/dac-uploads/YYYY-MM-DD/` in MFS
- View in WebUI: `http://localhost:5001/webui → Files → dac-uploads`
- List files: `npm run list-ipfs`
- Gateway access: `http://localhost:8080/ipfs/<CID>`

## Usage Examples

### Complete Workflow

```typescript
import { DacSDK, WaitMode } from './dacClient';
import { IpfsClient } from './ipfsClient';
import { createSolanaClient } from 'gill';

// 1. Initialize clients
const solanaClient = createSolanaClient('https://api.mainnet-beta.solana.com');
const dacClient = new DacSDK(solanaClient);
const ipfsClient = new IpfsClient({ apiUrl: 'http://localhost:5001' });

// 2. Initialize network
const { signature, networkConfigAddress } = await dacClient.initializeNetwork({
  authority: authorityKeypair,
  cidConfig: 'QmNetworkConfig...',
  allocateGoals: 10n,
  allocateTasks: 100n,
  approvedCodeMeasurements: [...],
  requiredValidations: 1
});

// 3. Upload agent config to IPFS
const agentConfig = { name: 'My Agent', model: 'gpt-4' };
const agentConfigCid = await ipfsClient.upload(JSON.stringify(agentConfig));

// 4. Create agent
const { signature, agentAddress, agentSlotId } = await dacClient.createAgent({
  agentOwner: agentOwnerKeypair,
  networkConfig: networkConfigAddress,
  agentConfigCid: agentConfigCid
});

// 5. Wait for agent to be validated
const validatedAgents = await dacClient.waitForAgentsStatus(
  [agentAddress],
  AgentStatus.Validated,
  {
    timeoutMs: 60000,
    waitMode: WaitMode.All
  }
);

// 6. Create a goal
const { signature, goalAddress, goalSlotId, taskAddress, taskSlotId } = await dacClient.createGoal({
  payer: goalOwnerKeypair,
  owner: goalOwnerKeypair,
  networkConfig: networkConfigAddress,
  isOwned: true,
  isConfidential: false
});

// 7. Set goal specification
await dacClient.setGoal({
  owner: goalOwnerKeypair,
  networkConfig: networkConfigAddress,
  goalSlotId: goalSlotId,
  agentSlotId: agentSlotId,
  taskSlotId: taskSlotId,
  specificationCid: 'QmGoalSpec...',
  maxIterations: 10n,
  initialDeposit: 1000000000n
});

// 8. Wait for nodes to become active
const activeNodes = await dacClient.waitForNodesStatus(
  [nodePubkey1, nodePubkey2],
  NodeStatus.Active,
  {
    timeoutMs: 30000,
    waitMode: WaitMode.All
  }
);
```

### Querying Data

```typescript
// Get network configuration
const networkConfig = await dacClient.getNetworkConfig(authorityAddress);

// Get agent by address
const agent = await dacClient.getAgent(agentAddress);

// Get agent by slot ID
const agent = await dacClient.getAgentBySlot(networkConfigAddress, 0n);

// Get all agents by status
const pendingAgents = await dacClient.getAgentsByStatus(AgentStatus.Pending);
const validatedAgents = await dacClient.getAgentsByStatus(AgentStatus.Validated);

// Get all nodes by status
const activeNodes = await dacClient.getNodesByStatus({ status: NodeStatus.Active });
const publicNodes = await dacClient.getNodesByStatus({ 
  status: NodeStatus.Active, 
  nodeType: NodeType.Public 
});

// Get goal and task
const goal = await dacClient.getGoal(networkConfigAddress, goalSlotId);
const task = await dacClient.getTask(networkConfigAddress, taskSlotId);
```
