mod data_for_event;
pub mod ensure_cases;

use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;

use quote::quote;
use syn::{self, parse::Parse, spanned::Spanned};

use crate::utils::{expect_key_value_ident_pair, expect_key_value_path_pair};

fn parse_attribute_arguments(tokens: TokenStream) -> Result<AttributeArguments, TokenStream> {
    syn::parse2::<AttributeArguments>(tokens).map_err(|err| err.to_compile_error())
}

pub fn simple_sub_enum_for_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let AttributeArguments {
        current_mod_path,
        enum_guard_macro_name,
        debug_group_tester_macro_name,
        groups,
    } = match parse_attribute_arguments(attr) {
        Ok(attr) => attr,
        Err(err) => return err,
    };

    let mut enum_item: syn::ItemEnum = match syn::parse2(item) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };
    let enum_name = enum_item.ident.clone();

    let mut group_to_variants: HashMap<String, Vec<String>> = HashMap::new();
    let mut variant_to_groups: HashMap<String, Vec<String>> = HashMap::new();

    let mut errors: Vec<TokenStream> = vec![];
    for variant in enum_item.variants.iter_mut() {
        let variant_name = variant.ident.to_string();
        let mut group_attr_index: Option<usize> = None;
        for (i, attr) in variant.attrs.iter().enumerate() {
            let Some(associated_groups) = try_parse_group_inert_attribute(attr) else {
                continue;
            };
            let associated_groups = match associated_groups {
                Ok(groups) => groups,
                Err(err) => {
                    errors.push(err.to_compile_error());
                    continue;
                }
            };

            for associated_group in associated_groups {
                let associated_group_name = associated_group.to_string();
                if !groups.contains(&associated_group_name) {
                    errors.push(
                        syn::Error::new(associated_group.span(), "unknown group name")
                            .to_compile_error(),
                    );
                    continue;
                }

                group_attr_index = Some(i);
                group_to_variants
                    .entry(associated_group_name.clone())
                    .or_default()
                    .push(variant_name.clone());
                variant_to_groups
                    .entry(variant_name.clone())
                    .or_default()
                    .push(associated_group_name);
            }
        }
        if let Some(group_attr_index) = group_attr_index {
            variant.attrs.remove(group_attr_index);
        }
    }
    if !errors.is_empty() {
        return errors.into_iter().collect();
    }

    let mut guard_macro_items: Vec<TokenStream> = vec![];
    let mut tester_macro_items: Vec<TokenStream> = vec![];
    for (group, variants) in &group_to_variants {
        let group = syn::Ident::new(group, proc_macro2::Span::call_site());

        let variants: Vec<syn::Ident> = variants
            .iter()
            .map(|variant| syn::Ident::new(variant, proc_macro2::Span::call_site()))
            .collect();
        for variant in variants.clone() {
            guard_macro_items.push(quote! {
                (#group, #variant $($tts:tt)*) => {
                    $ #current_mod_path :: #enum_name :: #variant $($tts)*
                };
            });
        }

        let variant_matchers: Vec<TokenStream> = variants
            .iter()
            .map(|variant| quote! { $ #current_mod_path :: #enum_name :: #variant { .. } })
            .collect();
        tester_macro_items.push(quote! {
            (#group, $($tts:tt)*) => {
                matches!($($tts)*, #(#variant_matchers)|*)
            };
        })
    }

    // 针对 `enum Event` 的特别逻辑。
    if let Err(token_stream) = check_for_event(&enum_name, &groups, &group_to_variants) {
        return token_stream;
    }

    quote! {
        macro_rules! #enum_guard_macro_name {
            #(#guard_macro_items)*
        }
        pub(crate) use #enum_guard_macro_name;

        #[allow(unused)]
        macro_rules! #debug_group_tester_macro_name {
            #(#tester_macro_items)*
        }
        #[allow(unused)]
        pub(crate) use #debug_group_tester_macro_name;

        #enum_item
    }
}

