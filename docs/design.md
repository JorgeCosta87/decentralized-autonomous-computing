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
   - Immediate payment to compute nodes upon task validation
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
   - Immediate payment to compute nodes upon task validation

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

6. **Immediate Payment**: Compute nodes are paid immediately upon task validation

7. **Proportional Refund System**: Contributors only pay for compute that occurred after their contribution, ensuring fair cost distribution

### Data Flow

1. **Goal Creation & Funding**: Owner creates goal → initializes vault → optional initial deposit → contributors add funds
2. **Contribution Phase**: Contributors can deposit or withdraw at any time while goal is Active
3. **Computing Phase**: Tasks execute while goal remains Active (funds continue to be available for withdrawals)
4. **Task Execution**: Compute node claims task → executes with LLM → submits result
5. **Payment**: Validator verifies result → transfers payment immediately to compute node treasury
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

**Node Keypair Generation**: The `node_pubkey` is generated on the server where the node software is launched. Each node instance creates its own keypair for identification and signing operations.

The NodeInfo PDA stores:
- `owner`: Public key of the account that registered the node (can be different from node_pubkey)
- `node_pubkey`: Public key of the node (generated on the server where node runs)
- `node_type`: Type of node (Validator or Compute)
- `status`: Current status of the node
- `node_info_cid`: IPFS CID of node metadata (for compute nodes)
- `code_measurement`: TEE code measurement (for validator nodes)
- `tee_signing_pubkey`: TEE signing public key (for validator nodes)
- `node_treasury`: Node treasury PDA address (SystemAccount for receiving payments)
- `total_earned`: Cumulative SOL earned by the node
- `total_tasks_completed`: Total number of tasks completed by this node
- `bump`: NodeInfo PDA bump seed

Seeds: `["node_info", node_pubkey]`

#### Node Treasury

The node treasury is a **SystemAccount PDA** (not a data account) that receives payments from goal vaults.

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
    participant Owner as Node Owner
    participant CN as Compute Node<br/>(Server)
    participant RPC as RPC
    participant IPFS as IPFS
    participant DAC as Smart Contract
    participant VN as Validator Node (TEE)
    
    Note over CN: Node generates keypair on server<br/>node_pubkey = keypair.pubkey()
    CN->>RPC: Subscribe to NodeInfo account changes<br/>(status = PendingClaim)
    VN->>RPC: Subscribe to NodeInfo account changes<br/>(status = AwaitingValidation)
    
    Owner->>DAC: register_node(node_pubkey, ComputeNode)<br/>(owner signs, node_pubkey passed as param)
    DAC->>DAC: Create NodeInfo<br/>owner = owner.key()<br/>node_pubkey = node_pubkey<br/>status = PendingClaim<br/>Increment compute_node_count
    
    RPC->>CN: Notify NodeInfo status change<br/>(PendingClaim)
    
    CN->>IPFS: Upload node metadata
    IPFS->>CN: Return node_info_cid
    CN->>DAC: claim_compute_node(node_info_cid)<br/>(compute_node signs with node_pubkey)
    DAC->>DAC: Store node_info_cid<br/>Set status = AwaitingValidation
    
    RPC->>VN: Notify NodeInfo status change<br/>(AwaitingValidation)
    
    VN->>DAC: Fetch node_info_cid from NodeInfo
    DAC->>VN: Return node_info_cid
    
    VN->>IPFS: Fetch node metadata using CID
    IPFS->>VN: Return node metadata
    
    VN->>CN: Request benchmark task
    CN->>CN: Perform benchmark task
    CN->>VN: Return benchmark results
    
    VN->>VN: Validate benchmark results<br/>Create message: ValidateComputeNodeMessage {<br/>  compute_node_pubkey,<br/>  approved<br/>}<br/>Sign message with TEE signing key
    VN->>DAC: Transaction with:<br/>1. Ed25519 instruction (signature verification)<br/>2. validate_compute_node()<br/>(validator_node_pubkey signs transaction)
    DAC->>DAC: Extract signature, pubkey, message from Ed25519 instruction<br/>Verify pubkey matches stored validator TEE signing pubkey<br/>Verify message.compute_node_pubkey matches compute_node<br/>Set status = Active (if approved) or Rejected
