use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token};

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
        // Create the mint account
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            decimals,
            ctx.accounts.payer.key,
            Some(ctx.accounts.payer.key),
        )?;

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
        seeds = [b"mint", payer.key().as_ref()],
        bump,
        payer = payer,
        space = 82
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 4 + 4 + 1 + 32
    )]
    pub token_info: Account<'info, TokenInfo>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub authority: Pubkey,
}
