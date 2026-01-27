# SDK Refactor Plan: Goal → Session + submit_task

## Program changes to reflect
- **Initialize network**: Program uses `initialize_network(cid_config, allocate_tasks, approved_code_measurements, required_validations)` only. **No allocate_goals, no pre-allocated sessions**. Remaining accounts are task PDAs only.
- **Session (was Goal)**: PDA seeds `["session"]` / `["session_vault", session.key()]`.
- **Events**: `SessionSet` (session_slot_id, owner, task_slot_id, specification_cid, max_iterations, initial_deposit). `SessionCompleted` (session_slot_id, final_iteration, vault_balance). All events use `session_slot_id`. `TaskValidationSubmitted.session_completed`.
- **Instructions**: `create_session`, `set_session(specification_cid, max_iterations, initial_deposit, compute_node, task_type)`, `contribute_to_session`, `withdraw_from_session`. **`submit_task(ctx, input_cid)`**.
- **SetSession**: Program takes `compute_node: Pubkey` and `task_type: TaskType`; requires agent account.
- **Accounts/types**: Goal → Session, GoalStatus → SessionStatus. NetworkConfig uses `session_count` (next session slot id).

## Prerequisite
- Regenerate client from IDL: `anchor build` then `npx codama run --all -c codama-dac.json` so `generated/dac` has Session, createSession, setSession, contributeToSession, withdrawFromSession, submitTask, SessionSet, SessionCompleted, and decoders using session_slot_id.

## Implementation order

1. **dacPdas.ts** – Session PDAs and vault
   - `deriveSessionAddress(program, networkConfig, sessionSlotId)` seeds `b"session"`.
   - `deriveSessionVaultAddress(program, sessionAddress)` seeds `b"session_vault"`.
   - Keep `deriveContributionAddress(program, sessionAddress, contributor)` (first param is session, not goal).
   - Retain or alias `deriveGoalAddress` → `deriveSessionAddress` for callers until they are migrated.

2. **dacService.ts** – Types and interfaces
   - Goal → Session, GoalStatus → SessionStatus in imports and interfaces.
   - Params: CreateGoalParams → CreateSessionParams, SetGoalParams → SetSessionParams (drop agentSlotId where removed), ContributeToGoalParams → ContributeToSessionParams, WithdrawFromGoalParams → WithdrawFromSessionParams. Add SubmitTaskParams.
   - IQueryService: getGoal → getSession, batch methods use sessionSlotIds.
   - ITransactionService: createGoal → createSession, setGoal → setSession, contributeToGoal → contributeToSession, withdrawFromGoal → withdrawFromSession, add submitTask.
   - IMonitoringService: waitForGoalsStatus → waitForSessionsStatus (or keep and map Goal→Session).
   - ISubscriptionService: subscribeToGoalEvents → subscribeToSessionEvents, fetchHistoricalEvents(networkConfig, sessionSlotId), GoalEvent → SessionEvent.

3. **dacTransactions.ts** – Instructions and submit_task
   - Switch to generated: createSession, setSession, contributeToSession, withdrawFromSession, and add submitTask (getSubmitTaskInstruction + SubmitTask accounts).
   - Use deriveSessionAddress, deriveSessionVaultAddress, deriveContributionAddress(session, contributor). Replace allocateGoals/allocateTasks with allocate_tasks-only init if that’s the new contract.

4. **dacQueries.ts** – Read API
   - getGoal → getSession(networkConfig, sessionSlotId) using deriveSessionAddress.
   - batchGetContributionsForGoals → batchGetContributionsForSessions(networkConfig, sessionSlotIds, contributor).
   - batchGetVaultBalances / getContributorsForGoals → session-oriented names and sessionSlotIds. Use fetchMaybeSession when generated exposes it.

5. **dacSubscriptions.ts** – Events and structs
   - GoalEvent → SessionEvent; goalSlotId → sessionSlotId.
   - Event data: TaskClaimedEvent.sessionSlotId, TaskResultSubmittedEvent.sessionSlotId, TaskValidationSubmittedEvent.sessionSlotId + sessionCompleted, ContributionMadeEvent.sessionSlotId.
   - GoalSetEvent → SessionSetEvent: sessionSlotId, owner, taskSlotId, specificationCid, maxIterations, initialDeposit (no agentSlotId).
   - GoalCompletedEvent → SessionCompletedEvent: sessionSlotId, finalIteration, vaultBalance.
   - NodeValidatedEvent / NodeRejectedEvent: goalSlotId → sessionSlotId (or keep as optional context).
   - Discriminators: `GoalSet` → `SessionSet`, `GoalCompleted` → `SessionCompleted`. Decoders: getSessionSetDecoder, getSessionCompletedDecoder (from generated after codama).
   - parseAnchorEvent: use sessionSlotId; decode SessionSet, SessionCompleted, and session_slot_id in all events.
   - subscribeToGoalEvents → subscribeToSessionEvents(networkConfig, sessionSlotId, callback).
   - fetchHistoricalEvents(networkConfig, sessionSlotId, options); derive session address for getSignaturesForAddress when needed.

6. **dacSDK.ts** – Public API
   - getGoal → getSession, createGoal → createSession, setGoal → setSession, contributeToGoal → contributeToSession, withdrawFromGoal → withdrawFromSession. Add submitTask.
   - subscribeToGoalEvents → subscribeToSessionEvents, fetchHistoricalEvents(networkConfig, sessionSlotId).
   - Batch helpers: goalSlotIds → sessionSlotIds.

7. **dacMonitoring.ts / statusUtils.ts**
   - GoalStatus → SessionStatus, Goal → Session, waitForGoalsStatus → waitForSessionsStatus if present.

8. **configService / dacUtils**
   - Replace any goal/slot or status references with session/sessionSlotId and SessionStatus.
