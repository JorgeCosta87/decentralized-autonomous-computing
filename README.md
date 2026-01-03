# Decentralized Autonomous Civilization

## Introduction

Decentralized autonomous civilization (DAC) network built on Solana. The protocol enables secure, verifiable execution of AI agent tasks using Trusted Execution Environments (TEEs) and implements a payment system for compute resources.

## Core Components

### Smart Contract (DAC)
The program running on Solana that manages all state objects and operations.

### Node Types
- **Validator Nodes**: Run in Intel SGX TEE, verify task executions
- **Compute Nodes**: Execute tasks for agents

### Data Storage
- **IPFS**: All task data and configurations files stored off-chain
- **On-Chain**: Only IPFS CIDs and cryptographic proofs stored

## Key Workflows

1. **Network Initialization**: Authority initializes network with approved code measurements and pre-allocates goals and tasks
2. **Node Registration**: Validators prove TEE hardware, compute nodes pass benchmark validation
3. **Agent Setup**: Create agents with validated configurations stored on IPFS
4. **Goal Creation**: Define objectives with treasury funding and iteration limits
5. **Task Execution**: Compute nodes claim tasks, execute with LLM, validators verify and trigger payments

## Security Features

- **TEE Attestation**: Validators prove genuine Intel SGX hardware
- **Code Measurement Whitelist**: Only pre-approved code can run
- **TEE Signatures**: All validations cryptographically signed
- **Chain Proofs**: SHA256 hashes verify data integrity

## Documentation

For detailed architecture, requirements, and specifications, see:
- [Design Document](./docs/design.md) - Complete architecture and design specifications
