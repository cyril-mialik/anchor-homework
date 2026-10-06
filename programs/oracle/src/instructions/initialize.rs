use anchor_lang::prelude::*;
use crate::state::OracleState;

pub const ORACLE_SEED: &[u8] = b"oracle";

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + OracleState::INIT_SPACE,
        seeds = [ORACLE_SEED],
        bump
    )]
    pub oracle_state: Account<'info, OracleState>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let state = &mut ctx.accounts.oracle_state;
    state.authority = ctx.accounts.payer.key();
    state.price = 0;
    state.last_updated = 0;
    state.bump = ctx.bumps.oracle_state;
    msg!("Oracle initialized");
    Ok(())
}
