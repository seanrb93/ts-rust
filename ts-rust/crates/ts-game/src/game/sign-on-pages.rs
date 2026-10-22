//! Functions: the sign on screens and their load, update and delete ticks.

use crate::*;

/// Start the sign on screens.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn signonMake() {
    let mut a1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut v0_2: i32 = 0;
    a1_1 = 2;
    a0_1 = 0x67380;
    v0_1 = memAlloc(a0_1, a1_1);
    s0_1 = 0;
    a0_2 = v0_1;
    sw(g::signonarray, v0_1);
    a1_2 = 0;
    a2_1 = 0x67380;
    memset(a0_2, a1_2, a2_1);
    'l1: loop {
        a0_3 = s0_1;
        signonDelete(a0_3);
        s0_1 = s0_1 + 1;
        if s0_1 >= 144 {
            break;
        }
    }
    s0_2 = 3;
    v0_2 = 0x1fbc164;
    'l3: loop {
        sw(v0_2, -1);
        s0_2 = s0_2 - 1;
        v0_2 = v0_2 - 4;
        if s0_2 < 0 {
            break;
        }
    }
    sw(0x3aff04, 0);
    return;
}

/// Tick of loading a player.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn signonLoadTick(mut a0: i32) -> i32 {
    let mut s5_1: i32 = 0;
    let mut v0_1: i64 = 0;
    let mut v0_2: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut s2_3: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut v0_24: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut v0_28: i32 = 0;
    'b1: {
        'b2: {
            s5_1 = 0;
            v0_1 = mcseqGetState();
            if (v0_1 as u64) < (11_i64 as u64) {
                'b3: {
                    match ((v0_1 as i32) << 2) >> 2 {
                        0 => {
                            if (lw(g::mcseq_msgflags) & 15) != 0 {
                                break 'b1;
                            }
                            v1_6 = 0x1fc11f8;
                            v0_26 = lw(v1_6 + 24) - 1;
                            sw(v1_6 + 24, v0_26);
                            s5_1 = ((v0_26 as u32) < (1 as u32)) as i32;
                            break 'b1;
                        }
                        1 | 4 => {
                        }
                        2 => {
                            if (lw(0x1fc11fc) & 12) == 0 {
                                s5_1 = 1;
                                break 'b1;
                            }
                            a0_29 = a0;
                            signonLoadRetry(a0_29);
                            break 'b1;
                        }
                        3 => {
                            v0_24 = mcseqGetResult();
                            if v0_24 != 4 {
                                if (lw(0x1fc11fc) & 12) == 0 {
                                    s5_1 = 1;
                                    break 'b1;
                                }
                                v0_25 = mcseqGetNumCards();
                                if v0_25 < 2 {
                                    s5_1 = 2;
                                    break 'b1;
                                }
                            }
                            a0_28 = a0;
                            signonLoadStart(a0_28);
                            break 'b1;
                        }
                        5 | 6 | 7 | 8 | 9 => {
                            break 'b2;
                        }
                        10 => {
                            'b5: {
                                'b6: {
                                    'b7: {
                                        v0_2 = mcseqGetResult();
                                        v1_1 = v0_2 - 1;
                                        if (v1_1 as u32) < (8 as u32) {
                                            'b8: {
                                                'b9: {
                                                    match (v1_1 << 2) >> 2 {
                                                        0 => {
                                                            a0_13 = lw(0x3aff3c);
                                                            a1_9 = 0x21315354;
                                                            a2_2 = 0xb790;
                                                            v0_10 = memcardFileCheck(a0_13, a1_9, a2_2);
                                                            a0_14 = lw(0x3aff3c);
                                                            if v0_10 == 0 {
                                                                break 'b8;
                                                            }
                                                            s2_1 = 0;
                                                            s1_1 = a0_14 + 4;
                                                            a0_15 = a0_14 + 20;
                                                            v0_11 = signonFileCount(a0_15);
                                                            s0_1 = 0;
                                                            if v0_11 != 0 {
                                                                'b14: {
                                                                    'b13: {
                                                                        'l11: loop {
                                                                            v0_15 = signonCount();
                                                                            if s0_1 >= v0_15 {
                                                                                break 'b13;
                                                                            }
                                                                            a1_14 = lw(s1_1 + 4);
                                                                            v1_4 = lw((s0_1 << 3) + (g::signoninfo + 4));
                                                                            s0_1 = s0_1 + 1;
                                                                            if v1_4 == a1_14 {
                                                                                break;
                                                                            }
                                                                        }
                                                                        s2_1 = 1;
                                                                        break 'b14;
                                                                    }
                                                                    a1_14 = lw(s1_1 + 4);
                                                                }
                                                                sw(0x3aff44, a1_14);
                                                                v0_16 = mcseqGetCurCard();
                                                                a0_20 = s1_1;
                                                                a1_15 = v0_16;
                                                                signonAddFile(a0_20, a1_15);
                                                                v0_17 = lw(g::cur_language);
                                                                if s2_1 == 0 {
                                                                    a0_23 = 4;
                                                                    v0_20 = mcseqGetMsg(a0_23);
                                                                    a0_22 = a0;
                                                                } else {
                                                                    s2_3 = g::languages;
                                                                    s1_3 = lw((v0_17 << 2) + s2_3);
                                                                    v0_18 = mcseqSlotGetLongName();
                                                                    a2_4 = v0_18;
                                                                    a1_16 = lw(s1_3 + 564);
                                                                    a0_21 = 0x3a19b0;
                                                                    v0_19 = lw((lw(g::cur_language) << 2) + s2_3);
                                                                    a4_1 = lw(v0_19 + 572);
                                                                    a3_2 = lw(v0_19 + 568);
                                                                    v0_20 = vafmt(&[a0_21 as i64, a1_16 as i64, a2_4 as i64, a3_2 as i64, a4_1 as i64]);
                                                                    a0_22 = a0;
                                                                }
                                                                a1_17 = v0_20;
                                                                v0_21 = GetPlayerSignOnMessage(a0_22, a1_17);
                                                                a2_5 = g::languages;
                                                                a1_13 = v0_21;
                                                                a0_19 = 2;
                                                                v1_5 = lw(g::cur_language) << 2;
                                                                break 'b6;
                                                            }
                                                            break 'b9;
                                                        }
                                                        1 => {
                                                            break 'b8;
                                                        }
                                                        2 => {
                                                            a0_10 = 142;
                                                            a1_7 = 0;
                                                            soundStart(a0_10, a1_7);
                                                            a0_11 = 3;
                                                            v0_8 = mcseqGetMsg(a0_11);
                                                            a0_12 = a0;
                                                            a1_8 = v0_8;
                                                            v0_9 = GetPlayerSignOnMessage(a0_12, a1_8);
                                                            a1_4 = v0_9;
                                                            a0_5 = 6;
                                                            a2_1 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1616);
                                                        }
                                                        3 => {
                                                            a0_6 = 142;
                                                            a1_5 = 0;
                                                            soundStart(a0_6, a1_5);
                                                            v0_5 = mcseqGetCurCard();
                                                            if v0_5 != -1 {
                                                                a0_9 = 1;
                                                                v0_6 = mcseqGetMsg(a0_9);
                                                                a0_8 = a0;
                                                            } else {
                                                                a0_7 = 0;
                                                                v0_6 = mcseqGetMsg(a0_7);
                                                                a0_8 = a0;
                                                            }
                                                            a1_6 = v0_6;
                                                            v0_7 = GetPlayerSignOnMessage(a0_8, a1_6);
                                                            a1_4 = v0_7;
                                                            a0_5 = 6;
                                                            a2_1 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1612);
                                                        }
                                                        4 => {
                                                            break 'b9;
                                                        }
                                                        5 => {
                                                            a0_2 = 142;
                                                            a1_2 = 0;
                                                            soundStart(a0_2, a1_2);
                                                            a0_3 = 2;
                                                            v0_3 = mcseqGetMsg(a0_3);
                                                            a0_4 = a0;
                                                            a1_3 = v0_3;
                                                            v0_4 = GetPlayerSignOnMessage(a0_4, a1_3);
                                                            a1_4 = v0_4;
                                                            a0_5 = 6;
                                                            a2_1 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1616);
                                                        }
                                                        6 => {
                                                            break 'b7;
                                                        }
                                                        7 => {
                                                            a0_1 = 142;
                                                            a1_1 = 0;
                                                            soundStart(a0_1, a1_1);
                                                            break 'b3;
                                                        }
                                                        _ => unreachable!(),
                                                    }
                                                    mcseqMessage(a0_5, a1_4, a2_1);
                                                    v1_2 = 0x1fc11f8;
                                                    sw(v1_2 + 4, lw(v1_2 + 4) | 4);
                                                    break 'b1;
                                                }
                                                a0_16 = 142;
                                                a1_10 = 0;
                                                soundStart(a0_16, a1_10);
                                                s2_2 = g::languages;
                                                s1_2 = lw((lw(g::cur_language) << 2) + s2_2);
                                                v0_12 = mcseqSlotGetLongName();
                                                a2_3 = lw(s1_2 + 560);
                                                a0_17 = 0x3a19a0;
                                                a1_11 = lw(s1_2 + 556);
                                                a3_1 = v0_12;
                                                v0_13 = vafmt(&[a0_17 as i64, a1_11 as i64, a2_3 as i64, a3_1 as i64]);
                                                a0_18 = a0;
                                                a1_12 = v0_13;
                                                v0_14 = GetPlayerSignOnMessage(a0_18, a1_12);
                                                a1_13 = v0_14;
                                                a0_19 = 2;
                                                v1_3 = (lw(g::cur_language) << 2) + s2_2;
                                                break 'b5;
                                            }
                                            a0_24 = a0;
                                            corruptLoadMessage(a0_24);
                                            break 'b1;
                                        }
                                    }
                                    a0_25 = 142;
                                    a1_18 = 0;
                                    soundStart(a0_25, a1_18);
                                    a0_26 = 3;
                                    v0_22 = mcseqGetMsg(a0_26);
                                    a0_27 = a0;
                                    a1_19 = v0_22;
                                    v0_23 = GetPlayerSignOnMessage(a0_27, a1_19);
                                    a2_5 = g::languages;
                                    a1_13 = v0_23;
                                    v1_5 = lw(g::cur_language) << 2;
                                    a0_19 = 2;
                                }
                                v1_3 = v1_5 + a2_5;
                            }
                            a2_6 = lw(lw(v1_3) + 1612);
                            mcseqMessage(a0_19, a1_13, a2_6);
                            break 'b1;
                        }
                        _ => unreachable!(),
                    }
                }
                s5_1 = 1;
                break 'b1;
            }
        }
        sw(0x1fc1210, 180);
    }
    v0_27 = s5_1;
    if s5_1 != 0 {
        mcseqWindowClose();
        v0_28 = 0x1fc11f8;
        sw(v0_28 + 4, lw(v0_28 + 4) & -2);
        v0_27 = s5_1;
    }
    return v0_27;
}

