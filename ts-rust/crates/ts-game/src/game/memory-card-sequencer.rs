//! Functions: the sequencer that the menus use to talk to the memory card, and its page.

use crate::*;

/// Set the message of the sequencer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseqMessage(mut a0: i32, mut a1: i32, mut a2: i32) {
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v0_1: i32 = 0;
    if a1 == 0 {
        s2_1 = 0x1fc3470;
    } else {
        a2_1 = 255;
        a0_1 = 0x1fc3470;
        strncpy(a0_1, a1, a2_1);
        s2_1 = 0x1fc3470;
    }
    sb(s2_1 + 255, 0);
    if a2 == 0 {
        v0_1 = 0x1fc3570;
    } else {
        a1_1 = a2;
        a0_2 = 0x1fc3570;
        a2_2 = 255;
        strncpy(a0_2, a1_1, a2_2);
        v0_1 = 0x1fc3570;
    }
    sw(0x3affc4, s2_1);
    sw(g::mcseq_msgflags, a0);
    sw(0x3affc8, v0_1);
    sw(g::mcseq_state, 3);
    sb(v0_1 + 255, 0);
    return;
}

/// Start loading with the sequencer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseqLoad(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32, mut a5: i32, mut a6: i32) {
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    sw(0x3affe0, a0);
    a0_1 = 0x1fc3670;
    a1_1 = a3;
    sw(0x3affd4, 0);
    sw(0x3affdc, 0);
    sw(0x3affe4, 0);
    strcpy(a0_1, a1_1);
    v1_1 = lw(g::mcseq_flags);
    sw(0x3affe8, a4);
    sw(0x3affec, a5);
    sw(g::mcseq_flags, v1_1 & 6);
    sw(0x3afff0, a6);
    sw(g::mcseq_state, 4);
    if a1 != 0 {
        sw(0x3affbc, a1);
        v0_1 = lbu(0x3affb8);
        sw(0x3affc0, if a2 != 0 { a2 } else { a1 });
        if v0_1 == 0 {
            mcseqWindowOpen();
            a1_2 = g::mcseq_page;
            a0_2 = lw(lw(g::front) + 20);
            frontPageSet(a0_2, a1_2);
            return;
        }
    }
    return;
}

/// Start saving with the sequencer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseqSave(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32, mut a5: i32, mut a6: i32) {
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    sw(0x3affe0, a0);
    sw(0x3affe4, 1);
    a0_1 = 0x1fc3670;
    a1_1 = a3;
    sw(0x3affd4, 0);
    sw(0x3affdc, 0);
    strcpy(a0_1, a1_1);
    v1_1 = lw(g::mcseq_flags);
    sw(0x3affe8, a4);
    sw(0x3affec, a5);
    sw(g::mcseq_flags, v1_1 & 6);
    sw(0x3afff0, a6);
    sw(g::mcseq_state, 4);
    if a1 != 0 {
        sw(0x3affbc, a1);
        v0_1 = lbu(0x3affb8);
        sw(0x3affc0, if a2 != 0 { a2 } else { a1 });
        if v0_1 == 0 {
            mcseqWindowOpen();
            a1_2 = g::mcseq_page;
            a0_2 = lw(lw(g::front) + 20);
            frontPageSet(a0_2, a1_2);
            return;
        }
    }
    return;
}

/// Make the name of a save slot.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseqSlotMakeName(mut a0: i32) -> i32 {
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    s0_1 = a0 >> 2;
    a0_1 = s0_1;
    s4_1 = 0x1fc33d0;
    v0_1 = joyMtapGetSlots(a0_1);
    s1_1 = g::languages;
    a0_2 = lw(lw((lw(g::cur_language) << 2) + s1_1) + 1460);
    v0_2 = strlen(a0_2);
    a0_3 = s4_1;
    a1_1 = 0x3ae8a0;
    a2_1 = lw(lw((lw(g::cur_language) << 2) + s1_1) + 1460);
    sprintf(&[a0_3 as i64, a1_1 as i64, a2_1 as i64]);
    sb(v0_2 + 1 + s4_1, s0_1 + 49);
    s1_2 = v0_2 + 2;
    if ((v0_1 < 2) as i32 ^ 1) != 0 {
        sb(s1_2 + s4_1, 45);
        v0_3 = v0_2 + s4_1;
        sb(v0_3 + 3, (a0 & 3) + 65);
        sb(v0_3 + 4, 0);
    } else {
        sb(s1_2 + s4_1, 0);
    }
    v0_4 = 0x1fc33d0;
    return v0_4;
}

/// State of the sequencer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseqGetState() -> i64 {
    let mut a0_1: i32 = 0;
    let mut v0_1: i64 = 0;
    let mut v1_1: i32 = 0;
    'b1: {
        a0_1 = lw(g::mcseq_state) - 1;
        if (a0_1 as u32) < (24 as u32) {
            match (a0_1 << 2) >> 2 {
                0 | 1 => {
                    v0_1 = lw(0x3affe0) as i64;
                    break 'b1;
                }
                2 => {
                    v1_1 = lw(g::mcseq_msgflags);
                    v0_1 = 1_i64;
                    if (v1_1 & 16) != 0 {
                        break 'b1;
                    }
                    v0_1 = 2_i64;
                    if (v1_1 & 32) != 0 {
                        break 'b1;
                    }
                    if (v1_1 & 64) == 0 {
                        v0_1 = ((v1_1 as i64 as u64) >> 5) as i64 & 4_i64;
                        break 'b1;
                    }
                    v0_1 = 3_i64;
                    break 'b1;
                }
                3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 => {
                }
                22 | 23 => {
                    v0_1 = mcseqTick() as i64;
                    break 'b1;
                }
                _ => unreachable!(),
            }
        }
        v0_1 = -1_i64;
    }
    return v0_1;
}

