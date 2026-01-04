# Decentralized Autonomous Civilization - Design Document

## Overview

The Decentralized Autonomous Civilization (DAC) is a Solana-based network enabling verifiable AI agent execution with granular payment management. 

## Problem Statement

Existing AI agent systems face several critical challenges:

1. **Lack of Trust**: No verifiable way to ensure AI computations are executed correctly and securely
2. **Centralization**: AI infrastructure is controlled by centralized entities, creating single points of failure and censorship
3. **Payment Friction**: No standardized, automated payment mechanism for compute resources in AI agent systems
4. **No Provenance**: Difficulty in tracking and verifying the full execution history of AI tasks
5. **Resource Allocation**: Inefficient matching of compute resources with AI workloads

## Requirements

### Functional Requirements

1. **Network Initialization**
   - Authority can initialize network with approved TEE code measurements and network configuration
   - Pre-allocation of goal and task slots for efficient resource management

2. **Node Management**
   - Validator nodes must prove genuine Intel SGX hardware through attestation
   - Compute nodes must pass benchmark validation before accepting tasks
   - Real-time node status tracking and event subscription

3. **Agent Management**
   - Agents must be validated before becoming active
   - Agent configuration stored off-chain (IPFS) with on-chain CID references
   - Agent memory state persistence across task executions

4. **Goal & Task Execution**
   - Goals define objectives with iteration limits and treasury funding
   - Tasks are pre-allocated and reused across goal iterations
   - Compute nodes claim and execute tasks using LLM
   - Validators verify task execution and determine completion

5. **Payment System**
   - Each goal has an associated vault (SystemAccount PDA)
   - Granular contribution tracking per contributor
   - Contributors can deposit and withdraw at any time before goal completion
   - Batch reward transfers to reduce transaction costs
   - Proportional refund system based on when funds entered
   - Full refunds available on goal cancellation
   - Vault balance checks before computing phase transition

### Non-Functional Requirements

1. **Security**
   - All validator operations must run in Intel SGX TEE
   - TEE attestation required for validator registration
   - Code measurement whitelist enforcement
   - Cryptographic signatures for all validations
   - SHA256 chain proofs for data integrity

2. **Performance**
   - Event-driven architecture using Solana account subscriptions
   - Off-chain data storage (IPFS) to minimize on-chain state
   - Pre-allocated task slots to avoid repeated account creation
   - Batch reward transfers to reduce per-task transaction costs

3. **Scalability**
   - Support for multiple validators and compute nodes
   - Configurable task and goal allocation limits
   - Reusable task accounts across goal iterations

4. **Reliability**
   - Task retry mechanism for failed executions
   - Node status tracking (Active/Disabled/Rejected)
   - Vault balance validation before computing phase
   - Proportional refund system ensures fair cost distribution
   - Goal cancellation with full refunds available

## Design

### Architecture Overview

The system follows a modular architecture with three main layers:

1. **On-Chain Layer (Solana Smart Contract)**
   - Manages all account state and transitions
   - Enforces security rules and payment logic
   - Provides event notifications via account changes

2. **Off-Chain Storage (IPFS)**
   - Stores agent configurations, memory state, and task data
   - Only CIDs stored on-chain for data integrity

3. **Node Layer**
   - **Validator Nodes**: Run in TEE, validate tasks, determine payments
   - **Compute Nodes**: Execute tasks using LLM, submit results

### Key Design Decisions

1. **Pre-Allocated Task Slots**: Tasks are created during network initialization and reused, avoiding repeated account creation overhead

2. **Event-Driven Architecture**: Nodes subscribe to Solana account changes for real-time notifications, eliminating polling

3. **Separation of Concerns**: Task execution (Compute Nodes) is separate from validation (Validator Nodes)

4. **TEE-Only Validation**: Only validators with proven TEE hardware can validate tasks and determine payments

5. **Vault-Per-Goal**: Each goal has its own vault SystemAccount PDA for isolated payment management

6. **Batch Reward Transfers**: Rewards accumulate and transfer in batches to reduce transaction costs

7. **Proportional Refund System**: Contributors only pay for compute that occurred after their contribution, ensuring fair cost distribution

### Data Flow

1. **Goal Creation & Funding**: Owner creates goal → initializes vault → optional initial deposit → contributors add funds
2. **Contribution Phase**: Contributors can deposit or withdraw at any time while goal is Active
3. **Computing Phase**: Tasks execute while goal remains Active (funds continue to be available for withdrawals)
4. **Task Execution**: Compute node claims task → executes with LLM → submits result
5. **Reward Recording**: Validator verifies result → records reward in batch → auto-transfers when threshold reached
6. **Goal Completion**: Validator detects goal completion in submit_task_validation() → automatically processes refunds → goal returns to Ready (can be reused)
7. **Refund Phase**: Automatic proportional refunds processed when goal completes or is cancelled, then goal returns to Ready

## Architecture Specification


### NetworkConfig

Global network configuration account that tracks system-wide statistics and approved code measurements. This is the root account initialized once during network setup.

The NetworkConfig PDA stores:
- `authority`: Public key of the network authority
- `network_config_cid`: IPFS CID of network configuration
- `genesis_hash`: SHA256 hash that initializes all chain proofs (computed as `SHA256("DAC_GENESIS")`)
- `agent_count`: Current number of registered agents
- `goal_count`: Current number of goals
- `task_count`: Current number of tasks
- `validator_node_count`: Current number of active validator nodes
- `compute_node_count`: Current number of active compute nodes
- `approved_code_measurements`: Vector of approved TEE code measurements (max 10)
  - Each entry contains: `measurement` (32 bytes) and `version` (semantic version: major.minor.patch)
  - Newest measurements are always at the beginning (index 0)
  - When adding a new measurement and vector is full, oldest measurement is removed
  - Versions use semantic versioning (major: u16, minor: u16, patch: u16)
- `bump`: NetworkConfig PDA bump seed

Seeds: `["dac_network_config"]`

#### Sequence