/// Tick of deleting a player.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn signonDeleteTick(mut a0: i32) -> i32 {
    let mut v1_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut v0_1: i64 = 0;
    let mut v0_2: i64 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_3: i64 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_7: i64 = 0;
    let mut v0_8: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut s1_4: i32 = 0;
    let mut s2_3: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut s1_5: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut a5_3: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut v0_24: i32 = 0;
    'b1: {
        v1_1 = lbu(0x3aff4b);
        s4_1 = 0;
        if (v1_1 as u32) < (5 as u32) {
            'b2: {
                'b3: {
                    match (v1_1 << 2) >> 2 {
                        0 => {
                            a0_21 = lw((lbu(0x3aff49) << 3) + g::signoninfo);
                            mcseqSetCurCard(a0_21);
                            s1_5 = lw((lw(g::cur_language) << 2) + g::languages);
                            v0_22 = signonFileSize();
                            a1_15 = lw(s1_5 + 1900);
                            a3_4 = 0x3a1960;
                            a4_4 = lw(0x3aff3c);
                            a5_3 = v0_22;
                            a0_22 = 10;
                            a2_9 = 0;
                            a6_2 = 0;
                            mcseqLoad(a0_22, a1_15, a2_9, a3_4, a4_4, a5_3, a6_2);
                            a0_23 = a0;
                            mcseqController(a0_23);
                            sb(0x3aff4b, 1);
                            break 'b1;
                        }
                        1 => {
                            v0_7 = mcseqGetState();
                            if v0_7 != 10_i64 {
                                break 'b1;
                            }
                            v0_8 = mcseqGetResult();
                            v1_2 = v0_8 - 1;
                            if (v1_2 as u32) < (8 as u32) {
                                match (v1_2 << 2) >> 2 {
                                    0 => {
                                        a0_10 = lw(0x3aff3c);
                                        a1_7 = 0x21315354;
                                        a2_3 = 0xb790;
                                        v0_13 = memcardFileCheck(a0_10, a1_7, a2_3);
                                        a2_4 = lbu(0x3aff49);
                                        if v0_13 == 0 {
                                            a0_18 = a0;
                                            corruptLoadMessage(a0_18);
                                            break 'b2;
                                        }
                                        s1_2 = lw(0x3aff3c);
                                        if lw(s1_2 + 8) == lw((a2_4 << 3) + (g::signoninfo + 4)) {
                                            s2_2 = s1_2 + 20;
                                            a0_11 = s2_2;
                                            a1_8 = lw(g::signonarray) + a2_4 * 2936;
                                            v0_14 = signonFileGetName(a0_11, a1_8);
                                            a0_12 = s2_2;
                                            if v0_14 >= 0 {
                                                a1_9 = v0_14;
                                                signonFileDelete(a0_12, a1_9);
                                                a0_13 = lw(0x3aff3c);
                                                if lw(s1_2 + 4) == v0_14 {
                                                    sw(s1_2 + 4, -1);
                                                    a0_13 = lw(0x3aff3c);
                                                }
                                                a1_10 = 0x21315354;
                                                a2_5 = 0xb790;
                                                memcardFileMakeCRC(a0_13, a1_10, a2_5);
                                                s1_3 = lw((lw(g::cur_language) << 2) + g::languages);
                                                v0_15 = signonFileSize();
                                                a1_11 = lw(s1_3 + 1896);
                                                a3_2 = 0x3a1960;
                                                a4_2 = lw(0x3aff3c);
                                                a5_1 = v0_15;
                                                a0_14 = 11;
                                                a2_6 = 0;
                                                a6_1 = 0;
                                                mcseqSave(a0_14, a1_11, a2_6, a3_2, a4_2, a5_1, a6_1);
                                                a0_15 = a0;
                                                mcseqController(a0_15);
                                                sb(0x3aff4b, 2);
                                                break 'b1;
                                            }
                                        }
                                        s1_4 = g::languages;
                                        s2_3 = lw((lw(g::cur_language) << 2) + s1_4);
                                        v0_16 = mcseqSlotGetLongName();
                                        a3_3 = v0_16;
                                        a2_7 = lw(s2_3 + 648);
                                        a0_16 = 0x3a1b20;
                                        a1_12 = lw(s2_3 + 644);
                                        v0_17 = lw((lw(g::cur_language) << 2) + s1_4);
                                        a5_2 = lw(v0_17 + 656);
                                        a4_3 = lw(v0_17 + 652);
                                        v0_18 = vafmt(&[a0_16 as i64, a1_12 as i64, a2_7 as i64, a3_3 as i64, a4_3 as i64, a5_2 as i64]);
                                        a0_17 = a0;
                                        a1_13 = v0_18;
                                        v0_19 = GetPlayerSignOnMessage(a0_17, a1_13);
                                        a1_6 = v0_19;
                                        a0_8 = 2;
                                        v1_3 = (lw(g::cur_language) << 2) + s1_4;
                                        break 'b3;
                                    }
                                    1 => {
                                        a0_9 = a0;
                                        corruptLoadMessage(a0_9);
                                        break 'b2;
                                    }
                                    2 | 7 => {
                                    }
                                    3 | 4 | 5 | 6 => {
                                        s2_1 = g::languages;
                                        s1_1 = lw((lw(g::cur_language) << 2) + s2_1);
                                        v0_9 = mcseqSlotGetLongName();
                                        a2_2 = v0_9;
                                        a1_4 = lw(s1_1 + 660);
                                        a0_6 = 0x3a19b0;
                                        v0_10 = lw((lw(g::cur_language) << 2) + s2_1);
                                        a4_1 = lw(v0_10 + 668);
                                        a3_1 = lw(v0_10 + 664);
                                        v0_11 = vafmt(&[a0_6 as i64, a1_4 as i64, a2_2 as i64, a3_1 as i64, a4_1 as i64]);
                                        a0_7 = a0;
                                        a1_5 = v0_11;
                                        v0_12 = GetPlayerSignOnMessage(a0_7, a1_5);
                                        a1_6 = v0_12;
                                        a0_8 = 2;
                                        v1_3 = (lw(g::cur_language) << 2) + s2_1;
                                        break 'b3;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        2 => {
                            v0_3 = mcseqGetState();
                            if v0_3 != 11_i64 {
                                break 'b1;
                            }
                            v0_4 = mcseqGetResult();
                            if v0_4 == 1 {
                                a0_2 = lbu(0x3aff49);
                                signonDelete(a0_2);
                                a0_3 = 9;
                                v0_5 = mcseqGetMsg(a0_3);
                                a0_4 = a0;
                                a1_2 = v0_5;
                                v0_6 = GetPlayerSignOnMessage(a0_4, a1_2);
                                a1_3 = v0_6;
                                a0_5 = 2;
                                a2_1 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1612);
                                mcseqMessage(a0_5, a1_3, a2_1);
                                sb(0x3aff4b, 4);
                                break 'b1;
                            }
                        }
                        3 => {
                            a0_1 = 142;
                            a1_1 = 0;
                            soundStart(a0_1, a1_1);
                            sb(0x3aff4b, 4);
                            break 'b1;
                        }
                        4 => {
                            v0_1 = mcseqGetState();
                            if v0_1 != 2_i64 {
                                v0_2 = mcseqGetState();
                                if v0_2 != 3_i64 {
                                    break 'b1;
                                }
                                if (lw(0x1fc11fc) & 8) == 0 {
                                    s4_1 = 2;
                                    break 'b1;
                                }
                                signonDeleteStart();
                                break 'b1;
                            }
                            s4_1 = 1;
                            break 'b1;
                        }
                        _ => unreachable!(),
                    }
                    a0_19 = 8;
                    v0_20 = mcseqGetMsg(a0_19);
                    a0_20 = a0;
                    a1_14 = v0_20;
                    v0_21 = GetPlayerSignOnMessage(a0_20, a1_14);
                    a1_6 = v0_21;
                    a0_8 = 2;
                    v1_3 = (lw(g::cur_language) << 2) + g::languages;
                }
                a2_8 = lw(lw(v1_3) + 1612);
                mcseqMessage(a0_8, a1_6, a2_8);
            }
            sb(0x3aff4b, 3);
        }
    }
    v0_23 = s4_1;
    if s4_1 != 0 {
        mcseqWindowClose();
        v0_24 = 0x1fc11f8;
        sw(v0_24 + 4, lw(v0_24 + 4) & -2);
        v0_23 = s4_1;
    }
    return v0_23;
}