/// Message of the sequencer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseqGetMsg(mut a0: i32) -> i32 {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut t4_1: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a7_2: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a5_3: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut s2_3: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut s1_4: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a3_6: i32 = 0;
    if (a0 as u32) >= (13 as u32) {
        v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1608);
    } else {
        match (a0 << 2) >> 2 {
            0 => {
                v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1496);
            }
            1 => {
                s1_4 = lw((lw(g::cur_language) << 2) + g::languages);
                v0_9 = mcseqSlotGetLongName();
                a2_6 = lw(s1_4 + 1504);
                a0_6 = 0x3a2000;
                a1_6 = lw(s1_4 + 1500);
                a3_6 = v0_9;
                v0_3 = vafmt(&[a0_6 as i64, a1_6 as i64, a2_6 as i64, a3_6 as i64]);
            }
            2 => {
                s1_3 = g::languages;
                s2_3 = lw((lw(g::cur_language) << 2) + s1_3);
                v0_8 = mcseqSlotGetLongName();
                a2_5 = v0_8;
                a1_5 = lw(s2_3 + 1508);
                a0_5 = 0x3a2010;
                a3_5 = lw(lw((lw(g::cur_language) << 2) + s1_3) + 1512);
                v0_3 = vafmt(&[a0_5 as i64, a1_5 as i64, a2_5 as i64, a3_5 as i64]);
            }
            3 => {
                v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1516);
            }
            4 => {
                v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1520);
            }
            5 => {
                v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1524);
            }
            6 => {
                a0_4 = 0x3ae8b0;
                a3_4 = lw((lw(g::cur_language) << 2) + g::languages);
                a2_4 = lw(a3_4 + 1532);
                a1_4 = lw(a3_4 + 1528);
                v0_3 = vafmt(&[a0_4 as i64, a1_4 as i64, a2_4 as i64, a3_4 as i64]);
            }
            7 => {
                v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1536);
            }
            8 => {
                v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1540);
            }
            9 => {
                v0_3 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1544);
            }
            10 => {
                s1_2 = g::languages;
                s2_2 = lw((lw(g::cur_language) << 2) + s1_2);
                v0_6 = mcseqSlotGetLongName();
                a3_3 = v0_6;
                a2_3 = lw(s2_2 + 1552);
                a0_3 = 0x3a2020;
                a1_3 = lw(s2_2 + 1548);
                v0_7 = lw((lw(g::cur_language) << 2) + s1_2);
                a5_3 = lw(v0_7 + 1560);
                a4_3 = lw(v0_7 + 1556);
                v0_3 = vafmt(&[a0_3 as i64, a1_3 as i64, a2_3 as i64, a3_3 as i64, a4_3 as i64, a5_3 as i64]);
            }
            11 => {
                s1_1 = g::languages;
                s4_1 = lw((lw(g::cur_language) << 2) + s1_1);
                v0_4 = mcseqSlotGetLongName();
                s2_1 = lw((lw(g::cur_language) << 2) + s1_1);
                v0_5 = mcseqGetRequiredSpace();
                a0_2 = 0x3a2030;
                a2_2 = lw(s4_1 + 1568);
                a3_2 = v0_4;
                a4_2 = lw(s2_1 + 1572);
                a1_2 = lw(s4_1 + 1564);
                t4_1 = lw((lw(g::cur_language) << 2) + s1_1);
                a5_2 = v0_5;
                a7_2 = lw(t4_1 + 1580);
                a6_2 = lw(t4_1 + 1576);
                v0_3 = vafmt(&[a0_2 as i64, a1_2 as i64, a2_2 as i64, a3_2 as i64, a4_2 as i64, a5_2 as i64, a6_2 as i64, a7_2 as i64]);
            }
            12 => {
                v0_1 = mcseqSlotGetLongName();
                v0_2 = mcseqGetRequiredSpace();
                a5_1 = v0_2;
                a0_1 = 0x3a2048;
                v1_1 = lw((lw(g::cur_language) << 2) + g::languages);
                a3_1 = v0_1;
                sw(sp, lw(v1_1 + 1604));
                a7_1 = lw(v1_1 + 1600);
                a1_1 = lw(v1_1 + 1584);
                a2_1 = lw(v1_1 + 1588);
                a4_1 = lw(v1_1 + 1592);
                a6_1 = lw(v1_1 + 1596);
                v0_3 = vafmt(&[a0_1 as i64, a1_1 as i64, a2_1 as i64, a3_1 as i64, a4_1 as i64, a5_1 as i64, a6_1 as i64, a7_1 as i64]);
            }
            _ => unreachable!(),
        }
    }
    return v0_3;
}

