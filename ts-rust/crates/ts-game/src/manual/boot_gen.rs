//! Boot flow ported from the PS2 executable (threads and GS/VBL interrupt machinery removed).
#![allow(unused_variables, unused_mut, unused_assignments, unreachable_code, non_snake_case, dead_code, unused_parens, unused_braces, unused_labels, arithmetic_overflow)]
use crate::*;

/// Game entry point (`main` in the original executable).
pub fn main_(mut a0: i32, mut a1: i32) {
    let _frame = Frame::new(224);
    let sp = _frame.sp();
    let mut s7_1: i32 = 0;
    let mut fp_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut bc_1: bool = false;
    let mut v0_4: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut s5_2: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut a0_27: i32 = 0;
    s7_1 = a0;
    fp_1 = a1;
    s0_1 = 1244;
    sw(sp + 48, (s7_1 < 2) as i32);
    __main();
    s5_1 = g::spanish + 4;
    s4_1 = g::german + 4;
    s3_1 = g::italian + 4;
    s2_1 = g::french + 4;
    s1_1 = g::english + 4;
    a0_1 = lw(s1_1);
    'l1: loop {
        s0_1 = s0_1 - 1;
        s1_1 = s1_1 + 4;
        checkstring(a0_1);
        a0_2 = lw(s2_1);
        s2_1 = s2_1 + 4;
        checkstring(a0_2);
        a0_3 = lw(s3_1);
        s3_1 = s3_1 + 4;
        checkstring(a0_3);
        a0_4 = lw(s4_1);
        s4_1 = s4_1 + 4;
        checkstring(a0_4);
        a0_5 = lw(s5_1);
        s5_1 = s5_1 + 4;
        checkstring(a0_5);
        if s0_1 < 0 {
            break;
        }
        a0_1 = lw(s1_1);
    }
    if lw(sp + 48) != 0 {
        s7_1 = 3;
        fp_1 = g::defaultargs;
    }
    sw(g::myargc, s7_1);
    s0_2 = 0x1fac580;
    sw(g::myargv, fp_1);
    bossMakeAll();
    if s7_1 >= 2 {
        // Direct level launch (port debugging aid): the front end normally signs a player profile in first.
        let name = ts_sdk::malloc(16);
        for (i, b) in b"DEBUG\0".iter().enumerate() {
            sb(name + i as i32, *b as i32);
        }
        let profile = signonNew(name);
        for player in 0..4 {
            signonSet(player, profile);
        }
    }
    v0_1 = rtcGetTime();
    a0_6 = v0_1;
    newrndseed(a0_6);
    a0_7 = s0_2;
    a1_1 = 0;
    a2_1 = 112;
    memset(a0_7, a1_1, a2_1);
    sw(s0_2 + 64, 104);
    sw(s0_2 + 72, 3);
    sw(s0_2 + 76, 1);
    sw(s0_2 + 80, 2);
    sw(0x1fac580, 1);
    a2_2 = lw(fp_1 + 4);
    sw(s0_2 + 4, 1);
    sw(s0_2 + 88, 0);
    sw(s0_2 + 92, 0);
    v0_2 = (s7_1 < 3) as i32;
    if lb(a2_2) != 45 {
        a0_8 = a2_2;
        v0_3 = lvGetNumFromName(a0_8);
        sw(s0_2 + 64, v0_3);
        v0_2 = (s7_1 < 3) as i32;
    }
    bc_1 = v0_2 != 0;
    v0_4 = lw(0x1fac580);
    if !bc_1 {
        a0_9 = lw(fp_1 + 8);
        v0_4 = lw(0x1fac580);
        if lb(a0_9) != 45 {
            v0_5 = atoi(a0_9);
            sw(s0_2 + 4, v0_5);
            sw(0x1fac580, v0_5);
            v0_4 = lw(0x1fac580);
        }
    }
    s0_3 = 0x1fac580;
    if ((v0_4 - 1) as u32) >= (4 as u32) {
        a0_10 = 0;
        exit(a0_10);
    }
    sw(g::bosscammode, 0);
    sw(g::bosscampropnum, 0);
    sw(s0_3 + 8, 0);
    if s7_1 >= 4 {
        a0_11 = lw(fp_1 + 12);
        if lb(a0_11) != 45 {
            v0_6 = atoi(a0_11);
            sw(s0_3 + 8, v0_6);
        }
    }
    a0_12 = 0x3adb80;
    v0_7 = bossCheckParm(a0_12);
    if v0_7 != 0 {
        v0_8 = bossGetNextParm();
        a0_13 = v0_8;
        v0_9 = atoi(a0_13);
        sw(0x1fac584, v0_9);
        ilinkConnect();
        a0_14 = 0;
        newrndseed(a0_14);
    }
    'b9: {
        'b10: {
            a0_15 = 0x3838e8;
            v0_10 = bossCheckParm(a0_15);
            v0_11 = 0x1fac580;
            if v0_10 != 0 {
                v0_12 = bossIsNextParm();
                v0_11 = 0x1fac580;
                if v0_12 != 0 {
                    v0_13 = bossGetNextParm();
                    a0_16 = v0_13;
                    v0_14 = atoi(a0_16);
                    sw(g::bosscampropnum, v0_14);
                    v0_15 = bossIsNextParm();
                    if v0_15 == 0 {
                        a0_17 = 0;
                        exit(a0_17);
                    }
                    'b12: {
                        'b13: {
                            v0_16 = bossGetNextParm();
                            v1_1 = lb(v0_16);
                            if v1_1 != 49 {
                                if v1_1 >= 50 {
                                    if v1_1 == 98 {
                                        break 'b13;
                                    }
                                    v0_11 = 0x1fac580;
                                    if v1_1 != 102 {
                                        break 'b10;
                                    }
                                } else {
                                    v0_11 = 0x1fac580;
                                    if v1_1 != 48 {
                                        a2_3 = 0x1fb0000;
                                        break 'b9;
                                    }
                                }
                                v0_17 = 4;
                                break 'b12;
                            }
                        }
                        v0_17 = 6;
                    }
                    sw(g::bosscammode, v0_17);
                    v0_11 = 0x1fac580;
                }
            }
        }
        a2_3 = 0x1fb0000;
    }
    'b15: {
        a1_2 = 0;
        a0_18 = lw(0x1fac580);
        if lw(v0_11 + 8) <= 0 {
            s5_2 = g::_gp;
        } else {
            s5_2 = g::_gp;
            'l16: loop {
                v0_18 = 0x1fac580;
                if (a0_18 & 1) != 0 {
                    v1_2 = 8;
                } else {
                    v0_18 = a2_3 - 0x3a80;
                    v1_2 = 54;
                }
                sb(v0_18 + a0_18 + 21, v1_2);
                if a0_18 == 0 {
                    sb(0x1fac595, 69);
                }
                a1_2 = a1_2 + 1;
                a0_18 = a0_18 + 1;
                a2_3 = 0x1fb0000;
                if a1_2 >= lw(0x1fac588) {
                    break 'b15;
                }
            }
        }
    }
    a0_19 = 0x1fac580;
    gameDeferedReset(a0_19);
    a1_3 = 0x1eb1f80;
    v1_3 = 0x1e91f80;
    if (v1_3 as u32) >= (a1_3 as u32) {
        a1_4 = 0x1ec1f80;
    } else {
        'l20: loop {
            sw(v1_3, 0x494e4954);
            v1_3 = v1_3 + 4;
            if (v1_3 as u32) >= (a1_3 as u32) {
                break;
            }
        }
        a1_4 = 0x1ec1f80;
    }
    v1_4 = 0x1eb1f80;
    v0_19 = 0x200c48;
    if (v1_4 as u32) < (a1_4 as u32) {
        'l23: loop {
            sw(v1_4, 0x494e4954);
            v1_4 = v1_4 + 4;
            if (v1_4 as u32) >= (a1_4 as u32) {
                break;
            }
        }
        v0_19 = 0x200c48;
    }
    // The PS2 original created a CPU thread and a GS thread here and put the main thread to sleep.
    // The port runs the frame loop directly on this thread.
    cpuMain();
    return;
}

