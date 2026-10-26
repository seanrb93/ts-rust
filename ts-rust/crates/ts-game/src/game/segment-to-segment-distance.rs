//! The squared distance between two line segments.

use crate::*;

/// A float widened to a double.
fn dp(x: f32) -> i64 {
    fptodp(x)
}

/// Twice a float, as a double.
fn twice(x: f32) -> i64 {
    dpadd(dp(x), dp(x))
}

/// `(a + 2 b) + c` summed in doubles and narrowed to a float.
fn quad(a: f32, b: f32, c: f32) -> f32 {
    dptofp(dpadd(dpadd(dp(a), twice(b)), dp(c)))
}

/// `a + 2 b` summed in doubles and narrowed to a float.
fn tail(a: f32, b: f32) -> f32 {
    dptofp(dpadd(dp(a), twice(b)))
}

/// `c + t (first + second)` in doubles, narrowed to a float.
fn along(c: i64, t: f32, first: i64, second: i64) -> f32 {
    dptofp(dpadd(c, dpmul(dp(t), dpadd(first, second))))
}

/// The two parameters (along the first segment and the second) of the closest points and the squared distance, from the dot products of the segments: `a00` the length squared
/// of the first direction, `a01` minus the dot product of the two directions, `a11` the length squared of the second, `b0` the dot product of the offset and the first direction,
/// `c` the length squared of the offset, `det` the size of the determinant `a00 a11 - a01 a01` and `d1d2` the dot product of the directions; `sp` holds the offset (at sp), the
/// first direction (at sp+16) and the second (at sp+32). This follows the usual case analysis for two segments (Magic Software's), with the mix of float and double sums the
/// game's compiler made of it.
fn closest(sp: i32, a00: f32, a01: f32, a11: f32, b0: f32, c: f32, det: f32, d1d2: f32) -> (f32, f32, f32) {
    let b1_positive = lwc1(sp) * lwc1(sp + 32) + lwc1(sp + 4) * lwc1(sp + 36) + lwc1(sp + 8) * lwc1(sp + 40);
    if dpcmp(dp(det), ld(0x39a0c0)) < 0 {
        // the segments are parallel
        let b1 = -b1_positive;
        if dpcmp(dp(a01), 0_i64) <= 0 {
            let negative_b0 = -b0;
            if a00 <= negative_b0 {
                return (1.0_f32, 0.0_f32, quad(a00, b0, c));
            }
            if dpcmp(dp(b0), 0_i64) <= 0 {
                let s = (negative_b0).pdiv(a00);
                return (s, 0.0_f32, b0 * s + c);
            }
            if d1d2 <= b0 {
                return (0.0_f32, 1.0_f32, quad(a11, b1, c));
            }
            let t = (negative_b0).pdiv(a01);
            return (0.0_f32, t, along(dp(c), t, twice(b1), dp(a11 * t)));
        }
        if dpcmp(dp(b0), 0_i64) >= 0 {
            return (0.0_f32, 0.0_f32, c);
        }
        let negative_b0 = -b0;
        if negative_b0 <= a00 {
            let s = (negative_b0).pdiv(a00);
            return (s, 0.0_f32, b0 * s + c);
        }
        let beyond = -(a00 + b0);
        if a01 <= beyond {
            return (1.0_f32, 1.0_f32, tail(a00 + a11 + c, a01 + b0 + b1));
        }
        let t = (beyond).pdiv(a01);
        let start = dpadd(dpadd(dp(a00), twice(b0)), dp(c));
        return (1.0_f32, t, along(start, t, dp(a11 * t), twice(a01 + b1)));
    }
    let b1 = -b1_positive;
    let s_numerator = a01 * b1 - a11 * b0;
    let t_numerator = a01 * b0 - a00 * b1;
    // the end of a side where the first parameter is 0 and the second is looked for from the offset: the second end at 1, or inside the segment
    let at_s_zero = || -> (f32, f32, f32) {
        if dpcmp(dp(b1), 0_i64) < 0 {
            if !(a11 <= b1_positive) {
                let t = (b1_positive).pdiv(a11);
                (0.0_f32, t, b1 * t + c)
            } else {
                (0.0_f32, 1.0_f32, quad(a11, b1, c))
            }
        } else {
            (0.0_f32, 0.0_f32, c)
        }
    };
    // the first parameter is 1: the second is looked for the same way
    let at_s_one = |sum: f32| -> (f32, f32, f32) {
        if dpcmp(dp(sum), 0_i64) < 0 {
            let negative = -sum;
            if !(a11 <= negative) {
                let t = (negative).pdiv(a11);
                (1.0_f32, t, quad(sum * t + a00, b0, c))
            } else {
                (1.0_f32, 1.0_f32, tail(a00 + a11 + c, b0 + sum))
            }
        } else {
            (1.0_f32, 0.0_f32, quad(a00, b0, c))
        }
    };
    // the second parameter is 1: the first is looked for from the sum
    let at_t_one = |sum: f32| -> (f32, f32, f32) {
        if dpcmp(dp(sum), 0_i64) < 0 {
            let negative = -sum;
            if !(a00 <= negative) {
                let s = (negative).pdiv(a00);
                (s, 1.0_f32, quad(sum * s + a11, b1, c))
            } else {
                (1.0_f32, 1.0_f32, tail(a00 + a11 + c, b1 + sum))
            }
        } else {
            (0.0_f32, 1.0_f32, quad(a11, b1, c))
        }
    };
    if dpcmp(dp(s_numerator), 0_i64) < 0 {
        if dpcmp(dp(t_numerator), 0_i64) < 0 {
            if dpcmp(dp(b0), 0_i64) < 0 {
                let negative_b0 = -b0;
                if !(a00 <= negative_b0) {
                    let s = (negative_b0).pdiv(a00);
                    return (s, 0.0_f32, b0 * s + c);
                }
                return (1.0_f32, 0.0_f32, quad(a00, b0, c));
            }
            return at_s_zero();
        }
        if !(t_numerator <= det) {
            let sum = a01 + b0;
            if dpcmp(dp(sum), 0_i64) < 0 {
                let negative = -sum;
                if !(a00 <= negative) {
                    let s = (negative).pdiv(a00);
                    return (s, 1.0_f32, quad(sum * s + a11, b1, c));
                }
                return (1.0_f32, 1.0_f32, tail(a00 + a11 + c, b1 + sum));
            }
        }
        return at_s_zero();
    }
    if !(s_numerator <= det) {
        if dpcmp(dp(t_numerator), 0_i64) >= 0 {
            if !(t_numerator <= det) {
                let sum = a01 + b0;
                let negative = -sum;
                if !(negative <= a00) {
                    return at_s_one(a01 + b1);
                }
                if dpcmp(dp(sum), 0_i64) < 0 {
                    let s = (negative).pdiv(a00);
                    return (s, 1.0_f32, quad(sum * s + a11, b1, c));
                }
                return (0.0_f32, 1.0_f32, quad(a11, b1, c));
            }
            return at_s_one(a01 + b1);
        }
        let negative_b0 = -b0;
        if !(negative_b0 < a00) {
            return at_s_one(a01 + b1);
        }
        if dpcmp(dp(b0), 0_i64) < 0 {
            let s = (negative_b0).pdiv(a00);
            return (s, 0.0_f32, b0 * s + c);
        }
        return (0.0_f32, 0.0_f32, c);
    }
    if dpcmp(dp(t_numerator), 0_i64) >= 0 {
        if !(t_numerator <= det) {
            return at_t_one(a01 + b0);
        }
        let inverse = dptofp(dpdiv(0x3ff0000000000000_i64, dp(det)));
        let s = s_numerator * inverse;
        let t = t_numerator * inverse;
        let first = dpmul(dp(s), dpadd(dp(a00 * s + a01 * t), twice(b0)));
        let second = dpmul(dp(t), dpadd(dp(a01 * s + a11 * t), twice(b1)));
        return (s, t, dptofp(dpadd(dpadd(first, second), dp(c))));
    }
    if dpcmp(dp(b0), 0_i64) < 0 {
        let negative_b0 = -b0;
        if !(a00 <= negative_b0) {
            let s = (negative_b0).pdiv(a00);
            return (s, 0.0_f32, b0 * s + c);
        }
        return (1.0_f32, 0.0_f32, quad(a00, b0, c));
    }
    (0.0_f32, 0.0_f32, c)
}

