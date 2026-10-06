use crate::{
    error::MinterError,
    instructions::initialize::{CONFIG_SEED, MINT_AUTHORITY_SEED},
    state::MinterConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin @ MinterError::Unauthorized,
        has_one = mint,
    )]
    pub config: Account<'info, MinterConfig>,
    pub admin: Signer<'info>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub recipient: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA, который является mint authority
    #[account(
        seeds = [MINT_AUTHORITY_SEED],
        bump = config.mint_authority_bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_mint(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    require!(amount > 0, MinterError::InvalidAmount);

    let seeds = &[
        MINT_AUTHORITY_SEED,
        &[ctx.accounts.config.mint_authority_bump],
    ];
    let signer = &[&seeds[..]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;

    msg!("Minted {} tokens", amount);
    Ok(())
}