```mermaid
sequenceDiagram
    participant Auth as Authority
    participant DAC as Smart Contract
    
    Auth->>DAC: initialize_network(cid_config, allocate_goals, allocate_tasks, approved_code_measurements)
    DAC->>DAC: Compute genesis_hash = SHA256("DAC_GENESIS")
    DAC->>DAC: Create NetworkConfig<br/>Set authority = authority<br/>Set genesis_hash = genesis_hash<br/>Set agent_count = 0<br/>goal_count = allocate_goals<br/>task_count = allocate_tasks<br/>Store approved_code_measurements
    
    loop For each goal (0..allocate_goals)
        DAC->>DAC: Create Goal PDA<br/>goal_slot_id = goal_id<br/>status = Ready<br/>chain_proof = genesis_hash
    end
    
    loop For each task (0..allocate_tasks)
        DAC->>DAC: Create Task PDA<br/>task_slot_id = task_id<br/>status = Ready<br/>chain_proof = genesis_hash
    end
    
    DAC->>Auth: Network initialized
```

### NodeInfo

Per-node account that stores registration information, TEE attestation data, node status, and reward tracking. Each validator or compute node has its own NodeInfo account.

The NodeInfo PDA stores:
- `node_pubkey`: Public key of the node
- `node_type`: Type of node (Validator or Compute)
- `status`: Current status of the node
- `node_info_cid`: IPFS CID of node metadata (for compute nodes)
- `code_measurement`: TEE code measurement (for validator nodes)
- `tee_signing_pubkey`: TEE signing public key (for validator nodes)
- `node_treasury`: Node treasury PDA address (SystemAccount for receiving payments)
- `recent_rewards`: Vector of recent reward entries awaiting batch transfer (max 64)
- `total_earned`: Cumulative SOL earned by the node
- `max_entries_before_transfer`: Batch size threshold for auto-transfer (default: 10)
- `last_transfer_slot`: Slot number of last auto-transfer
- `total_tasks_completed`: Total number of tasks completed by this node
- `bump`: NodeInfo PDA bump seed

Seeds: `["node_info", node_pubkey]`

#### Node Treasury

The node treasury is a **SystemAccount PDA** (not a data account) that receives batch transfers of rewards.

Seeds: `["node_treasury", node_info.key()]`

#### State

```mermaid
stateDiagram-v2
    [*] --> PendingClaim: register_node()
    PendingClaim --> AwaitingValidation: claim_compute_node()
    PendingClaim --> Active: claim_validator_node()<br/>(TEE verified)
    AwaitingValidation --> Active: validate_compute_node()<br/>(approved)
    AwaitingValidation --> Rejected: validate_compute_node()<br/>(rejected)
    Active --> Disabled: (admin action)
    Rejected --> [*]
    Disabled --> [*]
```

#### Sequence - Compute Node Registration

```mermaid
sequenceDiagram
    participant User as User
    participant CN as Compute Node
    participant RPC as RPC
    participant IPFS as IPFS
    participant DAC as Smart Contract
    participant VN as Validator Node (TEE)
    
    CN->>RPC: Subscribe to NodeInfo account changes<br/>(status = PendingClaim)
    VN->>RPC: Subscribe to NodeInfo account changes<br/>(status = AwaitingValidation)
    
    User->>DAC: register_node(node_pubkey, ComputeNode)
    DAC->>DAC: Create NodeInfo<br/>status = PendingClaim
    
    RPC->>CN: Notify NodeInfo status change<br/>(PendingClaim)
    
    CN->>IPFS: Upload node metadata
    IPFS->>CN: Return node_info_cid
    CN->>DAC: claim_compute_node(node_info_cid)
    DAC->>DAC: Store node_info_cid<br/>Set status = AwaitingValidation
    
    RPC->>VN: Notify NodeInfo status change<br/>(AwaitingValidation)
    
    VN->>DAC: Fetch node_info_cid from NodeInfo
    DAC->>VN: Return node_info_cid
    
    VN->>IPFS: Fetch node metadata using CID
    IPFS->>VN: Return node metadata
    
    VN->>CN: Request benchmark task
    CN->>CN: Perform benchmark task
    CN->>VN: Return benchmark results
    
    VN->>VN: Validate benchmark results<br/>message = node_pubkey + tee_signed_proof<br/>Sign with TEE signing key
    VN->>DAC: validate_compute_node(tee_signed_proof, tee_signature)
    DAC->>DAC: Verify TEE signature<br/>Set status = Active
```

#### Sequence - Validator Node Registration

```mermaid
sequenceDiagram
    participant VN as Validator Node
    participant TEE as TEE Enclave
    participant DAC as Smart Contract
    participant Intel as Intel SGX
    
    VN->>DAC: register_node(node_pubkey, ValidatorNode)
    DAC->>DAC: Create NodeInfo<br/>status = PendingClaim
    
    VN->>TEE: Request SGX quote
    TEE->>TEE: Generate TEE signing keypair (Ed25519)
    TEE->>TEE: Put node_pubkey[0..32] + tee_signing_pubkey[32..64] in report_data
    TEE->>Intel: Request certificate chain
    Intel->>TEE: Return QE cert + PCK cert
    TEE->>VN: Return quote + certificate_chain
    
    VN->>DAC: claim_validator_node(quote, cert_chain, IntelSGX)
    DAC->>DAC: Verify certificate chain<br/>(Intel Root CA → PCK → QE → Quote)
    DAC->>DAC: Extract MRENCLAVE from quote
    DAC->>DAC: Check MRENCLAVE in approved_code_measurements
    DAC->>DAC: Verify report_data[0..32] == node_pubkey
    DAC->>DAC: Extract tee_signing_pubkey from report_data[32..64]
    DAC->>DAC: Store code_measurement and tee_signing_pubkey
    DAC->>DAC: Set status = Active<br/>Increment validator_node_count
```

### Agent

Agent account that stores configuration and memory state for AI agents. Agents must be validated by a validator node before becoming active.

The Agent PDA stores:
- `agent_slot_id`: Unique slot identifier for the agent
- `owner`: Agent owner public key
- `agent_config_cid`: IPFS CID of agent configuration
- `agent_memory_cid`: IPFS CID of agent memory state
- `status`: Current status of the agent
- `bump`: Agent PDA bump seed

Seeds: `["agent", network_config, agent_slot_id.to_le_bytes()]`

#### State

```mermaid
stateDiagram-v2
    [*] --> Pending: create_agent()
    Pending --> Active: Validator validates<br/>agent_config_cid
    Active --> Inactive: (admin action)
    Inactive --> [*]
```

#### Sequence