/// Tick of updating a player.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn signonUpdateTick(mut a0: i32) -> i32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut v0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_3: i64 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_4: i64 = 0;
    let mut v0_5: i64 = 0;
    let mut a1_3: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_9: i64 = 0;
    let mut v0_10: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v0_11: i64 = 0;
    let mut v0_12: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut s5_2: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a3_2: i64 = 0;
    let mut a4_2: i64 = 0;
    let mut a5_2: i64 = 0;
    let mut a3_3: i64 = 0;
    let mut a4_3: i64 = 0;
    let mut a5_3: i64 = 0;
    let mut a1_10: i64 = 0;
    let mut a3_4: i64 = 0;
    let mut a0_11: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut a5_4: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut s5_3: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut v0_24: i64 = 0;
    let mut v0_25: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut s0_5: i32 = 0;
    let mut v0_28: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut s0_6: i32 = 0;
    let mut v0_29: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut v0_30: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut v0_31: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut v0_32: i32 = 0;
    let mut a4_5: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a1_20: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut a5_5: i32 = 0;
    let mut v0_33: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut a1_21: i32 = 0;
    let mut v0_34: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut v0_35: i32 = 0;
    let mut v0_36: i32 = 0;
    let mut s3_2: i32 = 0;
    let mut s2_3: i32 = 0;
    let mut s0_7: i32 = 0;
    let mut v0_37: i32 = 0;
    let mut a4_6: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut a1_23: i32 = 0;
    let mut a2_11: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut a5_6: i32 = 0;
    let mut v0_38: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut a1_24: i32 = 0;
    let mut v0_39: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut v0_40: i32 = 0;
    let mut a2_12: i64 = 0;
    let mut a3_8: i64 = 0;
    let mut a4_7: i64 = 0;
    let mut a1_25: i64 = 0;
    let mut a2_13: i64 = 0;
    let mut a3_9: i64 = 0;
    let mut a5_7: i64 = 0;
    let mut v0_41: i64 = 0;
    let mut a2_14: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut a1_26: i32 = 0;
    let mut s0_8: i32 = 0;
    let mut v0_42: i32 = 0;
    let mut a2_15: i32 = 0;
    let mut a1_27: i32 = 0;
    let mut a4_8: i32 = 0;
    let mut a3_10: i32 = 0;
    let mut a5_8: i32 = 0;
    let mut a0_30: i32 = 0;
    let mut a6_3: i32 = 0;
    let mut a0_31: i32 = 0;
    let mut v0_43: i64 = 0;
    let mut v0_44: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut a0_32: i32 = 0;
    let mut a0_33: i32 = 0;
    let mut a1_28: i32 = 0;
    let mut v0_45: i32 = 0;
    let mut a0_34: i32 = 0;
    let mut v0_46: i32 = 0;
    let mut a0_35: i32 = 0;
    let mut a0_36: i32 = 0;
    let mut a1_29: i32 = 0;
    let mut v0_47: i32 = 0;
    let mut a2_16: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut a0_37: i32 = 0;
    let mut a1_30: i32 = 0;
    let mut a2_17: i32 = 0;
    let mut v0_48: i32 = 0;
    let mut a0_38: i32 = 0;
    let mut v0_49: i32 = 0;
    let mut a0_39: i32 = 0;
    let mut a1_31: i32 = 0;
    let mut v0_50: i32 = 0;
    let mut a1_32: i32 = 0;
    let mut a0_40: i32 = 0;
    let mut a2_18: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut v0_51: i32 = 0;
    let mut a0_41: i32 = 0;
    let mut a1_33: i32 = 0;
    let mut v0_52: i32 = 0;
    let mut a1_34: i32 = 0;
    let mut a0_42: i32 = 0;
    let mut a2_19: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut v0_53: i32 = 0;
    let mut a4_9: i32 = 0;
    let mut a1_35: i32 = 0;
    let mut a3_11: i32 = 0;
    let mut a5_9: i32 = 0;
    let mut a0_43: i32 = 0;
    let mut a2_20: i32 = 0;
    let mut a6_4: i32 = 0;
    let mut a0_44: i32 = 0;
    let mut a0_45: i32 = 0;
    let mut v0_54: i32 = 0;
    let mut v1_15: i32 = 0;
    let mut a0_46: i32 = 0;
    let mut v0_55: i32 = 0;
    let mut a4_10: i32 = 0;
    let mut a1_36: i32 = 0;
    let mut a3_12: i32 = 0;
    let mut a5_10: i32 = 0;
    let mut a0_47: i32 = 0;
    let mut a2_21: i32 = 0;
    let mut a6_5: i32 = 0;
    let mut a0_48: i32 = 0;
    let mut v0_56: i32 = 0;
    let mut v0_57: i32 = 0;
    'b1: {
        'b2: {
            'b3: {
                'b4: {
                    sw(sp, 0);
                    v0_1 = signonGet(a0);
                    a0_1 = a0;
                    a1_1 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1920);
                    v0_2 = GetPlayerDialogTitle(a0_1, a1_1);
                    v1_1 = lbu(0x3aff4b);
                    if (v1_1 as u32) < (11 as u32) {
                        'b5: {
                            'b6: {
                                'b7: {
                                    'b8: {
                                        'b9: {
                                            'b10: {
                                                'b11: {
                                                    'b12: {
                                                        'b13: {
                                                            match (v1_1 << 2) >> 2 {
                                                                0 => {
                                                                    v1_14 = lw((v0_1 << 3) + g::signoninfo);
                                                                    if v1_14 != 8 {
                                                                        if v1_14 != -1 {
                                                                            a0_46 = v1_14;
                                                                            mcseqSetCurCard(a0_46);
                                                                            v0_55 = signonFileSize();
                                                                            a4_10 = lw(0x3aff3c);
                                                                            a1_36 = v0_2;
                                                                            a3_12 = 0x3a1960;
                                                                            a5_10 = v0_55;
                                                                            a0_47 = 10;
                                                                            a2_21 = 0;
                                                                            a6_5 = 0;
                                                                            mcseqLoad(a0_47, a1_36, a2_21, a3_12, a4_10, a5_10, a6_5);
                                                                            a0_48 = a0;
                                                                            mcseqController(a0_48);
                                                                            v0_54 = lw(g::mcseq_flags);
                                                                            v1_15 = 6;
                                                                        } else {
                                                                            mcseqUnvalidateCards();
                                                                            v0_53 = signonFileSize();
                                                                            a4_9 = lw(0x3aff3c);
                                                                            a1_35 = v0_2;
                                                                            a3_11 = 0x3a1960;
                                                                            a5_9 = v0_53;
                                                                            a0_43 = 10;
                                                                            a2_20 = 0;
                                                                            a6_4 = 0;
                                                                            mcseqLoad(a0_43, a1_35, a2_20, a3_11, a4_9, a5_9, a6_4);
                                                                            a0_44 = a0;
                                                                            mcseqController(a0_44);
                                                                            a0_45 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1928);
                                                                            mcseqAddOption(a0_45);
                                                                            v0_54 = lw(g::mcseq_flags);
                                                                            v1_15 = 1;
                                                                        }
                                                                        sb(0x3aff4b, v1_15);
                                                                        sw(g::mcseq_flags, v0_54 | 32);
                                                                        break 'b4;
                                                                    }
                                                                    sw(sp, 1);
                                                                    break 'b2;
                                                                }
                                                                1 => {
                                                                    v0_43 = mcseqGetState();
                                                                    a1_3 = lw(sp);
                                                                    if v0_43 != 10_i64 {
                                                                        break 'b3;
                                                                    }
                                                                    v0_44 = mcseqGetResult();
                                                                    v1_11 = v0_44 - 1;
                                                                    if (v1_11 as u32) >= (9 as u32) {
                                                                        break 'b6;
                                                                    }
                                                                    match (v1_11 << 2) >> 2 {
                                                                        0 => {
                                                                            a0_37 = lw(0x3aff3c);
                                                                            a1_30 = 0x21315354;
                                                                            a2_17 = 0xb790;
                                                                            v0_48 = memcardFileCheck(a0_37, a1_30, a2_17);
                                                                            if v0_48 == 0 {
                                                                                break 'b7;
                                                                            }
                                                                            sb(0x3aff4b, 3);
                                                                            break 'b4;
                                                                        }
                                                                        1 => {
                                                                            break 'b7;
                                                                        }
                                                                        2 => {
                                                                            break 'b6;
                                                                        }
                                                                        3 => {
                                                                            a0_33 = 142;
                                                                            a1_28 = 0;
                                                                            soundStart(a0_33, a1_28);
                                                                            v0_45 = mcseqGetCurCard();
                                                                            if v0_45 != -1 {
                                                                                a0_36 = 1;
                                                                                v0_46 = mcseqGetMsg(a0_36);
                                                                                a0_35 = a0;
                                                                            } else {
                                                                                a0_34 = 0;
                                                                                v0_46 = mcseqGetMsg(a0_34);
                                                                                a0_35 = a0;
                                                                            }
                                                                            a1_29 = v0_46;
                                                                            v0_47 = GetPlayerSignOnMessage(a0_35, a1_29);
                                                                            v1_8 = lw(g::cur_language);
                                                                            a2_8 = g::languages;
                                                                            a1_17 = v0_47;
                                                                            a0_18 = 6;
                                                                            break 'b9;
                                                                        }
                                                                        4 | 5 => {
                                                                            sb(0x3aff4b, 2);
                                                                            break 'b4;
                                                                        }
                                                                        6 => {
                                                                            a0_32 = 11;
                                                                            break 'b5;
                                                                        }
                                                                        7 => {
                                                                            break 'b10;
                                                                        }
                                                                        8 => {
                                                                            sw(sp, 1);
                                                                            sw((v0_1 << 3) + g::signoninfo, 8);
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                2 => {
                                                                    s0_5 = lw(0x3aff3c);
                                                                    v0_28 = signonMakeFileId();
                                                                    a0_19 = s0_5 + 20;
                                                                    sw(s0_5 + 8, v0_28);
                                                                    a1_18 = 0;
                                                                    a2_9 = 0xb780;
                                                                    memset(a0_19, a1_18, a2_9);
                                                                    break 'b11;
                                                                }
                                                                3 => {
                                                                    break 'b11;
                                                                }
                                                                4 => {
                                                                    v0_24 = mcseqGetState();
                                                                    a1_3 = lw(sp);
                                                                    if v0_24 != 10_i64 {
                                                                        break 'b3;
                                                                    }
                                                                    v0_25 = mcseqGetResult();
                                                                    v1_7 = v0_25 - 1;
                                                                    if (v1_7 as u32) < (8 as u32) {
                                                                        match (v1_7 << 2) >> 2 {
                                                                            0 => {
                                                                                sb(0x3aff4b, 5);
                                                                                break 'b4;
                                                                            }
                                                                            1 | 2 | 3 | 4 | 5 => {
                                                                            }
                                                                            6 => {
                                                                                break 'b13;
                                                                            }
                                                                            7 => {
                                                                                break 'b10;
                                                                            }
                                                                            _ => unreachable!(),
                                                                        }
                                                                    }
                                                                    a0_16 = 5;
                                                                    break 'b12;
                                                                }
                                                                5 => {
                                                                    s0_4 = lw(0x3aff3c) + 4;
                                                                    v0_22 = mcseqGetCurCard();
                                                                    a1_15 = v0_1 << 3;
                                                                    v1_6 = g::signoninfo;
                                                                    sw(a1_15 + v1_6, v0_22);
                                                                    v0_23 = lw(s0_4 + 4);
                                                                    sb(0x3aff4b, 8);
                                                                    sw(v1_6 + a1_15 + 4, v0_23);
                                                                    break 'b4;
                                                                }
                                                                6 => {
                                                                    v0_11 = mcseqGetState();
                                                                    a1_3 = lw(sp);
                                                                    if v0_11 != 10_i64 {
                                                                        break 'b3;
                                                                    }
                                                                    v0_12 = mcseqGetResult();
                                                                    v1_3 = v0_12 - 1;
                                                                    if (v1_3 as u32) >= (8 as u32) {
                                                                        break 'b6;
                                                                    }
                                                                    match (v1_3 << 2) >> 2 {
                                                                        0 => {
                                                                            a0_8 = lw(0x3aff3c);
                                                                            a1_8 = 0x21315354;
                                                                            a2_3 = 0xb790;
                                                                            v0_15 = memcardFileCheck(a0_8, a1_8, a2_3);
                                                                            a2_4 = lw(0x3aff3c);
                                                                            if v0_15 == 0 {
                                                                                break 'b7;
                                                                            }
                                                                            s5_2 = a2_4 + 4;
                                                                            if lw(s5_2 + 4) == lw((v0_1 << 3) + (g::signoninfo + 4)) {
                                                                                a0_9 = a2_4 + 20;
                                                                                a1_9 = lw(g::signonarray) + v0_1 * 2936;
                                                                                v0_16 = signonFileGetName(a0_9, a1_9);
                                                                                if v0_16 >= 0 {
                                                                                    'b19: {
                                                                                        v1_4 = v0_16 * 2936 + s5_2 + 16;
                                                                                        a0_10 = v0_1 * 2936 + lw(g::signonarray);
                                                                                        v0_17 = a0_10 + 2912;
                                                                                        if ((a0_10 | v1_4) & 7) == 0 {
                                                                                            'l20: loop {
                                                                                                a3_3 = ld(a0_10 + 8);
                                                                                                a4_3 = ld(a0_10 + 16);
                                                                                                a5_3 = ld(a0_10 + 24);
                                                                                                sd(v1_4, ld(a0_10));
                                                                                                sd(v1_4 + 8, a3_3);
                                                                                                sd(v1_4 + 16, a4_3);
                                                                                                sd(v1_4 + 24, a5_3);
                                                                                                a0_10 = a0_10 + 32;
                                                                                                v1_4 = v1_4 + 32;
                                                                                                if a0_10 == v0_17 {
                                                                                                    break 'b19;
                                                                                                }
                                                                                            }
                                                                                        } else {
                                                                                            'l21: loop {
                                                                                                a3_2 = ld(a0_10 + 8);
                                                                                                a4_2 = ld(a0_10 + 16);
                                                                                                a5_2 = ld(a0_10 + 24);
                                                                                                sd(v1_4, ld(a0_10));
                                                                                                sd(v1_4 + 8, a3_2);
                                                                                                sd(v1_4 + 16, a4_2);
                                                                                                sd(v1_4 + 24, a5_2);
                                                                                                a0_10 = a0_10 + 32;
                                                                                                v1_4 = v1_4 + 32;
                                                                                                if a0_10 == v0_17 {
                                                                                                    break 'b19;
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    a1_10 = ld(a0_10 + 8);
                                                                                    a3_4 = ld(a0_10 + 16);
                                                                                    sd(v1_4, ld(a0_10));
                                                                                    sd(v1_4 + 8, a1_10);
                                                                                    sd(v1_4 + 16, a3_4);
                                                                                    sw(s5_2, v0_16);
                                                                                    a0_11 = lw(0x3aff3c);
                                                                                    a1_11 = 0x21315354;
                                                                                    a2_5 = 0xb790;
                                                                                    memcardFileMakeCRC(a0_11, a1_11, a2_5);
                                                                                    s0_2 = lw((lw(g::cur_language) << 2) + g::languages);
                                                                                    v0_18 = signonFileSize();
                                                                                    a2_6 = lw(s0_2 + 1892);
                                                                                    a1_12 = v0_2;
                                                                                    a4_4 = lw(0x3aff3c);
                                                                                    a3_5 = 0x3a1960;
                                                                                    a5_4 = v0_18;
                                                                                    a0_12 = 11;
                                                                                    a6_2 = 0;
                                                                                    mcseqSave(a0_12, a1_12, a2_6, a3_5, a4_4, a5_4, a6_2);
                                                                                    a0_13 = a0;
                                                                                    mcseqController(a0_13);
                                                                                    sb(0x3aff4b, 7);
                                                                                    break 'b4;
                                                                                }
                                                                            }
                                                                            s4_1 = g::languages;
                                                                            s5_3 = lw((lw(g::cur_language) << 2) + s4_1);
                                                                            v0_19 = mcseqSlotGetLongName();
                                                                            s0_3 = lw((lw(g::cur_language) << 2) + s4_1);
                                                                            s1_2 = lw(g::signonarray) + v0_1 * 2936;
                                                                            v0_14 = mcseqSlotGetLongName();
                                                                            a1_7 = lw(s5_3 + 608);
                                                                            a0_7 = 0x3a1a40;
                                                                            a6_1 = lw(s0_3 + 620);
                                                                            a2_2 = v0_19;
                                                                            a3_1 = lw(s0_3 + 612);
                                                                            a5_1 = s1_2;
                                                                            a4_1 = lw(s0_3 + 616);
                                                                        }
                                                                        1 => {
                                                                            break 'b7;
                                                                        }
                                                                        2 | 7 => {
                                                                            break 'b6;
                                                                        }
                                                                        3 | 4 | 5 | 6 => {
                                                                            s4_1 = g::languages;
                                                                            s5_1 = lw((lw(g::cur_language) << 2) + s4_1);
                                                                            v0_13 = mcseqSlotGetLongName();
                                                                            s0_1 = lw((lw(g::cur_language) << 2) + s4_1);
                                                                            s1_1 = lw(g::signonarray) + v0_1 * 2936;
                                                                            v0_14 = mcseqSlotGetLongName();
                                                                            a1_7 = lw(s5_1 + 624);
                                                                            a0_7 = 0x3a1a40;
                                                                            a6_1 = lw(s0_1 + 636);
                                                                            a2_2 = v0_13;
                                                                            a3_1 = lw(s0_1 + 628);
                                                                            a5_1 = s1_1;
                                                                            a4_1 = lw(s0_1 + 632);
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                    a7_1 = v0_14;
                                                                    v0_20 = vafmt(&[a0_7 as i64, a1_7 as i64, a2_2 as i64, a3_1 as i64, a4_1 as i64, a5_1 as i64, a6_1 as i64, a7_1 as i64]);
                                                                    a0_14 = a0;
                                                                    a1_13 = v0_20;
                                                                    v0_21 = GetPlayerSignOnMessage(a0_14, a1_13);
                                                                    a1_14 = v0_21;
                                                                    a0_15 = 6;
                                                                    a2_7 = lw(lw((lw(g::cur_language) << 2) + s4_1) + 1616);
                                                                    mcseqMessage(a0_15, a1_14, a2_7);
                                                                    sb(0x3aff4b, 9);
                                                                    v1_5 = 0x1fc11f8;
                                                                    sw(v1_5 + 4, lw(v1_5 + 4) | 32);
                                                                    break 'b4;
                                                                }
                                                                7 => {
                                                                    v0_9 = mcseqGetState();
                                                                    a1_3 = lw(sp);
                                                                    if v0_9 != 11_i64 {
                                                                        break 'b3;
                                                                    }
                                                                    v0_10 = mcseqGetResult();
                                                                    v1_2 = v0_10 - 1;
                                                                    if (v1_2 as u32) >= (8 as u32) {
                                                                        break 'b6;
                                                                    }
                                                                    match (v1_2 << 2) >> 2 {
                                                                        0 => {
                                                                            sb(0x3aff4b, 8);
                                                                            break 'b4;
                                                                        }
                                                                        1 | 2 | 3 | 4 | 5 | 7 => {
                                                                            break 'b6;
                                                                        }
                                                                        6 => {
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                8 => {
                                                                    a0_4 = 7;
                                                                    v0_7 = mcseqGetMsg(a0_4);
                                                                    a0_5 = a0;
                                                                    a1_5 = v0_7;
                                                                    v0_8 = GetPlayerSignOnMessage(a0_5, a1_5);
                                                                    a1_6 = v0_8;
                                                                    a0_6 = 2;
                                                                    a2_1 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1612);
                                                                    mcseqMessage(a0_6, a1_6, a2_1);
                                                                    sb(0x3aff4b, 10);
                                                                    break 'b4;
                                                                }
                                                                9 => {
                                                                    a0_3 = 142;
                                                                    a1_4 = 0;
                                                                    soundStart(a0_3, a1_4);
                                                                    sb(0x3aff4b, 10);
                                                                    break 'b4;
                                                                }
                                                                10 => {
                                                                    v0_3 = mcseqGetState();
                                                                    if v0_3 == 1_i64 {
                                                                        a1_2 = 0x1fc11f8;
                                                                        a0_2 = lw(a1_2 + 4);
                                                                        if (a0_2 & 8) != 0 {
                                                                            sb(0x3aff4b, 2);
                                                                            sw(a1_2 + 4, a0_2 & -9);
                                                                        }
                                                                    }
                                                                    v0_4 = mcseqGetState();
                                                                    if v0_4 != 2_i64 {
                                                                        v0_5 = mcseqGetState();
                                                                        a1_3 = lw(sp);
                                                                        if v0_5 != 3_i64 {
                                                                            break 'b3;
                                                                        }
                                                                        v0_6 = lw(0x1fc11fc);
                                                                        if (v0_6 & 12) == 0 {
                                                                            sw(sp, 1);
                                                                            sw(sp, if (v0_6 & 32) != 0 { 2 } else { 1 });
                                                                            break 'b4;
                                                                        }
                                                                    } else if (lw(0x1fc11fc) & 36) == 0 {
                                                                        sw(sp, 1);
                                                                        break 'b4;
                                                                    }
                                                                    signonUpdateStart();
                                                                    a1_3 = lw(sp);
                                                                    break 'b3;
                                                                }
                                                                _ => unreachable!(),
                                                            }
                                                        }
                                                        a0_16 = 11;
                                                    }
                                                    v0_26 = mcseqGetMsg(a0_16);
                                                    a0_17 = a0;
                                                    a1_16 = v0_26;
                                                    v0_27 = GetPlayerSignOnMessage(a0_17, a1_16);
                                                    v1_8 = lw(g::cur_language);
                                                    a2_8 = g::languages;
                                                    a1_17 = v0_27;
                                                    a0_18 = 2;
                                                    break 'b9;
                                                }
                                                s0_6 = lw(0x3aff3c);
                                                v0_29 = mcseqGetCurCard();
                                                s2_1 = s0_6 + 4;
                                                a1_19 = v0_29;
                                                a0_20 = s2_1;
                                                s3_1 = s0_6 + 20;
                                                signonCheckSlot(a0_20, a1_19);
                                                a0_21 = a0;
                                                v0_30 = signonGet(a0_21);
                                                a0_22 = s3_1;
                                                v0_31 = signonFileGetFree(a0_22);
                                                if v0_31 >= 0 {
                                                    a0_25 = s3_1;
                                                    a1_22 = lw(g::signonarray) + v0_30 * 2936;
                                                    v0_35 = signonFileGetName(a0_25, a1_22);
                                                    v0_36 = lw(g::cur_language);
                                                    if v0_35 < 0 {
                                                        'b24: {
                                                            v1_10 = v0_31 * 2936 + s2_1 + 16;
                                                            a0_28 = v0_30 * 2936 + lw(g::signonarray);
                                                            v0_40 = a0_28 + 2912;
                                                            if ((a0_28 | v1_10) & 7) == 0 {
                                                                'l25: loop {
                                                                    a1_25 = ld(a0_28 + 8);
                                                                    a2_13 = ld(a0_28 + 16);
                                                                    a3_9 = ld(a0_28 + 24);
                                                                    sd(v1_10, ld(a0_28));
                                                                    sd(v1_10 + 8, a1_25);
                                                                    sd(v1_10 + 16, a2_13);
                                                                    sd(v1_10 + 24, a3_9);
                                                                    a0_28 = a0_28 + 32;
                                                                    v1_10 = v1_10 + 32;
                                                                    if a0_28 == v0_40 {
                                                                        break 'b24;
                                                                    }
                                                                }
                                                            } else {
                                                                'l26: loop {
                                                                    a2_12 = ld(a0_28 + 8);
                                                                    a3_8 = ld(a0_28 + 16);
                                                                    a4_7 = ld(a0_28 + 24);
                                                                    sd(v1_10, ld(a0_28));
                                                                    sd(v1_10 + 8, a2_12);
                                                                    sd(v1_10 + 16, a3_8);
                                                                    sd(v1_10 + 24, a4_7);
                                                                    a0_28 = a0_28 + 32;
                                                                    v1_10 = v1_10 + 32;
                                                                    if a0_28 == v0_40 {
                                                                        break 'b24;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        a5_7 = ld(a0_28 + 8);
                                                        v0_41 = ld(a0_28 + 16);
                                                        sd(v1_10, ld(a0_28));
                                                        sd(v1_10 + 8, a5_7);
                                                        a2_14 = 0xb790;
                                                        sd(v1_10 + 16, v0_41);
                                                        sw(s2_1, v0_31);
                                                        a0_29 = lw(0x3aff3c);
                                                        a1_26 = 0x21315354;
                                                        memcardFileMakeCRC(a0_29, a1_26, a2_14);
                                                        s0_8 = lw((lw(g::cur_language) << 2) + g::languages);
                                                        v0_42 = signonFileSize();
                                                        a2_15 = lw(s0_8 + 1892);
                                                        a1_27 = v0_2;
                                                        a4_8 = lw(0x3aff3c);
                                                        a3_10 = 0x3a1960;
                                                        a5_8 = v0_42;
                                                        a0_30 = 10;
                                                        a6_3 = 0;
                                                        mcseqSave(a0_30, a1_27, a2_15, a3_10, a4_8, a5_8, a6_3);
                                                        a0_31 = a0;
                                                        mcseqController(a0_31);
                                                        sb(0x3aff4b, 4);
                                                        break 'b4;
                                                    }
                                                    s3_2 = g::languages;
                                                    s2_3 = lw(g::signonarray) + v0_30 * 2936;
                                                    s0_7 = lw((v0_36 << 2) + s3_2);
                                                    v0_37 = mcseqSlotGetLongName();
                                                    a4_6 = lw(s0_7 + 604);
                                                    a0_26 = 0x3a1a28;
                                                    a1_23 = lw(s0_7 + 596);
                                                    a2_11 = s2_3;
                                                    a3_7 = lw(s0_7 + 600);
                                                    a5_6 = v0_37;
                                                    v0_38 = vafmt(&[a0_26 as i64, a1_23 as i64, a2_11 as i64, a3_7 as i64, a4_6 as i64, a5_6 as i64]);
                                                    a0_27 = a0;
                                                    a1_24 = v0_38;
                                                    v0_39 = GetPlayerSignOnMessage(a0_27, a1_24);
                                                    a1_17 = v0_39;
                                                    a0_18 = 6;
                                                    v1_9 = (lw(g::cur_language) << 2) + s3_2;
                                                    break 'b8;
                                                }
                                                s2_2 = g::languages;
                                                s1_3 = lw((lw(g::cur_language) << 2) + s2_2);
                                                v0_32 = mcseqSlotGetLongName();
                                                a4_5 = v0_32;
                                                a3_6 = lw(s1_3 + 588);
                                                a0_23 = 0x3a1a10;
                                                a1_20 = lw(s1_3 + 580);
                                                a2_10 = lw(s1_3 + 584);
                                                a5_5 = lw(lw((lw(g::cur_language) << 2) + s2_2) + 592);
                                                v0_33 = vafmt(&[a0_23 as i64, a1_20 as i64, a2_10 as i64, a3_6 as i64, a4_5 as i64, a5_5 as i64]);
                                                a0_24 = a0;
                                                a1_21 = v0_33;
                                                v0_34 = GetPlayerSignOnMessage(a0_24, a1_21);
                                                a1_17 = v0_34;
                                                a0_18 = 6;
                                                v1_9 = (lw(g::cur_language) << 2) + s2_2;
                                                break 'b8;
                                            }
                                            sw(sp, 2);
                                            break 'b4;
                                        }
                                        v1_9 = (v1_8 << 2) + a2_8;
                                    }
                                    a2_16 = lw(lw(v1_9) + 1612);
                                    mcseqMessage(a0_18, a1_17, a2_16);
                                    sb(0x3aff4b, 9);
                                    v1_12 = 0x1fc11f8;
                                    sw(v1_12 + 4, lw(v1_12 + 4) | 4);
                                    break 'b4;
                                }
                                a0_38 = 10;
                                v0_49 = mcseqGetMsg(a0_38);
                                a0_39 = a0;
                                a1_31 = v0_49;
                                v0_50 = GetPlayerSignOnMessage(a0_39, a1_31);
                                a1_32 = v0_50;
                                a0_40 = 5;
                                a2_18 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1624);
                                mcseqMessage(a0_40, a1_32, a2_18);
                                sb(0x3aff4b, 9);
                                v1_13 = 0x1fc11f8;
                                sw(v1_13 + 4, lw(v1_13 + 4) | 8);
                                break 'b4;
                            }
                            a0_32 = 5;
                        }
                        v0_51 = mcseqGetMsg(a0_32);
                        a0_41 = a0;
                        a1_33 = v0_51;
                        v0_52 = GetPlayerSignOnMessage(a0_41, a1_33);
                        a1_34 = v0_52;
                        a0_42 = 2;
                        a2_19 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1612);
                        mcseqMessage(a0_42, a1_34, a2_19);
                        sb(0x3aff4b, 9);
                    }
                }
                a1_3 = lw(sp);
            }
            v0_56 = lw(sp);
            if a1_3 == 0 {
                break 'b1;
            }
            mcseqWindowClose();
        }
        v0_57 = 0x1fc11f8;
        sw(v0_57 + 4, lw(v0_57 + 4) & -2);
        v0_56 = lw(sp);
    }
    return v0_56;
}

