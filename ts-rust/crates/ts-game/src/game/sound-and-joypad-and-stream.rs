//! Functions: the sound loading and playing, the joypads, the controller and the streams.

use crate::*;

/// Run the sound batch.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn soundBatchExec() {
    let mut a2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a3_2: i32 = 0;
    a2_1 = lw(g::soundbatchlen);
    if a2_1 <= 0 {
        return;
    }
    a1_1 = lw(0x3afe24);
    a2_2 = (a2_1 << 3) + 15 & -16;
    a0_1 = 0x1fb19c0;
    a3_1 = 1;
    sceSdTransToIOP(&[a0_1 as i64, a1_1 as i64, a2_2 as i64, a3_1 as i64]);
    a2_3 = lw(0x3afe24);
    a0_2 = 0;
    a4_1 = lw(g::soundbatchlen);
    a1_2 = 0x80b0;
    a3_2 = 0;
    sceSdRemote(&[a0_2 as i64, a1_2 as i64, a2_3 as i64, a3_2 as i64, a4_1 as i64]);
    return;
}

/// Start the sound.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn soundMake() {
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_1: i32 = 0;
    sceSifInitIopHeap(&[]);
    a0_1 = 0x40000;
    v0_1 = sceSifAllocIopHeap(&[a0_1 as i64]);
    sw(0x3afe20, v0_1);
    if v0_1 < 0 {
        a0_2 = -1;
        exit(a0_2);
    }
    a0_3 = 1600;
    v0_2 = sceSifAllocIopHeap(&[a0_3 as i64]);
    sw(0x3afe24, v0_2);
    if v0_2 < 0 {
        a0_4 = -1;
        exit(a0_4);
    }
    sceSdRemoteInit(&[]);
    a0_5 = 1;
    a1_1 = 0x8000;
    a2_1 = 0;
    sceSdRemote(&[a0_5 as i64, a1_1 as i64, a2_1 as i64]);
    a0_6 = 1;
    a1_2 = 0x8070;
    a2_2 = 10;
    a3_1 = 2048;
    sceSdRemote(&[a0_6 as i64, a1_2 as i64, a2_2 as i64, a3_1 as i64]);
    return;
}

/// Load the sound.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn soundLoad(mut a0: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mut s2_1: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a3_3: i32 = 0;
    'b1: {
        s2_1 = g::sounds;
        if lw(s2_1 + a0 * 36 + 24) == 0 {
            'b2: {
                s2_2 = a0 * 36 + (s2_1 + 4);
                a0_1 = lw(s2_2);
                v0_1 = fileSize(a0_1);
                s0_1 = g::scratchbuffer;
                s1_1 = v0_1 + 63 & -64;
                a0_2 = s0_1;
                a1_1 = s1_1;
                s3_1 = v0_1 - 64;
                sceSifWriteBackDCache(&[a0_2 as i64, a1_1 as i64]);
                s5_1 = s0_1 + 64;
                a1_2 = s1_1 + s0_1;
                a0_3 = s0_1;
                InvalidDCache(a0_3, a1_2);
                a0_4 = lw(s2_2);
                a1_3 = s0_1;
                a2_1 = v0_1;
                fileLoadTo(a0_4, a1_3, a2_1);
                if (s3_1 & 63) != 0 {
                    s1_2 = v0_1 - 80 + s5_1;
                    'l3: loop {
                        a0_5 = s1_2;
                        a1_4 = s5_1 + s3_1;
                        a2_2 = 16;
                        s3_1 = s3_1 + 16;
                        bcopy(a0_5, a1_4, a2_2);
                        s1_2 = s1_2 + 16;
                        if (s3_1 & 63) == 0 {
                            break 'b2;
                        }
                    }
                }
            }
            a0_6 = s5_1;
            a1_5 = s3_1;
            sceSifWriteBackDCache(&[a0_6 as i64, a1_5 as i64]);
            a1_6 = lw(0x3afe20);
            a0_7 = s5_1;
            a2_3 = s3_1;
            a3_1 = 1;
            sceSdTransToIOP(&[a0_7 as i64, a1_6 as i64, a2_3 as i64, a3_1 as i64]);
            a5_1 = lw(g::thissoundbuf);
            v1_1 = g::sounds;
            a4_1 = lw(0x3afe20);
            a6_1 = s3_1;
            a0_8 = 1;
            a1_7 = 0x80d0;
            a7_1 = a0 * 36 + (v1_1 + 24);
            sw(a7_1, lw(a5_1 + 8));
            a2_4 = 1;
            sw(v1_1 + a0 * 36 + 32, s3_1);
            a3_2 = 0;
            v1_2 = lw(a5_1 + 12) - s3_1;
            sw(a5_1 + 8, lw(a5_1 + 8) + s3_1);
            sw(a5_1 + 12, v1_2);
            a5_2 = lw(a7_1);
            sceSdRemote(&[a0_8 as i64, a1_7 as i64, a2_4 as i64, a3_2 as i64, a4_1 as i64, a5_2 as i64, a6_1 as i64, a7_1 as i64]);
            a0_9 = 1;
            a1_8 = 0x80f0;
            a2_5 = 1;
            a3_3 = 1;
            sceSdRemote(&[a0_9 as i64, a1_8 as i64, a2_5 as i64, a3_3 as i64]);
            sw(sp, 0);
            if 0xf423f >= lw(sp) {
                'l4: loop {
                    sw(sp + 4, lw(sp) * lw(sp));
                    sw(sp, lw(sp) + 1);
                    if 0xf423f < lw(sp) {
                        break 'b1;
                    }
                }
            }
        }
    }
    return;
}

