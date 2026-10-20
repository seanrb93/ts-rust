//! The spawning of a shattering glass particle.

use super::particle_spawners::random_as_float;
use crate::*;

/// The lowest of the four values `a`, `b`, `c` and `d` (the order the game compares them in matters for equal values and for NaN) minus `offset`.
fn lowest_minus(a: f32, b: f32, c: f32, d: f32, offset: f32) -> f32 {
    'b34: {
        'b35: {
            'b36: {
                'b37: {
                    if !(b < a) {
                        if !(c < a) {
                            break 'b37;
                        }
                    } else if !(c < b) {
                        break 'b37;
                    }
                    if d < c {
                        break 'b36;
                    }
                    break 'b35;
                }
                if !(b < a) {
                    if !(d < a) {
                        break 'b35;
                    }
                } else if !(d < b) {
                    break 'b35;
                }
            }
            return d - offset;
        }
        'b39: {
            if !(b < a) {
                if !(c < a) {
                    break 'b39;
                }
            } else if !(c < b) {
                break 'b39;
            }
            return c - offset;
        }
        if !(b < a) {
            return a - offset;
        } else {
            return b - offset;
        }
    }
}

/// The highest of the four values plus `offset`, compared the way the game does.
fn highest_plus(a: f32, b: f32, c: f32, d: f32, offset: f32) -> f32 {
    'b42: {
        'b43: {
            'b44: {
                if !(a < b) {
                    if !(a < c) {
                        break 'b44;
                    }
                } else if !(b < c) {
                    break 'b44;
                }
                if c < d {
                    break 'b43;
                }
                break 'b42;
            }
            if !(a < b) {
                if !(a < d) {
                    break 'b42;
                }
            } else if !(b < d) {
                break 'b42;
            }
        }
        return d + offset;
    }
    'b46: {
        if !(a < b) {
            if !(a < c) {
                break 'b46;
            }
        } else if !(b < c) {
            break 'b46;
        }
        return c + offset;
    }
    if !(a < b) {
        a + offset
    } else {
        b + offset
    }
}

/// Make the value at `dest` the `base` moved by a random amount of up to the constant at `scale_address`, keeping it between 0 and 1 (outside that the base is kept).
fn jitter(base: f32, scale_address: i32, dest: i32) {
    let roll = random_as_float();
    let value = base + (roll * 2.3283064e-10_f32 - 0.5_f32) * lwc1(scale_address);
    swc1(dest, value);
    if !(value <= 0.0_f32) {
        if !(1.0_f32 <= value) {
            return;
        }
    }
    swc1(dest, base);
}

/// Hand the pieces of one quad of the glass to `glassSetQuad`: the stack holds the arguments past the eighth (from sp up in steps of 8) which the original wrote just before the
/// call; `first` is the first quad of a triangle's fan, which fills the third to sixth of them in a different order.
fn set_quad(sp: i32, s6: i32, fp: i32, number: i32, size: f32, flag: i32, first: bool) {
    sw(sp, lw(sp + 496));
    sw(sp + 8, lw(sp + 556));
    sw(sp + 16, lw(sp + 528));
    if first {
        sw(sp + 24, lw(sp + 532));
        sw(sp + 32, lw(sp + 536));
        sw(sp + 40, lw(sp + 540));
        sw(sp + 48, lw(sp + 540));
    } else {
        sw(sp + 24, lw(sp + 536));
        sw(sp + 32, lw(sp + 532));
        sw(sp + 40, lw(sp + 540));
        sw(sp + 48, lw(sp + 552));
    }
    sw(sp + 56, s6);
    sw(sp + 64, lw(sp + 544));
    sw(sp + 72, lw(sp + 548));
    sw(sp + 80, flag);
    sw(sp + 88, lw(sp + 480));
    glassSetQuad(lw(sp + 504), lw(sp + 508), lw(sp + 512), fp, number, lw(sp + 484), lw(sp + 488), lw(sp + 492), size);
}

