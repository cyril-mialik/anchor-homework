use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("Update is stale")]
    StaleUpdate,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Unauthorized")]
    Unauthorized,
}
