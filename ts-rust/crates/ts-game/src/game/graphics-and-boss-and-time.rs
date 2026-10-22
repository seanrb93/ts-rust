//! Functions: the graphics synthesizer set up, the interrupt of the vertical blank, the main loop and the boss routines, the texture loading and the time text.

use crate::*;

/// Set the default double buffer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn sjeGsSetDefDBuff(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32, mut a5: i32, mut a6: i32) {
    let mut s0_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut v1_1: i64 = 0;
    let mut v0_1: i32 = 0;
    s0_1 = g::db;
    a7_1 = a3 << 16;
    a1_1 = (a0 << 16) >> 16;
    a0_1 = s0_1;
    a2_1 = (a1 << 16) >> 16;
    a3_1 = (a2 << 16) >> 16;
    a4_1 = a7_1 >> 16;
    a5_1 = (a4 << 16) >> 16;
    a6_1 = (a5 << 16) >> 16;
    sceGsSetDefDBuff(&[a0_1 as i64, a1_1 as i64, a2_1 as i64, a3_1 as i64, a4_1 as i64, a5_1 as i64, a6_1 as i64, a7_1 as i64]);
    v1_1 = ld(s0_1 + 64) & -0x1000_i64;
    sd(s0_1 + 24, ld(s0_1 + 24) & -0x1000_i64 | 676_i64);
    sd(s0_1 + 432, a6 as i64);
    v0_1 = lw(s0_1 + 24);
    sd(s0_1 + 192, a6 as i64);
    sd(s0_1 + 64, v1_1 | (v0_1 & 4095) as i64);
    return;
}

/// Set the frame buffer mask in the display list.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn dlSetFBMSK(mut a0: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_2: i64 = 0;
    let mut v0_5: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v0_11: i32 = 0;
    if lw(g::lastfrbuf) == 0 {
        s0_1 = g::db + 96;
    } else {
        s0_1 = g::db + 336;
    }
    a0_1 = 48;
    sd(sp, lwu(s0_1) | (a0 as i64) << 32);
    v0_1 = memdbAlloc(a0_1);
    sb(lw(g::gfxtag) + 3, 48);
    sw(v0_1, 0x6c028000);
    v1_1 = lw(g::gfxtag);
    sw(v1_1 + 4, v0_1);
    v0_2 = v0_1 + 4;
    sh(v1_1, 3);
    sw(v0_2, 0x8001);
    v0_3 = v0_2 + 4;
    sw(g::gfxtag, v1_1 + 16);
    sw(v0_3, 0x10000000);
    v0_4 = v0_3 + 4;
    a0_2 = ld(s0_1 + 8);
    sw(v0_4, 14);
    v0_5 = v0_4 + 4;
    sw(v0_5, 0);
    v0_6 = v0_5 + 4;
    sw(v0_6, ((ld(sp) << 32) >> 32) as i32);
    v0_7 = v0_6 + 4;
    sw(v0_7, lw(sp + 4));
    v0_8 = v0_7 + 4;
    sw(v0_8, ((a0_2 << 32) >> 32) as i32);
    v0_9 = v0_8 + 4;
    sw(v0_9, 0);
    v0_10 = v0_9 + 4;
    sw(v0_10, 0x140007fc);
    v0_11 = v0_10 + 4;
    sw(v0_11, 0x11000000);
    sw(v0_11 + 4, 0);
    return;
}

/// Interrupt of the vertical blank.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn vblIntHandler() {
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a0_6: i32 = 0;
    'b1: {
        timeTickRetrace();
        if lw(g::gsdone) != 0 {
            a0_1 = lw(g::gsavail_sid);
            v0_1 = iPollSema(a0_1);
            v1_1 = lw(g::lastfrbuf);
            if v0_1 != 0 {
                a1_1 = lw(g::framedropcount);
                s1_1 = 0;
                v0_2 = 1 - v1_1;
                s0_1 = 0;
                sw(g::lastfrbuf, v0_2);
                if a1_1 != 0 {
                    a1_1 = a1_1 - 1;
                    sw(g::framedropcount, a1_1);
                }
                a0_2 = ((0 as u32) < (v0_2 as u32)) as i32;
                if a1_1 != 0 {
                    a1_2 = 0;
                } else {
                    a1_2 = lw(g::framenum) & 1;
                }
                sjeGsSetHalfOffset(a0_2, a1_2);
                a0_3 = lw(g::lastfrbuf);
                sjeGsSwapDBuff(a0_3);
                a0_4 = lw(g::gsavail_sid);
                iSignalSema(a0_4);
                a1_3 = g::dbinfo;
                if lw(g::dbinfo) == 0 {
                    v1_2 = lw(g::framenum) - (lw(a1_3 + 4) - 128);
                    s1_1 = ((v1_2 - (((if -1 < v1_2 { v1_2 } else { v1_2 + 127 }) >> 7) << 7)) < 2) as i32 ^ 1;
                }
                if lw(a1_3 + 12) == 0 {
                    v1_3 = lw(g::framenum) - (lw(a1_3 + 16) - 128);
                    s0_1 = ((v1_3 - (((if -1 < v1_3 { v1_3 } else { v1_3 + 127 }) >> 7) << 7)) < 2) as i32 ^ 1;
                }
                if s1_1 != 0 {
                    if s0_1 != 0 {
                        if lw(a1_3 + 4) >= lw(a1_3 + 16) {
                            s0_1 = 0;
                        } else {
                            s1_1 = 0;
                        }
                    }
                    a0_5 = lw(g::gsstart_sid);
                    if s1_1 != 0 {
                        sw(g::curgsdmabuf, 0);
                        iSignalSema(a0_5);
                        break 'b1;
                    }
                }
                if s0_1 == 0 {
                    break 'b1;
                }
                a0_6 = lw(g::gsstart_sid);
                sw(g::curgsdmabuf, 1);
                iSignalSema(a0_6);
                break 'b1;
            }
        }
        sw(g::framedropcount, 30);
    }
    return;
}