/// Spawn the glass particle of the pane with the corners `a4`, `a5`, `a6` and `a7` (four positions; `a1` the flags, 256 for a mirror that reflects the direction in the normal
/// at the ninth stack argument): a chain of four groups is taken (the first with the type 4, the others 0), the pane is split into the 16 pieces of a ring fan around its
/// middle, each piece cut into five quads that fly apart along random speeds (`glassSetQuad`), the colour of a pane without the mirror flag follows the colour at the 10th stack
/// argument, and the rooms that the box of the pane touches are looked up (`bgCalcRoomArray`) (@ 0x299970). The 9th, 10th and 12th stack arguments are the colour limits at
/// +832, the direction at +840 and the normal at +856; the colour is at +848.
pub fn particleNewGlass(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, f12: f32) {
    let frame = Frame::new(832);
    let sp = frame.sp();
    let mut size = f12;
    sw(sp + 480, a1);
    sw(sp + 484, a4);
    sw(sp + 488, a5);
    sw(sp + 492, a6);
    sw(sp + 496, a7);
    let s6 = lw(sp + 832);
    let s3 = lw(sp + 840);
    let s4 = lw(sp + 856);
    let group = getgroup(7);
    sw(sp + 500, group);
    let f0_1 = lwc1(0x3ad4d8);
    let a0_2 = lw(sp + 500);
    sw(sp + 504, group + 928);
    let v0_2 = lw(a0_2 + 60);
    sw(a0_2 + 8, 4);
    sw(sp + 508, v0_2 + 928);
    sw(v0_2 + 8, 0);
    let v1_1 = lw(v0_2 + 60);
    sw(sp + 512, v1_1 + 928);
    sw(v1_1 + 8, 0);
    let a0_3 = lw(v1_1 + 60);
    sw(a0_3 + 8, 0);
    let fp = a0_3 + 928;
    let a4_1 = lw(sp + 500);
    swc1(a4_1 + 16, f0_1);
    sw(a4_1 + 12, 0);
    sw(a4_1 + 52, 0);
    sw(a4_1 + 4, -1);
    sw(a4_1 + 44, 0);
    sw(a4_1 + 24, 0);
    sw(a4_1 + 20, a0);
    sw(v0_2 + 12, 0);
    sw(v0_2 + 52, 0);
    sw(v0_2 + 16, 0);
    sw(v0_2 + 24, 0);
    sw(v0_2 + 4, -1);
    sw(v0_2 + 44, 0);
    sw(v0_2 + 20, 0);
    sw(v1_1 + 12, 0);
    sw(v1_1 + 16, 0);
    sw(v1_1 + 52, 0);
    sw(v1_1 + 24, 0);
    sw(v1_1 + 20, 0);
    sw(v1_1 + 4, -1);
    sw(v1_1 + 44, 0);
    sw(a0_3 + 4, -1);
    sw(a0_3 + 12, 0);
    sw(a0_3 + 20, 0);
    sw(a0_3 + 16, 0);
    sw(a0_3 + 52, 0);
    sw(a0_3 + 44, 0);
    sw(fp + 0x1180, a2);
    sw(fp + 0x1184, a3);
    if a3 != 1 {
        sw(fp + 0x118c, 0);
    } else {
        sw(fp + 0x118c, 2);
    }
    let f11 = lwc1(s3);
    let f10 = lwc1(s3 + 4);
    let f9 = lwc1(s3 + 8);
    let v0_3 = lw(sp + 484);
    let v1_2 = lw(sp + 488);
    let a0_4 = lw(sp + 492);
    let f6 = lwc1(v0_3);
    let f7 = lwc1(v0_3 + 4);
    let f8 = lwc1(v0_3 + 8);
    let f0_2 = lwc1(a0_4) - f6;
    let f4_1 = lwc1(v1_2 + 4) - f7;
    let f1_1 = lwc1(a0_4 + 8);
    let f5_1 = lwc1(v1_2 + 8) - f8;
    let f3_1 = lwc1(a0_4 + 4) - f7;
    swc1(sp + 96, lwc1(v1_2) - f6);
    swc1(sp + 100, f4_1);
    swc1(sp + 112, f0_2);
    swc1(sp + 116, f3_1);
    swc1(sp + 120, f1_1 - f8);
    swc1(sp + 104, f5_1);
    let f0_3 = 1.0_f32 / (f11 * f11 + f10 * f10 + f9 * f9).abs().sqrt();
    let a2_1 = lw(sp + 480) & 256;
    sw(sp + 516, a2_1);
    let f9_2 = f9 * f0_3;
    let f7_2 = f11 * f0_3;
    let f8_2 = f10 * f0_3;
    swc1(sp + 136, f9_2);
    swc1(sp + 128, f7_2);
    swc1(sp + 132, f8_2);
    if a2_1 != 0 {
        let f2_1 = lwc1(s4);
        let f6_2 = lwc1(s4 + 4);
        let f1_2 = lwc1(s4 + 8);
        let a3_1 = lw(sp + 500);
        size = size * 0.5_f32;
        sw(a3_1 + 8, lw(a3_1 + 8) | 256);
        let f0_4 = f7_2 * f2_1 + f8_2 * f6_2 + f9_2 * f1_2;
        let f0_5 = f0_4 + f0_4;
        swc1(sp + 136, f9_2 - f0_5 * f1_2);
        swc1(sp + 128, f7_2 - f0_5 * f2_1);
        swc1(sp + 132, f8_2 - f0_5 * f6_2);
    }
    sw(fp + 0x1188, 80);
    jitter(lwc1(s6), 0x3ad4dc, sp + 144);
    sw(sp + 148, 0);
    swc1(sp + 176, 1.0_f32);
    swc1(sp + 160, 1.0_f32);
    sw(sp + 164, 0);
    jitter(lwc1(s6 + 4), 0x3ad4e0, sp + 180);
    swc1(sp + 196, 1.0_f32);
    swc1(sp + 192, 1.0_f32);
    jitter(lwc1(s6), 0x3ad4e4, sp + 208);
    sw(sp + 224, 0);
    swc1(sp + 228, 1.0_f32);
    swc1(sp + 212, 1.0_f32);
    sw(sp + 240, 0);
    let f4_5 = lwc1(s6 + 4);
    jitter(f4_5, 0x3ad4e8, sp + 244);
    sw(sp + 256, 0);
    sw(sp + 260, 0);
    let f3_2 = lwc1(s6);
    sw(sp + 572, lw(sp + 500) + 72);
    sw(sp + 560, fp + 4);
    let mut a4_2 = 1;
    sw(sp + 564, fp + 8);
    sw(sp + 568, fp + 12);
    let mut a3_2 = sp + 156;
    let mut a2_2 = sp + 152;
    loop {
        let a0_5 = a4_2 + 1;
        let f1_15 = lwc1(a2_2 - 8);
        let v0_8 = if -1 < a0_5 { a0_5 } else { a4_2 + 16 };
        a4_2 += 2;
        let v1_3 = (a0_5 - ((v0_8 >> 4) << 4)) << 3;
        swc1(a2_2, f1_15 + (lwc1(sp + 144 + v1_3) - f1_15) * 0.5_f32);
        a2_2 += 16;
        let f1_16 = lwc1(a3_2 - 8);
        swc1(a3_2, f1_16 + (lwc1(sp + 148 + v1_3) - f1_16) * 0.5_f32);
        a3_2 += 16;
        if a4_2 >= 16 {
            break;
        }
    }
    let mut a1_1 = sp + 148;
    let mut a0_6 = sp + 144;
    let mut v1_4 = sp + 276;
    let mut v0_9 = sp + 272;
    let mut s7_1 = 15;
    loop {
        let f0_6 = lwc1(a0_6);
        s7_1 -= 1;
        a0_6 += 8;
        swc1(v0_9, f0_6 - f3_2);
        v0_9 += 8;
        let f0_7 = lwc1(a1_1);
        a1_1 += 8;
        swc1(v1_4, f0_7 - f4_5);
        v1_4 += 8;
        if s7_1 < 0 {
            break;
        }
    }
    sw(sp + 520, sp + 276);
    let mut s7_2 = 0;
    let f31 = lwc1(0x3ad4ec);
    sw(sp + 524, sp + 272);
    sw(sp + 556, sp + 96);
    sw(sp + 528, sp + 112);
    sw(sp + 532, sp + 416);
    sw(sp + 536, sp + 432);
    sw(sp + 540, sp + 448);
    sw(sp + 544, sp + 128);
    sw(sp + 548, sp + 400);
    sw(sp + 552, sp + 464);
    let mut s5_1 = s7_2 + 1;
    loop {
        let s2 = s7_2 << 3;
        let f12_1 = lwc1(lw(sp + 520) + s2);
        let s1 = (s5_1 - (((if -1 < s5_1 { s5_1 } else { s7_2 + 16 }) >> 4) << 4)) << 3;
        let v0_10 = fptodp(f12_1);
        let f12_2 = lwc1(lw(sp + 524) + s2);
        let v0_11 = fptodp(f12_2);
        let f0_8 = dptofp(atan2(v0_10, v0_11));
        let f12_3 = lwc1(lw(sp + 520) + s1);
        let v0_13 = fptodp(f12_3);
        let f12_4 = lwc1(lw(sp + 524) + s1);
        let v0_14 = fptodp(f12_4);
        let f0_9 = dptofp(atan2(v0_13, v0_14));
        let f1_17 = lwc1(0x3ad4f0);
        let a1_4 = (f0_9 * f1_17 + 0.5_f32) as i32;
        let v1_5 = (f0_8 * f1_17 + 0.5_f32) as i32;
        let mut f20 = lwc1(((v1_5 + 512 & 2047) << 2) + g::g_SinTable);
        let mut f21 = lwc1(((v1_5 & 2047) << 2) + g::g_SinTable);
        let mut f22 = lwc1(((a1_4 + 512 & 2047) << 2) + g::g_SinTable);
        let mut f23 = lwc1(((a1_4 & 2047) << 2) + g::g_SinTable);
        let f28 = random_as_float() * 2.3283064e-10_f32 * f31 + lwc1(0x3ad4f4);
        let f27 = random_as_float() * 2.3283064e-10_f32 * f31 + lwc1(0x3ad4f8);
        let f25 = random_as_float() * 2.3283064e-10_f32 * f31 + lwc1(0x3ad4fc);
        let f29 = f25 + random_as_float() * 2.3283064e-10_f32 * f31;
        let f24 = random_as_float() * 2.3283064e-10_f32 * f31 + 0.5_f32;
        let f0_20 = random_as_float();
        let v0_22 = lw(sp + 524);
        let s3_2 = v0_22 + s2;
        let s4_2 = v0_22 + s1;
        let f3_3 = lwc1(s3_2);
        let v1_6 = lw(sp + 520);
        let s2_2 = v1_6 + s2;
        let s1_2 = v1_6 + s1;
        let f3_4 = f3_3 + (lwc1(s4_2) - f3_3) * 0.5_f32;
        let f26 = f0_20 * 2.3283064e-10_f32 * f31 + lwc1(0x3ad500);
        swc1(sp + 400, f3_4);
        let f1_18 = lwc1(s2_2);
        let f1_19 = f1_18 + (lwc1(s1_2) - f1_18) * 0.5_f32;
        let f2_2 = (f3_4 * f3_4 + f1_19 * f1_19).abs().sqrt();
        swc1(sp + 404, f1_19);
        let f1_20 = lwc1(sp + 404) / f2_2;
        swc1(sp + 400, lwc1(sp + 400) / f2_2);
        swc1(sp + 404, f1_20);
        if f20 < 0.0_f32 {
            f20 = -f20;
        }
        if f21 < 0.0_f32 {
            f21 = -f21;
        }
        if f22 < 0.0_f32 {
            f22 = -f22;
        }
        if f23 < 0.0_f32 {
            f23 = -f23;
        }
        // the first quad
        let f2_3 = lwc1(s6);
        let f3_5 = lwc1(s6 + 4);
        let s0_1 = (s7_2 << 2) + s7_2;
        swc1(sp + 416, f2_3);
        swc1(sp + 420, f3_5);
        s7_2 = s5_1;
        let f0_22 = lwc1(s3_2);
        swc1(sp + 432, f2_3 + f0_22 * f28 * f20);
        swc1(sp + 436, f3_5 + lwc1(s2_2) * f28 * f21);
        swc1(sp + 448, f2_3 + lwc1(s4_2) * f27 * f22);
        swc1(sp + 452, f3_5 + lwc1(s1_2) * f27 * f23);
        set_quad(sp, s6, fp, s0_1, size, 0, true);
        // the second quad
        let f0_23 = lwc1(s3_2) * f25;
        let f2_4 = lwc1(s6);
        let f3_6 = lwc1(s6 + 4);
        swc1(sp + 416, f2_4 + f0_23 * f20);
        swc1(sp + 420, f3_6 + lwc1(s2_2) * f25 * f21);
        swc1(sp + 464, f2_4 + lwc1(s4_2) * f29 * f22);
        swc1(sp + 468, f3_6 + lwc1(s1_2) * f29 * f23);
        set_quad(sp, s6, fp, s0_1 + 1, size, 0, false);
        // the third quad
        let f0_24 = lwc1(s3_2) * f24;
        let f2_5 = lwc1(s6);
        let f3_7 = lwc1(s6 + 4);
        swc1(sp + 432, f2_5 + f0_24 * f20);
        swc1(sp + 436, f3_7 + lwc1(s2_2) * f24 * f21);
        swc1(sp + 448, f2_5 + lwc1(s4_2) * f24 * f22);
        swc1(sp + 452, f3_7 + lwc1(s1_2) * f24 * f23);
        set_quad(sp, s6, fp, s0_1 + 2, size, 0, false);
        // the fourth quad
        let f0_25 = lwc1(s3_2) * f26;
        let f2_6 = lwc1(s6);
        let f3_8 = lwc1(s6 + 4);
        swc1(sp + 416, f2_6 + f0_25 * f20);
        swc1(sp + 420, f3_8 + lwc1(s2_2) * f26 * f21);
        swc1(sp + 464, f2_6 + lwc1(s4_2) * f26 * f22);
        swc1(sp + 468, f3_8 + lwc1(s1_2) * f26 * f23);
        set_quad(sp, s6, fp, s0_1 + 3, size, 0, false);
        // the last quad, the cap
        let f0_26 = lwc1(s3_2);
        let f1_24 = lwc1(s6);
        let f2_7 = lwc1(s6 + 4);
        swc1(sp + 432, f1_24 + f0_26);
        swc1(sp + 436, f2_7 + lwc1(s2_2));
        swc1(sp + 448, f1_24 + lwc1(s4_2));
        swc1(sp + 452, f2_7 + lwc1(s1_2));
        set_quad(sp, s6, fp, s0_1 + 4, size, 1, false);
        if s7_2 >= 16 {
            break;
        }
        s5_1 = s7_2 + 1;
    }
    let mut v1_13 = 127;
    let a1_10;
    let v0_43;
    if lw(sp + 516) == 0 {
        let a2_9 = lw(sp + 848);
        a1_10 = (lwc1(a2_9) * 27.0_f32 + 1e+02_f32) as i32;
        v0_43 = (lwc1(a2_9 + 8) * 27.0_f32 + 1e+02_f32) as i32;
        v1_13 = (lwc1(a2_9 + 4) * 27.0_f32 + 1e+02_f32) as i32;
    } else {
        a1_10 = 127;
        v0_43 = 127;
    }
    let f17_1 = lwc1(lw(sp + 496) + 8);
    let a2_10 = lw(sp + 484);
    let v0_44 = v0_43 << 16 | a1_10 | (v1_13 << 8 | 0x7f000000);
    let f7_3 = lwc1(lw(sp + 484) + 4);
    let a4_11 = lw(sp + 492);
    let f9_3 = lwc1(a2_10 + 8);
    let f4_6 = lwc1(a2_10);
    let f1_25 = lwc1(a4_11);
    let f6_3 = lwc1(a4_11 + 4);
    let a4_12 = lw(sp + 496);
    let f8_3 = lwc1(lw(sp + 488) + 8);
    let f3_9 = lwc1(lw(sp + 488));
    let f5_2 = lwc1(lw(sp + 492) + 8);
    let f15_1 = lwc1(lw(sp + 488) + 4);
    let mut a3_11 = fp;
    let mut a1_11 = lw(sp + 568);
    let mut a2_11 = 79;
    let mut a0_16 = lw(sp + 564);
    let mut v1_14 = lw(sp + 560);
    let f13_1 = lwc1(a4_12);
    let f2_8 = lwc1(a4_12 + 4);
    loop {
        sw(a3_11, v0_44);
        a2_11 -= 1;
        sw(v1_14, v0_44);
        a3_11 += 16;
        sw(a0_16, v0_44);
        v1_14 += 16;
        sw(a1_11, v0_44);
        a0_16 += 16;
        a1_11 += 16;
        if a2_11 < 0 {
            break;
        }
    }
    let f12_10 = lowest_minus(f13_1, f1_25, f3_9, f4_6, 3.0_f32);
    let f13_2 = highest_plus(f13_1, f1_25, f3_9, f4_6, 3.0_f32);
    let f14_1 = lowest_minus(f2_8, f6_3, f15_1, f7_3, 6.0_f32);
    let f15_2 = highest_plus(f2_8, f6_3, f15_1, f7_3, 3.0_f32);
    let f16_1 = lowest_minus(f17_1, f5_2, f8_3, f9_3, 3.0_f32);
    let f17_2 = highest_plus(f17_1, f5_2, f8_3, f9_3, 3.0_f32);
    bgCalcRoomArray(lw(sp + 572), f12_10, f13_2, f14_1, f15_2, f16_1, f17_2);
}
