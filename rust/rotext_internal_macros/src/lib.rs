mod macros;
mod utils;

#[proc_macro]
pub fn make_markup_guard(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    macros::make_markup_guard::make_markup_guard(tokens.into()).into()
}