```mermaid
sequenceDiagram
    participant Owner as Agent Owner
    participant IPFS as IPFS
    participant DAC as Smart Contract
    participant RPC as RPC
    participant VN as Validator Node
    participant TEE as TEE Enclave
    
    Owner->>IPFS: Upload agent config
    IPFS->>Owner: Return agent_config_cid
    
    Owner->>DAC: create_agent(public, agent_config_cid)
    DAC->>DAC: Create Agent (Pending)<br/>agent_slot_id = agent_count<br/>Increment agent_count<br/>Set status = Pending
    
    Note over VN: Validator subscribes to<br/>Agent account changes via RPC
    
    RPC->>VN: Notify Agent status change<br/>(Pending)
    VN->>IPFS: Fetch agent_config_cid
    IPFS->>VN: Return agent config
    VN->>DAC: Get approved agent config CID
    VN->>VN: Verify agent_config_cid matches<br/>network approved config
    
    alt Config Valid
        VN->>TEE: Sign validation
        TEE->>VN: Return tee_signature
        VN->>DAC: validate_agent(tee_signature)
        DAC->>DAC: Set status = Active
    else Config Invalid
        VN->>DAC: Reject agent
        DAC->>DAC: Set status = Inactive
    end
```

### Goal

Goal account that defines objectives for agents to achieve. Each goal has an associated vault (SystemAccount PDA) for payment management and tracks iteration progress with granular contribution accounting.

The Goal PDA stores:
- `goal_slot_id`: Unique slot identifier for the goal
- `owner`: Goal owner public key
- `agent`: Associated agent public key
- `task`: Associated task public key
- `status`: Current status of the goal (Ready, Active)
- `specification_cid`: IPFS CID of goal specification (max 128 characters)
- `max_iterations`: Maximum number of iterations
- `current_iteration`: Current iteration count
- `task_index_at_goal_start`: Task index when goal started
- `task_index_at_goal_end`: Task index when goal ended
- `chain_proof`: SHA256 chain proof for data integrity (chained from genesis, updated only after TEE validation)
- `vault`: Goal vault PDA address (SystemAccount holding SOL)
- `total_shares`: Total shares issued for this goal (share-based accounting)
- `locked_for_tasks`: Total SOL locked for currently processing tasks (max cost locked when claimed, released after validation)
- `bump`: Goal PDA bump seed

**Share-Based Accounting:**
- Contributors receive shares when depositing SOL
- Share price = (vault.lamports() - locked_for_tasks) / total_shares
- Share price automatically adjusts as tasks are paid (vault decreases)
- Withdrawals/refunds calculated as: shares × share_price

Seeds: `["goal", network_config, goal_slot_id.to_le_bytes()]`

#### Goal Vault

The goal vault is a **SystemAccount PDA** (not a data account) that holds SOL for the goal. It has no stored data - the lamport balance is the state.

Seeds: `["goal_vault", goal.key()]`

#### State

```mermaid
stateDiagram-v2
    [*] --> Ready: create_goal() or<br/>initialize_network()
    Ready --> Active: set_goal()<br/>initialize vault
    Active --> Active: contribute_to_goal()<br/>withdraw_from_goal()<br/>claim_task()<br/>submit_task_validation()
    Active --> Ready: submit_task_validation()<br/>(goal complete detected)<br/>(automatic refunds processed)
    Active --> Ready: cancel_goal()<br/>(owner cancels)<br/>(automatic full refunds processed)
    Ready --> Ready: (goal can be reused)
```

#### Sequence - Goal Lifecycle

```mermaid
sequenceDiagram
    participant Owner as Goal Owner
    participant Contributor as Contributor
    participant DAC as Smart Contract
    participant Vault as Goal Vault PDA
    
    Owner->>DAC: create_goal()
    DAC->>DAC: Create Goal (Ready)
    
    Owner->>DAC: set_goal(specification_cid, max_iterations, agent, initial_deposit)
    DAC->>DAC: Initialize Vault PDA<br/>Create Contribution account<br/>share_price = 1.0 (first deposit)
    Owner->>Vault: Transfer initial_deposit
    DAC->>DAC: Mint shares = initial_deposit / 1.0<br/>contribution.shares = shares<br/>goal.total_shares = shares<br/>Set status = Active
    
    loop Contribution & Computing Phase (Active)
        Contributor->>DAC: contribute_to_goal(deposit_amount)
        DAC->>DAC: Calculate share_price = (vault - locked) / total_shares<br/>Calculate shares_to_mint = deposit_amount / share_price
        Contributor->>Vault: Transfer deposit_amount
        DAC->>DAC: contribution.shares += shares_to_mint<br/>goal.total_shares += shares_to_mint
        
        opt Contributor wants to withdraw
            Contributor->>DAC: withdraw_from_goal(shares_to_burn)
            DAC->>DAC: Calculate share_price = (vault - locked) / total_shares<br/>Calculate withdraw_amount = shares_to_burn × share_price
            Vault->>Contributor: Transfer withdraw_amount
            DAC->>DAC: contribution.shares -= shares_to_burn<br/>goal.total_shares -= shares_to_burn
        end
    end
    
```

### Contribution

Contribution account that tracks individual contributor's share ownership for a specific goal. Each contributor-goal pair has one contribution account.

The Contribution PDA stores:
- `goal`: Goal public key
- `contributor`: Contributor public key
- `shares`: Number of shares owned by this contributor
- `refund_amount`: Final refund amount received after goal completion/cancellation (for history)
- `bump`: Contribution PDA bump seed

**Share Mechanics:**
- When depositing: `shares_to_mint = deposit_amount / share_price`
- When withdrawing: `withdraw_amount = shares_to_burn × share_price`
- Share value automatically decreases as tasks consume vault funds

Seeds: `["contribution", goal.key(), contributor.key()]`

#### Sequence - Contribution & Refund Flow

