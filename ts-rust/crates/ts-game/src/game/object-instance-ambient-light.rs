//! The ambient light that falls on an object instance.

use crate::*;

/// One colour channel of a packed colour (`shift` 0, 8 or 16) as a float from 0 to 255.
fn channel(packed: i32, shift: u32) -> f32 {
    (((packed as u32) >> shift) & 255) as f32
}

/// Work out the ambient light colour (three floats) of the object instance `a0` into `a1` (@ 0x25aae0): a light cached in the object at +540 is used on the frames the object
/// is not due for a new one (every fourth frame by its place in the table), or the colour of the object it is attached to (+188) is copied, or a ray is cast from the object
/// (a little above its position) down through the faces of the rooms it is in (moving to the room beyond a portal when it hits nothing) and the colour at the nearest hit is
/// blended from the colours of the corners of the face; no hit gives a dim grey. The colour is kept at +540 as bytes. The light of the bullets of a player's gun that is flashing
/// is added (and clamped at one), and the object's own glow at +312 is mixed in last.
pub fn obinstCalcAmbientLight(a0: i32, a1: i32) {
    let frame = Frame::new(656);
    let sp = frame.sp();
    let mut fcc;
    let v1_1 = lw(g::obinstances);
    sw(sp + 288, a0);
    let v0_1 = lw(a0 + 244);
    sw(sp + 292, a1);
    sw(sp + 296, v0_1);
    let a0_1 = lw(g::updateframesi);
    let a2_1 = lw(sp + 296);
    let mut a4 = lw(sp + 296) + 112;
    sw(sp + 300, lw(lw(sp + 296) + 12));
    let mut a1_1 = lw(g::gameframe);
    sw(sp + 304, a2_1 + 48);
    if a0_1 >= 2 {
        let lo_1 = a1_1.wrapping_div(a0_1);
        if a0_1 == 0 {
            trap("break");
        }
        a1_1 = lo_1;
    }
    let mut a3_1 = 0;
    let mut a6_1 = 0;
    let v1_2 = lw(lw(sp + 296) + 8);
    sw(sp + 308, 0);
    sw(sp + 312, 0);
    if v1_2 != 2048 {
        let v1_3 = lw(sp + 300);
        if v1_2 != 8 {
            let a0_3 = lw(sp + 300);
            if ((v1_3 - 201) as u32) < 4 {
                let a0_4 = lw(g::players);
                sw(sp + 308, 1);
                let a0_5 = a0_4 + (a0_3 * 1820 - 0x594fc);
                sw(sp + 312, a0_5);
                a4 = a0_5 + 344;
                let a7_1 = lw(a0_5 + 796);
                sw(sp + 304, a0_5 + 152);
                sw(sp + 300, a7_1);
            }
        } else {
            let v0_3 = lw(lw(sp + 296) + 356);
            if v0_3 == 0 {
                sw(sp + 304, lw(lw(sp + 296) + 352) + 2808);
            } else {
                sw(sp + 304, v0_3 + 152);
            }
        }
    } else {
        let a0_2 = lw(sp + 296);
        let v0_2 = lw(lw(sp + 288) + 4);
        sw(sp + 304, sp);
        swc1(sp, lwc1(a0_2 + 48) + lwc1(v0_2 + 48));
        swc1(sp + 4, lwc1(a0_2 + 52) + lwc1(v0_2 + 52));
        swc1(sp + 8, lwc1(a0_2 + 56) + lwc1(v0_2 + 56));
    }
    let mut a0_6 = lw(lw(sp + 296) + 540);
    let mut v1_6;
    'b4: {
        'b5: {
            if a0_6 != -1 {
                if (((a0 - v1_1).wrapping_mul(-0x3e7063e7)) >> 3 & 3) != a1_1 & 3 {
                    swc1(lw(sp + 292), channel(a0_6, 0) * lwc1(0x3acad0));
                    swc1(lw(sp + 292) + 4, channel(a0_6, 8) * lwc1(0x3acad4));
                    a3_1 = 1;
                    swc1(lw(sp + 292) + 8, channel(a0_6, 16) * lwc1(0x3acad8));
                    break 'b5;
                }
            }
            let a7_2 = lw(sp + 296);
            let v0_7 = lw(a7_2 + 188);
            v1_6 = lw(g::roomlightinitial);
            if v0_7 == 0 {
                break 'b4;
            }
            let v0_8 = lw(lw(v0_7 + 244) + 540);
            sw(a7_2 + 540, v0_8);
            let f1_7 = channel(v0_8, 0);
            let v0_9 = lw(sp + 296);
            a0_6 = lw(v0_9 + 540);
            swc1(lw(sp + 292), f1_7 * lwc1(0x3acadc));
            swc1(lw(sp + 292) + 4, channel(a0_6, 8) * lwc1(0x3acae0));
            a3_1 = 1;
            swc1(lw(sp + 292) + 8, channel(a0_6, 16) * lwc1(0x3acae4));
        }
        v1_6 = lw(g::roomlightinitial);
    }
    let a1_3 = lw(sp + 300);
    if v1_6 != 0 {
        a6_1 = (((lw(a1_3 * 24 + v1_6 - 24) - 1) as u32) < 3) as i32;
    }
    'b14: {
        'b15: {
            if a3_1 == 0 {
                let a0_7;
                if a0_6 == -1 {
                    a0_7 = sp + 16;
                } else if a6_1 != 0 {
                    a0_7 = sp + 16;
                } else {
                    let a2_3 = lw(sp + 304);
                    if !(lwc1(a2_3) == lwc1(a4)) {
                        a0_7 = sp + 16;
                    } else {
                        if lwc1(a2_3 + 8) == lwc1(a4 + 8) {
                            break 'b15;
                        }
                        a0_7 = sp + 16;
                    }
                }
                memset(a0_7, 0, 12);
                sw(sp + 332, sp + 128);
                let a7_3 = lw(0x3a2688);
                sd(sp + 48, ld(0x3a2680));
                sw(sp + 56, a7_3);
                let f2_1 = lwc1(0x3acae8);
                let a1_5 = lw(sp + 304);
                let mut f21 = lwc1(0x3aea90);
                swc1(sp + 32, lwc1(a1_5));
                swc1(sp + 36, lwc1(a1_5 + 4) + f2_1);
                swc1(sp + 40, lwc1(a1_5 + 8));
                let mut a5_1 = lw(sp + 300);
                let mut rooms_tried = 0;
                loop {
                    // an object with an invalid position (NaN) would pass from room to room for ever (the console's FPU has no NaN)
                    rooms_tried += 1;
                    if rooms_tried > 64 {
                        break;
                    }
                    let v0_13 = lw(g::bgNumRooms);
                    if a5_1 <= 0 {
                        break;
                    }
                    let a6_2 = a5_1 << 2;
                    if v0_13 < a5_1 {
                        break;
                    }
                    'b19: {
                        let a0_8 = lw(g::roomprops);
                        sw(sp + 320, a6_2);
                        let mut a2_5 = 0_u128;
                        let v1_10 = lw(lw(lw(a6_2 + a0_8) + 32));
                        sw(sp + 316, v1_10);
                        let mut a4_2 = lw(v1_10);
                        sw(sp + 324, v1_10 - a4_2 * 80);
                        if a4_2 > 0 {
                            let mut v0_14 = lw(sp + 324);
                            loop {
                                'b21: {
                                    let mut a7_4;
                                    'b22: {
                                        let lo_2 = v0_14;
                                        let lo_3 = lo_2 + a2_5 as i32 * 80;
                                        let mut s3_1 = lw(lo_2 + a2_5 as i32 * 80 + 76);
                                        a7_4 = (a2_5 as i32 + 1) as u128;
                                        if s3_1 != 0 {
                                            let a0_9 = lw(g::roomprops);
                                            let mut s0_1;
                                            let mut t5_1;
                                            let mut t4_1;
                                            if lw(s3_1 + 24) == 0 {
                                                s0_1 = lw(lo_3 + 8);
                                                t5_1 = (lo_3 + 20) as u128;
                                                t4_1 = lw(a2_5 as i32 * 48 + lw(lw(lw(lw(sp + 320) + a0_9) + 32) + 248) + 12) as u128;
                                            } else {
                                                s0_1 = lw(lo_3 + 12);
                                                t5_1 = (lo_3 + 44) as u128;
                                                t4_1 = lw(a2_5 as i32 * 48 + lw(lw(lw(lw(sp + 320) + a0_9) + 32) + 248) + 36) as u128;
                                            }
                                            if (t4_1 as i64) == 0_i64 {
                                                t4_1 = lw(t5_1 as i32 + 12) as u128;
                                            }
                                            let v1_11 = lhu(lo_3 + 68);
                                            let a0_10 = lw(sp + 288);
                                            if v1_11 != 0 {
                                                a7_4 = (a2_5 as i32 + 1) as u128;
                                                if (lhu(a0_10 + 292) & v1_11) == 0 {
                                                    break 'b22;
                                                }
                                            }
                                            a7_4 = (a2_5 as i32 + 1) as u128;
                                            if s0_1 != 0 {
                                                if s3_1 == 0 {
                                                    a2_5 = a7_4;
                                                    break 'b21;
                                                }
                                                if lw(s0_1 + 20) >= 0 {
                                                    sw(sp + 340, sp + 32);
                                                    sw(sp + 344, sp + 48);
                                                    loop {
                                                        let v1_12 = lw(s0_1 + 12);
                                                        let v0_15 = lw(s0_1 + 4);
                                                        let a3_2 = s3_1 + 12;
                                                        sw(sp + 328, 0);
                                                        let mut s2_1 = t4_1 as i32 + (v1_12 << 2);
                                                        let a4_3 = 0;
                                                        let mut s7_1 = lw(t5_1 as i32) + (v0_15 << 4);
                                                        let a5_2 = 0;
                                                        let mut s1_1 = lw(t5_1 as i32 + 4) + v1_12 * 12;
                                                        let a0_11 = lw(sp + 340);
                                                        let a1_6 = lw(sp + 344);
                                                        let a2_6 = s3_1;
                                                        sq(sp + 352, a7_4);
                                                        sq(sp + 368, t4_1);
                                                        sq(sp + 384, t5_1);
                                                        let v0_16 = HitTestLineBox(a0_11, a1_6, a2_6, a3_2, a4_3, a5_2);
                                                        a7_4 = lq(sp + 352);
                                                        t4_1 = lq(sp + 368);
                                                        t5_1 = lq(sp + 384);
                                                        if v0_16 == 0 {
                                                            sw(sp + 348, s0_1 + 24);
                                                            sw(sp + 336, s3_1 + 36);
                                                            s0_1 = lw(sp + 348);
                                                        } else {
                                                            let v0_17 = lw(s7_1);
                                                            sw(sp + 348, s0_1 + 24);
                                                            sw(sp + 336, s3_1 + 36);
                                                            let mut t9_1 = (v0_17 & 0x7fff) as u128;
                                                            let mut t6_1 = (v0_17 & 0x8000) as u128;
                                                            let mut s0_2 = (v0_17 >> 16 ^ 1) & 1;
                                                            let mut fp_1 = t9_1 as i32 - 2;
                                                            loop {
                                                                'b31: {
                                                                    let mut s6_1 = 0;
                                                                    if fp_1 > 0 {
                                                                        let mut t7_1 = (sp + 128) as u128;
                                                                        loop {
                                                                            let s5_1;
                                                                            let a2_8;
                                                                            'b33: {
                                                                                let s4_1 = s1_1 + 24;
                                                                                s5_1 = s1_1 + 12;
                                                                                let a0_12 = lw(sp + 340);
                                                                                let a1_7 = lw(sp + 344);
                                                                                let s3_2 = if s0_2 == 0 { s5_1 } else { s4_1 };
                                                                                let a3_3 = if s0_2 != 0 { s5_1 } else { s4_1 };
                                                                                let a2_7 = s1_1;
                                                                                let a4_4 = s3_2;
                                                                                let a5_3 = t7_1;
                                                                                let a6_3 = 0;
                                                                                sq(sp + 352, a7_4);
                                                                                sq(sp + 368, t4_1);
                                                                                sq(sp + 384, t5_1);
                                                                                sq(sp + 400, t6_1);
                                                                                sq(sp + 416, t7_1);
                                                                                sq(sp + 432, t9_1);
                                                                                let v0_18 = hittestLineTri(a0_12, a1_7, a2_7, a3_3, a4_4, a5_3 as i32, a6_3);
                                                                                a7_4 = lq(sp + 352);
                                                                                t4_1 = lq(sp + 368);
                                                                                t5_1 = lq(sp + 384);
                                                                                t6_1 = lq(sp + 400);
                                                                                t7_1 = lq(sp + 416);
                                                                                t9_1 = lq(sp + 432);
                                                                                if v0_18 != 0 {
                                                                                    let f6_1 = lwc1(sp + 128);
                                                                                    let f7_1 = lwc1(sp + 132);
                                                                                    let f1_13 = f6_1 - lwc1(sp + 32);
                                                                                    let f5_1 = lwc1(sp + 136);
                                                                                    let f2_2 = f7_1 - lwc1(sp + 36);
                                                                                    let f0_1 = lwc1(sp + 40);
                                                                                    swc1(sp + 144, f1_13);
                                                                                    let f0_2 = f5_1 - f0_1;
                                                                                    swc1(sp + 148, f2_2);
                                                                                    let f3_1 = f1_13 * f1_13 + f2_2 * f2_2 + f0_2 * f0_2;
                                                                                    swc1(sp + 152, f0_2);
                                                                                    if f3_1 < f21 {
                                                                                        swc1(sp + 16, f6_1);
                                                                                        a2_8 = s2_1 + 4;
                                                                                        swc1(sp + 20, f7_1);
                                                                                        swc1(sp + 24, f5_1);
                                                                                        let f0_3 = lwc1(s1_1);
                                                                                        let a1_8 = if s0_2 == 0 { s4_1 } else { s5_1 };
                                                                                        sw(sp + 112, lw(s2_1));
                                                                                        f21 = f3_1;
                                                                                        swc1(sp + 64, f0_3);
                                                                                        let f0_4 = lwc1(s1_1 + 4);
                                                                                        sw(sp + 116, lw(if s0_2 != 0 { a2_8 } else { s2_1 + 8 }));
                                                                                        swc1(sp + 68, f0_4);
                                                                                        let v0_19 = lw(if s0_2 == 0 { a2_8 } else { s2_1 + 8 });
                                                                                        swc1(sp + 72, lwc1(s1_1 + 8));
                                                                                        sw(sp + 120, v0_19);
                                                                                        swc1(sp + 76, lwc1(a1_8));
                                                                                        swc1(sp + 80, lwc1(a1_8 + 4));
                                                                                        swc1(sp + 84, lwc1(a1_8 + 8));
                                                                                        swc1(sp + 88, lwc1(s3_2));
                                                                                        swc1(sp + 92, lwc1(s3_2 + 4));
                                                                                        swc1(sp + 96, lwc1(s3_2 + 8));
                                                                                        break 'b33;
                                                                                    }
                                                                                }
                                                                                a2_8 = s2_1 + 4;
                                                                            }
                                                                            s6_1 += 1;
                                                                            s1_1 = s5_1;
                                                                            s2_1 = a2_8;
                                                                            s0_2 = 1 - s0_2;
                                                                            if s6_1 >= fp_1 {
                                                                                break 'b31;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                s1_1 += 24;
                                                                s2_1 += 8;
                                                                if (t6_1 as i64) != 0_i64 {
                                                                    sw(sp + 328, 1);
                                                                } else {
                                                                    s7_1 += 16;
                                                                    let v0_20 = lw(s7_1);
                                                                    t9_1 = (v0_20 & 0x7fff) as u128;
                                                                    t6_1 = (v0_20 & 0x8000) as u128;
                                                                    s0_2 = (v0_20 >> 16 ^ 1) & 1;
                                                                }
                                                                if lw(sp + 328) != 0 {
                                                                    break;
                                                                }
                                                                fp_1 = t9_1 as i32 - 2;
                                                            }
                                                            s0_1 = lw(sp + 348);
                                                        }
                                                        s3_1 = lw(sp + 336);
                                                        if lw(s0_1 + 20) < 0 {
                                                            break;
                                                        }
                                                    }
                                                    a4_2 = lw(lw(sp + 316));
                                                }
                                            }
                                        }
                                    }
                                    a2_5 = a7_4;
                                }
                                v0_14 = lw(sp + 324);
                                if (a2_5 as i64) >= a4_2 as i64 {
                                    break 'b19;
                                }
                            }
                        }
                    }
                    let f20 = lwc1(0x3aea90);
                    fcc = f21 == f20;
                    if fcc {
                        let a0_13 = lw(sp + 304);
                        let a1_9 = lw(sp + 332);
                        let a3_4 = 0;
                        let a2_9 = lw(sp + 300);
                        let a6_4 = lw(0x3a2698);
                        sd(sp + 128, ld(0x3a2690));
                        sw(sp + 136, a6_4);
                        let a4_5 = 0;
                        let v0_21 = bgPortalCrossed(a0_13, a1_9, a2_9, a3_4, a4_5);
                        if lw(sp + 300) == v0_21 {
                            break;
                        }
                        sw(sp + 300, v0_21);
                        fcc = f21 == f20;
                    }
                    a5_1 = lw(sp + 300);
                    if !fcc {
                        break;
                    }
                }
                let v1_13 = lw(sp + 296);
                if f21 == lwc1(0x3aea90) {
                    let f0_10 = lwc1(0x3acb10);
                    sw(v1_13 + 540, 0x4c4c4c);
                    let a0_17 = lw(sp + 292);
                    swc1(a0_17 + 8, f0_10);
                    swc1(a0_17, f0_10);
                    swc1(a0_17 + 4, f0_10);
                    break 'b14;
                }
                let a0_14 = lw(sp + 112);
                swc1(sp + 208, channel(a0_14, 0) * lwc1(0x3acaec));
                swc1(sp + 212, channel(a0_14, 8) * lwc1(0x3acaf0));
                let a0_15 = lw(sp + 116);
                swc1(sp + 216, channel(a0_14, 16) * lwc1(0x3acaf4));
                swc1(sp + 220, channel(a0_15, 0) * lwc1(0x3acaf8));
                swc1(sp + 224, channel(a0_15, 8) * lwc1(0x3acafc));
                let a0_16 = lw(sp + 120);
                swc1(sp + 228, channel(a0_15, 16) * lwc1(0x3acb00));
                swc1(sp + 232, channel(a0_16, 0) * lwc1(0x3acb04));
                swc1(sp + 236, channel(a0_16, 8) * lwc1(0x3acb08));
                let f6_2 = channel(a0_16, 16);
                let f0_5 = lwc1(sp + 76);
                let f15_1 = lwc1(sp + 64);
                let f19_1 = lwc1(sp + 68);
                let f0_6 = f0_5 - f15_1;
                let f2_3 = lwc1(sp + 84);
                let f1_30 = lwc1(sp + 80) - f19_1;
                let f20_2 = lwc1(sp + 72);
                swc1(sp + 128, f0_6);
                let f2_4 = f2_3 - f20_2;
                let f3_2 = lwc1(0x3acb0c);
                swc1(sp + 132, f1_30);
                swc1(sp + 136, f2_4);
                swc1(sp + 240, f6_2 * f3_2);
                let f0_7 = 1.0_f32 / (f0_6 * f0_6 + f1_30 * f1_30 + f2_4 * f2_4).abs().sqrt();
                let f23_1 = lwc1(sp + 16);
                let f24_1 = lwc1(sp + 20);
                let f9_1 = f23_1 - f15_1;
                let f11_1 = f24_1 - f19_1;
                let f22_1 = lwc1(sp + 24);
                let f17_1 = lwc1(sp + 88);
                let f6_4 = lwc1(sp + 128) * f0_7;
                let f16_1 = lwc1(sp + 92);
                let f5_2 = lwc1(sp + 132) * f0_7;
                let f21_2 = lwc1(sp + 96);
                let f4_1 = lwc1(sp + 136) * f0_7;
                swc1(sp + 128, f6_4);
                swc1(sp + 132, f5_2);
                let f10_1 = f22_1 - f20_2;
                swc1(sp + 136, f4_1);
                let f1_31 = f17_1 - f15_1;
                let f2_5 = f16_1 - f19_1;
                let f3_3 = f21_2 - f20_2;
                let f14_1 = f9_1 * f6_4 + f11_1 * f5_2 + f10_1 * f4_1;
                let f14_2 = f1_31 * f6_4 + f2_5 * f5_2 + f3_3 * f4_1;
                let f9_2 = f9_1 - f14_1 * f6_4;
                let f11_2 = f11_1 - f14_1 * f5_2;
                let f1_32 = f1_31 - f14_2 * f6_4;
                let f2_6 = f2_5 - f14_2 * f5_2;
                let f3_4 = f3_3 - f14_2 * f4_1;
                swc1(sp + 160, f1_32);
                let f10_2 = f10_1 - f14_1 * f4_1;
                swc1(sp + 164, f2_6);
                swc1(sp + 168, f3_4);
                let f3_5 = lwc1(sp + 76);
                let f4_2 = lwc1(sp + 80);
                let f5_3 = lwc1(sp + 84);
                let f11_3 = lwc1(sp + 232);
                let f10_3 = lwc1(sp + 236);
                let f9_3 = lwc1(sp + 240);
                let f13_1 = lwc1(sp + 224);
                let f12_1 = lwc1(sp + 228);
                let f17_2 = lwc1(sp + 208);
                let ratio = ((f9_2 * f9_2 + f11_2 * f11_2 + f10_2 * f10_2) / (f1_32 * f1_32 + f2_6 * f2_6 + f3_4 * f3_4)).abs().sqrt();
                let f3_6 = f3_5 + ratio * (f17_1 - f3_5);
                let f16_2 = lwc1(sp + 212);
                let f4_3 = f4_2 + ratio * (f16_1 - f4_2);
                let f14_3 = lwc1(sp + 216);
                let f1_33 = f15_1 + ratio * (f17_1 - f15_1);
                swc1(sp + 268, f3_6);
                let f2_7 = f19_1 + ratio * (f16_1 - f19_1);
                swc1(sp + 272, f4_3);
                let f15_2 = lwc1(sp + 220);
                let f5_4 = f5_3 + ratio * (f21_2 - f5_3);
                swc1(sp + 256, f1_33);
                let f3_7 = f3_6 - f1_33;
                swc1(sp + 260, f2_7);
                let f0_8 = f20_2 + ratio * (f21_2 - f20_2);
                let f4_4 = f4_3 - f2_7;
                swc1(sp + 276, f5_4);
                let f1_34 = f23_1 - f1_33;
                swc1(sp + 160, f3_7);
                let f2_8 = f24_1 - f2_7;
                swc1(sp + 264, f0_8);
                let f5_5 = f5_4 - f0_8;
                swc1(sp + 164, f4_4);
                let f0_9 = f22_1 - f0_8;
                swc1(sp + 168, f5_5);
                swc1(sp + 176, f17_2 + ratio * (f11_3 - f17_2));
                swc1(sp + 180, f16_2 + ratio * (f10_3 - f16_2));
                swc1(sp + 184, f14_3 + ratio * (f9_3 - f14_3));
                swc1(sp + 188, f15_2 + ratio * (f11_3 - f15_2));
                swc1(sp + 192, f13_1 + ratio * (f10_3 - f13_1));
                swc1(sp + 196, f12_1 + ratio * (f9_3 - f12_1));
                let f5_6 = lwc1(sp + 180);
                let f4_5 = lwc1(sp + 176);
                let f6_5 = lwc1(sp + 184);
                let a2_10 = lw(sp + 292);
                let ratio2 = ((f1_34 * f1_34 + f2_8 * f2_8 + f0_9 * f0_9) / (f3_7 * f3_7 + f4_4 * f4_4 + f5_5 * f5_5)).abs().sqrt();
                let f5_7 = f5_6 + ratio2 * (lwc1(sp + 192) - f5_6);
                let f1_35 = ratio2 * (lwc1(sp + 196) - f6_5);
                let f4_6 = f4_5 + ratio2 * (lwc1(sp + 188) - f4_5);
                swc1(a2_10 + 4, f5_7);
                let f6_6 = f6_5 + f1_35;
                swc1(a2_10, f4_6);
                swc1(a2_10 + 8, f6_6);
                sw(lw(sp + 296) + 540, (f4_6 * 255.0_f32) as i32 | ((f5_7 * 255.0_f32) as i32) << 8 | ((f6_6 * 255.0_f32) as i32) << 16);
                break 'b14;
            }
        }
        swc1(lw(sp + 292), channel(a0_6, 0) * lwc1(0x3acb14));
        swc1(lw(sp + 292) + 4, channel(a0_6, 8) * lwc1(0x3acb18));
        swc1(lw(sp + 292) + 8, channel(a0_6, 16) * lwc1(0x3acb1c));
    }
    let mut a6_5;
    'b49: {
        let a7_5 = lw(sp + 312);
        if lw(sp + 308) != 0 {
            let v0_33 = lw(a7_5 + 420);
            let mut a0_18 = 0;
            let s1_2 = if v0_33 == 0 { 0 } else { lhu(lw(v0_33 + 32) + 292) & 1 };
            let v0_34 = lw(lw(sp + 312) + 612);
            let mut s0_3 = 0;
            if v0_34 != 0 {
                s0_3 = lhu(lw(v0_34 + 32) + 292) & 1;
            }
            let a1_10 = lw(sp + 312);
            if s1_2 == 0 {
                let a2_11 = lw(sp + 312);
                if s0_3 != 0 {
                    a0_18 = gunGetBulletType(a2_11 + 600);
                }
            } else {
                a0_18 = gunGetBulletType(a1_10 + 408);
            }
            a6_5 = lw(sp + 288);
            if a0_18 == 0 {
                break 'b49;
            }
            'b54: {
                'b55: {
                    bulletGetBulletColour(a0_18, sp + 16);
                    let a5_4 = lw(sp + 312);
                    if s1_2 != 0 {
                        if lw(lw(a5_4 + 420) + 32) == lw(sp + 288) {
                            break 'b55;
                        }
                    }
                    let a7_6 = lw(sp + 312);
                    if s0_3 != 0 {
                        if lw(lw(a7_6 + 612) + 32) == lw(sp + 288) {
                            break 'b55;
                        }
                    }
                    let f1_44 = lwc1(0x3acb24);
                    let a0_21 = lw(sp + 292);
                    let f0_12 = lwc1(a0_21 + 4) + lwc1(sp + 20) * f1_44;
                    let f1_45 = lwc1(a0_21 + 8) + lwc1(sp + 24) * f1_44;
                    swc1(a0_21, lwc1(a0_21) + lwc1(sp + 16) * f1_44);
                    swc1(a0_21 + 4, f0_12);
                    swc1(a0_21 + 8, f1_45);
                    break 'b54;
                }
                let f1_42 = lwc1(0x3acb20);
                let v1_23 = lw(sp + 292);
                let f0_11 = lwc1(v1_23 + 4) + lwc1(sp + 20) * f1_42;
                let f1_43 = lwc1(v1_23 + 8) + lwc1(sp + 24) * f1_42;
                swc1(v1_23, lwc1(v1_23) + lwc1(sp + 16) * f1_42);
                swc1(v1_23 + 4, f0_11);
                swc1(v1_23 + 8, f1_43);
            }
            let a1_12 = lw(sp + 292);
            if 1.0_f32 < lwc1(a1_12) {
                swc1(a1_12, 1.0_f32);
            }
            let a2_12 = lw(sp + 292);
            if 1.0_f32 < lwc1(a2_12 + 4) {
                swc1(a2_12 + 4, 1.0_f32);
            }
            let a5_5 = lw(sp + 292);
            if 1.0_f32 < lwc1(a5_5 + 8) {
                swc1(a5_5 + 8, 1.0_f32);
            }
        }
        a6_5 = lw(sp + 288);
    }
    a6_5 = lw(sp + 288);
    let f2_9 = lwc1(a6_5 + 312);
    let a7_7 = lw(sp + 292);
    if 0.0_f32 < f2_9 {
        let f4_7 = lw(a6_5 + 320) as f32;
        let f0_13 = 1.0_f32 - f2_9;
        let f7_2 = lwc1(0x3acb28);
        let f5_8 = lw(a6_5 + 324) as f32;
        let f2_10 = lwc1(a7_7 + 4) * f0_13;
        let f6_7 = lwc1(a7_7 + 8) * f0_13;
        swc1(a7_7, lwc1(a7_7) * f0_13 + lw(a6_5 + 316) as f32 * f2_9 * f7_2);
        swc1(a7_7 + 4, f2_10 + f4_7 * lwc1(a6_5 + 312) * f7_2);
        swc1(a7_7 + 8, f6_7 + f5_8 * lwc1(a6_5 + 312) * f7_2);
    }
}
