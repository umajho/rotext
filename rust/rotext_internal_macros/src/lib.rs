mod macros;
mod utils;

#[proc_macro]
pub fn make_markup_guard(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    macros::make_markup_guard::make_markup_guard(tokens.into()).into()
}

/// 针对 `enum Event`。
/// 简单的、零开销的子枚举实现。
///
/// 在类型安全方面，没有 crate `subenum` 安全，但是没有运行时开销（无需类型转换）。
///
/// TODO: 也许可以考虑使用 newtype 来实现一定程度的类型安全。子枚举之间的转换在
/// debug_assertion 时检查类型。
#[proc_macro_attribute]
pub fn simple_sub_enum_for_event(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    macros::simple_sub_enum::simple_sub_enum_for_event(attr.into(), item.into()).into()
}
/// 针对 `enum Event`。
/// 确保对子枚举的 match 正确。（具体来说：确保不会去匹配在基础枚举中有而在子枚举
/// 中没有的变体，或者反过来的情况。）
///
/// 参数中的 `prefix` 用于指定匹配时表示基础枚举的路径。
#[proc_macro_attribute]
pub fn ensure_cases_for_event(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    macros::simple_sub_enum::ensure_cases::ensure_cases_for_event(attr.into(), item.into()).into()
}
