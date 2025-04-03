use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use mpl_token_metadata::{
    instructions::CreateV1Builder,
    types::{TokenStandard},
};

declare_id!("AkugdJHDjDvBaxUGC6pjyrfqEpDfJ4Z9Ji9NED6Lmddg");

#[program]
pub mod token_contract {
    use super::*;

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        decimals: u8,
        uri: String,
        initial_supply: u64,
    ) -> Result<()> {
        // Debug log to verify received decimals
        msg!("Creating token with {} decimals", decimals);
        // Get the bump seed for the token account PDA
        let bump = ctx.bumps.token_account;
        let payer_key = ctx.accounts.payer.key();
        let mint_key = ctx.accounts.mint.key();

        // Create metadata using CPI builder
        let create_ix = CreateV1Builder::new()
            .metadata(ctx.accounts.metadata.key())
            .mint(mint_key, false)
            .authority(payer_key)
            .payer(payer_key)
            .update_authority(payer_key, true)
            .is_mutable(true)
            .primary_sale_happened(false)
            .name(name.clone())
            .symbol(symbol.clone())
            .uri(uri)
            .seller_fee_basis_points(0)
            .token_standard(TokenStandard::Fungible)
            .instruction();

        anchor_lang::solana_program::program::invoke(
            &create_ix,
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.sysvar_instructions.to_account_info(),
                ctx.accounts.token_program.to_account_info(), 
                ctx.accounts.token_metadata_program.to_account_info(),
            ],
        )?;

        // Create token account PDA seeds
        let seeds = &[
            b"token-account",
            payer_key.as_ref(),
            mint_key.as_ref(),
            &[bump],
        ];
        let signer = [&seeds[..]];

        // Initialize the token account
        let _cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::InitializeAccount {
                account: ctx.accounts.token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer,
        );
        //token::initialize_account(cpi_ctx)?;

        // Mint initial supply to token account
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            initial_supply * 10u64.pow(decimals as u32),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = payer,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Validated by CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: This is the token account we're creating
    #[account(
        init,
        payer = payer,
        seeds = [b"token-account", payer.key().as_ref(), mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = payer,
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    /// CHECK: Required by Metaplex
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: Metaplex program
    pub token_metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}