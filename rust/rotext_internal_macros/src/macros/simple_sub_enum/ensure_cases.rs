use std::collections::HashSet;

use proc_macro2::TokenStream;

use quote::{quote, ToTokens};
use syn::{self, parse::Parse, spanned::Spanned};

use crate::utils::{expect_key_value_ident_pair, expect_key_value_path_pair};

use super::data_for_event::{ALL_EVENTS, AVAILABLE_GROUPS, GROUP_TO_EVENT};

fn parse_attribute_arguments(tokens: TokenStream) -> Result<AttributeArguments, TokenStream> {
    syn::parse2::<AttributeArguments>(tokens).map_err(|err| err.to_compile_error())
}

pub fn ensure_cases_for_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let AttributeArguments { prefix, group } = match parse_attribute_arguments(attr) {
        Ok(attr) => attr,
        Err(err) => return err,
    };

    let match_expr: syn::ExprMatch = match syn::parse2(item) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };

    let group_name = &group.to_string();

    if !AVAILABLE_GROUPS.contains(group_name) {
        return syn::Error::new(group.span(), format!("unknown group {}", group))
            .to_compile_error();
    }

    let events_in_group = GROUP_TO_EVENT
        .get(group_name)
        .unwrap_or_else(|| panic!("group {} is missing in GROUP_TO_EVENT", group));

    let mut errors: Vec<TokenStream> = vec![];
    let mut seen_variants: HashSet<String> = HashSet::new();
    let mut has_wild: bool = false;
    let mut reconstructed_match = {
        let mut reconstructed_match = match_expr.clone();
        reconstructed_match.arms = vec![];
        reconstructed_match
    };

    for arm in match_expr.clone().arms {
        match check_pattern(
            group_name,
            &arm.pat,
            &prefix,
            &mut seen_variants,
            &mut has_wild,
            events_in_group,
            &ALL_EVENTS,
        ) {
            Ok(()) => {
                reconstructed_match.arms.push(arm.clone());
            }
            Err(err) => {
                errors.push(err.to_compile_error());
            }
        }
    }

    if !errors.is_empty() {
        return errors.into_iter().collect();
    }

    if !has_wild {
        let missing_variants: HashSet<_> = events_in_group.difference(&seen_variants).collect();
        if !missing_variants.is_empty() {
            return syn::Error::new(
                match_expr.expr.span(),
                format!(
                    "missing arms for events: {}",
                    missing_variants
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
            .to_compile_error();
        }

        reconstructed_match.arms.push(make_wild_arm());
    }

    reconstructed_match.to_token_stream()
}

fn check_pattern(
    group_name: &str,
    pat: &syn::Pat,
    prefix: &syn::Path,
    seen_variants: &mut HashSet<String>,
    has_wild: &mut bool,
    expected_events: &HashSet<String>,
    known_events: &HashSet<String>,
) -> syn::Result<()> {
    match pat {
        syn::Pat::Or(pat) => {
            for pat in &pat.cases {
                check_pattern(
                    group_name,
                    pat,
                    prefix,
                    seen_variants,
                    has_wild,
                    expected_events,
                    known_events,
                )?;
            }
            Ok(())
        }
        syn::Pat::Path(pat) => {
            let variant = trim_prefix(&pat.path, prefix)?;
            check_event_availability(group_name, &variant, expected_events, known_events)?;
            seen_variants.insert(variant.to_string());
            Ok(())
        }
        syn::Pat::TupleStruct(pat) => {
            let variant = trim_prefix(&pat.path, prefix)?;
            check_event_availability(group_name, &variant, expected_events, known_events)?;
            seen_variants.insert(variant.to_string());
            Ok(())
        }
        syn::Pat::Wild(_) => {
            *has_wild = true;
            Ok(())
        }
        _ => Err(syn::Error::new(pat.span(), "pattern not (yet?) supported")),
    }
}

fn check_event_availability(
    group_name: &str,
    ident: &syn::Ident,
    expected_events: &HashSet<String>,
    known_events: &HashSet<String>,
) -> syn::Result<()> {
    let name = ident.to_string();
    if expected_events.contains(&name) {
        Ok(())
    } else if known_events.contains(&name) {
        Err(syn::Error::new(
            ident.span(),
            format!("event {} is not in group {}", ident, group_name),
        ))
    } else {
        Err(syn::Error::new(
            ident.span(),
            format!("unknown event {}", ident),
        ))
    }
}

fn trim_prefix(path: &syn::Path, prefix: &syn::Path) -> syn::Result<syn::Ident> {
    if prefix.segments.len() != path.segments.len() - 1 {
        return Err(syn::Error::new(
            path.span(),
            "path should have exactly one more segment than prefix",
        ));
    }

    for (seg_prefix, seg_path) in prefix.segments.iter().zip(path.segments.iter()) {
        check_path_segment_no_arguments(seg_prefix)?;
        check_path_segment_no_arguments(seg_path)?;
        if seg_prefix.ident != seg_path.ident {
            return Err(syn::Error::new(
                seg_path.ident.span(),
                format!(
                    "expected segment {} but found {}",
                    seg_prefix.ident, seg_path.ident
                ),
            ));
        }
    }

    let last = path.segments.last().unwrap();
    check_path_segment_no_arguments(last)?;
    Ok(last.ident.clone())
}

fn check_path_segment_no_arguments(seg: &syn::PathSegment) -> syn::Result<()> {
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return Err(syn::Error::new(
            seg.span(),
            "arguments in path not supported",
        ));
    }

    Ok(())
}

fn make_wild_arm() -> syn::Arm {
    let match_expr: syn::ExprMatch = syn::parse2(quote! {
        match foo {
            _ => panic!("unreachable")
        }
    })
    .unwrap();
    match_expr.arms.first().cloned().unwrap()
}

struct AttributeArguments {
    prefix: syn::Path,
    group: syn::Ident,
}

impl Parse for AttributeArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let prefix = expect_key_value_path_pair(&input, "prefix")?;
        let group = expect_key_value_ident_pair(&input, "group")?;

        Ok(Self { prefix, group })
    }
}
