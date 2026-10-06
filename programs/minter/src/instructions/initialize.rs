use anchor_lang::prelude::*;
use crate::state::MinterConfig;

pub const CONFIG_SEED: &[u8] = b"config";
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: произвольный SPL Mint, создаётся отдельно
    pub mint: UncheckedAccount<'info>,
    /// CHECK: аккаунт oracle
    pub oracle_state: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + MinterConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, MinterConfig>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let (_, mint_authority_bump) = Pubkey::find_program_address(
        &[MINT_AUTHORITY_SEED],
        ctx.program_id,
    );

    let cfg = &mut ctx.accounts.config;
    cfg.admin = ctx.accounts.admin.key();
    cfg.mint = ctx.accounts.mint.key();
    cfg.oracle_state = ctx.accounts.oracle_state.key();
    cfg.bump = ctx.bumps.config;
    cfg.mint_authority_bump = mint_authority_bump;
    msg!("Minter initialized");
    Ok(())
}
