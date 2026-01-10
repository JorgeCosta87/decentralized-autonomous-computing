# User Stories & Instructions Reference

This document contains user stories and technical implementation details for all DAC instructions.

## Network Setup

### User Story: Initialize the Network
**As a** network authority  
**I want to** initialize the DAC network with configuration and pre-allocated resources  
**So that** the network is ready for nodes, agents, and goals

**Technical Implementation:**
- **Instruction**: `initialize_network(cid_config, allocate_goals, allocate_tasks, approved_code_measurements, required_validations)`
- **Accounts**: Authority (signer, mut), NetworkConfig (init), SystemProgram
- **Parameters**:
  - `required_validations`: Number of validations required for consensus (applies to agents, nodes, and tasks)
- **Actions**:
  - Creates NetworkConfig PDA with authority, network config CID, approved TEE code measurements, and required_validations
  - Computes genesis_hash = SHA256("DAC_GENESIS")
  - Pre-allocates goal accounts (status = Ready, chain_proof = genesis_hash, is_confidential = false)
  - Pre-allocates task accounts (status = Ready, chain_proof = genesis_hash, approved_validators = [], rejected_validators = [])
  - Initializes counters: agent_count = 0, goal_count = allocate_goals, task_count = allocate_tasks
  - Stores approved_code_measurements (max 10, newest at index 0)

## Node Management

### User Story: Register a Node
**As a** node operator  
**I want to** register my node in the network  
**So that** I can participate as either a public or confidential node

**Technical Implementation:**
- **Instruction**: `register_node(node_pubkey, node_type)`
- **Accounts**: NetworkConfig, NodeInfo (init), Node (signer), SystemProgram
- **Parameters**:
  - `node_type`: Either `Public` (standard node) or `Confidential` (TEE-enabled node)
- **Actions**:
  - Creates NodeInfo PDA with status = PendingClaim
  - Sets node_type (Public or Confidential)
  - Initializes `approved_validators = []` and `rejected_validators = []`
  - Node must then claim their role (compute node or confidential node)

### User Story: Claim Compute Node Role
**As a** public node operator  
**I want to** claim my compute node role with metadata  
**So that** validators can validate my node and I can start accepting tasks

**Technical Implementation:**
- **Instruction**: `claim_compute_node(node_info_cid)`
- **Accounts**: Node (signer, mut), NetworkConfig, NodeInfo (mut)
- **Guards**:
  - `node_info.status == PendingClaim`
- **Actions**:
  - Stores node_info_cid (IPFS CID of node metadata)
  - Sets status = AwaitingValidation
  - **Note**: Works for both Public and Confidential nodes
  - Validators will then validate the node through benchmark testing

### User Story: Claim Confidential Node Role
**As a** confidential node operator  
**I want to** claim my confidential node role with TEE attestation  
**So that** I can execute confidential tasks and validate any tasks

**Technical Implementation:**
- **Instruction**: `claim_confidential_node(code_measurement, tee_signing_pubkey)`
- **Accounts**: ConfidentialNode (signer, mut), NetworkConfig (mut), NodeInfo (mut)
- **Guards**:
  - `node_info.node_type == Confidential`
  - `node_info.status == PendingClaim`
  - `code_measurement` is in approved_code_measurements list
- **Actions**:
  - Stores code_measurement (MRENCLAVE from SGX quote)
  - Stores tee_signing_pubkey (Ed25519 public key from TEE)
  - Sets status = Active
  - Increments network_config.confidential_node_count
  - **Note**: Confidential nodes are self-approved (TEE attestation is sufficient)
  - **Note**: Full SGX attestation verification (certificate chain, quote parsing) should be implemented

### User Story: Validate Public Node
**As a** node operator (public or confidential)  
**I want to** validate a public node after benchmark testing  
**So that** only capable nodes can participate in the network