/// Tick of the player page.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn signonplr_pageTick(mut a0: i32) {
    let mut a4_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut v0_24: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a1_20: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut a1_21: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut a0_30: i32 = 0;
    let mut v0_28: i32 = 0;
    'b1: {
        'b2: {
            a4_1 = lw(g::front);
            a2_1 = a4_1;
            s5_1 = 0;
            a1_1 = lbu(0x3aff4a);
            sw((lw(a4_1) << 2) + 0x1fc1200, 0);
            if a1_1 == 0 {
                if lbu(0x3aff40) != 0 {
                    v0_2 = lw(a2_1 + 12);
                    break 'b2;
                }
                if lbu(0x3aff48) != 0 {
                    v0_2 = lw(a2_1 + 12);
                    break 'b2;
                }
                if lbu(0x3aff94) == 0 {
                    v0_1 = lw(a4_1);
                    break 'b1;
                }
            }
            v0_2 = lw(a2_1 + 12);
        }
        a0_1 = lw(a2_1);
        sw(a2_1 + 12, v0_2 & -61);
        joyClearAll(a0_1);
        a4_1 = lw(g::front);
        v0_1 = lw(a4_1);
    }
    'b4: {
        'b5: {
            v1_1 = lbu(v0_1 + 0x3aff28) - 1;
            if (v1_1 as u32) < (5 as u32) {
                match (v1_1 << 2) >> 2 {
                    0 => {
                        a0_25 = lw(a4_1);
                        a1_20 = -1;
                        signonSet(a0_25, a1_20);
                        a0_26 = lw(lw(g::front));
                        sh(a0 + 4, lhu((a0_26 << 1) + 0x3aff88));
                        sb(a0_26 + 0x3aff28, 2);
                        break 'b4;
                    }
                    1 => {
                        v0_9 = (lw(a4_1) << 1) + 0x3aff30;
                        v1_6 = lhu(v0_9);
                        if v1_6 == 0 {
                            a1_8 = lhu(a0 + 4);
                        } else {
                            sh(a0 + 4, v1_6);
                            sh(v0_9, 0);
                            a1_8 = lhu(a0 + 4);
                        }
                        if (a1_8 as u32) >= (2000 as u32) {
                            v0_11 = lw(a4_1);
                        } else {
                            s0_2 = a1_8 & 0xffff;
                            v0_10 = signonCount();
                            if v0_10 >= s0_2 {
                                a4_3 = lw(g::front);
                                a1_8 = lhu(a0 + 4);
                            } else {
                                a4_3 = lw(g::front);
                                sh(a0 + 4, 2000);
                                a1_8 = 2000;
                            }
                            v0_11 = lw(a4_3);
                        }
                        a0_12 = g::signonplr_menu;
                        sh((v0_11 << 1) + 0x3aff88, a1_8);
                        a1_9 = lhu(a0 + 4);
                        v0_12 = menuitem(a0_12, a1_9);
                        v0_13 = lbu(0x3aff4a);
                        if v0_12 != 0 {
                            v0_13 = lbu(0x3aff4a);
                            if (lhu(v0_12 + 10) & 64) == 0 {
                                s1_2 = 0x1fc1200;
                                s0_3 = lhu(a0 + 4) - 1;
                                v1_7 = (lw(lw(g::front)) << 2) + s1_2;
                                sw(v1_7, lw(v1_7) | 2);
                                v0_14 = signonCount();
                                v0_15 = lw(g::front);
                                if (s0_3 as u32) < (v0_14 as u32) {
                                    v1_8 = (lw(v0_15) << 2) + s1_2;
                                    sw(v1_8, lw(v1_8) | 8);
                                }
                                v0_13 = lbu(0x3aff4a);
                            }
                        }
                        if v0_13 != 0 {
                            s5_1 = g::signonplr_menu;
                        } else if lbu(0x3aff40) != 0 {
                            s5_1 = g::signonplr_menu;
                        } else if lbu(0x3aff48) != 0 {
                            s5_1 = g::signonplr_menu;
                        } else if lbu(0x3aff94) != 0 {
                            s5_1 = g::signonplr_menu;
                        } else {
                            a0_13 = lw(lw(g::front));
                            v0_16 = joyNewBut(a0_13);
                            v0_17 = lw(g::front);
                            if (v0_16 & 16) == 0 {
                                'b11: {
                                    a0_17 = lw(v0_17);
                                    v0_19 = joyNewBut(a0_17);
                                    a4_4 = lw(g::front);
                                    if (v0_19 & 128) != 0 {
                                        a5_2 = 0x1fc11f8;
                                        v0_20 = lbu(0x3aff48);
                                        if (lw(a5_2 + (lw(a4_4) << 2) + 8) & 8) != 0 {
                                            if v0_20 == 0 {
                                                a2_4 = lw(a5_2 + 4);
                                                a0_18 = lhu(a0 + 4);
                                                sb(0x3aff48, lbu(a4_4) + 1);
                                                v1_10 = lw(a4_4);
                                                a0_19 = 140;
                                                a1_13 = lbu(a0_18 + 0x1fc330f) + 1;
                                                sw(a5_2 + 4, a2_4 & -3);
                                                sb(v1_10 + 0x3aff90, a1_13);
                                                break 'b11;
                                            }
                                        }
                                    }
                                    a0_20 = lw(a4_4);
                                    v0_21 = joyNewBut(a0_20);
                                    v0_22 = lw(g::front);
                                    if (v0_21 & 64) == 0 {
                                        s5_1 = g::signonplr_menu;
                                        break 'b5;
                                    }
                                    s1_3 = 0x1fc11f8;
                                    a0_19 = 142;
                                    if (lw(s1_3 + (lw(v0_22) << 2) + 8) & 2) != 0 {
                                        s0_4 = lhu(a0 + 4) - 1;
                                        v0_23 = signonCount();
                                        a0_21 = 140;
                                        if (s0_4 as u32) < (v0_23 as u32) {
                                            a1_14 = 0;
                                            sh(v0_12 + 10, lhu(v0_12 + 10) | 64);
                                            soundStart(a0_21, a1_14);
                                            a0_22 = lw(lw(g::front));
                                            a1_15 = lbu(lhu(a0 + 4) + 0x1fc330f);
                                            signonSet(a0_22, a1_15);
                                            s5_1 = g::signon_ready_waiting;
                                            sb(lw(lw(g::front)) + 0x3aff28, 5);
                                            break 'b5;
                                        }
                                        v1_11 = lhu(a0 + 4);
                                        if v1_11 != 2000 {
                                            if v1_11 != 2001 {
                                                s5_1 = g::signonplr_menu;
                                                break 'b5;
                                            }
                                            if lbu(0x3aff40) != 0 {
                                                s5_1 = g::signonplr_menu;
                                                break 'b5;
                                            }
                                            a0_24 = 141;
                                            a1_18 = 0;
                                            v0_25 = lbu(lw(g::front)) + 1;
                                            sw(s1_3 + 4, lw(s1_3 + 4) & -3);
                                            sb(0x3aff40, v0_25);
                                            soundStart(a0_24, a1_18);
                                            break 'b5;
                                        }
                                        v0_24 = signonCount();
                                        a0_23 = 141;
                                        if v0_24 < 144 {
                                            a1_16 = 0;
                                            soundStart(a0_23, a1_16);
                                            a1_17 = lw(g::front);
                                            sb(lw(a1_17) * 30 + 0x1fc3298, 0);
                                            sb(lw(a1_17) + 0x3aff28, 3);
                                            break 'b4;
                                        }
                                        a0_19 = 142;
                                    }
                                }
                                a1_19 = 0;
                                soundStart(a0_19, a1_19);
                                s5_1 = g::signonplr_menu;
                            } else {
                                a1_10 = 0xffff;
                                a0_14 = lw(v0_17);
                                joyClearButs(a0_14, a1_10);
                                if lw(lw(g::front)) != 0 {
                                    a0_16 = 142;
                                } else {
                                    'b14: {
                                        a2_3 = lw(0x3aff20);
                                        a1_11 = 0;
                                        if a2_3 > 0 {
                                            v1_9 = lbu(0x3aff28);
                                            if lw(0x1fc1488) == 0 {
                                                if v1_9 == 2 {
                                                    a1_11 = a1_11 + 1;
                                                } else {
                                                    if v1_9 != 5 {
                                                        break 'b14;
                                                    }
                                                    a1_11 = a1_11 + 1;
                                                }
                                                'l16: loop {
                                                    if a1_11 >= a2_3 {
                                                        break 'b14;
                                                    }
                                                    v0_18 = a1_11 + 0x3aff28;
                                                    if lw(a1_11 * 260 + 0x1fc1488) != 0 {
                                                        break 'b14;
                                                    }
                                                    a0_15 = lbu(v0_18);
                                                    if a0_15 == 2 {
                                                        a1_11 = a1_11 + 1;
                                                    } else {
                                                        if a0_15 != 5 {
                                                            break 'b14;
                                                        }
                                                        a1_11 = a1_11 + 1;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    a0_16 = 142;
                                    if a1_11 == a2_3 {
                                        pageOut();
                                        break 'b5;
                                    }
                                }
                                a1_12 = 0;
                                soundStart(a0_16, a1_12);
                            }
                        }
                    }
                    2 => {
                        a0_11 = lw(a4_1);
                        a2_2 = 30;
                        a3_1 = 1;
                        a4_2 = 174;
                        a5_1 = 0;
                        a6_1 = 0;
                        a1_7 = a0_11 * 30 + 0x1fc3298;
                        stringeditStart(a0_11, a1_7, a2_2, a3_1, a4_2, a5_1, a6_1);
                        sb(lw(lw(g::front)) + 0x3aff28, 4);
                    }
                    3 => {
                        s5_1 = 0;
                        if lbu(0x3aff48) == 0 {
                            v0_6 = lbu(0x3aff40);
                            if lbu(0x3aff4a) == 0 {
                                v0_7 = lbu(0x3aff94);
                                if v0_6 == 0 {
                                    if v0_7 == 0 {
                                        a0_7 = lw(a4_1);
                                        stringeditTick(a0_7);
                                        v1_4 = lw(g::textbox);
                                        if lw(v1_4 + 32) == 0 {
                                            v1_5 = lw(g::front);
                                            if lw(v1_4 + 36) != 0 {
                                                sb(lw(v1_5) + 0x3aff28, 2);
                                                sh(a0 + 4, 2000);
                                            }
                                        } else {
                                            a0_8 = lw(lw(g::front)) * 30 + 0x1fc3298;
                                            v0_8 = signonNew(a0_8);
                                            a1_6 = v0_8;
                                            a0_9 = lw(lw(g::front));
                                            signonSet(a0_9, a1_6);
                                            a0_10 = lw(g::front);
                                            sb(0x3aff4a, lbu(a0_10) + 1);
                                            sb(lw(a0_10) + 0x3aff28, 2);
                                        }
                                        s5_1 = 0;
                                    }
                                }
                            }
                        }
                    }
                    4 => {
                        sh(a0 + 4, 0);
                        s1_1 = 0;
                        'l18: loop {
                            v0_3 = signonCount();
                            if s1_1 >= v0_3 {
                                break;
                            }
                            a0_6 = lw(lw(g::front));
                            s0_1 = lbu(s1_1 + 0x1fc3310);
                            v0_5 = signonGet(a0_6);
                            a1_5 = s1_1 + 1;
                            if s0_1 == v0_5 {
                                sh((lw(lw(g::front)) << 1) + 0x3aff88, a1_5);
                            }
                            s1_1 = a1_5;
                        }
                        a0_2 = lw(lw(g::front));
                        v0_4 = joyNewBut(a0_2);
                        v1_2 = lw(g::front);
                        if (v0_4 & 16) != 0 {
                            a1_2 = -1;
                            a0_3 = lw(v1_2);
                            sh(a0 + 4, lhu((a0_3 << 1) + 0x3aff88));
                            signonSet(a0_3, a1_2);
                            v1_3 = lw(g::front);
                            a1_3 = 16;
                            sb(lw(v1_3) + 0x3aff28, 2);
                            a0_4 = lw(v1_3);
                            joyClearButs(a0_4, a1_3);
                            a0_5 = 142;
                            a1_4 = 0;
                            soundStart(a0_5, a1_4);
                        }
                        s5_1 = g::signon_ready_waiting;
                    }
                    _ => unreachable!(),
                }
            }
        }
        v1_12 = lw(g::front);
        if s5_1 != 0 {
            if lbu(lw(v1_12) + 0x3aff28) == 2 {
                a0_27 = g::signonplr_list;
                v0_26 = 8;
                if lw(g::window + 92) > 0 {
                    v0_26 = 5;
                }
                v1_13 = lhu(g::signonplr_list);
                sh(a0_27 + 2, v0_26);
                if (v1_13 as u32) < (v0_26 as u32) {
                    sh(a0_27 + 2, v1_13);
                }
            }
            a0_28 = lw(0x3aff20);
            v1_14 = 80;
            if a0_28 == 2 {
                v1_14 = 140;
            }
            sh(g::signon_ready_waiting + 6, v1_14);
            v0_27 = 40;
            if a0_28 == 2 {
                v0_27 = 130;
            }
            sh(g::signonplr_list + 6, v0_27);
            a0_29 = a0;
            a1_21 = s5_1;
            menutick_230238(a0_29, a1_21);
            a1_22 = s5_1;
            a0_30 = lw(a0 + 8) + lw(a0 + 12) * 24;
            v0_28 = std_menumake(a0_30, a1_22);
            sw(a0 + 12, lw(a0 + 12) + v0_28);
        }
    }
    return;
}

