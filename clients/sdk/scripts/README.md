# Test Keypair Generation Scripts

## generate-test-keypairs.ts

Generates test keypairs for DAC testing and saves them in Solana format.

### Generated Keypairs:
- **authority**: Network authority keypair
- **node-owner**: Owner of compute/validator nodes
- **compute-node**: Compute node's own keypair
- **validator-owner**: Owner of validator node
- **validator-node**: Validator node's own keypair

### Usage:

```bash
# Run the script (recommended)
npm run generate-keypairs

# Or use npx directly
npx tsx scripts/generate-test-keypairs.ts
```

### Output:

The script creates a `test-keypairs/` directory with:
- `{name}.json` - Solana keypair format (64 bytes array) - compatible with Rust nodes
- `{name}.info.json` - Readable format with address
- `keypairs-summary.json` - Summary of all addresses

### Files Created:
- `authority.json` / `authority.info.json`
- `node-owner.json` / `node-owner.info.json`
- `compute-node.json` / `compute-node.info.json`
- `validator-owner.json` / `validator-owner.info.json`
- `validator-node.json` / `validator-node.info.json`
- `keypairs-summary.json`

## load-test-keypairs.ts

Helper module to load generated keypairs in test files.

### Usage:

```typescript
import { loadTestKeypairs } from './scripts/load-test-keypairs.js';

const keypairs = await loadTestKeypairs();
// Use keypairs.authority, keypairs.nodeOwner, etc.
```

### Example:

```typescript
import { loadTestKeypairs } from './scripts/load-test-keypairs.js';
import { DacFrontendClient } from './index.js';

const keypairs = await loadTestKeypairs();

// Use in tests
const result = await dacClient.initializeNetwork({
  authority: keypairs.authority,
  // ...
});
```

## Notes:

- Keypairs are saved in Solana format (64-byte array) compatible with Rust nodes
- The `test-keypairs/` directory is git-ignored for security
- Regenerate keypairs if you need fresh ones for testing
