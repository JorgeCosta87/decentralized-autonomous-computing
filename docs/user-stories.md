# User Stories & Instructions Reference

This document contains user stories and technical implementation details for all DAC instructions.

## Network Setup

### User Story: Initialize the Network
**As a** network authority  
**I want to** initialize the DAC network with configuration and pre-allocated resources  
**So that** the network is ready for nodes, agents, and goals

**Technical Implementation:**
- **Instruction**: `initialize_network(cid_config, allocate_goals, allocate_tasks, approved_code_measurements)`
- **Accounts**: Authority (signer, mut), NetworkConfig (init), SystemProgram
- **Actions**:
  - Creates NetworkConfig PDA with authority, network config CID, and approved TEE code measurements
  - Computes genesis_hash = SHA256("DAC_GENESIS")
  - Pre-allocates goal accounts (status = Ready, chain_proof = genesis_hash)
  - Pre-allocates task accounts (status = Ready, chain_proof = genesis_hash)
  - Initializes counters: agent_count = 0, goal_count = allocate_goals, task_count = allocate_tasks
  - Stores approved_code_measurements (max 10, newest at index 0)

## Node Management

### User Story: Register a Node
**As a** node operator  
**I want to** register my node in the network  
**So that** I can participate as either a validator or compute node

**Technical Implementation:**
- **Instruction**: `register_node(node_pubkey, node_type)`
- **Accounts**: NetworkConfig, NodeInfo (init), Node (signer), SystemProgram
- **Actions**:
  - Creates NodeInfo PDA with status = PendingClaim
  - Sets node_type (Validator or Compute)
  - Node must then claim their role (compute or validator)

### User Story: Claim Compute Node Role
**As a** compute node operator  
**I want to** claim my compute node role with metadata  
**So that** validators can validate my node and I can start accepting tasks

**Technical Implementation:**
- **Instruction**: `claim_compute_node(node_info_cid)`
- **Accounts**: ComputeNode (signer, mut), NetworkConfig, NodeInfo (mut)
- **Guards**:
  - `node_info.node_type == Compute`
  - `node_info.status == PendingClaim`
- **Actions**:
  - Stores node_info_cid (IPFS CID of node metadata)
  - Sets status = AwaitingValidation
  - Validators will then validate the node through benchmark testing

### User Story: Claim Validator Node Role
**As a** validator node operator  
**I want to** claim my validator role with TEE attestation  
**So that** I can validate tasks and compute nodes

**Technical Implementation:**
- **Instruction**: `claim_validator_node(code_measurement, tee_signing_pubkey)`
- **Accounts**: ValidatorNode (signer, mut), NetworkConfig (mut), NodeInfo (mut)
- **Guards**:
  - `node_info.node_type == Validator`
  - `node_info.status == PendingClaim`
  - `code_measurement` is in approved_code_measurements list
- **Actions**:
  - Stores code_measurement (MRENCLAVE from SGX quote)
  - Stores tee_signing_pubkey (Ed25519 public key from TEE)
  - Sets status = Active
  - Increments network_config.validator_node_count
  - Note: Full SGX attestation verification (certificate chain, quote parsing) should be implemented

### User Story: Validate Compute Node
**As a** validator node operator  
**I want to** validate a compute node after benchmark testing  
**So that** only capable compute nodes can participate in the network

**Technical Implementation:**
- **Instruction**: `validate_compute_node()` (no parameters - all data from Ed25519 instruction)
- **Transaction Structure**: Must include Ed25519 signature verification instruction before `validate_compute_node`
- **Message**: `ValidateComputeNodeMessage { compute_node_pubkey, approved }` signed with TEE signing key
- **Accounts**: ValidatorNode (signer), NetworkConfig (mut), ValidatorNodeInfo, ComputeNodeInfo (mut), InstructionsSysvar
- **Guards**:
  - `validator_node_info.status == Active`
  - `validator_node_info.node_type == Validator`
  - `compute_node_info.status == AwaitingValidation`
  - Ed25519 instruction exists in transaction (previous instruction)
  - TEE signing pubkey in Ed25519 instruction matches stored `validator_node_info.tee_signing_pubkey`
  - Message `compute_node_pubkey` matches `compute_node_info.node_pubkey`
  - Ed25519 program cryptographically verifies signature
- **Actions**:
  - If `message.approved == true`:
    - Sets `compute_node_info.status = Active`
    - Increments `network_config.compute_node_count`
  - If `message.approved == false`:
    - Sets `compute_node_info.status = Rejected`

## Agent Management

### User Story: Create an Agent
**As an** agent owner  
**I want to** create an AI agent with configuration  
**So that** it can be validated and used for goal execution

