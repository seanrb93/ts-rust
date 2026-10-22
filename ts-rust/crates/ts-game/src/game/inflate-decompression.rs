//! Functions: the inflate decompressor of the packed files.

use crate::*;

/// Inflate a stream.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn Inflate() {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_3: i32 = 0;
    sw(0x3afec8, 0);
    sw(0x3afed8, 0);
    sw(0x3afed4, 0);
    'l1: loop {
        a0_1 = sp;
        v0_1 = inflate_block(a0_1);
        v0_2 = lw(sp);
        if v0_1 != 0 {
            a0_2 = 1;
            v0_2 = exit(a0_2);
        }
        a0_3 = lw(0x3afed8);
        if v0_2 != 0 {
            break;
        }
    }
    'b4: {
        v1_1 = lw(0x3afebc);
        if (a0_3 as u32) >= (8 as u32) {
            'l5: loop {
                v0_3 = a0_3 - 8;
                v1_1 = v1_1 - 1;
                a0_3 = v0_3;
                sw(0x3afed8, v0_3);
                sw(0x3afebc, v1_1);
                if (a0_3 as u32) < (8 as u32) {
                    break 'b4;
                }
            }
        }
    }
    FlushOut();
    return;
}

/// Inflate one block.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn inflate_block(mut a0: i32) -> i32 {
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_3: i32 = 0;
    'b1: {
        a1_1 = lw(0x3afed8);
        a2_1 = lw(0x3afed4);
        if a1_1 == 0 {
            a3_1 = lw(0x3afec0);
            v1_1 = lw(0x3afebc);
            'l2: loop {
                v0_1 = lw(0x3afeb8);
                if (v1_1 as u32) >= (a3_1 as u32) {
                    a2_1 = a2_1 | -1 << (a1_1 & 31);
                } else {
                    a0_1 = v1_1 + 1;
                    v1_2 = lbu(v0_1 + v1_1);
                    sw(0x3afebc, a0_1);
                    a2_1 = a2_1 | v1_2 << (a1_1 & 31);
                }
                a1_1 = a1_1 + 8;
                v1_1 = lw(0x3afebc);
                if a1_1 != 0 {
                    break 'b1;
                }
            }
        }
    }
    'b4: {
        a1_2 = a1_1 - 1;
        sw(a0, a2_1 & 1);
        a2_2 = ((a2_1 as u32) >> 1) as i32;
        if (a1_2 as u32) < (2 as u32) {
            a3_2 = lw(0x3afec0);
            v1_3 = lw(0x3afebc);
            'l5: loop {
                v0_2 = lw(0x3afeb8);
                if (v1_3 as u32) >= (a3_2 as u32) {
                    a2_2 = a2_2 | -1 << (a1_2 & 31);
                } else {
                    a0_2 = v1_3 + 1;
                    v1_4 = lbu(v0_2 + v1_3);
                    sw(0x3afebc, a0_2);
                    a2_2 = a2_2 | v1_4 << (a1_2 & 31);
                }
                a1_2 = a1_2 + 8;
                v1_3 = lw(0x3afebc);
                if (a1_2 as u32) >= (2 as u32) {
                    break 'b4;
                }
            }
        }
    }
    s0_1 = a2_2 & 3;
    sw(0x3afed8, a1_2 - 2);
    sw(0x3afed4, ((a2_2 as u32) >> 2) as i32);
    AllocReset();
    if s0_1 != 2 {
        if s0_1 != 0 {
            if s0_1 != 1 {
                a0_3 = 1;
                v0_3 = exit(a0_3);
            } else {
                v0_3 = inflate_fixed();
            }
        } else {
            v0_3 = inflate_stored();
        }
    } else {
        v0_3 = inflate_dynamic();
    }
    return v0_3;
}

/// Inflate a block with fixed codes.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn inflate_fixed() -> i32 {
    let frame = Frame::new(1184);
    let sp = frame.sp();
    let mut a5_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut v0_6: i32 = 0;
    a5_1 = sp + 1152;
    a6_1 = sp + 1156;
    a0_1 = 143;
    v0_1 = sp + 572;
    'l1: loop {
        sw(v0_1, 8);
        a0_1 = a0_1 - 1;
        v0_1 = v0_1 - 4;
        if a0_1 < 0 {
            break;
        }
    }
    v0_2 = sp + 576;
    a0_2 = 111;
    'l3: loop {
        sw(v0_2, 9);
        a0_2 = a0_2 - 1;
        v0_2 = v0_2 + 4;
        if a0_2 < 0 {
            break;
        }
    }
    a0_3 = 256;
    v1_1 = sp + 1024;
    'l5: loop {
        sw(v1_1, 7);
        a0_3 = a0_3 + 1;
        v1_1 = v1_1 + 4;
        if a0_3 >= 280 {
            break;
        }
    }
    'b7: {
        v0_3 = a0_3 << 2;
        if a0_3 < 288 {
            v1_2 = v0_3 + sp;
            'l8: loop {
                sw(v1_2, 8);
                a0_3 = a0_3 + 1;
                v1_2 = v1_2 + 4;
                if a0_3 >= 288 {
                    break 'b7;
                }
            }
        }
    }
    a3_1 = g::cplens;
    sw(sp + 1156, 7);
    a4_1 = g::cplext;
    a0_4 = sp;
    a1_1 = 288;
    a2_1 = 257;
    v0_4 = huft_build(a0_4, a1_1, a2_1, a3_1, a4_1, a5_1, a6_1);
    a0_5 = v0_4;
    if a0_5 == 0 {
        a5_2 = sp + 1160;
        a6_2 = sp + 1164;
        v1_3 = sp;
        'l10: loop {
            sw(v1_3, 5);
            a0_5 = a0_5 + 1;
            v1_3 = v1_3 + 4;
            if a0_5 >= 30 {
                break;
            }
        }
        a3_2 = g::cpdist;
        sw(sp + 1164, 5);
        a4_2 = g::cpdext;
        a0_6 = sp;
        a1_2 = 30;
        a2_2 = 0;
        v0_5 = huft_build(a0_6, a1_2, a2_2, a3_2, a4_2, a5_2, a6_2);
        if v0_5 < 2 {
            a0_7 = lw(sp + 1152);
            a1_3 = lw(sp + 1160);
            a2_3 = lw(sp + 1156);
            a3_3 = lw(sp + 1164);
            v0_6 = inflate_codes(a0_7, a1_3, a2_3, a3_3);
            v0_4 = ((0 as u32) < (v0_6 as u32)) as i32;
        } else {
            v0_4 = v0_5;
        }
    }
    return v0_4;
}