/// Start a range of sounds.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn soundStartRange(mut a0: i32, mut a1: i32, mut a2: i32, mut f12: f32) -> i32 {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let mut s0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut f12_1: f32 = 0.0;
    let mut a4_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut v0_3: i32 = 0;
    'b1: {
        'b2: {
            s0_1 = a0 * 36 + g::sounds;
            if lw(g::gv_playsound) != 0 {
                'b3: {
                    v1_1 = lw(g::tickframenum);
                    a2_1 = 0;
                    if v1_1 == lw(g::lastsoundframenum_160) {
                        a2_1 = lw(g::numsounds_161);
                        a1_1 = 0;
                        a4_1 = a2_1;
                        if a2_1 > 0 {
                            v0_1 = 0x1fb1600;
                            'l4: loop {
                                a0_1 = lw((a1_1 << 2) + v0_1);
                                a1_1 = a1_1 + 1;
                                if a0_1 == a0 {
                                    break 'b2;
                                }
                                v0_1 = 0x1fb1600;
                                if a1_1 >= a4_1 {
                                    break 'b3;
                                }
                            }
                        }
                    } else {
                        sw(g::lastsoundframenum_160, v1_1);
                        sw(g::numsounds_161, 0);
                    }
                }
                v1_2 = a2_1 << 2;
                if a2_1 < 100 {
                    sw(g::numsounds_161, a2_1 + 1);
                    sw(v1_2 + 0x1fb1600, a0);
                }
                v1_3 = lw(g::sound_voice);
                if lw(s0_1 + 24) != 0 {
                    v0_2 = lw(g::sound_core);
                    if v1_3 < 0 {
                        a0_2 = sp;
                        a1_2 = sp | 4;
                        getNextChannel(a0_2, a1_2);
                    } else {
                        sw(sp + 4, v1_3);
                        sw(sp, v0_2);
                    }
                    a1_3 = a1;
                    f12_1 = f12;
                    a4_2 = a2;
                    a0_3 = s0_1;
                    a2_2 = sp | 8;
                    a3_1 = sp | 12;
                    soundCalcVol(a0_3, a1_3, a2_2, a3_1, a4_2, f12_1);
                    a0_4 = 1;
                    a2_3 = lw(sp + 8);
                    a1_4 = (lhu(sp) | lhu(sp + 4) << 1) & 0xffff;
                    soundBatchAdd(a0_4, a1_4, a2_3);
                    a0_5 = 1;
                    a2_4 = lw(sp + 12);
                    a1_5 = (lhu(sp) | lhu(sp + 4) << 1 | 256) & 0xffff;
                    soundBatchAdd(a0_5, a1_5, a2_4);
                    a0_6 = 1;
                    a2_5 = lhu(s0_1 + 18);
                    a1_6 = (lhu(sp) | lhu(sp + 4) << 1 | 512) & 0xffff;
                    soundBatchAdd(a0_6, a1_6, a2_5);
                    a0_7 = 1;
                    a1_7 = (lhu(sp) | lhu(sp + 4) << 1 | 768) & 0xffff;
                    a2_6 = (lhu(s0_1 + 8) & 127) << 8 | lhu(s0_1 + 14) & 15 | ((lhu(s0_1 + 10) & 15) << 4 | 0x8000);
                    soundBatchAdd(a0_7, a1_7, a2_6);
                    a0_8 = 1;
                    a1_8 = (lhu(sp) | lhu(sp + 4) << 1 | 1024) & 0xffff;
                    a2_7 = (lhu(s0_1 + 12) & 127) << 6 | (lhu(s0_1 + 16) & 31 | 0xc020);
                    soundBatchAdd(a0_8, a1_8, a2_7);
                    a0_9 = 3;
                    a2_8 = lw(s0_1 + 24);
                    a1_9 = (lhu(sp) | lhu(sp + 4) << 1 | 0x2040) & 0xffff;
                    soundBatchAdd(a0_9, a1_9, a2_8);
                    a0_10 = 2;
                    a1_10 = lhu(sp) | 0x1500;
                    a2_9 = 1 << (lw(sp + 4) & 31);
                    soundBatchAdd(a0_10, a1_10, a2_9);
                    v0_3 = lw(sp) | lw(sp + 4) << 1;
                    break 'b1;
                }
            }
        }
        v0_3 = -1;
    }
    return v0_3;
}