/// Main loop of the graphics.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn gsMain() {
    let mut a0_1: i32 = 0;
    let mut f0_1: f32 = 0.0;
    let mut f1_1: f32 = 0.0;
    let mut f0_2: f32 = 0.0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut f0_3: f32 = 0.0;
    let mut f2_1: f32 = 0.0;
    let mut f1_2: f32 = 0.0;
    let mut v0_2: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a0_14: i32 = 0;
    'l1: loop {
        a0_1 = lw(g::gsstart_sid);
        sw(g::gsdone, 1);
        WaitSema(a0_1);
        f0_1 = lwc1(0x3afe04);
        f1_1 = lwc1(0x3afe08);
        sw(g::gsdone, 0);
        swc1(g::lastgfxstarttime, f0_1);
        swc1(g::lastgfxendtime, f1_1);
        f0_2 = timeGet();
        sw(0x10000010, 131);
        sw(0x10000000, 0);
        a1_1 = g::dbinfo + 8;
        swc1(0x3afe04, f0_2);
        a0_2 = lw(g::curgsdmabuf) * 12 + a1_1;
        sw(a0_2, lw(a0_2) + 1);
        if lw(lw(g::curgsdmabuf) * 12 + a1_1) < 3 {
            if lw(lw(g::curgsdmabuf) * 12 + a1_1) == 1 {
                a0_3 = lw(g::gsavail_sid);
                WaitSema(a0_3);
                zbtestCopyZB();
                a0_4 = lw(g::gsavail_sid);
                SignalSema(a0_4);
                zbtestDoTest();
                zbtestAdvanceTestPtr();
            }
            a0_5 = 3144;
            a1_2 = 0x200f10;
            a2_1 = lw((lw(g::curgsdmabuf) << 2) + g::dmalists);
            v0_1 = SetAlarm(&[a0_5 as i64, a1_2 as i64, a2_1 as i64]);
            a0_6 = lw(g::gsavail_sid);
            sw(g::alarm_activated, 0);
            WaitSema(a0_6);
            a0_7 = 0;
            FlushCache(a0_7);
            a0_8 = lw(g::dmaVif1);
            a1_3 = lw((lw(g::curgsdmabuf) << 2) + g::dmalists);
            sceDmaSend(&[a0_8 as i64, a1_3 as i64]);
            a0_9 = lw(g::gsdone_sid);
            WaitSema(a0_9);
            a0_10 = lw(g::gsavail_sid);
            SignalSema(a0_10);
            if lw(g::alarm_activated) != 0 {
                a0_12 = 0x3838d0;
                scePrintf(&[a0_12 as i64]);
                a1_4 = 0x1000f590;
                sw(g::numdmafail, lw(g::numdmafail) + 1);
                sw(a1_4, 0);
                sw(0x10009000, 5);
                sw(0x10003c10, 1);
                sw(0x10003000, 1);
                sd(0x12001000, 256_i64);
                sw(a1_4, 1);
            } else {
                a0_11 = v0_1;
                ReleaseAlarm(&[a0_11 as i64]);
            }
        }
        'b4: {
            f0_3 = timeGet();
            f2_1 = lwc1(0x3afe04);
            f1_2 = f0_3;
            swc1(0x3afe08, f1_2);
            if f1_2 < f2_1 {
                'l5: loop {
                    f1_2 = f1_2 + 128.0_f32;
                    swc1(0x3afe08, f1_2);
                    if !(f1_2 < f2_1) {
                        break 'b4;
                    }
                }
            }
        }
        sw(0x10000810, 131);
        v0_2 = lw(0x10000800);
        a0_13 = 0x112492;
        v1_1 = lw(g::cpudone);
        sw(g::ucodecycles, (((((((v0_2 as i64) * (a0_13 as i64)) >> 32) as i32 as i64) << 32 | (((((v0_2 * a0_13) as i64) << 32) as u64) >> 32) as i64) << 32) >> 32) as i32);
        if v1_1 != 0 {
            a0_14 = lw(g::bufffree_sid);
            SignalSema(a0_14);
        }
    }
}