/// Inflate a stored block.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn inflate_stored() -> i32 {
    let mut a3_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    'b1: {
        a3_1 = lw(0x3afec8);
        s0_1 = lw(0x3afed8);
        a1_1 = s0_1 & 7;
        s0_2 = s0_1 - a1_1;
        s1_1 = ((lw(0x3afed4) as u32) >> (a1_1 & 31)) as i32;
        if (s0_2 as u32) < (16 as u32) {
            a2_1 = lw(0x3afec0);
            v1_1 = lw(0x3afebc);
            'l2: loop {
                v0_1 = lw(0x3afeb8);
                if (v1_1 as u32) >= (a2_1 as u32) {
                    s1_1 = s1_1 | -1 << (s0_2 & 31);
                } else {
                    a0_1 = v1_1 + 1;
                    v1_2 = lbu(v0_1 + v1_1);
                    sw(0x3afebc, a0_1);
                    s1_1 = s1_1 | v1_2 << (s0_2 & 31);
                }
                s0_2 = s0_2 + 8;
                v1_1 = lw(0x3afebc);
                if (s0_2 as u32) >= (16 as u32) {
                    break 'b1;
                }
            }
        }
    }
    'b4: {
        a1_2 = s1_1 & 0xffff;
        s0_3 = s0_2 - 16;
        s1_2 = ((s1_1 as u32) >> 16) as i32;
        if (s0_3 as u32) < (16 as u32) {
            a2_2 = lw(0x3afec0);
            v1_3 = lw(0x3afebc);
            'l5: loop {
                v0_2 = lw(0x3afeb8);
                if (v1_3 as u32) >= (a2_2 as u32) {
                    s1_2 = s1_2 | -1 << (s0_3 & 31);
                } else {
                    a0_2 = v1_3 + 1;
                    v1_4 = lbu(v0_2 + v1_3);
                    sw(0x3afebc, a0_2);
                    s1_2 = s1_2 | v1_4 << (s0_3 & 31);
                }
                s0_3 = s0_3 + 8;
                v1_3 = lw(0x3afebc);
                if (s0_3 as u32) >= (16 as u32) {
                    break 'b4;
                }
            }
        }
    }
    if a1_2 == !(0 | s1_2) & 0xffff {
        s1_2 = ((s1_2 as u32) >> 16) as i32;
    } else {
        a0_3 = 1;
        exit(a0_3);
    }
    'b8: {
        a1_3 = a1_2 - 1;
        s0_4 = s0_3 - 16;
        if a1_3 != -1 {
            s2_1 = -1;
            'l9: loop {
                s3_1 = a1_3 - 1;
                if (s0_4 as u32) >= (8 as u32) {
                    a1_4 = a3_1 + 1;
                    v0_4 = lw(0x3afec4) + a3_1;
                } else {
                    a2_3 = lw(0x3afec0);
                    a4_1 = lw(0x3afec4);
                    a1_4 = a3_1 + 1;
                    v1_5 = lw(0x3afebc);
                    'l11: loop {
                        v0_3 = lw(0x3afeb8);
                        if (v1_5 as u32) >= (a2_3 as u32) {
                            s1_2 = s1_2 | s2_1 << (s0_4 & 31);
                        } else {
                            a0_4 = v1_5 + 1;
                            v1_6 = lbu(v0_3 + v1_5);
                            sw(0x3afebc, a0_4);
                            s1_2 = s1_2 | v1_6 << (s0_4 & 31);
                        }
                        s0_4 = s0_4 + 8;
                        v1_5 = lw(0x3afebc);
                        if (s0_4 as u32) >= (8 as u32) {
                            break;
                        }
                    }
                    v0_4 = a4_1 + a3_1;
                }
                a3_1 = a1_4;
                sb(v0_4, s1_2);
                if a3_1 == 0x8000 {
                    sw(0x3afec8, a3_1);
                    FlushOut();
                    a3_1 = 0;
                }
                a1_3 = s3_1;
                s1_2 = ((s1_2 as u32) >> 8) as i32;
                s0_4 = s0_4 - 8;
                if a1_3 == s2_1 {
                    break 'b8;
                }
            }
        }
    }
    sw(0x3afed4, s1_2);
    v0_5 = 0;
    sw(0x3afed8, s0_4);
    sw(0x3afec8, a3_1);
    return v0_5;
}

