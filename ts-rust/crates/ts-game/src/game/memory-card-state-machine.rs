//! Functions: the memory card state machine and the directory helpers it uses.

use crate::*;

/// Check the directory of the memory card.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcardCheckDir(mut a0: i32) {
    let mut s1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    if a0 != 0 {
        s1_1 = 0x39a0a8;
    } else {
        s1_1 = 0x39a090;
    }
    a0_1 = a0;
    makeIconSys(a0_1);
    a1_1 = s1_1;
    a0_2 = 0x1fb2940;
    strcpy(a0_2, a1_1);
    sw(0x3afe4c, a0);
    sw(g::mcardstate, 16);
    sw(0x3afe68, 0);
    sw(g::mcardresult, 0);
    return;
}

/// Reset the state of the memory card.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcardReset() {
    let mut a0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut v0_2: i32 = 0;
    a0_1 = 0x399c78;
    sw(0x3afe50, 0x399c20);
    sw(0x3afe54, 0x399c68);
    a2_1 = 0x3afe60;
    a1_1 = 1;
    v0_1 = fileLoad(a0_1, a1_1, a2_1);
    sw(0x3afe58, v0_1);
    a0_2 = 0x399c90;
    a2_2 = 0x3afe64;
    a1_2 = 1;
    v0_2 = fileLoad(a0_2, a1_2, a2_2);
    sw(0x3afe5c, v0_2);
    sw(g::mcardstate, 0);
    return;
}

/// Make the directory of the game on the memory card.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcardMkdir(mut a0: i32, mut a1: i32, mut a2: i32) {
    let mut s1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    if a0 != 0 {
        s1_1 = 0x39a0a8;
    } else {
        s1_1 = 0x39a090;
    }
    a0_1 = a0;
    makeIconSys(a0_1);
    a1_1 = s1_1;
    a0_2 = 0x1fb2940;
    strcpy(a0_2, a1_1);
    sw(0x3afe4c, a0);
    sw(g::mkdir_overwriteok, a1);
    sw(g::mkdir_copyprotect, a2);
    sw(g::mcardstate, 14);
    sw(0x3afe68, 0);
    sw(g::mcardresult, 0);
    return;
}

/// Read the details of the root directory of the memory card.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn GetRootDirDetails(mut a0: i32, mut a1: i32) {
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v0_2: i32 = 0;
    if lw(g::readrootdir_6) == 0 {
        a0_1 = 11;
        a1_1 = 16;
        cd_read(a0_1, a1_1);
        sw(g::readrootdir_6, 1);
    }
    a0_2 = 16;
    a1_2 = 158;
    v0_1 = makeInt(a0_2, a1_2);
    sw(a0, v0_1);
    a0_3 = 16;
    a1_3 = 166;
    v0_2 = makeInt(a0_3, a1_3);
    sw(a1, (if -1 < v0_2 { v0_2 } else { v0_2 + 2047 }) >> 11);
    return;
}

/// Whether a file is in the directory of the memory card.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn IsFileInDir(mut a0: i32, mut a1: i32, mut a2: i32) -> i64 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut a1_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v0_4: i64 = 0;
    let mut v0_5: i32 = 0;
    'b1: {
        'b2: {
            a1_1 = lw(a1);
            s4_1 = 0;
            if a1_1 > 0 {
                v1_1 = lw(a0);
                'l3: loop {
                    s3_1 = v1_1 + s4_1;
                    s1_1 = ((s3_1 - 16) << 11) + g::loadbuf;
                    s2_1 = 0;
                    if lbu(s1_1) != 0 {
                        s0_1 = s1_1;
                        v1_2 = lbu(s0_1 + 32);
                        'l6: loop {
                            'b8: {
                                a1_2 = s2_1 + 33;
                                a2_1 = s2_1 + (v1_2 + 32);
                                a3_1 = 0;
                                if a2_1 >= a1_2 {
                                    'l9: loop {
                                        a0_1 = sp + a3_1;
                                        v1_3 = lbu(s1_1 + a1_2);
                                        a1_2 = a1_2 + 1;
                                        a3_1 = a3_1 + 1;
                                        sb(a0_1, v1_3);
                                        if a2_1 < a1_2 {
                                            break 'b8;
                                        }
                                    }
                                }
                            }
                            a0_2 = sp;
                            sb(sp + a3_1, 0);
                            a1_3 = a2;
                            v0_1 = strcmp(a0_2, a1_3);
                            if v0_1 == 0 {
                                break 'l3;
                            }
                            v0_5 = lbu(s0_1);
                            s2_1 = s2_1 + v0_5;
                            s0_1 = s1_1 + s2_1;
                            if lbu(s0_1) == 0 {
                                break;
                            }
                            v1_2 = lbu(s0_1 + 32);
                        }
                        a1_1 = lw(a1);
                    }
                    s4_1 = s4_1 + 1;
                    if s4_1 >= a1_1 {
                        break 'b2;
                    }
                    v1_1 = lw(a0);
                }
                a0_3 = s3_1;
                a1_4 = s2_1 + 2;
                v0_2 = makeInt(a0_3, a1_4);
                a0_4 = s3_1;
                a1_5 = s2_1 + 10;
                v0_3 = makeInt(a0_4, a1_5);
                sw(a0, (((v0_2 as i64) << 32) >> 32) as i32);
                v0_4 = ((v0_3 as i64) << 32) >> 32;
                sw(a1, (((v0_3 as i64) << 21) >> 32) as i32);
                break 'b1;
            }
        }
        v0_4 = 0_i64;
    }
    return v0_4;
}