```mermaid
sequenceDiagram
    participant C as Contributor
    participant DAC as Smart Contract
    participant Vault as Goal Vault
    participant Goal as Goal Account
    participant Contrib as Contribution Account
    
    Note over Goal: Status = Active
    
    C->>DAC: contribute_to_goal(deposit_amount)
    DAC->>DAC: Calculate share_price = (vault - locked) / total_shares<br/>Calculate shares_to_mint = deposit_amount / share_price
    C->>Vault: Transfer deposit_amount SOL
    DAC->>Contrib: contribution.shares += shares_to_mint
    DAC->>Goal: goal.total_shares += shares_to_mint
    
    Note over Goal: Status = Active<br/>(tasks executing, vault balance decreases as tasks are paid)
    
    Note over Goal: Share price automatically decreases as<br/>vault balance decreases (tasks paid)
    
    alt Goal Completed (detected in submit_task_validation)
        DAC->>Goal: Process automatic refunds<br/>status = Ready<br/>(goal can be reused)
        
        Note over C,Vault: Automatic refunds to all contributors
        
        DAC->>DAC: Calculate share_price = vault.lamports() / goal.total_shares
        
        loop For each contributor with shares > 0
            DAC->>DAC: refund = contribution.shares × share_price
            Vault->>C: Transfer refund automatically
            DAC->>Contrib: refund_amount = refund<br/>shares = 0
        end
        
        DAC->>Goal: total_shares = 0
        
    else Goal Cancelled
        DAC->>Goal: cancel_goal()<br/>status = Ready<br/>(goal can be reused)
        
        Note over C,Vault: Automatic refunds to all contributors (based on current share value)
        
        DAC->>DAC: Calculate share_price = vault.lamports() / goal.total_shares
        
        loop For each contributor with shares > 0
            DAC->>DAC: refund = contribution.shares × share_price
            Vault->>C: Transfer refund automatically
            DAC->>Contrib: refund_amount = refund<br/>shares = 0
        end
        
        DAC->>Goal: total_shares = 0
    end
```

### Task

Task execution account that tracks the lifecycle of individual tasks. Tasks are pre-allocated during network initialization and reused across goal iterations.

The Task PDA stores:
- `task_slot_id`: Unique slot identifier for the task
- `action_type`: Type of action (e.g., LLM)
- `agent`: Associated agent public key
- `status`: Current status of the task
- `compute_node`: Compute node assigned to the task (optional)
- `input_cid`: IPFS CID of last validated task input data (used in chain_proof)
- `output_cid`: IPFS CID of last validated task output data (used in chain_proof)
- `pending_input_cid`: IPFS CID of task input data awaiting validation (optional)
- `pending_output_cid`: IPFS CID of task output data awaiting validation (optional)
- `chain_proof`: SHA256 chain proof for validation (chained from genesis, updated only after TEE validation)
- `execution_count`: Number of times task has been executed (includes both validated and rejected attempts, used in chain_proof for unique audit trail)
- `max_task_cost`: Maximum cost locked when task is claimed (actual cost determined at validation)
- `bump`: Task PDA bump seed

**Note:** When a task is claimed, `max_task_cost` is locked. When validated, the actual payment amount (which may be less) is paid to the compute node, and the max lock is released.

Seeds: `["task", network_config, task_slot_id.to_le_bytes()]`

#### State

```mermaid
stateDiagram-v2
    [*] --> Ready: create_task() or<br/>initialize_network()
    Ready --> Pending: (assigned to goal)<br/>(clear old input_cid/output_cid if reusing)
    Pending --> Processing: claim_task(compute_node, max_task_cost)<br/>(locks max_task_cost)
    Processing --> AwaitingValidation: submit_task_result(output_cid)
    AwaitingValidation --> Pending: submit_task_validation()<br/>(approved, goal not complete)<br/>(lock released)
    AwaitingValidation --> Ready: submit_task_validation()<br/>(rejected)<br/>(lock released, clear pending)
    AwaitingValidation --> [*]: submit_task_validation()<br/>(approved, goal complete)<br/>(lock released, clear validated CIDs for reuse)
```

#### Sequence

```mermaid
sequenceDiagram
    participant CN as Compute Node
    participant DAC as Smart Contract
    participant IPFS as IPFS
    participant RPC as RPC
    participant LLM as LLM
    participant VN as Validator Node (TEE)
    
    CN->>RPC: Subscribe to Task account changes<br/>(status = Pending)
    VN->>RPC: Subscribe to Task status changes<br/>(AwaitingValidation)
    
    Note over DAC: Task already exists<br/>status = Pending<br/>input_cid and output_cid set
    
    RPC->>CN: Notify Task status change<br/>(Pending)
    
    CN->>DAC: claim_task(compute_node, max_task_cost)
    DAC->>DAC: Verify goal.total_shares > 0 (has contributors)<br/>Verify vault.lamports() - goal.locked_for_tasks >= max_task_cost<br/>(available balance sufficient)
    DAC->>DAC: Lock max_task_cost: goal.locked_for_tasks += max_task_cost<br/>Set task.max_task_cost = max_task_cost<br/>Set compute_node<br/>Set status = Processing
    Note over DAC: Max cost locked - share price automatically decreases<br/>(locked funds excluded from share price calculation)<br/>Lock released when task validated or fails
    
    CN->>DAC: Fetch task context:<br/>- goal.specification_cid<br/>- agent.agent_config_cid<br/>- agent.agent_memory_cid<br/>- task.input_cid (validated) or pending_input_cid
    DAC->>CN: Return all CIDs
    
    CN->>IPFS: Fetch all required data:<br/>- goal specification<br/>- agent config<br/>- agent memory<br/>- task input
    IPFS->>CN: Return all data
    
    alt current_iteration == 0
        CN->>CN: Build input context from:<br/>- goal description<br/>- agent config<br/>- agent memory<br/>
        CN->>LLM: Run LLM with context
    else current_iteration > 0
        CN->>CN: Build input context from:<br/>- previous iteration output<br/>- agent config<br/>- agent memory<br/>
        CN->>LLM: Run LLM with context
    end
    
    LLM->>CN: Return output
    CN->>IPFS: Upload output data
    IPFS->>CN: Return output_cid
    CN->>DAC: submit_task_result(input_cid, output_cid)
    DAC->>DAC: Store pending_input_cid and pending_output_cid<br/>Set status = AwaitingValidation<br/>(validated input_cid/output_cid preserved)
    
    RPC->>VN: Notify Task status change<br/>(AwaitingValidation)
    
    VN->>IPFS: Fetch pending_input_cid and pending_output_cid
    IPFS->>VN: Return input and output data
    VN->>VN: Recompute partial execution<br/>Compute validation_proof = SHA256(pending_input_cid + pending_output_cid)<br/>Determine payment_amount<br/>Determine goal_completed (based on llm output)<br/>Sign validation with TEE signing key
    VN->>DAC: submit_task_validation(goal_id, task_cid, payment_amount, validation_proof, tee_signature)
    DAC->>DAC: Verify TEE signature<br/>Verify validation_proof == SHA256(pending_input_cid + pending_output_cid)<br/>Verify goal.status == Active
    
    alt Validation Approved
        DAC->>DAC: Update task chain_proof = SHA256(old_chain_proof + input_cid + output_cid + execution_count)<br/>(uses previous validated input_cid/output_cid)<br/>Move pending to validated: input_cid = pending_input_cid, output_cid = pending_output_cid<br/>Clear pending values<br/>Update goal chain_proof = SHA256(old_goal_proof + task_chain_proof + task_id + iteration)<br/>Release lock: goal.locked_for_tasks -= task.max_task_cost<br/>Pay node: vault → node_treasury (payment_amount)<br/>Add RewardEntry to node.recent_rewards<br/>total_tasks_completed++
        
        Note over DAC: Share price automatically drops!<br/>Vault decreased by payment_amount
        
        alt recent_rewards.len() >= max_entries_before_transfer
            Note over DAC: Batch threshold reached - auto-transfer
            DAC->>DAC: Calculate total_amount = sum(recent_rewards)
            DAC->>DAC: Transfer from vault to node_treasury<br/>node.total_earned += total_amount
            DAC->>DAC: Clear recent_rewards vector<br/>last_transfer_slot = current_slot
        else recent_rewards.len() < max_entries_before_transfer
            Note over DAC: Reward recorded, waiting for more to batch
        end
        
        DAC->>DAC: Update goal progress<br/>current_iteration++
        
        alt Goal Complete (detected by validator)
            DAC->>DAC: Calculate share_price = vault / total_shares<br/>Process automatic refunds<br/>status = Ready<br/>(goal can be reused)
            loop For each contributor with shares > 0
                DAC->>DAC: Calculate refund = shares × share_price
                Vault->>Contributor: Transfer refund automatically
                DAC->>Contrib: refund_amount = refund<br/>shares = 0
            end
            DAC->>DAC: total_shares = 0
        end
    else Validation Rejected
        DAC->>DAC: Release lock: goal.locked_for_tasks -= task.max_task_cost<br/>Set task.status = Ready<br/>(vault unchanged, share price returns to previous value)
    end
```

