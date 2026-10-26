//! The arithmetic of the console's floating point unit, which the game was written for and which differs from the IEEE arithmetic of the host where it matters to the
//! game: it has no infinity and no NaN (a result too large for a float, or a division by zero, is the largest float with the sign the result would have), it treats a denormal
//! number as zero, and the square root of a negative number is the square root of its absolute value.  `ts-verify` uses the same functions for the instructions of the
//! original, and the hand-written code uses them where a division or a square root may meet a zero or an overflow.

const MAX: u32 = 0x7f7f_ffff;

/// A denormal number is zero (the sign is kept).
#[inline]
pub fn flush(x: f32) -> f32 {
    let b = x.to_bits();
    if (b >> 23) & 0xff == 0 {
        f32::from_bits(b & 0x8000_0000)
    } else {
        x
    }
}

/// A result of the unit: infinity and NaN become the largest float with the sign of the result, a denormal result is zero.
#[inline]
pub fn result(x: f32) -> f32 {
    let b = x.to_bits();
    match (b >> 23) & 0xff {
        0xff => f32::from_bits((b & 0x8000_0000) | MAX),
        0 => f32::from_bits(b & 0x8000_0000),
        _ => x,
    }
}

#[inline]
pub fn add(a: f32, b: f32) -> f32 {
    result(flush(a) + flush(b))
}

#[inline]
pub fn sub(a: f32, b: f32) -> f32 {
    result(flush(a) - flush(b))
}

#[inline]
pub fn mul(a: f32, b: f32) -> f32 {
    result(flush(a) * flush(b))
}

/// `a / b`: a zero divisor gives the largest float (sign of `a` xor sign of `b`), also for 0 / 0.
#[inline]
pub fn div(a: f32, b: f32) -> f32 {
    let (a, b) = (flush(a), flush(b));
    if b == 0.0 {
        f32::from_bits(((a.to_bits() ^ b.to_bits()) & 0x8000_0000) | MAX)
    } else {
        result(a / b)
    }
}

/// The square root of the absolute value.
#[inline]
pub fn sqrt(b: f32) -> f32 {
    flush(b).abs().sqrt()
}

/// `a / sqrt(|b|)` (the instruction RSQRT.S): a zero `b` gives the largest float with the sign of `a`.
#[inline]
pub fn rsqrt(a: f32, b: f32) -> f32 {
    let (a, b) = (flush(a), flush(b));
    if b == 0.0 {
        f32::from_bits((a.to_bits() & 0x8000_0000) | MAX)
    } else {
        result(a / b.abs().sqrt())
    }
}

/// Methods of the unit for `f32`, for the hand-written code: `a.pdiv(b)` is `a / b` and `x.psqrt()` is the square root of `x`, as the console does them (see above).
pub trait Ps2F {
    fn pdiv(self, b: f32) -> f32;
    fn psqrt(self) -> f32;
}

impl Ps2F for f32 {
    #[inline]
    fn pdiv(self, b: f32) -> f32 {
        div(self, b)
    }
    #[inline]
    fn psqrt(self) -> f32 {
        sqrt(self)
    }
}