/// Restart the sounds.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn soundRestart() {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let mut v0_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a3_8: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a0_10: i32 = 0;
    v0_1 = 0x1fb1870;
    s0_1 = 9;
    'l1: loop {
        sw(v0_1, 0);
        s0_1 = s0_1 - 1;
        sh(v0_1 + 10, 0);
        v0_1 = v0_1 + 32;
        if s0_1 < 0 {
            break;
        }
    }
    sw(g::transinprogress, 0);
    sw(0x3afe28, 0);
    s0_2 = 0;
    'l3: loop {
        a2_1 = s0_2 | 0x1600;
        a0_1 = 1;
        a1_1 = 0x8030;
        a3_1 = 0xffffff;
        sceSdRemote(&[a0_1 as i64, a1_1 as i64, a2_1 as i64, a3_1 as i64]);
        a2_2 = s0_2 | 0x1d00;
        a0_2 = 1;
        a3_2 = if s0_2 != 0 { 0x1fffff } else { 0x1dffff };
        a1_2 = 0x8050;
        sceSdRemote(&[a0_2 as i64, a1_2 as i64, a2_2 as i64, a3_2 as i64]);
        a2_3 = s0_2 | 2;
        a0_3 = 1;
        a1_3 = 0x8070;
        a3_3 = 1;
        sceSdRemote(&[a0_3 as i64, a1_3 as i64, a2_3 as i64, a3_3 as i64]);
        a2_4 = s0_2 | 2944;
        a0_4 = 1;
        a1_4 = 0x8010;
        a3_4 = 0;
        sceSdRemote(&[a0_4 as i64, a1_4 as i64, a2_4 as i64, a3_4 as i64]);
        a2_5 = s0_2 | 3200;
        a0_5 = 1;
        a1_5 = 0x8010;
        a3_5 = 0;
        sceSdRemote(&[a0_5 as i64, a1_5 as i64, a2_5 as i64, a3_5 as i64]);
        a2_6 = s0_2 | 2432;
        a0_6 = 1;
        a1_6 = 0x8010;
        a3_6 = 0x3fff;
        sceSdRemote(&[a0_6 as i64, a1_6 as i64, a2_6 as i64, a3_6 as i64]);
        a2_7 = s0_2 | 2688;
        a0_7 = 1;
        a1_7 = 0x8010;
        a3_7 = 0x3fff;
        sceSdRemote(&[a0_7 as i64, a1_7 as i64, a2_7 as i64, a3_7 as i64]);
        a2_8 = s0_2;
        sh(sp + 8, 0);
        sh(sp + 10, 0);
        a0_8 = 1;
        sw(sp + 4, 261);
        a1_8 = 0x8130;
        a3_8 = sp;
        s0_2 = s0_2 + 1;
        sceSdRemote(&[a0_8 as i64, a1_8 as i64, a2_8 as i64, a3_8 as i64]);
        if s0_2 >= 2 {
            break;
        }
    }
    s0_3 = 0;
    s1_1 = g::sounds + 22;
    'l5: loop {
        'b7: {
            a0_9 = lhu(s1_1);
            if (a0_9 & 16) != 0 {
                v1_1 = lw(g::lvlevelnum);
                if (a0_9 & 8) != 0 {
                    if (((a0_9 as u32) >> 8) as i32) != v1_1 {
                        s0_3 = s0_3 + 1;
                        break 'b7;
                    }
                    a0_10 = s0_3;
                    soundAddPreLoad(a0_10);
                }
            }
            s0_3 = s0_3 + 1;
        }
        s1_1 = s1_1 + 36;
        if s0_3 >= 1215 {
            break;
        }
    }
    sw(g::announcecountdown, -1);
    sw(g::lastannounce, -1);
    return;
}

