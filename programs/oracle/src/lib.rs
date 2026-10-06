pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
pub use instructions::*;
pub use state::*;

declare_id!("Oracle111111111111111111111111111111111111");

#[program]
pub mod oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handle_initialize(ctx)
    }

    pub fn update_price(
        ctx: Context<UpdatePrice>,
        price: u64,
        timestamp: i64,
    ) -> Result<()> {
        instructions::update_price::handle_update_price(ctx, price, timestamp)
    }
}
