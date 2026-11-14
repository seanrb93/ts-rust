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

/// `add` and `mul` of four lanes at once, each lane as the scalar function does it.
#[cfg(target_arch = "x86_64")]
pub mod lanes {
    use std::arch::x86_64::*;

    /// (SSE2 is part of x86_64)
    #[inline(always)]
    fn flush(x: __m128) -> __m128 {
        unsafe {
            let b = _mm_castps_si128(x);
            let zero_exp = _mm_cmpeq_epi32(_mm_and_si128(b, _mm_set1_epi32(0x7f80_0000)), _mm_setzero_si128());
            let sign = _mm_and_si128(b, _mm_set1_epi32(i32::MIN));
            _mm_castsi128_ps(_mm_or_si128(_mm_and_si128(zero_exp, sign), _mm_andnot_si128(zero_exp, b)))
        }
    }

    #[inline(always)]
    fn result(x: __m128) -> __m128 {
        unsafe {
            let b = _mm_castps_si128(x);
            let exp = _mm_and_si128(b, _mm_set1_epi32(0x7f80_0000));
            let zero_exp = _mm_cmpeq_epi32(exp, _mm_setzero_si128());
            let full_exp = _mm_cmpeq_epi32(exp, _mm_set1_epi32(0x7f80_0000));
            let sign = _mm_and_si128(b, _mm_set1_epi32(i32::MIN));
            let kept = _mm_andnot_si128(_mm_or_si128(zero_exp, full_exp), b);
            let largest = _mm_and_si128(full_exp, _mm_or_si128(sign, _mm_set1_epi32(super::MAX as i32)));
            let zero = _mm_and_si128(zero_exp, sign);
            _mm_castsi128_ps(_mm_or_si128(kept, _mm_or_si128(largest, zero)))
        }
    }

    #[inline(always)]
    pub fn load(v: [f32; 4]) -> __m128 {
        unsafe { _mm_loadu_ps(v.as_ptr()) }
    }

    #[inline(always)]
    pub fn store(v: __m128) -> [f32; 4] {
        let mut out = [0.0f32; 4];
        unsafe { _mm_storeu_ps(out.as_mut_ptr(), v) };
        out
    }

    #[inline(always)]
    pub fn splat(x: f32) -> __m128 {
        unsafe { _mm_set1_ps(x) }
    }

    #[inline(always)]
    pub fn add(a: __m128, b: __m128) -> __m128 {
        result(unsafe { _mm_add_ps(flush(a), flush(b)) })
    }

    #[inline(always)]
    pub fn mul(a: __m128, b: __m128) -> __m128 {
        result(unsafe { _mm_mul_ps(flush(a), flush(b)) })
    }
}

/// `add` and `mul` of four lanes at once, each lane as the scalar function does it.
#[cfg(not(target_arch = "x86_64"))]
pub mod lanes {
    pub type V = [f32; 4];
    #[inline(always)]
    pub fn load(v: [f32; 4]) -> V {
        v
    }
    #[inline(always)]
    pub fn store(v: V) -> [f32; 4] {
        v
    }
    #[inline(always)]
    pub fn splat(x: f32) -> V {
        [x; 4]
    }
    #[inline(always)]
    pub fn add(a: V, b: V) -> V {
        std::array::from_fn(|i| super::add(a[i], b[i]))
    }
    #[inline(always)]
    pub fn mul(a: V, b: V) -> V {
        std::array::from_fn(|i| super::mul(a[i], b[i]))
    }
}

#[cfg(test)]
mod lane_tests {
    use super::*;

    /// Each lane is what the scalar function gives, for numbers at the edges (denormals, zeros, overflow, infinity).
    #[test]
    fn lanes_are_the_scalar_functions() {
        let odd = [0.0f32, -0.0, 1.0, -1.0, 1e-40, -1e-40, 1e-20, -1e-20, 1e20, -1e20, 3e38, -3e38, f32::MAX, f32::INFINITY, f32::NEG_INFINITY, 1.5e-38, 0.75, -2.5];
        for &a in &odd {
            for &b in &odd {
                let va = [a, b, -a, 0.5 * b];
                let vb = [b, a, b, a];
                let s = lanes::store(lanes::mul(lanes::load(va), lanes::load(vb)));
                let t = lanes::store(lanes::add(lanes::load(va), lanes::load(vb)));
                for i in 0..4 {
                    assert_eq!(s[i].to_bits(), mul(va[i], vb[i]).to_bits(), "mul {} {}", va[i], vb[i]);
                    assert_eq!(t[i].to_bits(), add(va[i], vb[i]).to_bits(), "add {} {}", va[i], vb[i]);
                }
            }
        }
    }
}