pub fn cpuMain() {
    sw(g::gsdone, 1);
    let mut v0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut lo_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    timeTickStart();
    v0_1 = lw(g::restart);
    'l1: loop {
        if v0_1 == 0 {
            sw(g::displayallfileops, 0);
            profileReset();
            memReset();
            gameReset();
            v1_1 = lw(g::lvlevelnum);
            if lw(g::firsttime_129) != 0 {
                if v1_1 != 104 {
                    animMake();
                    sw(g::firsttime_129, 0);
                }
            }
            lvResetBefore();
            lvPreload();
            preloadExec();
            soundPreload();
            bgPreloadTextures();
            preloadPass2Exec();
            sw(g::displayallfileops, 1);
            lvResetAfter();
            lvLevelSetup();
            gameResetAfter();
        } else {
            sw(g::displayallfileops, 1);
            profileRestart();
            memRestart();
            gameRestart();
            lvRestartBefore();
            lvRestartAfter();
            lvLevelSetup();
            gameResetAfter();
        }
        timeTickEnd();
        timeTickStart();
        v0_2 = lw(g::gsdone);
        sw(g::lvfirstframe_130, 6);
        if v0_2 == 0 {
            a0_1 = 0x3838a0;
            scePrintf(&[a0_1 as i64]);
            sw(g::lvend, 0);
        } else {
            sw(g::curcpudmabuf, 0);
            sw(g::curgsdmabuf, 0);
            sw(g::cpudone, 0);
            sw(g::lvend, 0);
        }
        'l5: loop {
            s2_1 = g::dbinfo;
            s0_1 = 0xfffffff;
            'l7: loop {
                a1_1 = lw(g::framenum);
                lo_1 = lw(g::curcpudmabuf) * 12;
                sw(g::ucodetris, 0);
                sw(g::bgtris, 0);
                sw(g::bgcurtris, 0);
                sw(lo_1 + s2_1, 1);
                sw(lw(g::curcpudmabuf) * 12 + (s2_1 + 4), a1_1);
                sw(lw(g::curcpudmabuf) * 12 + (s2_1 + 8), 0);
                bossMainLoop();
                a0_2 = ((((lw(g::gfxtag) & s0_1) - (lw((lw(g::curcpudmabuf) << 2) + g::dmalists) & s0_1)) as u32) >> 4) as i32;
                if lw(g::maxdmalistlen) < a0_2 {
                    sw(g::maxdmalistlen, a0_2);
                }
                timeTickEnd();
                v1_2 = lw(g::lvfirstframe_130);
                if v1_2 == 0 {
                    sw(lw(g::curcpudmabuf) * 12 + s2_1, 0);
                }
                'b10: {
                    present_frame();
                    sw(g::curcpudmabuf, 1 - lw(g::curcpudmabuf));
                    sw(g::cpudone, 1);
                    v1_3 = v1_2 - 1;
                    if v1_2 == 0 {
                        v1_4 = lw(g::lvfirstframe_130);
                        v1_3 = v1_4 - 1;
                        if v1_4 == 0 {
                            break 'b10;
                        }
                    }
                    sw(g::lvfirstframe_130, v1_3);
                }
                'l12: loop {
                    v0_3 = ilinkOKToTick();
                    if v0_3 != 0 {
                        break;
                    }
                    ilinkTick();
                }
                sw(g::cpudone, 0);
                timeTickStart();
                v1_5 = lw(g::lvend);
                if v1_5 == 0 {
                    continue 'l5;
                }
                sw(g::cpudone, 0);
                sw(g::dbinfo, 1);
                sw(s2_1 + 12, 1);
                if lw(g::gsdone) == 0 {
                    a0_4 = 0x3838b8;
                    scePrintf(&[a0_4 as i64]);
                    v1_5 = lw(g::lvend);
                }
                if v1_5 != 0 {
                    break 'l5;
                }
            }
        }
        profileEnd();
        gameEnd();
        v0_1 = lw(g::restart);
    }
}