/// Clear the video memory.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn clearvram(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_2: i64 = 0;
    let mut a0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a1_10: i64 = 0;
    let mut a0_12: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a1_16: i64 = 0;
    let mut a0_19: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut a1_20: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a1_21: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut a1_23: i32 = 0;
    a0_1 = 2;
    v0_1 = sceDmaGetChan(&[a0_1 as i64, a1 as i64, a2 as i64, a3 as i64]);
    a1_1 = g::base_ptr;
    a0_2 = sp;
    sceGifPkInit(&[a0_2 as i64, a1_1 as i64]);
    a0_3 = sp;
    sceGifPkReset(&[a0_3 as i64]);
    a1_2 = 0x1000000000008009_i64;
    a0_4 = sp;
    sceGifPkAddGsData(&[a0_4 as i64, a1_2]);
    a0_5 = sp;
    a1_3 = 14;
    sceGifPkAddGsData(&[a0_5 as i64, a1_3 as i64]);
    a0_6 = sp;
    a1_4 = 1;
    sceGifPkAddGsData(&[a0_6 as i64, a1_4 as i64]);
    a0_7 = sp;
    a1_5 = 26;
    sceGifPkAddGsData(&[a0_7 as i64, a1_5 as i64]);
    a0_8 = sp;
    a1_6 = 0x100000;
    sceGifPkAddGsData(&[a0_8 as i64, a1_6 as i64]);
    a0_9 = sp;
    a1_7 = 76;
    sceGifPkAddGsData(&[a0_9 as i64, a1_7 as i64]);
    a0_10 = sp;
    a1_8 = 0;
    sceGifPkAddGsData(&[a0_10 as i64, a1_8 as i64]);
    a0_11 = sp;
    a1_9 = 24;
    sceGifPkAddGsData(&[a0_11 as i64, a1_9 as i64]);
    a1_10 = 0x400000004000000_i64;
    a0_12 = sp;
    sceGifPkAddGsData(&[a0_12 as i64, a1_10]);
    a0_13 = sp;
    a1_11 = 64;
    sceGifPkAddGsData(&[a0_13 as i64, a1_11 as i64]);
    a0_14 = sp;
    a1_12 = 0;
    sceGifPkAddGsData(&[a0_14 as i64, a1_12 as i64]);
    a0_15 = sp;
    a1_13 = 71;
    sceGifPkAddGsData(&[a0_15 as i64, a1_13 as i64]);
    a0_16 = sp;
    a1_14 = 6;
    sceGifPkAddGsData(&[a0_16 as i64, a1_14 as i64]);
    a0_17 = sp;
    a1_15 = 0;
    sceGifPkAddGsData(&[a0_17 as i64, a1_15 as i64]);
    a0_18 = sp;
    a1_16 = (a0 & 255) as i64 | 0x3f80000000000000_i64 | (((a2 & 255) as i64) << 16 | ((a1 & 255) as i64) << 8) | ((a3 & 255) as i64) << 24;
    sceGifPkAddGsData(&[a0_18 as i64, a1_16]);
    a0_19 = sp;
    a1_17 = 1;
    sceGifPkAddGsData(&[a0_19 as i64, a1_17 as i64]);
    a0_20 = sp;
    a1_18 = 0;
    sceGifPkAddGsData(&[a0_20 as i64, a1_18 as i64]);
    a0_21 = sp;
    a1_19 = 4;
    sceGifPkAddGsData(&[a0_21 as i64, a1_19 as i64]);
    a1_20 = 0x40004000;
    a0_22 = sp;
    sceGifPkAddGsData(&[a0_22 as i64, a1_20 as i64]);
    a0_23 = sp;
    a1_21 = 4;
    sceGifPkAddGsData(&[a0_23 as i64, a1_21 as i64]);
    a0_24 = sp;
    sceGifPkTerminate(&[a0_24 as i64]);
    a0_25 = 0;
    FlushCache(a0_25);
    a0_26 = sp;
    s0_1 = lw(sp + 4) & 0xfffffff;
    v0_2 = sceGifPkSize(&[a0_26 as i64]);
    a0_27 = v0_1;
    a1_22 = s0_1;
    a2_1 = v0_2;
    sceDmaSendN(&[a0_27 as i64, a1_22 as i64, a2_1 as i64]);
    a0_28 = 0;
    a1_23 = 0;
    sceGsSyncPath(&[a0_28 as i64, a1_23 as i64]);
    return;
}

/// Start the boss routines.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn bossMakeAll() {
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    bossMake();
    trigMake();
    a0_1 = 0x3b0380;
    a1_1 = 0x1ae1c00;
    memMake(a0_1, a1_1);
    packfileMake();
    profileMake();
    windowMake();
    joyMake();
    soundMake();
    musicMake();
    mcardMake();
    mcseqMake();
    ilinkMake();
    cheatsMake();
    signonMake();
    frontmpgMake();
    mapmakerMake();
    return;
}