## Payment System

### Payment Flow Overview

The DAC payment system implements a three-phase lifecycle with granular contribution tracking and proportional refund mechanisms:

1. **Contribution Phase (Active)**: Contributors deposit funds, can withdraw at any time
2. **Computing Phase (Active)**: Tasks executing, rewards distributing, withdrawals still allowed
3. **Refund Phase (Active → Ready)**: Automatic proportional refunds processed when goal completes or is cancelled, then goal returns to Ready for reuse

### Payment Sequence - Complete Flow

```mermaid
sequenceDiagram
    participant Owner as Goal Owner
    participant Contributor as Contributor
    participant DAC as Smart Contract
    participant Vault as Goal Vault
    participant VN as Validator Node
    participant CN as Compute Node
    participant NodeTreasury as Node Treasury
    
    rect rgb(200, 230, 255)
        Note over Owner,Vault: PHASE 1: CONTRIBUTION (Active)
        
        Owner->>DAC: set_goal(initial_deposit)
        DAC->>DAC: Create Vault PDA<br/>share_price = 1.0 (first deposit)
        Owner->>Vault: Transfer initial_deposit
        DAC->>DAC: Mint shares = initial_deposit / 1.0<br/>contribution.shares = shares<br/>goal.total_shares = shares
        
        Contributor->>DAC: contribute_to_goal(deposit_amount)
        DAC->>DAC: Calculate share_price = (vault - locked) / total_shares<br/>Mint shares = deposit_amount / share_price
        Contributor->>Vault: Transfer deposit_amount
        DAC->>DAC: contribution.shares += shares<br/>goal.total_shares += shares
        
        opt Withdraw at any time
            Contributor->>DAC: withdraw_from_goal(shares_to_burn)
            DAC->>DAC: Calculate share_price = (vault - locked) / total_shares<br/>Calculate refund = shares_to_burn × share_price
            Vault->>Contributor: Transfer refund
            DAC->>DAC: contribution.shares -= shares_to_burn<br/>goal.total_shares -= shares_to_burn
        end
        
        Note over DAC: Goal remains Active during task execution
    end
    
    rect rgb(255, 230, 200)
        Note over VN,NodeTreasury: PHASE 2: COMPUTING (Batch Rewards)<br/>Note: Withdrawals still allowed<br/>Share price decreases as tasks are paid
        
        loop For each task
            CN->>CN: Execute task with LLM
            CN->>DAC: submit_task_result()
            
            VN->>VN: Validate task in TEE
            VN->>DAC: submit_task_validation(goal_id, task_cid, payment_amount, proof, signature)
            DAC->>DAC: Verify signature & proof<br/>Release lock: locked_for_tasks -= max_task_cost<br/>Pay node: vault → node_treasury (payment_amount)<br/>Add to node.recent_rewards
            
            Note over DAC: Share price automatically drops!<br/>Vault decreased by payment_amount
            
            alt Batch threshold reached (e.g., 10 rewards)
                DAC->>DAC: sum_rewards = sum(recent_rewards)
                Vault->>NodeTreasury: Transfer sum_rewards
                DAC->>DAC: node.total_earned += sum_rewards<br/>clear recent_rewards
                Note over DAC: Batch auto-transferred
            end
        end
        
        Note over DAC: Goal completion detected automatically<br/>in submit_task_validation()
        DAC->>DAC: Process automatic refunds<br/>status = Ready<br/>(goal can be reused)
        
        Note over DAC: Automatic refund processing
        
        DAC->>DAC: Calculate share_price = vault / total_shares
        
        loop For each contributor with shares > 0
            DAC->>DAC: Calculate refund = shares × share_price
            Vault->>Contributor: Transfer refund automatically
            DAC->>Contrib: refund_amount = refund<br/>shares = 0
        end
        
        DAC->>DAC: total_shares = 0
        
    end
    
    rect rgb(200, 255, 200)
        Note over Contributor,Vault: PHASE 3: REFUNDS (Automatic & Proportional)
    end
```

### Share-Based Accounting System

The DAC uses a **share-based accounting system** (similar to mutual funds) to automatically distribute compute costs among contributors proportionally. This eliminates complex per-contributor tracking and enables O(1) cost distribution.