**Technical Implementation:**
- **Instruction**: `create_agent(agent_config_cid)`
- **Accounts**: AgentOwner (signer), NetworkConfig (mut), Agent (init), SystemProgram
- **Actions**:
  - Creates Agent PDA with agent_slot_id = agent_count
  - Stores agent_config_cid (IPFS CID of agent configuration)
  - Sets status = Pending
  - Increments network_config.agent_count
  - Validators will then validate the agent configuration

### User Story: Validate an Agent
**As a** validator node operator  
**I want to** validate an agent's configuration  
**So that** only approved agents can execute tasks

**Technical Implementation:**
- **Instruction**: `validate_agent()`
- **Accounts**: Validator (signer, mut), Agent (mut), NetworkConfig
- **Guards**:
  - `agent.status == Pending`
- **Actions**:
  - Sets agent.status = Active
  - Note: For now, validation is simplified. TODO: Add TEE signature verification and agent config validation

## Task Execution

### User Story: Claim a Task for Execution
**As a** compute node operator  
**I want to** claim a pending task  
**So that** I can execute it and earn rewards

**Technical Implementation:**
- **Instruction**: `claim_task(compute_node, max_task_cost)`
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

### User Story: Submit Task Execution Results
**As a** compute node operator  
**I want to** submit my task execution results  
**So that** validators can verify and approve my work

**Technical Implementation:**
- **Instruction**: `submit_task_result(input_cid, output_cid)`
- **Accounts**: Task (mut), ComputeNode (signer)
- **Guards**:
  - `task.status == Processing`
  - `task.compute_node == compute_node`
- **Actions**:
  - Stores input_cid and output_cid in `pending_input_cid` and `pending_output_cid`
  - Sets task.status = AwaitingValidation
  - Note: `input_cid`/`output_cid` (validated) are preserved for chain_proof calculation
  - Note: chain_proof is NOT updated here - only after TEE validation

### User Story: Validate Task Execution
**As a** validator node operator  
**I want to** validate task execution results  
**So that** compute nodes get paid for correct work and goals progress

**Technical Implementation:**
- **Instruction**: `submit_task_validation(goal_id, task_cid, payment_amount, validation_proof, tee_signature)`
- **Accounts**: Goal (mut), Vault (mut), NodeInfo (mut), NodeTreasury (mut), Validator (signer)
- **Guards**: 
  - `goal.status == Active`
  - `goal.goal_slot_id == goal_id`
  - `payment_amount > 0`
  - `vault.lamports() >= payment_amount`
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
    - Releases lock: `goal.locked_for_tasks -= task.max_task_cost`
    - Transfers payment_amount from vault to node_treasury immediately
    - Updates node.total_earned += payment_amount
    - Increments node.total_tasks_completed
    - Updates goal.current_iteration
    - **If goal is complete** (determined by validator based on LLM output):
      - Sets goal.status = Ready (goal can be reused)
  - If validation rejected:
    - Releases lock: `goal.locked_for_tasks -= task.max_task_cost`
    - Sets task.status = Ready (task can be retried)
    - Clears pending: `pending_input_cid = None`, `pending_output_cid = None`
    - Note: Validated input_cid/output_cid remain (used in chain_proof), but will be cleared when task is reused for new goal
  - Note: Funds are locked at claim time, so payment is guaranteed if validation succeeds
  - Note: Payment is transferred immediately on each validation
  - Note: Goal completion is detected automatically by validator, no separate instruction needed

## Payment & Contribution

### User Story: Create a Goal
**As a** goal owner  
**I want to** create a new goal account  
**So that** I can later configure and fund it

**Technical Implementation:**
- **Instruction**: `create_goal(is_public: bool)`
- **Accounts**: Payer (signer, mut), Owner (signer, mut), NetworkConfig (mut), Goal (init), SystemProgram
- **Parameters**:
  - `is_public`: If `true`, goal owner is set to `Pubkey::default()` (public, anyone can set this goal). If `false`, goal owner is set to the provided owner.
- **Actions**:
  - Creates Goal PDA with goal_slot_id = goal_count
  - Sets owner: `Pubkey::default()` if `is_public == true`, otherwise sets to `owner.key()`
  - Sets status = Ready
  - Sets chain_proof = genesis_hash (or continues from previous if reusing)
  - Increments network_config.goal_count
  - Note: Goals can also be pre-allocated during network initialization

### User Story: Initialize a Goal
**As a** goal owner  
**I want to** initialize a goal with configuration and funding  
**So that** agents can work towards achieving it