/// Save the sound data.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn sounddataSave() {
    let frame = Frame::new(1216);
    let sp = frame.sp();
    let mut a0_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a7_2: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut t4_1: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a7_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a5_3: i32 = 0;
    let mut a6_3: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a7_4: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a0_14: i32 = 0;
    a0_1 = 0x100000;
    s3_1 = 0x3998b0;
    v0_1 = malloc(a0_1);
    if v0_1 != 0 {
        a0_2 = v0_1;
        a1_1 = 0x3998c8;
        s2_1 = 0;
        sprintf(&[a0_2 as i64, a1_1 as i64]);
        a0_3 = v0_1;
        v0_2 = strlen(a0_3);
        s0_1 = v0_1 + v0_2;
        'l2: loop {
            'b4: {
                a2_1 = s2_1 * 36 + g::sounds;
                a3_1 = lhu(a2_1 + 22);
                a7_1 = a2_1 + 8;
                if (a3_1 & 8) == 0 {
                    a0_7 = s0_1;
                    if (a3_1 & 16) == 0 {
                        a1_3 = 0x3999d8;
                        if (a3_1 & 4) == 0 {
                            a0_7 = s0_1;
                            if (a3_1 & 1) == 0 {
                                a1_3 = 0x399a58;
                                if (a3_1 & 2) == 0 {
                                    a0_8 = s0_1;
                                    a4_3 = lh(a2_1 + 8);
                                    a1_4 = 0x399a98;
                                    a5_3 = lh(a7_1 + 2);
                                    a6_3 = lh(a7_1 + 4);
                                    v1_4 = lhu(a2_1 + 18);
                                    v0_4 = lhu(a2_1 + 20);
                                    a3_4 = lw(a2_1 + 4);
                                    a7_4 = lh(a7_1 + 6);
                                    a2_4 = lw(a2_1);
                                    sw(sp, lh(a7_1 + 8));
                                    sw(sp + 8, v1_4);
                                    sw(sp + 16, v0_4);
                                    sprintf(&[a0_8 as i64, a1_4 as i64, a2_4 as i64, a3_4 as i64, a4_3 as i64, a5_3 as i64, a6_3 as i64, a7_4 as i64]);
                                    a0_6 = s0_1;
                                    break 'b4;
                                }
                                t4_1 = lh(a7_1 + 8);
                                a4_2 = lh(a2_1 + 8);
                            } else {
                                t4_1 = lh(a7_1 + 8);
                                a4_2 = lh(a2_1 + 8);
                                a1_3 = 0x399a18;
                            }
                        } else {
                            t4_1 = lh(a7_1 + 8);
                            a4_2 = lh(a2_1 + 8);
                        }
                    } else {
                        t4_1 = lh(a7_1 + 8);
                        a4_2 = lh(a2_1 + 8);
                        a1_3 = 0x399998;
                    }
                    a5_2 = lh(a7_1 + 2);
                    a6_2 = lh(a7_1 + 4);
                    v1_3 = lhu(a2_1 + 18);
                    v0_3 = lhu(a2_1 + 20);
                    a3_3 = lw(a2_1 + 4);
                    a7_3 = lh(a7_1 + 6);
                    a2_3 = lw(a2_1);
                    sw(sp, t4_1);
                    sw(sp + 8, v1_3);
                    sw(sp + 16, v0_3);
                    sprintf(&[a0_7 as i64, a1_3 as i64, a2_3 as i64, a3_3 as i64, a4_2 as i64, a5_2 as i64, a6_2 as i64, a7_3 as i64]);
                    a0_6 = s0_1;
                } else {
                    a0_4 = lhu(a2_1 + 18);
                    v1_1 = lhu(a2_1 + 20);
                    sw(sp, lh(a7_1 + 8));
                    sw(sp + 8, a0_4);
                    sw(sp + 16, v1_1);
                    a5_1 = lh(a7_1 + 2);
                    a6_1 = lh(a7_1 + 4);
                    a4_1 = lh(a2_1 + 8);
                    a7_2 = lh(a7_1 + 6);
                    if (a3_1 & 16) == 0 {
                        v1_2 = g::snd + 8;
                    } else {
                        v1_2 = 0x399980;
                    }
                    a1_2 = 0x399938;
                    a3_2 = lw(a2_1 + 4);
                    a2_2 = lw(a2_1);
                    a0_5 = s0_1;
                    sw(sp + 24, v1_2);
                    sw(sp + 32, (((a3_1 & 0xffff) as u32) >> 8) as i32);
                    sprintf(&[a0_5 as i64, a1_2 as i64, a2_2 as i64, a3_2 as i64, a4_1 as i64, a5_1 as i64, a6_1 as i64, a7_2 as i64]);
                    a0_6 = s0_1;
                }
            }
            s2_1 = s2_1 + 1;
            v0_5 = strlen(a0_6);
            s0_1 = s0_1 + v0_5;
            if s2_1 >= 1215 {
                break;
            }
        }
        a0_9 = s0_1;
        a1_5 = 0x3ae340;
        sprintf(&[a0_9 as i64, a1_5 as i64]);
        a0_10 = s0_1;
        v0_6 = strlen(a0_10);
        s0_2 = s0_1 + v0_6;
        a1_6 = 0x3ae348;
        a0_11 = sp + 48;
        a2_5 = s3_1;
        sprintf(&[a0_11 as i64, a1_6 as i64, a2_5 as i64]);
        a0_12 = s3_1;
        v0_7 = fileSize(a0_12);
        a0_13 = s3_1;
        if v0_7 > 0 {
            a1_7 = sp + 48;
            fileCopy(a0_13, a1_7);
            a0_13 = s3_1;
        }
        a2_6 = s0_2 - v0_1;
        a1_8 = v0_1;
        fileSave(a0_13, a1_8, a2_6);
        a0_14 = v0_1;
        free(a0_14);
    }
    return;
}

