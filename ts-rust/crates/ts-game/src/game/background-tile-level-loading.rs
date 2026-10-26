//! Function: loading the rooms, portals and textures of a level from its background tiles.

use crate::*;

/// Load the background tiles of the level from the map buffer (@ 0x254e60): read every tile (kind 0x1324628) of the file, number the rooms, portals and textures across
/// the tiles, copy them into the level arrays, join the portals that lie against each other, remove the portals that have a room missing, then make the room and portal
/// data and the initial light of each room from the tiles.
pub fn bgTileLoadLevel() {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let mut s6_1 = 0;
    let mut s7_1 = 0;
    sw(g::bgNumRooms, 0);
    sw(g::bgNumPortals, 0);
    sw(g::bgNumVolumes, 0);
    memset(0x1fc4828, 0, 400);
    let mut a1_2 = lw(g::mmtempbuffer);
    sw(sp + 160, -1);
    let s2_1 = a1_2 + 8;
    'b1: {
        if (s2_1 as u32) < ((a1_2 + 0x1ffc) as u32) {
            let mut s3_1 = s2_1;
            let v1_1 = lw(a1_2 + 8);
            let mut s2_2 = a1_2 + 16;
            if v1_1 != 0x1324629 {
                let mut a0_2 = v1_1;
                loop {
                    let v1_2;
                    'b3: {
                        'b4: {
                            if a0_2 == 0x1324627 {
                                sb(s2_2 + 43, 0);
                                sw(sp + 160, lw(s2_2 + 4));
                            } else {
                                if a0_2 != 0x1324628 {
                                    v1_2 = lw(s3_1 + 4);
                                    break 'b3;
                                }
                                let a1_3 = lw(sp + 160);
                                let mut s1_1 = s2_2;
                                let mut s4_1 = 0;
                                let a0_3 = a1_3 * 60 + 15 & -16;
                                let v0_1 = memAlloc(a0_3, 0);
                                s6_1 = v0_1;
                                if lw(sp + 160) > 0 {
                                    loop {
                                        let f1_1 = 5.0_f32;
                                        let lo_1 = s4_1 * 60;
                                        sw(sp, lw(s1_1));
                                        s4_1 = s4_1 + 1;
                                        let s0_1 = lhu(s1_1 + 10);
                                        let a0_4 = lo_1 + s6_1;
                                        let a3_1 = lhu(s1_1 + 8);
                                        swc1(sp + 4, ps2float::mul(lb(s1_1 + 4) as f32, f1_1));
                                        swc1(sp + 8, ps2float::mul(lb(s1_1 + 5) as f32, 4.0_f32));
                                        swc1(sp + 12, ps2float::mul(lb(s1_1 + 6) as f32, f1_1));
                                        let v1_3 = lbu(s1_1 + 7);
                                        sh(sp + 20, a3_1);
                                        sw(sp + 16, v1_3);
                                        sw(sp + 24, lw(s1_1 + 12));
                                        sw(sp + 28, lbu(s1_1 + 16));
                                        let v0_2 = lbu(s1_1 + 17);
                                        let s1_2 = s1_1 + 36;
                                        sw(sp + 32, v0_2);
                                        bgTileLoad(a0_4, sp);
                                        s1_1 = s1_2 + (s0_1 << 2);
                                        if s4_1 >= lw(sp + 160) {
                                            break 'b4;
                                        }
                                    }
                                }
                            }
                        }
                        a1_2 = lw(g::mmtempbuffer);
                        v1_2 = lw(s3_1 + 4);
                    }
                    let s2_3 = s2_2 + v1_2;
                    if (s2_3 as u32) >= ((a1_2 + 0x1ffc) as u32) {
                        break 'b1;
                    }
                    s3_1 = s2_3;
                    let v1_4 = lw(s3_1);
                    s2_2 = s3_1 + 8;
                    a0_2 = v1_4;
                    if v1_4 == 0x1324629 {
                        break 'b1;
                    }
                }
            }
        }
    }
    let a1_6 = lw(sp + 160);
    let mut s4_2 = 0;
    let s1_3 = a1_6 - 1;
    let a0_6;
    if a1_6 <= 0 {
        a0_6 = lw(g::bgNumRooms);
    } else {
        loop {
            let lo_2 = s4_2 * 60;
            let a4_1 = lw(g::bgNumRooms);
            s4_2 = s4_2 + 1;
            let a5_1 = lw(g::bgNumPortals);
            let a6_1 = (s4_2 < lw(sp + 160)) as i32;
            let v0_3 = lo_2 + s6_1;
            let a1_7 = lw(v0_3 + 20);
            let a3_2 = lw(g::bgNumVolumes) + lw(v0_3 + 28);
            let a2_2 = lw(v0_3 + 24);
            let a0_5 = a4_1 + lw(v0_3 + 16);
            sw(v0_3 + 40, s7_1);
            sw(v0_3 + 32, a4_1);
            s7_1 = s7_1 + a2_2;
            sw(v0_3 + 36, a5_1);
            sw(g::bgNumRooms, a0_5);
            sw(g::bgNumPortals, a5_1 + a1_7);
            sw(g::bgNumVolumes, a3_2);
            if a6_1 == 0 {
                break;
            }
        }
        a0_6 = lw(g::bgNumRooms);
    }
    {
        let s0_2 = -16;
        let a0_7 = (a0_6 + 1) * 44 + 15 & s0_2;
        let v0_4 = memAlloc(a0_7, 1);
        let a0_8 = lw(g::bgNumPortals);
        sw(g::levelrooms, v0_4);
        let a0_9 = (a0_8 << 2) + 19 & s0_2;
        let v0_5 = memAlloc(a0_9, 1);
        sw(g::levelportals, v0_5);
        let a0_10 = ((s7_1 + 1) << 4) + 15 & s0_2;
        let v0_6 = memAlloc(a0_10, 1);
        let a2_3 = lw(sp + 160);
        sw(g::leveltextures, v0_6);
        if a2_3 > 0 {
            let mut s0_3 = s6_1;
            let mut s4_3 = a2_3;
            loop {
                s4_3 = s4_3 - 1;
                bgTileSetRoomNums(s0_3);
                bgTileSetPortalNums(s0_3);
                bgTileSetTextureNums(s0_3);
                s0_3 = s0_3 + 60;
                if s4_3 == 0 {
                    break;
                }
            }
        }
    }
    {
        let mut s4_4 = s1_3;
        if s4_4 >= 0 {
            let mut s0_4 = s4_4 * 60 + s6_1;
            loop {
                s4_4 = s4_4 - 1;
                let a1_11 = lw(s0_4);
                let a2_4 = lw(s0_4 + 16) * 44 + 44;
                let a0_14 = lw(s0_4 + 32) * 44 + lw(g::levelrooms);
                memcpy(a0_14, a1_11, a2_4);
                let a1_12 = lw(s0_4 + 4);
                let a0_15 = (lw(s0_4 + 36) << 2) + lw(g::levelportals);
                let a2_5 = (lw(s0_4 + 20) + 1) << 2;
                memcpy(a0_15, a1_12, a2_5);
                s0_4 = s0_4 - 60;
                if s4_4 < 0 {
                    break;
                }
            }
        }
    }
    {
        let a0_16 = lw(sp + 160);
        let mut s4_5 = a0_16;
        if a0_16 > 0 {
            let mut s0_5 = s6_1;
            loop {
                s4_5 = s4_5 - 1;
                let a1_13 = lw(s0_5 + 8);
                let a0_17 = (lw(s0_5 + 40) << 4) + lw(g::leveltextures);
                let a2_6 = (lw(s0_5 + 24) + 1) << 4;
                memcpy(a0_17, a1_13, a2_6);
                s0_5 = s0_5 + 60;
                if s4_5 == 0 {
                    break;
                }
            }
        }
    }
    let mut a1_14 = lw(g::bgNumPortals);
    let mut s7_2 = 1;
    loop {
        'b17: {
            let mut s4_6 = 0;
            if (a1_14 - 1) > 0 {
                let mut a7_1 = lw(g::levelportals);
                loop {
                    'b19: {
                        let s2_4 = lw((s4_6 << 2) + a7_1 + 4);
                        let mut s3_2 = s4_6 + 1;
                        if lw(s2_4) >= 0 {
                            if lw(s2_4 + 4) >= 0 {
                                s4_6 = s3_2;
                                break 'b19;
                            }
                            s3_2 = s4_6 + 1;
                        }
                        let mut s1_4 = s3_2;
                        if s3_2 >= a1_14 {
                            s4_6 = s3_2;
                        } else {
                            loop {
                                'b23: {
                                    let a7_2 = s1_4 + 1;
                                    'b24: {
                                        let s0_6 = lw((s1_4 << 2) + a7_1 + 4);
                                        let f1_2;
                                        if lw(s0_6) < 0 {
                                            f1_2 = lwc1(s0_6 + 8);
                                        } else {
                                            if lw(s0_6 + 4) >= 0 {
                                                break 'b24;
                                            }
                                            f1_2 = lwc1(s0_6 + 8);
                                        }
                                        if ps2float::add(ps2float::add(ps2float::mul(lwc1(s2_4 + 8), f1_2), ps2float::mul(lwc1(s2_4 + 12), lwc1(s0_6 + 12))), ps2float::mul(lwc1(s2_4 + 16), lwc1(s0_6 + 16))) < lwc1(0x3acaa8) {
                                            memset(sp, 0, 12);
                                            let a4_2 = 0x3a25a0;
                                            let a2_8 = lw(a4_2 + 8);
                                            sd(sp + 16, ld(a4_2));
                                            sw(sp + 24, a2_8);
                                            let a0_19 = sp + 64;
                                            let a4_3 = 0x3a25b0;
                                            let v1_5 = lw(a4_3 + 8);
                                            sd(sp + 48, ld(a4_3));
                                            sw(sp + 56, v1_5);
                                            memset(a0_19, 0, 12);
                                            let a1_17 = 0x3a25a0;
                                            let v1_6 = lw(a1_17 + 8);
                                            sd(sp + 80, ld(a1_17));
                                            sw(sp + 88, v1_6);
                                            let v0_7 = 0x3a25b0;
                                            let a3_3 = lw(v0_7 + 8);
                                            sd(sp + 96, ld(v0_7));
                                            sw(sp + 104, a3_3);
                                            let mut a1_18 = 0;
                                            let a6_2 = lhu(s2_4 + 20);
                                            let mut f17_1 = lwc1(sp + 48);
                                            let mut f16_1 = lwc1(sp + 52);
                                            let mut f18_1 = lwc1(sp + 56);
                                            let mut f14_1 = lwc1(sp + 16);
                                            let mut f13_1 = lwc1(sp + 20);
                                            let mut f15_1 = lwc1(sp + 24);
                                            let a2_10 = lhu(s0_6 + 20);
                                            let mut f11_1 = lwc1(sp + 64);
                                            let mut f4_1 = lwc1(sp + 68);
                                            let mut f12_1 = lwc1(sp + 72);
                                            let mut f5_1 = lwc1(sp + 96);
                                            let mut f9_1 = lwc1(sp + 100);
                                            let mut f8_1 = lwc1(sp + 104);
                                            let mut f6_1 = lwc1(sp + 80);
                                            let mut f10_1 = lwc1(sp + 84);
                                            let mut f7_1 = lwc1(sp + 88);
                                            if lh(s2_4 + 20) > 0 {
                                                loop {
                                                    let f2_1 = lwc1(sp + 4);
                                                    let f3_1 = lwc1(sp + 8);
                                                    let v1_7 = s2_4 + 24 + a1_18 * 12;
                                                    let a0_20 = s2_4 + 32 + a1_18 * 12;
                                                    let v0_8 = s2_4 + 28 + a1_18 * 12;
                                                    swc1(sp, ps2float::add(lwc1(sp), lwc1(v1_7)));
                                                    swc1(sp + 4, ps2float::add(f2_1, lwc1(v0_8)));
                                                    swc1(sp + 8, ps2float::add(f3_1, lwc1(a0_20)));
                                                    let f0_1 = lwc1(v1_7);
                                                    if f17_1 < f0_1 {
                                                        swc1(sp + 48, f0_1);
                                                        f17_1 = f0_1;
                                                    }
                                                    let f0_2 = lwc1(v0_8);
                                                    if f16_1 < f0_2 {
                                                        swc1(sp + 52, f0_2);
                                                        f16_1 = f0_2;
                                                    }
                                                    let f0_3 = lwc1(a0_20);
                                                    if f18_1 < f0_3 {
                                                        swc1(sp + 56, f0_3);
                                                        f18_1 = f0_3;
                                                    }
                                                    let f0_4 = lwc1(v1_7);
                                                    if f0_4 < f14_1 {
                                                        swc1(sp + 16, f0_4);
                                                        f14_1 = f0_4;
                                                    }
                                                    let f0_5 = lwc1(v0_8);
                                                    if f0_5 < f13_1 {
                                                        swc1(sp + 20, f0_5);
                                                        f13_1 = f0_5;
                                                    }
                                                    let f0_6 = lwc1(a0_20);
                                                    if f0_6 < f15_1 {
                                                        swc1(sp + 24, f0_6);
                                                        f15_1 = f0_6;
                                                    }
                                                    a1_18 = a1_18 + 1;
                                                    if a1_18 >= (a6_2 << 16) >> 16 {
                                                        break;
                                                    }
                                                }
                                            }
                                            let v0_9 = a6_2 << 16;
                                            let f0_7 = (v0_9 >> 16) as f32;
                                            let mut a1_19 = 0;
                                            let f3_2 = (lwc1(sp)).pdiv(f0_7);
                                            let f2_2 = (lwc1(sp + 4)).pdiv(f0_7);
                                            let f1_3 = (lwc1(sp + 8)).pdiv(f0_7);
                                            swc1(sp, f3_2);
                                            swc1(sp + 4, f2_2);
                                            swc1(sp + 8, f1_3);
                                            let f21_1 = f3_2;
                                            let f19_1 = f2_2;
                                            let f20_1 = f1_3;
                                            if (a2_10 << 16) > 0 {
                                                loop {
                                                    let v1_8 = s0_6 + 24 + a1_19 * 12;
                                                    let a0_21 = s0_6 + 32 + a1_19 * 12;
                                                    let v0_10 = s0_6 + 28 + a1_19 * 12;
                                                    swc1(sp + 64, ps2float::add(f11_1, lwc1(v1_8)));
                                                    swc1(sp + 68, ps2float::add(f4_1, lwc1(v0_10)));
                                                    swc1(sp + 72, ps2float::add(f12_1, lwc1(a0_21)));
                                                    let f0_8 = lwc1(v1_8);
                                                    if f5_1 < f0_8 {
                                                        swc1(sp + 96, f0_8);
                                                        f5_1 = f0_8;
                                                    }
                                                    let f0_9 = lwc1(v0_10);
                                                    if f9_1 < f0_9 {
                                                        swc1(sp + 100, f0_9);
                                                        f9_1 = f0_9;
                                                    }
                                                    let f0_10 = lwc1(a0_21);
                                                    if f8_1 < f0_10 {
                                                        swc1(sp + 104, f0_10);
                                                        f8_1 = f0_10;
                                                    }
                                                    let f0_11 = lwc1(v1_8);
                                                    if f0_11 < f6_1 {
                                                        swc1(sp + 80, f0_11);
                                                        f6_1 = f0_11;
                                                    }
                                                    let f1_4 = lwc1(v0_10);
                                                    if f1_4 < f10_1 {
                                                        swc1(sp + 84, f1_4);
                                                        f10_1 = f1_4;
                                                    }
                                                    let f0_12 = lwc1(a0_21);
                                                    let v0_11 = a2_10 << 16;
                                                    if f0_12 < f7_1 {
                                                        swc1(sp + 88, f0_12);
                                                        f7_1 = f0_12;
                                                    }
                                                    a1_19 = a1_19 + 1;
                                                    f11_1 = lwc1(sp + 64);
                                                    if a1_19 >= v0_11 >> 16 {
                                                        break;
                                                    }
                                                    f4_1 = lwc1(sp + 68);
                                                    f12_1 = lwc1(sp + 72);
                                                }
                                                f4_1 = lwc1(sp + 68);
                                                f12_1 = lwc1(sp + 72);
                                            }
                                            let f10_2 = ps2float::sub(f10_1, f13_1);
                                            let f6_2 = ps2float::sub(f6_1, f14_1);
                                            let f3_3 = ((a2_10 << 16) >> 16) as f32;
                                            let f13_2 = lwc1(0x3acaac);
                                            let f5_2 = ps2float::sub(f5_1, f17_1);
                                            swc1(sp + 132, f10_2);
                                            let f9_2 = ps2float::sub(f9_1, f16_1);
                                            swc1(sp + 128, f6_2);
                                            let f2_3 = (f11_1).pdiv(f3_3);
                                            let f4_2 = (f4_1).pdiv(f3_3);
                                            swc1(sp + 144, f5_2);
                                            let f3_4 = (f12_1).pdiv(f3_3);
                                            swc1(sp + 148, f9_2);
                                            let f7_2 = ps2float::sub(f7_1, f15_1);
                                            let f8_2 = ps2float::sub(f8_1, f18_1);
                                            let f0_13 = ps2float::sub(f2_3, f21_1);
                                            swc1(sp + 64, f2_3);
                                            let f1_5 = ps2float::sub(f4_2, f19_1);
                                            swc1(sp + 68, f4_2);
                                            let f2_4 = ps2float::sub(f3_4, f20_1);
                                            swc1(sp + 72, f3_4);
                                            swc1(sp + 112, f0_13);
                                            swc1(sp + 136, f7_2);
                                            swc1(sp + 152, f8_2);
                                            swc1(sp + 116, f1_5);
                                            swc1(sp + 120, f2_4);
                                            if ps2float::add(ps2float::add(ps2float::mul(f0_13, f0_13), ps2float::mul(f1_5, f1_5)), ps2float::mul(f2_4, f2_4)) <= f13_2 {
                                                a1_14 = lw(g::bgNumPortals);
                                                if !(ps2float::add(ps2float::add(ps2float::mul(f6_2, f6_2), ps2float::mul(f10_2, f10_2)), ps2float::mul(f7_2, f7_2)) <= f13_2) {
                                                    break 'b24;
                                                }
                                                s1_4 = a7_2;
                                                if !(ps2float::add(ps2float::add(ps2float::mul(f5_2, f5_2), ps2float::mul(f9_2, f9_2)), ps2float::mul(f8_2, f8_2)) <= f13_2) {
                                                    break 'b23;
                                                }
                                                let mut v1_9 = lw(s2_4);
                                                let a0_22 = v1_9;
                                                if v1_9 < 0 {
                                                    v1_9 = lw(s2_4 + 4);
                                                }
                                                let mut v0_12 = lw(s0_6);
                                                if v0_12 < 0 {
                                                    v0_12 = lw(s0_6 + 4);
                                                }
                                                if a0_22 >= 0 {
                                                    sw(s2_4 + 4, v0_12);
                                                } else {
                                                    sw(s2_4, v0_12);
                                                }
                                                if lw(s0_6) >= 0 {
                                                    sw(s0_6 + 4, v1_9);
                                                } else {
                                                    sw(s0_6, v1_9);
                                                }
                                                'b49: {
                                                    let mut a0_23 = lw(g::bgNumRooms);
                                                    let mut v1_10 = 0;
                                                    if a0_23 > 0 {
                                                        let a6_3 = lw(g::levelrooms);
                                                        loop {
                                                            let a3_4 = lw(v1_10 * 44 + a6_3 + 48);
                                                            let a2_11 = lw(a3_4);
                                                            let mut a1_20 = 0;
                                                            let a4_4 = v1_10 + 1;
                                                            if a2_11 > 0 {
                                                                let mut v0_13 = a1_20 << 2;
                                                                loop {
                                                                    let v0_14 = v0_13 + a3_4;
                                                                    if lw(v0_14 + 4) == a7_2 {
                                                                        sw(v0_14 + 4, s3_2);
                                                                    }
                                                                    a1_20 = a1_20 + 1;
                                                                    v0_13 = a1_20 << 2;
                                                                    if a1_20 >= a2_11 {
                                                                        break;
                                                                    }
                                                                }
                                                                a0_23 = lw(g::bgNumRooms);
                                                            }
                                                            v1_10 = a4_4;
                                                            if v1_10 >= a0_23 {
                                                                break 'b49;
                                                            }
                                                        }
                                                    }
                                                }
                                                s7_2 = 0;
                                            }
                                            a1_14 = lw(g::bgNumPortals);
                                        }
                                    }
                                    s1_4 = a7_2;
                                }
                                a7_1 = lw(g::levelportals);
                                if s1_4 >= a1_14 {
                                    break;
                                }
                            }
                            s4_6 = s3_2;
                        }
                    }
                    a7_1 = lw(g::levelportals);
                    if s4_6 >= a1_14 - 1 {
                        break 'b17;
                    }
                }
            }
        }
        if s7_2 != 0 {
            break;
        }
        s7_2 = 1;
    }
    let mut s4_7 = 0;
    let a0_26;
    if a1_14 <= 0 {
        a0_26 = lw(g::bgNumRooms) + 1;
    } else {
        let mut a0_24 = lw(g::bgNumRooms);
        let a7_3 = lw(g::levelportals);
        let mut v0_15 = s4_7 << 2;
        loop {
            let mut s3_3;
            'b58: {
                let v0_16 = lw(v0_15 + a7_3 + 4);
                if lw(v0_16) >= 0 {
                    s3_3 = s4_7 + 1;
                    if lw(v0_16 + 4) >= 0 {
                        break 'b58;
                    }
                }
                let mut v1_11 = 0;
                if a0_24 <= 0 {
                    s3_3 = s4_7 + 1;
                } else {
                    let a6_4 = lw(g::levelrooms);
                    s3_3 = s4_7 + 1;
                    loop {
                        let mut a2_12 = 0;
                        let a5_2 = lw(v1_11 * 44 + a6_4 + 48);
                        let a3_5 = lw(a5_2);
                        let a4_5 = v1_11 + 1;
                        'b64: {
                            let mut a1_21 = 0;
                            loop {
                                let v0_17 = a2_12 << 2;
                                if a2_12 >= a3_5 {
                                    break 'b64;
                                }
                                a1_21 = v0_17 + a5_2;
                                if lw(a1_21 + 4) == s3_3 {
                                    break;
                                }
                                a2_12 = a2_12 + 1;
                            }
                            let a0_25 = a3_5 - 1;
                            'b65: {
                                let mut v1_12 = a1_21;
                                if a2_12 < a0_25 {
                                    let mut a2_13 = a0_25 - a2_12;
                                    loop {
                                        a2_13 = a2_13 - 1;
                                        sw(v1_12 + 4, lw(v1_12 + 8));
                                        v1_12 = v1_12 + 4;
                                        if a2_13 == 0 {
                                            break 'b65;
                                        }
                                    }
                                }
                            }
                            sw(a5_2, a0_25);
                            a0_24 = lw(g::bgNumRooms);
                        }
                        v1_11 = a4_5;
                        if v1_11 >= a0_24 {
                            break;
                        }
                    }
                    a1_14 = lw(g::bgNumPortals);
                }
            }
            s4_7 = s3_3;
            v0_15 = s4_7 << 2;
            if s4_7 >= a1_14 {
                break;
            }
        }
        a0_26 = a0_24 + 1;
    }
    'b67: {
        let s2_5 = g::roomdata;
        let a0_27 = a0_26 * 48;
        let mut s4_8 = 0;
        let v0_18 = memAlloc(a0_27, 1);
        let a0_28 = lw(g::bgNumRooms);
        sw(g::roombb, v0_18);
        let a0_29 = (a0_28 + 1) * 20 + 15 & -16;
        let v0_19 = memAlloc(a0_29, 1);
        let a0_30 = lw(g::bgNumRooms);
        sw(g::roomdata, v0_19);
        let a0_31 = a0_30 * 20 + 35 & -16;
        let v0_20 = memAlloc(a0_31, 1);
        let a0_32 = lw(g::bgNumRooms);
        sw(s2_5 + 4, v0_20);
        let a0_33 = a0_32 * 20 + 35 & -16;
        let v0_21 = memAlloc(a0_33, 1);
        let a0_34 = lw(g::bgNumRooms);
        sw(s2_5 + 8, v0_21);
        let a0_35 = a0_34 * 20 + 35 & -16;
        let v0_22 = memAlloc(a0_35, 1);
        let a0_36 = lw(g::bgNumRooms);
        sw(s2_5 + 12, v0_22);
        let a0_37 = (a0_36 << 2) + 19 & -16;
        let v0_23 = memAlloc(a0_37, 1);
        let a0_38 = lw(g::bgNumPortals);
        sw(0x3b0084, v0_23);
        let a0_39 = a0_38 * 48 + 63 & -16;
        let v0_24 = memAlloc(a0_39, 1);
        sw(g::portaldata, v0_24);
        bgRoomInitBB();
        bgPortalInitBB();
        let a0_40 = (lw(g::bgNumRooms) << 2) + 19 & -16;
        let v0_25 = memAlloc(a0_40, 1);
        let a0_41 = lw(g::bgNumRooms);
        sw(g::roomprops, v0_25);
        let a0_42 = a0_41 * 24 + 15 & -16;
        let v0_26 = memAlloc(a0_42, 1);
        let v1_13 = lw(sp + 160);
        sw(g::roomlightinitial, v0_26);
        if v1_13 > 0 {
            loop {
                let v0_27 = s4_8 * 60 + s6_1;
                let mut s1_5 = lw(v0_27 + 32);
                let a2_14 = lw(g::roomlightinitial);
                let s3_4 = s4_8 + 1;
                let a0_45;
                if s1_5 >= s1_5 + lw(v0_27 + 16) {
                    a0_45 = lw(sp + 160);
                } else {
                    loop {
                        let a0_43 = s4_8 * 60 + s6_1;
                        let a1_31 = s1_5 * 24 + a2_14;
                        let v0_28 = lw(a0_43 + 44);
                        let v1_14 = v0_28 - 1;
                        if v0_28 != 0 {
                            'b73: {
                                'b74: {
                                    if (v1_14 as u32) < (14 as u32) {
                                        let f0_14;
                                        let f1_7;
                                        'b75: {
                                            let mut v1_15 = 0;
                                            let a0_44;
                                            let f2_5;
                                            let f1_6;
                                            'b76: {
                                                match (v1_14 << 2) >> 2 {
                                                    0 => {
                                                        sw(a1_31, 0);
                                                        swc1(a1_31 + 4, 0.5_f32);
                                                        sw(a1_31 + 8, 0);
                                                        break 'b73;
                                                    }
                                                    1 | 2 | 6 | 10 => {
                                                        break 'b74;
                                                    }
                                                    3 => {
                                                        f0_14 = lwc1(0x3acab8);
                                                        f1_7 = lwc1(0x3acabc);
                                                        break 'b75;
                                                    }
                                                    4 => {
                                                        f0_14 = 1.0_f32;
                                                        f1_7 = lwc1(0x3acab4);
                                                        break 'b75;
                                                    }
                                                    5 => {
                                                        f0_14 = 3.0_f32;
                                                        f1_7 = lwc1(0x3acab0);
                                                        break 'b75;
                                                    }
                                                    7 => {
                                                        v1_15 = 2;
                                                    }
                                                    8 => {
                                                        a0_44 = s4_8 * 60;
                                                        f2_5 = 12.0_f32;
                                                        f1_6 = 1.0_f32;
                                                        sw(a1_31, 2);
                                                        swc1(a1_31 + 4, f1_6);
                                                        swc1(a1_31 + 8, (lw(a0_44 + s6_1 + 48) as f32).pdiv(f2_5));
                                                        break 'b73;
                                                    }
                                                    9 => {
                                                        a0_44 = s4_8 * 60;
                                                        f2_5 = 12.0_f32;
                                                        f1_6 = 3.0_f32;
                                                        sw(a1_31, 2);
                                                        swc1(a1_31 + 4, f1_6);
                                                        swc1(a1_31 + 8, (lw(a0_44 + s6_1 + 48) as f32).pdiv(f2_5));
                                                        break 'b73;
                                                    }
                                                    11 => {
                                                        v1_15 = 1;
                                                    }
                                                    12 => {
                                                        a0_44 = s4_8 * 60;
                                                        f2_5 = 12.0_f32;
                                                        f1_6 = 1.0_f32;
                                                        sw(a1_31, 1);
                                                        swc1(a1_31 + 4, f1_6);
                                                        swc1(a1_31 + 8, (lw(a0_44 + s6_1 + 48) as f32).pdiv(f2_5));
                                                        break 'b73;
                                                    }
                                                    _ => {
                                                        a0_44 = s4_8 * 60;
                                                        f2_5 = 12.0_f32;
                                                        f1_6 = 3.0_f32;
                                                        sw(a1_31, 1);
                                                        swc1(a1_31 + 4, f1_6);
                                                        swc1(a1_31 + 8, (lw(a0_44 + s6_1 + 48) as f32).pdiv(f2_5));
                                                        break 'b73;
                                                    }
                                                }
                                                a0_44 = s4_8 * 60;
                                                sw(a1_31, v1_15);
                                                f2_5 = 12.0_f32;
                                                f1_6 = lwc1(0x3acac0);
                                                swc1(a1_31 + 4, f1_6);
                                                swc1(a1_31 + 8, (lw(a0_44 + s6_1 + 48) as f32).pdiv(f2_5));
                                                break 'b73;
                                            }
                                        }
                                        sw(a1_31, 3);
                                        swc1(a1_31 + 4, f0_14);
                                        swc1(a1_31 + 8, f1_7);
                                        break 'b73;
                                    }
                                }
                                swc1(a1_31 + 4, 0.5_f32);
                                swc1(a1_31 + 8, 1.0_f32);
                                sw(a1_31, 0);
                            }
                            sw(a1_31 + 12, -0xefeff0);
                            sw(a1_31 + 16, -1);
                            sw(a1_31 + 20, lw(s4_8 * 60 + s6_1 + 56));
                        } else {
                            sw(a1_31 + 12, -0xefeff0);
                            sw(a1_31 + 16, -1);
                            sw(a1_31, 0);
                            sw(a1_31 + 20, lw(a0_43 + 56));
                            swc1(a1_31 + 4, 0.5_f32);
                            swc1(a1_31 + 8, 1.0_f32);
                        }
                        s1_5 = s1_5 + 1;
                        let v0_29 = s4_8 * 60 + s6_1;
                        if s1_5 >= lw(v0_29 + 32) + lw(v0_29 + 16) {
                            break;
                        }
                    }
                    a0_45 = lw(sp + 160);
                }
                s4_8 = s3_4;
                if s4_8 >= a0_45 {
                    break 'b67;
                }
            }
        }
    }
    if s6_1 != 0 {
        memFree(s6_1);
    }
}