fn check_for_event(
    enum_name: &syn::Ident,
    groups: &HashSet<String>,
    group_to_variants: &HashMap<String, Vec<String>>,
) -> Result<(), TokenStream> {
    let mut errors: Vec<TokenStream> = vec![];

    if enum_name != "Event" {
        errors.push(
            syn::Error::new(
                enum_name.span(),
                "this macro is only designed for `enum Event`",
            )
            .to_compile_error(),
        );
    }

    let missing_groups: Vec<String> = groups
        .difference(&data_for_event::AVAILABLE_GROUPS)
        .cloned()
        .collect();
    let unexpected_groups: Vec<String> = data_for_event::AVAILABLE_GROUPS
        .difference(groups)
        .cloned()
        .collect();
    if !missing_groups.is_empty() {
        errors.push(
            syn::Error::new(
                enum_name.span(),
                format!("missing groups: {:?}", missing_groups),
            )
            .to_compile_error(),
        );
    }
    if !unexpected_groups.is_empty() {
        errors.push(
            syn::Error::new(
                enum_name.span(),
                format!("unexpected groups: {:?}", unexpected_groups),
            )
            .to_compile_error(),
        );
    }
    for (group, expected_variants) in data_for_event::GROUP_TO_EVENT.iter() {
        let Some(variants) = group_to_variants.get(group) else {
            continue;
        };
        let variants: HashSet<String> = variants.iter().map(|v| v.to_string()).collect();
        let expected_variants: HashSet<String> =
            expected_variants.iter().map(|v| v.to_string()).collect();
        let missing_variants: Vec<String> =
            expected_variants.difference(&variants).cloned().collect();
        let unexpected_variants: Vec<String> =
            variants.difference(&expected_variants).cloned().collect();
        if !missing_variants.is_empty() {
            errors.push(
                syn::Error::new(
                    enum_name.span(),
                    format!(
                        "missing variants in group `{}`: {:?}",
                        group, missing_variants
                    ),
                )
                .to_compile_error(),
            );
        }
        if !unexpected_variants.is_empty() {
            errors.push(
                syn::Error::new(
                    enum_name.span(),
                    format!(
                        "unexpected variants in group `{}`: {:?}",
                        group, unexpected_variants
                    ),
                )
                .to_compile_error(),
            );
        }
    }

    if !errors.is_empty() {
        Err(errors.into_iter().collect())
    } else {
        Ok(())
    }
}

struct AttributeArguments {
    current_mod_path: syn::Path,
    enum_guard_macro_name: syn::Ident,
    debug_group_tester_macro_name: syn::Ident,
    groups: HashSet<String>,
}

impl Parse for AttributeArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let current_mod_path = expect_key_value_path_pair(&input, "current_mod_path")?;
        let enum_guard_macro_name = expect_key_value_ident_pair(&input, "enum_guard_macro_name")?;
        let debug_group_tester_macro_name =
            expect_key_value_ident_pair(&input, "debug_group_tester_macro_name")?;

        let groups: HashSet<String> =
            syn::punctuated::Punctuated::<syn::Ident, syn::Token![|]>::parse_separated_nonempty(
                input,
            )?
            .into_iter()
            .map(|ident| ident.to_string())
            .collect();

        Ok(Self {
            current_mod_path,
            enum_guard_macro_name,
            debug_group_tester_macro_name,
            groups,
        })
    }
}

fn try_parse_group_inert_attribute(attr: &syn::Attribute) -> Option<syn::Result<Vec<syn::Ident>>> {
    if !attr.meta.path().is_ident("groups") {
        return None;
    }

    let syn::Meta::List(list_meta) = attr.meta.clone() else {
        return Some(Err(syn::Error::new(
            attr.span(),
            "expect meta to be in list style",
        )));
    };

    let groups: syn::Result<Vec<syn::Ident>> =
        list_meta.parse_args_with(|input: &syn::parse::ParseBuffer| {
            let idents =
                syn::punctuated::Punctuated::<syn::Ident, syn::Token![|]>::parse_separated_nonempty(input)?;

            Ok(idents.into_iter().collect())
        });

    Some(groups)
}