/// Tick of the multitap.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn joyMtapTick() {
    let mut v1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut hi_1: i32 = 0;
    'b1: {
        v1_1 = lw(g::mtappollcount);
        if v1_1 != 0 {
            if v1_1 != 60 {
                break 'b1;
            }
            a0_2 = 1;
            v0_2 = scePadGetSlotMax(&[a0_2 as i64]);
            sw(g::max_slots + 4, v0_2);
        } else {
            a0_1 = 0;
            v0_1 = scePadGetSlotMax(&[a0_1 as i64]);
            sw(g::max_slots, v0_1);
        }
        v1_1 = lw(g::mtappollcount);
    }
    hi_1 = (v1_1 + 1).wrapping_rem(120);
    sw(g::mtappollcount, hi_1);
    return;
}

/// Tick of the joypads.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn joyTick() {
    let mut s0_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    joyMtapTick();
    sw(g::joy_justdisconnected, 0);
    s0_1 = g::controllers;
    s1_1 = s0_1 + 3072;
    sw(s0_1 + 340, 0);
    'l1: loop {
        a0_1 = s0_1;
        controllerTick(a0_1);
        s0_1 = s0_1 + 384;
        if s0_1 >= s1_1 {
            break;
        }
        sw(s0_1 + 340, 0);
    }
    cheatTick();
    replayTick();
    return;
}