/// Inflate a block with dynamic codes.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn inflate_dynamic() -> i32 {
    let frame = Frame::new(1392);
    let sp = frame.sp();
    let mut s0_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut s1_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut a5_3: i32 = 0;
    let mut t4_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v1_15: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut v1_16: i32 = 0;
    let mut a3_8: i32 = 0;
    let mut v1_17: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v1_18: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut v1_19: i32 = 0;
    let mut a3_9: i32 = 0;
    let mut v1_20: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v1_21: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut v0_23: i32 = 0;
    let mut a3_10: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut a5_4: i32 = 0;
    let mut a6_3: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut v0_24: i32 = 0;
    let mut a0_11: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a3_11: i32 = 0;
    let mut a4_5: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut a5_5: i32 = 0;
    let mut a6_4: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut a0_14: i32 = 0;
    let mut a1_8: i32 = 0;
    let mut a2_11: i32 = 0;
    let mut a3_12: i32 = 0;
    let mut v0_26: i32 = 0;
    'b1: {
        s0_1 = lw(0x3afed8);
        s1_1 = lw(0x3afed4);
        if (s0_1 as u32) < (5 as u32) {
            a3_1 = lw(0x3afec0);
            v1_1 = lw(0x3afebc);
            'l2: loop {
                v0_1 = lw(0x3afeb8);
                if (v1_1 as u32) >= (a3_1 as u32) {
                    s1_1 = s1_1 | -1 << (s0_1 & 31);
                } else {
                    a0_1 = v1_1 + 1;
                    v1_2 = lbu(v0_1 + v1_1);
                    sw(0x3afebc, a0_1);
                    s1_1 = s1_1 | v1_2 << (s0_1 & 31);
                }
                s0_1 = s0_1 + 8;
                v1_1 = lw(0x3afebc);
                if (s0_1 as u32) >= (5 as u32) {
                    break 'b1;
                }
            }
        }
    }
    s0_2 = s0_1 - 5;
    s2_1 = (s1_1 & 31) + 257;
    s1_2 = ((s1_1 as u32) >> 5) as i32;
    if (s0_2 as u32) >= (5 as u32) {
        a1_1 = ((s2_1 as u32) < (287 as u32)) as i32;
        v0_3 = s1_2 & 31;
    } else {
        a3_2 = lw(0x3afec0);
        a1_1 = ((s2_1 as u32) < (287 as u32)) as i32;
        v1_3 = lw(0x3afebc);
        'l5: loop {
            v0_2 = lw(0x3afeb8);
            if (v1_3 as u32) >= (a3_2 as u32) {
                s1_2 = s1_2 | -1 << (s0_2 & 31);
            } else {
                a0_2 = v1_3 + 1;
                v1_4 = lbu(v0_2 + v1_3);
                sw(0x3afebc, a0_2);
                s1_2 = s1_2 | v1_4 << (s0_2 & 31);
            }
            s0_2 = s0_2 + 8;
            v1_3 = lw(0x3afebc);
            if (s0_2 as u32) >= (5 as u32) {
                break;
            }
        }
        v0_3 = s1_2 & 31;
    }
    'b8: {
        s0_3 = s0_2 - 5;
        s5_1 = v0_3 + 1;
        s1_3 = ((s1_2 as u32) >> 5) as i32;
        if (s0_3 as u32) < (4 as u32) {
            a3_3 = lw(0x3afec0);
            v1_5 = lw(0x3afebc);
            'l9: loop {
                v0_4 = lw(0x3afeb8);
                if (v1_5 as u32) >= (a3_3 as u32) {
                    s1_3 = s1_3 | -1 << (s0_3 & 31);
                } else {
                    a0_3 = v1_5 + 1;
                    v1_6 = lbu(v0_4 + v1_5);
                    sw(0x3afebc, a0_3);
                    s1_3 = s1_3 | v1_6 << (s0_3 & 31);
                }
                s0_3 = s0_3 + 8;
                v1_5 = lw(0x3afebc);
                if (s0_3 as u32) >= (4 as u32) {
                    break 'b8;
                }
            }
        }
    }
    'b11: {
        'b12: {
            s0_4 = s0_3 - 4;
            a5_1 = (s1_3 & 15) + 4;
            s1_4 = ((s1_3 as u32) >> 4) as i32;
            if a1_1 != 0 {
                v0_5 = 1;
                if (s5_1 as u32) >= (31 as u32) {
                    break 'b11;
                }
                a2_1 = 0;
                if a5_1 == 0 {
                    v0_8 = ((a2_1 as u32) < (19 as u32)) as i32;
                } else {
                    'l14: loop {
                        a3_4 = lw(0x3afec0);
                        if (s0_4 as u32) >= (3 as u32) {
                            a4_1 = a2_1 + 1;
                            a1_2 = a2_1 << 2;
                            v0_7 = g::border;
                        } else {
                            a4_1 = a2_1 + 1;
                            a1_2 = a2_1 << 2;
                            v1_7 = lw(0x3afebc);
                            'l17: loop {
                                v0_6 = lw(0x3afeb8);
                                if (v1_7 as u32) >= (a3_4 as u32) {
                                    s1_4 = s1_4 | -1 << (s0_4 & 31);
                                } else {
                                    a0_4 = v1_7 + 1;
                                    v1_8 = lbu(v0_6 + v1_7);
                                    sw(0x3afebc, a0_4);
                                    s1_4 = s1_4 | v1_8 << (s0_4 & 31);
                                }
                                s0_4 = s0_4 + 8;
                                v1_7 = lw(0x3afebc);
                                if (s0_4 as u32) >= (3 as u32) {
                                    break;
                                }
                            }
                            v0_7 = g::border;
                        }
                        a2_1 = a4_1;
                        s0_4 = s0_4 - 3;
                        v1_9 = lw(a1_2 + v0_7);
                        a1_3 = s1_4 & 7;
                        s1_4 = ((s1_4 as u32) >> 3) as i32;
                        sw(sp + (v1_9 << 2), a1_3);
                        if (a2_1 as u32) >= (a5_1 as u32) {
                            break;
                        }
                    }
                    v0_8 = ((a2_1 as u32) < (19 as u32)) as i32;
                }
                'b20: {
                    if v0_8 != 0 {
                        'l21: loop {
                            v1_10 = (a2_1 << 2) + g::border;
                            a2_1 = a2_1 + 1;
                            sw(sp + (lw(v1_10) << 2), 0);
                            if (a2_1 as u32) >= (19 as u32) {
                                break 'b20;
                            }
                        }
                    }
                }
                a0_5 = sp;
                sw(sp + 1268, 7);
                a1_4 = 19;
                a2_2 = 19;
                a3_5 = 0;
                a4_2 = 0;
                a5_2 = sp + 1264;
                a6_1 = sp + 1268;
                v0_5 = huft_build(a0_5, a1_4, a2_2, a3_5, a4_2, a5_2, a6_1);
                a1_5 = v0_5;
                if a1_5 != 0 {
                    break 'b11;
                }
                'b22: {
                    a2_3 = lw(sp + 1268);
                    a4_3 = s2_1 + s5_1;
                    a6_2 = 0;
                    a5_3 = ((a1_5 as u32) < (a4_3 as u32)) as i32;
                    t4_1 = lhu((a2_3 << 1) + g::mask_bits);
                    if a5_3 != 0 {
                        a7_1 = lw(sp + 1264);
                        'l23: loop {
                            v0_9 = s1_4 & t4_1;
                            if (s0_4 as u32) < (a2_3 as u32) {
                                a3_6 = lw(0x3afec0);
                                v1_11 = lw(0x3afebc);
                                'l25: loop {
                                    v0_10 = lw(0x3afeb8);
                                    if (v1_11 as u32) >= (a3_6 as u32) {
                                        s1_4 = s1_4 | -1 << (s0_4 & 31);
                                    } else {
                                        a0_6 = v1_11 + 1;
                                        v1_12 = lbu(v0_10 + v1_11);
                                        sw(0x3afebc, a0_6);
                                        s1_4 = s1_4 | v1_12 << (s0_4 & 31);
                                    }
                                    s0_4 = s0_4 + 8;
                                    v1_11 = lw(0x3afebc);
                                    if (s0_4 as u32) >= (a2_3 as u32) {
                                        break;
                                    }
                                }
                                v0_9 = s1_4 & t4_1;
                            }
                            v0_11 = a7_1 + (v0_9 << 3);
                            sw(sp + 1272, v0_11);
                            a2_4 = lbu(v0_11 + 1);
                            s0_4 = s0_4 - a2_4;
                            s1_4 = ((s1_4 as u32) >> (a2_4 & 31)) as i32;
                            a2_5 = lhu(v0_11 + 4);
                            v0_12 = a1_5 << 2;
                            if (a2_5 as u32) >= (16 as u32) {
                                if a2_5 != 16 {
                                    v0_16 = ((s0_4 as u32) < (7 as u32)) as i32;
                                    if a2_5 != 17 {
                                        v1_19 = s1_4 & 127;
                                        if v0_16 != 0 {
                                            a3_9 = lw(0x3afec0);
                                            v1_20 = lw(0x3afebc);
                                            'l31: loop {
                                                v0_20 = lw(0x3afeb8);
                                                if (v1_20 as u32) >= (a3_9 as u32) {
                                                    s1_4 = s1_4 | -1 << (s0_4 & 31);
                                                } else {
                                                    a0_9 = v1_20 + 1;
                                                    v1_21 = lbu(v0_20 + v1_20);
                                                    sw(0x3afebc, a0_9);
                                                    s1_4 = s1_4 | v1_21 << (s0_4 & 31);
                                                }
                                                s0_4 = s0_4 + 8;
                                                v1_20 = lw(0x3afebc);
                                                if (s0_4 as u32) >= (7 as u32) {
                                                    break;
                                                }
                                            }
                                            v1_19 = s1_4 & 127;
                                        }
                                        s0_4 = s0_4 - 7;
                                        s1_4 = ((s1_4 as u32) >> 7) as i32;
                                        if (a4_3 as u32) < ((a1_5 + (v1_19 + 11)) as u32) {
                                            break 'b12;
                                        }
                                        a2_8 = v1_19 + 10;
                                        v0_21 = a1_5 << 2;
                                        if a2_8 != -1 {
                                            v0_22 = v0_21 + sp;
                                            'l34: loop {
                                                sw(v0_22, 0);
                                                a1_5 = a1_5 + 1;
                                                v0_22 = v0_22 + 4;
                                                a2_8 = a2_8 - 1;
                                                if a2_8 == -1 {
                                                    break;
                                                }
                                            }
                                            a5_3 = ((a1_5 as u32) < (a4_3 as u32)) as i32;
                                        }
                                    } else {
                                        v1_16 = s1_4 & 7;
                                        if (s0_4 as u32) < (3 as u32) {
                                            a3_8 = lw(0x3afec0);
                                            v1_17 = lw(0x3afebc);
                                            'l37: loop {
                                                v0_17 = lw(0x3afeb8);
                                                if (v1_17 as u32) >= (a3_8 as u32) {
                                                    s1_4 = s1_4 | -1 << (s0_4 & 31);
                                                } else {
                                                    a0_8 = v1_17 + 1;
                                                    v1_18 = lbu(v0_17 + v1_17);
                                                    sw(0x3afebc, a0_8);
                                                    s1_4 = s1_4 | v1_18 << (s0_4 & 31);
                                                }
                                                s0_4 = s0_4 + 8;
                                                v1_17 = lw(0x3afebc);
                                                if (s0_4 as u32) >= (3 as u32) {
                                                    break;
                                                }
                                            }
                                            v1_16 = s1_4 & 7;
                                        }
                                        s0_4 = s0_4 - 3;
                                        s1_4 = ((s1_4 as u32) >> 3) as i32;
                                        if (a4_3 as u32) < ((a1_5 + (v1_16 + 3)) as u32) {
                                            break 'b12;
                                        }
                                        a2_7 = v1_16 + 2;
                                        v0_18 = a1_5 << 2;
                                        if a2_7 != -1 {
                                            v0_19 = v0_18 + sp;
                                            'l40: loop {
                                                sw(v0_19, 0);
                                                a1_5 = a1_5 + 1;
                                                v0_19 = v0_19 + 4;
                                                a2_7 = a2_7 - 1;
                                                if a2_7 == -1 {
                                                    break;
                                                }
                                            }
                                            a5_3 = ((a1_5 as u32) < (a4_3 as u32)) as i32;
                                        }
                                    }
                                    a6_2 = 0;
                                } else {
                                    v1_13 = s1_4 & 3;
                                    if (s0_4 as u32) < (2 as u32) {
                                        a3_7 = lw(0x3afec0);
                                        v1_14 = lw(0x3afebc);
                                        'l43: loop {
                                            v0_13 = lw(0x3afeb8);
                                            if (v1_14 as u32) >= (a3_7 as u32) {
                                                s1_4 = s1_4 | -1 << (s0_4 & 31);
                                            } else {
                                                a0_7 = v1_14 + 1;
                                                v1_15 = lbu(v0_13 + v1_14);
                                                sw(0x3afebc, a0_7);
                                                s1_4 = s1_4 | v1_15 << (s0_4 & 31);
                                            }
                                            s0_4 = s0_4 + 8;
                                            v1_14 = lw(0x3afebc);
                                            if (s0_4 as u32) >= (2 as u32) {
                                                break;
                                            }
                                        }
                                        v1_13 = s1_4 & 3;
                                    }
                                    s0_4 = s0_4 - 2;
                                    s1_4 = ((s1_4 as u32) >> 2) as i32;
                                    if (a4_3 as u32) < ((a1_5 + (v1_13 + 3)) as u32) {
                                        break 'b12;
                                    }
                                    a2_6 = v1_13 + 2;
                                    v0_14 = a1_5 << 2;
                                    if a2_6 != -1 {
                                        v0_15 = v0_14 + sp;
                                        'l46: loop {
                                            sw(v0_15, a6_2);
                                            a1_5 = a1_5 + 1;
                                            v0_15 = v0_15 + 4;
                                            a2_6 = a2_6 - 1;
                                            if a2_6 == -1 {
                                                break;
                                            }
                                        }
                                        a5_3 = ((a1_5 as u32) < (a4_3 as u32)) as i32;
                                    }
                                }
                            } else {
                                a6_2 = a2_5;
                                a1_5 = a1_5 + 1;
                                sw(sp + v0_12, a6_2);
                                a5_3 = ((a1_5 as u32) < (a4_3 as u32)) as i32;
                            }
                            a2_3 = lw(sp + 1268);
                            if a5_3 == 0 {
                                break 'b22;
                            }
                        }
                    }
                }
                v0_23 = lw(g::lbits);
                sw(0x3afed4, s1_4);
                sw(0x3afed8, s0_4);
                a3_10 = g::cplens;
                sw(sp + 1268, v0_23);
                a4_4 = g::cplext;
                a5_4 = sp + 1264;
                a6_3 = sp + 1268;
                a0_10 = sp;
                a1_6 = s2_1;
                a2_9 = 257;
                v0_24 = huft_build(a0_10, a1_6, a2_9, a3_10, a4_4, a5_4, a6_3);
                if v0_24 != 0 {
                    v0_5 = v0_24;
                    if v0_24 != 1 {
                        break 'b11;
                    }
                    a0_11 = 1;
                    exit(a0_11);
                }
                sw(sp + 1276, lw(g::dbits));
                a0_12 = sp + (s2_1 << 2);
                a1_7 = s5_1;
                a3_11 = g::cpdist;
                a4_5 = g::cpdext;
                a2_10 = 0;
                a5_5 = sp + 1272;
                a6_4 = sp + 1276;
                v0_25 = huft_build(a0_12, a1_7, a2_10, a3_11, a4_5, a5_5, a6_4);
                if v0_25 == 0 {
                    a0_14 = lw(sp + 1264);
                    a1_8 = lw(sp + 1272);
                    a2_11 = lw(sp + 1268);
                    a3_12 = lw(sp + 1276);
                    v0_26 = inflate_codes(a0_14, a1_8, a2_11, a3_12);
                    v0_5 = ((0 as u32) < (v0_26 as u32)) as i32;
                    break 'b11;
                }
                v0_5 = v0_25;
                if v0_25 != 1 {
                    break 'b11;
                }
                a0_13 = 1;
                exit(a0_13);
            }
        }
        v0_5 = 1;
    }
    return v0_5;
}

