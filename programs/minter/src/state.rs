use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MinterConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub oracle_state: Pubkey,
    pub bump: u8,
    pub mint_authority_bump: u8,
}
