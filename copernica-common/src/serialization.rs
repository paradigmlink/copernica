pub fn u16_to_u8(i: u16) -> [u8; 2] {
    [(i >> 8) as u8, i as u8]
}
pub fn u8_to_u16(i: [u8; 2]) -> u16 {
    ((i[0] as u16) << 8) | i[1] as u16
}
pub fn u8_to_u64(v: [u8; 8]) -> u64 {
    let mut x: u64 = 0;
    for i in 0..v.len() {
        x = ((x << 8) | v[i] as u64) as u64;
    }
    x
}
pub fn u64_to_u8(x: u64) -> [u8; 8] {
    [((x >> 56) & 0xff) as u8,
    ((x  >> 48) & 0xff) as u8,
    ((x  >> 40) & 0xff) as u8,
    ((x  >> 32) & 0xff) as u8,
    ((x  >> 24) & 0xff) as u8,
    ((x  >> 16) & 0xff) as u8,
    ((x  >> 8)  & 0xff) as u8,
    (x          & 0xff) as u8]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_u16_to_fro_u8() {
        let actual: u16 = u16::MIN;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u16 = 1;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u16 = u16::MAX;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_bfi_to_fro_u8() {
        let actual: BFI = [0u16; BLOOM_FILTER_INDEX_ELEMENT_LENGTH];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: BFI = [0, 1, 2, 3];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: BFI = [u16::MAX, u16::MAX, u16::MAX, u16::MAX];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_u64_to_fro_u8() {
        let actual: u64 = 0;
        let expected: u64 = u8_to_u64(u64_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u64 = u64::MAX/2;
        let expected: u64 = u8_to_u64(u64_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u64 = u64::MAX;
        let expected: u64 = u8_to_u64(u64_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
}