/// Tick of the sequencer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseqTick() -> i32 {
    let frame = Frame::new(464);
    let sp = frame.sp();
    let mut v1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a0_7: i64 = 0;
    let mut a2_4: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a0_10: i64 = 0;
    let mut a3_4: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a2_11: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut a2_12: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut v0_24: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut a2_13: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut a1_20: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut a0_30: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut a1_21: i32 = 0;
    let mut a2_14: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut a0_31: i32 = 0;
    let mut a2_15: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut a1_23: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut a0_32: i32 = 0;
    let mut v0_28: i32 = 0;
    let mut a1_24: i32 = 0;
    let mut v0_29: i32 = 0;
    let mut a0_33: i32 = 0;
    let mut v0_30: i32 = 0;
    let mut a1_25: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut a0_34: i32 = 0;
    let mut a1_26: i32 = 0;
    let mut a2_16: i32 = 0;
    let mut a0_35: i32 = 0;
    let mut v0_31: i32 = 0;
    'b1: {
        'b2: {
            v1_1 = lw(g::mcseq_state);
            if (v1_1 as u32) < (25 as u32) {
                'b3: {
                    'b4: {
                        'b5: {
                            'b6: {
                                'b7: {
                                    match (v1_1 << 2) >> 2 {
                                        0 => {
                                            v0_31 = mcardGetResult();
                                            sw(0x3afff8, v0_31);
                                            if (v0_31 as u32) < (8 as u32) {
                                                match (v0_31 << 2) >> 2 {
                                                    0 => {
                                                        break 'b2;
                                                    }
                                                    1 => {
                                                        sw(g::mcseq_state, lw(0x3afffc));
                                                        break 'b2;
                                                    }
                                                    2 | 3 | 4 | 5 | 6 | 7 => {
                                                    }
                                                    _ => unreachable!(),
                                                }
                                            }
                                            sw(g::mcseq_state, lw(0x3b0000));
                                            break 'b2;
                                        }
                                        1 => {
                                            v0_3 = lw(0x3affe0);
                                            break 'b1;
                                        }
                                        2 => {
                                            v0_3 = lw(0x3affe0);
                                            sw(g::mcseq_flags, lw(g::mcseq_flags) & -25);
                                            break 'b1;
                                        }
                                        3 => {
                                            v0_3 = 0;
                                            break 'b1;
                                        }
                                        4 => {
                                            v1_13 = lw(0x3b0008);
                                            if v1_13 != -1 {
                                                if lw((v1_13 << 2) + 0x1fc3710) != 1 {
                                                    sw(g::mcseq_state, 9);
                                                    break 'b2;
                                                }
                                            }
                                            'b11: {
                                                a0_32 = 0;
                                                v0_28 = joyMtapGetSlots(a0_32);
                                                a1_24 = v0_28;
                                                if a1_24 < 4 {
                                                    'l12: loop {
                                                        v0_29 = (a1_24 << 2) + 0x1fc3710;
                                                        a1_24 = a1_24 + 1;
                                                        sw(v0_29, 0);
                                                        if a1_24 >= 4 {
                                                            break 'b11;
                                                        }
                                                    }
                                                }
                                            }
                                            'b13: {
                                                a0_33 = 1;
                                                v0_30 = joyMtapGetSlots(a0_33);
                                                a1_25 = v0_30 + 4;
                                                if a1_25 < 8 {
                                                    v1_14 = (a1_25 << 2) + 0x1fc3710;
                                                    'l14: loop {
                                                        sw(v1_14, 0);
                                                        a1_25 = a1_25 + 1;
                                                        v1_14 = v1_14 + 4;
                                                        if a1_25 >= 8 {
                                                            break 'b13;
                                                        }
                                                    }
                                                }
                                            }
                                            a0_34 = 0x1fc3710;
                                            a1_26 = 0x1fc3730;
                                            a2_16 = 0x1fc3750;
                                            mcardCheckCards(a0_34, a1_26, a2_16);
                                            a0_35 = if !(0 | lw(0x3b0008)) != 0 { 9 } else { 5 };
                                            sw(g::mcseq_flags, lw(g::mcseq_flags) | 16);
                                            sw(0x3afffc, a0_35);
                                            sw(0x3b0000, 2);
                                            sw(g::mcseq_state, 0);
                                            break 'b2;
                                        }
                                        5 => {
                                            a3_7 = lw(0x3affd4);
                                            a0_31 = 0;
                                            a2_15 = 0x1fc3710;
                                            a1_22 = 7;
                                            'l15: loop {
                                                v0_27 = lw(a2_15);
                                                a2_15 = a2_15 + 4;
                                                a1_22 = a1_22 - 1;
                                                a0_31 = if v0_27 >= 4 { a0_31 + 1 } else { a0_31 };
                                                if a1_22 < 0 {
                                                    break;
                                                }
                                            }
                                            if a3_7 != 0 {
                                                break 'b3;
                                            }
                                            if a0_31 != 1 {
                                                break 'b3;
                                            }
                                            'b17: {
                                                a1_23 = 0;
                                                if lw(0x1fc3710) < 4 {
                                                    v1_12 = 0x1fc3710;
                                                    a1_23 = a1_23 + 1;
                                                    'l18: loop {
                                                        v1_12 = v1_12 + 4;
                                                        if a1_23 >= 8 {
                                                            break 'b17;
                                                        }
                                                        if lw(v1_12) >= 4 {
                                                            break 'b17;
                                                        }
                                                        a1_23 = a1_23 + 1;
                                                    }
                                                }
                                            }
                                            sw(0x3b0008, a1_23);
                                            sw(g::mcseq_state, 9);
                                            break 'b2;
                                        }
                                        6 => {
                                            sw(g::mcseq_flags, lw(g::mcseq_flags) & -17);
                                            a2_12 = 0x1fc36f0;
                                            v1_10 = 0x1fc3710;
                                            a1_16 = 7;
                                            'l19: loop {
                                                v0_21 = lw(v1_10);
                                                a1_16 = a1_16 - 1;
                                                v1_10 = v1_10 + 4;
                                                sw(a2_12, (v0_21 < 4) as i32 ^ 1);
                                                a2_12 = a2_12 + 4;
                                                if a1_16 < 0 {
                                                    break;
                                                }
                                            }
                                            sw(0x3b0004, -1);
                                            sw(g::mcseq_state, 7);
                                            sw(0x3affd0, 0);
                                            break 'b4;
                                        }
                                        7 => {
                                            break 'b4;
                                        }
                                        8 => {
                                            v0_19 = mcardGetResult();
                                            if v0_19 == 0 {
                                                break 'b2;
                                            }
                                            v0_20 = lw(0x3affd0);
                                            if v0_19 == 1 {
                                                a1_15 = lw(0x3b0004) << 2;
                                                sw(a1_15 + 0x1fc3710, lw(a1_15 + 0x1fc36f0));
                                                v0_20 = lw(0x3affd0);
                                            }
                                            v1_9 = lw(0x3afffc);
                                            sw(0x3affd0, v0_20 + 1);
                                            sw(g::mcseq_state, v1_9);
                                            break 'b2;
                                        }
                                        9 => {
                                            a0_25 = lw(0x3b0008);
                                            a1_14 = a0_25 & 3;
                                            a0_26 = a0_25 >> 2;
                                            mcardSelectCard(a0_26, a1_14);
                                            if lw((lw(0x3b0008) << 2) + 0x1fc3710) != 5 {
                                                v1_8 = lw(g::mcseq_flags);
                                                sw(g::mcseq_flags, v1_8 & -17);
                                                if (v1_8 & 6) == 0 {
                                                    sw(g::mcseq_state, 12);
                                                    break 'b2;
                                                }
                                                sw(g::mcseq_state, 16);
                                                break 'b2;
                                            }
                                            if lw(0x3affe4) == 0 {
                                                if (lw(g::mcseq_flags) & 32) == 0 {
                                                    v1_6 = 6;
                                                    break 'b5;
                                                }
                                            }
                                            sw(g::mcseq_state, 10);
                                            break 'b2;
                                        }
                                        10 => {
                                            s2_1 = g::languages;
                                            s3_1 = 0x1fc33f0;
                                            s1_2 = lw((lw(g::cur_language) << 2) + s2_1);
                                            v0_14 = mcseqSlotGetLongName();
                                            a2_11 = v0_14;
                                            a1_11 = lw(s1_2 + 1464);
                                            a0_20 = 0x3a1eb0;
                                            v0_15 = lw((lw(g::cur_language) << 2) + s2_1);
                                            a4_3 = lw(v0_15 + 1472);
                                            a3_5 = lw(v0_15 + 1468);
                                            v0_16 = vafmt(&[a0_20 as i64, a1_11 as i64, a2_11 as i64, a3_5 as i64, a4_3 as i64]);
                                            a1_12 = v0_16;
                                            a0_21 = s3_1;
                                            strcpy(a0_21, a1_12);
                                            a0_22 = lw(0x3affdc);
                                            v1_7 = lw(g::cur_language) << 2;
                                            sw(0x3affc4, s3_1);
                                            sw(0x3affc8, lw(lw(v1_7 + s2_1) + 1476));
                                            v0_17 = joyNewBut(a0_22);
                                            a0_23 = 140;
                                            if (v0_17 & 32) == 0 {
                                                a0_24 = lw(0x3affdc);
                                                v0_18 = joyNewBut(a0_24);
                                                if (v0_18 & 16) == 0 {
                                                    break 'b2;
                                                }
                                                break 'b3;
                                            }
                                            a1_13 = 0;
                                            soundStart(a0_23, a1_13);
                                            sw(g::mcseq_flags, lw(g::mcseq_flags) | 8);
                                            mcardFormat();
                                            sw((lw(0x3b0008) << 2) + 0x1fc3710, 1);
                                            sw(0x3afffc, 11);
                                            sw(0x3b0000, 2);
                                            sw(g::mcseq_state, 0);
                                            break 'b2;
                                        }
                                        11 => {
                                            sw(g::mcseq_flags, lw(g::mcseq_flags) & -9 & -17);
                                            a0_19 = 0x1fc3710;
                                            a1_10 = 0x1fc3730;
                                            a2_10 = 0x1fc3750;
                                            mcardCheckCards(a0_19, a1_10, a2_10);
                                            v0_5 = 9;
                                            break 'b7;
                                        }
                                        12 => {
                                            a0_16 = sp;
                                            s0_3 = 0x1fc3670;
                                            a1_8 = s0_3;
                                            strcpy(a0_16, a1_8);
                                            a1_9 = s0_3;
                                            a0_17 = sp + 21;
                                            strcpy(a0_17, a1_9);
                                            sb(sp + 42, 0);
                                            a0_18 = sp;
                                            mcardFileExists(a0_18);
                                            v0_5 = 16;
                                            v1_4 = 13;
                                            break 'b6;
                                        }
                                        13 => {
                                            if lw(0x3afff8) == 5 {
                                                if lw(0x3affe4) != 0 {
                                                    a1_7 = 0x3b000c;
                                                    a0_15 = 0x3ae8a8;
                                                    mcardGetEntSpace(a0_15, a1_7);
                                                    v0_5 = 14;
                                                    break 'b7;
                                                }
                                            }
                                        }
                                        14 => {
                                            a4_2 = 0x1fc3670;
                                            a0_10 = ld(a4_2 + 8);
                                            a3_4 = lw(a4_2 + 16);
                                            sd(sp + 128, ld(a4_2));
                                            sd(sp + 136, a0_10);
                                            sw(sp + 144, a3_4);
                                            sb(sp + 148, lb(a4_2 + 20));
                                            sw(0x3b0010, 3);
                                            a1_5 = 0x3a1ec0;
                                            sb(sp + 149, 0);
                                            a0_11 = sp + 128;
                                            a2_6 = 21;
                                            v0_10 = strncmp(a0_11, a1_5, a2_6);
                                            a0_12 = 1;
                                            if v0_10 != 0 {
                                                s1_1 = 1;
                                                v0_13 = mcardIconClusters(a0_12);
                                                s0_2 = 0;
                                                sw(0x3b0010, lw(0x3b0010) + v0_13);
                                                v0_12 = mmfileMaxSize();
                                                a2_7 = lw(0x3b0010);
                                            } else {
                                                a0_13 = 0;
                                                s1_1 = 0;
                                                v0_11 = mcardIconClusters(a0_13);
                                                s0_2 = 1;
                                                sw(0x3b0010, lw(0x3b0010) + v0_11);
                                                v0_12 = signonFileSize();
                                                a2_7 = lw(0x3b0010);
                                            }
                                            a2_8 = a2_7 + (((v0_12 + 1023) as u32) >> 10) as i32;
                                            sw(0x3b0010, a2_8);
                                            if lw((lw(0x3b0008) << 2) + 0x1fc3730) < a2_8 {
                                                v1_6 = 7;
                                                break 'b5;
                                            }
                                            a0_14 = s1_1;
                                            a2_9 = s0_2;
                                            a1_6 = 1;
                                            mcardMkdir(a0_14, a1_6, a2_9);
                                            v0_5 = 15;
                                            break 'b7;
                                        }
                                        15 => {
                                            v1_5 = (lw(0x3b0008) << 2) + 0x1fc3710;
                                            sw(g::mcseq_flags, lw(g::mcseq_flags) | 4);
                                            sw(v1_5, 1);
                                            sw(g::mcseq_state, 4);
                                            break 'b2;
                                        }
                                        16 => {
                                            if lw(0x3affe4) != 0 {
                                                a3_3 = 0x1fc3670;
                                                a0_7 = ld(a3_3 + 8);
                                                a2_4 = lw(a3_3 + 16);
                                                sd(sp + 256, ld(a3_3));
                                                sd(sp + 264, a0_7);
                                                sw(sp + 272, a2_4);
                                                sb(sp + 276, lb(a3_3 + 20));
                                                a1_4 = 0x3a1ec0;
                                                sb(sp + 277, 0);
                                                a0_8 = sp + 256;
                                                a2_5 = 21;
                                                v0_6 = strncmp(a0_8, a1_4, a2_5);
                                                if v0_6 != 0 {
                                                    s0_1 = 1;
                                                    v0_9 = mmfileMaxSize();
                                                    v0_8 = v0_9 + 1023;
                                                } else {
                                                    s0_1 = 0;
                                                    v0_7 = signonFileSize();
                                                    v0_8 = v0_7 + 1023;
                                                }
                                                sw(0x3b0010, ((v0_8 as u32) >> 10) as i32);
                                                a0_9 = s0_1;
                                                mcardCheckDir(a0_9);
                                                v0_5 = 17;
                                                break 'b7;
                                            }
                                            sw(g::mcseq_state, 17);
                                            break 'b2;
                                        }
                                        17 => {
                                            v1_3 = lw(0x3affe4);
                                            sw(g::mcseq_flags, lw(g::mcseq_flags) | 2);
                                            if v1_3 == 0 {
                                                a1_3 = lw(0x3affe8);
                                                a2_3 = lw(0x3affec);
                                                a0_6 = 0x1fc3670;
                                                a3_2 = lw(0x3afff0);
                                                mcardFileRead(a0_6, a1_3, a2_3, a3_2);
                                                v0_5 = 1;
                                                break 'b7;
                                            }
                                            if v1_3 == 1 {
                                                a0_5 = 0x1fc3670;
                                                mcardFileExists(a0_5);
                                                v0_5 = 20;
                                                v1_4 = 19;
                                                break 'b6;
                                            }
                                        }
                                        18 => {
                                            break 'b2;
                                        }
                                        19 => {
                                            sw(0x3afff4, 0);
                                            sw(g::mcseq_state, 21);
                                            break 'b2;
                                        }
                                        20 => {
                                            sw(0x3afff4, 1);
                                            sw(g::mcseq_state, 21);
                                            break 'b2;
                                        }
                                        21 => {
                                            a1_2 = lw(0x3affe8);
                                            a2_2 = lw(0x3affec);
                                            a0_4 = 0x1fc3670;
                                            a3_1 = lw(0x3afff4);
                                            a4_1 = lw(0x3afff0);
                                            mcardFileWrite(a0_4, a1_2, a2_2, a3_1, a4_1);
                                            v0_5 = 22;
                                            break 'b7;
                                        }
                                        22 => {
                                            v0_4 = lw(0x3b0008) << 2;
                                            sw(g::mcseq_state, 1);
                                            sw(v0_4 + 0x1fc3710, 1);
                                            break 'b2;
                                        }
                                        23 => {
                                            a0_1 = lw(0x3affd0);
                                            a0_2 = a0_1 + 1;
                                            if a0_1 == 60 {
                                                a0_3 = 0x1fc3710;
                                                a2_1 = 0x1fc3750;
                                                sw((lw(0x3b0008) << 2) + a0_3, 1);
                                                a1_1 = 0;
                                                mcardCheckCards(a0_3, a1_1, a2_1);
                                                sw(0x3affd0, 0);
                                                sw(g::mcseq_state, 24);
                                                a0_2 = 1;
                                            }
                                            sw(0x3affd0, a0_2);
                                            break 'b2;
                                        }
                                        24 => {
                                            v0_1 = mcardGetResult();
                                            sw(0x3afff8, v0_1);
                                            if v0_1 == 0 {
                                                break 'b2;
                                            }
                                            if v0_1 == 1 {
                                                v0_2 = lw(0x3b0008) << 2;
                                                sw(g::mcseq_state, 23);
                                                sw(0x3affcc, v0_1);
                                                v1_2 = lw(v0_2 + 0x1fc3750);
                                                if v1_2 == v0_1 {
                                                    v0_3 = 6;
                                                    break 'b1;
                                                }
                                                if v1_2 >= 2 {
                                                    if v1_2 == 2 {
                                                        v0_3 = 7;
                                                        break 'b1;
                                                    }
                                                    if v1_2 == 3 {
                                                        v0_3 = 8;
                                                        break 'b1;
                                                    }
                                                } else if v1_2 == 0 {
                                                    v0_3 = 5;
                                                    break 'b1;
                                                }
                                            }
                                            sw(0x3affcc, 1);
                                            v0_3 = 3;
                                            sw(g::mcseq_state, 23);
                                            break 'b1;
                                        }
                                        _ => unreachable!(),
                                    }
                                    sw(g::mcseq_state, 2);
                                    break 'b2;
                                }
                                v1_4 = 2;
                            }
                            sw(0x3afffc, v0_5);
                            sw(0x3b0000, v1_4);
                            sw(g::mcseq_state, 0);
                            break 'b2;
                        }
                        sw(g::mcseq_state, 2);
                        sw(0x3afff8, v1_6);
                        break 'b2;
                    }
                    a1_17 = 7;
                    v0_22 = 0x1fc370c;
                    'l27: loop {
                        sw(v0_22, 2);
                        a1_17 = a1_17 - 1;
                        v0_22 = v0_22 - 4;
                        if a1_17 < 0 {
                            break;
                        }
                    }
                    'b29: {
                        a0_27 = 0;
                        v0_23 = joyMtapGetSlots(a0_27);
                        a1_18 = v0_23;
                        if a1_18 < 4 {
                            v1_11 = (a1_18 << 2) + 0x1fc36f0;
                            'l30: loop {
                                sw(v1_11, 0);
                                a1_18 = a1_18 + 1;
                                v1_11 = v1_11 + 4;
                                if a1_18 >= 4 {
                                    break 'b29;
                                }
                            }
                        }
                    }
                    'b31: {
                        a0_28 = 1;
                        v0_24 = joyMtapGetSlots(a0_28);
                        a1_19 = v0_24 + 4;
                        a2_13 = lw(0x3b0004);
                        if a1_19 < 8 {
                            'l32: loop {
                                v0_25 = (a1_19 << 2) + 0x1fc36f0;
                                a1_19 = a1_19 + 1;
                                sw(v0_25, 0);
                                if a1_19 >= 8 {
                                    break 'b31;
                                }
                            }
                        }
                    }
                    a1_20 = lw(0x3b0004);
                    a0_29 = 0;
                    'l33: loop {
                        v0_26 = a2_13 << 2;
                        if a0_29 >= 8 {
                            break;
                        }
                        a2_13 = a2_13 + 1;
                        a1_20 = a2_13;
                        if a2_13 >= 8 {
                            a2_13 = 0;
                            a1_20 = 0;
                        }
                        v0_26 = a2_13 << 2;
                        if lw((a2_13 << 2) + 0x1fc36f0) != 0 {
                            break;
                        }
                        a0_29 = a0_29 + 1;
                    }
                    a0_30 = 0x1fc36f0;
                    sw(0x3b0004, a1_20);
                    a3_6 = v0_26 + a0_30;
                    if lw(a3_6) == 0 {
                        break 'b2;
                    }
                    sw(a3_6, 1);
                    a1_21 = 0x1fc3730;
                    a2_14 = 0x1fc3750;
                    mcardCheckCards(a0_30, a1_21, a2_14);
                    sw(0x3afffc, 7);
                    sw(g::mcseq_state, 8);
                    break 'b2;
                }
                sw(g::mcseq_state, 6);
            }
        }
        v0_3 = -1;
    }
    return v0_3;
}