**Technical Implementation:**
- **Instruction**: `validate_public_node(approved: bool)`
- **Accounts**: ValidatorNode (signer, mut), NetworkConfig (mut), ValidatorNodeInfo, NodeInfo (mut)
- **Guards**:
  - `validator_node_info.status == Active`
  - `validator_node_info.node_type == Public || validator_node_info.node_type == Confidential` (any active node can validate)
  - `node_info.status == AwaitingValidation`
  - `node_info.node_type == Public` (only public nodes need validation)
  - Validator not already in `node_info.approved_validators` or `node_info.rejected_validators` lists
- **Actions**:
  - Adds validator to `node_info.approved_validators` list (if approved) or `node_info.rejected_validators` list (if rejected)
  - Checks if `node_info.approved_validators.len() >= network_config.required_validations` (for approval) or `node_info.rejected_validators.len() >= network_config.required_validations` (for rejection)
  - If `approved == true` and threshold reached:
    - Sets `node_info.status = Active`
    - Increments `network_config.public_node_count`
  - If `approved == false`:
    - Sets `node_info.status = Rejected`
  - **Note**: Multiple validators must validate before node becomes Active (consensus)
  - **Note**: **Any active node** (public or confidential) can validate public nodes
  - **Note**: Confidential nodes are self-approved via TEE attestation (no validation needed)

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
**As a** node operator (public or confidential)  
**I want to** validate an agent's configuration  
**So that** only approved agents can execute tasks

**Technical Implementation:**
- **Instruction**: `validate_agent()`
- **Accounts**: Validator (signer, mut), Agent (mut), ValidatorNodeInfo, NetworkConfig
- **Guards**:
  - `agent.status == Pending`
  - `validator_node_info.status == Active` (any active node can validate)
  - Validator not already in `agent.approved_validators` or `agent.rejected_validators` lists
- **Actions**:
  - Adds validator to `agent.approved_validators` list
  - Checks if `agent.approved_validators.len() >= network_config.required_validations`
  - If threshold reached:
    - Sets `agent.status = Active`
  - **Note**: Multiple validators must validate before agent becomes Active (consensus)
  - **Note**: **Any active node** (public or confidential) can validate agents

## Task Execution

### User Story: Claim a Task for Execution
**As a** node operator  
**I want to** claim a pending task  
**So that** I can execute it and earn rewards

**Technical Implementation:**
- **Instruction**: `claim_task(max_task_cost)`
- **Accounts**: Task (mut), Goal (mut), Vault (mut), Node (signer), NodeInfo, NetworkConfig
- **Guards**: 
  - `task.status == Pending`
  - `goal.status == Active`
  - `node_info.status == Active`
  - **If `goal.is_confidential == true`**: `node_info.node_type == Confidential` (**ONLY confidential nodes can claim confidential tasks**)
  - **If `goal.is_confidential == false`**: Any active node (public or confidential) can claim
  - `vault.lamports() - goal.locked_for_tasks - rent_exempt_minimum >= max_task_cost` (available balance sufficient)
  - `max_task_cost > 0`
  - `goal.total_shares > 0` (ensures at least one contributor exists)
- **Actions**:
  - Verifies available vault balance (total - locked - rent) is sufficient for maximum task cost
  - Locks maximum cost: `goal.locked_for_tasks += max_task_cost` (atomic with check)
  - Sets task.max_task_cost = max_task_cost
  - Sets task.compute_node = node.key()
  - Sets task.status = Processing
  - Increments task.execution_count
  - Resets task validation tracking: `task.approved_validators = []`, `task.rejected_validators = []`
  - Note: Locked funds cannot be withdrawn until task completes or fails
  - Note: Share price automatically decreases when funds are locked (excluded from available balance)
  - **Note**: **Key distinction**: Only confidential nodes can **CLAIM/EXECUTE** confidential tasks (TEE protection for private data). However, **any active node** (public or confidential) can **VALIDATE** task execution results.

### User Story: Submit Task Execution Results
**As a** node operator  
**I want to** submit my task execution results  
**So that** validators can verify and approve my work

**Technical Implementation:**
- **Instruction**: `submit_task_result(input_cid, output_cid, next_input_cid)`
- **Accounts**: Task (mut), Goal (mut), Node (signer), NetworkConfig
- **Guards**:
  - `task.status == Processing`
  - `task.compute_node == Some(node.key())`
  - `input_cid.len() <= 128`
  - `output_cid.len() <= 128`