/// Main loop of the boss routines.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn bossMainLoop() {
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut t4_1: i32 = 0;
    let mut t4_2: i32 = 0;
    let mut t4_3: i32 = 0;
    let mut t4_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut t4_5: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut t4_6: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut t4_7: i32 = 0;
    let mut t4_8: i32 = 0;
    let mut t4_9: i32 = 0;
    let mut t4_10: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a3_8: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a3_9: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut t4_11: i32 = 0;
    let mut t4_12: i32 = 0;
    let mut t4_13: i32 = 0;
    let mut t4_14: i32 = 0;
    let mut t4_15: i32 = 0;
    let mut t4_16: i32 = 0;
    let mut t4_17: i32 = 0;
    let mut t4_18: i32 = 0;
    let mut t4_19: i32 = 0;
    let mut t4_20: i32 = 0;
    let mut t4_21: i32 = 0;
    let mut t4_22: i32 = 0;
    let mut t4_23: i32 = 0;
    let mut t4_24: i32 = 0;
    let mut t4_25: i32 = 0;
    let mut t4_26: i32 = 0;
    let mut t4_27: i32 = 0;
    let mut t4_28: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a2_11: i32 = 0;
    let mut a3_10: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a3_11: i32 = 0;
    let mut a2_12: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut a3_12: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a2_13: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a4_5: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a2_14: i32 = 0;
    let mut a3_13: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a4_6: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a2_15: i32 = 0;
    let mut a3_14: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut a1_17: i32 = 0;
    s0_1 = 0;
    sw(g::currentplayer, lw(g::players));
    profileTick();
    debugmenuTick();
    a0_1 = 0;
    a1_1 = 0;
    a2_1 = 639;
    a3_1 = 223;
    windowSet(a0_1, a1_1, a2_1, a3_1);
    ilinkTick();
    a0_2 = s0_1;
    'l1: loop {
        a1_2 = 0;
        a2_2 = 0;
        s0_1 = s0_1 + 1;
        joySetShock(a0_2, a1_2, a2_2);
        a0_2 = s0_1;
        if s0_1 >= 4 {
            break;
        }
    }
    'b3: {
        s1_1 = 0;
        memdbTick();
        gameTick();
        lvTickBefore();
        if lw(g::lvnumplayers) > 0 {
            s0_2 = 0;
            'l4: loop {
                v0_1 = lw(g::players) + s0_2;
                v1_1 = lw(v0_1 + 384);
                sw(g::currentplayer, v0_1);
                if v1_1 != 0 {
                    'b6: {
                        v0_2 = camGetMode();
                        v0_3 = lw(g::lvlevelnum);
                        if v0_2 != 2 {
                            a1_3 = lw(g::currentplayer);
                            if v0_3 != 6 {
                                a0_3 = lw(a1_3 + 384);
                                v0_4 = lw(a0_3 + 16) & !(0 | 0x1000000 << (s1_1 & 31));
                                break 'b6;
                            }
                        }
                        a0_3 = lw(lw(g::currentplayer) + 384);
                        v0_4 = lw(a0_3 + 16) | 0x1000000 << (s1_1 & 31);
                    }
                    sw(a0_3 + 16, v0_4);
                }
                s1_1 = s1_1 + 1;
                lvTickPlayer();
                s0_2 = s0_2 + 1820;
                if s1_1 >= lw(g::lvnumplayers) {
                    break 'b3;
                }
            }
        }
    }
    lvTickAfter();
    a3_2 = 0x2eeff0;
    a2_3 = lw((lw(g::curcpudmabuf) << 2) + g::dmalists);
    a0_4 = 48;
    sw(g::gfxtag, a2_3);
    sb(a2_3 + 3, 48);
    v0_5 = lw(g::gfxtag);
    sw(v0_5 + 4, a3_2);
    sh(v0_5, (0x2f2bc0 - a3_2) >> 4);
    sw(g::gfxtag, v0_5 + 16);
    v0_6 = memdbAlloc(a0_4);
    sb(lw(g::gfxtag) + 3, 48);
    sw(v0_6, 0x1000404);
    v1_2 = lw(g::gfxtag);
    sw(v1_2 + 4, v0_6);
    t4_1 = v0_6 + 4;
    sw(t4_1, 0x6c028000);
    t4_2 = t4_1 + 4;
    sw(t4_2, 0x8001);
    t4_3 = t4_2 + 4;
    sw(t4_3, 0x10000000);
    t4_4 = t4_3 + 4;
    sw(g::gfxtag, v1_2 + 16);
    sw(t4_4, 14);
    a0_5 = 0;
    t4_5 = t4_4 + 4;
    sh(v1_2, 3);
    sw(t4_5, 0);
    a1_4 = 0;
    t4_6 = t4_5 + 4;
    a2_4 = 640;
    sw(t4_6, 1);
    a3_3 = 224;
    t4_7 = t4_6 + 4;
    sw(t4_7, 0);
    t4_8 = t4_7 + 4;
    sw(t4_8, 70);
    t4_9 = t4_8 + 4;
    sw(t4_9, 0);
    t4_10 = t4_9 + 4;
    sw(t4_10 + 4, 0x11000000);
    sw(t4_10, 0x140007fc);
    dlSetClip(a0_5, a1_4, a2_4, a3_3);
    a0_6 = 1;
    dlSetZB(a0_6);
    v0_7 = lvGetBgCol();
    a4_1 = v0_7;
    a0_7 = 0;
    a1_5 = 0;
    a2_5 = 640;
    a3_4 = 224;
    dlClearFB(a0_7, a1_5, a2_5, a3_4, a4_1);
    a0_8 = 3;
    dlSetZB(a0_8);
    v0_8 = ilinkGetStartLocalPlayerNum();
    s1_2 = v0_8;
    'l8: loop {
        v0_9 = ilinkGetStartLocalPlayerNum();
        v0_10 = ilinkGetNumLocalPlayers();
        if s1_2 >= v0_9 + v0_10 {
            break;
        }
        v0_15 = lw(g::players) + s1_2 * 1820;
        a3_9 = lw(v0_15 + 16);
        a0_16 = lw(v0_15 + 4);
        a1_10 = lw(v0_15 + 8);
        a2_10 = lw(v0_15 + 12);
        sw(g::currentplayer, v0_15);
        windowSet(a0_16, a1_10, a2_10, a3_9);
        a0_17 = 80;
        v0_16 = memdbAlloc(a0_17);
        sb(lw(g::gfxtag) + 3, 48);
        sw(v0_16, 0x6c048000);
        a1_11 = lw(g::gfxtag);
        a0_18 = lw(lw(g::currentplayer) + 1760);
        sw(a1_11 + 4, v0_16);
        v1_4 = lw(a0_18);
        t4_11 = v0_16 + 4;
        sw(g::gfxtag, a1_11 + 16);
        sw(t4_11, v1_4);
        t4_12 = t4_11 + 4;
        sh(a1_11, 5);
        sw(t4_12, lw(a0_18 + 4));
        t4_13 = t4_12 + 4;
        sw(t4_13, lw(a0_18 + 8));
        t4_14 = t4_13 + 4;
        sw(t4_14, lw(a0_18 + 12));
        t4_15 = t4_14 + 4;
        sw(t4_15, lw(a0_18 + 16));
        t4_16 = t4_15 + 4;
        sw(t4_16, lw(a0_18 + 20));
        t4_17 = t4_16 + 4;
        sw(t4_17, lw(a0_18 + 24));
        t4_18 = t4_17 + 4;
        sw(t4_18, lw(a0_18 + 28));
        t4_19 = t4_18 + 4;
        sw(t4_19, lw(a0_18 + 32));
        t4_20 = t4_19 + 4;
        sw(t4_20, lw(a0_18 + 36));
        t4_21 = t4_20 + 4;
        sw(t4_21, lw(a0_18 + 40));
        t4_22 = t4_21 + 4;
        sw(t4_22, lw(a0_18 + 44));
        t4_23 = t4_22 + 4;
        sw(t4_23, lw(a0_18 + 48));
        t4_24 = t4_23 + 4;
        sw(t4_24, lw(a0_18 + 52));
        t4_25 = t4_24 + 4;
        sw(t4_25, lw(a0_18 + 56));
        t4_26 = t4_25 + 4;
        sw(t4_26, lw(a0_18 + 60));
        t4_27 = t4_26 + 4;
        sw(t4_27, 0x1400079d);
        t4_28 = t4_27 + 4;
        sw(t4_28, 0x11000000);
        sw(t4_28 + 4, 0);
        lvGfx();
        v0_17 = lw(g::turtlegs);
        if lw(g::turtlecpu) != 0 {
            bossTurtleCpu();
            v0_17 = lw(g::turtlegs);
        }
        if v0_17 == 0 {
            a0_19 = 0;
        } else {
            bossTurtleGs();
            a0_19 = 0;
        }
        a1_12 = 0;
        a2_11 = 639;
        a3_10 = 223;
        dlSetClip(a0_19, a1_12, a2_11, a3_10);
        s1_2 = s1_2 + 1;
        a0_20 = 0;
        dlSetBlend(a0_20);
        v1_5 = lw(g::currentplayer);
        a4_3 = 128;
        a1_13 = lw(v1_5 + 8);
        a3_11 = a1_13;
        a2_12 = lw(v1_5 + 12) + 1;
        a1_14 = a1_13 - 1;
        a0_21 = lw(v1_5 + 4) - 2;
        dlFillRectangle(a0_21, a1_14, a2_12, a3_11, a4_3);
        v0_18 = lw(g::currentplayer);
        a4_4 = 128;
        a1_15 = lw(v0_18 + 16);
        a3_12 = a1_15 + 1;
        a0_22 = lw(v0_18 + 4) - 2;
        a2_13 = lw(v0_18 + 12) + 1;
        dlFillRectangle(a0_22, a1_15, a2_13, a3_12, a4_4);
        v0_19 = lw(g::currentplayer);
        a4_5 = 128;
        a0_23 = lw(v0_19 + 4);
        a2_14 = a0_23 + 1;
        a3_13 = lw(v0_19 + 16) + 1;
        a0_24 = a0_23 - 2;
        a1_16 = lw(v0_19 + 8) - 1;
        dlFillRectangle(a0_24, a1_16, a2_14, a3_13, a4_5);
        v0_20 = lw(g::currentplayer);
        a4_6 = 128;
        a0_25 = lw(v0_20 + 12);
        a2_15 = a0_25 + 2;
        a3_14 = lw(v0_20 + 16) + 1;
        a0_26 = a0_25 - 1;
        a1_17 = lw(v0_20 + 8) - 1;
        dlFillRectangle(a0_26, a1_17, a2_15, a3_14, a4_6);
    }
    v0_11 = ilinkGetNumLocalPlayers();
    if v0_11 == 3 {
        v0_12 = lvGetBgCol();
        s0_3 = lw(g::players);
        if v0_12 != 128 {
            a0_9 = 1;
            v0_13 = dlSetZB(a0_9);
            sw(s0_3 + 0x1554, v0_11);
            s0_4 = s0_3 + 0x1554;
            a0_10 = s0_4;
            playerSetWindow(a0_10);
            a0_11 = lw(s0_4 + 4);
            a1_6 = lw(s0_4 + 8);
            a2_6 = lw(s0_4 + 12);
            a3_5 = lw(s0_4 + 16);
            windowSet(a0_11, a1_6, a2_6, a3_5);
            a4_2 = 128;
            a0_12 = lw(s0_4 + 4) - 2;
            a3_6 = lw(s0_4 + 16);
            a1_7 = lw(s0_4 + 8) - 1;
            a2_7 = lw(s0_4 + 12) + 2;
            dlClearFB(a0_12, a1_7, a2_7, a3_6, a4_2);
            a0_13 = v0_13;
            dlSetZB(a0_13);
        }
    }
    zbtestAdvanceAddPtr();
    zbtestAdvanceDrawPtr();
    a0_14 = 0;
    a1_8 = 0;
    a2_8 = 639;
    a3_7 = 223;
    windowSet(a0_14, a1_8, a2_8, a3_7);
    v0_14 = g::window;
    a0_15 = lw(g::window);
    a3_8 = lw(v0_14 + 24);
    a1_9 = lw(v0_14 + 20);
    a2_9 = lw(v0_14 + 4);
    dlSetClip(a0_15, a1_9, a2_9, a3_8);
    if lw(g::gamestate) == 6 {
        guiLoadingGfx();
    }
    debugmenuGfx();
    profileGfx();
    dlFinish();
    sb(lw(g::gfxtag) + 3, 112);
    v1_3 = lw(g::gfxtag);
    sw(v1_3 + 4, 0);
    sw(g::gfxtag, v1_3 + 16);
    sh(v1_3, 0);
    return;
}