/// Start the joypads.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn joyMake() {
    let mut a0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_10: i32 = 0;
    sceMtapInit(&[]);
    a0_1 = 0;
    sceMtapPortOpen(&[a0_1 as i64]);
    a0_2 = 1;
    sceMtapPortOpen(&[a0_2 as i64]);
    a0_3 = 2;
    sceMtapPortOpen(&[a0_3 as i64]);
    a0_4 = 3;
    sceMtapPortOpen(&[a0_4 as i64]);
    a0_5 = 0;
    scePadInit(&[a0_5 as i64]);
    s0_1 = g::controllers;
    s1_1 = s0_1 + 3072;
    'l1: loop {
        a0_6 = s0_1;
        controllerOpen(a0_6);
        s0_1 = s0_1 + 384;
        if s0_1 >= s1_1 {
            break;
        }
    }
    a0_7 = 0;
    v0_1 = sceMtapGetConnection(&[a0_7 as i64]);
    if v0_1 == 0 {
        sw(g::max_slots, 1);
    } else {
        a0_8 = 0;
        v0_2 = scePadGetSlotMax(&[a0_8 as i64]);
        sw(g::max_slots, v0_2);
    }
    a0_9 = 1;
    v0_3 = sceMtapGetConnection(&[a0_9 as i64]);
    v0_4 = 1;
    if v0_3 != 0 {
        a0_10 = 1;
        v0_4 = scePadGetSlotMax(&[a0_10 as i64]);
    }
    sw(g::max_slots + 4, v0_4);
    return;
}

/// Open the controller.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn controllerOpen(mut a0: i32) {
    let mut a3_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    a3_1 = a0;
    v1_1 = 5;
    v0_1 = a0 + 325;
    'l1: loop {
        sb(v0_1, 0);
        v1_1 = v1_1 - 1;
        v0_1 = v0_1 - 1;
        if v1_1 < 0 {
            break;
        }
    }
    a2_1 = a0;
    a0_1 = lw(a0 + 256);
    a1_1 = lw(a0 + 260);
    swc1(a0 + 316, 0.25_f32);
    v0_2 = scePadPortOpen(&[a0_1 as i64, a1_1 as i64, a2_1 as i64, a3_1 as i64]);
    if v0_2 != 0 {
        return;
    }
    a0_2 = 0;
    exit(a0_2);
    return;
}

