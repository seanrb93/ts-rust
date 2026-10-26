//! Vectors of three floats.

use crate::*;

/// `dst` = `src` scaled to length 1 (@ 0x25a5b8).  Divides, so it is exact; `v3NormalizeFaster` multiplies by a reciprocal.
pub fn v3Normalize(dst: i32, src: i32) {
    let (x, y, z) = (lwc1(src), lwc1(src + 4), lwc1(src + 8));
    let len = ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt();
    swc1(dst, (x).pdiv(len));
    swc1(dst + 4, (lwc1(src + 4)).pdiv(len));
    swc1(dst + 8, (lwc1(src + 8)).pdiv(len));
}

/// Like `v3Normalize` but multiplies by `1 / length` (@ 0x25a668).
pub fn v3NormalizeFaster(dst: i32, src: i32) {
    let (x, y, z) = (lwc1(src), lwc1(src + 4), lwc1(src + 8));
    let inv = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
    swc1(dst, ps2float::mul(x, inv));
    swc1(dst + 4, ps2float::mul(lwc1(src + 4), inv));
    swc1(dst + 8, ps2float::mul(lwc1(src + 8), inv));
}