/// Tick of the page of the sequencer.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcseq_pageTick(mut a0: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let mut s2_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s7_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a4_1: i64 = 0;
    let mut a5_1: i64 = 0;
    let mut v0_2: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_4: i64 = 0;
    let mut a1_4: i64 = 0;
    let mut v0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut s3_2: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut v1_3: i64 = 0;
    let mut a0_6: i64 = 0;
    let mut v1_4: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut s2_3: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut a4_2: i64 = 0;
    let mut a5_2: i64 = 0;
    let mut v0_8: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut bc_1: bool = false;
    let mut v0_21: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut bc_2: bool = false;
    let mut v0_23: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut v0_24: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut v0_28: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut a6_3: i32 = 0;
    let mut a4_3: i64 = 0;
    let mut a5_3: i64 = 0;
    let mut v1_11: i32 = 0;
    let mut v0_29: i32 = 0;
    let mut v0_30: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut s3_3: i32 = 0;
    let mut s2_4: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut s3_4: i32 = 0;
    let mut a5_4: i32 = 0;
    let mut a3_2: i64 = 0;
    let mut a4_4: i64 = 0;
    let mut a1_15: i32 = 0;
    let mut s2_5: i32 = 0;
    let mut a0_30: i32 = 0;
    let mut v0_31: i64 = 0;
    let mut v1_12: i64 = 0;
    let mut a2_10: i32 = 0;
    let mut a0_31: i32 = 0;
    let mut a1_16: i32 = 0;
    'b1: {
        'b2: {
            'b3: {
                s2_1 = 0x1fc3770;
                s3_1 = lw(a0 + 8);
                mcseqTick();
                mcseqWindowSet();
                v1_1 = lw(g::mcseq_state);
                if (v1_1 as u32) < (11 as u32) {
                    match (v1_1 << 2) >> 2 {
                        0 | 1 | 2 | 4 | 5 | 9 => {
                        }
                        3 | 10 => {
                            v1_8 = lw(g::mcseq_msgflags);
                            v0_19 = v1_8 & 2;
                            if (v1_8 & 1) != 0 {
                                a0_21 = lw(0x3affdc);
                                v0_20 = joyNewBut(a0_21);
                                v1_8 = lw(g::mcseq_msgflags);
                                if (v0_20 & 32) != 0 {
                                    v1_8 = v1_8 | 16;
                                    sw(g::mcseq_msgflags, v1_8);
                                }
                                v0_19 = v1_8 & 2;
                            }
                            bc_1 = v0_19 == 0;
                            v0_21 = v1_8 & 4;
                            if !bc_1 {
                                a0_22 = lw(0x3affdc);
                                v0_22 = joyNewBut(a0_22);
                                v1_8 = lw(g::mcseq_msgflags);
                                if (v0_22 & 64) != 0 {
                                    v1_8 = v1_8 | 32;
                                    sw(g::mcseq_msgflags, v1_8);
                                }
                                v0_21 = v1_8 & 4;
                            }
                            bc_2 = v0_21 == 0;
                            v0_23 = v1_8 & 8;
                            if !bc_2 {
                                a0_23 = lw(0x3affdc);
                                v0_24 = joyNewBut(a0_23);
                                v1_9 = lw(g::mcseq_msgflags);
                                if (v0_24 & 16) != 0 {
                                    v1_9 = v1_9 | 64;
                                    sw(g::mcseq_msgflags, v1_9);
                                }
                                v0_23 = v1_9 & 8;
                            }
                            if v0_23 != 0 {
                                a0_24 = lw(0x3affdc);
                                v0_25 = joyNewBut(a0_24);
                                v0_26 = lw(g::mcseq_msgflags);
                                if (v0_25 & 128) != 0 {
                                    sw(g::mcseq_msgflags, v0_26 | 128);
                                }
                            }
                            a0_25 = lw(0x3affdc);
                            joyClearAll(a0_25);
                            a0_26 = lw(0x3affc4);
                            if a0_26 == 0 {
                                v0_13 = lw(a0 + 8);
                                break 'b1;
                            }
                            v0_27 = textWidth(a0_26);
                            a1_13 = v0_27 + 8;
                            a2_7 = g::zoomboxes;
                            a0_27 = lw(a2_7 + 28);
                            if a0_27 >= a1_13 {
                                break 'b2;
                            }
                            v0_28 = a1_13 - a0_27;
                            if a1_13 >= lw(g::window + 12) {
                                break 'b2;
                            }
                            v1_10 = lw(a2_7 + 20);
                            sw(a2_7 + 28, a1_13);
                            sw(a2_7 + 20, v1_10 - ((v0_28 + ((v0_28 as u32) >> 31) as i32) >> 1));
                            break 'b2;
                        }
                        6 => {
                            v0_18 = 1;
                            break 'b3;
                        }
                        7 | 8 => {
                            v1_2 = lw(0x3affd4);
                            a2_1 = lhu(a0 + 4);
                            a1_1 = 0x1fc3710;
                            a3_1 = lw(0x3affbc);
                            s5_1 = 0;
                            s1_1 = 0;
                            s7_1 = s3_1 + 24;
                            s0_1 = s2_1 + 32;
                            a0_1 = sp + 16;
                            'l12: loop {
                                if lw(a1_1) < 4 {
                                    s1_1 = s1_1 + 1;
                                } else {
                                    sw(a0_1, s1_1);
                                    s5_1 = s5_1 + 1;
                                    a0_1 = a0_1 + 4;
                                    s1_1 = s1_1 + 1;
                                }
                                a1_1 = a1_1 + 4;
                                if s1_1 >= 8 {
                                    break;
                                }
                            }
                            s4_1 = s5_1 + v1_2;
                            v0_1 = (s4_1 < 4) as i32;
                            if ((a2_1 - 1) as u32) >= (s4_1 as u32) {
                                sh(a0 + 4, 1);
                                v0_1 = (s4_1 < 4) as i32;
                            }
                            if v0_1 != 0 {
                                sh(sp, 0);
                            } else {
                                a0_2 = g::zoomboxes;
                                a1_2 = lw(a0_2 + 24) - (((s4_1 - 3) * 10) >> 1);
                                sw(a0_2 + 32, lw(a0_2 + 32) + (s4_1 - 3) * 10);
                                sw(a0_2 + 24, a1_2);
                                sh(sp, 0);
                            }
                            a0_3 = s3_1;
                            a2_2 = s2_1;
                            a1_3 = sp;
                            sh(sp + 2, lhu(g::zoomboxes + 24) + 10);
                            a6_1 = g::mi_msg;
                            a4_1 = ld(a6_1 + 8);
                            a5_1 = ld(a6_1 + 16);
                            sd(s2_1, ld(a6_1));
                            sd(s2_1 + 8, a4_1);
                            sd(s2_1 + 16, a5_1);
                            sd(s2_1 + 24, ld(a6_1 + 24));
                            sw(s2_1, a3_1);
                            sh(s2_1 + 10, lhu(s2_1 + 10) | 256);
                            std_menuitemmake(a0_3, a1_3, a2_2);
                            v0_2 = lhu(sp + 2) + 15;
                            sh(sp + 2, v0_2);
                            if s4_1 == 0 {
                                sh(sp + 2, v0_2 + 20);
                            }
                            s2_2 = s0_1;
                            a2_3 = g::mi_msg;
                            a0_4 = ld(a2_3 + 8);
                            a1_4 = ld(a2_3 + 16);
                            sd(s2_2, ld(a2_3));
                            sd(s2_2 + 8, a0_4);
                            sd(s2_2 + 16, a1_4);
                            sd(s2_2 + 24, ld(a2_3 + 24));
                            v0_3 = lw(g::cur_language);
                            if s5_1 == 0 {
                                v0_5 = lw(g::cur_language);
                                if lw(0x3affd4) == 0 {
                                    v0_4 = lw(lw((v0_5 << 2) + g::languages) + 1908);
                                } else {
                                    v0_4 = lw(lw((v0_5 << 2) + g::languages) + 1880);
                                }
                            } else {
                                v0_4 = lw(lw((v0_3 << 2) + g::languages) + 1884);
                            }
                            'b19: {
                                sw(s0_1, v0_4);
                                a0_5 = s7_1;
                                a2_4 = s0_1;
                                a1_5 = sp;
                                std_menuitemmake(a0_5, a1_5, a2_4);
                                s3_2 = s7_1 + 24;
                                s1_2 = 0;
                                sh(sp + 2, lhu(sp + 2) + 20);
                                if s4_1 > 0 {
                                    'l20: loop {
                                        s2_2 = s2_2 + 32;
                                        a1_6 = g::mi_card;
                                        v1_3 = ld(a1_6 + 8);
                                        a0_6 = ld(a1_6 + 16);
                                        sd(s2_2, ld(a1_6));
                                        sd(s2_2 + 8, v1_3);
                                        sd(s2_2 + 16, a0_6);
                                        sd(s2_2 + 24, ld(a1_6 + 24));
                                        v1_4 = s1_2 << 2;
                                        if s1_2 >= s5_1 {
                                            sw(s2_2, lw(((s1_2 - s5_1) << 2) + 0x3affd8));
                                        } else {
                                            a0_7 = lw(sp + v1_4 + 16);
                                            s0_2 = (s1_2 << 5) + 0x1fc38f0;
                                            v0_6 = mcseqSlotMakeName(a0_7);
                                            a1_7 = v0_6;
                                            a0_8 = s0_2;
                                            strcpy(a0_8, a1_7);
                                            sw(s2_2, s0_2);
                                        }
                                        s0_3 = s1_2 + 1;
                                        a0_9 = s3_2;
                                        sh(s2_2 + 8, s0_3);
                                        a1_8 = sp;
                                        a2_5 = s2_2;
                                        s1_2 = s0_3;
                                        std_menuitemmake(a0_9, a1_8, a2_5);
                                        s3_2 = s3_2 + 24;
                                        sh(sp + 2, lhu(sp + 2) + 10);
                                        if s1_2 >= s4_1 {
                                            break 'b19;
                                        }
                                    }
                                }
                            }
                            v0_7 = g::zoomboxes;
                            s2_3 = s2_2 + 32;
                            sh(sp + 2, lhu(v0_7 + 24) + lhu(v0_7 + 32) - 30);
                            a6_2 = g::mi_msg;
                            a4_2 = ld(a6_2 + 8);
                            a5_2 = ld(a6_2 + 16);
                            sd(s2_3, ld(a6_2));
                            sd(s2_3 + 8, a4_2);
                            sd(s2_3 + 16, a5_2);
                            sd(s2_3 + 24, ld(a6_2 + 24));
                            v0_8 = lw(g::cur_language);
                            if s4_1 == 0 {
                                v0_9 = lw(lw((v0_8 << 2) + g::languages) + 1484);
                            } else {
                                v0_9 = lw(lw((v0_8 << 2) + g::languages) + 1480);
                            }
                            'b23: {
                                a0_10 = s3_2;
                                sw(s2_3, v0_9);
                                a2_6 = s2_3;
                                a1_9 = sp;
                                std_menuitemmake(a0_10, a1_9, a2_6);
                                s3_1 = s3_2 + 24;
                                a0_11 = lw(0x3affdc);
                                v0_10 = joyNewBut(a0_11);
                                if (v0_10 & 0x1000) != 0 {
                                    v1_5 = lhu(a0 + 4);
                                    v0_11 = v1_5 - 1;
                                    if (v1_5 as u32) >= (2 as u32) {
                                        break 'b23;
                                    }
                                }
                                a0_12 = lw(0x3affdc);
                                v0_12 = joyNewBut(a0_12);
                                if (v0_12 & 0x4000) != 0 {
                                    v1_6 = lhu(a0 + 4);
                                    v0_11 = v1_6 + 1;
                                    if v1_6 < s4_1 {
                                        break 'b23;
                                    }
                                }
                                a0_14 = lw(0x3affdc);
                                v0_14 = joyNewBut(a0_14);
                                if (v0_14 & 64) == 0 {
                                    a0_19 = lw(0x3affdc);
                                    v0_16 = joyNewBut(a0_19);
                                    a0_20 = lw(g::cur_language);
                                    if (v0_16 & 16) == 0 {
                                        break 'b2;
                                    }
                                    sw(g::mcseq_state, 2);
                                    v0_17 = lw((a0_20 << 2) + g::languages);
                                    sw(0x3afff8, 8);
                                    sw(0x3affc4, lw(v0_17 + 1488));
                                    break 'b2;
                                }
                                a0_15 = lw(0x3affd4);
                                if s5_1 != 0 {
                                    a0_16 = 140;
                                    if ((lhu(a0 + 4) - 1) as u32) < (s5_1 as u32) {
                                        a1_11 = 0;
                                        soundStart(a0_16, a1_11);
                                        a0_17 = lhu(a0 + 4);
                                        sw(0x3afffc, 9);
                                        sw(g::mcseq_state, 8);
                                        sw(0x3b0008, lw(sp + ((a0_17 - 1) << 2) + 16));
                                        break 'b2;
                                    }
                                    a0_15 = lw(0x3affd4);
                                }
                                v1_7 = s5_1 + 1;
                                if a0_15 <= 0 {
                                    break 'b2;
                                }
                                v0_15 = (((lhu(a0 + 4) - v1_7) as u32) < (a0_15 as u32)) as i32;
                                a0_18 = 140;
                                if v0_15 == 0 {
                                    break 'b2;
                                }
                                a1_12 = 0;
                                soundStart(a0_18, a1_12);
                                sw(0x3b0008, -1);
                                sw(g::mcseq_state, 1);
                                sw(0x3afff8, 9);
                                break 'b2;
                            }
                            a0_13 = 141;
                            sh(a0 + 4, v0_11);
                            a1_10 = 0;
                            soundStart(a0_13, a1_10);
                            v0_13 = lw(a0 + 8);
                            break 'b1;
                        }
                        _ => unreachable!(),
                    }
                }
                'b27: {
                    sh(sp, 0);
                    sh(sp + 2, lhu(g::zoomboxes + 24) + 10);
                    a6_3 = g::mi_msg;
                    a4_3 = ld(a6_3 + 8);
                    a5_3 = ld(a6_3 + 16);
                    sd(s2_1, ld(a6_3));
                    sd(s2_1 + 8, a4_3);
                    sd(s2_1 + 16, a5_3);
                    sd(s2_1 + 24, ld(a6_3 + 24));
                    v1_11 = lw(g::mcseq_flags);
                    v0_29 = lw(0x3affbc);
                    if (v1_11 & 16) == 0 {
                        v0_30 = lw(g::cur_language);
                        if (v1_11 & 8) != 0 {
                            sw(s2_1, lw(lw((v0_30 << 2) + g::languages) + 1904));
                            break 'b27;
                        }
                        v0_29 = lw(0x3affc0);
                    }
                    sw(s2_1, v0_29);
                }
                a0_28 = s3_1;
                a2_8 = s2_1;
                a1_14 = sp;
                s0_4 = g::zoomboxes;
                sh(s2_1 + 10, lhu(s2_1 + 10) | 256);
                s3_3 = s3_1 + 24;
                std_menuitemmake(a0_28, a1_14, a2_8);
                s2_4 = s2_1 + 32;
                s1_3 = g::languages;
                a2_9 = s2_4;
                a0_29 = s3_3;
                sh(sp + 2, lhu(s0_4 + 24) + 45);
                s3_4 = s3_3 + 24;
                a5_4 = g::mi_msg;
                a3_2 = ld(a5_4 + 8);
                a4_4 = ld(a5_4 + 16);
                sd(s2_4, ld(a5_4));
                sd(s2_4 + 8, a3_2);
                sd(s2_4 + 16, a4_4);
                sd(s2_4 + 24, ld(a5_4 + 24));
                a1_15 = sp;
                sw(s2_4, lw(lw((lw(g::cur_language) << 2) + s1_3) + 1492));
                std_menuitemmake(a0_29, a1_15, a2_9);
                s2_5 = s2_4 + 32;
                a0_30 = g::mi_msg;
                v0_31 = ld(a0_30 + 8);
                v1_12 = ld(a0_30 + 16);
                sd(s2_5, ld(a0_30));
                sd(s2_5 + 8, v0_31);
                sd(s2_5 + 16, v1_12);
                sd(s2_5 + 24, ld(a0_30 + 24));
                a2_10 = s2_5;
                a0_31 = s3_4;
                a1_16 = sp;
                s3_1 = s3_4 + 24;
                sw(s2_5, lw(lw((lw(g::cur_language) << 2) + s1_3) + 1912));
                sh(sp + 2, lhu(s0_4 + 24) + lhu(s0_4 + 32) - 30);
                std_menuitemmake(a0_31, a1_16, a2_10);
                v0_18 = 666;
            }
            sh(a0 + 4, v0_18);
        }
        v0_13 = lw(a0 + 8);
    }
    sw(a0 + 12, ((s3_1 - v0_13) * -0x55555555) >> 3);
    return;
}
