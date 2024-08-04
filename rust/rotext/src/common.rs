use rotext_internal_macros::make_markup_pseudo_enum;

make_markup_pseudo_enum! {
    current_module_absolute_path = crate::common,
    markup_internal_module_name = __markup,
    markup_macro_name = m,
    is_markup_function_name = is_markup,
    0x21u8..=0x2fu8, 0x3au8..=0x40u8, 0x5bu8..=0x60u8, 0x7bu8..=0x7eu8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    start: usize,
    length: usize,
}

impl Range {
    #[inline(always)]
    pub fn new(start: usize, length: usize) -> Range {
        Range { start, length }
    }

    #[inline(always)]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline(always)]
    pub fn length(&self) -> usize {
        self.length
    }

    #[inline(always)]
    pub fn set_length(&mut self, value: usize) {
        self.length = value;
    }

    #[inline(always)]
    pub fn increase_length(&mut self, delta: usize) {
        self.length += delta;
    }

    pub fn content_in_u8_array<'a>(&self, input: &'a [u8]) -> &'a [u8] {
        &input[self.start..self.start + self.length]
    }
    pub fn content<'a>(&self, input: &'a [u8]) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.content_in_u8_array(input)) }
    }
}
