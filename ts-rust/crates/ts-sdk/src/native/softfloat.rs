//! libgcc soft-float double-precision helpers.  The EE has no double-precision FPU, so the original compiler routed
//! every `double` operation through these.  Doubles are passed around as their 64-bit IEEE patterns.

#[inline]
fn d(a: i64) -> f64 {
    f64::from_bits(a as u64)
}
#[inline]
fn b(x: f64) -> i64 {
    x.to_bits() as i64
}

pub fn fptodp(f12: f32) -> i64 {
    b(f12 as f64)
}
pub fn dptofp(a0: i64) -> f32 {
    d(a0) as f32
}
pub fn dpadd(a0: i64, a1: i64) -> i64 {
    b(d(a0) + d(a1))
}
pub fn dpsub(a0: i64, a1: i64) -> i64 {
    b(d(a0) - d(a1))
}
pub fn dpmul(a0: i64, a1: i64) -> i64 {
    b(d(a0) * d(a1))
}
pub fn dpdiv(a0: i64, a1: i64) -> i64 {
    b(d(a0) / d(a1))
}
/// Three-way compare: negative, zero or positive.
pub fn dpcmp(a0: i64, a1: i64) -> i32 {
    match d(a0).partial_cmp(&d(a1)) {
        Some(std::cmp::Ordering::Less) => -1,
        Some(std::cmp::Ordering::Equal) => 0,
        _ => 1,
    }
}
pub fn dptoli(a0: i64) -> i32 {
    d(a0) as i32
}
pub fn dptoul(a0: i64) -> i32 {
    d(a0) as u32 as i32
}
pub fn fptoui(f12: f32) -> i32 {
    f12 as u32 as i32
}
pub fn __divdi3(a0: i64, a1: i64) -> i64 {
    a0.wrapping_div(a1)
}
