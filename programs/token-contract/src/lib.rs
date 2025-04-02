use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use mpl_token_metadata::{
    instructions::CreateV1Builder,
    types::{TokenStandard},
};

declare_id!("GXRnCXCjonoPUUh1qGfZoxYFtZMrxPbTCZskq5B7qGhp");

#[program]
pub mod token_contract {
    use super::*;

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        decimals: u8,
        uri: String,
    ) -> Result<()> {
        // Create metadata using CPI builder
        let create_ix = CreateV1Builder::new()
            .metadata(ctx.accounts.metadata.key())
            .mint(ctx.accounts.mint.key(), false)
            .authority(ctx.accounts.payer.key())
            .payer(ctx.accounts.payer.key())
            .update_authority(ctx.accounts.payer.key(), true)
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
                ctx.accounts.token_program.to_account_info(), // Changed from spl_token_program
            ],
        )?;

        // Store additional info
        let token_info = &mut ctx.accounts.token_info;
        token_info.mint = ctx.accounts.mint.key();
        token_info.name = name;
        token_info.symbol = symbol;
        token_info.decimals = decimals;
        token_info.authority = ctx.accounts.payer.key();

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, decimals: u8, uri: String)]
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

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 4 + name.len() + 4 + symbol.len() + 1 + 32
    )]
    pub token_info: Account<'info, TokenInfo>,

    /// CHECK: Validated by CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>, // This is the SPL Token program
    /// CHECK: Required by Metaplex
    pub sysvar_instructions: UncheckedAccount<'info>,
    /// CHECK: Metaplex program
    pub token_metadata_program: UncheckedAccount<'info>,
}

#[account]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub authority: Pubkey,
}