/// Build the icon system file of a save.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn makeIconSys(mut a0: i32) {
    let mut s3_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a1_3: i64 = 0;
    let mut a2_3: i64 = 0;
    let mut a1_4: i64 = 0;
    let mut a2_4: i64 = 0;
    let mut a1_5: i64 = 0;
    let mut a2_5: i32 = 0;
    let mut v0_1: i64 = 0;
    let mut a1_6: i64 = 0;
    let mut v0_2: i64 = 0;
    let mut a1_7: i64 = 0;
    let mut a1_8: i32 = 0;
    let mut a4_2: i64 = 0;
    let mut v1_1: i64 = 0;
    let mut a4_3: i64 = 0;
    let mut v1_2: i64 = 0;
    let mut a0_3: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a3_3: i64 = 0;
    let mut a1_9: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_11: i32 = 0;
    if a0 != 0 {
        s3_1 = 0x399c30;
        s2_1 = 0x399c68;
    } else {
        s3_1 = 0x399be0;
        s2_1 = 0x399c20;
    }
    a1_1 = 0;
    s0_1 = 0x1fb29c0;
    a2_1 = 964;
    a0_1 = s0_1;
    memset(a0_1, a1_1, a2_1);
    a1_2 = s3_1;
    a4_1 = 0x3ae428;
    a3_1 = 0x1fb29c0;
    a2_2 = lb(a4_1 + 4);
    sw(a3_1, lw(a4_1));
    sb(a3_1 + 4, a2_2);
    a0_2 = s0_1 + 192;
    strcpy(a0_2, a1_2);
    sh(s0_1 + 6, 26);
    sw(s0_1 + 12, 48);
    a3_2 = g::bgcolor_75;
    a1_3 = ld(a3_2 + 8);
    a2_3 = ld(a3_2 + 16);
    sd(s0_1 + 16, ld(a3_2));
    sd(s0_1 + 24, a1_3);
    sd(s0_1 + 32, a2_3);
    a1_4 = ld(a3_2 + 32);
    a2_4 = ld(a3_2 + 40);
    sd(s0_1 + 40, ld(a3_2 + 24));
    sd(s0_1 + 48, a1_4);
    sd(s0_1 + 56, a2_4);
    a1_5 = ld(a3_2 + 56);
    sd(s0_1 + 64, ld(a3_2 + 48));
    sd(s0_1 + 72, a1_5);
    a2_5 = g::lightdir_76;
    v0_1 = ld(a2_5 + 8);
    a1_6 = ld(a2_5 + 16);
    sd(s0_1 + 80, ld(a2_5));
    sd(s0_1 + 88, v0_1);
    sd(s0_1 + 96, a1_6);
    v0_2 = ld(a2_5 + 32);
    a1_7 = ld(a2_5 + 40);
    sd(s0_1 + 104, ld(a2_5 + 24));
    sd(s0_1 + 112, v0_2);
    sd(s0_1 + 120, a1_7);
    a1_8 = g::lightcol_77;
    a4_2 = ld(a1_8 + 8);
    v1_1 = ld(a1_8 + 16);
    sd(s0_1 + 128, ld(a1_8));
    sd(s0_1 + 136, a4_2);
    sd(s0_1 + 144, v1_1);
    a4_3 = ld(a1_8 + 32);
    v1_2 = ld(a1_8 + 40);
    sd(s0_1 + 152, ld(a1_8 + 24));
    sd(s0_1 + 160, a4_3);
    sd(s0_1 + 168, v1_2);
    a0_3 = s0_1 + 260;
    v1_3 = g::ambient_78;
    a3_3 = ld(v1_3 + 8);
    sd(s0_1 + 176, ld(v1_3));
    sd(s0_1 + 184, a3_3);
    a1_9 = s2_1;
    strcpy(a0_3, a1_9);
    a0_4 = s0_1 + 324;
    a1_10 = s2_1;
    strcpy(a0_4, a1_10);
    a0_5 = s0_1 + 388;
    a1_11 = s2_1;
    strcpy(a0_5, a1_11);
    return;
}

