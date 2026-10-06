use anchor_lang::prelude::*;
use crate::{error::OracleError, state::OracleState, instructions::initialize::ORACLE_SEED};

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(
        mut,
        seeds = [ORACLE_SEED],
        bump = oracle_state.bump,
        has_one = authority @ OracleError::Unauthorized,
    )]
    pub oracle_state: Account<'info, OracleState>,
    pub authority: Signer<'info>,
}

pub fn handle_update_price(
    ctx: Context<UpdatePrice>,
    price: u64,
    timestamp: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.oracle_state;
    require!(timestamp > state.last_updated, OracleError::StaleUpdate);

    state.price = price;
    state.last_updated = timestamp;
    msg!("Price updated: {}", price);
    Ok(())
}
