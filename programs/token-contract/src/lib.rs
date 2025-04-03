use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, Mint, Token, TokenAccount, SetAuthority};
use anchor_spl::token::spl_token::instruction::AuthorityType;
use mpl_token_metadata::{
    instructions::{CreateV1Builder, UpdateV1Builder},
    types::TokenStandard,
};

// Constant deployer public key
const DEPLOYER_PUBKEY: Pubkey = pubkey!("6oKdNekVDKYPeBLeCs33DttaMaVwzxeHJBfroPZeWwGk");

declare_id!("AkugdJHDjDvBaxUGC6pjyrfqEpDfJ4Z9Ji9NED6Lmddg");

#[program]
pub mod token_contract {
    use super::*;

    /// Initializes program security settings (call once after deployment)
    pub fn initialize_security(ctx: Context<InitializeSecurity>) -> Result<()> {
        // Check if already initialized
        if ctx.accounts.security.admin != Pubkey::default() {
            return Err(ErrorCode::AlreadyInitialized.into());
        }
        
        ctx.accounts.security.admin = ctx.accounts.authority.key();
        ctx.accounts.security.security_txt = String::from(
            "Contact: support@swapforge.app\n\
            Website: https://swapforge.app/\n\
            Twitter: https://x.com/SwapForgeApp\n\
            Policy: https://swapforge.app/security\n\
            Encryption: https://swapforge.app/pgp-key.txt"
        );
        ctx.accounts.security.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Updates security settings (admin only)
    pub fn upgrade_security(ctx: Context<UpdateSecurity>, new_txt: String) -> Result<()> {
        ctx.accounts.security.version += 1;
        ctx.accounts.security.security_txt = new_txt;
        ctx.accounts.security.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Creates a new token with metadata
    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        decimals: u8,
        uri: String,
        initial_supply: u64,
        revoke_mint: bool,
        revoke_freeze: bool,
        revoke_update: bool,
    ) -> Result<()> {
        // Validate fee payment (0.01 SOL)
        let fee_lamports = 10_000_000;
        if **ctx.accounts.payer.to_account_info().lamports.borrow() < fee_lamports {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Transfer fee to deployer
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.payer.key(),
            &DEPLOYER_PUBKEY,
            fee_lamports,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.deployer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Token creation logic (unchanged from your original)
        let bump = ctx.bumps.token_account;
        let payer_key = ctx.accounts.payer.key();
        let mint_key = ctx.accounts.mint.key();

        let create_ix = CreateV1Builder::new()
            .metadata(ctx.accounts.metadata.key())
            .mint(mint_key, false)
            .authority(payer_key)
            .payer(payer_key)
            .update_authority(payer_key, !revoke_update)
            .is_mutable(!revoke_update)
            .primary_sale_happened(false)
            .name(name)
            .symbol(symbol)
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

        // Initialize token account
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

        // Mint initial supply
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

        // Revoke authorities if requested
        if revoke_update {
            let update_ix = UpdateV1Builder::new()
                .metadata(ctx.accounts.metadata.key())
                .mint(ctx.accounts.mint.key())
                .authority(payer_key)
                .payer(payer_key)
                .new_update_authority(Pubkey::default())
                .instruction();

            anchor_lang::solana_program::program::invoke(
                &update_ix,
                &[
                    ctx.accounts.metadata.to_account_info(),
                    ctx.accounts.mint.to_account_info(),
                    ctx.accounts.payer.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                    ctx.accounts.sysvar_instructions.to_account_info(),
                    ctx.accounts.token_metadata_program.to_account_info(),
                ],
            )?;
        }

        if revoke_mint {
            token::set_authority(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    SetAuthority {
                        current_authority: ctx.accounts.payer.to_account_info(),
                        account_or_mint: ctx.accounts.mint.to_account_info(),
                    },
                ),
                AuthorityType::MintTokens,
                None,
            )?;
        }

        if revoke_freeze {
            token::set_authority(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    SetAuthority {
                        current_authority: ctx.accounts.payer.to_account_info(),
                        account_or_mint: ctx.accounts.mint.to_account_info(),
                    },
                ),
                AuthorityType::FreezeAccount,
                None,
            )?;
        }

        Ok(())
    }
}

/// Security initialization
#[derive(Accounts)]
pub struct InitializeSecurity<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 4 + 1000,  // 8 discriminator + 32 admin + 4 len + content
        seeds = [b"program-security"],
        bump
    )]
    pub security: Account<'info, ProgramSecurity>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Security update
#[derive(Accounts)]
pub struct UpdateSecurity<'info> {
    #[account(
        mut,
        seeds = [b"program-security"],
        bump,
        constraint = authority.key() == security.admin @ ErrorCode::Unauthorized // Correct verification
    )]
    pub security: Account<'info, ProgramSecurity>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Token creation
#[derive(Accounts)]
#[instruction(name: String, symbol: String, decimals: u8)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub deployer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = payer,
        mint::freeze_authority = payer,
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

/// Security account data
#[account]
pub struct ProgramSecurity {
    pub version: u32,
    pub admin: Pubkey,           // Admin pubkey (should match DEPLOYER_PUBKEY)
    pub security_txt: String,    // security.txt content
    pub last_updated: i64,       // Unix timestamp
}

/// Custom errors
#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized: Signer does not have admin privileges")]
    Unauthorized,
    #[msg("Insufficient funds for transaction")]
    InsufficientFunds,
    #[msg("Security account already initialized")]
    AlreadyInitialized,
}