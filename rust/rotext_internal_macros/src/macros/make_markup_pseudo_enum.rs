use std::ops::RangeInclusive;

use proc_macro2::TokenStream;

use quote::quote;
use syn::{self, parse::Parse, Token};

use crate::utils::{expect_key_value_ident_pair, expect_key_value_path_pair};

fn parse_input(tokens: TokenStream) -> Result<Input, TokenStream> {
    syn::parse2::<Input>(tokens).map_err(|err| err.to_compile_error())
}

pub fn make_markup_pseudo_enum(tokens: TokenStream) -> TokenStream {
    let Input {
        current_module_absolute_path,
        markup_internal_module_name,
        markup_macro_name,
        is_markup_function_name,
        markup_ranges,
    } = match parse_input(tokens) {
        Ok(input) => input,
        Err(err) => return err,
    };

    let mut mod_items = TokenStream::new();
    let mut macro_branches = TokenStream::new();
    let mut is_markup_tests = TokenStream::new();

    for range in markup_ranges {
        let start = *range.start();
        let end = *range.end();

        for char_u8 in range {
            let name = syn::Ident::new(&format!("X{:X}", char_u8), proc_macro2::Span::call_site());
            let value = syn::LitInt::new(&char_u8.to_string(), proc_macro2::Span::call_site());
            let char = syn::LitChar::new(
                char::from_u32(char_u8 as u32).unwrap(),
                proc_macro2::Span::call_site(),
            );

            mod_items.extend(quote! {
                pub const #name: u8 = #value;
            });

            macro_branches.extend(quote! {
                (#char) => { #current_module_absolute_path::#markup_internal_module_name::#name };
            });
        }

        is_markup_tests.extend(quote! {
            if (#start..=#end).contains(&char) {
                return true;
            }
        })
    }

    let output: TokenStream = quote! {
        pub(crate) mod #markup_internal_module_name {
            #mod_items
        }

        macro_rules! #markup_macro_name {
            #macro_branches
        }
        pub(crate) use #markup_macro_name;

        pub(crate) fn #is_markup_function_name(char: u8) -> bool {
            #is_markup_tests
            return false;
        }
    };

    output
}

struct Input {
    current_module_absolute_path: syn::Path,
    markup_internal_module_name: syn::Ident,
    markup_macro_name: syn::Ident,
    is_markup_function_name: syn::Ident,
    markup_ranges: Vec<RangeInclusive<u8>>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let current_module_absolute_path =
            expect_key_value_path_pair(&input, "current_module_absolute_path")?;
        let markup_internal_module_name =
            expect_key_value_ident_pair(&input, "markup_internal_module_name")?;
        let markup_macro_name = expect_key_value_ident_pair(&input, "markup_macro_name")?;
        let is_markup_function_name =
            expect_key_value_ident_pair(&input, "is_markup_function_name")?;

        let mut markup_ranges: Vec<RangeInclusive<u8>> = vec![];
        loop {
            let start: syn::LitInt = input.parse()?;
            if start.suffix() != "u8" {
                return Err(syn::Error::new(start.span(), "expect suffix `u8`"));
            }
            let start: u8 = start.base10_parse()?;

            let _: Token![..=] = input.parse()?;

            let end: syn::LitInt = input.parse()?;
            if end.suffix() != "u8" {
                return Err(syn::Error::new(end.span(), "expect suffix `u8`"));
            }
            let end: u8 = end.base10_parse()?;

            markup_ranges.push(start..=end);

            if input.is_empty() {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        Ok(Self {
            current_module_absolute_path,
            markup_internal_module_name,
            markup_macro_name,
            is_markup_function_name,
            markup_ranges,
        })
    }
}
