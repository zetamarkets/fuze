extern crate proc_macro;

use anchor_syn::{codegen::program::common::SIGHASH_GLOBAL_NAMESPACE, parser};
use heck::SnakeCase;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn global_interface(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_trait = parse_macro_input!(input as syn::ItemTrait);

    let mod_name: proc_macro2::TokenStream = item_trait
        .ident
        .to_string()
        .to_snake_case()
        .parse()
        .unwrap();

    let methods: Vec<proc_macro2::TokenStream> = item_trait
        .items
        .iter()
        .filter_map(|trait_item: &syn::TraitItem| match trait_item {
            syn::TraitItem::Method(m) => Some(m),
            _ => None,
        })
        .map(|method: &syn::TraitItemMethod| {
            let method_name = &method.sig.ident;
            let args: Vec<&syn::PatType> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg: &syn::FnArg| match arg {
                    syn::FnArg::Typed(pat_ty) => Some(pat_ty),
                    // TODO: just map this to None once we allow this feature.
                    _ => panic!("Invalid syntax. No self allowed."),
                })
                .filter(|pat_ty| {
                    let mut ty = parser::tts_to_string(&pat_ty.ty);
                    ty.retain(|s| !s.is_whitespace());
                    !ty.starts_with("Context<")
                })
                .collect();
            let args_no_tys: Vec<&Box<syn::Pat>> = args
                .iter()
                .map(|arg| {
                    &arg.pat
                })
                .collect();
            let args_struct = {
                if args.is_empty() {
                    quote! {
                        #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
                        struct Args;
                    }
                } else {
                    quote! {
                        #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
                        struct Args {
                            #(#args),*
                        }
                    }
                }
            };

            let sighash_arr = anchor_syn::codegen::program::common::sighash(&SIGHASH_GLOBAL_NAMESPACE, &method_name.to_string());
            let sighash_tts: proc_macro2::TokenStream =
                format!("{:?}", sighash_arr).parse().unwrap();
            quote! {
                pub fn #method_name<'a,'b, 'c, 'info, T: anchor_lang::Accounts<'info> + anchor_lang::ToAccountMetas + anchor_lang::ToAccountInfos<'info>>(
                    ctx: anchor_lang::prelude::CpiContext<'a, 'b, 'c, 'info, T>,
                    #(#args),*
                ) -> anchor_lang::prelude::Result<()> {
                    #args_struct

                    let ix = {
                        let ix = Args {
                            #(#args_no_tys),*
                        };
                        let mut ix_data = anchor_lang::AnchorSerialize::try_to_vec(&ix)
                            .map_err(|_| anchor_lang::error::ErrorCode::InstructionDidNotSerialize)?;
                        let mut data = #sighash_tts.to_vec();
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
                    )?;

                    Ok(())
                }
            }
        })
        .collect();

    proc_macro::TokenStream::from(quote! {
        #item_trait

        /// Anchor generated module for invoking programs implementing an
        /// `#[global_interface]` via CPI.
        mod #mod_name {
            use super::*;
            #(#methods)*
        }
    })
}
