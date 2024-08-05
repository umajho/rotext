use std::ops::RangeInclusive;

use proc_macro2::TokenStream;

use quote::quote;
use syn::{self, parse::Parse, Token};

use crate::utils::expect_key_value_ident_pair;

fn parse_input(tokens: TokenStream) -> Result<Input, TokenStream> {
    syn::parse2::<Input>(tokens).map_err(|err| err.to_compile_error())
}

pub fn make_markup_guard(tokens: TokenStream) -> TokenStream {
    let Input {
        markup_guard_macro_name,
        is_markup_function_name,
        markup_ranges,
    } = match parse_input(tokens) {
        Ok(input) => input,
        Err(err) => return err,
    };

    let mut macro_branches = TokenStream::new();
    let mut is_markup_tests = TokenStream::new();

    for range in markup_ranges {
        let start = *range.start();
        let end = *range.end();

        for char_u8 in range {
            let value = syn::LitInt::new(&char_u8.to_string(), proc_macro2::Span::call_site());
            let char = syn::LitChar::new(
                char::from_u32(char_u8 as u32).unwrap(),
                proc_macro2::Span::call_site(),
            );

            macro_branches.extend(quote! {
                (#char) => { #value };
            });
        }

        is_markup_tests.extend(quote! {
            if (#start..=#end).contains(&char) {
                return true;
            }
        })
    }

    macro_branches.extend(quote! {
        ($x:literal) => { ::std::compile_error!(concat!("“", $x, "” is not a valid markup (character)!")) };
    });

    let output: TokenStream = quote! {
        macro_rules! #markup_guard_macro_name {
            #macro_branches
        }
        pub(crate) use #markup_guard_macro_name;

        pub(crate) fn #is_markup_function_name(char: u8) -> bool {
            #is_markup_tests
            return false;
        }
    };

    output
}

struct Input {
    markup_guard_macro_name: syn::Ident,
    is_markup_function_name: syn::Ident,
    markup_ranges: Vec<RangeInclusive<u8>>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let markup_guard_macro_name =
            expect_key_value_ident_pair(&input, "markup_guard_macro_name")?;
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
            markup_guard_macro_name,
            is_markup_function_name,
            markup_ranges,
        })
    }
}
