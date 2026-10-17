//! libm.  Single-precision functions map to Rust's f32 methods; double-precision ones operate on the 64-bit
//! patterns the original soft-float code passes in integer registers.
#![allow(non_snake_case)]

pub fn sqrtf(f12: f32) -> f32 {
    f12.abs().sqrt()
}
pub fn sinf(f12: f32) -> f32 {
    f12.sin()
}
pub fn cosf(f12: f32) -> f32 {
    f12.cos()
}
pub fn tanf(f12: f32) -> f32 {
    f12.tan()
}
pub fn acosf(f12: f32) -> f32 {
    f12.acos()
}
pub fn asinf(f12: f32) -> f32 {
    f12.asin()
}
pub fn floorf(f12: f32) -> f32 {
    f12.floor()
}
pub fn atan2f(f12: f32, f13: f32) -> f32 {
    f12.atan2(f13)
}
pub fn powf(f12: f32, f13: f32) -> f32 {
    f12.powf(f13)
}

#[inline]
fn d(a: i64) -> f64 {
    f64::from_bits(a as u64)
}
#[inline]
fn b(x: f64) -> i64 {
    x.to_bits() as i64
}

pub fn sqrt(a0: i64) -> i64 {
    b(d(a0).abs().sqrt())
}
pub fn atan(a0: i64) -> i64 {
    b(d(a0).atan())
}
pub fn floor(a0: i64) -> i64 {
    b(d(a0).floor())
}
pub fn atan2(a0: i64, a1: i64) -> i64 {
    b(d(a0).atan2(d(a1)))
}
pub fn pow(a0: i64, a1: i64) -> i64 {
    b(d(a0).powf(d(a1)))
}