```

#### Sequence - Validator Node Registration

```mermaid
sequenceDiagram
    participant Owner as Node Owner
    participant VN as Validator Node<br/>(Server)
    participant TEE as TEE Enclave
    participant DAC as Smart Contract
    participant Intel as Intel SGX
    
    Note over VN: Node generates keypair on server<br/>node_pubkey = keypair.pubkey()
    
    Owner->>DAC: register_node(node_pubkey, ValidatorNode)<br/>(owner signs, node_pubkey passed as param)
    DAC->>DAC: Create NodeInfo<br/>owner = owner.key()<br/>node_pubkey = node_pubkey<br/>status = PendingClaim<br/>Increment validator_node_count
    
    VN->>TEE: Request SGX quote
    TEE->>TEE: Generate TEE signing keypair (Ed25519)
    TEE->>TEE: Put node_pubkey[0..32] + tee_signing_pubkey[32..64] in report_data
    TEE->>Intel: Request certificate chain
    Intel->>TEE: Return QE cert + PCK cert
    TEE->>VN: Return quote + certificate_chain
    
    VN->>VN: Extract code_measurement (MRENCLAVE) from quote<br/>Extract tee_signing_pubkey from report_data
    VN->>DAC: claim_validator_node(code_measurement, tee_signing_pubkey)<br/>(validator_node signs with node_pubkey)
    DAC->>DAC: Verify code_measurement in approved_code_measurements<br/>Store code_measurement and tee_signing_pubkey<br/>Set status = Active
    
    Note over DAC: TODO: Full SGX quote verification<br/>(certificate chain, report_data validation)<br/>Currently simplified to code_measurement check
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
        VN->>DAC: validate_agent()
        DAC->>DAC: Verify agent.status == Pending<br/>Set status = Active
        Note over DAC: TODO: Add TEE signature verification<br/>and agent config validation
    else Config Invalid
        Note over VN: Rejection not yet implemented<br/>(will set status = Inactive)
    end
```

### Goal

Goal account that defines objectives for agents to achieve. Each goal has an associated vault (SystemAccount PDA) for payment management and tracks iteration progress with granular contribution accounting.

The Goal PDA stores:
- `goal_slot_id`: Unique slot identifier for the goal
- `owner`: Goal owner public key (Pubkey::default() if unowned/public, allowing anyone to set the goal)
- `agent`: Associated agent public key
- `task`: Associated task public key
- `status`: Current status of the goal (Ready, Active)
- `specification_cid`: IPFS CID of goal specification (max 128 characters)
- `max_iterations`: Maximum number of iterations
- `current_iteration`: Current iteration count
- `task_index_at_goal_start`: Task index when goal started
- `task_index_at_goal_end`: Task index when goal ended
- `chain_proof`: SHA256 chain proof for data integrity (chained from genesis, updated only after TEE validation)
- `total_shares`: Total shares issued for this goal (share-based accounting)
- `locked_for_tasks`: Total SOL locked for currently processing tasks (max cost locked when claimed, released after validation)
- `vault_bump`: Vault PDA bump seed
- `bump`: Goal PDA bump seed

**Goal Ownership:**
- Goals can be unowned (owner = Pubkey::default()) - anyone can set these goals
- Goals can be owned (owner = specific pubkey) - only the owner can set these goals
- After goal completion, goals can become unowned (public), allowing reuse by anyone

**Share-Based Accounting:**
- Contributors receive shares when depositing SOL
- Share price calculation:
  - If `total_shares == 0`: share_price = 1.0 (first deposit or all funds withdrawn)
  - Else: share_price = (vault.lamports() - locked_for_tasks - rent_exempt_minimum) / total_shares
  - Note: Rent lamports are excluded from share price calculation (they're for account maintenance, not user deposits)
- Share price automatically adjusts as tasks are paid (vault decreases)
- Withdrawals/refunds calculated as: shares × share_price
- If all funds are withdrawn (total_shares == 0), the next contribution treats it as a fresh start (share_price = 1.0)

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
    
    Owner->>DAC: create_goal(is_public)
    DAC->>DAC: Create Goal (Ready)<br/>Set owner = Pubkey::default() if is_public,<br/>otherwise set owner = owner.key()
    
    Owner->>DAC: set_goal(specification_cid, max_iterations, initial_deposit)
    DAC->>DAC: Verify task.status == Ready<br/>Verify agent.status == Active<br/>Verify vault only has rent or is empty<br/>Initialize Vault PDA<br/>Create Contribution account<br/>share_price = 1.0 (first deposit)
    Owner->>Vault: Transfer initial_deposit
    DAC->>DAC: Mint shares = initial_deposit / 1.0<br/>contribution.shares = shares<br/>goal.total_shares = shares<br/>Set goal.agent = agent.key()<br/>Set goal.task = task.key()<br/>Set task.status = Pending<br/>Set task.agent = agent.key()<br/>Set task.action_type = Llm<br/>Set goal status = Active
    
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
    Ready --> Pending: set_goal()<br/>(task assigned to goal,<br/>status = Pending, agent set, action_type = Llm)
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
    
    Note over DAC: Task assigned to goal in set_goal()<br/>status = Pending, agent set, action_type = Llm
    
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
        DAC->>DAC: Update task chain_proof = SHA256(old_chain_proof + input_cid + output_cid + execution_count)<br/>(uses previous validated input_cid/output_cid)<br/>Move pending to validated: input_cid = pending_input_cid, output_cid = pending_output_cid<br/>Clear pending values<br/>Update goal chain_proof = SHA256(old_goal_proof + task_chain_proof + task_id + iteration)<br/>Release lock: goal.locked_for_tasks -= task.max_task_cost<br/>Transfer payment: vault → node_treasury (payment_amount)<br/>Update node.total_earned += payment_amount<br/>Increment node.total_tasks_completed<br/>Update goal.current_iteration++
        
        Note over DAC: Share price automatically drops!<br/>Vault decreased by payment_amount
        
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
        
        Owner->>DAC: set_goal(specification_cid, max_iterations, initial_deposit)
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
        Note over VN,NodeTreasury: PHASE 2: COMPUTING<br/>Note: Withdrawals still allowed<br/>Share price decreases as tasks are paid
        
        loop For each task
            CN->>CN: Execute task with LLM
            CN->>DAC: submit_task_result()
            
            VN->>VN: Validate task in TEE
            VN->>DAC: submit_task_validation(goal_id, task_cid, payment_amount, proof, signature)
            DAC->>DAC: Verify signature & proof<br/>Release lock: locked_for_tasks -= max_task_cost<br/>Transfer payment: vault → node_treasury (payment_amount)<br/>Update node.total_earned += payment_amount
            
            Note over DAC: Share price automatically drops!<br/>Vault decreased by payment_amount
        end
        
        Note over DAC: Goal completion detected automatically<br/>in submit_task_validation()
        DAC->>DAC: Process automatic refunds<br/>status = Ready<br/>(goal can be reused)
        
        Note over DAC: Automatic refund processing
        
        DAC->>DAC: Calculate share_price = vault / total_shares
        
        loop For each contributor with shares > 0
            DAC->>DAC: Calculate refund = contribution.shares × share_price
            Vault->>Contributor: Transfer refund automatically
            DAC->>Contrib: refund_amount = refund<br/>shares = 0
        end
        
        DAC->>DAC: total_shares = 0 (all tokens burned)
        
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


### Payment Mechanism

Compute nodes are paid immediately when their tasks are validated. Here's how it works:

**The Process:**

1. **Task Validation**: When a validator validates a completed task, they determine the payment amount based on the task execution quality and cost.

2. **Immediate Payment**: The payment amount is transferred immediately from the goal vault to the compute node's treasury account.

3. **Tracking**: The node's `total_earned` counter is updated with the payment amount, and `total_tasks_completed` is incremented.

**Benefits:**
- **Immediate Payment**: Compute nodes receive payment as soon as their work is validated
- **Simple**: No batching or accumulation logic needed
- **Transparent**: All payments are tracked in account state for auditing

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
            DAC->>Contrib: refund_amount = refund<br/>shares = 0
        end
        
        DAC->>DAC: total_shares = 0
    
    Note over DAC: All contributors automatically refunded<br/>Goal status = Ready (can be reused)
```



