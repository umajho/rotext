use rotext_internal_macros::make_markup_guard;

make_markup_guard! {
    markup_guard_macro_name = m,
    is_markup_function_name = is_markup,
    0x21u8..=0x2fu8, 0x3au8..=0x40u8, 0x5bu8..=0x60u8, 0x7bu8..=0x7eu8
}

pub fn is_valid_character_in_name(char: u8) -> bool {
    !matches!(
        char,
        m!('{') | m!('}') | m!('[') | m!(']') | m!('<') | m!('>') | m!('|')
    )
}
