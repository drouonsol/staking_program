use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Approve, Mint, MintTo, Revoke, Token, TokenAccount},
};
use mpl_token_metadata::{
    instruction::{freeze_delegated_account, thaw_delegated_account},
    ID as MetadataTokenId,
};

declare_id!("BnbQDatiZBxcUxujTso24U3CURd6sTzbq1XRVCwghKXX");

#[program]
pub mod anchor_nft_staking {
    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {


        
        let clock = Clock::get().unwrap();
        msg!("Approving delegate");

        let cpi_approve_program = ctx.accounts.token_program.to_account_info();
        let cpi_approve_accounts = Approve {
            to: ctx.accounts.nft_token_account.to_account_info(),
            delegate: ctx.accounts.program_authority.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_approve_ctx = CpiContext::new(cpi_approve_program, cpi_approve_accounts);
        token::approve(cpi_approve_ctx, 1)?;

        msg!("Freezing token account");
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        invoke_signed(
            &freeze_delegated_account(
                ctx.accounts.metadata_program.key(),
                ctx.accounts.program_authority.key(),
                ctx.accounts.nft_token_account.key(),
                ctx.accounts.nft_edition.key(),
                ctx.accounts.nft_mint.key(),
            ),
            &[
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.nft_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.metadata_program.to_account_info(),
            ],
            &[&[b"authority", &[authority_bump]]],
        )?;


        let fee_amount  = 60 * anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL / 10000 ; 
        
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault_fees.key(),
            fee_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault_fees.to_account_info(),
            ],
        );

        if  ctx.accounts.stake_state.stake_account == 0 {
            ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;

        }

       


        ctx.accounts.stake_state.stake_account += 1;
        if ctx.accounts.stake_state.stake_account == 0 {
            ctx.accounts.stake_state.stake_account = 1;
        };
        
        let time = clock.unix_timestamp - ctx.accounts.stake_state.stake_start_time; 
        let redeem_amount = (10 * i64::pow(10, 2) * time) / (24 * 60 * 60) * (ctx.accounts.stake_state.stake_account as i64)  + ctx.accounts.stake_state.tokens_owed;
        ctx.accounts.stake_state.tokens_owed = redeem_amount;
        ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;
        ctx.accounts.nft_state.skate_state = StakeState::Staked;
        msg!("Staked NFTs: {:?}",   ctx.accounts.stake_state.stake_account);
        msg!("Amount Owed: {:?}",   ctx.accounts.stake_state.tokens_owed);
        ctx.accounts.vault_fees.total_staked +=  ctx.accounts.stake_state.stake_account as i64;
        msg!("Total NFTs Staked: {:?}", ctx.accounts.vault_fees.total_staked);
        

        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        require!(
            ctx.accounts.stake_state.stake_account > 0, // If this is true 
            StakeError::UninitializedAccount
        );



        let clock = Clock::get().unwrap();

        msg!(
            "Stake last redeem: {:?}",
            ctx.accounts.stake_state.last_stake_redeem
        );



     
        let clock = Clock::get().unwrap();
        let time = clock.unix_timestamp - ctx.accounts.stake_state.stake_start_time; 
        let redeem_amount = (10 * i64::pow(10, 2) * time) / (24 * 60 * 60) * (ctx.accounts.stake_state.stake_account as i64)  + ctx.accounts.stake_state.tokens_owed;
        ctx.accounts.stake_state.tokens_owed = redeem_amount;


        msg!("Staked NFTs: {:?}",   ctx.accounts.stake_state.stake_account);
        msg!("Amount Owed: {:?}",   ctx.accounts.stake_state.tokens_owed);
      ctx.accounts.stake_state.tokens_owed = 0;
        ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;
        msg!("Reedem");