## Security Considerations

### TEE Attestation
- **Validators** must provide valid Intel SGX attestation quotes during registration
- **Code Measurement Whitelist**: Only approved MRENCLAVE values can register as validators
- **Certificate Chain Verification**: Full chain validation from Intel Root CA to quote


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
- Vault withdrawals only through payments to compute nodes or refunds
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

#### Payment Safety
- **Atomic Operations**: Payment transfer occurs in the same transaction as validation
- **Balance Validation**: All transfers check sufficient vault balance before execution
- **Clear Audit Trail**: All payments are tracked in account state for full auditability

#### State Transition Guards
- **Status Checks**: All operations verify correct goal status
- **Owner Verification**: Critical operations (cancel_goal) require owner signature
- **Automatic Transitions**: 
  - Goal remains Active during task execution
  - Goal transitions from Active → Ready automatically in submit_task_validation() when goal completion is detected (after refunds processed)
- **Contributor Verification**: Refund processing verifies contributor matches contribution.contributor (automatic refunds)
- **Automatic Refunds**: Refunds processed automatically in submit_task_validation() (on goal completion) and cancel_goal(), preventing manual claiming errors

#### Payment Security
- **Fund Locking**: Tasks lock funds on claim (`goal.locked_for_tasks`), preventing withdrawal until validation completes
- **Node Treasury**: PDA-derived from node_info, prevents payment hijacking
- **Refunds**: Automatic on goal completion/cancellation, divided equally among active contributors
- **Immediate Payments**: Payments transferred immediately upon validation, with checked arithmetic to prevent overflow

#### Cryptographic Signatures
- **TEE-Generated Keys**: All validator operations signed using Ed25519 keypairs generated in TEE
- **Ed25519 Instruction**: Required before `validate_compute_node`, extracts signature/pubkey/message via instructions sysvar
- **Pubkey Verification**: Verifies signature created by validator's stored TEE signing pubkey (tamper-proof PDA storage)
- **Message**: `ValidateComputeNodeMessage { compute_node_pubkey, approved }` - Borsh serialized (33 bytes)


#### Goal Status Transition Security
- **Automatic Transitions**: 
  - Goal remains Active during task execution
  - Active → Ready: Automatic in submit_task_validation() when validator detects goal completion (after refunds processed)
  - Active → Ready: Automatic in cancel_goal() when owner cancels (after full refunds processed)
- **Owner-Only Operations**: Only cancel_goal() requires owner signature
- **Mitigation**: Owner-only operations verify `ctx.accounts.owner.key() == goal.owner`
- **Status Guards**: Each instruction verifies correct status before allowing operation
- **Validator Authority**: Only validators can trigger goal completion via submit_task_validation()


