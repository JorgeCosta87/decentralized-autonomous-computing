# Sol-Mind Protocol TypeScript SDK

TypeScript SDK for interacting with the Sol-Mind Protocol DAC smart contract, IPFS, and node operations.

## Installation

```bash
npm install @sol-mind-protocol/sdk
```

## Components

### DacManagerClient
Interact with the DAC smart contract on Solana.

```typescript
import { DacManagerClient } from '@sol-mind-protocol/sdk';
import { createSolanaClient } from 'gill';

const client = createSolanaClient({ rpc: 'https://api.mainnet-beta.solana.com' });
const dacClient = new DacManagerClient(client);

// Create agent
const { agentAddress } = await dacClient.createAgent({
  payer: signer,
  owner: signer,
  agentId: 0n,
  computeNodeOwner: computeNodeOwner,
  computeNodePubkey: computeNodePubkey,
  public: true,
  allocatedGoals: 10,
  allocatedTasks: 100,
});
```

### IPFSClient
Upload and download data from IPFS. Files are automatically added to MFS (Mutable File System) so they appear in WebUI.

```typescript
import { IPFSClient } from '@sol-mind-protocol/sdk';

const ipfsClient = new IPFSClient({
  apiUrl: 'http://localhost:5001',
});

// Upload data (automatically added to MFS for WebUI visibility)
const cid = await ipfsClient.upload('Hello World', 'hello.txt');

// Download data
const data = await ipfsClient.download(cid);

// List all pinned files with access URLs
const files = await ipfsClient.listPinnedWithDetails();
files.forEach(file => {
  console.log(file.gatewayUrl); // http://localhost:8080/ipfs/<CID>
});

// List files in MFS (visible in WebUI)
const mfsFiles = await ipfsClient.listMfsFiles('/dac-uploads');
```

**File Tracking:**
- Files uploaded via API are automatically added to `/dac-uploads/YYYY-MM-DD/` in MFS
- View in WebUI: `http://localhost:5001/webui → Files → dac-uploads`
- List all files: `npm run list-ipfs`
- Access via Gateway: `http://localhost:8080/ipfs/<CID>`

### ValidatorNodeClient
TEE-based validator node operations.

```typescript
import { ValidatorNodeClient } from '@sol-mind-protocol/sdk';

const validatorClient = new ValidatorNodeClient(
  dacClient,
  ipfsClient,
  {
    nodePubkey: validatorPubkey,
    teeSigningKey: teePrivateKey,
  },
  client
);

// Subscribe to task validations
validatorClient.subscribeToTaskValidation(taskDataAddress, (taskData) => {
  // Validate task
  await validatorClient.validateTask({...});
});
```

### ComputeNodeClient
Compute node for task execution.

```typescript
import { ComputeNodeClient } from '@sol-mind-protocol/sdk';

const computeClient = new ComputeNodeClient(
  dacClient,
  ipfsClient,
  {
    nodePubkey: computeNodePubkey,
    llmProvider: 'https://api.openai.com/v1/completions',
    llmApiKey: 'your-api-key',
  },
  client
);

// Subscribe to tasks
computeClient.subscribeToTasks(agentAddress, async (taskData) => {
  // Claim and execute task
  await computeClient.claimAndExecuteTask({
    taskDataAddress,
    agentAddress,
    goalAddress,
    payer: signer,
    computeNode: signer,
  });
});
```

## Usage Examples

### Complete Workflow

```typescript
import {
  DacManagerClient,
  IPFSClient,
  ValidatorNodeClient,
  ComputeNodeClient,
} from '@sol-mind-protocol/sdk';

// 1. Initialize clients
const solanaClient = createSolanaClient({ rpc: RPC_URL });
const dacClient = new DacManagerClient(solanaClient);
const ipfsClient = new IPFSClient({ apiUrl: IPFS_API_URL, apiKey: IPFS_API_KEY });

// 2. Upload agent config to IPFS
const agentConfig = { name: 'My Agent', model: 'gpt-4' };
const agentConfigCid = await ipfsClient.upload(agentConfig);

// 3. Create agent
const { agentAddress } = await dacClient.createAgent({
  payer: signer,
  owner: signer,
  agentId: 0n,
  computeNodeOwner: computeNodeOwner,
  computeNodePubkey: computeNodePubkey,
  public: true,
  allocatedGoals: 10,
  allocatedTasks: 100,
});

// 4. Setup compute node
const computeClient = new ComputeNodeClient(
  dacClient,
  ipfsClient,
  { nodePubkey: computeNodePubkey, llmProvider: LLM_URL },
  solanaClient
);

// 5. Setup validator node
const validatorClient = new ValidatorNodeClient(
  dacClient,
  ipfsClient,
  { nodePubkey: validatorPubkey, teeSigningKey: teeKey },
  solanaClient
);

// 6. Compute node subscribes to tasks
computeClient.subscribeToTasks(agentAddress, async (taskData) => {
  await computeClient.claimAndExecuteTask({...});
});

// 7. Validator subscribes to validations
validatorClient.subscribeToTaskValidation(taskDataAddress, async (taskData) => {
  await validatorClient.validateTask({...});
});
```
