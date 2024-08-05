use rotext_internal_macros::make_markup_pseudo_enum;

make_markup_pseudo_enum! {
    current_module_absolute_path = crate::common,
    markup_internal_module_name = __markup,
    markup_macro_name = m,
    is_markup_function_name = is_markup,
    0x21u8..=0x2fu8, 0x3au8..=0x40u8, 0x5bu8..=0x60u8, 0x7bu8..=0x7eu8
}