- **Actions**:
  - Stores input_cid, output_cid, and next_input_cid in `pending_input_cid`, `pending_output_cid`, and `next_input_cid`
  - Sets task.status = AwaitingValidation
  - Note: `input_cid`/`output_cid` (validated) are preserved for chain_proof calculation
  - Note: chain_proof is NOT updated here - only after validation threshold is reached

### User Story: Validate Task Execution (Confidential Goal)
**As a** node operator (public or confidential)  
**I want to** validate confidential task execution results  
**So that** nodes get paid for correct work and goals progress

**Technical Implementation:**
- **Instruction**: `submit_confidential_task_validation()` (no parameters - all data from Ed25519 instruction)
- **Transaction Structure**: Must include Ed25519 signature verification instruction before `submit_confidential_task_validation`
- **Message**: `SubmitTaskValidationMessage { goal_id, task_slot_id, payment_amount, validation_proof, approved, goal_completed }` signed with TEE signing key
- **Accounts**: Goal (mut), Vault (mut), Task (mut), NodeInfo (mut), NodeTreasury (mut), ValidatorNodeInfo, Validator (signer), NetworkConfig, InstructionSysvar, SystemProgram
- **Guards**: 
  - `goal.is_confidential == true`
  - `validator_node_info.status == Active`
  - `validator_node_info.node_type == Confidential` (only confidential nodes can validate confidential tasks with TEE signature)
  - `node_info.status == Active`
  - `goal.status == Active`
  - `task.status == AwaitingValidation`
  - Validator not already in `task.approved_validators` or `task.rejected_validators` lists
  - Ed25519 instruction exists in transaction (previous instruction)
  - TEE signing pubkey in Ed25519 instruction matches stored `validator_node_info.tee_signing_pubkey`
  - Message `goal_id` matches `goal.goal_slot_id`
  - Message `task_slot_id` matches `task.task_slot_id`
  - Message `validation_proof` matches `SHA256(pending_input_cid + pending_output_cid)`
  - Message `payment_amount > 0`
  - `vault.lamports() >= payment_amount`
  - Ed25519 program cryptographically verifies signature
- **Actions**:
  - Extracts signature, pubkey, and message from Ed25519 instruction via instructions sysvar
  - Verifies TEE signature and message integrity
  - Verifies validation_proof matches expected proof (recomputed from pending_input_cid + pending_output_cid)
  - Adds validator to `task.approved_validators` list (if approved) or `task.rejected_validators` list (if rejected)
  - Checks if `task.approved_validators.len() >= network_config.required_validations` (for approval) or `task.rejected_validators.len() >= network_config.required_validations` (for rejection)
  - **If threshold reached**:
    - If `message.approved == true`:
      - Updates task chain_proof: `SHA256(old_chain_proof + input_cid + output_cid + execution_count)`
      - Moves pending to validated: `input_cid = pending_input_cid`, `output_cid = pending_output_cid`
      - Clears pending: `pending_input_cid = None`, `pending_output_cid = None`
      - Updates goal chain_proof: `SHA256(old_goal_proof + task_chain_proof + task_id + iteration)`
      - Releases lock: `goal.locked_for_tasks -= task.max_task_cost`
      - Transfers `message.payment_amount` from vault to node_treasury immediately
      - Updates `node_info.total_earned += payment_amount`
      - Increments `node_info.total_tasks_completed`
      - Updates `goal.current_iteration++`
      - **If `message.goal_completed == true`**:
        - Sets `goal.status = Ready` (goal can be reused)
      - Else:
        - Sets `task.status = Pending` (task ready for next iteration)
    - If `message.approved == false`:
      - Releases lock: `goal.locked_for_tasks -= task.max_task_cost`
      - Sets `task.status = Ready` (task can be retried)
      - Clears pending: `pending_input_cid = None`, `pending_output_cid = None`
  - **Note**: Multiple validators must validate before task result is processed (consensus)
  - **Note**: Payment is transferred immediately when threshold is reached
  - **Note**: Goal completion is detected automatically by validator
  - **Note**: Confidential task validation requires TEE signature (only confidential nodes can provide this)

