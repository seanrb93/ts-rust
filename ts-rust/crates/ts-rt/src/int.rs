//! Integer helpers matching R5900 semantics that have no direct Rust operator.

/// High word of `{hi, lo} + a*b` (signed multiply-accumulate; the low word is computed inline by the caller).
#[inline(always)]
pub fn maddhi(hi: i32, lo: i32, a: i32, b: i32) -> i32 {
    let acc = ((hi as i64) << 32) | (lo as u32 as i64);
    (acc.wrapping_add((a as i64) * (b as i64)) >> 32) as i32
}

#[inline(always)]
pub fn maddhiu(hi: i32, lo: i32, a: i32, b: i32) -> i32 {
    let acc = ((hi as u32 as u64) << 32) | (lo as u32 as u64);
    (acc.wrapping_add((a as u32 as u64) * (b as u32 as u64)) >> 32) as i32
}
