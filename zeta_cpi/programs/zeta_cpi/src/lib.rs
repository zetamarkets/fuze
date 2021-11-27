
use anchor_lang::prelude::*;
pub mod constants;
use crate::constants::*;
pub mod client;
use sha2::{Digest, Sha256};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);

    let mut hasher = Sha256::new();
    hasher.update(preimage.as_bytes());
    let result = hasher.finalize();

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&result[..8]);
    sighash
}

// #[interface]
// pub trait SIGHASH_GLOBAL_NAMESPACE<'info, T: Accounts<'info>> {
//     fn initialize_margin_account(
//         ctx: Context<T>,
//         nonce: u8,
//     ) -> ProgramResult;
// }

// TODO: Refactor cpi calls into proc_macro

pub fn create_margin_account_cpi<'a,'b, 'c, 'info, T: anchor_lang::Accounts<'info> + anchor_lang::ToAccountMetas + anchor_lang::ToAccountInfos<'info>>(
    ctx: anchor_lang::CpiContext<'a, 'b, 'c, 'info, T>,
    nonce: u8,
) -> anchor_lang::solana_program::entrypoint::ProgramResult {
    let SIGHASH_GLOBAL_NAMESPACE = "global";
    let sighash_arr = sighash(&SIGHASH_GLOBAL_NAMESPACE, &"create_margin_account");
    #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
    struct Args {
        nonce: u8
    };

    let ix = {
        let ix = Args {
            nonce
        };
        let mut ix_data = anchor_lang::AnchorSerialize::try_to_vec(&ix)
            .map_err(|_| anchor_lang::__private::ErrorCode::InstructionDidNotSerialize)?;
        let mut data = sighash_arr.to_vec();
        data.append(&mut ix_data);
        let accounts = ctx.to_account_metas(None);
        anchor_lang::solana_program::instruction::Instruction {
            program_id: *ctx.program.key,
            accounts,
            data,
        }
    };
    let mut acc_infos = ctx.to_account_infos();
    acc_infos.push(ctx.program.clone());
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &acc_infos,
        ctx.signer_seeds,
    )
}

pub fn initialize_margin_account_cpi<'a,'b, 'c, 'info, T: anchor_lang::Accounts<'info> + anchor_lang::ToAccountMetas + anchor_lang::ToAccountInfos<'info>>(
    ctx: anchor_lang::CpiContext<'a, 'b, 'c, 'info, T>,
    nonce: u8,
) -> anchor_lang::solana_program::entrypoint::ProgramResult {
    let SIGHASH_GLOBAL_NAMESPACE = "global";
    let sighash_arr = sighash(&SIGHASH_GLOBAL_NAMESPACE, &"initialize_margin_account");
    #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
    struct Args {
        nonce: u8
    }

    let ix = {
        let ix = Args {
            nonce
        };
        let mut ix_data = anchor_lang::AnchorSerialize::try_to_vec(&ix)
            .map_err(|_| anchor_lang::__private::ErrorCode::InstructionDidNotSerialize)?;
        let mut data = sighash_arr.to_vec();
        data.append(&mut ix_data);
        let accounts = ctx.to_account_metas(None);
        anchor_lang::solana_program::instruction::Instruction {
            program_id: *ctx.program.key,
            accounts,
            data,
        }
    };
    let mut acc_infos = ctx.to_account_infos();
    acc_infos.push(ctx.program.clone());
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &acc_infos,
        ctx.signer_seeds,
    )
}

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn create_margin_account(ctx: Context<CreateMarginAccountCaller>) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = CreateMarginAccount {
            margin_account: ctx.accounts.margin_account.clone(),
            authority: ctx.accounts.authority.clone(),
            system_program: ctx.accounts.system_program.clone(),
            zeta_program: cpi_program.clone(),
            zeta_group: ctx.accounts.zeta_group.clone(),
        };
        let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_bytes(), ctx.accounts.zeta_group.key.as_ref(), ctx.accounts.authority.key.as_ref()], &cpi_program.key.clone());
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        create_margin_account_cpi(cpi_ctx, nonce)
    }

    pub fn initialize_margin_account(ctx: Context<InitializeMarginAccountCaller>) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = InitializeMarginAccount {
            zeta_group: ctx.accounts.zeta_group.clone(),
            margin_account: ctx.accounts.margin_account.clone(),
            authority: ctx.accounts.authority.clone(),
            system_program: ctx.accounts.system_program.clone(),
        };
        let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_bytes(), ctx.accounts.zeta_group.key.as_ref(), ctx.accounts.authority.key.as_ref()], &cpi_program.key.clone());
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        initialize_margin_account_cpi(cpi_ctx, nonce)
    }
}

#[derive(Accounts)]
pub struct CreateMarginAccount<'info> {
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(address = ID)]
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]

pub struct InitializeMarginAccount<'info> {
    pub zeta_group: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [MARGIN_SEED.as_bytes().as_ref(), zeta_group.key.as_ref(), authority.key.as_ref()],
        bump = nonce,
    )]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Dummy caller fns

#[derive(Accounts)]
pub struct CreateMarginAccountCaller<'info> {
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
}

// Note to self: don't do seeds validation in the outer call because that will use the calling programid
#[derive(Accounts)]
pub struct InitializeMarginAccountCaller<'info> {
    pub zeta_program: AccountInfo<'info>,
    pub zeta_group: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}