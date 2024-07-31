pub(crate) fn read_unsigned_short(bytes: &[u8], index: &mut usize) -> u16 {
    let value = ((bytes[*index] as u16) << 8) | (bytes[*index + 1] as u16);
    *index += 2;
    value
}