### User Story: Validate Task Execution (Public Goal)
**As a** node operator (public or confidential)  
**I want to** validate public task execution results  
**So that** nodes get paid for correct work and goals progress

**Technical Implementation:**
- **Instruction**: `submit_public_task_validation(payment_amount, approved, goal_completed)`
- **Accounts**: Goal (mut), Vault (mut), Task (mut), NodeInfo (mut), NodeTreasury (mut), ValidatorNodeInfo, Validator (signer), NetworkConfig, SystemProgram
- **Guards**: 
  - `goal.is_confidential == false`
  - `validator_node_info.status == Active`
  - `validator_node_info.node_type == Public || validator_node_info.node_type == Confidential` (**any active node can validate public tasks**)
  - `node_info.status == Active`
  - `goal.status == Active`
  - `task.status == AwaitingValidation`
  - Validator not already in `task.approved_validators` or `task.rejected_validators` lists
  - `payment_amount > 0`
  - `vault.lamports() >= payment_amount`
- **Actions**:
  - Adds validator to `task.approved_validators` list (if approved) or `task.rejected_validators` list (if rejected)
  - Checks if `task.approved_validators.len() >= network_config.required_validations` (for approval) or `task.rejected_validators.len() >= network_config.required_validations` (for rejection)
  - **If threshold reached**:
    - If `approved == true`:
      - Updates task chain_proof: `SHA256(old_chain_proof + input_cid + output_cid + execution_count)`
      - Moves pending to validated: `input_cid = pending_input_cid`, `output_cid = pending_output_cid`
      - Clears pending: `pending_input_cid = None`, `pending_output_cid = None`
      - Updates goal chain_proof: `SHA256(old_goal_proof + task_chain_proof + task_id + iteration)`
      - Releases lock: `goal.locked_for_tasks -= task.max_task_cost`
      - Transfers `payment_amount` from vault to node_treasury immediately
      - Updates `node_info.total_earned += payment_amount`
      - Increments `node_info.total_tasks_completed`
      - Updates `goal.current_iteration++`
      - **If `goal_completed == true`**:
        - Sets `goal.status = Ready` (goal can be reused)
      - Else:
        - Sets `task.status = Pending` (task ready for next iteration)
    - If `approved == false`:
      - Releases lock: `goal.locked_for_tasks -= task.max_task_cost`
      - Sets `task.status = Ready` (task can be retried)
      - Clears pending: `pending_input_cid = None`, `pending_output_cid = None`
  - **Note**: Multiple validators must validate before task result is processed (consensus)
  - **Note**: No TEE signature required for public goals (direct parameters)
  - **Note**: **Any active node** (public or confidential) can validate public task execution
  - **Note**: Payment is transferred immediately when threshold is reached
  - **Note**: Goal completion is determined by validator

## Payment & Contribution

### User Story: Create a Goal
**As a** goal owner  
**I want to** create a new goal account  
**So that** I can later configure and fund it

**Technical Implementation:**
- **Instruction**: `create_goal(is_public: bool, is_confidential: bool)`
- **Accounts**: Payer (signer, mut), Owner (signer, mut), NetworkConfig (mut), Goal (init), SystemProgram
- **Parameters**:
  - `is_public`: If `true`, goal owner is set to `Pubkey::default()` (public, anyone can set this goal). If `false`, goal owner is set to the provided owner.
  - `is_confidential`: If `true`, goal requires confidential (TEE) execution. Only confidential nodes can claim tasks for this goal.
- **Actions**:
  - Creates Goal PDA with goal_slot_id = goal_count
  - Sets owner: `Pubkey::default()` if `is_public == true`, otherwise sets to `owner.key()`
  - Sets `is_confidential` flag
  - Sets status = Ready
  - Sets chain_proof = genesis_hash (or continues from previous if reusing)
  - Increments network_config.goal_count
  - **Note**: Confidential goals can only be claimed by confidential nodes (TEE protection)
  - **Note**: Goals can also be pre-allocated during network initialization

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
