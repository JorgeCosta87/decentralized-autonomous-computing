use anchor_lang::error_code;

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow")]
    Overflow,
    #[msg("Missing account")]
    MissingAccount,
    #[msg("Need at least one code measurement")]
    NeedAtLeastOneCodeMeasurement,
    #[msg("At most 10 code measurements are allowed")]
    TooManyCodeMeasurements,
    #[msg("Invalid PDA account")]
    InvalidPDAAccount,
    #[msg("Account already initialized")]
    AccountAlreadyInitialized,
    #[msg("Invalid node type")]
    InvalidNodeType,
    #[msg("Invalid node status")]
    InvalidNodeStatus,
    #[msg("Invalid TEE signature")]
    InvalidTeeSignature,
    #[msg("Code measurement not approved")]
    CodeMeasurementNotApproved,
    #[msg("Node already registered")]
    NodeAlreadyRegistered,
    #[msg("Invalid instruction sysvar")]
    InvalidInstructionSysvar,
    #[msg("Bad Ed25519 program")]
    BadEd25519Program,
    #[msg("Bad Ed25519 accounts")]
    BadEd25519Accounts,
    #[msg("Invalid validator TEE signing pubkey")]
    InvalidValidatorTeeSigningPubkey,
    #[msg("Invalid compute node pubkey")]
    InvalidComputeNodePubkey,
    #[msg("Invalid goal status")]
    InvalidGoalStatus,
    #[msg("Underflow")]
    Underflow,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Deposit too small")]
    DepositTooSmall,
    #[msg("Invalid goal owner")]
    InvalidGoalOwner,
    #[msg("Invalid task status")]
    InvalidTaskStatus,
    #[msg("Invalid agent status")]
    InvalidAgentStatus,
    #[msg("Vault has leftover funds from previous goal")]
    VaultHasLeftoverFunds,
    #[msg("Invalid validator message")]
    InvalidValidatorMessage,
    #[msg("Invalid ipfs CID")]
    InvalidCID,
    #[msg("Invalid authority")]
    InvalidAuthority,
}
