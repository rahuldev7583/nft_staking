pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Gq49ENdEbCN4uQXbbd69BHLpza2aFQW5XkepyQYmtmdV");

#[program]
pub mod anchor_nft_staking {
    use super::*;

}