/// Tick of the sign on page.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn signon_pageTick(mut a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut s5_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut s7_1: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut a0_30: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut a0_31: i32 = 0;
    let mut a0_32: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut a0_33: i32 = 0;
    let mut v0_24: i32 = 0;
    let mut s3_2: i32 = 0;
    let mut s1_4: i32 = 0;
    let mut s0_5: i32 = 0;
    let mut a0_34: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut a0_35: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut s1_5: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a0_36: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut v0_28: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut a0_37: i32 = 0;
    let mut v0_29: i32 = 0;
    let mut a0_38: i32 = 0;
    let mut a0_39: i32 = 0;
    let mut v0_30: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut a1_20: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut v0_31: i32 = 0;
    let mut a1_21: i32 = 0;
    let mut a0_40: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut a0_41: i32 = 0;
    let mut v0_32: i32 = 0;
    let mut a0_42: i32 = 0;
    let mut v0_33: i32 = 0;
    let mut a0_43: i32 = 0;
    let mut a0_44: i32 = 0;
    let mut a0_45: i32 = 0;
    let mut a1_23: i32 = 0;
    let mut a0_46: i32 = 0;
    let mut a1_24: i32 = 0;
    let mut a0_47: i32 = 0;
    let mut a0_48: i32 = 0;
    let mut a0_49: i32 = 0;
    let mut v0_34: i32 = 0;
    let mut a0_50: i32 = 0;
    let mut a1_25: i32 = 0;
    let mut a1_26: i32 = 0;
    let mut a0_51: i32 = 0;
    let mut v0_35: i32 = 0;
    let mut s1_6: i32 = 0;
    let mut a0_52: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_36: i32 = 0;
    let mut a0_53: i32 = 0;
    let mut v0_37: i32 = 0;
    let mut s0_6: i32 = 0;
    let mut a0_54: i32 = 0;
    let mut v0_38: i32 = 0;
    let mut s1_7: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut a0_55: i32 = 0;
    let mut v0_39: i32 = 0;
    let mut a1_27: i32 = 0;
    let mut a0_56: i32 = 0;
    let mut a0_57: i32 = 0;
    let mut a0_58: i32 = 0;
    'b1: {
        s5_1 = g::languages;
        v1_1 = (lw(g::cur_language) << 2) + s5_1;
        a1_1 = g::signon_page;
        sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
        a2_1 = lw(v1_1);
        sw(sp, a0);
        a0_1 = lw(a2_1 + 472);
        pageSet(a0_1, a1_1);
        a0_2 = 0;
        frontfxTick(a0_2);
        v0_1 = lw(0x3aff98);
        if lbu(lw(lw(g::front)) + 0x3aff28) != 1 {
            if v0_1 <= 0 {
                v0_2 = lbu(0x3aff94);
                s0_1 = v0_2 - 1;
                if v0_2 == 0 {
                    'b2: {
                        v0_4 = lbu(0x3aff40);
                        v1_2 = lw(0x3aff9c);
                        if v0_4 == 0 {
                            v0_12 = lbu(0x3aff4a);
                            if v0_12 == 0 {
                                v0_24 = lbu(0x3aff48);
                                if v0_24 == 0 {
                                    break 'b1;
                                }
                                s3_2 = v0_24 - 1;
                                s1_4 = s3_2 + 0x3aff90;
                                s0_5 = lbu(s1_4) - 1;
                                if lw(0x3aff9c) != 0 {
                                    v0_25 = lw(0x1fc11fc);
                                    if (v0_25 & 2) != 0 {
                                        if (v0_25 & 1) != 0 {
                                            a0_49 = s3_2;
                                            v0_34 = signonDeleteTick(a0_49);
                                            if v0_34 == 0 {
                                                break 'b1;
                                            }
                                            sb(s1_4, 0);
                                            sb(0x3aff48, 0);
                                            break 'b2;
                                        }
                                        a0_42 = s3_2;
                                        v0_33 = splitMessageTick(a0_42);
                                        if v0_33 != 2 {
                                            a0_46 = 142;
                                            if v0_33 == 3 {
                                                sb(s1_4, 0);
                                                sb(0x3aff48, 0);
                                                a1_24 = 0;
                                                sw(0x3aff9c, 0);
                                                soundStart(a0_46, a1_24);
                                            }
                                        } else {
                                            a0_43 = ((lw((s0_5 << 3) + g::signoninfo) as u32) < (8 as u32)) as i32;
                                            sh((s3_2 << 1) + 0x3aff30, 2000);
                                            if a0_43 == 0 {
                                                a0_45 = s0_5;
                                                signonDelete(a0_45);
                                                sb(s1_4, 0);
                                                sb(0x3aff48, 0);
                                                sw(0x3aff9c, 0);
                                                a0_44 = 140;
                                            } else {
                                                sb(0x3aff49, s0_5);
                                                signonDeleteStart();
                                                a0_44 = 140;
                                            }
                                            a1_23 = 0;
                                            soundStart(a0_44, a1_23);
                                        }
                                        if v0_33 != 0 {
                                            a0_47 = s3_2;
                                            splitMessageClear(a0_47);
                                        }
                                        a0_48 = s3_2;
                                        joyClearAll(a0_48);
                                        break 'b1;
                                    }
                                    a0_35 = lw((s0_5 << 3) + g::signoninfo);
                                    if (a0_35 as u32) >= (8 as u32) {
                                        a0_39 = 0x3a1e40;
                                        v0_30 = lw((lw(g::cur_language) << 2) + s5_1);
                                        a2_7 = lw(g::signonarray) + s0_5 * 2936;
                                        a4_4 = lw(v0_30 + 1372);
                                        a1_20 = lw(v0_30 + 1364);
                                        a3_4 = lw(v0_30 + 1368);
                                        v0_31 = vafmt(&[a0_39 as i64, a1_20 as i64, a2_7 as i64, a3_4 as i64, a4_4 as i64]);
                                        a1_21 = v0_31;
                                        a0_40 = s3_2;
                                        v0_29 = GetPlayerSignOnMessage(a0_40, a1_21);
                                        a0_38 = s3_2;
                                    } else {
                                        s2_2 = lw((lw(g::cur_language) << 2) + s5_1);
                                        s1_5 = lw(g::signonarray) + s0_5 * 2936;
                                        v0_26 = mcseqSlotMakeName(a0_35);
                                        a4_3 = v0_26;
                                        a3_3 = lw(s2_2 + 1348);
                                        a0_36 = 0x3a1e28;
                                        a1_18 = lw(s2_2 + 1344);
                                        a2_6 = s1_5;
                                        v0_27 = lw((lw(g::cur_language) << 2) + s5_1);
                                        a6_1 = lw(v0_27 + 1356);
                                        a5_2 = lw(v0_27 + 1352);
                                        v0_28 = vafmt(&[a0_36 as i64, a1_18 as i64, a2_6 as i64, a3_3 as i64, a4_3 as i64, a5_2 as i64, a6_1 as i64]);
                                        a1_19 = v0_28;
                                        a0_37 = s3_2;
                                        v0_29 = GetPlayerSignOnMessage(a0_37, a1_19);
                                        a0_38 = s3_2;
                                    }
                                    a2_8 = v0_29;
                                    a1_22 = 1;
                                    splitMessageStart(a0_38, a1_22, a2_8);
                                    a0_41 = 0x1fc11f8;
                                    v0_32 = lw(a0_41 + 4);
                                    sw(0x3aff98, -1);
                                    sw(a0_41 + 4, v0_32 | 2);
                                    break 'b1;
                                }
                                a0_34 = s3_2;
                                soZoomStart(a0_34);
                                break 'b1;
                            }
                            s2_1 = v0_12 - 1;
                            if v1_2 == 0 {
                                a0_15 = s2_1;
                                soZoomStart(a0_15);
                                break 'b1;
                            }
                            if (lw(0x1fc11fc) & 1) == 0 {
                                signonUpdateStart();
                                break 'b1;
                            }
                            a0_16 = s2_1;
                            v0_13 = signonUpdateTick(a0_16);
                            a0_17 = s2_1;
                            if v0_13 != 0 {
                                v0_14 = signonGet(a0_17);
                                if v0_13 != 2 {
                                    v1_4 = lw((v0_14 << 3) + g::signoninfo);
                                    if v1_4 == -1 {
                                        a0_31 = v0_14;
                                        signonDelete(a0_31);
                                        a0_32 = s2_1;
                                        a1_16 = -1;
                                        signonSet(a0_32, a1_16);
                                        sh((s2_1 << 1) + 0x3aff30, 2000);
                                        a1_17 = 0;
                                        a0_33 = 142;
                                        soundStart(a0_33, a1_17);
                                        sb(s2_1 + 0x3aff28, 2);
                                    } else {
                                        if v1_4 == 8 {
                                            'b10: {
                                                s7_1 = 0;
                                                v0_15 = signonMemCount();
                                                v0_16 = lw(g::cur_language);
                                                if v0_15 < 16 {
                                                    s0_4 = 0;
                                                    v0_18 = signonCount();
                                                    'l11: loop {
                                                        s3_1 = (s0_4 < v0_18) as i32;
                                                        if s3_1 == 0 {
                                                            break 'b10;
                                                        }
                                                        if s0_4 == v0_14 {
                                                            s0_4 = s0_4 + 1;
                                                        } else {
                                                            s1_3 = s2_1 * 30 + 0x1fc3298;
                                                            a0_22 = lw(g::signonarray) + s0_4 * 2936;
                                                            a1_10 = s1_3;
                                                            v0_19 = strcmp(a0_22, a1_10);
                                                            if v0_19 != 0 {
                                                                s0_4 = s0_4 + 1;
                                                            } else {
                                                                if lw((s0_4 << 3) + g::signoninfo) == 8 {
                                                                    break;
                                                                }
                                                                s0_4 = s0_4 + 1;
                                                            }
                                                        }
                                                    }
                                                    v1_6 = lw(g::cur_language);
                                                    if s3_1 != 0 {
                                                        a2_4 = s1_3;
                                                        a0_23 = 0x3a1e18;
                                                        v0_20 = lw((v1_6 << 2) + g::languages);
                                                        a4_2 = lw(v0_20 + 1336);
                                                        a1_11 = lw(v0_20 + 1328);
                                                        a3_2 = lw(v0_20 + 1332);
                                                        v0_21 = vafmt(&[a0_23 as i64, a1_11 as i64, a2_4 as i64, a3_2 as i64, a4_2 as i64]);
                                                        s7_1 = v0_21;
                                                    }
                                                } else {
                                                    v1_5 = lw((v0_16 << 2) + s5_1);
                                                    a0_21 = 0x3a1e08;
                                                    a3_1 = s2_1 * 30 + 0x1fc3298;
                                                    a5_1 = lw(v1_5 + 1324);
                                                    a1_9 = lw(v1_5 + 1312);
                                                    a2_3 = lw(v1_5 + 1316);
                                                    a4_1 = lw(v1_5 + 1320);
                                                    v0_17 = vafmt(&[a0_21 as i64, a1_9 as i64, a2_3 as i64, a3_1 as i64, a4_1 as i64, a5_1 as i64]);
                                                    s7_1 = v0_17;
                                                }
                                            }
                                            if s7_1 != 0 {
                                                a0_24 = v0_14;
                                                signonDelete(a0_24);
                                                a0_25 = s2_1;
                                                a1_12 = -1;
                                                signonSet(a0_25, a1_12);
                                                sh((s2_1 << 1) + 0x3aff30, 2000);
                                                a1_13 = 0;
                                                a0_26 = 142;
                                                soundStart(a0_26, a1_13);
                                                sb(s2_1 + 0x3aff28, 2);
                                                a2_5 = s7_1;
                                                a0_27 = s2_1;
                                                a1_14 = 2;
                                                splitMessageStart(a0_27, a1_14, a2_5);
                                                a0_28 = 0x1fc11f8;
                                                v0_22 = lw(a0_28 + 4);
                                                sb(0x3aff94, s2_1 + 1);
                                                sw(a0_28 + 4, v0_22 | 2);
                                            }
                                        }
                                        a0_29 = s2_1;
                                        v0_23 = signonGet(a0_29);
                                        if v0_23 >= 0 {
                                            sb(s2_1 + 0x3aff28, 5);
                                            a0_30 = 140;
                                            a1_15 = 0;
                                            soundStart(a0_30, a1_15);
                                            sb(0x3aff4a, 0);
                                            break 'b2;
                                        }
                                    }
                                } else {
                                    a0_18 = v0_14;
                                    signonDelete(a0_18);
                                    a0_19 = s2_1;
                                    a1_7 = -1;
                                    signonSet(a0_19, a1_7);
                                    sh((s2_1 << 1) + 0x3aff30, 2000);
                                    a1_8 = 0;
                                    a0_20 = 142;
                                    soundStart(a0_20, a1_8);
                                    sb(s2_1 + 0x3aff28, 3);
                                }
                                sb(0x3aff4a, 0);
                                break 'b2;
                            }
                            if lw(0x3aff98) != 0 {
                                break 'b1;
                            }
                        } else {
                            s1_1 = v0_4 - 1;
                            if v1_2 == 0 {
                                a0_9 = s1_1;
                                soZoomStart(a0_9);
                                break 'b1;
                            }
                            if (lw(0x1fc11fc) & 1) != 0 {
                                a0_11 = s1_1;
                                v0_5 = signonLoadTick(a0_11);
                                v0_6 = lw(0x3aff44);
                                if v0_5 == 0 {
                                    break 'b1;
                                }
                                'b13: {
                                    v0_7 = lw(0x3aff20);
                                    if v0_6 == 0 {
                                        a0_14 = 142;
                                        a1_6 = 0;
                                        soundStart(a0_14, a1_6);
                                    } else {
                                        'b14: {
                                            s0_2 = 0;
                                            if v0_7 > 0 {
                                                'l15: loop {
                                                    a0_12 = s0_2;
                                                    v0_8 = signonGet(a0_12);
                                                    v0_9 = lw(0x3aff20);
                                                    if v0_8 == -1 {
                                                        a1_4 = s0_2 + 0x3aff28;
                                                        v0_9 = lw(0x3aff20);
                                                        if lbu(a1_4) == 5 {
                                                            sb(a1_4, 2);
                                                            sh((s0_2 << 1) + 0x3aff30, 2000);
                                                            v0_9 = lw(0x3aff20);
                                                        }
                                                    }
                                                    s0_2 = s0_2 + 1;
                                                    if s0_2 >= v0_9 {
                                                        break 'b14;
                                                    }
                                                }
                                            }
                                        }
                                        s1_2 = g::signoninfo + 4;
                                        s0_3 = 0;
                                        'l17: loop {
                                            v0_10 = signonCount();
                                            v0_11 = lw(0x3aff44);
                                            if s0_3 >= v0_10 {
                                                break 'b13;
                                            }
                                            v1_3 = lw(s1_2);
                                            s1_2 = s1_2 + 8;
                                            if v1_3 == v0_11 {
                                                break;
                                            }
                                            s0_3 = s0_3 + 1;
                                        }
                                        a0_13 = 140;
                                        a1_5 = 0;
                                        soundStart(a0_13, a1_5);
                                        sh((s1_1 << 1) + 0x3aff30, s0_3 + 1);
                                    }
                                }
                                sb(0x3aff40, 0);
                                break 'b2;
                            }
                            a0_10 = s1_1;
                            signonLoadStart(a0_10);
                        }
                        sw(0x3aff98, -1);
                        break 'b1;
                    }
                    sw(0x3aff9c, 0);
                } else {
                    a0_5 = s0_1;
                    v0_3 = splitMessageTick(a0_5);
                    if ((v0_3 - 2) as u32) < (2 as u32) {
                        sb(0x3aff94, 0);
                        a0_6 = 140;
                        sw(0x3aff9c, 0);
                        a1_3 = 0;
                        soundStart(a0_6, a1_3);
                        a0_7 = s0_1;
                        splitMessageClear(a0_7);
                        sb(s0_1 + 0x3aff28, 3);
                    }
                    a0_8 = s0_1;
                    joyClearAll(a0_8);
                }
            } else {
                sw(0x3aff98, v0_1 - 1);
            }
        } else {
            a0_3 = g::signonplr_page;
            splitpageInit(a0_3);
            sb(0x3aff40, 0);
            sw(0x3aff98, -2);
            a0_4 = 0x3aff90;
            sb(0x3aff48, 0);
            a1_2 = 0;
            sb(0x3aff4a, 0);
            a2_2 = 4;
            sb(0x3aff94, 0);
            sw(0x3aff9c, 0);
            memset(a0_4, a1_2, a2_2);
        }
    }
    'b20: {
        makesignonmenu();
        a0_50 = lw(sp);
        a1_25 = g::signonplr_page;
        splitpageTick(a0_50, a1_25);
        a1_26 = lw(0x3aff20);
        a0_51 = 0;
        if a1_26 > 0 {
            if lbu(0x3aff28) == 5 {
                a0_51 = a0_51 + 1;
                'l21: loop {
                    v0_35 = a0_51 + 0x3aff28;
                    if a0_51 >= a1_26 {
                        break 'b20;
                    }
                    if lbu(v0_35) != 5 {
                        break 'b20;
                    }
                    a0_51 = a0_51 + 1;
                }
            }
        }
    }
    if a0_51 != a1_26 {
        a0_58 = 0x1fc11f8;
        sw(a0_58 + 4, lw(a0_58 + 4) & -513);
        return;
    }
    s1_6 = 0;
    settingsToMenus();
    a0_52 = lw(0x3aff20);
    statsCombine(a0_52);
    v1_7 = lw(0x3aff20);
    v0_36 = 0x1fc11f8;
    sw(v0_36 + 44, 0);
    sw(v0_36 + 32, 0);
    sw(v0_36 + 36, 0);
    if v1_7 <= 0 {
        s1_7 = 0;
    } else {
        'l23: loop {
            a0_53 = s1_6;
            v0_37 = signonGet(a0_53);
            s0_6 = lw(g::signonarray) + v0_37 * 2936 + 32;
            a0_54 = lw(s0_6 + 84);
            v0_38 = chrPlayable(a0_54);
            if v0_38 == 0 {
                sw(s0_6 + 84, 0);
            }
            s1_6 = s1_6 + 1;
            if s1_6 >= lw(0x3aff20) {
                break;
            }
        }
        s1_7 = 0;
    }
    v1_8 = 0x1fc11f8;
    sw(v1_8 + 4, lw(v1_8 + 4) | 512);
    'l26: loop {
        a0_55 = s1_7;
        v0_39 = frontGet(a0_55);
        s1_7 = s1_7 + 1;
        sw(lw(v0_39 + 4) + 12, 0);
        if s1_7 >= 4 {
            break;
        }
    }
    checkUnavailableThings();
    a1_27 = g::gametype_page;
    a0_56 = 0;
    frontPageSet(a0_56, a1_27);
    a0_57 = lw(sp);
    pageIn(a0_57);
    return;
}