#### Core Concept: The "Mutual Fund" Model

Instead of tracking exact SOL amounts per contributor, the system tracks **shares**:
- When you deposit SOL, you "buy" shares at the current share price
- When you withdraw, you "sell" shares back at the current share price
- The share price changes automatically as the vault balance changes

**Share Price Formula:**
```
share_price = (vault.lamports() - locked_for_tasks) / goal.total_shares
```

When tasks are paid, the vault balance decreases, causing the share price to drop automatically. All contributors' shares lose value proportionally without any per-contributor updates needed.

#### Example Walkthrough

**Step 1: Alice Deposits 100 SOL**
- Vault: 0 → 100 SOL
- Share price: 1.0 (default for first deposit)
- Alice receives: 100 / 1.0 = **100 shares**
- Total shares: 100

**Step 2: Bob Deposits 50 SOL**
- Vault: 100 → 150 SOL
- Share price: 150 / 150 = 1.0
- Bob receives: 50 / 1.0 = **50 shares**
- Total shares: 150

**Step 3: Tasks Execute, Cost 30 SOL**
- Vault: 150 → 120 SOL (30 paid to compute nodes)
- Total shares: 150 (unchanged)
- **New share price: 120 / 150 = 0.8 SOL/share**
- Alice's shares value: 100 × 0.8 = 80 SOL (paid 20 SOL)
- Bob's shares value: 50 × 0.8 = 40 SOL (paid 10 SOL)
- **No account updates needed! Share price adjusted automatically.**

**Step 4: Bob Withdraws Early**
- Bob burns his 50 shares at current price (0.8)
- Refund: 50 × 0.8 = **40 SOL**
- Vault: 120 → 80 SOL
- Total shares: 150 → 100
- New share price: 80 / 100 = 0.8 (unchanged for Alice)

**Step 5: Goal Completes, Alice Gets Refund**
- Alice's shares: 100
- Share price: 0.8
- Refund: 100 × 0.8 = **80 SOL**

**Result:** Alice paid 20 SOL, Bob paid 10 SOL. Perfectly proportional (2:1 ratio) without any complex calculations.


### Batch Transfer Mechanism

Validators record rewards incrementally, and transfers occur in batches to reduce transaction costs. Here's how it works:

**The Process:**

1. **Task Validation**: When a validator validates a completed task, they record the reward amount. This reward is added to a list of pending rewards for that compute node.

2. **Reward Accumulation**: Rewards accumulate in the list (up to 64 entries). Each time a task completes, its reward is added to this list instead of being paid immediately.

3. **Automatic Transfer**: When the list reaches a certain size (default: 10 rewards), the system automatically:
   - Calculates the total amount of all rewards in the list
   - Transfers that total amount from the goal vault to the compute node's treasury
   - Updates the goal's total pending payment counter
   - Updates the node's total earnings counter
   - Clears the reward list to start fresh

**Why This Helps:**
- **Saves Money**: Instead of paying for each task separately (which costs transaction fees), we pay for multiple tasks at once
- **Flexible**: The batch size can be configured
- **Accurate**: Every payment is tracked precisely
- **Transparent**: All batch transfers are tracked in account state for auditing

### Cancel Goal Flow

```mermaid
sequenceDiagram
    participant Owner as Goal Owner
    participant Contributor as Contributors
    participant DAC as Smart Contract
    participant Vault as Goal Vault
    
    Note over DAC: Goal can be cancelled while Active
    
    Owner->>DAC: cancel_goal()
    DAC->>DAC: Verify owner == goal.owner<br/>Verify status == Active
    DAC->>DAC: Calculate share_price = vault / total_shares<br/>Process automatic refunds<br/>status = Ready<br/>(goal can be reused)
    
    Note over Contributor,Vault: Automatic refunds to all contributors (based on share value)
    
        loop For each contributor with shares > 0
            DAC->>DAC: Calculate refund = contribution.shares × share_price
            Vault->>Contributor: Transfer refund automatically
            DAC->>DAC: Record refund_amount = refund<br/>shares = 0
        end
        
        DAC->>DAC: total_shares = 0
    
    Note over DAC: All contributors automatically refunded<br/>Goal status = Ready (can be reused)
```

## Instructions Reference

### Node Management Instructions

(Existing instructions remain unchanged: `register_node`, `claim_compute_node`, `validate_compute_node`, `claim_validator_node`)

### Task Execution Instructions

#### `claim_task(compute_node, max_task_cost)`
- **Description**: Claims a pending task for execution by a compute node, locks maximum cost
- **Accounts**: Task (mut), Goal (mut), Vault, ComputeNode (signer)
- **Guards**: 
  - `task.status == Pending`
  - `goal.status == Active`
  - `vault.lamports() - goal.locked_for_tasks >= max_task_cost` (available balance sufficient)
  - `max_task_cost > 0`
  - `goal.total_shares > 0` (ensures at least one contributor exists)
- **Actions**:
  - Verifies available vault balance (total - locked) is sufficient for maximum task cost
  - Locks maximum cost: `goal.locked_for_tasks += max_task_cost` (atomic with check)
  - Sets task.max_task_cost = max_task_cost
  - Sets task.compute_node = compute_node
  - Sets task.status = Processing
  - Increments task.execution_count
  - Note: Locked funds cannot be withdrawn until task completes or fails
  - Note: Share price automatically decreases when funds are locked (excluded from available balance)

#### `submit_task_result(input_cid, output_cid)`
- **Description**: Submits task execution results with input/output CIDs
- **Accounts**: Task (mut), ComputeNode (signer)
- **Guards**:
  - `task.status == Processing`
  - `task.compute_node == compute_node`
- **Actions**:
  - Stores input_cid and output_cid in `pending_input_cid` and `pending_output_cid`
  - Sets task.status = AwaitingValidation
  - Note: `input_cid`/`output_cid` (validated) are preserved for chain_proof calculation
  - Note: chain_proof is NOT updated here - only after TEE validation

### Payment & Contribution Instructions

