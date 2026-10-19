//! Merging the polygons of two models that share an edge.

use crate::*;

/// Merge the polygons of the models `second` (`a2`) and `third` (`a3`) into the list of polygons of `first` (`a1`), through the output record `out` (`a0`: the number of polygons
/// written at +24 and of indices at +28): a polygon of `a2` and one of `a3` with almost the same normal (their dot product at least the game's constant) that share an edge
/// (the same vertex positions, in the opposite order, from the vertices at `a4`), and whose join stays convex, are made one polygon (both are then cleared), and the others
/// are copied; returns 1 if any were joined.
///
/// This is the machine translation with the stack scratch turned into named locals (`slot_N` is the word at N bytes into the original's frame); the control flow and the
/// names of the original's registers are kept as they are, to be tidied when the structures it works on are typed.
pub fn obMergePolys(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32) -> i32 {
    let mut slot_0: f32 = 0.0;
    let mut slot_4: f32 = 0.0;
    let mut slot_8: f32 = 0.0;
    let mut slot_16: f32 = 0.0;
    let mut slot_20: f32 = 0.0;
    let mut slot_24: f32 = 0.0;
    let mut slot_32: i32 = 0;
    let mut slot_36: i32 = 0;
    let mut slot_48: i32 = 0;
    let mut slot_52: i32 = 0;
    let mut slot_64: f32 = 0.0;
    let mut slot_68: f32 = 0.0;
    let mut slot_72: f32 = 0.0;
    let mut slot_80: f32 = 0.0;
    let mut slot_84: f32 = 0.0;
    let mut slot_88: f32 = 0.0;
    let mut slot_96: f32 = 0.0;
    let mut slot_100: f32 = 0.0;
    let mut slot_104: f32 = 0.0;
    let mut slot_112: i32 = 0;
    let mut slot_116: i32 = 0;
    let mut slot_128: i32 = 0;
    let mut slot_132: i32 = 0;
    let mut slot_136: i32 = 0;
    let mut slot_140: i32 = 0;
    let mut slot_144: i32 = 0;
    let mut slot_148: i32 = 0;
    let mut slot_152: i32 = 0;
    let mut slot_156: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut f23_1: f32 = 0.0;
    let mut a6_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut f4_1: f32 = 0.0;
    let mut f1_1: f32 = 0.0;
    let mut f2_1: f32 = 0.0;
    let mut f3_1: f32 = 0.0;
    let mut f0_1: f32 = 0.0;
    let mut f0_2: f32 = 0.0;
    let mut f0_3: f32 = 0.0;
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a6_3: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut f5_1: f32 = 0.0;
    let mut f4_2: f32 = 0.0;
    let mut f3_2: f32 = 0.0;
    let mut v1_4: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v1_5: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut f2_2: f32 = 0.0;
    let mut f1_2: f32 = 0.0;
    let mut a3_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut t5_1: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a5_3: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut a7_2: i32 = 0;
    let mut t4_1: i32 = 0;
    let mut a6_4: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut t8_1: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut s7_1: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut s6_1: i32 = 0;
    let mut f0_4: f32 = 0.0;
    let mut f1_3: f32 = 0.0;
    let mut f2_3: f32 = 0.0;
    let mut f3_3: f32 = 0.0;
    let mut f6_1: f32 = 0.0;
    let mut f0_5: f32 = 0.0;
    let mut f1_4: f32 = 0.0;
    let mut f9_1: f32 = 0.0;
    let mut f8_1: f32 = 0.0;
    let mut f7_1: f32 = 0.0;
    let mut f1_5: f32 = 0.0;
    let mut f4_3: f32 = 0.0;
    let mut f3_4: f32 = 0.0;
    let mut f0_6: f32 = 0.0;
    let mut f10_1: f32 = 0.0;
    let mut f11_1: f32 = 0.0;
    let mut s4_1: i32 = 0;
    let mut f4_4: f32 = 0.0;
    let mut f22_1: f32 = 0.0;
    let mut f6_2: f32 = 0.0;
    let mut f5_2: f32 = 0.0;
    let mut f7_2: f32 = 0.0;
    let mut f1_6: f32 = 0.0;
    let mut f0_7: f32 = 0.0;
    let mut f2_4: f32 = 0.0;
    let mut f3_5: f32 = 0.0;
    let mut f5_3: f32 = 0.0;
    let mut f6_3: f32 = 0.0;
    let mut f1_7: f32 = 0.0;
    let mut f2_5: f32 = 0.0;
    let mut f4_5: f32 = 0.0;
    let mut f0_8: f32 = 0.0;
    let mut v1_8: i32 = 0;
    let mut f0_9: f32 = 0.0;
    let mut f1_8: f32 = 0.0;
    let mut f2_6: f32 = 0.0;
    let mut f3_6: f32 = 0.0;
    let mut f6_4: f32 = 0.0;
    let mut f0_10: f32 = 0.0;
    let mut f1_9: f32 = 0.0;
    let mut f9_2: f32 = 0.0;
    let mut f4_6: f32 = 0.0;
    let mut f8_2: f32 = 0.0;
    let mut f7_3: f32 = 0.0;
    let mut f3_7: f32 = 0.0;
    let mut f2_7: f32 = 0.0;
    let mut f0_11: f32 = 0.0;
    let mut f10_2: f32 = 0.0;
    let mut f11_2: f32 = 0.0;
    let mut s4_2: i32 = 0;
    let mut f4_7: f32 = 0.0;
    let mut f6_5: f32 = 0.0;
    let mut f5_4: f32 = 0.0;
    let mut f7_4: f32 = 0.0;
    let mut f1_10: f32 = 0.0;
    let mut f0_12: f32 = 0.0;
    let mut f2_8: f32 = 0.0;
    let mut f3_8: f32 = 0.0;
    let mut f5_5: f32 = 0.0;
    let mut f6_6: f32 = 0.0;
    let mut f1_11: f32 = 0.0;
    let mut f2_9: f32 = 0.0;
    let mut f4_8: f32 = 0.0;
    let mut f0_13: f32 = 0.0;
    let mut a0_6: i32 = 0;
    let mut a6_5: i32 = 0;
    let mut a5_4: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut bc_1: bool = false;
    let mut v0_16: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut bc_2: bool = false;
    let mut v0_19: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut a6_6: i32 = 0;
    let mut v1_15: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a5_5: i32 = 0;
    let mut a3_8: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut lo_1: i32 = 0;
    let mut a6_7: i32 = 0;
    let mut lo_2: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut lo_3: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut v1_16: i32 = 0;
    let mut v1_17: i32 = 0;
    let mut v0_23: i32 = 0;
    'b1: {
        slot_132 = a2;
        sw(a0 + 24, 0);
        slot_128 = a1;
        sw(a0 + 28, 0);
        slot_136 = a3;
        slot_140 = 0;
        a1_1 = lw(slot_132 + 28);
        slot_144 = 0;
        if a1_1 > 0 {
            f23_1 = lwc1(0x3acb48);
            'l2: loop {
                a6_1 = slot_136;
                if lbu(slot_140 * 12 + lw(slot_132 + 32)) == 0 {
                    slot_156 = slot_140 + 1;
                } else {
                    'b4: {
                        'b5: {
                            a7_1 = lw(a6_1 + 28);
                            slot_148 = 0;
                            slot_152 = 0;
                            if a7_1 > 0 {
                                slot_156 = slot_140 + 1;
                                'b8: {
                                    'l6: loop {
                                        'b9: {
                                            a6_2 = slot_148;
                                            s2_1 = a6_2 * 12 + lw(slot_136 + 32);
                                            v0_1 = slot_140;
                                            if lbu(s2_1) != 0 {
                                                s0_1 = lw(slot_132 + 32) + v0_1 * 12;
                                                if s0_1 == s2_1 {
                                                    a6_6 = a6_2 + 1;
                                                    break 'b9;
                                                }
                                                'b11: {
                                                    f4_1 = lwc1(0x3acb4c);
                                                    f1_1 = lh(s0_1 + 2) as f32 / f23_1;
                                                    slot_0 = f1_1;
                                                    f2_1 = lh(s0_1 + 4) as f32 / f23_1;
                                                    slot_4 = f2_1;
                                                    f3_1 = lh(s0_1 + 6) as f32 / f23_1;
                                                    slot_8 = f3_1;
                                                    f0_1 = lh(s2_1 + 2) as f32 / f23_1;
                                                    slot_16 = f0_1;
                                                    f0_2 = lh(s2_1 + 4) as f32 / f23_1;
                                                    slot_20 = f0_2;
                                                    f0_3 = lh(s2_1 + 6) as f32 / f23_1;
                                                    slot_24 = f0_3;
                                                    if f4_1 <= f1_1 * f0_1 + f2_1 * f0_2 + f3_1 * f0_3 {
                                                        'b12: {
                                                            slot_52 = -1;
                                                            a2_1 = 0;
                                                            slot_48 = -1;
                                                            slot_36 = -1;
                                                            slot_32 = -1;
                                                            a0_1 = lbu(s0_1);
                                                            a1_2 = a0_1;
                                                            if a0_1 == 0 {
                                                                a4_1 = -1;
                                                            } else {
                                                                a4_1 = -1;
                                                                'l13: loop {
                                                                    v1_1 = a4_1 + 1;
                                                                    a4_1 = slot_32;
                                                                    v1_2 = if v1_1 >= a1_2 { 0 } else { v1_1 };
                                                                    v0_2 = a4_1 - 1;
                                                                    a6_3 = a4_1;
                                                                    if v0_2 < 0 {
                                                                        v0_2 = a0_1 - 1;
                                                                    }
                                                                    'b15: {
                                                                        if a4_1 < 0 {
                                                                            v1_3 = lw(s0_1 + 8);
                                                                        } else if a2_1 == v1_2 {
                                                                            v1_3 = lw(s0_1 + 8);
                                                                        } else {
                                                                            if a2_1 != v0_2 {
                                                                                a2_1 = a2_1 + 1;
                                                                                break 'b15;
                                                                            }
                                                                            v1_3 = lw(s0_1 + 8);
                                                                        }
                                                                        'b17: {
                                                                            a1_3 = 0;
                                                                            a0_2 = lbu(s2_1);
                                                                            a5_1 = a0_2;
                                                                            v0_3 = lbu(v1_3 + a2_1) * 12 + a4;
                                                                            f5_1 = lwc1(v0_3 + 8);
                                                                            f4_2 = lwc1(v0_3);
                                                                            f3_2 = lwc1(v0_3 + 4);
                                                                            if a0_2 > 0 {
                                                                                v1_4 = slot_36 + 1;
                                                                                'l18: loop {
                                                                                    a3_1 = slot_36;
                                                                                    v1_5 = if v1_4 >= a0_2 { 0 } else { v1_4 };
                                                                                    v0_4 = a3_1 - 1;
                                                                                    if v0_4 < 0 {
                                                                                        v0_4 = a5_1 - 1;
                                                                                    }
                                                                                    'b21: {
                                                                                        if a6_3 < 0 {
                                                                                            v0_5 = lw(s2_1 + 8);
                                                                                        } else if a1_3 == v1_5 {
                                                                                            v0_5 = lw(s2_1 + 8);
                                                                                        } else {
                                                                                            if a1_3 != v0_4 {
                                                                                                a1_3 = a1_3 + 1;
                                                                                                break 'b21;
                                                                                            }
                                                                                            v0_5 = lw(s2_1 + 8);
                                                                                        }
                                                                                        v1_6 = lbu(v0_5 + a1_3) * 12 + a4;
                                                                                        f2_2 = lwc1(v1_6 + 8);
                                                                                        f1_2 = lwc1(v1_6 + 4);
                                                                                        if !(f4_2 == lwc1(v1_6)) {
                                                                                            a1_3 = a1_3 + 1;
                                                                                        } else if !(f3_2 == f1_2) {
                                                                                            a1_3 = a1_3 + 1;
                                                                                        } else {
                                                                                            if f5_1 == f2_2 {
                                                                                                break 'l18;
                                                                                            }
                                                                                            a1_3 = a1_3 + 1;
                                                                                        }
                                                                                    }
                                                                                    a0_2 = a5_1;
                                                                                    v1_4 = a3_1 + 1;
                                                                                    if a1_3 >= a0_2 {
                                                                                        break 'b17;
                                                                                    }
                                                                                }
                                                                                if a6_3 >= 0 {
                                                                                    slot_52 = a1_3;
                                                                                    slot_48 = a2_1;
                                                                                } else {
                                                                                    slot_36 = a1_3;
                                                                                    a4_1 = a2_1;
                                                                                    slot_32 = a2_1;
                                                                                }
                                                                            }
                                                                        }
                                                                        a2_1 = a2_1 + 1;
                                                                        if slot_48 >= 0 {
                                                                            break 'b12;
                                                                        }
                                                                        a0_1 = lbu(s0_1);
                                                                    }
                                                                    a1_2 = a0_1;
                                                                    if a2_1 >= a0_1 {
                                                                        break 'b12;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        a3_2 = slot_48;
                                                        if a4_1 >= 0 {
                                                            'b23: {
                                                                a0_3 = a4_1 + 1;
                                                                if a3_2 >= 0 {
                                                                    a5_2 = slot_36;
                                                                    a2_2 = a5_2 - 1;
                                                                    a1_4 = if (a3_2 ^ (if a0_3 < lbu(s0_1) { a0_3 } else { 0 })) == 0 { 0 } else { 1 };
                                                                    if a2_2 < 0 {
                                                                        a2_2 = lbu(s2_1) - 1;
                                                                    }
                                                                    v1_7 = slot_52;
                                                                    a0_4 = if (v1_7 ^ a2_2) == 0 { 0 } else { 1 };
                                                                    if a1_4 == 0 {
                                                                        if a0_4 == 0 {
                                                                            break 'b23;
                                                                        }
                                                                        slot_52 = -1;
                                                                    } else {
                                                                        if a0_4 != 0 {
                                                                            slot_48 = a4_1;
                                                                            slot_36 = v1_7;
                                                                            a4_1 = a3_2;
                                                                            slot_52 = a5_2;
                                                                            slot_32 = a3_2;
                                                                            break 'b23;
                                                                        }
                                                                        slot_52 = -1;
                                                                    }
                                                                    a4_1 = -1;
                                                                    slot_48 = -1;
                                                                    slot_36 = -1;
                                                                    slot_32 = -1;
                                                                }
                                                            }
                                                            t5_1 = slot_48;
                                                            if a4_1 >= 0 {
                                                                a0_5 = slot_52;
                                                                if t5_1 >= 0 {
                                                                    a2_3 = a4_1 + 1;
                                                                    a5_3 = lw(s0_1 + 8);
                                                                    a3_3 = a4_1 - 1;
                                                                    a7_2 = lw(s2_1 + 8);
                                                                    t4_1 = lbu(s0_1);
                                                                    a6_4 = lbu(a5_3 + a4_1) * 12 + a4;
                                                                    s1_1 = lbu(a7_2 + a0_5) * 12 + a4;
                                                                    if a3_3 < 0 {
                                                                        a3_3 = t4_1 - 1;
                                                                    }
                                                                    v0_6 = a5_3 + (if a2_3 < t4_1 { a2_3 } else { 0 });
                                                                    if a3_3 != t5_1 {
                                                                        v0_6 = a5_3 + a3_3;
                                                                    }
                                                                    t8_1 = lbu(v0_6) * 12 + a4;
                                                                    v0_7 = slot_36;
                                                                    a1_5 = lbu(s2_1);
                                                                    v0_8 = v0_7 + 1;
                                                                    a3_4 = v0_7 - 1;
                                                                    if a3_4 < 0 {
                                                                        a3_4 = a1_5 - 1;
                                                                    }
                                                                    v0_9 = a7_2 + (if v0_8 >= a1_5 { 0 } else { v0_8 });
                                                                    if a3_4 != a0_5 {
                                                                        v0_9 = a7_2 + a3_4;
                                                                    }
                                                                    a2_4 = lbu(v0_9) * 12 + a4;
                                                                    v0_10 = t5_1 + 1;
                                                                    a3_5 = slot_48 - 1;
                                                                    if a3_5 < 0 {
                                                                        a3_5 = t4_1 - 1;
                                                                    }
                                                                    if a3_5 != a4_1 {
                                                                        v0_11 = a5_3 + a3_5;
                                                                    } else {
                                                                        v0_11 = a5_3 + (if v0_10 >= t4_1 { 0 } else { v0_10 });
                                                                    }
                                                                    s7_1 = lbu(v0_11) * 12 + a4;
                                                                    v0_12 = a0_5 + 1;
                                                                    a3_6 = slot_52 - 1;
                                                                    if a3_6 < 0 {
                                                                        a3_6 = a1_5 - 1;
                                                                    }
                                                                    if a3_6 != v0_7 {
                                                                        v0_13 = a7_2 + a3_6;
                                                                    } else {
                                                                        v0_13 = a7_2 + (if v0_12 >= a1_5 { 0 } else { v0_12 });
                                                                    }
                                                                    s6_1 = lbu(v0_13) * 12 + a4;
                                                                    f0_4 = lwc1(a6_4);
                                                                    f1_3 = lwc1(a6_4 + 4);
                                                                    f2_3 = lwc1(a2_4) - f0_4;
                                                                    f3_3 = lwc1(a6_4 + 8);
                                                                    slot_64 = f2_3;
                                                                    f6_1 = lwc1(a6_4 + 8);
                                                                    f0_5 = lwc1(a2_4 + 4) - f1_3;
                                                                    slot_68 = f0_5;
                                                                    f1_4 = lwc1(a2_4 + 8) - f3_3;
                                                                    slot_72 = f1_4;
                                                                    slot_80 = lwc1(t8_1) - f0_4;
                                                                    slot_84 = lwc1(t8_1 + 4) - f1_3;
                                                                    slot_88 = lwc1(t8_1 + 8) - f6_1;
                                                                    f9_1 = slot_80;
                                                                    f8_1 = slot_84;
                                                                    f7_1 = slot_88;
                                                                    f1_5 = 1.0_f32 / (f2_3 * f2_3 + f0_5 * f0_5 + f1_4 * f1_4).abs().sqrt();
                                                                    f4_3 = slot_64 * f1_5;
                                                                    f3_4 = slot_68 * f1_5;
                                                                    slot_72 = slot_72 * f1_5;
                                                                    slot_64 = f4_3;
                                                                    slot_68 = f3_4;
                                                                    f0_6 = 1.0_f32 / (f9_1 * f9_1 + f8_1 * f8_1 + f7_1 * f7_1).abs().sqrt();
                                                                    f10_1 = slot_64;
                                                                    f11_1 = slot_68;
                                                                    s4_1 = 1;
                                                                    f4_4 = slot_72;
                                                                    f22_1 = lwc1(0x3acb50);
                                                                    f6_2 = f7_1 * f0_6;
                                                                    f5_2 = f8_1 * f0_6;
                                                                    f7_2 = f9_1 * f0_6;
                                                                    slot_88 = f6_2;
                                                                    slot_84 = f5_2;
                                                                    slot_80 = f7_2;
                                                                    if !((f10_1 * f7_2 + f11_1 * f5_2 + f4_4 * f6_2) <= f22_1) {
                                                                        s4_1 = 0;
                                                                    }
                                                                    f1_6 = f4_4 * f7_2 - f10_1 * f6_2;
                                                                    f0_7 = f11_1 * f6_2 - f4_4 * f5_2;
                                                                    slot_100 = f1_6;
                                                                    slot_96 = f0_7;
                                                                    f2_4 = f10_1 * f5_2 - f11_1 * f7_2;
                                                                    slot_104 = f2_4;
                                                                    f3_5 = 1.0_f32 / (f0_7 * f0_7 + f1_6 * f1_6 + f2_4 * f2_4).abs().sqrt();
                                                                    f5_3 = slot_4;
                                                                    f6_3 = slot_8;
                                                                    f1_7 = slot_96 * f3_5;
                                                                    f2_5 = slot_100 * f3_5;
                                                                    f4_5 = slot_104 * f3_5;
                                                                    f0_8 = slot_0 * f1_7;
                                                                    slot_96 = f1_7;
                                                                    slot_100 = f2_5;
                                                                    slot_104 = f4_5;
                                                                    if !(0.0_f32 <= f0_8 + f5_3 * f2_5 + f6_3 * f4_5) {
                                                                        v1_8 = slot_136;
                                                                        if s4_1 == 0 {
                                                                            a7_1 = lw(v1_8 + 28);
                                                                            break 'b11;
                                                                        }
                                                                    }
                                                                    f0_9 = lwc1(s1_1);
                                                                    f1_8 = lwc1(s1_1 + 4);
                                                                    f2_6 = lwc1(s7_1) - f0_9;
                                                                    f3_6 = lwc1(s1_1 + 8);
                                                                    slot_64 = f2_6;
                                                                    f6_4 = lwc1(s1_1 + 8);
                                                                    f0_10 = lwc1(s7_1 + 4) - f1_8;
                                                                    slot_112 = s4_1;
                                                                    slot_68 = f0_10;
                                                                    f1_9 = lwc1(s7_1 + 8) - f3_6;
                                                                    slot_72 = f1_9;
                                                                    slot_80 = lwc1(s6_1) - f0_9;
                                                                    slot_84 = lwc1(s6_1 + 4) - f1_8;
                                                                    slot_88 = lwc1(s6_1 + 8) - f6_4;
                                                                    f9_2 = slot_80;
                                                                    f4_6 = 1.0_f32 / (f2_6 * f2_6 + f0_10 * f0_10 + f1_9 * f1_9).abs().sqrt();
                                                                    f8_2 = slot_84;
                                                                    f7_3 = slot_88;
                                                                    f3_7 = slot_68;
                                                                    f2_7 = slot_64 * f4_6;
                                                                    slot_72 = slot_72 * f4_6;
                                                                    slot_64 = f2_7;
                                                                    slot_68 = f3_7 * f4_6;
                                                                    f0_11 = 1.0_f32 / (f9_2 * f9_2 + f8_2 * f8_2 + f7_3 * f7_3).abs().sqrt();
                                                                    f10_2 = slot_64;
                                                                    f11_2 = slot_68;
                                                                    s4_2 = 1;
                                                                    f4_7 = slot_72;
                                                                    f6_5 = f7_3 * f0_11;
                                                                    f5_4 = f8_2 * f0_11;
                                                                    f7_4 = f9_2 * f0_11;
                                                                    slot_88 = f6_5;
                                                                    slot_84 = f5_4;
                                                                    slot_80 = f7_4;
                                                                    if !((f10_2 * f7_4 + f11_2 * f5_4 + f4_7 * f6_5) <= f22_1) {
                                                                        s4_2 = 0;
                                                                    }
                                                                    f1_10 = f4_7 * f7_4 - f10_2 * f6_5;
                                                                    f0_12 = f11_2 * f6_5 - f4_7 * f5_4;
                                                                    slot_100 = f1_10;
                                                                    slot_96 = f0_12;
                                                                    f2_8 = f10_2 * f5_4 - f11_2 * f7_4;
                                                                    slot_104 = f2_8;
                                                                    f3_8 = 1.0_f32 / (f0_12 * f0_12 + f1_10 * f1_10 + f2_8 * f2_8).abs().sqrt();
                                                                    f5_5 = slot_4;
                                                                    f6_6 = slot_8;
                                                                    f1_11 = slot_96 * f3_8;
                                                                    f2_9 = slot_100 * f3_8;
                                                                    f4_8 = slot_104 * f3_8;
                                                                    f0_13 = slot_0 * f1_11;
                                                                    slot_96 = f1_11;
                                                                    slot_100 = f2_9;
                                                                    slot_104 = f4_8;
                                                                    if 0.0_f32 <= f0_13 + f5_5 * f2_9 + f6_6 * f4_8 {
                                                                        break 'l6;
                                                                    }
                                                                    a0_6 = slot_136;
                                                                    if s4_2 != 0 {
                                                                        break 'l6;
                                                                    }
                                                                    a7_1 = lw(a0_6 + 28);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                a6_2 = slot_148;
                                            }
                                            a6_6 = a6_2 + 1;
                                        }
                                        slot_148 = a6_6;
                                        if a6_6 >= a7_1 {
                                            break 'b8;
                                        }
                                    }
                                    'b37: {
                                        a6_5 = 0;
                                        a5_4 = lw(a0 + 24);
                                        a3_7 = lw(slot_128 + 32);
                                        a2_5 = slot_48;
                                        v0_14 = lw(a3_7 + 8) + lw(a0 + 28);
                                        a4_2 = slot_32;
                                        slot_116 = s4_2;
                                        sw(a5_4 * 12 + a3_7 + 8, v0_14);
                                        if a2_5 != a4_2 {
                                            v0_15 = slot_116;
                                            'l38: loop {
                                                'b39: {
                                                    bc_1 = v0_15 == 0;
                                                    v0_16 = slot_48;
                                                    if !bc_1 {
                                                        if a2_5 == v0_16 {
                                                            v0_17 = lbu(s0_1);
                                                            break 'b39;
                                                        }
                                                    }
                                                    v1_9 = lw(a5_4 * 12 + a3_7 + 8) + a6_5;
                                                    a6_5 = a6_5 + 1;
                                                    sb(v1_9, lbu(lw(s0_1 + 8) + a2_5));
                                                    a4_2 = slot_32;
                                                    a5_4 = lw(a0 + 24);
                                                    a3_7 = lw(slot_128 + 32);
                                                    v0_17 = lbu(s0_1);
                                                }
                                                v1_10 = a2_5 + 1;
                                                a2_5 = if v1_10 < v0_17 { v1_10 } else { 0 };
                                                v0_15 = slot_116;
                                                if a2_5 == a4_2 {
                                                    break 'b37;
                                                }
                                            }
                                        }
                                    }
                                    'b41: {
                                        a2_6 = slot_36;
                                        a0_7 = slot_52;
                                        if a2_6 != a0_7 {
                                            v0_18 = slot_112;
                                            'l42: loop {
                                                'b43: {
                                                    bc_2 = v0_18 == 0;
                                                    v0_19 = slot_36;
                                                    if !bc_2 {
                                                        if a2_6 == v0_19 {
                                                            v0_20 = lbu(s2_1);
                                                            break 'b43;
                                                        }
                                                    }
                                                    v1_11 = lw(a5_4 * 12 + a3_7 + 8) + a6_5;
                                                    a6_5 = a6_5 + 1;
                                                    sb(v1_11, lbu(lw(s2_1 + 8) + a2_6));
                                                    a0_7 = slot_52;
                                                    a5_4 = lw(a0 + 24);
                                                    a3_7 = lw(slot_128 + 32);
                                                    v0_20 = lbu(s2_1);
                                                }
                                                v1_12 = a2_6 + 1;
                                                a2_6 = if v1_12 < v0_20 { v1_12 } else { 0 };
                                                v0_18 = slot_112;
                                                if a2_6 == a0_7 {
                                                    break 'b41;
                                                }
                                            }
                                        }
                                    }
                                    v1_13 = lhu(s0_1 + 2);
                                    a0_8 = a5_4 * 12 + a3_7;
                                    slot_152 = 1;
                                    slot_144 = 1;
                                    sh(a0_8 + 2, v1_13);
                                    sh(a0_8 + 4, lhu(s0_1 + 4));
                                    v1_14 = lhu(s0_1 + 6);
                                    sb(a0_8, a6_5);
                                    sh(a0_8 + 6, v1_14);
                                    sw(a0 + 28, lw(a0 + 28) + a6_5);
                                    sb(s0_1, 0);
                                    sb(s2_1, 0);
                                    sw(a0 + 24, lw(a0 + 24) + 1);
                                    break 'b5;
                                }
                                v1_15 = slot_152;
                                break 'b4;
                            }
                            slot_156 = slot_140 + 1;
                        }
                        v1_15 = slot_152;
                    }
                    v0_21 = slot_132;
                    if v1_15 != 0 {
                        a1_1 = lw(v0_21 + 28);
                    } else {
                        a2_7 = 0;
                        a5_5 = lw(a0 + 24);
                        a3_8 = lw(slot_128 + 32);
                        a4_3 = lw(slot_132 + 32);
                        lo_1 = slot_140 * 12;
                        sw(a5_5 * 12 + a3_8 + 8, lw(a3_8 + 8) + lw(a0 + 28));
                        a6_7 = a4_3;
                        if lbu(lo_1 + a4_3) != 0 {
                            lo_2 = slot_140 * 12;
                            'l46: loop {
                                a0_9 = lbu(lw(lo_2 + a6_7 + 8) + a2_7);
                                v0_22 = lw(a5_5 * 12 + a3_8 + 8) + a2_7;
                                a2_7 = a2_7 + 1;
                                sb(v0_22, a0_9);
                                a4_3 = lw(slot_132 + 32);
                                a6_7 = a4_3;
                                if a2_7 >= lbu(lo_2 + a4_3) {
                                    break;
                                }
                                a5_5 = lw(a0 + 24);
                                a3_8 = lw(slot_128 + 32);
                            }
                            a5_5 = lw(a0 + 24);
                            a3_8 = lw(slot_128 + 32);
                        }
                        lo_3 = slot_140 * 12;
                        a1_6 = a5_5 * 12 + a3_8;
                        a0_10 = slot_140 * 12 + a4_3;
                        sh(a1_6 + 2, lhu(a0_10 + 2));
                        sh(a1_6 + 4, lhu(a0_10 + 4));
                        sh(a1_6 + 6, lhu(a0_10 + 6));
                        sb(a1_6, lbu(a0_10));
                        a1_7 = lw(a0 + 28);
                        v1_16 = lbu(lo_3 + lw(slot_132 + 32));
                        sw(a0 + 24, lw(a0 + 24) + 1);
                        sw(a0 + 28, a1_7 + v1_16);
                        a1_1 = lw(slot_132 + 28);
                    }
                }
                v1_17 = slot_156;
                slot_140 = v1_17;
                if v1_17 >= a1_1 {
                    break 'b1;
                }
            }
        }
    }
    v0_23 = slot_144;
    sw(slot_128 + 28, lw(a0 + 24));
    return v0_23;
}