/// Tick of the memory card state machine.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn mcardTick() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut v1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a1_9: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut a0_15: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut a2_11: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a2_12: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut a2_13: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a2_14: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut a2_15: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut a0_21: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut a2_16: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut v0_24: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut a1_20: i32 = 0;
    let mut a2_17: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut a2_18: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut a1_21: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a2_19: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut a2_20: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a1_23: i32 = 0;
    let mut a2_21: i32 = 0;
    let mut v0_28: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut v0_29: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut a1_24: i32 = 0;
    let mut a2_22: i32 = 0;
    let mut v0_30: i32 = 0;
    let mut a1_25: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut v0_31: i32 = 0;
    let mut a2_23: i32 = 0;
    let mut a1_26: i32 = 0;
    let mut v0_32: i32 = 0;
    let mut a0_30: i32 = 0;
    let mut a1_27: i32 = 0;
    let mut a2_24: i32 = 0;
    let mut v0_33: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut v0_34: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut a1_28: i32 = 0;
    let mut a2_25: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a0_31: i32 = 0;
    let mut a0_32: i32 = 0;
    let mut a2_26: i32 = 0;
    let mut a1_29: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut v0_35: i32 = 0;
    let mut a0_33: i32 = 0;
    let mut a1_30: i32 = 0;
    let mut a2_27: i32 = 0;
    let mut v0_36: i32 = 0;
    let mut a0_34: i32 = 0;
    let mut a0_35: i32 = 0;
    let mut v0_37: i32 = 0;
    let mut a0_36: i32 = 0;
    let mut a1_31: i32 = 0;
    let mut a2_28: i32 = 0;
    let mut v0_38: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut v1_15: i32 = 0;
    let mut a0_37: i32 = 0;
    let mut a1_32: i32 = 0;
    let mut a2_29: i32 = 0;
    let mut v0_39: i32 = 0;
    let mut a0_38: i32 = 0;
    let mut a1_33: i32 = 0;
    let mut a2_30: i32 = 0;
    let mut v0_40: i32 = 0;
    let mut v1_16: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut a1_34: i32 = 0;
    let mut a2_31: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut a0_39: i32 = 0;
    let mut a0_40: i32 = 0;
    let mut a1_35: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut a2_32: i32 = 0;
    let mut v0_41: i32 = 0;
    let mut a0_41: i32 = 0;
    let mut a1_36: i32 = 0;
    let mut a2_33: i32 = 0;
    let mut v0_42: i32 = 0;
    let mut a1_37: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut a1_38: i32 = 0;
    let mut a0_42: i32 = 0;
    let mut v0_43: i32 = 0;
    let mut v0_44: i32 = 0;
    let mut a1_39: i32 = 0;
    let mut a0_43: i32 = 0;
    let mut v0_45: i32 = 0;
    let mut a0_44: i32 = 0;
    let mut a1_40: i32 = 0;
    let mut v0_46: i32 = 0;
    let mut a2_34: i32 = 0;
    let mut a1_41: i32 = 0;
    let mut a0_45: i32 = 0;
    let mut v0_47: i32 = 0;
    let mut v1_17: i32 = 0;
    let mut s0_5: i32 = 0;
    let mut a1_42: i32 = 0;
    let mut a2_35: i32 = 0;
    let mut a0_46: i32 = 0;
    let mut a0_47: i32 = 0;
    let mut a1_43: i32 = 0;
    let mut a2_36: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a3_8: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut v0_48: i32 = 0;
    let mut a0_48: i32 = 0;
    let mut a1_44: i32 = 0;
    let mut a2_37: i32 = 0;
    let mut v0_49: i32 = 0;
    let mut a0_49: i32 = 0;
    let mut v0_50: i32 = 0;
    let mut a0_50: i32 = 0;
    let mut a1_45: i32 = 0;
    let mut a2_38: i32 = 0;
    let mut v0_51: i32 = 0;
    let mut a0_51: i32 = 0;
    let mut a1_46: i32 = 0;
    let mut a2_39: i32 = 0;
    let mut v0_52: i32 = 0;
    let mut a1_47: i32 = 0;
    let mut v1_18: i32 = 0;
    let mut a0_52: i32 = 0;
    let mut v0_53: i32 = 0;
    let mut a0_53: i32 = 0;
    let mut a1_48: i32 = 0;
    let mut a2_40: i32 = 0;
    let mut v0_54: i32 = 0;
    let mut a1_49: i32 = 0;
    let mut v1_19: i32 = 0;
    let mut a0_54: i32 = 0;
    let mut a1_50: i32 = 0;
    let mut a2_41: i32 = 0;
    let mut a3_9: i32 = 0;
    let mut v0_55: i32 = 0;
    let mut a0_55: i32 = 0;
    let mut a1_51: i32 = 0;
    let mut a2_42: i32 = 0;
    let mut v0_56: i32 = 0;
    let mut v1_20: i32 = 0;
    let mut v0_57: i32 = 0;
    let mut a0_56: i32 = 0;
    let mut a1_52: i32 = 0;
    let mut a2_43: i32 = 0;
    let mut v0_58: i32 = 0;
    let mut a0_57: i32 = 0;
    let mut a1_53: i32 = 0;
    let mut a2_44: i32 = 0;
    let mut v0_59: i32 = 0;
    let mut a0_58: i32 = 0;
    let mut a1_54: i32 = 0;
    let mut v0_60: i32 = 0;
    let mut a1_55: i32 = 0;
    let mut a0_59: i32 = 0;
    let mut a2_45: i32 = 0;
    let mut v1_21: i32 = 0;
    let mut a0_60: i32 = 0;
    let mut a1_56: i32 = 0;
    let mut a2_46: i32 = 0;
    let mut v0_61: i32 = 0;
    let mut a3_10: i32 = 0;
    let mut v0_62: i32 = 0;
    let mut a1_57: i32 = 0;
    let mut v1_22: i32 = 0;
    let mut v0_63: i32 = 0;
    let mut v1_23: i32 = 0;
    let mut v0_64: i32 = 0;
    let mut a1_58: i32 = 0;
    let mut v0_65: i32 = 0;
    let mut a0_61: i32 = 0;
    let mut a0_62: i32 = 0;
    let mut a2_47: i32 = 0;
    let mut a1_59: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a3_11: i32 = 0;
    let mut v0_66: i32 = 0;
    let mut a0_63: i32 = 0;
    let mut a1_60: i32 = 0;
    let mut a2_48: i32 = 0;
    let mut v0_67: i32 = 0;
    let mut a3_12: i32 = 0;
    let mut v0_68: i32 = 0;
    let mut a1_61: i32 = 0;
    let mut a2_49: i32 = 0;
    let mut v1_24: i32 = 0;
    'b1: {
        v1_1 = lw(g::mcardstate);
        if (v1_1 as u32) < (50 as u32) {
            'b2: {
                'b3: {
                    'b4: {
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
                                                                    break 'b1;
                                                                }
                                                                1 => {
                                                                    a3_12 = lw(0x3afe38);
                                                                    sw(g::checkcards_port, 0);
                                                                    v0_68 = lw(a3_12);
                                                                    sw(g::checkcards_slot, 0);
                                                                    if v0_68 == 1 {
                                                                        break 'b2;
                                                                    }
                                                                    a1_61 = 0;
                                                                    a2_49 = 1;
                                                                    'l15: loop {
                                                                        sw(g::checkcards_slot, a2_49);
                                                                        if a2_49 == 4 {
                                                                            a1_61 = a1_61 + 1;
                                                                            a2_49 = 0;
                                                                            sw(g::checkcards_slot, 0);
                                                                            sw(g::checkcards_port, a1_61);
                                                                            if a1_61 == 2 {
                                                                                break;
                                                                            }
                                                                        }
                                                                        v1_24 = lw((((a1_61 << 2) + a2_49) << 2) + a3_12);
                                                                        a2_49 = a2_49 + 1;
                                                                        if v1_24 == 1 {
                                                                            break 'b2;
                                                                        }
                                                                    }
                                                                    v0_2 = 1;
                                                                    break 'b4;
                                                                }
                                                                2 => {
                                                                    a0_63 = 1;
                                                                    a1_60 = 0;
                                                                    a2_48 = sp;
                                                                    v0_67 = sceMcSync(&[a0_63 as i64, a1_60 as i64, a2_48 as i64]);
                                                                    if v0_67 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    break 'b2;
                                                                }
                                                                3 => {
                                                                    a0_62 = lw(g::checkcards_port);
                                                                    a2_47 = g::checkcards_type;
                                                                    a1_59 = lw(g::checkcards_slot);
                                                                    a4_3 = g::checkcards_format;
                                                                    a3_11 = if lw(0x3afe3c) == 0 { 0 } else { g::checkcards_free };
                                                                    v0_66 = sceMcGetInfo(&[a0_62 as i64, a1_59 as i64, a2_47 as i64, a3_11 as i64, a4_3 as i64]);
                                                                    if v0_66 != 0 {
                                                                        if v0_66 <= 0 {
                                                                            break 'b5;
                                                                        }
                                                                        sw(g::mcardstate, 2);
                                                                        break 'b1;
                                                                    }
                                                                    sw(g::mcardstate, 4);
                                                                    break 'b1;
                                                                }
                                                                4 => {
                                                                    a0_60 = 1;
                                                                    a1_56 = 0;
                                                                    a2_46 = sp;
                                                                    v0_61 = sceMcSync(&[a0_60 as i64, a1_56 as i64, a2_46 as i64]);
                                                                    if v0_61 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    'b18: {
                                                                        a3_10 = lw(0x3afe40);
                                                                        v0_62 = lw(g::checkcards_port);
                                                                        if a3_10 != 0 {
                                                                            a1_57 = lw(sp);
                                                                            v1_22 = a3_10 + (((v0_62 << 2) + lw(g::checkcards_slot)) << 2);
                                                                            if a1_57 == -1 {
                                                                                sw(v1_22, v0_61);
                                                                            } else {
                                                                                if a1_57 >= 0 {
                                                                                    v0_63 = 3;
                                                                                    if a1_57 == 0 {
                                                                                        sw(v1_22, 0);
                                                                                        break 'b18;
                                                                                    }
                                                                                } else {
                                                                                    if a1_57 != -2 {
                                                                                        sw(v1_22, 3);
                                                                                        break 'b18;
                                                                                    }
                                                                                    v0_63 = 2;
                                                                                }
                                                                                sw(v1_22, v0_63);
                                                                            }
                                                                        }
                                                                    }
                                                                    'b20: {
                                                                        'b21: {
                                                                            v1_23 = lw(sp);
                                                                            if v1_23 == -10 {
                                                                                v0_64 = lw(g::checkcards_port);
                                                                            } else {
                                                                                v0_64 = lw(g::checkcards_port);
                                                                                if v1_23 >= -10 {
                                                                                    a1_58 = 1;
                                                                                    if v1_23 > 0 {
                                                                                        break 'b21;
                                                                                    }
                                                                                    v0_64 = lw(g::checkcards_port);
                                                                                    if v1_23 < -2 {
                                                                                        break 'b21;
                                                                                    }
                                                                                    v0_64 = lw(g::checkcards_port);
                                                                                    if lw(g::checkcards_type) != 2 {
                                                                                        a1_58 = 3;
                                                                                        break 'b21;
                                                                                    }
                                                                                    v0_64 = lw(g::checkcards_port);
                                                                                    if lw(g::checkcards_format) == 0 {
                                                                                        a1_58 = 5;
                                                                                        break 'b21;
                                                                                    }
                                                                                    v0_65 = ((v0_64 << 2) + lw(g::checkcards_slot)) << 2;
                                                                                    a0_61 = lw(0x3afe3c);
                                                                                    sw(v0_65 + lw(0x3afe38), 4);
                                                                                    if a0_61 == 0 {
                                                                                        break 'b20;
                                                                                    }
                                                                                    sw(v0_65 + a0_61, lw(g::checkcards_free));
                                                                                    break 'b20;
                                                                                }
                                                                            }
                                                                            a1_58 = 1;
                                                                        }
                                                                        sw((((v0_64 << 2) + lw(g::checkcards_slot)) << 2) + lw(0x3afe38), a1_58);
                                                                    }
                                                                    sw(g::mcardstate, 5);
                                                                    break 'b1;
                                                                }
                                                                5 => {
                                                                    a1_55 = lw(g::checkcards_port);
                                                                    a0_59 = lw(0x3afe38);
                                                                    a2_45 = lw(g::checkcards_slot) + 1;
                                                                    'l23: loop {
                                                                        sw(g::checkcards_slot, a2_45);
                                                                        if a2_45 == 4 {
                                                                            a1_55 = a1_55 + 1;
                                                                            a2_45 = 0;
                                                                            sw(g::checkcards_slot, 0);
                                                                            sw(g::checkcards_port, a1_55);
                                                                            if a1_55 == 2 {
                                                                                break;
                                                                            }
                                                                        }
                                                                        v1_21 = lw((((a1_55 << 2) + a2_45) << 2) + a0_59);
                                                                        a2_45 = a2_45 + 1;
                                                                        if v1_21 == 1 {
                                                                            break 'b2;
                                                                        }
                                                                    }
                                                                    v0_2 = 1;
                                                                    break 'b4;
                                                                }
                                                                6 => {
                                                                    a0_58 = lw(g::port);
                                                                    a1_54 = lw(g::slot);
                                                                    v0_60 = sceMcFormat(&[a0_58 as i64, a1_54 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_60 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 7);
                                                                    break 'b1;
                                                                }
                                                                7 => {
                                                                    a0_57 = 1;
                                                                    a1_53 = 0;
                                                                    a2_44 = sp;
                                                                    v0_59 = sceMcSync(&[a0_57 as i64, a1_53 as i64, a2_44 as i64]);
                                                                    a2_18 = v0_59;
                                                                    if a2_18 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v0_2 = 3;
                                                                    if lw(sp) == 0 {
                                                                        break 'b6;
                                                                    }
                                                                    sw(g::mcardstate, 0);
                                                                    break 'b3;
                                                                }
                                                                8 => {
                                                                    a0_56 = lw(g::port);
                                                                    a1_52 = lw(g::slot);
                                                                    a2_43 = lw(0x3afe44);
                                                                    v0_58 = sceMcGetEntSpace(&[a0_56 as i64, a1_52 as i64, a2_43 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_58 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 9);
                                                                    break 'b1;
                                                                }
                                                                9 => {
                                                                    a0_55 = 1;
                                                                    a1_51 = 0;
                                                                    a2_42 = sp;
                                                                    v0_56 = sceMcSync(&[a0_55 as i64, a1_51 as i64, a2_42 as i64]);
                                                                    a2_18 = v0_56;
                                                                    if a2_18 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_20 = lw(sp);
                                                                    v0_57 = lw(0x3afe48);
                                                                    if v1_20 < 0 {
                                                                        break 'b5;
                                                                    }
                                                                    sw(v0_57, v1_20);
                                                                    break 'b6;
                                                                }
                                                                10 => {
                                                                    a0_54 = lw(g::port);
                                                                    a1_50 = lw(g::slot);
                                                                    a2_41 = 0x1fb3140;
                                                                    a3_9 = 1;
                                                                    v0_55 = sceMcOpen(&[a0_54 as i64, a1_50 as i64, a2_41 as i64, a3_9 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_55 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 11);
                                                                    break 'b1;
                                                                }
                                                                11 => {
                                                                    a0_53 = 1;
                                                                    a1_48 = 0;
                                                                    a2_40 = sp;
                                                                    v0_54 = sceMcSync(&[a0_53 as i64, a1_48 as i64, a2_40 as i64]);
                                                                    if v0_54 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a1_49 = lw(sp);
                                                                    if a1_49 < 0 {
                                                                        v1_19 = a1_49 + 10;
                                                                        if (v1_19 as u32) < (9 as u32) {
                                                                            match (v1_19 << 2) >> 2 {
                                                                                0 | 1 | 2 | 3 | 4 | 5 | 7 => {
                                                                                }
                                                                                6 => {
                                                                                    v0_2 = 5;
                                                                                    break 'b4;
                                                                                }
                                                                                8 => {
                                                                                    v0_2 = 6;
                                                                                    break 'b4;
                                                                                }
                                                                                _ => unreachable!(),
                                                                            }
                                                                        }
                                                                        sw(g::mcardstate, 0);
                                                                        if a1_49 >= -9 {
                                                                            v0_2 = 3;
                                                                            break 'b3;
                                                                        }
                                                                        v0_2 = 4;
                                                                        break 'b3;
                                                                    }
                                                                    sw(g::mkdir_fd, a1_49);
                                                                    sw(g::mcardstate, 12);
                                                                    break 'b1;
                                                                }
                                                                12 => {
                                                                    a0_52 = lw(g::mkdir_fd);
                                                                    v0_53 = sceMcClose(&[a0_52 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_53 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 13);
                                                                    break 'b1;
                                                                }
                                                                13 => {
                                                                    a0_51 = 1;
                                                                    a1_46 = 0;
                                                                    a2_39 = sp;
                                                                    v0_52 = sceMcSync(&[a0_51 as i64, a1_46 as i64, a2_39 as i64]);
                                                                    if v0_52 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a1_47 = lw(sp);
                                                                    v1_18 = a1_47 + 10;
                                                                    if (v1_18 as u32) < (11 as u32) {
                                                                        match (v1_18 << 2) >> 2 {
                                                                            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => {
                                                                            }
                                                                            8 | 9 => {
                                                                                v0_2 = 6;
                                                                                break 'b4;
                                                                            }
                                                                            10 => {
                                                                                v0_2 = 1;
                                                                                break 'b4;
                                                                            }
                                                                            _ => unreachable!(),
                                                                        }
                                                                    }
                                                                    sw(g::mcardstate, 0);
                                                                    if a1_47 >= -9 {
                                                                        v0_2 = 3;
                                                                        break 'b3;
                                                                    }
                                                                    v0_2 = 4;
                                                                    break 'b3;
                                                                }
                                                                14 => {
                                                                    a0_50 = lw(g::port);
                                                                    a1_45 = lw(g::slot);
                                                                    a2_38 = 0x1fb2940;
                                                                    v0_51 = sceMcMkdir(&[a0_50 as i64, a1_45 as i64, a2_38 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_51 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 15);
                                                                    break 'b1;
                                                                }
                                                                15 => {
                                                                    a0_48 = 1;
                                                                    a1_44 = 0;
                                                                    a2_37 = sp;
                                                                    v0_49 = sceMcSync(&[a0_48 as i64, a1_44 as i64, a2_37 as i64]);
                                                                    if v0_49 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a0_49 = lw(sp) + 10;
                                                                    if (a0_49 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (a0_49 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 9 => {
                                                                            break 'b5;
                                                                        }
                                                                        6 => {
                                                                            if lw(g::mkdir_overwriteok) == 0 {
                                                                                break 'b5;
                                                                            }
                                                                            sw(g::mcardstate, 16);
                                                                            break 'b1;
                                                                        }
                                                                        7 => {
                                                                            v0_2 = 7;
                                                                            break 'b4;
                                                                        }
                                                                        8 => {
                                                                            v0_2 = 6;
                                                                            break 'b4;
                                                                        }
                                                                        10 => {
                                                                            v0_50 = lw(0x3afe68);
                                                                            sw(g::mcardstate, 18);
                                                                            sw(0x3afe68, v0_50 | 256);
                                                                            break 'b1;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                16 => {
                                                                    s0_5 = g::mkdir_tmpbuf;
                                                                    a1_42 = 0x3ae430;
                                                                    a2_35 = 0x1fb2940;
                                                                    a0_46 = s0_5;
                                                                    sprintf(&[a0_46 as i64, a1_42 as i64, a2_35 as i64]);
                                                                    a0_47 = lw(g::port);
                                                                    a1_43 = lw(g::slot);
                                                                    a2_36 = s0_5;
                                                                    a5_1 = 0x1fb2e00;
                                                                    a3_8 = 0;
                                                                    a4_2 = 7;
                                                                    v0_48 = sceMcGetDir(&[a0_47 as i64, a1_43 as i64, a2_36 as i64, a3_8 as i64, a4_2 as i64, a5_1 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_48 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 17);
                                                                    break 'b1;
                                                                }
                                                                17 => {
                                                                    a0_41 = 1;
                                                                    a1_36 = 0;
                                                                    a2_33 = sp;
                                                                    v0_42 = sceMcSync(&[a0_41 as i64, a1_36 as i64, a2_33 as i64]);
                                                                    if v0_42 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a1_37 = lw(sp);
                                                                    if a1_37 >= 0 {
                                                                        'b28: {
                                                                            s1_1 = a1_37;
                                                                            if a1_37 > 0 {
                                                                                'l29: loop {
                                                                                    s1_1 = s1_1 - 1;
                                                                                    if s1_1 == 0 {
                                                                                        break 'b28;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                        'b30: {
                                                                            s1_2 = 0;
                                                                            if a1_37 > 0 {
                                                                                s2_1 = s1_2 << 6;
                                                                                'l31: loop {
                                                                                    'b32: {
                                                                                        'b33: {
                                                                                            s3_1 = 0x1fb2e20;
                                                                                            s0_4 = s2_1 + s3_1;
                                                                                            a1_38 = 0x399ca8;
                                                                                            a0_42 = s0_4;
                                                                                            v0_43 = strcmp(a0_42, a1_38);
                                                                                            v0_44 = lw(0x3afe4c);
                                                                                            if v0_43 != 0 {
                                                                                                a0_44 = s0_4;
                                                                                                a1_40 = lw((v0_44 << 2) + 0x3afe50);
                                                                                                v0_46 = strcmp(a0_44, a1_40);
                                                                                                if v0_46 != 0 {
                                                                                                    s1_2 = s1_2 + 1;
                                                                                                    break 'b32;
                                                                                                }
                                                                                                a2_34 = lw(0x3afe68);
                                                                                                a1_41 = lw(s3_1 + s2_1 - 16);
                                                                                                a0_45 = lw((lw(0x3afe4c) << 2) + 0x3afe60);
                                                                                                sw(0x3afe68, a2_34 | 2);
                                                                                                if a1_41 == a0_45 {
                                                                                                    break 'b33;
                                                                                                }
                                                                                                v0_45 = a2_34 | 34;
                                                                                            } else {
                                                                                                a1_39 = lw(0x3afe68);
                                                                                                a0_43 = lw(s3_1 + s2_1 - 16);
                                                                                                sw(0x3afe68, a1_39 | 1);
                                                                                                if a0_43 == 964 {
                                                                                                    break 'b33;
                                                                                                }
                                                                                                v0_45 = a1_39 | 17;
                                                                                            }
                                                                                            sw(0x3afe68, v0_45);
                                                                                        }
                                                                                        s1_2 = s1_2 + 1;
                                                                                    }
                                                                                    v0_47 = lw(sp);
                                                                                    if s1_2 >= 7 {
                                                                                        break 'b30;
                                                                                    }
                                                                                    s2_1 = s1_2 << 6;
                                                                                    if s1_2 >= v0_47 {
                                                                                        break 'b30;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                        sw(g::mcardstate, 18);
                                                                        break 'b1;
                                                                    }
                                                                    v1_17 = a1_37 + 10;
                                                                    if (v1_17 as u32) >= (9 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (v1_17 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => {
                                                                            break 'b5;
                                                                        }
                                                                        8 => {
                                                                            v0_2 = 6;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                18 => {
                                                                    if (lw(0x3afe68) & 17) == 1 {
                                                                        break 'b7;
                                                                    }
                                                                    s0_3 = g::mkdir_tmpbuf;
                                                                    a1_34 = 0x3ae438;
                                                                    a2_31 = 0x1fb2940;
                                                                    a3_6 = 0x399ca8;
                                                                    a0_39 = s0_3;
                                                                    sprintf(&[a0_39 as i64, a1_34 as i64, a2_31 as i64, a3_6 as i64]);
                                                                    a0_40 = lw(g::port);
                                                                    a1_35 = lw(g::slot);
                                                                    a3_7 = if (lw(0x3afe68) & 1) == 0 { 514 } else { 2 };
                                                                    a2_32 = s0_3;
                                                                    v0_41 = sceMcOpen(&[a0_40 as i64, a1_35 as i64, a2_32 as i64, a3_7 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_41 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 19);
                                                                    break 'b1;
                                                                }
                                                                19 => {
                                                                    a0_38 = 1;
                                                                    a1_33 = 0;
                                                                    a2_30 = sp;
                                                                    v0_40 = sceMcSync(&[a0_38 as i64, a1_33 as i64, a2_30 as i64]);
                                                                    if v0_40 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_12 = lw(sp);
                                                                    v0_34 = 20;
                                                                    if v1_12 >= 0 {
                                                                        break 'b8;
                                                                    }
                                                                    v1_16 = v1_12 + 10;
                                                                    if (v1_16 as u32) >= (9 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (v1_16 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                                                            break 'b5;
                                                                        }
                                                                        7 => {
                                                                            v0_2 = 7;
                                                                            break 'b4;
                                                                        }
                                                                        8 => {
                                                                            v0_2 = 6;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                20 => {
                                                                    a0_37 = lw(g::mkdir_fd);
                                                                    a1_32 = 0x1fb29c0;
                                                                    a2_29 = 964;
                                                                    v0_39 = sceMcWrite(&[a0_37 as i64, a1_32 as i64, a2_29 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_39 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 21);
                                                                    break 'b1;
                                                                }
                                                                21 => {
                                                                    a0_36 = 1;
                                                                    a1_31 = 0;
                                                                    a2_28 = sp;
                                                                    v0_38 = sceMcSync(&[a0_36 as i64, a1_31 as i64, a2_28 as i64]);
                                                                    if v0_38 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_14 = lw(sp);
                                                                    v1_15 = v1_14 + 10;
                                                                    if v1_14 == 964 {
                                                                        sw(g::mcardstate, 22);
                                                                        break 'b1;
                                                                    }
                                                                    if (v1_15 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (v1_15 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 8 | 9 | 10 => {
                                                                            break 'b5;
                                                                        }
                                                                        7 => {
                                                                            v0_2 = 7;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                22 => {
                                                                    a0_35 = lw(g::mkdir_fd);
                                                                    v0_37 = sceMcClose(&[a0_35 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_37 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 23);
                                                                    break 'b1;
                                                                }
                                                                23 => {
                                                                    a0_33 = 1;
                                                                    a1_30 = 0;
                                                                    a2_27 = sp;
                                                                    v0_36 = sceMcSync(&[a0_33 as i64, a1_30 as i64, a2_27 as i64]);
                                                                    if v0_36 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a0_34 = lw(sp) + 10;
                                                                    if (a0_34 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (a0_34 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 => {
                                                                            break 'b5;
                                                                        }
                                                                        10 => {
                                                                            break 'b7;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                24 => {
                                                                    if (lw(0x3afe68) & 34) == 2 {
                                                                        break 'b9;
                                                                    }
                                                                    s0_2 = g::mkdir_tmpbuf;
                                                                    a1_28 = 0x3ae438;
                                                                    a2_25 = 0x1fb2940;
                                                                    a3_4 = 0x1fb2ac4;
                                                                    a0_31 = s0_2;
                                                                    sprintf(&[a0_31 as i64, a1_28 as i64, a2_25 as i64, a3_4 as i64]);
                                                                    a0_32 = lw(g::port);
                                                                    a2_26 = s0_2;
                                                                    a1_29 = lw(g::slot);
                                                                    a3_5 = if (lw(0x3afe68) & 2) != 0 { 2 } else { 514 };
                                                                    v0_35 = sceMcOpen(&[a0_32 as i64, a1_29 as i64, a2_26 as i64, a3_5 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_35 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 25);
                                                                    break 'b1;
                                                                }
                                                                25 => {
                                                                    a0_30 = 1;
                                                                    a1_27 = sp | 4;
                                                                    a2_24 = sp;
                                                                    v0_33 = sceMcSync(&[a0_30 as i64, a1_27 as i64, a2_24 as i64]);
                                                                    if v0_33 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_12 = lw(sp);
                                                                    v0_34 = 26;
                                                                    if v1_12 >= 0 {
                                                                        break 'b8;
                                                                    }
                                                                    v1_13 = v1_12 + 10;
                                                                    if (v1_13 as u32) >= (9 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (v1_13 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 8 => {
                                                                            break 'b5;
                                                                        }
                                                                        7 => {
                                                                            v0_2 = 7;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                26 => {
                                                                    a0_29 = lw(g::mkdir_fd);
                                                                    v0_31 = lw(0x3afe4c) << 2;
                                                                    a2_23 = lw(v0_31 + 0x3afe60);
                                                                    a1_26 = lw(v0_31 + 0x3afe58);
                                                                    v0_32 = sceMcWrite(&[a0_29 as i64, a1_26 as i64, a2_23 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_32 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 27);
                                                                    break 'b1;
                                                                }
                                                                27 => {
                                                                    a0_28 = 1;
                                                                    a1_24 = 0;
                                                                    a2_22 = sp;
                                                                    v0_30 = sceMcSync(&[a0_28 as i64, a1_24 as i64, a2_22 as i64]);
                                                                    if v0_30 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a1_25 = lw(sp);
                                                                    v1_11 = a1_25 + 10;
                                                                    if a1_25 == lw((lw(0x3afe4c) << 2) + 0x3afe60) {
                                                                        sw(g::mcardstate, 28);
                                                                        break 'b1;
                                                                    }
                                                                    if (v1_11 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (v1_11 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 8 | 9 | 10 => {
                                                                            break 'b5;
                                                                        }
                                                                        7 => {
                                                                            v0_2 = 7;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                28 => {
                                                                    a0_27 = lw(g::mkdir_fd);
                                                                    v0_29 = sceMcClose(&[a0_27 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_29 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 29);
                                                                    break 'b1;
                                                                }
                                                                29 => {
                                                                    a0_25 = 1;
                                                                    a1_23 = 0;
                                                                    a2_21 = sp;
                                                                    v0_28 = sceMcSync(&[a0_25 as i64, a1_23 as i64, a2_21 as i64]);
                                                                    if v0_28 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a0_26 = lw(sp) + 10;
                                                                    if (a0_26 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (a0_26 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 => {
                                                                            break 'b5;
                                                                        }
                                                                        10 => {
                                                                            break 'b9;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                30 => {
                                                                    v0_26 = lw(g::mkdir_copyprotect);
                                                                    if (lw(0x3afe68) & 256) != 0 {
                                                                        if v0_26 != 0 {
                                                                            a1_21 = 0;
                                                                            s0_1 = 0x1fb2dc0;
                                                                            a2_19 = 64;
                                                                            a0_23 = s0_1;
                                                                            memset(a0_23, a1_21, a2_19);
                                                                            a0_24 = lw(g::port);
                                                                            a2_20 = 0x1fb2940;
                                                                            a1_22 = lw(g::slot);
                                                                            a3_3 = s0_1;
                                                                            sh(s0_1 + 20, 47);
                                                                            a4_1 = 4;
                                                                            v0_27 = sceMcSetFileInfo(&[a0_24 as i64, a1_22 as i64, a2_20 as i64, a3_3 as i64, a4_1 as i64]);
                                                                            v0_2 = 3;
                                                                            if v0_27 != 0 {
                                                                                break 'b4;
                                                                            }
                                                                            sw(g::mcardstate, 31);
                                                                            break 'b1;
                                                                        }
                                                                    }
                                                                    v0_2 = 1;
                                                                    break 'b4;
                                                                }
                                                                31 => {
                                                                    a0_22 = 1;
                                                                    a1_20 = 0;
                                                                    a2_17 = sp;
                                                                    v0_25 = sceMcSync(&[a0_22 as i64, a1_20 as i64, a2_17 as i64]);
                                                                    a2_18 = v0_25;
                                                                    if a2_18 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_10 = lw(sp);
                                                                    if v1_10 == -4 {
                                                                        break 'b5;
                                                                    }
                                                                    v0_2 = 3;
                                                                    if v1_10 < -3 {
                                                                        break 'b4;
                                                                    }
                                                                    if v1_10 == -2 {
                                                                        v0_2 = 6;
                                                                        break 'b4;
                                                                    }
                                                                    v0_2 = 3;
                                                                    if v1_10 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    break 'b6;
                                                                }
                                                                32 => {
                                                                    a0_21 = lw(g::port);
                                                                    a1_19 = lw(g::slot);
                                                                    a2_16 = 0x1fb2fc0;
                                                                    a3_2 = 1;
                                                                    v0_24 = sceMcOpen(&[a0_21 as i64, a1_19 as i64, a2_16 as i64, a3_2 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_24 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 33);
                                                                    break 'b1;
                                                                }
                                                                33 => {
                                                                    a0_20 = 1;
                                                                    a1_17 = 0;
                                                                    a2_15 = sp;
                                                                    v0_22 = sceMcSync(&[a0_20 as i64, a1_17 as i64, a2_15 as i64]);
                                                                    if v0_22 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a1_18 = lw(sp);
                                                                    if a1_18 < 0 {
                                                                        v1_9 = a1_18 + 10;
                                                                        if (v1_9 as u32) < (9 as u32) {
                                                                            match (v1_9 << 2) >> 2 {
                                                                                0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => {
                                                                                }
                                                                                8 => {
                                                                                    v0_2 = 6;
                                                                                    break 'b4;
                                                                                }
                                                                                _ => unreachable!(),
                                                                            }
                                                                        }
                                                                        if a1_18 != -4 {
                                                                            if a1_18 >= -9 {
                                                                                break 'b12;
                                                                            }
                                                                            sw(g::mcardresult, 4);
                                                                            break 'b10;
                                                                        }
                                                                        v0_15 = 5;
                                                                        break 'b11;
                                                                    }
                                                                    v0_23 = lw(0x3afe70);
                                                                    sw(g::read_fd, a1_18);
                                                                    if v0_23 <= 0 {
                                                                        break 'b13;
                                                                    }
                                                                    sw(g::mcardstate, 34);
                                                                    break 'b1;
                                                                }
                                                                34 => {
                                                                    a0_19 = lw(g::read_fd);
                                                                    a2_14 = 0;
                                                                    a1_16 = lw(0x3afe70);
                                                                    v0_21 = sceMcSeek(&[a0_19 as i64, a1_16 as i64, a2_14 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_21 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 35);
                                                                    break 'b1;
                                                                }
                                                                35 => {
                                                                    a0_18 = 1;
                                                                    a1_15 = 0;
                                                                    a2_13 = sp;
                                                                    v0_20 = sceMcSync(&[a0_18 as i64, a1_15 as i64, a2_13 as i64]);
                                                                    if v0_20 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_8 = lw(sp);
                                                                    if v1_8 != lw(0x3afe70) {
                                                                        if v1_8 == -4 {
                                                                            break 'b5;
                                                                        }
                                                                        v0_2 = 3;
                                                                        if v1_8 < -3 {
                                                                            break 'b4;
                                                                        }
                                                                        v0_2 = 3;
                                                                        if v1_8 != -2 {
                                                                            break 'b4;
                                                                        }
                                                                        v0_2 = 6;
                                                                        break 'b4;
                                                                    }
                                                                    break 'b13;
                                                                }
                                                                36 => {
                                                                    a0_17 = lw(g::read_fd);
                                                                    a1_14 = lw(g::read_buffer);
                                                                    a2_12 = lw(g::request_read_len);
                                                                    v0_19 = sceMcRead(&[a0_17 as i64, a1_14 as i64, a2_12 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_19 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 37);
                                                                    break 'b1;
                                                                }
                                                                37 => {
                                                                    a0_16 = 1;
                                                                    a1_12 = 0;
                                                                    a2_11 = sp;
                                                                    v0_17 = sceMcSync(&[a0_16 as i64, a1_12 as i64, a2_11 as i64]);
                                                                    if v0_17 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a1_13 = lw(sp);
                                                                    if a1_13 < 0 {
                                                                        if a1_13 == -4 {
                                                                            v0_18 = (a1_13 < -9) as i32;
                                                                        } else {
                                                                            v0_18 = (a1_13 < -9) as i32;
                                                                            if a1_13 >= -3 {
                                                                                if a1_13 < 0 {
                                                                                    v0_18 = (a1_13 < -9) as i32;
                                                                                    if a1_13 >= -2 {
                                                                                        v0_2 = 6;
                                                                                        break 'b4;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                        if v0_18 == 0 {
                                                                            break 'b12;
                                                                        }
                                                                        sw(g::mcardresult, 4);
                                                                        break 'b10;
                                                                    }
                                                                    sw(0x3afe6c, a1_13);
                                                                    sw(g::mcardstate, 38);
                                                                    break 'b1;
                                                                }
                                                                38 => {
                                                                    a0_15 = lw(g::read_fd);
                                                                    v0_16 = sceMcClose(&[a0_15 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_16 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 39);
                                                                    break 'b1;
                                                                }
                                                                39 => {
                                                                    a0_13 = 1;
                                                                    a1_10 = 0;
                                                                    a2_10 = sp;
                                                                    v0_13 = sceMcSync(&[a0_13 as i64, a1_10 as i64, a2_10 as i64]);
                                                                    if v0_13 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a1_11 = lw(sp);
                                                                    v1_7 = a1_11 + 10;
                                                                    if (v1_7 as u32) < (11 as u32) {
                                                                        match (v1_7 << 2) >> 2 {
                                                                            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => {
                                                                            }
                                                                            8 | 9 => {
                                                                                v0_2 = 6;
                                                                                break 'b4;
                                                                            }
                                                                            10 => {
                                                                                v0_14 = lw(0x3afe6c);
                                                                                a0_14 = lw(g::request_read_len);
                                                                                sw(g::mcardstate, 0);
                                                                                sw(g::mcardresult, if (v0_14 ^ a0_14) == 0 { 1 } else { 2 });
                                                                                break 'b1;
                                                                            }
                                                                            _ => unreachable!(),
                                                                        }
                                                                    }
                                                                    v0_15 = 4;
                                                                    if a1_11 < -9 {
                                                                        break 'b11;
                                                                    }
                                                                    break 'b12;
                                                                }
                                                                40 => {
                                                                    a0_12 = lw(g::port);
                                                                    a3_1 = if lw(g::write_overwriteok) == 0 { 514 } else { 2 };
                                                                    a1_9 = lw(g::slot);
                                                                    a2_9 = 0x1fb3040;
                                                                    v0_12 = sceMcOpen(&[a0_12 as i64, a1_9 as i64, a2_9 as i64, a3_1 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_12 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 41);
                                                                    break 'b1;
                                                                }
                                                                41 => {
                                                                    a0_11 = 1;
                                                                    a1_8 = 0;
                                                                    a2_8 = sp;
                                                                    v0_10 = sceMcSync(&[a0_11 as i64, a1_8 as i64, a2_8 as i64]);
                                                                    if v0_10 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_5 = lw(sp);
                                                                    v0_11 = lw(0x3afe74);
                                                                    if v1_5 < 0 {
                                                                        v1_6 = v1_5 + 10;
                                                                        if (v1_6 as u32) >= (9 as u32) {
                                                                            break 'b5;
                                                                        }
                                                                        match (v1_6 << 2) >> 2 {
                                                                            0 => {
                                                                                v0_2 = 4;
                                                                                break 'b4;
                                                                            }
                                                                            1 | 2 | 3 | 4 | 5 | 6 => {
                                                                                break 'b5;
                                                                            }
                                                                            7 => {
                                                                                v0_2 = 7;
                                                                                break 'b4;
                                                                            }
                                                                            8 => {
                                                                                v0_2 = 6;
                                                                                break 'b4;
                                                                            }
                                                                            _ => unreachable!(),
                                                                        }
                                                                    } else {
                                                                        sw(g::write_fd, v1_5);
                                                                        if v0_11 > 0 {
                                                                            sw(g::mcardstate, 42);
                                                                            break 'b1;
                                                                        }
                                                                    }
                                                                }
                                                                42 => {
                                                                    a0_10 = lw(g::write_fd);
                                                                    a2_7 = 0;
                                                                    a1_7 = lw(0x3afe74);
                                                                    v0_9 = sceMcSeek(&[a0_10 as i64, a1_7 as i64, a2_7 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_9 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 43);
                                                                    break 'b1;
                                                                }
                                                                43 => {
                                                                    a0_9 = 1;
                                                                    a1_6 = 0;
                                                                    a2_6 = sp;
                                                                    v0_8 = sceMcSync(&[a0_9 as i64, a1_6 as i64, a2_6 as i64]);
                                                                    if v0_8 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_4 = lw(sp);
                                                                    if v1_4 != lw(0x3afe74) {
                                                                        if v1_4 == -4 {
                                                                            break 'b5;
                                                                        }
                                                                        v0_2 = 3;
                                                                        if v1_4 < -3 {
                                                                            break 'b4;
                                                                        }
                                                                        v0_2 = 3;
                                                                        if v1_4 != -2 {
                                                                            break 'b4;
                                                                        }
                                                                        v0_2 = 6;
                                                                        break 'b4;
                                                                    }
                                                                }
                                                                44 => {
                                                                    a0_8 = lw(g::write_fd);
                                                                    a1_5 = lw(g::write_buffer);
                                                                    a2_5 = lw(g::write_len);
                                                                    v0_7 = sceMcWrite(&[a0_8 as i64, a1_5 as i64, a2_5 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_7 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 45);
                                                                    break 'b1;
                                                                }
                                                                45 => {
                                                                    a0_7 = 1;
                                                                    a1_4 = 0;
                                                                    a2_4 = sp;
                                                                    v0_6 = sceMcSync(&[a0_7 as i64, a1_4 as i64, a2_4 as i64]);
                                                                    if v0_6 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    v1_2 = lw(sp);
                                                                    v1_3 = v1_2 + 10;
                                                                    if v1_2 == lw(g::write_len) {
                                                                        sw(g::mcardstate, 46);
                                                                        break 'b1;
                                                                    }
                                                                    if (v1_3 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (v1_3 << 2) >> 2 {
                                                                        0 => {
                                                                            v0_2 = 4;
                                                                            break 'b4;
                                                                        }
                                                                        1 | 2 | 3 | 4 | 5 | 6 | 9 | 10 => {
                                                                            break 'b5;
                                                                        }
                                                                        7 => {
                                                                            v0_2 = 7;
                                                                            break 'b4;
                                                                        }
                                                                        8 => {
                                                                            v0_2 = 6;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                46 => {
                                                                    a0_6 = lw(g::write_fd);
                                                                    v0_5 = sceMcClose(&[a0_6 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_5 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 47);
                                                                    break 'b1;
                                                                }
                                                                47 => {
                                                                    a0_4 = 1;
                                                                    a1_3 = 0;
                                                                    a2_3 = sp;
                                                                    v0_4 = sceMcSync(&[a0_4 as i64, a1_3 as i64, a2_3 as i64]);
                                                                    if v0_4 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a0_5 = lw(sp) + 10;
                                                                    if (a0_5 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (a0_5 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => {
                                                                            break 'b5;
                                                                        }
                                                                        8 | 9 => {
                                                                            v0_2 = 6;
                                                                            break 'b4;
                                                                        }
                                                                        10 => {
                                                                            v0_2 = 1;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                48 => {
                                                                    a0_3 = lw(g::port);
                                                                    a1_2 = lw(g::slot);
                                                                    a2_2 = 0x1fb30c0;
                                                                    v0_3 = sceMcDelete(&[a0_3 as i64, a1_2 as i64, a2_2 as i64]);
                                                                    v0_2 = 3;
                                                                    if v0_3 != 0 {
                                                                        break 'b4;
                                                                    }
                                                                    sw(g::mcardstate, 49);
                                                                    break 'b1;
                                                                }
                                                                49 => {
                                                                    a0_1 = 1;
                                                                    a1_1 = 0;
                                                                    a2_1 = sp;
                                                                    v0_1 = sceMcSync(&[a0_1 as i64, a1_1 as i64, a2_1 as i64]);
                                                                    if v0_1 != 1 {
                                                                        break 'b1;
                                                                    }
                                                                    a0_2 = lw(sp) + 10;
                                                                    if (a0_2 as u32) >= (11 as u32) {
                                                                        break 'b5;
                                                                    }
                                                                    match (a0_2 << 2) >> 2 {
                                                                        0 | 1 | 2 | 3 | 4 | 5 | 7 | 8 | 9 => {
                                                                            break 'b5;
                                                                        }
                                                                        6 => {
                                                                            v0_2 = 5;
                                                                            break 'b4;
                                                                        }
                                                                        10 => {
                                                                            v0_2 = 1;
                                                                            break 'b4;
                                                                        }
                                                                        _ => unreachable!(),
                                                                    }
                                                                }
                                                                _ => unreachable!(),
                                                            }
                                                            sw(g::mcardstate, 44);
                                                            break 'b1;
                                                        }
                                                        sw(g::mcardstate, 36);
                                                        break 'b1;
                                                    }
                                                    v0_15 = 3;
                                                }
                                                sw(g::mcardresult, v0_15);
                                            }
                                            sw(g::mcardstate, 0);
                                            break 'b1;
                                        }
                                        sw(g::mcardstate, 30);
                                        break 'b1;
                                    }
                                    sw(g::mkdir_fd, v1_12);
                                    sw(g::mcardstate, v0_34);
                                    break 'b1;
                                }
                                sw(g::mcardstate, 24);
                                break 'b1;
                            }
                            sw(g::mcardresult, a2_18);
                            sw(g::mcardstate, 0);
                            break 'b1;
                        }
                        v0_2 = 3;
                    }
                    sw(g::mcardstate, 0);
                }
                sw(g::mcardresult, v0_2);
                break 'b1;
            }
            sw(g::mcardstate, 3);
        }
    }
    return;
}