#### `set_goal(specification_cid, max_iterations, agent, initial_deposit)`
- **Description**: Initializes a goal with configuration and optional initial deposit
- **Accounts**: Goal, Vault (init), Owner Contribution (init), Owner (signer, mut)
- **Actions**:
  - If goal.status == Ready and goal.current_iteration > 0 (reusing goal):
    - Resets execution state: `current_iteration = 0`, `task_index_at_goal_start = task_index_at_goal_end`, `task_index_at_goal_end = 0`
    - Resets payment state: `total_shares = 0`, `locked_for_tasks = 0`
    - Preserves chain_proof (continues audit trail)
    - Can update specification_cid
  - Creates goal vault SystemAccount PDA (if not exists)
  - Transfers initial_deposit from owner to vault
  - Creates owner's contribution account
  - Calculates share_price = 1.0 (first deposit)
  - Mints shares: `shares = initial_deposit / 1.0 = initial_deposit`
  - Sets contribution.shares = shares
  - Sets goal.total_shares = shares
  - Sets goal status to Active

#### `contribute_to_goal(deposit_amount)`
- **Description**: Contributes SOL to an active goal and receives shares
- **Accounts**: Goal (mut), Vault (mut), Contribution (init_if_needed), Contributor (signer, mut)
- **Guards**: `goal.status == Active`, `deposit_amount > 0`
- **Actions**:
  - Calculates current share_price:
    - If `goal.total_shares == 0`: share_price = 1.0 (first deposit)
    - Else: `share_price = (vault.lamports() - goal.locked_for_tasks) / goal.total_shares`
  - Calculates shares_to_mint: `deposit_amount / share_price`
  - Transfers deposit_amount from contributor to vault
  - Creates/updates contributor's contribution account
  - Increments contribution.shares by shares_to_mint
  - Increments goal.total_shares by shares_to_mint
  - Note: Contributor now owns a proportional percentage of the vault

#### `withdraw_from_goal(shares_to_burn)`
- **Description**: Withdraws funds by burning shares at the current share price
- **Accounts**: Goal (mut), Vault (mut), Contribution (mut), Contributor (signer, mut)
- **Guards**: 
  - `goal.status == Active`
  - `shares_to_burn > 0`
  - `contribution.shares >= shares_to_burn`
- **Actions**:
  - Calculates current share_price: `(vault.lamports() - goal.locked_for_tasks) / goal.total_shares`
  - Calculates withdraw_amount: `shares_to_burn × share_price`
  - Verifies available balance: `withdraw_amount <= (vault.lamports() - goal.locked_for_tasks)`
  - Transfers withdraw_amount from vault to contributor
  - Decrements contribution.shares by shares_to_burn
  - Decrements goal.total_shares by shares_to_burn
  - Note: Contributor receives their proportional share of available vault balance

#### `submit_task_validation(goal_id, task_cid, payment_amount, validation_proof, tee_signature)`
- **Description**: Submits task validation and records reward, triggering batch transfer when threshold reached
- **Accounts**: Goal (mut), Vault (mut), NodeInfo (mut), NodeTreasury (mut), Validator (signer)
- **Guards**: 
  - `goal.status == Active`
  - `goal.goal_slot_id == goal_id`
  - `payment_amount > 0`
  - `vault.lamports() >= total_amount` (if batch transfer triggered)
- **Actions**:
  - Verifies TEE signature
  - Verifies validation_proof matches expected proof (recomputed from pending_input_cid + pending_output_cid)
  - Verifies task hasn't been validated before (prevents replay)
  - If validation approved:
    - Updates task chain_proof: `SHA256(old_chain_proof + input_cid + output_cid + execution_count)` (uses previous validated values)
    - Moves pending to validated: `input_cid = pending_input_cid`, `output_cid = pending_output_cid`
    - Clears pending: `pending_input_cid = None`, `pending_output_cid = None`
    - Increments task.execution_count
    - Updates goal chain_proof: `SHA256(old_goal_proof + task_chain_proof + task_id + iteration)`
    - Releases lock: `goal.locked_for_tasks -= task.task_cost`
    - Adds RewardEntry to node.recent_rewards
    - Increments node.total_tasks_completed
    - Updates goal.current_iteration
    - If recent_rewards.len() >= max_entries_before_transfer:
      - Calculates total_amount = sum(recent_rewards) using checked arithmetic
      - Transfers total_amount from vault to node_treasury (funds were locked, guaranteed available)
      - Updates goal.total_pending_payment
      - Updates node.total_earned
      - Clears recent_rewards vector
    - **If goal is complete** (determined by validator based on LLM output):
      - Transfers any remaining rewards in recent_rewards (even if below threshold)
      - **Automatically processes refunds for all contributors:**
        - Uses current active contributors: `active_contributors = goal.active_contributors`
        - Requires `active_contributors > 0` (prevents division by zero)
        - Calculates total cost: `total_cost = sum(all completed task.task_cost)`
        - Calculates `cost_per_contributor = total_cost / active_contributors` (using checked division)
        - For each contributor with amount > 0:
          - Calculates `amount_owed = min(cost_per_contributor, contribution.amount)`
          - Calculates `refund = contribution.amount - amount_owed`
          - Transfers refund from vault to contributor
          - Records refund_amount = refund
          - Sets contribution.amount = 0
      - Sets goal.status = Ready (goal can be reused)
  - If validation rejected:
    - Releases lock: `goal.locked_for_tasks -= task.task_cost`



    - Sets task.status = Ready (task can be retried)
    - Clears pending: `pending_input_cid = None`, `pending_output_cid = None`
    - Note: Validated input_cid/output_cid remain (used in chain_proof), but will be cleared when task is reused for new goal
  - Note: Funds are locked at claim time, so payment is guaranteed if validation succeeds
  - Note: Goal completion is detected automatically by validator, no separate instruction needed

#### `cancel_goal()`
- **Description**: Cancels goal and automatically refunds all contributors based on current share value
- **Accounts**: Goal (mut), Vault (mut), All Contribution accounts (mut), Owner (signer)
- **Guards**: `goal.status == Active`, `owner.key() == goal.owner`
- **Actions**:
  - Calculates current share_price: `vault.lamports() / goal.total_shares` (if locked_for_tasks > 0, this includes locked funds)
  - **Automatically processes refunds for all contributors:**
    - For each contribution with shares > 0:
      - Calculates refund: `contribution.shares × share_price`
      - Transfers refund from vault to contributor
      - Records contribution.refund_amount = refund
      - Sets contribution.shares = 0
    - Sets goal.total_shares = 0
  - Sets goal.status = Ready (goal can be reused)
  - Note: If tasks are in-progress (locked_for_tasks > 0), those locked funds are included in refunds (full vault refund)


