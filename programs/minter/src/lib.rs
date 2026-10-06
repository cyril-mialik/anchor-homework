pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
pub use instructions::*;
pub use state::*;

declare_id!("Minter1111111111111111111111111111111111111");

#[program]
pub mod minter {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
    ) -> Result<()> {
        instructions::initialize::handle_initialize(ctx)
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        instructions::mint::handle_mint(ctx, amount)
    }
}
