use anchor_lang::prelude::*;
use anchor_spl::{ metadata::{mpl_token_metadata::instructions::{ ThawDelegatedAccountCpi, ThawDelegatedAccountCpiAccounts}, Metadata, MetadataAccount}, token::{  revoke, Mint, Revoke, Token, TokenAccount}};

use crate::{error::StakeError, StakeAccount, StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct UnStake<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub nft_mint: Account<'info, Mint>,
    pub nft_collection_mint: Account<'info, Mint>,

    #[account(mut, associated_token::mint = nft_mint, associated_token::authority = stake_account)]
    pub user_nft_ata: Account<'info, TokenAccount>,

    pub metadata_program: Program<'info, Metadata>,

    #[account(seeds= [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref(), b"edition"], seeds::program  = metadata_program.key(), bump)]
    pub edition: Account<'info, MetadataAccount>,

    #[account(seeds=[b"config"], bump = config.bump)]
    pub config: Account<'info, StakeConfig>,

    #[account(mut, seeds = [b"user", user.key().as_ref()], bump = user_account.bump)]
    user_account: Account<'info, UserAccount>,

    #[account(mut, close = user, seeds = [b"state", nft_mint.key().as_ref(), config.key().as_ref()], bump, )]
    pub stake_account: Account<'info, StakeAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

impl <'info> UnStake <'info>{
    pub fn Unstake(&mut self)-> Result<()>{
        let time_elapsed = ((Clock::get()?.unix_timestamp - self.stake_account.staked_at) / 86400) as u32;

        require!(time_elapsed > self.config.freeze_period, StakeError::FreezePeriod);

        self.user_account.point += (self.config.points_per_stake) as u32 * time_elapsed;

        let seeds = &[b"stake", self.nft_mint.to_account_info().key.as_ref(), self.config.to_account_info().key.as_ref(), &[self.stake_account.bump]];

        let signers_seeds = &[&seeds[..]];

        let delegate  = &self.stake_account.to_account_info();
        let token_account = &self.user_nft_ata.to_account_info();
        let edition = &self.edition.to_account_info();
        let mint = &self.nft_mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();
        
        let cpi_accounts = ThawDelegatedAccountCpiAccounts{
            delegate,
            token_account,
            edition,
            mint,
            token_program
        };

        ThawDelegatedAccountCpi::new(metadata_program, cpi_accounts).invoke_signed(signers_seeds)?;

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Revoke{
            source: self.user_nft_ata.to_account_info(),
            authority: self.user.to_account_info()
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        revoke(cpi_context)?;
        self.user_account.amount_staked -= 1;


       Ok(())
    }
}