**Technical Implementation:**
- **Instruction**: `set_goal(specification_cid, max_iterations, initial_deposit)`
- **Accounts**: Goal (mut), Vault (init), Owner Contribution (init), Task (mut), Agent, Owner (signer, mut), NetworkConfig, SystemProgram
- **Guards**: 
  - `goal.status == Ready`
  - `goal.owner == Pubkey::default() || goal.owner == owner.key()` (goal must be unowned or owned by caller)
  - `task.status == TaskStatus::Ready` (task must be ready to be assigned)
  - `agent.status == AgentStatus::Active` (agent must be validated and active)
  - Vault must only contain rent lamports or be empty (no leftover funds from previous goal)
- **Actions**:
  - If goal.status == Ready and goal.current_iteration > 0 (reusing goal):
    - Resets execution state: `current_iteration = 0`, `task_index_at_goal_start = task_index_at_goal_end`, `task_index_at_goal_end = 0`
    - Resets payment state: `total_shares = 0`, `locked_for_tasks = 0`
    - Preserves chain_proof (continues audit trail)
    - Can update specification_cid
  - Creates goal vault SystemAccount PDA with initial_deposit + rent (or transfers initial_deposit if vault exists)
  - Creates owner's contribution account
  - Calculates share_price = 1.0 (first deposit)
  - Mints shares: `shares = initial_deposit / 1.0 = initial_deposit`
  - Sets contribution.shares = shares
  - Sets goal.total_shares = shares
  - Sets goal.owner = owner.key()
  - Sets goal.agent = agent.key()
  - Sets goal.task = task.key()
  - Sets goal.vault_bump = vault bump
  - Sets goal.task_index_at_goal_start = task.execution_count
  - Sets goal status to Active
  - **Assigns task to goal:**
    - Sets `task.status = TaskStatus::Pending`
    - Sets `task.agent = agent.key()`
    - Sets `task.action_type = ActionType::Llm`
    - Sets `goal.task_index_at_goal_start = task.execution_count`
    - Note: Task account must be provided and must have status = Ready
    - Note: Agent account must be provided and must have status = Active
  - Note: Goals with owner = Pubkey::default() are public and can be set by anyone

### User Story: Contribute to a Goal
**As a** contributor  
**I want to** deposit funds into an active goal  
**So that** I can support the goal's execution and receive proportional refunds

**Technical Implementation:**
- **Instruction**: `contribute_to_goal(deposit_amount)`
- **Accounts**: Goal (mut), Vault (mut), Contribution (init_if_needed), Contributor (signer, mut), SystemProgram
- **Guards**: `goal.status == Active`, `deposit_amount > 0`
- **Actions**:
  - Calculates current share_price:
    - If `goal.total_shares == 0`: share_price = 1.0 (first deposit or all funds previously withdrawn)
    - Else: `share_price = (vault.lamports() - goal.locked_for_tasks) / goal.total_shares`
  - Calculates shares_to_mint: `deposit_amount / share_price`
  - Transfers deposit_amount from contributor to vault using system_program::transfer
  - Creates/updates contributor's contribution account (init_if_needed)
  - Increments contribution.shares by shares_to_mint
  - Increments goal.total_shares by shares_to_mint
  - Note: If total_shares == 0 (all funds withdrawn), next contribution treats it as fresh start
  - Note: Contributor now owns a proportional percentage of the vault

### User Story: Withdraw from a Goal
**As a** contributor  
**I want to** withdraw my funds from an active goal  
**So that** I can exit before goal completion if needed

**Technical Implementation:**
- **Instruction**: `withdraw_from_goal(shares_to_burn)`
- **Accounts**: Goal (mut), Vault (mut), Contribution (mut), Contributor (signer, mut), SystemProgram
- **Guards**: 
  - `goal.status == Active`
  - `shares_to_burn > 0`
  - `contribution.shares >= shares_to_burn`
- **Actions**:
  - Calculates current share_price: `(vault.lamports() - goal.locked_for_tasks) / goal.total_shares`
  - Calculates withdraw_amount: `shares_to_burn × share_price`
  - Verifies available balance: `withdraw_amount <= (vault.lamports() - goal.locked_for_tasks)`
  - Transfers withdraw_amount from vault (PDA) to contributor using system_program::transfer with PDA signer
  - Decrements contribution.shares by shares_to_burn
  - Decrements goal.total_shares by shares_to_burn
  - Note: If all shares are withdrawn (total_shares == 0), goal can accept new contributions at share_price = 1.0
  - Note: Contributor receives their proportional share of available vault balance

### User Story: Cancel a Goal
**As a** goal owner  
**I want to** cancel my goal  
**So that** all contributors get automatic refunds and the goal can be reused

**Technical Implementation:**
- **Instruction**: `cancel_goal()`
- **Accounts**: Goal (mut), Vault (mut), All Contribution accounts (mut), Owner (signer), SystemProgram
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