/// The squared distance between the segment from `a0` to `a1` and the segment from `a2` to `a3` (positions of three floats); the closest points are written to `a4` (on the
/// first segment) and `a5` (on the second) when they are not null (@ 0x20b1a8).
pub fn HitTestDistanceLineSegmentToLineSegmentSquared(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) -> f32 {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let f7_1 = lwc1(a2);
    let f0_1 = lwc1(a0 + 4);
    let f1_1 = lwc1(a2 + 4);
    let f3_1 = lwc1(a3) - f7_1;
    let f31_1 = lwc1(a0);
    let f12_1 = f0_1 - f1_1;
    let f8_1 = f31_1 - f7_1;
    let f2_1 = lwc1(a1) - f31_1;
    let f5_1 = lwc1(a1 + 4) - f0_1;
    let f14_1 = lwc1(a2 + 8);
    let f6_1 = lwc1(a3 + 4) - f1_1;
    let f1_2 = lwc1(a0 + 8);
    let f0_2 = lwc1(a3 + 8);
    let f4_1 = lwc1(a1 + 8) - f1_2;
    swc1(sp, f8_1);
    swc1(sp + 4, f12_1);
    let f0_3 = f0_2 - f14_1;
    swc1(sp + 32, f3_1);
    swc1(sp + 36, f6_1);
    swc1(sp + 16, f2_1);
    swc1(sp + 40, f0_3);
    swc1(sp + 20, f5_1);
    swc1(sp + 24, f4_1);
    let d1d2 = f2_1 * f3_1 + f5_1 * f6_1 + f4_1 * f0_3;
    let a01 = -d1d2;
    let a11 = f3_1 * f3_1 + f6_1 * f6_1 + f0_3 * f0_3;
    let a00 = f2_1 * f2_1 + f5_1 * f5_1 + f4_1 * f4_1;
    let f1_3 = f1_2 - f14_1;
    swc1(sp + 8, f1_3);
    let mut det = a00 * a11 - a01 * a01;
    let b0 = f8_1 * f2_1 + f12_1 * f5_1 + f1_3 * f4_1;
    let c = f8_1 * f8_1 + f12_1 * f12_1 + f1_3 * f1_3;
    if !(0.0_f32 <= det) {
        det = -det;
    }
    let (s, t, mut squared) = closest(sp, a00, a01, a11, b0, c, det, d1d2);
    let f1_5 = lwc1(sp + 16);
    if a4 != 0 {
        let f2_2 = s * lwc1(sp + 20);
        let f3_2 = s * lwc1(sp + 24);
        swc1(a4, f31_1 + s * f1_5);
        swc1(a4 + 4, lwc1(a0 + 4) + f2_2);
        swc1(a4 + 8, lwc1(a0 + 8) + f3_2);
    }
    let f0_17 = lwc1(sp + 32);
    if a5 != 0 {
        let f2_3 = t * lwc1(sp + 36);
        let f3_3 = t * lwc1(sp + 40);
        swc1(a5, lwc1(a2) + t * f0_17);
        swc1(a5 + 4, lwc1(a2 + 4) + f2_3);
        swc1(a5 + 8, lwc1(a2 + 8) + f3_3);
    }
    if !(0.0_f32 <= squared) {
        squared = -squared;
    }
    squared
}