/// Tick of the controller.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn controllerTick(mut a0: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let mut a2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a0_16: i32 = 0;
    a2_1 = lw(a0 + 256);
    a1_1 = lw(a0 + 260);
    sw(a0 + 268, lw(a0 + 264));
    if lw((a2_1 << 2) + g::max_slots) >= a1_1 {
        a0_1 = a2_1;
        v0_1 = scePadGetState(&[a0_1 as i64, a1_1 as i64, a2_1 as i64]);
        sw(a0 + 264, v0_1);
    } else {
        sw(a0 + 264, 0);
    }
    if lw(a0 + 264) != 0 {
        v1_1 = lw(a0 + 272);
    } else {
        if lw(a0 + 268) != 0 {
            sw(g::joy_justdisconnected, 1);
        }
        sw(a0 + 272, 0);
        v1_1 = lw(a0 + 272);
    }
    'b4: {
        'b5: {
            v0_2 = (v1_1 < 71) as i32;
            if v1_1 == 70 {
                a0_11 = lw(a0 + 256);
                a2_6 = -1;
                a1_10 = lw(a0 + 260);
                a3_5 = 0;
                v0_13 = scePadInfoAct(&[a0_11 as i64, a1_10 as i64, a2_6 as i64, a3_5 as i64]);
                if v0_13 == 0 {
                    sw(a0 + 272, 99);
                }
                sb(a0 + 326, 0);
                a2_7 = a0 + 326;
                a0_12 = lw(a0 + 256);
                a1_11 = lw(a0 + 260);
                sb(a0 + 327, 1);
                sb(a0 + 331, 255);
                sb(a0 + 328, 255);
                sb(a0 + 329, 255);
                sb(a0 + 330, 255);
                v0_14 = scePadSetActAlign(&[a0_12 as i64, a1_11 as i64, a2_7 as i64]);
                if v0_14 == 0 {
                    v1_1 = lw(a0 + 272);
                    break 'b4;
                }
                v0_5 = lw(a0 + 272);
            } else {
                if v0_2 == 0 {
                    if v1_1 == 71 {
                        a0_9 = lw(a0 + 256);
                        a1_8 = lw(a0 + 260);
                        v0_11 = scePadGetReqState(&[a0_9 as i64, a1_8 as i64, a2_1 as i64]);
                        if v0_11 == 1 {
                            sw(a0 + 272, 70);
                        }
                        a0_10 = lw(a0 + 256);
                        a1_9 = lw(a0 + 260);
                        v0_12 = scePadGetReqState(&[a0_10 as i64, a1_9 as i64]);
                        if v0_12 != 0 {
                            v1_1 = lw(a0 + 272);
                            break 'b4;
                        }
                        v1_1 = 99;
                        sw(a0 + 272, 99);
                        break 'b4;
                    }
                    break 'b4;
                }
                v0_3 = (v1_1 < 42) as i32;
                if v1_1 != 41 {
                    if v0_3 == 0 {
                        if v1_1 != 42 {
                            break 'b4;
                        }
                        a0_6 = lw(a0 + 256);
                        a1_5 = lw(a0 + 260);
                        v0_8 = scePadGetReqState(&[a0_6 as i64, a1_5 as i64, a2_1 as i64]);
                        if v0_8 == 1 {
                            sw(a0 + 272, 41);
                        }
                        a0_7 = lw(a0 + 256);
                        a1_6 = lw(a0 + 260);
                        v0_9 = scePadGetReqState(&[a0_7 as i64, a1_6 as i64]);
                        if v0_9 != 0 {
                            v1_1 = lw(a0 + 272);
                            break 'b4;
                        }
                        sw(a0 + 272, 0);
                    } else {
                        if v1_1 == 0 {
                            a0_3 = lw(a0 + 264);
                            if a0_3 != 6 {
                                if a0_3 != 2 {
                                    break 'b4;
                                }
                            }
                            a0_4 = lw(a0 + 256);
                            a2_3 = 1;
                            a1_3 = lw(a0 + 260);
                            a3_2 = 0;
                            v0_6 = scePadInfoMode(&[a0_4 as i64, a1_3 as i64, a2_3 as i64, a3_2 as i64]);
                            a2_4 = 2;
                            if v0_6 == 0 {
                                v1_1 = lw(a0 + 272);
                                break 'b4;
                            }
                            a0_5 = lw(a0 + 256);
                            a1_4 = lw(a0 + 260);
                            a3_3 = 0;
                            v0_7 = scePadInfoMode(&[a0_5 as i64, a1_4 as i64, a2_4 as i64, a3_3 as i64]);
                            s1_1 = if v0_7 >= 1 { v0_7 } else { v0_6 };
                            if s1_1 == 4 {
                                v1_1 = 40;
                                sw(a0 + 272, 40);
                                break 'b4;
                            }
                            if s1_1 == 7 {
                                sw(a0 + 272, 70);
                                v1_1 = 70;
                                break 'b4;
                            }
                            v1_1 = lw(a0 + 272);
                            break 'b4;
                        }
                        if v1_1 != 40 {
                            break 'b4;
                        }
                        a0_2 = lw(a0 + 256);
                        a2_2 = 2;
                        a1_2 = lw(a0 + 260);
                        a3_1 = 0;
                        v0_4 = scePadInfoMode(&[a0_2 as i64, a1_2 as i64, a2_2 as i64, a3_1 as i64]);
                        if v0_4 != 0 {
                            v0_5 = lw(a0 + 272);
                            break 'b5;
                        }
                        sw(a0 + 272, 0);
                    }
                    v1_1 = 0;
                    break 'b4;
                }
                a0_8 = lw(a0 + 256);
                a2_5 = 1;
                a1_7 = lw(a0 + 260);
                a3_4 = 3;
                v0_10 = scePadSetMainMode(&[a0_8 as i64, a1_7 as i64, a2_5 as i64, a3_4 as i64]);
                if v0_10 != 1 {
                    v1_1 = lw(a0 + 272);
                    break 'b4;
                }
                v0_5 = lw(a0 + 272);
            }
        }
        v0_15 = v0_5 + 1;
        v1_1 = v0_15;
        sw(a0 + 272, v0_15);
    }
    'b11: {
        if v1_1 == 99 {
            a2_8 = sp;
            a0_13 = lw(a0 + 256);
            a1_12 = lw(a0 + 260);
            sw(a0 + 284, lw(a0 + 288));
            v0_16 = scePadRead(&[a0_13 as i64, a1_12 as i64, a2_8 as i64]);
            v0_17 = lbu(sp + 2);
            if v0_16 <= 0 {
                sw(a0 + 272, 0);
                v1_4 = lw(a0 + 284);
            } else {
                'b14: {
                    a2_9 = lbu(sp + 6);
                    a0_14 = lbu(sp + 7);
                    a1_13 = lbu(sp + 4);
                    v1_2 = lbu(sp + 5);
                    a4_1 = lbu(sp + 1);
                    a3_6 = lw(a0 + 280);
                    sw(a0 + 288, (v0_17 << 8 | lbu(sp + 3)) ^ 0xffff);
                    sw(a0 + 300, a2_9);
                    sw(a0 + 308, a0_14);
                    sw(a0 + 304, a1_13);
                    sw(a0 + 312, v1_2);
                    sw(a0 + 276, a4_1);
                    if a3_6 != 0 {
                        if a4_1 == a3_6 {
                            v0_18 = lw(a0 + 276);
                            break 'b14;
                        }
                        sw(a0 + 280, 0);
                        sw(a0 + 272, 0);
                    }
                    v0_18 = lw(a0 + 276);
                }
                v1_3 = lbu(a0 + 320);
                sw(a0 + 280, v0_18);
                if v1_3 != 0 {
                    sb(a0 + 320, v1_3 + 128);
                }
                v0_19 = lbu(a0 + 321);
                if v0_19 != 0 {
                    sb(a0 + 321, v0_19 + 128);
                }
                a0_15 = lw(a0 + 256);
                a2_10 = a0 + 320;
                a1_14 = lw(a0 + 260);
                scePadSetActDirect(&[a0_15 as i64, a1_14 as i64, a2_10 as i64, a3_6 as i64, a4_1 as i64]);
                v1_4 = lw(a0 + 284);
            }
            v0_20 = lw(a0 + 288);
            v0_21 = v0_20 & !(0 | v1_4);
            sw(a0 + 296, v1_4 & !(0 | v0_20));
            sw(a0 + 292, v0_21);
            if (v0_21 & 0xf0f0) != 0 {
                a0_16 = lw(g::cycbufpos);
                sh((a0_16 << 1) + g::cycbuf, lhu(a0 + 288) & 0xf0ff);
                sw(g::cycbufpos, (a0_16 + 1).wrapping_rem(20));
            }
            if lw(a0 + 272) == 99 {
                break 'b11;
            }
        }
        sw(a0 + 292, 0);
        sw(a0 + 312, 128);
        sw(a0 + 296, 0);
        sw(a0 + 288, 0);
        sw(a0 + 300, 128);
        sw(a0 + 308, 128);
        sw(a0 + 304, 128);
    }
    return;
}

