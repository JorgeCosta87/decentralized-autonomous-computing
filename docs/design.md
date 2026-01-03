# Decentralized Autonomous Civilization - Design Document

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
   - Each goal has an associated treasury PDA
   - Payments automatically transferred upon successful validation
   - Treasury balance checks before task creation
   - Support for contributions to goal treasuries

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

3. **Scalability**
   - Support for multiple validators and compute nodes
   - Configurable task and goal allocation limits
   - Reusable task accounts across goal iterations

4. **Reliability**
   - Task retry mechanism for failed executions
   - Node status tracking (Active/Disabled/Rejected)
   - Treasury balance validation before task creation

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

5. **Treasury-Per-Goal**: Each goal has its own treasury PDA for isolated payment management

### Data Flow

1. **Goal Creation**: Owner creates goal → deposits initial funds → selects agent
2. **Task Assignment**: Available task moved from Ready → Pending when assigned to goal
3. **Task Execution**: Compute node claims task → executes with LLM → submits result
4. **Validation**: Validator verifies result → signs validation → triggers payment
5. **Iteration**: If goal not complete, task returns to Pending; otherwise, task returns to Ready

## Architecture Specification


### NetworkConfig

Global network configuration account that tracks system-wide statistics and approved code measurements. This is the root account initialized once during network setup.

The NetworkConfig PDA stores:
- `netowrk_config_cid`: IPFS CID of network configuration
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

Seeds: `["network_config"]`

#### Sequence

```mermaid
sequenceDiagram
    participant Auth as Authority
    participant DAC as Smart Contract
    
    Auth->>DAC: initialize_network(cid_config, allocate_goals, allocate_tasks, approved_code_measurements)
    DAC->>DAC: Create NetworkConfig<br/>Set agent_count = 0<br/>goal_count = allocate_goals<br/>task_count = allocate_tasks<br/>Store approved_code_measurements
    
    loop For each goal (0..allocate_goals)
        DAC->>DAC: Create Goal PDA<br/>goal_slot_id = goal_id<br/>status = Pending
    end
    
    loop For each task (0..allocate_tasks)
        DAC->>DAC: Create Task PDA<br/>task_slot_id = task_id<br/>status = Ready
    end
    
    DAC->>Auth: Network initialized
```

### NodeInfo

Per-node account that stores registration information, TEE attestation data, and node status. Each validator or compute node has its own NodeInfo account.

The NodeInfo PDA stores:
- `node_pubkey`: Public key of the node
- `node_type`: Type of node (Validator or Compute)
- `status`: Current status of the node
- `node_info_cid`: IPFS CID of node metadata (for compute nodes)
- `code_measurement`: TEE code measurement (for validator nodes)
- `tee_signing_pubkey`: TEE signing public key (for validator nodes)
- `bump`: NodeInfo PDA bump seed

Seeds: `["node_info", node_pubkey]`

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

Goal account that defines objectives for agents to achieve. Each goal has an associated treasury for payments and tracks iteration progress.

The Goal PDA stores:
- `goal_slot_id`: Unique slot identifier for the goal
- `owner`: Goal owner public key
- `agent`: Associated agent public key
- `status`: Current status of the goal
- `description`: Goal description
- `max_iterations`: Maximum number of iterations
- `current_iteration`: Current iteration count
- `task_index_at_goal_start`: Task index when goal started
- `task_index_at_goal_end`: Task index when goal ended
- `bump`: Goal PDA bump seed

Seeds: `["goal", network_config, goal_slot_id.to_le_bytes()]`

#### State

```mermaid
stateDiagram-v2
    [*] --> Pending: create_goal() or<br/>initialize_network()
    Pending --> Active: set_goal()<br/>with deposit
    Active --> Active: validate_task()<br/>goal_completed = false<br/>current_iteration++
    Active --> Completed: validate_task()<br/>goal_completed = true
    Completed --> [*]
```

#### Sequence

```mermaid
sequenceDiagram
    participant Owner as Goal Owner
    participant DAC as Smart Contract
    participant Contributor as Contributor
    participant VN as Validator
    
    Owner->>DAC: Select available agent
    DAC->>Owner: Agent available
    
    Owner->>DAC: create_goal()
    DAC->>DAC: Create Goal (Pending)
    
    Owner->>DAC: set_goal(description, max_iterations, initial_deposit)
    DAC->>DAC: Initialize treasury PDA<br/>Transfer initial_deposit<br/>Set status = Active<br/>Link to agent
    
    Contributor->>DAC: deposit_to_goal(amount)
    DAC->>DAC: Transfer amount to treasury
    
    Note over DAC,VN: Goal executes tasks...
    
    VN->>DAC: validate_task(..., goal_completed = true)
    DAC->>DAC: Set status = Completed
```

### Task

Task execution account that tracks the lifecycle of individual tasks. Tasks are pre-allocated during network initialization and reused across goal iterations.