        msg!("Minting staking rewards");
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.stake_mint.to_account_info(),
                    to: ctx.accounts.user_stake_ata.to_account_info(),
                    authority: ctx.accounts.stake_authority.to_account_info(),
                },
                &[&[
                    b"mint".as_ref(),
                    &[*ctx.bumps.get("stake_authority").unwrap()],
                ]],
            ),
            redeem_amount.try_into().unwrap(),
        )?;

        ctx.accounts.stake_state.last_stake_redeem = clock.unix_timestamp;
        msg!(
            "Updated last stake redeem time: {:?}",
            ctx.accounts.stake_state.last_stake_redeem
        );

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {

        require!(
            ctx.accounts.stake_state.stake_account > 0, // If this is true 
            StakeError::UninitializedAccount
        ); 


        msg!("Thawing token account");
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        invoke_signed(
            &thaw_delegated_account(
                ctx.accounts.metadata_program.key(),
                ctx.accounts.program_authority.key(),
                ctx.accounts.nft_token_account.key(),
                ctx.accounts.nft_edition.key(),
                ctx.accounts.nft_mint.key(),
            ),
            &[
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.nft_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.metadata_program.to_account_info(),
            ],
            &[&[b"authority", &[authority_bump]]],
        )?;

        msg!("Revoking delegate");

        let cpi_revoke_program = ctx.accounts.token_program.to_account_info();
        let cpi_revoke_accounts = Revoke {
            source: ctx.accounts.nft_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };




        let cpi_revoke_ctx = CpiContext::new(cpi_revoke_program, cpi_revoke_accounts);
        token::revoke(cpi_revoke_ctx)?;

        let clock = Clock::get()?;

        msg!(
            "Stake last redeem: {:?}",
            ctx.accounts.stake_state.last_stake_redeem
        );
        let time = clock.unix_timestamp - ctx.accounts.stake_state.stake_start_time; 
        let redeem_amount = (10 * i64::pow(10, 2) * time) / (24 * 60 * 60) * (ctx.accounts.stake_state.stake_account as i64)  + ctx.accounts.stake_state.tokens_owed;
        ctx.accounts.stake_state.tokens_owed = redeem_amount;
        ctx.accounts.nft_state.skate_state = StakeState::Unstaked;

        msg!("Minting staking rewards");
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.stake_mint.to_account_info(),
                    to: ctx.accounts.user_stake_ata.to_account_info(),
                    authority: ctx.accounts.stake_authority.to_account_info(),
                },
                &[&[
                    b"mint".as_ref(),
                    &[*ctx.bumps.get("stake_authority").unwrap()],
                ]],
            ),
            redeem_amount.try_into().unwrap(),
        )?;

        ctx.accounts.stake_state.last_stake_redeem = clock.unix_timestamp;
        msg!(
            "Updated last stake redeem time: {:?}",
            ctx.accounts.stake_state.last_stake_redeem
        );

        ctx.accounts.stake_state.tokens_owed = 0;
        ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;
        msg!("Staked NFTs: {:?}",   ctx.accounts.stake_state.stake_account);
        ctx.accounts.stake_state.stake_account -= 1;
        msg!("Staked NFTs: {:?}",   ctx.accounts.stake_state.stake_account);



        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        associated_token::mint=nft_mint,
        associated_token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(owner=MetadataTokenId)]
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        space = std::mem::size_of::<UserStakeInfo>() + 8,
        seeds = [user.key().as_ref(), b"elysianstake".as_ref()],
        bump
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    #[account(
        init_if_needed,
        payer=user,
        space = std::mem::size_of::<NftStakeInfo>() + 8,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump
    )]
    pub nft_state: Account<'info, NftStakeInfo>,
    #[account(
        init_if_needed,
        payer=user,
        space = std::mem::size_of::<VaultInfo>() + 8,
        seeds = [b"vault_elysian".as_ref()],
        bump
    )]
    pub vault_fees: Account<'info, VaultInfo>,
    /// CHECK: Manual validation
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), b"elysianstake".as_ref()],
        bump
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump
    )]
    pub nft_state: Account<'info, NftStakeInfo>,
    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,
    /// CHECK: manual check
    #[account(seeds = ["mint".as_bytes().as_ref()], bump)]
    pub stake_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority=user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK: Manual validation
    #[account(owner=MetadataTokenId)]
    pub nft_edition: UncheckedAccount<'info>,
  
    #[account(
        mut,
        seeds = [user.key().as_ref(), b"elysianstake".as_ref()],
        bump,
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump
    )]
    pub nft_state: Account<'info, NftStakeInfo>,

    /// CHECK: manual check
    #[account(mut, seeds=["authority".as_bytes().as_ref()], bump)]
    pub program_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub stake_mint: Account<'info, Mint>,
    /// CHECK: manual check
    #[account(seeds = ["mint".as_bytes().as_ref()], bump)]
    pub stake_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=stake_mint,
        associated_token::authority=user
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub metadata_program: Program<'info, Metadata>,
}

#[derive(Clone)]
pub struct Metadata;

impl anchor_lang::Id for Metadata {
    fn id() -> Pubkey {
        MetadataTokenId
    }
}

#[account]
pub struct UserStakeInfo {
    pub stake_start_time: i64,
    pub last_stake_redeem: i64,
    pub stake_account: i8,
    pub tokens_owed: i64,
}

#[account]
pub struct VaultInfo {
    pub total_staked: i64,

}


#[account]
pub struct NftStakeInfo {
    pub skate_state: StakeState,
}

#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize, Clone)]
pub enum StakeState {
    Unstaked,
    Staked,
}

impl Default for StakeState {
    fn default() -> Self {
        StakeState::Unstaked
    }
}

#[error_code]
pub enum StakeError {
    #[msg("NFT already staked")]
    AlreadyStaked,

    #[msg("State account is uninitialized")]
    UninitializedAccount,

    #[msg("User has never staked NFTs")]
    InvalidWaitingTIme,

    #[msg("Authority Doesn't Matches")]
    InvalidStakingToken,
}
