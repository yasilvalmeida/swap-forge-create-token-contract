use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, Mint, Token, TokenAccount, SetAuthority};
use anchor_spl::token::spl_token::instruction::AuthorityType;
use mpl_token_metadata::{
    instructions::{CreateV1Builder, UpdateV1Builder},
    types::TokenStandard,
};

// Constants
const TREASURY_PUBKEY: Pubkey = pubkey!("DW69JZRd1j3Y2DsEhF2biwk3DPdn6BLeG51AFXg18ho2");
const BASE_FEE: u64 = 200_000_000; // 0.2 SOL
const REVOKE_DISCOUNT: u64 = 50_000_000; // 0.05 SOL per revoke

declare_id!("AkugdJHDjDvBaxUGC6pjyrfqEpDfJ4Z9Ji9NED6Lmddg");

#[program]
pub mod token_contract {
    use super::*;

    /// Initializes program security settings (call once after deployment)
    pub fn initialize_security(ctx: Context<InitializeSecurity>) -> Result<()> {
        require!(
            ctx.accounts.security.to_account_info().data_is_empty(),
            ErrorCode::AlreadyInitialized
        );
        
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
    pub fn update_security(
        ctx: Context<UpdateSecurity>,
        new_content: String,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.authority.key(),
            ctx.accounts.security.admin,
            ErrorCode::UnauthorizedSigner
        );
        
        require!(
            new_content.len() <= 1000,
            ErrorCode::ContentTooLong
        );
        
        ctx.accounts.security.security_txt = new_content;
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
        // Input validation
        require!(!name.is_empty() && name.len() <= 32, ErrorCode::InvalidTokenName);
        require!(!symbol.is_empty() && symbol.len() <= 10, ErrorCode::InvalidTokenSymbol);
        require!(decimals <= 18, ErrorCode::InvalidDecimals);
        require!(!uri.is_empty(), ErrorCode::InvalidUri);
        require!(initial_supply > 0, ErrorCode::InvalidInitialSupply);

        // Fee calculation with overflow protection
        let fee_lamports = BASE_FEE
            .checked_sub(if !revoke_mint { REVOKE_DISCOUNT } else { 0 })
            .and_then(|v| v.checked_sub(if !revoke_freeze { REVOKE_DISCOUNT } else { 0 }))
            .and_then(|v| v.checked_sub(if !revoke_update { REVOKE_DISCOUNT } else { 0 }))
            .ok_or(ErrorCode::InvalidFeeCalculation)?;

        require!(
            **ctx.accounts.payer.to_account_info().lamports.borrow() >= fee_lamports,
            ErrorCode::InsufficientFunds
        );

        // Transfer fee to treasury
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.payer.key(),
            &TREASURY_PUBKEY,
            fee_lamports,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Token creation
        let bump = ctx.bumps.token_account;
        let payer_key = ctx.accounts.payer.key();
        let mint_key = ctx.accounts.mint.key();

        // Validate metadata account
        let binding = ctx.accounts.token_metadata_program.key();
        let metadata_seeds = &[
            b"metadata",
            binding.as_ref(),
            mint_key.as_ref(),
        ];
        let (expected_metadata_key, _) = Pubkey::find_program_address(
            metadata_seeds,
            &ctx.accounts.token_metadata_program.key()
        );
        require_keys_eq!(
            ctx.accounts.metadata.key(),
            expected_metadata_key,
            ErrorCode::InvalidMetadataAccount
        );

        // Create metadata
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
            initial_supply.checked_mul(10u64.pow(decimals as u32))
                .ok_or(ErrorCode::InvalidInitialSupply)?,
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
        constraint = authority.key() == security.admin @ ErrorCode::UnauthorizedSigner
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
    
    /// CHECK: Treasury account not signer
    #[account(
        mut,
        address = TREASURY_PUBKEY @ ErrorCode::UnauthorizedTreasury
    )]
    pub treasury: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = payer,
        mint::freeze_authority = payer,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Validated in instruction
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

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
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: Metaplex program
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Security account data
#[account]
pub struct ProgramSecurity {
    pub admin: Pubkey,
    pub security_txt: String,
    pub last_updated: i64,
}

/// Reentrancy guard account
#[account]
pub struct ReentrancyGuard {
    pub last_execution: i64,
}

/// Custom errors
#[error_code]
pub enum ErrorCode {
    #[msg("Already initialized")]
    AlreadyInitialized,
    #[msg("Unauthorized signer")]
    UnauthorizedSigner,
    #[msg("Unauthorized Treasury")]
    UnauthorizedTreasury,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Invalid fee calculation")]
    InvalidFeeCalculation,
    #[msg("Token account already initialized")]
    TokenAccountAlreadyInitialized,
    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,
    #[msg("Invalid metadata account")]
    InvalidMetadataAccount,
    #[msg("Invalid token name")]
    InvalidTokenName,
    #[msg("Invalid token symbol")]
    InvalidTokenSymbol,
    #[msg("Invalid decimals")]
    InvalidDecimals,
    #[msg("Invalid URI")]
    InvalidUri,
    #[msg("Invalid initial supply")]
    InvalidInitialSupply,
    #[msg("Content too long")]
    ContentTooLong,
    #[msg("Reentrancy detected")]
    ReentrancyDetected,
    #[msg("Invalid token standard")]
    InvalidTokenStandard,
}