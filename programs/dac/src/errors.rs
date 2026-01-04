use anchor_lang::error_code;

#[error_code]
pub enum ErrorCode {
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
}