/// Load a texture.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn emLoadTexture(mut a0: i32, mut a1: i32, mut a2: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let mut a0_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v1_1: i64 = 0;
    let mut a0_5: i64 = 0;
    let mut a1_4: i64 = 0;
    let mut s4_1: i32 = 0;
    let mut fp_1: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a7_2: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut lo_1: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut a7_3: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut s1_4: i32 = 0;
    let mut s1_5: i32 = 0;
    let mut s1_6: i32 = 0;
    let mut s1_7: i32 = 0;
    let mut s1_8: i32 = 0;
    let mut s1_9: i32 = 0;
    let mut s1_10: i32 = 0;
    let mut s1_11: i32 = 0;
    let mut s1_12: i32 = 0;
    let mut s1_13: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut v1_2: i64 = 0;
    let mut v0_10: i64 = 0;
    let mut v0_11: i32 = 0;
    let mut v0_12: i64 = 0;
    let mut a0_23: i64 = 0;
    let mut s1_14: i32 = 0;
    let mut s1_15: i32 = 0;
    let mut s1_16: i32 = 0;
    let mut s1_17: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut v0_15: i64 = 0;
    let mut a0_26: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a0_28: i64 = 0;
    let mut s1_18: i32 = 0;
    let mut s1_19: i32 = 0;
    let mut s1_20: i32 = 0;
    let mut s1_21: i32 = 0;
    let mut v0_18: i64 = 0;
    let mut a0_29: i64 = 0;
    let mut s1_22: i32 = 0;
    let mut s1_23: i32 = 0;
    let mut s1_24: i32 = 0;
    let mut s1_25: i32 = 0;
    let mut v0_19: i64 = 0;
    let mut a0_30: i64 = 0;
    let mut s1_26: i32 = 0;
    let mut a0_31: i32 = 0;
    let mut s1_27: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut s1_28: i32 = 0;
    let mut a1_15: i64 = 0;
    let mut a0_32: i64 = 0;
    let mut s1_29: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut v0_21: i64 = 0;
    let mut a0_33: i64 = 0;
    let mut s1_30: i32 = 0;
    let mut s1_31: i32 = 0;
    a0_1 = 0;
    s3_1 = lw(lw(g::thistexbuf) + 4) + a0 * 40;
    if a2 != 0 {
        a0_1 = lw(a2 + 4);
    }
    sw(s3_1 + 8, a0_1);
    v0_1 = 0;
    if a2 != 0 {
        v0_1 = lw(a2 + 8);
    }
    'b3: {
        sw(s3_1 + 12, v0_1);
        if a2 != 0 {
            a0_2 = a1;
            if (lw(a2 + 12) & 0x10000000) != 0 {
                a1_1 = s3_1;
                ImageLoadFromMem(a0_2, a1_1);
                v0_2 = lbu(s3_1 + 36);
                break 'b3;
            }
        }
        a0_3 = a1;
        a1_2 = s3_1;
        ImageLoad(a0_3, a1_2);
        v0_2 = lbu(s3_1 + 36);
    }
    a0_4 = s3_1;
    if (v0_2 & 1) != 0 {
        a1_3 = 1;
        emGetNextPos(a0_4, a1_3);
    }
    'b6: {
        v1_1 = ld(s3_1 + 8);
        a0_5 = ld(s3_1 + 16);
        a1_4 = ld(s3_1 + 24);
        sd(sp, ld(s3_1));
        sd(sp + 8, v1_1);
        sd(sp + 16, a0_5);
        sd(sp + 24, a1_4);
        sd(sp + 32, ld(s3_1 + 32));
        s4_1 = 0;
        if lbu(s3_1 + 37) == 0 {
            fp_1 = a0 << 7;
            v0_7 = lbu(s3_1 + 36);
        } else {
            fp_1 = a0 << 7;
            a0_6 = sp;
            'b9: {
                'l7: loop {
                    a1_5 = 0;
                    v0_3 = emGetNextPos(a0_6, a1_5);
                    if v0_3 == 0 {
                        sw(sp + 16, 0);
                    }
                    v0_4 = s4_1 << 2;
                    s1_1 = sp + 48 + v0_4;
                    s2_1 = sp + 80 + v0_4;
                    sw(s1_1, lw(sp + 16));
                    a0_7 = sp;
                    v0_5 = GSBufferWidth(a0_7);
                    sw(s2_1, v0_5);
                    if lw(sp + 16) == 0 {
                        break 'b9;
                    }
                    'b11: {
                        a0_8 = 2;
                        sceDmaGetChan(&[a0_8 as i64]);
                        a1_6 = lw(sp + 4);
                        a3_1 = lhu(sp + 2);
                        if (a1_6 & 7) == 0 {
                            if (a3_1 & 3) != 0 {
                                a0_15 = lhu(sp + 2);
                                break 'b11;
                            }
                            a7_1 = a3_1 << 16;
                            s0_1 = g::pTex;
                            a1_7 = lh(s1_1);
                            a2_1 = lh(s2_1);
                            a3_2 = if (lbu(s3_1 + 36) & 1) == 0 { 0 } else { 19 };
                            a6_1 = (lhu(sp + 4) << 16) >> 16;
                            a7_2 = a7_1 >> 16;
                            a0_9 = s0_1;
                            a4_1 = 0;
                            a5_1 = 0;
                            sceGsSetDefLoadImage(&[a0_9 as i64, a1_7 as i64, a2_1 as i64, a3_2 as i64, a4_1 as i64, a5_1 as i64, a6_1 as i64, a7_2 as i64]);
                            a0_10 = 0;
                            FlushCache(a0_10);
                            a0_11 = lw(g::gsavail_sid);
                            WaitSema(a0_11);
                            a1_8 = lw(sp + 20);
                            a0_12 = s0_1;
                            sjeGsExecLoadImage(a0_12, a1_8);
                            a0_13 = 0;
                            a1_9 = 0;
                            sceGsSyncPath(&[a0_13 as i64, a1_9 as i64]);
                            a0_14 = lw(g::gsavail_sid);
                            SignalSema(a0_14);
                            a1_6 = lw(sp + 4);
                            a3_1 = lhu(sp + 2);
                        }
                        a0_15 = lhu(sp + 2);
                    }
                    a2_2 = lbu(s3_1 + 36);
                    lo_1 = a1_6 * a0_15;
                    sw(sp + 4, ((a1_6 as u32) >> 1) as i32);
                    a1_10 = lw(sp + 20);
                    v0_6 = lhu(sp);
                    s4_1 = s4_1 + 1;
                    sh(sp + 2, ((a3_1 as u32) >> 1) as i32);
                    sh(sp, ((v0_6 as u32) >> 1) as i32);
                    sw(sp + 20, a1_10 + (if (a2_2 & 1) == 0 { lo_1 << 2 } else { lo_1 }));
                    a0_6 = sp;
                    if s4_1 >= lbu(s3_1 + 37) {
                        break;
                    }
                }
                v0_7 = lbu(s3_1 + 36);
                break 'b6;
            }
            v0_7 = lbu(s3_1 + 36);
        }
    }
    sb(s3_1 + 37, s4_1);
    if (v0_7 & 1) != 0 {
        a1_11 = lh(s3_1 + 24);
        s0_2 = g::pTex;
        a2_3 = 1;
        a0_16 = s0_2;
        a3_3 = 0;
        a4_2 = 0;
        a5_2 = 0;
        a6_2 = 16;
        a7_3 = 16;
        sceGsSetDefLoadImage(&[a0_16 as i64, a1_11 as i64, a2_3 as i64, a3_3 as i64, a4_2 as i64, a5_2 as i64, a6_2 as i64, a7_3 as i64]);
        a0_17 = 0;
        FlushCache(a0_17);
        a0_18 = lw(g::gsavail_sid);
        WaitSema(a0_18);
        a1_12 = lw(s3_1 + 28);
        a0_19 = s0_2;
        sjeGsExecLoadImage(a0_19, a1_12);
        a0_20 = 0;
        a1_13 = 0;
        sceGsSyncPath(&[a0_20 as i64, a1_13 as i64]);
        a0_21 = lw(g::gsavail_sid);
        SignalSema(a0_21);
    }
    v0_8 = lw(lw(g::thistexbuf)) + fp_1;
    sw(s3_1 + 32, v0_8);
    sw(v0_8, 0);
    s1_2 = v0_8 + 4;
    sw(s1_2, 0);
    s1_3 = s1_2 + 4;
    sw(s1_3, 0);
    s1_4 = s1_3 + 4;
    sw(s1_4, 0x6c0780d8);
    s1_5 = s1_4 + 4;
    sw(s1_5, 1);
    s1_6 = s1_5 + 4;
    sw(s1_6, 0x60000000);
    s1_7 = s1_6 + 4;
    sw(s1_7, 0xeeeeee);
    s1_8 = s1_7 + 4;
    sw(s1_8, 0);
    s1_9 = s1_8 + 4;
    sw(s1_9, 0);
    s1_10 = s1_9 + 4;
    sw(s1_10, 0);
    s1_11 = s1_10 + 4;
    sw(s1_11, 63);
    s1_12 = s1_11 + 4;
    sw(s1_12, 0);
    s1_13 = s1_12 + 4;
    if (lbu(s3_1 + 36) & 2) == 0 {
        'b15: {
            if a2 != 0 {
                v0_11 = 0;
                if (lw(a2 + 12) & 0x20000000) != 0 {
                    break 'b15;
                }
            }
            v0_11 = 1;
        }
        v1_2 = (v0_11 as i64) << 6;
        v0_10 = (v0_11 as i64) << 5;
    } else {
        a1_14 = 4;
        if a2 != 0 {
            a1_14 = if (lw(a2 + 12) & 0x20000000) != 0 { 2 } else { a1_14 };
        }
        'b18: {
            a0_22 = 0;
            if a2 != 0 {
                if (lw(a2 + 12) & 0x20000000) != 0 {
                    v0_9 = lbu(s3_1 + 37);
                    break 'b18;
                }
            }
            a0_22 = 1;
            v0_9 = lbu(s3_1 + 37);
        }
        v1_2 = (a1_14 as i64) << 6 | (a0_22 as i64) << 5;
        v0_10 = ((v0_9 - 1) as i64) << 2 | -0x1e00000000_i64;
    }
    v0_12 = v0_10 | v1_2;
    a0_23 = 0xffffffff_i64;
    sw(s1_13, (((v0_12 & a0_23) << 32) >> 32) as i32);
    s1_14 = s1_13 + 4;
    sw(s1_14, (((((v0_12 as u64) >> 32) as i64 & a0_23) << 32) >> 32) as i32);
    s1_15 = s1_14 + 4;
    sw(s1_15, 20);
    s1_16 = s1_15 + 4;
    sw(s1_16, 0);
    s1_17 = s1_16 + 4;
    if (lbu(s3_1 + 36) & 1) == 0 {
        a0_26 = lhu(s3_1);
        v0_16 = PowerOf2(a0_26);
        a0_27 = lhu(s3_1 + 2);
        v0_17 = PowerOf2(a0_27);
        v0_15 = lwu(sp + 48) | 0x400000000_i64 | ((v0_16 as i64) << 26 | lwu(sp + 80) << 14) | (v0_17 as i64) << 30;
    } else {
        a0_24 = lhu(s3_1);
        v0_13 = PowerOf2(a0_24);
        a0_25 = lhu(s3_1 + 2);
        v0_14 = PowerOf2(a0_25);
        v0_15 = lwu(sp + 48) | -0x8000000000000000_i64 | (lwu(sp + 80) << 14 | 0x400000000_i64) | ((v0_13 as i64) << 26 | 0x1300000_i64 | ((v0_14 as i64) << 30 | lwu(s3_1 + 24) << 37));
    }
    a0_28 = 0xffffffff_i64;
    sw(s1_17, (((v0_15 & a0_28) << 32) >> 32) as i32);
    s1_18 = s1_17 + 4;
    sw(s1_18, (((((v0_15 as u64) >> 32) as i64 & a0_28) << 32) >> 32) as i32);
    s1_19 = s1_18 + 4;
    sw(s1_19, 6);
    s1_20 = s1_19 + 4;
    sw(s1_20, 0);
    s1_21 = s1_20 + 4;
    if (lbu(s3_1 + 36) & 2) == 0 {
        v0_18 = 0_i64;
    } else {
        v0_18 = lwu(sp + 52) | lwu(sp + 92) << 54 | (lwu(sp + 88) << 34 | lwu(sp + 56) << 20) | (lwu(sp + 84) << 14 | lwu(sp + 60) << 40);
    }
    a0_29 = 0xffffffff_i64;
    sw(s1_21, (((v0_18 & a0_29) << 32) >> 32) as i32);
    s1_22 = s1_21 + 4;
    sw(s1_22, (((((v0_18 as u64) >> 32) as i64 & a0_29) << 32) >> 32) as i32);
    s1_23 = s1_22 + 4;
    sw(s1_23, 52);
    s1_24 = s1_23 + 4;
    sw(s1_24, 0);
    s1_25 = s1_24 + 4;
    if (lbu(s3_1 + 36) & 2) == 0 {
        v0_19 = 0_i64;
    } else if (lbu(s3_1 + 37) as u32) < (4 as u32) {
        v0_19 = 0_i64;
    } else {
        v0_19 = lwu(sp + 64) | lwu(sp + 104) << 54 | (lwu(sp + 100) << 34 | lwu(sp + 68) << 20) | (lwu(sp + 96) << 14 | lwu(sp + 72) << 40);
    }
    a0_30 = 0xffffffff_i64;
    sw(s1_25, (((v0_19 & a0_30) << 32) >> 32) as i32);
    s1_26 = s1_25 + 4;
    a0_31 = lhu(s3_1);
    sw(s1_26, (((((v0_19 as u64) >> 32) as i64 & a0_30) << 32) >> 32) as i32);
    s1_27 = s1_26 + 4;
    v0_20 = lhu(s3_1 + 2);
    sw(s1_27, 54);
    s1_28 = s1_27 + 4;
    sw(s1_28, 0);
    a1_15 = ((a0_31 - 1) as i64) << 14;
    a0_32 = ((v0_20 - 1) as i64) << 34;
    s1_29 = s1_28 + 4;
    a2_4 = if lw(s3_1 + 8) == 0 { 0 } else { 2 };
    if lw(s3_1 + 12) == 0 {
        v0_21 = a2_4 as i64 | a1_15 | a0_32;
    } else {
        v0_21 = a2_4 as i64 | a0_32 | (a1_15 | 8_i64);
    }
    a0_33 = 0xffffffff_i64;
    sw(s1_29, (((v0_21 & a0_33) << 32) >> 32) as i32);
    s1_30 = s1_29 + 4;
    sw(s1_30, (((((v0_21 as u64) >> 32) as i64 & a0_33) << 32) >> 32) as i32);
    s1_31 = s1_30 + 4;
    sw(s1_31, 8);
    sw(s1_31 + 4, 0);
    return;
}

/// Format the time of the clock.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn rtcFormatTime(mut a0: i32) -> i32 {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let mut a1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_3: i32 = 0;
    if a0 == -1 {
        a0_3 = lw(g::rettime_96);
        a1_3 = 0x3afc38;
        sprintf(&[a0_3 as i64, a1_3 as i64]);
        v0_1 = lw(g::rettime_96);
    } else {
        a1_1 = a0;
        a0_1 = sp;
        u322clock(a0_1, a1_1);
        a2_1 = lbu(sp + 5);
        a3_1 = lbu(sp + 6);
        a1_2 = 0x3a9a60;
        a4_1 = lbu(sp + 7);
        a5_1 = lbu(sp + 3);
        a6_1 = lbu(sp + 2);
        a7_1 = lbu(sp + 1);
        a0_2 = lw(g::rettime_96);
        sprintf(&[a0_2 as i64, a1_2 as i64, a2_1 as i64, a3_1 as i64, a4_1 as i64, a5_1 as i64, a6_1 as i64, a7_1 as i64]);
        v0_1 = lw(g::rettime_96);
    }
    return v0_1;
}
