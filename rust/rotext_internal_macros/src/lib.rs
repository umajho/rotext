mod macros;
mod utils;

#[proc_macro]
pub fn make_markup_pseudo_enum(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    macros::make_markup_pseudo_enum::make_markup_pseudo_enum(tokens.into()).into()
}