The Task PDA stores:
- `task_slot_id`: Unique slot identifier for the task
- `action_type`: Type of action (e.g., LLM)
- `agent`: Associated agent public key
- `status`: Current status of the task
- `compute_node`: Compute node assigned to the task (optional)
- `input_cid`: IPFS CID of task input data (optional)
- `output_cid`: IPFS CID of task output data (optional)
- `chain_proof`: SHA256 hash proof for validation
- `execution_count`: Number of times task has been executed
- `bump`: Task PDA bump seed

Seeds: `["task", network_config, task_slot_id.to_le_bytes()]`

#### State

```mermaid
stateDiagram-v2
    [*] --> Ready: create_task() or<br/>initialize_network()
    Ready --> Pending: (assigned to goal)
    Pending --> Processing: claim_task(compute_node)
    Processing --> AwaitingValidation: submit_task_result(output_cid)
    AwaitingValidation --> Pending: validate_task()<br/>(approved, goal not complete)
    AwaitingValidation --> Ready: validate_task()<br/>(rejected)
    AwaitingValidation --> [*]: validate_task()<br/>(approved, goal complete)
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
    
    CN->>DAC: claim_task(compute_node)
    DAC->>DAC: Set compute_node<br/>Set status = Processing
    
    CN->>DAC: Get goal description
    DAC->>CN: Return goal description
    CN->>DAC: Get agent config CID
    DAC->>CN: Return agent_config_cid
    CN->>DAC: Get agent memory CID
    DAC->>CN: Return agent_memory_cid
    
    CN->>IPFS: Fetch goal description
    IPFS->>CN: Return goal data
    CN->>IPFS: Fetch agent_config_cid
    IPFS->>CN: Return agent config
    CN->>IPFS: Fetch agent_memory_cid
    IPFS->>CN: Return agent memory
    CN->>IPFS: Fetch input_cid
    IPFS->>CN: Return input data
    
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
    DAC->>DAC: Store input and output_cid<br/>Set status = AwaitingValidation
    
    RPC->>VN: Notify Task status change<br/>(AwaitingValidation)
    
    VN->>IPFS: Fetch input_cid and output_cid
    IPFS->>VN: Return input and output data
    VN->>VN: Recompute partial execution<br/>Compute validation_proof = chain_proof<br/>Determine payment_amount<br/>Determine goal_completed (based on llm output)<br/>Sign validation with TEE signing key
    VN->>DAC: validate_task(approved, validation_proof, payment_amount, goal_completed, tee_signature)
    DAC->>DAC: Verify TEE signature
    DAC->>DAC: Verify validation_proof == chain_proof
    DAC->>DAC: Transfer payment to compute node treasury
    DAC->>DAC: Update goal progress
```

### GoalTreasury

Treasury account that holds SOL for payments to compute nodes. Each goal has an associated treasury PDA that receives deposits and makes payments upon task validation.

The GoalTreasury PDA stores:
- Treasury account balance (SOL)
- `bump`: GoalTreasury PDA bump seed

Seeds: `["treasury", goal]`

#### State

```mermaid
stateDiagram-v2
    [*] --> Empty: Goal created
    Empty --> Funded: set_goal()<br/>(initial_deposit)
    Funded --> Funded: deposit_to_goal()<br/>(contributions)
    Funded --> Funded: validate_task()<br/>(payment transferred)
    Funded --> Empty: All funds depleted
    Empty --> [*]
```

#### Sequence

```mermaid
sequenceDiagram
    participant Owner as Goal Owner
    participant Contributor as Contributor
    participant Agent as Agent
    participant DAC as Smart Contract
    participant VN as Validator
    participant CN as Compute Node
    
    Owner->>DAC: set_goal(..., initial_deposit)
    DAC->>DAC: Transfer initial_deposit
    
    Contributor->>DAC: deposit_to_goal(amount)
    DAC->>DAC: Transfer amount to treasury
    
    Note over DAC: Treasury accumulates funds
    
    Agent->>DAC: request_task()
    DAC->>DAC: Check treasury balance >= max_task_cost<br/>(accounting for rent exemption)
    alt Insufficient funds
        DAC->>Agent: Error: Insufficient funds
    else Sufficient funds
        DAC->>DAC: Create task (status = Pending)
    end
    
    VN->>DAC: validate_task(..., payment_amount, ...)
    DAC->>DAC: Transfer payment_amount to compute_node
    
    Note over CN: Compute node receives payment
```

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
- **Chain Proofs**: SHA256 hashes verify data integrity between compute and validation
- **IPFS CIDs**: Content-addressed storage ensures data immutability
- **On-Chain State**: Only critical state and proofs stored on-chain

### Access Control
- Agent ownership enforced through Solana account ownership
- Treasury withdrawals only through validated task completions
- Admin actions clearly separated and auditable