/// Start the stream RPC.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn stream_RPCinit() -> i32 {
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut v0_3: i32 = 0;
    'b3: {
        'l1: loop {
            s0_1 = 0x1fb2040;
            a0_1 = s0_1;
            a1_1 = 0x534a4521;
            a2_1 = 0;
            v0_1 = sceSifBindRpc(&[a0_1 as i64, a1_1 as i64, a2_1 as i64]);
            if v0_1 < 0 {
                break;
            }
            v0_3 = lw(s0_1 + 36);
            v0_2 = 0;
            if v0_3 != 0 {
                break 'b3;
            }
        }
        v0_2 = -1;
    }
    return v0_2;
}

/// Stream RPC.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn stream_RPC(mut a0: i32, mut a1: i32) -> i32 {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let mut a1_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut a7_2: i32 = 0;
    if (a0 & 0xfff0) != 0x8020 {
        sw(0x1fb2000, a1);
        a3_2 = 0x1fb2000;
        a1_2 = a0;
        a0_2 = 0x1fb2040;
        sw(sp, 0);
        a2_2 = 0;
        a4_2 = 16;
        a5_2 = a3_2;
        a6_2 = 64;
        a7_2 = 0;
        sceSifCallRpc(&[a0_2 as i64, a1_2 as i64, a2_2 as i64, a3_2 as i64, a4_2 as i64, a5_2 as i64, a6_2 as i64, a7_2 as i64]);
        v0_1 = lw(0x1fb2000);
    } else {
        a1_1 = a0;
        a3_1 = a1;
        a0_1 = 0x1fb2040;
        sw(sp, 0);
        a2_1 = 0;
        a4_1 = 64;
        a5_1 = 0x1fb2000;
        a6_1 = 64;
        a7_1 = 0;
        sceSifCallRpc(&[a0_1 as i64, a1_1 as i64, a2_1 as i64, a3_1 as i64, a4_1 as i64, a5_1 as i64, a6_1 as i64, a7_1 as i64]);
        v0_1 = lw(0x1fb2000);
    }
    return v0_1;
}
