# Test Keypair Generation Scripts

## generate_keypair.sh

The bash script in `agent-network/scripts/generate_keypair.sh` generates **all** keypairs needed for DAC testing and airdrops 10 SOL to each address.

### Generated Keypairs:
- **public-node**: Public node keypair (saved to `keypairs/` folder in project root)
- **confidential-node**: Confidential node keypair (saved to `keypairs/` folder in project root)
- **authority**: Network authority keypair (saved to `keypairs/` folder in project root)
- **node-owner**: Owner of nodes (saved to `keypairs/` folder in project root)
- **validator-owner**: Owner of confidential node (saved to `keypairs/` folder in project root)

### Usage:

```bash
# From project root
cd agent-network && ./scripts/generate_keypair.sh

# Or from agent-network directory
./scripts/generate_keypair.sh
```

### Output:

All keypairs saved to `keypairs/` folder in project root:
  - `keypairs/public-node-keypair.json`
  - `keypairs/confidential-node-keypair.json`
  - `keypairs/authority-keypair.json`
  - `keypairs/node-owner-keypair.json`
  - `keypairs/validator-owner-keypair.json`
  - `keypairs/keypairs.json` (summary with all pubkeys)

- **Airdrop**: Automatically airdrops 10 SOL to all 5 addresses

## load-test-keypairs.ts

Helper module to load generated keypairs in test files. Located in `src/load-test-keypairs.ts`.

### Usage:

```typescript
import { loadTestKeypairs } from './load-test-keypairs.js';

const keypairs = await loadTestKeypairs();
// Use keypairs.authority, keypairs.nodeOwner, etc.
```

### Example:

```typescript
import { loadTestKeypairs } from './load-test-keypairs.js';
import { DacSDK } from './index.js';

const keypairs = await loadTestKeypairs();

// Use in tests
const result = await dacClient.initializeNetwork({
  authority: keypairs.authority,
  // ...
});
```

## Notes:

- **Recommended**: Use `generate_keypair.sh` to generate all keypairs at once
- Keypairs are saved in Solana format (64-byte array) compatible with Rust nodes
- The `keypairs/` directory should be git-ignored for security
- Regenerate keypairs if you need fresh ones for testing
