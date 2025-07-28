use anchor_lang::prelude::*;
use anchor_spl::{ metadata::{mpl_token_metadata::instructions::{FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts}, Metadata, MetadataAccount}, token::{ approve, Approve, Mint, Token, TokenAccount}};

use crate::{error::StakeError, StakeAccount, StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Stake<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub nft_mint: Account<'info, Mint>,
    pub nft_collection_mint: Account<'info, Mint>,

    #[account(mut, associated_token::mint = nft_mint, associated_token::authority = user)]
    pub user_nft_ata: Account<'info, TokenAccount>,

    #[account(seeds= [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref()], seeds::program  = metadata_program.key(), bump, constraint = metadata.collection.as_ref().unwrap().key.as_ref() == nft_collection_mint.key().as_ref(), constraint = metadata.collection.as_ref().unwrap().verified == true)]
    pub metadata: Account<'info, MetadataAccount>,
    pub metadata_program: Program<'info, Metadata>,

    #[account(seeds= [b"metadata", metadata_program.key().as_ref(), nft_mint.key().as_ref(), b"edition"], seeds::program  = metadata_program.key(), bump)]
    pub edition: Account<'info, MetadataAccount>,

    #[account(seeds=[b"config"], bump = config.bump)]
    pub config: Account<'info, StakeConfig>,

    #[account(mut, seeds = [b"user", user.key().as_ref()], bump = user_account.bump)]
    user_account: Account<'info, UserAccount>,

    #[account(init, payer = user, seeds = [b"state", nft_mint.key().as_ref(), config.key().as_ref()], bump, space =  8+ StakeAccount::INIT_SPACE)]
    pub stake_account: Account<'info, StakeAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

impl <'info> Stake <'info>{
    pub fn stake(&mut self, bumps: &StakeBumps)-> Result<()>{

        require!(self.user_account.amount_staked < self.config.max_stake, StakeError::MaxStake);
        
        self.stake_account.set_inner(StakeAccount { owner: self.user.key(), mint: self.nft_mint.key(), staked_at: Clock::get()?.unix_timestamp, bump: bumps.stake_account});

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Approve{
            to: self.user_nft_ata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            authority: self.user.to_account_info()
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        approve(cpi_context, 1)?;

        let seeds = &[b"stake", self.nft_mint.to_account_info().key.as_ref(), self.config.to_account_info().key.as_ref(), &[self.stake_account.bump]];

        let signers_seeds = &[&seeds[..]];

        let delegate  = &self.stake_account.to_account_info();
        let token_account = &self.user_nft_ata.to_account_info();
        let edition = &self.edition.to_account_info();
        let mint = &self.nft_mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        FreezeDelegatedAccountCpi::new(metadata_program, FreezeDelegatedAccountCpiAccounts{
            delegate,
            token_account,
            edition,
            mint,
            token_program
        }).invoke_signed(signers_seeds)?;
        self.user_account.amount_staked += 1;

       Ok(())
    }
}