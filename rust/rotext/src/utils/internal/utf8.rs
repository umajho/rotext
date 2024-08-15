pub fn get_byte_length_by_first_char(first_byte: u8) -> usize {
    match first_byte >> 4 {
        0b1111 => 4,
        0b1110 => 3,
        0b1100 => 2,
        _ => 1,
    }
}
