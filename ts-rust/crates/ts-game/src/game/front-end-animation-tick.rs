//! Function: the tick of the character display of the front end (the character select pages and the character animation pages).

use crate::*;

/// Tick of the front end character display (@ 0x1ff3d8): moves the character, its stage and its stage prop together, plays the animations of the list of the character in
/// order, creates and removes the prop and the guns that go with an animation, flashes the guns during some animations, and chooses what to draw.
pub fn animTick(a0: i32) {
    let fal = g::fe_animation_list;
    let front = lw(g::front);
    let chr = lw((lw(front) << 2) + 0x1fc1238);
    let f1_1 = lwc1(chr + 48);
    let idx = crate::manual::chartable::front_end_chr(lh(lw(chr + 352) + 4));
    sw(chr + 344, 0);
    let v0_1 = lw(front) << 2;
    let a1_1 = v0_1 + 0x1fc1258;
    let stage = lw(v0_1 + g::stage);
    let stageprop = lw(v0_1 + g::stageprop);
    swc1(chr + 48, ps2float::sub(f1_1, lwc1(a1_1)));
    swc1(stage + 48, ps2float::sub(lwc1(stage + 48), lwc1(a1_1)));
    if stageprop != 0 {
        swc1(stageprop + 48, ps2float::sub(lwc1(stageprop + 48), lwc1(a1_1)));
    }
    if idx == 29 {
        let v1_1 = lw(front) << 2;
        let gun = lw(v1_1 + g::gun);
        if gun != 0 {
            swc1(gun + 48, lwc1(lw(v1_1 + g::stage) + 48));
        }
    }
    'b3: {
        let page = lw(a0);
        sw(chr + 344, 0);
        if page != g::characterselect_page {
            if page != g::chrplayer_page {
                if page != g::storychrsel_page {
                    if page != g::storychr_page {
                        if page != g::challengechr_page {
                            break 'b3;
                        }
                    }
                }
            }
        }
        'b5: {
            let mut v1_15 = 0;
            'b6: {
                let mut v0_3 = 0;
                'b7: {
                    'b8: {
                        let a2_1 = lw(front);
                        let a0_4 = lbu(a2_1 + 0x3aff28);
                        let mut a0_23 = 0;
                        let mut f12_7 = 0.0_f32;
                        if a0_4 >= 6 {
                            if a0_4 != 6 {
                                v0_3 = lw(front);
                                break 'b7;
                            }
                            let a2_19 = a2_1 << 2;
                            let a0_53 = lw(a2_19 + g::stageprop);
                            if a0_53 == 0 {
                                break 'b8;
                            }
                            let a1_23 = lw(a0_53 + 32);
                            let a0_54 = lw(a2_19 + 0x1fc1238);
                            sw(a1_23 + 140, 0);
                            a0_23 = a1_23;
                            if lw(lw(lw(a0_54 + 32) + 88) + 12) != 0 {
                                f12_7 = lwc1(0x3ac9fc);
                            } else {
                                f12_7 = lwc1(0x3ac9f8);
                            }
                        } else {
                            if a0_4 >= 3 {
                                'b10: {
                                    let s3_2 = idx * 20 + (fal + 16);
                                    let a0_24 = lw(s3_2);
                                    if a0_24 != 0 {
                                        if lw((a2_1 << 2) + g::stageprop) == 0 {
                                            let f12_8 = lwc1(chr + 48);
                                            let a0_25 = lw(((a0_24 - 1) << 4) + g::fe_propanim);
                                            let f13_6 = lwc1(chr + 52);
                                            let f15_3 = 1.8e+02_f32;
                                            let f14_3 = lwc1(chr + 56);
                                            let v0_16 = propNew(a0_25, f12_8, f13_6, f14_3, f15_3);
                                            let a2_7 = lw(g::front);
                                            let v1_7 = lw(a2_7);
                                            sw(v0_16 + 576, v1_7);
                                            let a0_26 = lw(a2_7);
                                            let a1_10 = lw(s3_2);
                                            sw((v1_7 << 2) + g::stageprop, v0_16);
                                            let a0_27 = lw(lw((a0_26 << 2) + g::stageprop) + 32);
                                            let a1_11 = lw(((a1_10 - 1) << 4) + (g::fe_propanim + 8));
                                            obInstSetAnim(a0_27, a1_11, 1.0_f32);
                                            let a2_8 = lw(g::front);
                                            let a0_28 = lw((lw(a2_8) << 2) + g::stageprop);
                                            sw(a0_28 + 16, lw(a0_28 + 16) | 0x1000000);
                                            sw((lw(a2_8) << 2) + g::propdrawflag, 0);
                                            break 'b10;
                                        }
                                    }
                                    let a0_29 = lw(idx * 20 + (fal + 16));
                                    if a0_29 != 0 {
                                        let a4_2 = lw(front);
                                        let a5_1 = lw((a4_2 << 2) + g::stageprop);
                                        if a5_1 != 0 {
                                            let v0_17;
                                            'b12: {
                                                let mut a2_9 = lw(chr + 32);
                                                let a1_12 = lw(((a0_29 - 1) << 4) + (g::fe_propanim + 8));
                                                if lw(a2_9 + 96) != a1_12 {
                                                    if lw((idx << 2) + a4_2 * 320 + 0x1fc0cf8) != 0 {
                                                        v0_17 = lw(a2_9 + 88);
                                                        break 'b12;
                                                    }
                                                    if a1_12 == -1 {
                                                        v0_17 = lw(a2_9 + 88);
                                                        break 'b12;
                                                    }
                                                    let a0_30 = lw(a5_1 + 32);
                                                    obInstSetAnim(a0_30, a1_12, 1.0_f32);
                                                    a2_9 = lw(chr + 32);
                                                    sw((lw(lw(g::front)) << 2) + g::propdrawflag, 0);
                                                }
                                                v0_17 = lw(a2_9 + 88);
                                            }
                                            let v1_8 = lw(v0_17 + 12);
                                            let v0_18 = lw(lw(g::front));
                                            if v1_8 != 0 {
                                                let a0_32 = lw(lw((v0_18 << 2) + g::stageprop) + 32);
                                                calPropMatrices(a0_32, lwc1(0x3aca14));
                                            } else {
                                                let a0_31 = lw(lw((v0_18 << 2) + g::stageprop) + 32);
                                                calPropMatrices(a0_31, lwc1(0x3aca10));
                                            }
                                        }
                                    }
                                }
                                let v0_21;
                                'b14: {
                                    let a1_13 = lw(idx * 20 + fal);
                                    if curAnim(chr, a1_13) != 0 {
                                        let s0_3 = idx * 20 + (fal + 4);
                                        let a1_14 = lw(s0_3);
                                        let v0_20 = curAnim(chr, a1_14);
                                        if v0_20 != 0 {
                                            if !(lwc1(lw(chr + 32) + 140) == 0.0_f32) {
                                                v0_21 = lw(lw(g::front));
                                                break 'b14;
                                            }
                                        }
                                        let a4_3 = (idx << 2) + lw(lw(g::front)) * 320 + 0x1fc0cf8;
                                        if lw(a4_3) != 0 {
                                            v0_21 = lw(lw(g::front));
                                            break 'b14;
                                        }
                                        let s3_3 = lh(lw(idx * 44 + (CHRS_TABLE + 4)) + 48);
                                        sw(a4_3, 1);
                                        if idx != 29 {
                                            let a1_16 = lw(s0_3);
                                            setAnim(chr, a1_16, 0.0_f32, 1.0_f32);
                                        } else {
                                            let a1_15 = lw(fal + 584);
                                            setAnim(chr, a1_15, 0.0_f32, lwc1(0x3aca18));
                                        }
                                        let v1_9 = lw(g::front);
                                        let v1_10 = lw((lw(v1_9) << 2) + g::stage);
                                        swc1(chr + 48, lwc1(v1_10 + 48));
                                        swc1(chr + 52, lwc1(v1_10 + 52));
                                        swc1(chr + 56, lwc1(v1_10 + 56));
                                        tlhAdjustChrPosition(chr);
                                        let a3_1 = lw(g::front);
                                        let v1_11 = lw((lw(a3_1) << 2) + g::stageprop);
                                        if v1_11 != 0 {
                                            let v0_22 = lw(v1_11 + 32);
                                            swc1(v1_11 + 48, lwc1(chr + 48));
                                            swc1(v1_11 + 52, lwc1(chr + 52));
                                            swc1(v1_11 + 56, lwc1(chr + 56));
                                            sw(v0_22 + 100, 0);
                                        }
                                        if s3_3 != 0 {
                                            if lw((lw(a3_1) << 2) + 0x1fc12f8) >= 0 {
                                                v0_21 = lw(a3_1);
                                                break 'b14;
                                            }
                                            let v0_23 = soundDelayStartEx(s3_3, 0, 0, 1);
                                            let a3_1 = lw(g::front);
                                            sw((lw(a3_1) << 2) + 0x1fc12f8, v0_23);
                                        }
                                    }
                                    v0_21 = lw(lw(g::front));
                                }
                                let mut s0_4 = g::gun;
                                if lw((v0_21 << 2) + s0_4) != 0 {
                                    v1_15 = lw(lw(g::front));
                                    break 'b6;
                                }
                                if lw(idx * 20 + (fal + 12)) != 2 {
                                    v0_3 = lw(lw(g::front));
                                    break 'b7;
                                }
                                let a1_21;
                                let a2_16;
                                let a3_5;
                                let v0_26;
                                if lw(idx * 20 + fal) != 675 {
                                    let v0_27 = propNew(370, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                                    let a2_17 = lw(g::front);
                                    let a0_49 = lw(a2_17);
                                    sw(v0_27 + 576, a0_49);
                                    let v1_14 = lw(a2_17);
                                    sw((a0_49 << 2) + s0_4, v0_27);
                                    let a0_50 = lw(lw((v1_14 << 2) + s0_4) + 32);
                                    obInstSetAnim(a0_50, 1, 0.0_f32);
                                    let a2_18 = lw(g::front);
                                    a3_5 = 1;
                                    a1_21 = lw(chr + 32);
                                    let a0_51 = lw(a1_21 + 92);
                                    sw(lw((lw(a2_18) << 2) + s0_4) + 12, lw(chr + 12));
                                    v0_26 = lw(a2_18);
                                    a2_16 = lw(a0_51 + 28);
                                } else {
                                    let v0_24 = propNew(381, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                                    let a2_11 = lw(g::front);
                                    let a0_41 = lw(a2_11);
                                    sw(v0_24 + 576, a0_41);
                                    let v1_12 = lw(a2_11);
                                    sw((a0_41 << 2) + s0_4, v0_24);
                                    let a0_42 = lw(lw((v1_12 << 2) + s0_4) + 32);
                                    obInstSetAnim(a0_42, 1, 0.0_f32);
                                    let a2_12 = lw(g::front);
                                    let a3_4 = 1;
                                    let a1_19 = lw(chr + 32);
                                    let a0_43 = lw(a1_19 + 92);
                                    sw(lw((lw(a2_12) << 2) + s0_4) + 12, lw(chr + 12));
                                    let a2_13 = lw(a0_43 + 28);
                                    let a0_44 = lw((lw(a2_12) << 2) + s0_4);
                                    propSetAttached(a0_44, a1_19, a2_13, a3_4);
                                    sw(lw((lw(lw(g::front)) << 2) + s0_4) + 12, lw(chr + 12));
                                    let v0_25 = propNew(381, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                                    let a2_14 = lw(g::front);
                                    s0_4 = g::gun2;
                                    let a0_46 = lw(a2_14);
                                    sw(v0_25 + 576, a0_46);
                                    let v1_13 = lw(a2_14);
                                    sw((a0_46 << 2) + s0_4, v0_25);
                                    let a0_47 = lw(lw((v1_13 << 2) + s0_4) + 32);
                                    obInstSetAnim(a0_47, 1, 0.0_f32);
                                    let a2_15 = lw(g::front);
                                    a3_5 = 2;
                                    a1_21 = lw(chr + 32);
                                    let a0_48 = lw(a1_21 + 92);
                                    sw(lw((lw(a2_15) << 2) + s0_4) + 12, lw(chr + 12));
                                    v0_26 = lw(a2_15);
                                    a2_16 = lw(a0_48 + 24);
                                }
                                let a0_52 = lw((v0_26 << 2) + s0_4);
                                propSetAttached(a0_52, a1_21, a2_16, a3_5);
                                let a3_1 = lw(g::front);
                                sw(lw((lw(a3_1) << 2) + s0_4) + 12, lw(chr + 12));
                                break 'b8;
                            }
                            if a0_4 != 2 {
                                v0_3 = lw(lw(g::front));
                                break 'b7;
                            }
                            'b20: {
                                'b21: {
                                    let v1_2 = lw((a2_1 << 2) + g::stage);
                                    let f1_2 = ps2float::sub(lwc1(chr + 48), lwc1(v1_2 + 48));
                                    let f2_1 = ps2float::sub(lwc1(chr + 52), lwc1(v1_2 + 52));
                                    let f0_1 = ps2float::sub(lwc1(chr + 56), lwc1(v1_2 + 56));
                                    let s0_1 = idx * 20 + fal;
                                    let a1_2 = lw(s0_1);
                                    let v0_4 = curAnim(chr, a1_2);
                                    if v0_4 != 0 {
                                        let a1_3 = lw(s0_1);
                                        if a1_3 == 524 {
                                            if 0.0_f32 < lwc1(lw(chr + 32) + 140) {
                                                break 'b21;
                                            }
                                        }
                                        let mut v0_5 = idx * 20 + fal;
                                        if a1_3 == 525 {
                                            if 0.0_f32 < lwc1(lw(chr + 32) + 140) {
                                                break 'b21;
                                            }
                                            v0_5 = idx * 20 + fal;
                                        }
                                        if a1_3 != lw(v0_5 + 8) {
                                            break 'b20;
                                        }
                                        if !(0.5_f32 < ps2float::add(ps2float::add(ps2float::mul(f1_2, f1_2), ps2float::mul(f2_1, f2_1)), ps2float::mul(f0_1, f0_1)).abs().psqrt()) {
                                            break 'b20;
                                        }
                                    }
                                }
                                let a1_4 = lw(idx * 20 + fal);
                                if ((a1_4 - 524) as u32) >= (2 as u32) {
                                    if idx != 29 {
                                        setAnim(chr, a1_4, 0.0_f32, 1.0_f32);
                                    } else {
                                        let a1_5 = lw(fal + 580);
                                        setAnim(chr, a1_5, 0.0_f32, lwc1(0x3aca00));
                                    }
                                } else {
                                    setAnim(chr, a1_4, 0.0_f32, 0.0_f32);
                                }
                                let a0_7 = lw(g::front);
                                let v0_6 = lw(a0_7);
                                sw(chr + 16, lw(chr + 16) | 0x1000000);
                                let v1_3 = lw((v0_6 << 2) + g::stage);
                                swc1(chr + 48, lwc1(v1_3 + 48));
                                swc1(chr + 52, lwc1(v1_3 + 52));
                                swc1(chr + 56, lwc1(v1_3 + 56));
                                tlhAdjustChrPosition(chr);
                            }
                            let a3_1 = lw(g::front);
                            let a2_2 = lw(a3_1) << 2;
                            let a0_15;
                            if lw(a2_2 + g::gun) != 0 {
                                a0_15 = lw(a3_1);
                            } else {
                                let a0_11 = lw(idx * 20 + (fal + 12));
                                if a0_11 != 1 {
                                    a0_15 = lw(a3_1);
                                } else {
                                    let mut a0_12 = 0;
                                    let mut f12_4 = 0.0_f32;
                                    let mut f13_4 = 0.0_f32;
                                    let mut f14_1 = 0.0_f32;
                                    let mut f15_1 = 0.0_f32;
                                    'b26: {
                                        'b27: {
                                            if idx != 13 {
                                                if idx != a0_11 {
                                                    if idx != 31 {
                                                        if idx != 40 {
                                                            if lw(idx * 20 + (fal + 4)) != 473 {
                                                                if idx != 29 {
                                                                    a0_12 = 361;
                                                                    if idx != 6 {
                                                                        f12_4 = 0.0_f32;
                                                                        break 'b27;
                                                                    }
                                                                    f12_4 = 0.0_f32;
                                                                    a0_12 = 366;
                                                                    break 'b27;
                                                                }
                                                                f13_4 = -1.0_f32;
                                                                f14_1 = lwc1(0x3aca04);
                                                                a0_12 = 165;
                                                                f15_1 = 1.8e+02_f32;
                                                                f12_4 = lwc1(lw(a2_2 + g::stage) + 48);
                                                                break 'b26;
                                                            }
                                                            f12_4 = 0.0_f32;
                                                            a0_12 = 367;
                                                            break 'b27;
                                                        }
                                                    }
                                                    f12_4 = 0.0_f32;
                                                    a0_12 = 370;
                                                } else {
                                                    f12_4 = 0.0_f32;
                                                    a0_12 = 360;
                                                }
                                            } else {
                                                f12_4 = 0.0_f32;
                                                a0_12 = 379;
                                            }
                                        }
                                        f13_4 = f12_4;
                                        f14_1 = f12_4;
                                        f15_1 = f12_4;
                                    }
                                    let v0_7 = propNew(a0_12, f12_4, f13_4, f14_1, f15_1);
                                    let a3_1 = lw(g::front);
                                    sw((lw(a3_1) << 2) + g::gun, v0_7);
                                    let a0_13 = lw(a3_1);
                                    let s0_2 = g::gun;
                                    sw(lw((a0_13 << 2) + s0_2) + 576, a0_13);
                                    sw(lw((lw(a3_1) << 2) + s0_2) + 12, lw(chr + 12));
                                    if idx != 29 {
                                        let v0_8 = lw(a3_1);
                                        let a1_6 = lw(chr + 32);
                                        let a3_2 = 1;
                                        let a0_14 = lw((v0_8 << 2) + s0_2);
                                        let a2_3 = lw(lw(a1_6 + 92) + 28);
                                        propSetAttached(a0_14, a1_6, a2_3, a3_2);
                                    }
                                    let a3_1 = lw(g::front);
                                    sw(lw((lw(a3_1) << 2) + s0_2) + 12, lw(chr + 12));
                                    sw((lw(a3_1) << 2) + g::gundrawflag, 0);
                                    a0_15 = lw(a3_1);
                                }
                            }
                            let a3_1 = lw(g::front);
                            let v0_10;
                            'b30: {
                                let s2_1 = g::gun;
                                sw((idx << 2) + a0_15 * 320 + 0x1fc0cf8, 0);
                                let a2_4 = lw((lw(a3_1) << 2) + s2_1);
                                if a2_4 != 0 {
                                    if lw(idx * 20 + (fal + 12)) != 2 {
                                        v0_10 = lw(a3_1);
                                        break 'b30;
                                    }
                                    propDelete(a2_4);
                                    let a3_1 = lw(g::front);
                                    let v1_4 = lw(a3_1) << 2;
                                    sw(v1_4 + g::gundrawflag, 0);
                                    sw(v1_4 + s2_1, 0);
                                    let a0_17 = lw((lw(a3_1) << 2) + g::gun2);
                                    if a0_17 == 0 {
                                        v0_10 = lw(a3_1);
                                        break 'b30;
                                    }
                                    propDelete(a0_17);
                                    let a3_1 = lw(g::front);
                                    let v0_9 = lw(a3_1) << 2;
                                    sw(v0_9 + g::gundrawflag2, 0);
                                    sw(v0_9 + g::gun2, 0);
                                    v0_10 = lw(a3_1);
                                    break 'b30;
                                }
                                v0_10 = lw(a3_1);
                            }
                            let a3_1 = lw(g::front);
                            'b32: {
                                let a4_1 = lw((v0_10 << 2) + g::stageprop);
                                if a4_1 != 0 {
                                    let v0_12;
                                    if lw(lw(a4_1 + 32) + 96) == lw(((lw(idx * 20 + (fal + 16)) - 1) << 4) + (g::fe_propanim + 4)) {
                                        v0_12 = lw(a3_1);
                                    } else {
                                        propDelete(a4_1);
                                        let a3_1 = lw(g::front);
                                        let v0_11 = lw(a3_1) << 2;
                                        sw(v0_11 + g::propdrawflag, 0);
                                        sw(v0_11 + g::stageprop, 0);
                                        v0_12 = lw(a3_1);
                                    }
                                    if lw((v0_12 << 2) + g::stageprop) != 0 {
                                        v0_3 = lw(lw(g::front));
                                        break 'b32;
                                    }
                                }
                                let a3_1 = lw(g::front);
                                let v0_14;
                                if lw(idx * 20 + (fal + 12)) != 4 {
                                    v0_14 = lw(a3_1);
                                } else {
                                    let s2_2 = idx * 20 + (fal + 16);
                                    let s3_1 = g::fe_propanim + 4;
                                    let a2_5 = (lw(s2_2) - 1) << 4;
                                    if lw(a2_5 + s3_1) != -1 {
                                        let f12_5 = lwc1(chr + 48);
                                        let a0_19 = lw(a2_5 + g::fe_propanim);
                                        let f13_5 = lwc1(chr + 52);
                                        let f15_2 = 1.8e+02_f32;
                                        let f14_2 = lwc1(chr + 56);
                                        let v0_13 = propNew(a0_19, f12_5, f13_5, f14_2, f15_2);
                                        let a2_6 = lw(g::front);
                                        let a0_20 = lw(a2_6);
                                        sw(v0_13 + 576, a0_20);
                                        let a1_7 = lw(a2_6);
                                        let v1_5 = lw(s2_2);
                                        sw((a0_20 << 2) + g::stageprop, v0_13);
                                        let a0_21 = lw(lw((a1_7 << 2) + g::stageprop) + 32);
                                        let a1_8 = lw(((v1_5 - 1) << 4) + s3_1);
                                        obInstSetAnim(a0_21, a1_8, 1.0_f32);
                                        let a3_1 = lw(g::front);
                                        let a0_22 = lw((lw(a3_1) << 2) + g::stageprop);
                                        sw(a0_22 + 16, lw(a0_22 + 16) | 0x1000000);
                                        sw((lw(a3_1) << 2) + g::propdrawflag, 0);
                                        break 'b8;
                                    }
                                    v0_14 = lw(a3_1);
                                }
                                let v1_6 = lw((v0_14 << 2) + g::stageprop);
                                v0_3 = lw(a3_1);
                                if v1_6 == 0 {
                                    break 'b7;
                                }
                            }
                            let a3_1 = lw(g::front);
                            let a1_9 = v0_3 << 2;
                            if lw(a1_9 + g::propdrawflag) == 0 {
                                v0_3 = lw(a3_1);
                                break 'b7;
                            }
                            let v0_15;
                            if lw(lw(lw(chr + 32) + 88) + 12) != 0 {
                                f12_7 = lwc1(0x3aca0c);
                                v0_15 = a1_9 + g::stageprop;
                            } else {
                                f12_7 = lwc1(0x3aca08);
                                v0_15 = a1_9 + g::stageprop;
                            }
                            a0_23 = lw(lw(v0_15) + 32);
                        }
                        calPropMatrices(a0_23, f12_7);
                    }
                    v0_3 = lw(lw(g::front));
                }
                let a0_55 = v0_3 << 2;
                if lw(a0_55 + g::gun) != 0 {
                    v1_15 = lw(lw(g::front));
                } else {
                    if lw(a0_55 + g::stageprop) == 0 {
                        break 'b5;
                    }
                    v1_15 = lw(lw(g::front));
                }
            }
            let mut s2_3 = 0;
            let mut s5_1 = 0;
            let v0_34;
            'b37: {
                if ((lbu(v1_15 + 0x3aff28) - 3) as u32) < (3 as u32) {
                    let s0_5 = idx * 20 + (fal + 4);
                    let a1_24 = lw(s0_5);
                    let v0_28 = curAnim(chr, a1_24);
                    if v0_28 != 0 {
                        let v1_16 = lw(s0_5);
                        if v1_16 != 695 {
                            if v1_16 != 674 {
                                if v1_16 == 667 {
                                    let f1_8 = lwc1(lw(chr + 32) + 100);
                                    if !(3.5_f32 <= f1_8) {
                                        v0_34 = lw(lw(g::front));
                                        break 'b37;
                                    }
                                    let a0_57 = lw(g::tickframenum);
                                    if f1_8 < lwc1(0x3aca4c) {
                                        s2_3 = ((a0_57.wrapping_rem(10) as u32) < (1 as u32)) as i32;
                                        s5_1 = (((a0_57.wrapping_rem(10) ^ 5) as u32) < (1 as u32)) as i32;
                                    }
                                }
                            } else {
                                let a2_20 = lw(chr + 32);
                                let mut f1_4 = lwc1(lw(chr + 32) + 100);
                                if lwc1(0x3aca24) <= f1_4 {
                                    let v0_29 = lw(g::tickframenum);
                                    if f1_4 < lwc1(0x3aca28) {
                                        s2_3 = (((v0_29 & 3) as u32) < (1 as u32)) as i32;
                                        f1_4 = lwc1(a2_20 + 100);
                                    }
                                }
                                let f1_5;
                                if !(lwc1(0x3aca2c) <= f1_4) {
                                    f1_5 = lwc1(a2_20 + 100);
                                } else {
                                    let v0_30 = lw(g::tickframenum);
                                    if f1_4 < lwc1(0x3aca30) {
                                        s2_3 = (((v0_30 & 3) as u32) < (1 as u32)) as i32;
                                    }
                                    f1_5 = lwc1(a2_20 + 100);
                                }
                                let f1_6;
                                if !(lwc1(0x3aca34) <= f1_5) {
                                    f1_6 = lwc1(a2_20 + 100);
                                } else {
                                    let v0_31 = lw(g::tickframenum);
                                    if f1_5 < lwc1(0x3aca38) {
                                        s2_3 = (((v0_31 & 3) as u32) < (1 as u32)) as i32;
                                    }
                                    f1_6 = lwc1(a2_20 + 100);
                                }
                                let f1_7;
                                if !(lwc1(0x3aca3c) <= f1_6) {
                                    f1_7 = lwc1(a2_20 + 100);
                                } else {
                                    let v0_32 = lw(g::tickframenum);
                                    if f1_6 < lwc1(0x3aca40) {
                                        s2_3 = (((v0_32 & 3) as u32) < (1 as u32)) as i32;
                                    }
                                    f1_7 = lwc1(a2_20 + 100);
                                }
                                if lwc1(0x3aca44) <= f1_7 {
                                    let v0_33 = lw(g::tickframenum);
                                    if f1_7 < lwc1(0x3aca48) {
                                        s2_3 = (((v0_33 & 3) as u32) < (1 as u32)) as i32;
                                    }
                                }
                            }
                        } else {
                            let f1_3 = lwc1(lw(chr + 32) + 100);
                            if lwc1(0x3aca1c) <= f1_3 {
                                let v1_17 = lw(g::tickframenum);
                                if f1_3 < lwc1(0x3aca20) {
                                    s2_3 = ((v1_17.wrapping_rem(6) as u32) < (1 as u32)) as i32;
                                }
                            }
                        }
                    }
                }
                v0_34 = lw(lw(g::front));
            }
            let v0_35 = lw((v0_34 << 2) + g::gun);
            let v0_37;
            if v0_35 == 0 {
                v0_37 = lw(lw(g::front));
            } else {
                let v1_18 = lw(v0_35 + 32);
                let v0_36 = if s2_3 == 0 { lhu(v1_18 + 292) & 0xfffe } else { lhu(v1_18 + 292) | 1 };
                sh(v1_18 + 292, v0_36);
                v0_37 = lw(lw(g::front));
            }
            let v0_38 = lw((v0_37 << 2) + g::gun2);
            if v0_38 != 0 {
                let v1_19 = lw(v0_38 + 32);
                let v0_39 = if s5_1 == 0 { lhu(v1_19 + 292) & 0xfffe } else { lhu(v1_19 + 292) | 1 };
                sh(v1_19 + 292, v0_39);
            }
            let v1_20 = idx * 20 + fal;
            let a1_25 = lw(v1_20 + 12);
            let v1_24;
            if ((a1_25 - 2) as u32) >= (2 as u32) {
                let v1_25 = lw(lw(g::front));
                if a1_25 == 1 {
                    sw((v1_25 << 2) + g::gundrawflag, a1_25);
                    sw((lw(lw(g::front)) << 2) + g::gundrawflag2, a1_25);
                    break 'b5;
                }
                sw((v1_25 << 2) + g::gundrawflag, 0);
                sw((lw(lw(g::front)) << 2) + g::gundrawflag2, 0);
                v1_24 = lw(lw(g::front));
            } else {
                let a1_26 = lw(v1_20 + 4);
                let v0_40 = curAnim(chr, a1_26);
                if v0_40 == 0 {
                    break 'b5;
                }
                let a4_4 = lw(chr + 32);
                let f2_2 = lwc1(a4_4 + 100);
                if ((lw((lw(a4_4 + 96) << 2) + g::animRightTag) as f32).pdiv(6e+01_f32)) <= f2_2 {
                    if lw(idx * 20 + fal) == 649 {
                        let v1_21 = lw(g::front);
                        if lwc1(0x3aca50) <= f2_2 {
                            let a0_59 = lw((lw(v1_21) << 2) + g::stageprop);
                            propDelete(a0_59);
                            let v0_41 = lw(lw(g::front)) << 2;
                            sw(v0_41 + g::propdrawflag, 0);
                            sw(v0_41 + g::stageprop, 0);
                            break 'b5;
                        }
                    }
                    let a1_27 = lw(g::front);
                    if lw(idx * 20 + (fal + 12)) != 2 {
                        let a0_61 = lw((lw(a1_27) << 2) + g::stageprop);
                        let v1_23 = lw(a0_61 + 16);
                        if (v1_23 & 0x1000000) != 0 {
                            sw(a0_61 + 16, v1_23 | 0x1000000);
                            sw((lw(a1_27) << 2) + g::propdrawflag, 1);
                            break 'b5;
                        }
                        if (v1_23 & 256) != 0 {
                            break 'b5;
                        }
                        sw(a0_61 + 16, v1_23 | 0x1000000);
                        sw((lw(a1_27) << 2) + g::propdrawflag, 0);
                        break 'b5;
                    }
                    let a3_6 = lw(g::front);
                    let a0_60 = lw((lw(a3_6) << 2) + g::gun);
                    sw(a0_60 + 16, lw(a0_60 + 16) | 0x1000000);
                    sw((lw(a3_6) << 2) + g::gundrawflag, 1);
                    let v1_22 = lw((lw(a3_6) << 2) + g::gun2);
                    if v1_22 == 0 {
                        break 'b5;
                    }
                    if !(2.0_f32 <= lwc1(a4_4 + 100)) {
                        break 'b5;
                    }
                    sw(v1_22 + 16, lw(v1_22 + 16) | 0x1000000);
                    sw((lw(a3_6) << 2) + g::gundrawflag2, 1);
                    break 'b5;
                }
                let a2_21 = lw(g::front);
                sw((lw(a2_21) << 2) + g::gundrawflag, 0);
                sw((lw(a2_21) << 2) + g::gundrawflag2, 0);
                v1_24 = lw(a2_21);
            }
            sw((v1_24 << 2) + g::propdrawflag, 0);
        }
        let a1_28 = lw(idx * 20 + (fal + 4));
        let v0_42 = curAnim(chr, a1_28);
        if v0_42 != 0 {
            let a0_63 = lw(chr + 32);
            let v0_43 = curAnimComplete(a0_63);
            if v0_43 != 0 {
                let s2_4 = idx * 20 + (fal + 8);
                let a1_29 = lw(s2_4);
                if a1_29 == -1 {
                    sw((lw(lw(g::front)) << 2) + g::propdrawflag, 0);
                } else {
                    setAnim(chr, a1_29, 0.0_f32, 1.0_f32);
                    let a2_22 = lw((lw(lw(g::front)) << 2) + g::stageprop);
                    if a2_22 != 0 {
                        if lw(idx * 20 + fal) == lw(s2_4) {
                            let a1_30 = lw(((lw(idx * 20 + (fal + 16)) - 1) << 4) + (g::fe_propanim + 4));
                            if a1_30 != -1 {
                                let a0_65 = lw(a2_22 + 32);
                                obInstSetAnim(a0_65, a1_30, 1.0_f32);
                            }
                        }
                    }
                }
                let v1_26 = idx * 20 + fal;
                let a1_31 = lw(((lw(v1_26 + 16) - 1) << 4) + (g::fe_propanim + 12));
                if lw(lw(chr + 32) + 96) != a1_31 {
                    if ((lw(v1_26 + 12) - 3) as u32) < (2 as u32) {
                        let v1_27 = lw(g::front);
                        if a1_31 != -1 {
                            let a0_66 = lw(lw((lw(v1_27) << 2) + g::stageprop) + 32);
                            obInstSetAnim(a0_66, a1_31, 1.0_f32);
                            sw((lw(lw(g::front)) << 2) + g::propdrawflag, 0);
                        }
                    }
                }
            }
        }
        if lw(idx * 20 + (fal + 12)) == 4 {
            let a3_7 = lw(g::front);
            let a2_23 = lw((lw(a3_7) << 2) + g::stageprop);
            if a2_23 != 0 {
                sw(a2_23 + 16, lw(a2_23 + 16) | 0x1000000);
                sw((lw(a3_7) << 2) + g::propdrawflag, 1);
            }
        }
    }
}
