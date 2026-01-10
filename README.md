# Decentralized Autonomous Civilization

## Introduction

The Decentralized Autonomous Civilization (DAC) is a Solana-based network that enables verifiable, decentralized execution of AI agent tasks with granular payment management. The protocol supports both **public nodes** (standard execution) and **confidential nodes** (TEE-enabled for private data processing), implementing a multi-validator consensus system to ensure trust and correctness.

**Key Features:**
- **Dual Node Architecture**: Public nodes for standard tasks, confidential nodes (Intel SGX TEE) for private data processing
- **Multi-Validator Consensus**: Agents, nodes, and tasks require multiple validations before approval/processing
- **Share-Based Payment System**: Proportional cost distribution with automatic refunds and granular contribution tracking
- **Goal-Oriented Execution**: Define objectives with iteration limits, treasury funding, and agent assignments
- **Cryptographic Integrity**: SHA256 chain proofs verify complete execution history
- **Decentralized Validation**: Any active node (public or confidential) can validate task execution results

## Core Components

### Smart Contract (DAC)
The program running on Solana that manages all state objects and operations.

**Program Address (Devnet)**: [`BaY9vp3RXAQugzAoBojkBEZs9fJKS4dNManN7vwDZSFh`](https://solscan.io/account/BaY9vp3RXAQugzAoBojkBEZs9fJKS4dNManN7vwDZSFh?cluster=devnet)

### Node Types
- **Public Nodes**: Standard nodes that can execute public tasks and validate any tasks/nodes
- **Confidential Nodes**: Run in Intel SGX TEE (Trusted Execution Environment), can execute both public and confidential tasks, and validate any tasks/nodes
- **Key Distinction**: Only confidential nodes can **CLAIM/EXECUTE** confidential tasks (TEE protection for private data). However, **any active node** (public or confidential) can **VALIDATE** task execution results and other public nodes.

### Data Storage
- **IPFS**: All task data and configurations files stored off-chain
- **On-Chain**: Only IPFS CIDs and cryptographic proofs stored

## Key Workflows

1. **Network Initialization**: Authority initializes network with approved code measurements and pre-allocates goals and tasks
2. **Node Registration**: 
   - Public nodes register and must pass benchmark validation by multiple validators
   - Confidential nodes register and prove TEE hardware (self-approved via TEE attestation)
3. **Agent Setup**: Create agents with validated configurations stored on IPFS (requires multi-validator consensus)
4. **Goal Creation**: Define objectives with treasury funding and iteration limits (can be public or confidential)
5. **Task Execution**: 
   - Nodes claim tasks (confidential tasks only claimable by confidential nodes)
   - Nodes execute tasks with LLM
   - Any active node validates execution results (multi-validator consensus required)
   - Payments triggered when validation threshold reached

## Security Features

- **TEE Attestation**: Confidential nodes prove genuine Intel SGX hardware
- **Code Measurement Whitelist**: Only pre-approved code can run in TEE
- **Multi-Validator Consensus**: Agents, nodes, and tasks require multiple validations before approval/processing
- **TEE Signatures**: Confidential task validations cryptographically signed (only confidential nodes can provide)
- **Chain Proofs**: SHA256 hashes verify data integrity across all executions
- **Access Control**: Confidential goals can only be claimed/executed by confidential nodes (TEE protection)

## Building and Testing

### Build
Build the Solana program and generate clients:
```bash
anchor run build
```
This command:
- Compiles the Solana program
- Generates TypeScript and Rust clients in `clients/` directory
- Creates the program IDL

### Test
Run the test suite:
```bash
anchor run test
```

Or use the verbose flag directly:
```bash
anchor test run --verbose
```

## Documentation

For detailed architecture, requirements, and specifications, see:
- [Design Document](./docs/design.md) - Complete architecture and design specifications
- [User Stories](./docs/user-stories.md) - Detailed user stories and technical implementation