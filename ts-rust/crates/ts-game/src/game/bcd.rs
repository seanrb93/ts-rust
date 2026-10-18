//! Binary-coded decimal, as the memory card's clock fields use.

/// @ 0x2b3988
pub fn bcd2int(bcd: i32) -> i32 {
    (bcd >> 4) * 10 + (bcd & 15)
}

/// @ 0x2b39a0
pub fn int2bcd(n: i32) -> i32 {
    ((n.wrapping_div(10)) << 4) + (n - n.wrapping_div(10) * 10)
}
