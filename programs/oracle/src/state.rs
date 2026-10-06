use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct OracleState {
    pub authority: Pubkey,
    pub price: u64,
    pub last_updated: i64,
    pub bump: u8,
}
