# Sol-Mind Protocol - System Overview

## Introduction

Sol-Mind Protocol is a decentralized autonomous computing (DAC) network built on Solana. The protocol enables secure, verifiable execution of AI agent tasks using Trusted Execution Environments (TEEs) and implements a payment system for compute resources.

## Core Components

### Smart Contract (DAC)
The program running on Solana that manages all state objects and operations.

### State Objects
On-chain accounts storing system state:
- **NetworkConfig**: Global configuration, approved code measurements, node and resource counts
- **NodeInfo**: Per-node information, TEE data, and status
- **Agent**: Agent configuration and memory CIDs
- **Goal**: Goal definition, progress, and treasury
- **TaskData**: Task execution state, input/output CIDs, and chain proofs
- **GoalTreasury**: Payment escrow PDAs for each goal

### Node Types
- **Validator Nodes**: Run in Intel SGX TEE, verify task executions
- **Compute Nodes**: Execute tasks for agents

### Data Storage
- **IPFS**: All task data stored off-chain
- **On-Chain**: Only IPFS CIDs and cryptographic proofs stored

## Key Workflows

### 1. Network Initialization
Authority initializes network with approved code measurements and pre-allocates goals and tasks.

### 2. Node Registration
- **Validator Nodes**: Register → Claim with TEE attestation → Active
- **Compute Nodes**: Register → Claim → Await validation → Active
- Nodes subscribe to account changes via RPC for real-time event handling

### 3. Agent & Goal Setup
- Create agent with config CID (validated by validator)
- Select available agent for goal
- Create goal with treasury deposit

### 4. Task Execution
1. Agent submits task with input CID
2. Compute node subscribes to TaskData changes via RPC
3. Compute node claims task and executes with LLM
4. Compute node submits output CID
5. Validator subscribes to TaskData status changes via RPC
6. Validator validates and transfers payment

### 5. Payment System
- Each goal has a treasury PDA
- Validator determines payment amount per task
- Payment transferred automatically on successful validation

## Security Features

- **TEE Attestation**: Validators prove genuine Intel SGX hardware
- **Code Measurement Whitelist**: Only pre-approved code can run
- **TEE Signatures**: All validations cryptographically signed
- **Chain Proofs**: SHA256 hashes verify data integrity

## Documentation

### State Diagrams
- [Agent States](./diagrams/agent-states.md#agent-state-machine)
- [Goal States](./diagrams/goal-states.md#goal-state-machine)
- [Task States](./diagrams/task-states.md#task-state-machine)
- [NodeInfo States](./diagrams/node-registration.md#nodeinfo-state-machine)

### Workflow Diagrams
- [Network Initialization](./diagrams/network-initialization.md)
- [Node Registration](./diagrams/node-registration.md)
- [Task Execution](./diagrams/task-execution.md)
- [Validation Flow](./diagrams/validation-flow.md)
- [Payment System](./diagrams/payment-system.md)
