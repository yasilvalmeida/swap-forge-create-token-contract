use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

declare_id!("GXRnCXCjonoPUUh1qGfZoxYFtZMrxPbTCZskq5B7qGhp");

#[program]
pub mod token_contract {
    use super::*;

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        decimals: u8,
    ) -> Result<()> {
        // Store token metadata
        let token_info = &mut ctx.accounts.token_info;
        token_info.mint = ctx.accounts.mint.key();
        token_info.name = name;
        token_info.symbol = symbol;
        token_info.decimals = decimals;
        token_info.authority = ctx.accounts.payer.key();

        msg!("Token created successfully!");
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, decimals: u8)]
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

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub authority: Pubkey,
}