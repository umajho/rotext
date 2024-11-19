use syn::{self, Token};

pub fn expect_key_value_path_pair(
    input: &syn::parse::ParseStream,
    key: &'static str,
) -> syn::Result<syn::Path> {
    let ident: syn::Ident = input.parse()?;
    if ident != key {
        return Err(input.error(format!("expect `{}`", key)));
    }
    let _: Token![=] = input.parse()?;
    let path: syn::Path = input.parse()?;
    let _: Token![,] = input.parse()?;

    Ok(path)
}

pub fn expect_key_value_ident_pair(
    input: &syn::parse::ParseStream,
    key: &'static str,
) -> syn::Result<syn::Ident> {
    let ident: syn::Ident = input.parse()?;
    if ident != key {
        return Err(input.error(format!("expect `{}`", key)));
    }
    let _: Token![=] = input.parse()?;
    let path: syn::Ident = input.parse()?;
    let _: Token![,] = input.parse()?;

    Ok(path)
}