## Security Considerations

### TEE Attestation
- **Validators** must provide valid Intel SGX attestation quotes during registration
- **Code Measurement Whitelist**: Only approved MRENCLAVE values can register as validators
- **Certificate Chain Verification**: Full chain validation from Intel Root CA to quote

### Cryptographic Signatures
- All validator operations are signed using TEE-generated Ed25519 keypairs
- Signatures are verified on-chain before state changes
- TEE signing public key is extracted and stored during attestation

### Data Integrity
- **Hash Chain System**: SHA256 chain proofs provide tamper-proof execution history
  - All chains start from `genesis_hash` (created during network initialization)
  - Task chain: Updated only after TEE validation: `SHA256(old_chain_proof + input_cid + output_cid + execution_count)` (uses previous validated input_cid/output_cid, then moves pending to validated)
  - Goal chain: Updated when validated task completes: `SHA256(old_goal_proof + task_chain_proof + task_id + iteration)`
  - Chains continue across reuses - full execution history preserved
  - Off-chain audit: Start from genesis_hash, walk through all validated executions, recompute and verify
- **IPFS CIDs**: Content-addressed storage ensures data immutability
- **On-Chain State**: Only critical state and chain proofs stored on-chain

### Access Control
- Agent ownership enforced through Solana account ownership
- Vault withdrawals only through batch transfers or refunds
- Admin actions clearly separated and auditable

### Payment Security

#### Vault Protection
- **PDA Derivation**: Vault address derived from goal key, preventing unauthorized access
- **Program-Only Transfers**: Only program instructions can modify vault lamports
- **Balance Validation**: All transfers check sufficient balance before execution
- **Rent Exemption**: Vault maintains rent-exempt minimum balance
- **Withdrawal Anytime**: Contributors can withdraw at any time while goal is Active

#### Contribution Accounting
- **Snapshot Mechanism**: Records total_pending_payment at contribution time to prevent gaming refunds
- **Overflow Protection**: All arithmetic uses checked operations (checked_add, checked_sub)
- **Underflow Prevention**: Refund calculations protected with .ok_or(Error::Underflow)
- **Per-Contributor Isolation**: Each contributor has separate PDA account

#### Batch Transfer Safety
- **Atomic Operations**: Reward recording and batch transfer occur in single transaction
- **Vector Bounds**: recent_rewards limited to max_len(64)
- **Threshold Validation**: max_entries_before_transfer <= 64 enforced
- **Clear Audit Trail**: All transfers are tracked in account state for full auditability

#### State Transition Guards
- **Status Checks**: All operations verify correct goal status
- **Owner Verification**: Critical operations (cancel_goal) require owner signature
- **Automatic Transitions**: 
  - Goal remains Active during task execution
  - Goal transitions from Active → Ready automatically in submit_task_validation() when goal completion is detected (after refunds processed)
- **Contributor Verification**: Refund processing verifies contributor matches contribution.contributor (automatic refunds)
- **Automatic Refunds**: Refunds processed automatically in submit_task_validation() (on goal completion) and cancel_goal(), preventing manual claiming errors

#### Node Treasury Validation
- **PDA Derivation**: Node treasury must be derived from node_info account
- **Seeds Validation**: Seeds: ["node_treasury", node_info.key()]
- **Prevents Payment Hijacking**: Ensures rewards only go to legitimate node treasuries

#### Fund Locking Mechanism
- **Lock on Claim**: When compute node claims task, system locks `task.max_task_cost` by incrementing `goal.locked_for_tasks`
- **Atomic Operation**: Check and lock happen in same Solana instruction (atomic), preventing concurrent claim race conditions
- **Withdrawal Protection**: Contributors can only withdraw available balance: `vault.lamports() - goal.locked_for_tasks`
- **Lock Overflow Protection**: Uses checked_add to prevent lock overflow, verifies `locked_for_tasks <= vault.lamports()`
- **Lock Release**: Lock is released when:
  - Task is validated and payment made (lock released, payment transferred)
  - Task is rejected (lock released, funds available again)
- **Guaranteed Payment**: Since funds are locked at claim time, compute node is guaranteed payment if validation succeeds
- **No Race Condition**: Contributors cannot withdraw locked funds, eliminating the race condition

#### Refund Calculation Security
- **Active Contributors on Goal**: `goal.active_contributors` is updated every time a task is claimed, reflecting the current count of active contributors
- **Fair Cost Distribution**: Total cost of all completed tasks is divided equally among current active contributors at goal completion
- **Withdrawal During Computing**: Contributors can withdraw during computing; refund calculation uses current active contributors at goal completion
- **Division by Zero Protection**: Requires `goal.active_contributors > 0` before claiming task and before refund calculation (prevents task claim and goal completion if no active contributors)
- **Checked Arithmetic**: All division uses checked_div to prevent overflow/underflow

#### Batch Transfer Security
- **Vector Overflow**: `recent_rewards` limited to max_len(64), but `max_entries_before_transfer` could be set > 64
- **Mitigation**: Enforce `max_entries_before_transfer <= 64` in node initialization (validation guard required)
- **Sum Overflow**: Large batch transfers could overflow u64
- **Mitigation**: Use checked arithmetic when summing rewards
- **Final Batch Transfer**: Remaining rewards in recent_rewards are automatically transferred when goal completes, even if below threshold (ensures compute nodes always get paid)

#### TEE Signature Verification
- **Replay Attacks**: Same signature could be reused
- **Mitigation**: Include task_cid and goal_slot_id in signed message, verify task hasn't been validated before
- **Signature Validation**: Must verify TEE signing pubkey matches stored pubkey from attestation

#### Goal Status Transition Security
- **Automatic Transitions**: 
  - Goal remains Active during task execution
  - Active → Ready: Automatic in submit_task_validation() when validator detects goal completion (after refunds processed)
  - Active → Ready: Automatic in cancel_goal() when owner cancels (after full refunds processed)
- **Owner-Only Operations**: Only cancel_goal() requires owner signature
- **Mitigation**: Owner-only operations verify `ctx.accounts.owner.key() == goal.owner`
- **Status Guards**: Each instruction verifies correct status before allowing operation
- **Validator Authority**: Only validators can trigger goal completion via submit_task_validation()