/// Inflate the codes of a block.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn inflate_codes(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32) -> i32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut a2_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut s2_2: i32 = 0;
    let mut s6_1: i32 = 0;
    let mut s3_2: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a7_2: i32 = 0;
    let mut bc_1: bool = false;
    let mut s0_4: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut v1_15: i32 = 0;
    let mut s2_3: i32 = 0;
    let mut s3_3: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v1_16: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut v1_17: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s0_5: i32 = 0;
    let mut s0_6: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v1_18: i32 = 0;
    let mut pc: u32 = 0x21ad98;
    'disp: loop {
        match pc {
            0x21ad98 => {
                a2_1 = g::mask_bits;
                s3_1 = lw(0x3afed4);
                v0_1 = lhu((a2 << 1) + a2_1);
                sw(sp, a0);
                sw(sp + 8, v0_1);
                s2_1 = lw(0x3afed8);
                v1_1 = lhu((a3 << 1) + a2_1);
                sw(sp + 4, a1);
                s4_1 = lw(0x3afec8);
                sw(sp + 12, v1_1);
                pc = 0x21b114; continue 'disp;
            }
            0x21b114 => {
                s5_1 = ((s2_1 as u32) < (a2 as u32)) as i32;
                pc = 0x21b118; continue 'disp;
            }
            0x21b118 => {
                v1_2 = lw(sp + 8);
                if s5_1 == 0 {
                    pc = 0x21b174; continue 'disp;
                } else {
                    pc = 0x21b120; continue 'disp;
                }
            }
            0x21b120 => {
                a1_1 = lw(0x3afec0);
                v1_3 = lw(0x3afebc);
                pc = 0x21b128; continue 'disp;
            }
            0x21b128 => {
                v0_2 = lw(0x3afeb8);
                if (v1_3 as u32) >= (a1_1 as u32) {
                    pc = 0x21b150; continue 'disp;
                } else {
                    pc = 0x21b134; continue 'disp;
                }
            }
            0x21b134 => {
                a0_1 = v1_3 + 1;
                v1_4 = lbu(v0_2 + v1_3);
                sw(0x3afebc, a0_1);
                s3_1 = s3_1 | v1_4 << (s2_1 & 31);
                pc = 0x21b160; continue 'disp;
            }
            0x21b150 => {
                s3_1 = s3_1 | -1 << (s2_1 & 31);
                pc = 0x21b160; continue 'disp;
            }
            0x21b160 => {
                s2_1 = s2_1 + 8;
                v1_3 = lw(0x3afebc);
                if (s2_1 as u32) < (a2 as u32) {
                    pc = 0x21b128; continue 'disp;
                } else {
                    pc = 0x21b170; continue 'disp;
                }
            }
            0x21b170 => {
                v1_2 = lw(sp + 8);
                pc = 0x21b174; continue 'disp;
            }
            0x21b174 => {
                a4_1 = lw(sp) + ((s3_1 & v1_2) << 3);
                s0_1 = lbu(a4_1);
                if (s0_1 as u32) < (17 as u32) {
                    pc = 0x21b190; continue 'disp;
                } else {
                    pc = 0x21b194; continue 'disp;
                }
            }
            0x21b194 => {
                a5_1 = lw(0x3afeb8);
                a7_1 = g::mask_bits;
                pc = 0x21b1b0; continue 'disp;
            }
            0x21b1b0 => {
                if s0_1 != 99 {
                    pc = 0x21b1b4; continue 'disp;
                } else {
                    pc = 0x21b1b8; continue 'disp;
                }
            }
            0x21b1b4 => {
                v1_5 = lbu(a4_1 + 1);
                s0_2 = s0_1 - 16;
                s2_1 = s2_1 - v1_5;
                s3_1 = ((s3_1 as u32) >> (v1_5 & 31)) as i32;
                if (s2_1 as u32) >= (s0_2 as u32) {
                    pc = 0x21b224; continue 'disp;
                } else {
                    pc = 0x21b1d4; continue 'disp;
                }
            }
            0x21b1d4 => {
                a1_2 = lw(0x3afec0);
                a0_2 = lw(0x3afebc);
                pc = 0x21b1e0; continue 'disp;
            }
            0x21b1e0 => {
                v0_3 = a5_1 + a0_2;
                if (a0_2 as u32) >= (a1_2 as u32) {
                    pc = 0x21b204; continue 'disp;
                } else {
                    pc = 0x21b1ec; continue 'disp;
                }
            }
            0x21b1ec => {
                v1_6 = lbu(v0_3);
                sw(0x3afebc, a0_2 + 1);
                s3_1 = s3_1 | v1_6 << (s2_1 & 31);
                pc = 0x21b20c; continue 'disp;
            }
            0x21b204 => {
                s3_1 = s3_1 | -1 << (s2_1 & 31);
                pc = 0x21b20c; continue 'disp;
            }
            0x21b20c => {
                s2_1 = s2_1 + 8;
                a0_2 = lw(0x3afebc);
                if (s2_1 as u32) < (s0_2 as u32) {
                    pc = 0x21b1e0; continue 'disp;
                } else {
                    pc = 0x21b21c; continue 'disp;
                }
            }
            0x21b21c => {
                v1_7 = (s0_2 << 1) + a7_1;
                pc = 0x21b22c; continue 'disp;
            }
            0x21b224 => {
                v1_7 = (s0_2 << 1) + a7_1;
                pc = 0x21b22c; continue 'disp;
            }
            0x21b22c => {
                a4_1 = lw(a4_1 + 4) + ((s3_1 & lhu(v1_7)) << 3);
                s0_1 = lbu(a4_1);
                if (s0_1 as u32) >= (17 as u32) {
                    pc = 0x21b1b0; continue 'disp;
                } else {
                    pc = 0x21b250; continue 'disp;
                }
            }
            0x21b250 => {
                v0_4 = lbu(a4_1 + 1);
                pc = 0x21b254; continue 'disp;
            }
            0x21b190 => {
                v0_4 = lbu(a4_1 + 1);
                pc = 0x21b254; continue 'disp;
            }
            0x21b254 => {
                s2_1 = s2_1 - v0_4;
                s3_1 = ((s3_1 as u32) >> (v0_4 & 31)) as i32;
                if s0_1 != 16 {
                    pc = 0x21b294; continue 'disp;
                } else {
                    pc = 0x21b264; continue 'disp;
                }
            }
            0x21b264 => {
                v0_5 = lw(0x3afec4) + s4_1;
                s4_1 = s4_1 + 1;
                sb(v0_5, lbu(a4_1 + 4));
                if s4_1 != 0x8000 {
                    pc = 0x21b114; continue 'disp;
                } else {
                    pc = 0x21b280; continue 'disp;
                }
            }
            0x21b280 => {
                sw(0x3afec8, s4_1);
                s5_1 = ((s2_1 as u32) < (a2 as u32)) as i32;
                FlushOut();
                s4_1 = 0;
                pc = 0x21b118; continue 'disp;
            }
            0x21b294 => {
                v0_6 = ((s2_1 as u32) < (s0_1 as u32)) as i32;
                if s0_1 != 15 {
                    pc = 0x21ae10; continue 'disp;
                } else {
                    pc = 0x21b2a0; continue 'disp;
                }
            }
            0x21b2a0 => {
                sw(0x3afec8, s4_1);
                v0_7 = 0;
                sw(0x3afed4, s3_1);
                sw(0x3afed8, s2_1);
                pc = 0x21b2b0; continue 'disp;
            }
            0x21ae10 => {
                a1_3 = lw(0x3afec0);
                if v0_6 == 0 {
                    pc = 0x21ae70; continue 'disp;
                } else {
                    pc = 0x21ae18; continue 'disp;
                }
            }
            0x21ae18 => {
                a3_1 = s0_1 << 1;
                v1_8 = lw(0x3afebc);
                pc = 0x21ae20; continue 'disp;
            }
            0x21ae20 => {
                v0_8 = lw(0x3afeb8);
                if (v1_8 as u32) >= (a1_3 as u32) {
                    pc = 0x21ae48; continue 'disp;
                } else {
                    pc = 0x21ae2c; continue 'disp;
                }
            }
            0x21ae2c => {
                a0_3 = v1_8 + 1;
                v1_9 = lbu(v0_8 + v1_8);
                sw(0x3afebc, a0_3);
                s3_1 = s3_1 | v1_9 << (s2_1 & 31);
                pc = 0x21ae58; continue 'disp;
            }
            0x21ae48 => {
                s3_1 = s3_1 | -1 << (s2_1 & 31);
                pc = 0x21ae58; continue 'disp;
            }
            0x21ae58 => {
                s2_1 = s2_1 + 8;
                v1_8 = lw(0x3afebc);
                if (s2_1 as u32) < (s0_1 as u32) {
                    pc = 0x21ae20; continue 'disp;
                } else {
                    pc = 0x21ae78; continue 'disp;
                }
            }
            0x21ae70 => {
                a3_1 = s0_1 << 1;
                pc = 0x21ae78; continue 'disp;
            }
            0x21ae78 => {
                s2_2 = s2_1 - s0_1;
                s6_1 = lhu(a4_1 + 4) + (s3_1 & lhu(a3_1 + g::mask_bits));
                s3_2 = ((s3_1 as u32) >> (s0_1 & 31)) as i32;
                if (s2_2 as u32) >= (a3 as u32) {
                    pc = 0x21aef0; continue 'disp;
                } else {
                    pc = 0x21aea0; continue 'disp;
                }
            }
            0x21aea0 => {
                a1_4 = lw(0x3afec0);
                v1_10 = lw(0x3afebc);
                pc = 0x21aea8; continue 'disp;
            }
            0x21aea8 => {
                v0_9 = lw(0x3afeb8);
                if (v1_10 as u32) >= (a1_4 as u32) {
                    pc = 0x21aed0; continue 'disp;
                } else {
                    pc = 0x21aeb4; continue 'disp;
                }
            }
            0x21aeb4 => {
                a0_4 = v1_10 + 1;
                v1_11 = lbu(v0_9 + v1_10);
                sw(0x3afebc, a0_4);
                s3_2 = s3_2 | v1_11 << (s2_2 & 31);
                pc = 0x21aee0; continue 'disp;
            }
            0x21aed0 => {
                s3_2 = s3_2 | -1 << (s2_2 & 31);
                pc = 0x21aee0; continue 'disp;
            }
            0x21aee0 => {
                s2_2 = s2_2 + 8;
                v1_10 = lw(0x3afebc);
                if (s2_2 as u32) < (a3 as u32) {
                    pc = 0x21aea8; continue 'disp;
                } else {
                    pc = 0x21aef0; continue 'disp;
                }
            }
            0x21aef0 => {
                a4_2 = lw(sp + 4) + ((s3_2 & lw(sp + 12)) << 3);
                s0_3 = lbu(a4_2);
                if (s0_3 as u32) < (17 as u32) {
                    pc = 0x21af10; continue 'disp;
                } else {
                    pc = 0x21af14; continue 'disp;
                }
            }
            0x21af14 => {
                a5_2 = lw(0x3afeb8);
                a7_2 = g::mask_bits;
                pc = 0x21af30; continue 'disp;
            }
            0x21af30 => {
                bc_1 = s0_3 == 99;
                s0_4 = s0_3 - 16;
                if bc_1 {
                    pc = 0x21b1b8; continue 'disp;
                } else {
                    pc = 0x21af38; continue 'disp;
                }
            }
            0x21af38 => {
                v1_12 = lbu(a4_2 + 1);
                s2_2 = s2_2 - v1_12;
                s3_2 = ((s3_2 as u32) >> (v1_12 & 31)) as i32;
                if (s2_2 as u32) >= (s0_4 as u32) {
                    pc = 0x21af9c; continue 'disp;
                } else {
                    pc = 0x21af4c; continue 'disp;
                }
            }
            0x21af4c => {
                a1_5 = lw(0x3afec0);
                a0_5 = lw(0x3afebc);
                pc = 0x21af58; continue 'disp;
            }
            0x21af58 => {
                v0_10 = a5_2 + a0_5;
                if (a0_5 as u32) >= (a1_5 as u32) {
                    pc = 0x21af7c; continue 'disp;
                } else {
                    pc = 0x21af64; continue 'disp;
                }
            }
            0x21af64 => {
                v1_13 = lbu(v0_10);
                sw(0x3afebc, a0_5 + 1);
                s3_2 = s3_2 | v1_13 << (s2_2 & 31);
                pc = 0x21af84; continue 'disp;
            }
            0x21af7c => {
                s3_2 = s3_2 | -1 << (s2_2 & 31);
                pc = 0x21af84; continue 'disp;
            }
            0x21af84 => {
                s2_2 = s2_2 + 8;
                a0_5 = lw(0x3afebc);
                if (s2_2 as u32) < (s0_4 as u32) {
                    pc = 0x21af58; continue 'disp;
                } else {
                    pc = 0x21af94; continue 'disp;
                }
            }
            0x21af94 => {
                v1_14 = (s0_4 << 1) + a7_2;
                pc = 0x21afa4; continue 'disp;
            }
            0x21af9c => {
                v1_14 = (s0_4 << 1) + a7_2;
                pc = 0x21afa4; continue 'disp;
            }
            0x21afa4 => {
                a4_2 = lw(a4_2 + 4) + ((s3_2 & lhu(v1_14)) << 3);
                s0_3 = lbu(a4_2);
                if (s0_3 as u32) >= (17 as u32) {
                    pc = 0x21af30; continue 'disp;
                } else {
                    pc = 0x21afc8; continue 'disp;
                }
            }
            0x21afc8 => {
                v1_15 = lbu(a4_2 + 1);
                pc = 0x21afcc; continue 'disp;
            }
            0x21b1b8 => {
                v0_7 = 1;
                pc = 0x21b2b0; continue 'disp;
            }
            0x21b2b0 => {
                return v0_7;
            }
            0x21af10 => {
                v1_15 = lbu(a4_2 + 1);
                pc = 0x21afcc; continue 'disp;
            }
            0x21afcc => {
                s2_3 = s2_2 - v1_15;
                s3_3 = ((s3_2 as u32) >> (v1_15 & 31)) as i32;
                if (s2_3 as u32) >= (s0_3 as u32) {
                    pc = 0x21b040; continue 'disp;
                } else {
                    pc = 0x21afdc; continue 'disp;
                }
            }
            0x21afdc => {
                a1_6 = lw(0x3afec0);
                a3_2 = s0_3 << 1;
                a2_2 = lw(0x3afec4);
                v1_16 = lw(0x3afebc);
                pc = 0x21aff0; continue 'disp;
            }
            0x21aff0 => {
                v0_11 = lw(0x3afeb8);
                if (v1_16 as u32) >= (a1_6 as u32) {
                    pc = 0x21b018; continue 'disp;
                } else {
                    pc = 0x21affc; continue 'disp;
                }
            }
            0x21affc => {
                a0_6 = v1_16 + 1;
                v1_17 = lbu(v0_11 + v1_16);
                sw(0x3afebc, a0_6);
                s3_3 = s3_3 | v1_17 << (s2_3 & 31);
                pc = 0x21b028; continue 'disp;
            }
            0x21b018 => {
                s3_3 = s3_3 | -1 << (s2_3 & 31);
                pc = 0x21b028; continue 'disp;
            }
            0x21b028 => {
                s2_3 = s2_3 + 8;
                v1_16 = lw(0x3afebc);
                if (s2_3 as u32) < (s0_3 as u32) {
                    pc = 0x21aff0; continue 'disp;
                } else {
                    pc = 0x21b04c; continue 'disp;
                }
            }
            0x21b040 => {
                a2_2 = lw(0x3afec4);
                a3_2 = s0_3 << 1;
                pc = 0x21b04c; continue 'disp;
            }
            0x21b04c => {
                s2_1 = s2_3 - s0_3;
                s5_1 = ((s2_1 as u32) < (a2 as u32)) as i32;
                s1_1 = s4_1 - lhu(a4_2 + 4) - (s3_3 & lhu(a3_2 + g::mask_bits));
                s3_1 = ((s3_3 as u32) >> (s0_3 & 31)) as i32;
                pc = 0x21b078; continue 'disp;
            }
            0x21b078 => {
                s1_1 = s1_1 & 0x7fff;
                if (s4_1 as u32) >= (s1_1 as u32) {
                    pc = 0x21b090; continue 'disp;
                } else {
                    pc = 0x21b088; continue 'disp;
                }
            }
            0x21b088 => {
                s0_5 = 0x8000 - s1_1;
                pc = 0x21b094; continue 'disp;
            }
            0x21b090 => {
                s0_5 = 0x8000 - s4_1;
                pc = 0x21b094; continue 'disp;
            }
            0x21b094 => {
                s0_6 = if (s6_1 as u32) < (s0_5 as u32) { s6_1 } else { s0_5 };
                s6_1 = s6_1 - s0_6;
                if ((s4_1 - s1_1) as u32) < (s0_6 as u32) {
                    pc = 0x21b0cc; continue 'disp;
                } else {
                    pc = 0x21b0ac; continue 'disp;
                }
            }
            0x21b0ac => {
                a1_7 = a2_2 + s1_1;
                a0_7 = a2_2 + s4_1;
                a2_3 = s0_6;
                s1_1 = s1_1 + s0_6;
                memcpy(a0_7, a1_7, a2_3);
                s4_1 = s4_1 + s0_6;
                pc = 0x21b0ec; continue 'disp;
            }
            0x21b0cc => {
                a0_8 = a2_2 + s4_1;
                v1_18 = lbu(a2_2 + s1_1);
                s1_1 = s1_1 + 1;
                s4_1 = s4_1 + 1;
                s0_6 = s0_6 - 1;
                sb(a0_8, v1_18);
                if s0_6 != 0 {
                    pc = 0x21b0c8; continue 'disp;
                } else {
                    pc = 0x21b0ec; continue 'disp;
                }
            }
            0x21b0ec => {
                if s4_1 != 0x8000 {
                    pc = 0x21b104; continue 'disp;
                } else {
                    pc = 0x21b0f8; continue 'disp;
                }
            }
            0x21b0f8 => {
                sw(0x3afec8, s4_1);
                FlushOut();
                s4_1 = 0;
                pc = 0x21b104; continue 'disp;
            }
            0x21b104 => {
                a2_2 = lw(0x3afec4);
                if s6_1 != 0 {
                    pc = 0x21b078; continue 'disp;
                } else {
                    pc = 0x21b118; continue 'disp;
                }
            }
            0x21b0c8 => {
                a2_2 = lw(0x3afec4);
                pc = 0x21b0cc; continue 'disp;
            }
            _ => unreachable!(),
        }
    }
}

