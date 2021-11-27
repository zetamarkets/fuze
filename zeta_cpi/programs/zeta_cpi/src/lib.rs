
// extern crate proc_macro;

use anchor_lang::prelude::*;
pub mod constants;
use crate::constants::*;
pub mod client;
// use crate::client::initialize;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[interface]
pub trait SIGHASH_GLOBAL_NAMESPACE<'info, T: Accounts<'info>> {
    fn initialize_margin_account(
        ctx: Context<T>,
        nonce: u8,
    ) -> ProgramResult;
}

// pub fn initialize_margin_account<'a,'b, 'c, 'info, T: anchor_lang::Accounts<'info> + anchor_lang::ToAccountMetas + anchor_lang::ToAccountInfos<'info>>(
//     ctx: anchor_lang::CpiContext<'a, 'b, 'c, 'info, T>,
//     nonce: u8,
// ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
//     use anchor_syn::codegen::program::common::{sighash, SIGHASH_GLOBAL_NAMESPACE};
//     let sighash_arr = sighash(&SIGHASH_GLOBAL_NAMESPACE, &"initialize_margin_account");
//     let sighash_tts: proc_macro2::TokenStream =
//         format!("{:?}", sighash_arr).parse().unwrap();
//     use anchor_lang::prelude::borsh;
//     #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
//     struct Args {
//         nonce: u8
//     }

//     let ix = {
//         let ix = Args {
//             nonce
//         };
//         let mut ix_data = anchor_lang::AnchorSerialize::try_to_vec(&ix)
//             .map_err(|_| anchor_lang::__private::ErrorCode::InstructionDidNotSerialize)?;
//         let mut data = sighash_tts.to_string().as_bytes().to_vec();
//         data.append(&mut ix_data);
//         let accounts = ctx.to_account_metas(None);
//         anchor_lang::solana_program::instruction::Instruction {
//             program_id: *ctx.program.key,
//             accounts,
//             data,
//         }
//     };
//     let mut acc_infos = ctx.to_account_infos();
//     acc_infos.push(ctx.program.clone());
//     anchor_lang::solana_program::program::invoke_signed(
//         &ix,
//         &acc_infos,
//         ctx.signer_seeds,
//     )
// }

#[program]
pub mod zeta_cpi {
    use super::*;

    pub fn initialize_margin_account(ctx: Context<InitializeMarginAccountCaller>) -> ProgramResult {
        let cpi_program = ctx.accounts.zeta_program.clone();
        let cpi_accounts = InitializeMarginAccount {
            zeta_group: ctx.accounts.zeta_group.to_account_info(),
            margin_account: ctx.accounts.margin_account.to_account_info(),
            authority: ctx.accounts.authority.clone(),
            system_program: ctx.accounts.system_program.clone(),
        };
        // TODO: seeds = [MARGIN_SEED.as_bytes().as_ref(), zeta_group.key().as_ref(), authority.key.as_ref()]
        let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_bytes()], &cpi_program.key.clone());
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // use lowercase
        sighash_global_namespace::initialize_margin_account(cpi_ctx, nonce)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Empty {}

#[derive(Accounts)]
pub struct InitializeMarginAccount<'info> {
    pub zeta_group: AccountInfo<'info>,
    #[account(mut)]
    pub margin_account: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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

/*
Error: 101: Fallback functions are not supported
"As far as I know this means that the sighash of the instruction you're trying to run doesn't match anything in the program. 
Is it possible you're trying to call a stale version of the program or something?""
*/

// pub fn initialize_margin_account<'a, 'b, 'c, 'info>(ctx: CpiContext<'a, 'b, 'c, 'info, InitializeMarginAccountCaller<'info>>) -> ProgramResult {
//     let cpi_program = ctx.accounts.zeta_program.clone();
//     let (_pda, nonce)  = Pubkey::find_program_address(&[MARGIN_SEED.as_bytes()], &cpi_program.key.clone());
//     let ix = initialize(ctx.accounts.zeta_group.key, ctx.accounts.margin_account.key, ctx.accounts.authority.key, ctx.accounts.system_program.key, nonce)?;
//     solana_program::program::invoke_signed(       
//         &ix,
//         &[
//             ctx.accounts.zeta_group.clone(),
//             ctx.accounts.margin_account.clone(),
//             ctx.accounts.authority.to_account_info(),
//             ctx.accounts.system_program.to_account_info(),
//         ],
//         ctx.signer_seeds,
//     )
// }