/// Build a Huffman table.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn huft_build(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32, mut a5: i32, mut a6: i32) -> i32 {
    let frame = Frame::new(1744);
    let sp = frame.sp();
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut s6_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut s7_1: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s6_2: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut t6_1: u128 = 0;
    let mut t4_1: i32 = 0;
    let mut s6_3: u128 = 0;
    let mut v1_2: i32 = 0;
    let mut t4_2: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut t4_3: u128 = 0;
    let mut v0_6: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut s4_2: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut s4_3: i32 = 0;
    let mut s1_4: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s1_5: i32 = 0;
    let mut a7_1: u128 = 0;
    let mut s4_4: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a6_1: u128 = 0;
    let mut t7_1: u128 = 0;
    let mut s5_1: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut t5_1: u128 = 0;
    let mut fp_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut a4_1: u128 = 0;
    let mut a6_2: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a7_2: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut a6_3: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a2_4: u128 = 0;
    let mut v1_7: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v1_8: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut s0_5: i32 = 0;
    let mut t8_1: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut s0_6: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut s0_7: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut t9_1: i32 = 0;
    let mut a0_11: i32 = 0;
    sw(sp + 1392, a1);
    a0_1 = sp;
    sw(sp + 1396, a2);
    a1_1 = 0;
    a2_1 = 68;
    s4_1 = a0;
    sw(sp + 1400, a3);
    sw(sp + 1404, a4);
    sw(sp + 1408, a5);
    memset(a0_1, a1_1, a2_1);
    s1_1 = lw(sp + 1392);
    'l1: loop {
        v0_1 = lw(s4_1);
        s1_1 = s1_1 - 1;
        s4_1 = s4_1 + 4;
        a0_2 = sp + (v0_1 << 2);
        sw(a0_2, lw(a0_2) + 1);
        if s1_1 == 0 {
            break;
        }
    }
    'b3: {
        v0_2 = lw(sp + 4);
        if lw(sp) != lw(sp + 1392) {
            'b4: {
                'b5: {
                    s0_1 = 1;
                    s6_1 = lw(a6);
                    if v0_2 == 0 {
                        a1_2 = lw(sp + 64);
                        'l6: loop {
                            s0_1 = s0_1 + 1;
                            v0_4 = s0_1 << 2;
                            if (s0_1 as u32) >= (17 as u32) {
                                break 'b5;
                            }
                            s7_1 = s0_1;
                            if lw(sp + v0_4) != 0 {
                                break;
                            }
                        }
                        s1_2 = 16;
                        break 'b4;
                    }
                    a1_2 = lw(sp + 64);
                }
                s7_1 = s0_1;
                s1_2 = 16;
            }
            'b8: {
                s6_2 = if (s6_1 as u32) < (s7_1 as u32) { s7_1 } else { s6_1 };
                if a1_2 == 0 {
                    v1_1 = sp + 64;
                    s1_2 = s1_2 - 1;
                    'l9: loop {
                        v1_1 = v1_1 - 4;
                        if s1_2 == 0 {
                            break 'b8;
                        }
                        if lw(v1_1) != 0 {
                            break 'b8;
                        }
                        s1_2 = s1_2 - 1;
                    }
                }
            }
            'b10: {
                'b11: {
                    t6_1 = s1_2 as u128;
                    t4_1 = 1 << (s0_1 & 31);
                    s6_3 = if (t6_1 as i32 as u32) < (s6_2 as u32) { t6_1 } else { s6_2 as u128 };
                    sw(a6, s6_3 as i32);
                    if (s0_1 as u32) < (t6_1 as i32 as u32) {
                        v1_2 = (s0_1 << 2) + sp;
                        'l12: loop {
                            t4_2 = t4_1 - lw(v1_2);
                            s0_1 = s0_1 + 1;
                            if t4_2 < 0 {
                                break 'b10;
                            }
                            v1_2 = v1_2 + 4;
                            t4_1 = t4_2 << 1;
                            if (s0_1 as u32) >= (s1_2 as u32) {
                                break 'b11;
                            }
                        }
                    }
                }
                v1_3 = sp + (s1_2 << 2);
                v0_5 = lw(v1_3);
                t4_3 = (t4_1 - v0_5) as u128;
                v0_6 = v0_5 + t4_3 as i32;
                if (t4_3 as i64) >= 0_i64 {
                    s0_2 = 0;
                    sw(v1_3, v0_6);
                    s4_2 = sp + 4;
                    sw(sp + 1316, 0);
                    s1_3 = s1_2 - 1;
                    a1_3 = sp + 1320;
                    if s1_3 == 0 {
                        a2_2 = 0 - s6_3 as i32;
                        sw(sp + 1416, sp + 160);
                        a3_1 = ((t6_1 as i64) < s7_1 as i64) as i32;
                        a1_4 = lw(sp + 1416);
                    } else {
                        a2_2 = 0 - s6_3 as i32;
                        sw(sp + 1416, sp + 160);
                        a3_1 = ((t6_1 as i64) < s7_1 as i64) as i32;
                        'l14: loop {
                            v0_7 = lw(s4_2);
                            s1_3 = s1_3 - 1;
                            s4_2 = s4_2 + 4;
                            s0_2 = s0_2 + v0_7;
                            sw(a1_3, s0_2);
                            a1_3 = a1_3 + 4;
                            if s1_3 == 0 {
                                break;
                            }
                        }
                        a1_4 = lw(sp + 1416);
                    }
                    s4_3 = a0;
                    s1_4 = 0;
                    s0_3 = lw(s4_3);
                    'l16: loop {
                        s4_3 = s4_3 + 4;
                        if s0_3 != 0 {
                            a0_4 = sp + 1312 + (s0_3 << 2);
                            v1_4 = lw(a0_4);
                            sw(a1_4 + (v1_4 << 2), s1_4);
                            sw(a0_4, v1_4 + 1);
                        }
                        s1_4 = s1_4 + 1;
                        if (s1_4 as u32) >= (lw(sp + 1392) as u32) {
                            break;
                        }
                        s0_3 = lw(s4_3);
                    }
                    'b19: {
                        s3_1 = a2_2;
                        s1_5 = 0;
                        sw(sp + 1312, 0);
                        a7_1 = 340282366920938463463374607431768211455_u128;
                        s4_4 = lw(sp + 1416);
                        a5_1 = 0;
                        sw(sp + 96, 0);
                        a6_1 = 0_u128;
                        if a3_1 == 0 {
                            sw(sp + 1568, -0x10000);
                            sw(sp + 1568, -1);
                            t7_1 = (s7_1 << 2) as u128;
                            'l20: loop {
                                'b21: {
                                    s5_1 = lw(sp + t7_1 as i32) - 1;
                                    v1_5 = lw(sp + 1392);
                                    if s5_1 != lw(sp + 1568) {
                                        t5_1 = 1_u128;
                                        fp_1 = (a7_1 as i32) << 2;
                                        sw(sp + 1412, lw(sp + 1416) + (v1_5 << 2));
                                        s2_1 = s3_1 + s6_3 as i32;
                                        'l22: loop {
                                            a4_1 = (s5_1 + 1) as u128;
                                            if s2_1 >= s7_1 {
                                                a2_4 = (s7_1 - s3_1) as u128;
                                                a3_2 = (a7_1 as i32) << 2;
                                                a0_9 = lw(sp + 1412);
                                            } else {
                                                s3_1 = s2_1;
                                                'l25: loop {
                                                    'b27: {
                                                        fp_1 = fp_1 + 4;
                                                        a6_2 = t6_1 as i32 - s3_1;
                                                        s0_4 = s7_1 - s3_1;
                                                        a0_5 = (t5_1 as i32) << (s0_4 & 31);
                                                        a7_2 = a7_1 as i32 + 1;
                                                        a2_3 = s0_4;
                                                        if (a4_1 as i32 as u32) < (a0_5 as u32) {
                                                            s0_4 = s0_4 + 1;
                                                            a0_6 = a0_5 - 1 - s5_1;
                                                            a1_5 = sp + t7_1 as i32;
                                                            'l28: loop {
                                                                a1_5 = a1_5 + 4;
                                                                if (s0_4 as u32) >= ((if (s6_3 as i32 as u32) < (a6_2 as u32) { s6_3 } else { a6_2 as u128 }) as i32 as u32) {
                                                                    break 'b27;
                                                                }
                                                                a0_7 = a0_6 << 1;
                                                                v1_6 = lw(a1_5);
                                                                if (v1_6 as u32) >= (a0_7 as u32) {
                                                                    break 'b27;
                                                                }
                                                                s0_4 = s0_4 + 1;
                                                                a0_6 = a0_7 - v1_6;
                                                            }
                                                        }
                                                    }
                                                    a6_3 = (t5_1 as i32) << (s0_4 & 31);
                                                    sq(sp + 1424, a2_3 as u128);
                                                    sq(sp + 1440, a4_1);
                                                    a0_8 = (a6_3 + 1) << 3;
                                                    sq(sp + 1456, a6_3 as u128);
                                                    sq(sp + 1472, a7_2 as u128);
                                                    sq(sp + 1488, t4_3);
                                                    sq(sp + 1504, t5_1);
                                                    sq(sp + 1520, t6_1);
                                                    sq(sp + 1536, t7_1);
                                                    v0_8 = AllocMem(a0_8);
                                                    a2_4 = lq(sp + 1424);
                                                    a4_1 = lq(sp + 1440);
                                                    a6_1 = lq(sp + 1456);
                                                    a7_1 = lq(sp + 1472);
                                                    t4_3 = lq(sp + 1488);
                                                    t5_1 = lq(sp + 1504);
                                                    t6_1 = lq(sp + 1520);
                                                    t7_1 = lq(sp + 1536);
                                                    if v0_8 == 0 {
                                                        break 'l22;
                                                    }
                                                    v1_7 = v0_8 + 8;
                                                    a3_2 = fp_1;
                                                    sw(lw(sp + 1408), v1_7);
                                                    sw(v0_8 + 4, 0);
                                                    sw(sp + 1408, v0_8 + 4);
                                                    a5_1 = v1_7;
                                                    sw(sp + 96 + a3_2, a5_1);
                                                    if (a7_1 as i64) != 0_i64 {
                                                        sw(sp + a3_2 + 1312, s1_5);
                                                        sb(sp + 80, s0_4 + 16);
                                                        sb(sp + 81, s6_3 as i32);
                                                        sw(sp + 84, a5_1);
                                                        sd(((((s1_5 as u32) >> (s2_1 - s6_3 as i32 & 31)) as i32) << 3) + lw(sp + 96 + ((a7_1 as i32 - 1) << 2)), ld(sp + 80));
                                                    }
                                                    s2_1 = s2_1 + s6_3 as i32;
                                                    if s2_1 >= s7_1 {
                                                        break;
                                                    }
                                                    s3_1 = s2_1;
                                                }
                                                a0_9 = lw(sp + 1412);
                                            }
                                            sb(sp + 81, a2_4 as i32);
                                            if (s4_4 as u32) < (a0_9 as u32) {
                                                v1_8 = lw(s4_4);
                                                if (v1_8 as u32) >= (lw(sp + 1396) as u32) {
                                                    s0_5 = lw(sp + 1396);
                                                    t8_1 = lw(sp + 1400);
                                                    sb(sp + 80, lbu(((v1_8 - s0_5) << 1) + lw(sp + 1404)));
                                                    v0_10 = lw(s4_4);
                                                    s4_4 = s4_4 + 4;
                                                    sh(sp + 84, lhu(((v0_10 - s0_5) << 1) + t8_1));
                                                } else {
                                                    v1_9 = 15;
                                                    if (v1_8 as u32) < (256 as u32) {
                                                        v1_9 = 16;
                                                    }
                                                    sb(sp + 80, v1_9);
                                                    v0_9 = lhu(s4_4);
                                                    s4_4 = s4_4 + 4;
                                                    sh(sp + 84, v0_9);
                                                }
                                            } else {
                                                sb(sp + 80, 99);
                                            }
                                            s0_6 = ((s1_5 as u32) >> (s3_1 & 31)) as i32;
                                            a0_10 = (t5_1 as i32) << (a2_4 & 31_u128) as i32;
                                            if (s0_6 as u32) >= (a6_1 as i32 as u32) {
                                                a4_2 = s5_1 - 1;
                                                s0_7 = (t5_1 as i32) << (s7_1 - 1 & 31);
                                            } else {
                                                a4_2 = s5_1 - 1;
                                                a1_6 = s7_1 - 1;
                                                'l33: loop {
                                                    v0_11 = (s0_6 << 3) + a5_1;
                                                    s0_6 = s0_6 + a0_10;
                                                    sd(v0_11, ld(sp + 80));
                                                    if (s0_6 as u32) >= (a6_1 as i32 as u32) {
                                                        break;
                                                    }
                                                }
                                                s0_7 = (t5_1 as i32) << (a1_6 & 31);
                                            }
                                            v0_12 = (t5_1 as i32) << (s3_1 & 31);
                                            if (s1_5 & s0_7) != 0 {
                                                'l36: loop {
                                                    s1_5 = s1_5 ^ s0_7;
                                                    s0_7 = ((s0_7 as u32) >> 1) as i32;
                                                    if (s1_5 & s0_7) == 0 {
                                                        break;
                                                    }
                                                }
                                                v0_12 = (t5_1 as i32) << (s3_1 & 31);
                                            }
                                            s1_5 = s1_5 ^ s0_7;
                                            t9_1 = lw(sp + 1568);
                                            if (s1_5 & v0_12 - 1) != lw(sp + 1312 + fp_1) {
                                                a0_11 = a3_2 + (sp + 1312);
                                                'l39: loop {
                                                    s3_1 = s3_1 - s6_3 as i32;
                                                    a0_11 = a0_11 - 4;
                                                    fp_1 = fp_1 - 4;
                                                    a7_1 = (a7_1 as i32 - 1) as u128;
                                                    if (s1_5 & (1 << (s3_1 & 31)) - 1) == lw(a0_11) {
                                                        break;
                                                    }
                                                }
                                                t9_1 = lw(sp + 1568);
                                            }
                                            s5_1 = a4_2;
                                            s2_1 = s3_1 + s6_3 as i32;
                                            if s5_1 == t9_1 {
                                                break 'b21;
                                            }
                                        }
                                        v0_3 = 3;
                                        break 'b3;
                                    }
                                }
                                s7_1 = s7_1 + 1;
                                if (t6_1 as i64) < s7_1 as i64 {
                                    break 'b19;
                                }
                                t7_1 = (s7_1 << 2) as u128;
                            }
                        }
                    }
                    v0_3 = 0;
                    if (t4_3 as i64) == 0_i64 {
                        break 'b3;
                    }
                    v0_3 = ((0 as u32) < ((t6_1 ^ 1_u128) as i32 as u32)) as i32;
                    break 'b3;
                }
            }
            v0_3 = 2;
        } else {
            a0_3 = lw(sp + 1408);
            v0_3 = 0;
            sw(a6, 0);
            sw(a0_3, 0);
        }
    }
    return v0_3;
}
