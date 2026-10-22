//! Functions: the tick of the player and the tick of the bullets in flight. These two are the machine translation (the structure and the register names of the original kept), to be tidied as the structures they work on are typed.

use crate::*;

/// Tick of the player (@ 0x281248): read the sticks and buttons, turn and move the player by the mode they are in (0 on the ground, 1 pushed, 2 falling, 3 swimming, 4 dead, 5 none, 6 going into a remote prop, 7 riding one), keep the camera vectors, the room, the sight line when zoomed, and the rumble.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn playerTick() {
    let frame = Frame::new(560);
    let sp = frame.sp();
    let mut v0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut fp_1: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut f14_1: f32 = 0.0;
    let mut f12_1: f32 = 0.0;
    let mut f13_1: f32 = 0.0;
    let mut f15_1: f32 = 0.0;
    let mut s3_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut f1_1: f32 = 0.0;
    let mut a3_1: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut f0_1: f32 = 0.0;
    let mut a0_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut f12_2: f32 = 0.0;
    let mut a0_4: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut f0_2: f32 = 0.0;
    let mut v0_7: i32 = 0;
    let mut f23_1: f32 = 0.0;
    let mut a1_4: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut s1_3: i32 = 0;
    let mut f2_1: f32 = 0.0;
    let mut f0_3: f32 = 0.0;
    let mut f21_1: f32 = 0.0;
    let mut f1_2: f32 = 0.0;
    let mut f20_1: f32 = 0.0;
    let mut s0_2: i32 = 0;
    let mut s0_3: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut f1_3: f32 = 0.0;
    let mut f2_2: f32 = 0.0;
    let mut f3_1: f32 = 0.0;
    let mut f4_1: f32 = 0.0;
    let mut f5_1: f32 = 0.0;
    let mut f6_1: f32 = 0.0;
    let mut f7_1: f32 = 0.0;
    let mut a0_8: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut v0_10: i32 = 0;
    let mut v0_11: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut v0_12: i32 = 0;
    let mut v0_13: i32 = 0;
    let mut f0_4: f32 = 0.0;
    let mut a0_11: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut f12_3: f32 = 0.0;
    let mut s2_2: i32 = 0;
    let mut v0_14: i32 = 0;
    let mut a0_12: i32 = 0;
    let mut v0_15: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut a0_13: i32 = 0;
    let mut v0_17: i32 = 0;
    let mut v0_18: i32 = 0;
    let mut bc_1: bool = false;
    let mut v0_19: i32 = 0;
    let mut f0_5: f32 = 0.0;
    let mut a0_14: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut s1_4: i32 = 0;
    let mut f1_4: f32 = 0.0;
    let mut a0_15: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut f1_5: f32 = 0.0;
    let mut f1_6: f32 = 0.0;
    let mut f0_6: f32 = 0.0;
    let mut a0_16: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut f12_4: f32 = 0.0;
    let mut f1_7: f32 = 0.0;
    let mut f2_3: f32 = 0.0;
    let mut f0_7: f32 = 0.0;
    let mut f1_8: f32 = 0.0;
    let mut f0_8: f32 = 0.0;
    let mut f0_9: f32 = 0.0;
    let mut v1_5: i32 = 0;
    let mut a0_17: i32 = 0;
    let mut v1_6: i32 = 0;
    let mut f12_5: f32 = 0.0;
    let mut a0_18: i32 = 0;
    let mut v0_22: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut f0_10: f32 = 0.0;
    let mut v0_23: i32 = 0;
    let mut f23_2: f32 = 0.0;
    let mut a1_8: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut f0_11: f32 = 0.0;
    let mut f12_6: f32 = 0.0;
    let mut a0_21: i32 = 0;
    let mut f23_3: f32 = 0.0;
    let mut f21_2: f32 = 0.0;
    let mut s1_5: i32 = 0;
    let mut f31_1: f32 = 0.0;
    let mut f0_12: f32 = 0.0;
    let mut f0_13: f32 = 0.0;
    let mut f22_1: f32 = 0.0;
    let mut f0_14: f32 = 0.0;
    let mut f0_15: f32 = 0.0;
    let mut a1_9: i32 = 0;
    let mut a0_22: i32 = 0;
    let mut f0_16: f32 = 0.0;
    let mut v0_24: i32 = 0;
    let mut a1_10: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut f0_17: f32 = 0.0;
    let mut f0_18: f32 = 0.0;
    let mut f0_19: f32 = 0.0;
    let mut s1_6: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut a1_11: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut f0_20: f32 = 0.0;
    let mut f1_9: f32 = 0.0;
    let mut f2_4: f32 = 0.0;
    let mut a0_25: i32 = 0;
    let mut v0_25: i32 = 0;
    let mut f20_2: f32 = 0.0;
    let mut s5_1: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut v0_26: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut s2_3: i32 = 0;
    let mut f24_1: f32 = 0.0;
    let mut s1_7: i32 = 0;
    let mut s0_4: i32 = 0;
    let mut v0_27: i32 = 0;
    let mut a0_27: i32 = 0;
    let mut f14_2: f32 = 0.0;
    let mut f12_7: f32 = 0.0;
    let mut f15_2: f32 = 0.0;
    let mut f13_2: f32 = 0.0;
    let mut a0_28: i32 = 0;
    let mut a1_12: i32 = 0;
    let mut v0_28: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut f2_5: f32 = 0.0;
    let mut v1_8: i32 = 0;
    let mut f0_21: f32 = 0.0;
    let mut f14_3: f32 = 0.0;
    let mut f12_8: f32 = 0.0;
    let mut f15_3: f32 = 0.0;
    let mut f13_3: f32 = 0.0;
    let mut a0_30: i32 = 0;
    let mut a1_13: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a1_14: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut f1_10: f32 = 0.0;
    let mut a0_31: i32 = 0;
    let mut f0_22: f32 = 0.0;
    let mut a0_32: i32 = 0;
    let mut f0_23: f32 = 0.0;
    let mut f20_3: f32 = 0.0;
    let mut a0_33: i32 = 0;
    let mut s0_5: i32 = 0;
    let mut v0_29: i32 = 0;
    let mut f14_4: f32 = 0.0;
    let mut f12_9: f32 = 0.0;
    let mut f15_4: f32 = 0.0;
    let mut f13_4: f32 = 0.0;
    let mut a0_34: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut v0_30: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a0_35: i32 = 0;
    let mut f0_24: f32 = 0.0;
    let mut v1_10: i32 = 0;
    let mut f1_11: f32 = 0.0;
    let mut f14_5: f32 = 0.0;
    let mut f12_10: f32 = 0.0;
    let mut f15_5: f32 = 0.0;
    let mut f13_5: f32 = 0.0;
    let mut a0_36: i32 = 0;
    let mut a1_16: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut a1_17: i32 = 0;
    let mut f1_12: f32 = 0.0;
    let mut v0_31: i32 = 0;
    let mut f0_25: f32 = 0.0;
    let mut a0_37: i32 = 0;
    let mut v0_32: i32 = 0;
    let mut f3_2: f32 = 0.0;
    let mut f2_6: f32 = 0.0;
    let mut f4_2: f32 = 0.0;
    let mut f1_13: f32 = 0.0;
    let mut v0_33: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut a0_38: i32 = 0;
    let mut f0_26: f32 = 0.0;
    let mut v0_34: i32 = 0;
    let mut f0_27: f32 = 0.0;
    let mut f2_7: f32 = 0.0;
    let mut f3_3: f32 = 0.0;
    let mut f4_3: f32 = 0.0;
    let mut f5_2: f32 = 0.0;
    let mut v0_35: i32 = 0;
    let mut f3_4: f32 = 0.0;
    let mut a1_19: i32 = 0;
    let mut a0_39: i32 = 0;
    let mut f0_28: f32 = 0.0;
    let mut v0_36: i32 = 0;
    let mut f25_1: f32 = 0.0;
    let mut a1_20: i32 = 0;
    let mut a0_40: i32 = 0;
    let mut f0_29: f32 = 0.0;
    let mut f29_1: f32 = 0.0;
    let mut a1_21: i32 = 0;
    let mut a0_41: i32 = 0;
    let mut f0_30: f32 = 0.0;
    let mut v0_37: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut a0_42: i32 = 0;
    let mut f0_31: f32 = 0.0;
    let mut s1_8: i32 = 0;
    let mut f0_32: f32 = 0.0;
    let mut a0_43: i32 = 0;
    let mut a1_23: i32 = 0;
    let mut f12_11: f32 = 0.0;
    let mut f0_33: f32 = 0.0;
    let mut f0_34: f32 = 0.0;
    let mut f3_5: f32 = 0.0;
    let mut f2_8: f32 = 0.0;
    let mut f1_14: f32 = 0.0;
    let mut f1_15: f32 = 0.0;
    let mut v0_38: i32 = 0;
    let mut s0_6: i32 = 0;
    let mut a0_44: i32 = 0;
    let mut v0_39: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut f0_35: f32 = 0.0;
    let mut f1_16: f32 = 0.0;
    let mut f0_36: f32 = 0.0;
    let mut f3_6: f32 = 0.0;
    let mut v0_40: i32 = 0;
    let mut f0_37: f32 = 0.0;
    let mut f1_17: f32 = 0.0;
    let mut f0_38: f32 = 0.0;
    let mut v0_41: i32 = 0;
    let mut a0_45: i32 = 0;
    let mut f1_18: f32 = 0.0;
    let mut f0_39: f32 = 0.0;
    let mut v0_42: i32 = 0;
    let mut f0_40: f32 = 0.0;
    let mut v0_43: i32 = 0;
    let mut v1_14: i32 = 0;
    let mut f2_9: f32 = 0.0;
    let mut f4_4: f32 = 0.0;
    let mut f3_7: f32 = 0.0;
    let mut f0_41: f32 = 0.0;
    let mut f1_19: f32 = 0.0;
    let mut f0_42: f32 = 0.0;
    let mut f0_43: f32 = 0.0;
    let mut f0_44: f32 = 0.0;
    let mut f0_45: f32 = 0.0;
    let mut f1_20: f32 = 0.0;
    let mut f1_21: f32 = 0.0;
    let mut f3_8: f32 = 0.0;
    let mut f1_22: f32 = 0.0;
    let mut f2_10: f32 = 0.0;
    let mut f0_46: f32 = 0.0;
    let mut f2_11: f32 = 0.0;
    let mut f0_47: f32 = 0.0;
    let mut f2_12: f32 = 0.0;
    let mut f0_48: f32 = 0.0;
    let mut f21_3: f32 = 0.0;
    let mut f12_12: f32 = 0.0;
    let mut f20_4: f32 = 0.0;
    let mut f0_49: f32 = 0.0;
    let mut s0_7: i32 = 0;
    let mut f12_13: f32 = 0.0;
    let mut f0_50: f32 = 0.0;
    let mut f0_51: f32 = 0.0;
    let mut f22_2: f32 = 0.0;
    let mut f12_14: f32 = 0.0;
    let mut f0_52: f32 = 0.0;
    let mut v0_44: i32 = 0;
    let mut f12_15: f32 = 0.0;
    let mut f0_53: f32 = 0.0;
    let mut v0_45: i32 = 0;
    let mut f12_16: f32 = 0.0;
    let mut f0_54: f32 = 0.0;
    let mut v0_46: i32 = 0;
    let mut f12_17: f32 = 0.0;
    let mut f0_55: f32 = 0.0;
    let mut v0_47: i32 = 0;
    let mut f12_18: f32 = 0.0;
    let mut f12_19: f32 = 0.0;
    let mut f0_56: f32 = 0.0;
    let mut f12_20: f32 = 0.0;
    let mut f0_57: f32 = 0.0;
    let mut f12_21: f32 = 0.0;
    let mut f12_22: f32 = 0.0;
    let mut v0_48: i32 = 0;
    let mut f5_3: f32 = 0.0;
    let mut f2_13: f32 = 0.0;
    let mut f4_5: f32 = 0.0;
    let mut f0_58: f32 = 0.0;
    let mut f8_1: f32 = 0.0;
    let mut f1_23: f32 = 0.0;
    let mut f7_2: f32 = 0.0;
    let mut f1_24: f32 = 0.0;
    let mut f0_59: f32 = 0.0;
    let mut f0_60: f32 = 0.0;
    let mut f3_9: f32 = 0.0;
    let mut f2_14: f32 = 0.0;
    let mut f1_25: f32 = 0.0;
    let mut f0_61: f32 = 0.0;
    let mut f0_62: f32 = 0.0;
    let mut f20_5: f32 = 0.0;
    let mut f0_63: f32 = 0.0;
    let mut f24_2: f32 = 0.0;
    let mut f12_23: f32 = 0.0;
    let mut f23_4: f32 = 0.0;
    let mut a1_24: i32 = 0;
    let mut a0_46: i32 = 0;
    let mut a1_25: i32 = 0;
    let mut f12_24: f32 = 0.0;
    let mut a0_47: i32 = 0;
    let mut f12_25: f32 = 0.0;
    let mut v0_49: i64 = 0;
    let mut a0_48: i64 = 0;
    let mut a1_26: i32 = 0;
    let mut v0_50: i32 = 0;
    let mut a1_27: i32 = 0;
    let mut f12_26: f32 = 0.0;
    let mut a0_49: i32 = 0;
    let mut s1_9: i32 = 0;
    let mut a0_50: i32 = 0;
    let mut f12_27: f32 = 0.0;
    let mut a5_1: i32 = 0;
    let mut v1_15: i32 = 0;
    let mut v0_51: i32 = 0;
    let mut f1_26: f32 = 0.0;
    let mut f2_15: f32 = 0.0;
    let mut s1_10: i32 = 0;
    let mut f12_28: f32 = 0.0;
    let mut a1_28: i32 = 0;
    let mut a0_51: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut v0_52: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a0_52: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a1_29: i32 = 0;
    let mut f12_29: f32 = 0.0;
    let mut f13_6: f32 = 0.0;
    let mut v0_53: i32 = 0;
    let mut a0_53: i32 = 0;
    let mut a1_30: i32 = 0;
    let mut f0_64: f32 = 0.0;
    let mut f2_16: f32 = 0.0;
    let mut f1_27: f32 = 0.0;
    let mut f4_6: f32 = 0.0;
    let mut f1_28: f32 = 0.0;
    let mut f3_10: f32 = 0.0;
    let mut f0_65: f32 = 0.0;
    let mut f2_17: f32 = 0.0;
    let mut s0_8: i32 = 0;
    let mut f13_7: f32 = 0.0;
    let mut s2_4: i32 = 0;
    let mut f12_30: f32 = 0.0;
    let mut f0_66: f32 = 0.0;
    let mut a0_54: i32 = 0;
    let mut f12_31: f32 = 0.0;
    let mut a1_31: i32 = 0;
    let mut f20_6: f32 = 0.0;
    let mut s1_11: i32 = 0;
    let mut f0_67: f32 = 0.0;
    let mut f0_68: f32 = 0.0;
    let mut f13_8: f32 = 0.0;
    let mut f12_32: f32 = 0.0;
    let mut f0_69: f32 = 0.0;
    let mut f20_7: f32 = 0.0;
    let mut f0_70: f32 = 0.0;
    let mut f0_71: f32 = 0.0;
    let mut f0_72: f32 = 0.0;
    let mut f1_29: f32 = 0.0;
    let mut f2_18: f32 = 0.0;
    let mut f3_11: f32 = 0.0;
    let mut f0_73: f32 = 0.0;
    let mut f1_30: f32 = 0.0;
    let mut f3_12: f32 = 0.0;
    let mut f0_74: f32 = 0.0;
    let mut f1_31: f32 = 0.0;
    let mut v1_16: i32 = 0;
    let mut f3_13: f32 = 0.0;
    let mut v0_54: i32 = 0;
    let mut f12_33: f32 = 0.0;
    let mut a0_55: i32 = 0;
    let mut a0_56: i32 = 0;
    let mut a1_32: i32 = 0;
    let mut v1_17: i32 = 0;
    let mut a2_11: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut v0_55: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a0_57: i32 = 0;
    let mut a1_33: i32 = 0;
    let mut f1_32: f32 = 0.0;
    let mut f2_19: f32 = 0.0;
    let mut f13_9: f32 = 0.0;
    let mut f12_34: f32 = 0.0;
    let mut v1_18: i32 = 0;
    let mut a0_58: i32 = 0;
    let mut v0_56: i32 = 0;
    let mut f12_35: f32 = 0.0;
    let mut f13_10: f32 = 0.0;
    let mut v0_57: i32 = 0;
    let mut s1_12: i32 = 0;
    let mut a0_59: i32 = 0;
    let mut v1_19: i32 = 0;
    let mut a0_60: i32 = 0;
    let mut v0_58: i32 = 0;
    let mut s1_13: i32 = 0;
    let mut a0_61: i32 = 0;
    let mut v0_59: i32 = 0;
    let mut a1_34: i32 = 0;
    let mut a0_62: i32 = 0;
    let mut a2_12: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut a1_35: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a0_63: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut f3_14: f32 = 0.0;
    let mut f6_2: f32 = 0.0;
    let mut a1_36: i32 = 0;
    let mut f4_7: f32 = 0.0;
    let mut a2_13: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut f1_33: f32 = 0.0;
    let mut f2_20: f32 = 0.0;
    let mut f1_34: f32 = 0.0;
    let mut f0_75: f32 = 0.0;
    let mut f12_36: f32 = 0.0;
    let mut f13_11: f32 = 0.0;
    let mut v0_60: i32 = 0;
    let mut f2_21: f32 = 0.0;
    let mut f1_35: f32 = 0.0;
    let mut a3_7: i32 = 0;
    let mut f2_22: f32 = 0.0;
    let mut a1_37: i32 = 0;
    let mut a2_14: i32 = 0;
    let mut a0_64: i32 = 0;
    let mut f3_15: f32 = 0.0;
    let mut a4_5: i32 = 0;
    let mut f12_37: f32 = 0.0;
    let mut f13_12: f32 = 0.0;
    let mut v0_61: i32 = 0;
    let mut v1_20: i32 = 0;
    let mut f4_8: f32 = 0.0;
    let mut v0_62: i32 = 0;
    let mut a0_65: i32 = 0;
    let mut a1_38: i32 = 0;
    let mut f1_36: f32 = 0.0;
    let mut f3_16: f32 = 0.0;
    let mut f0_76: f32 = 0.0;
    let mut f2_23: f32 = 0.0;
    let mut f1_37: f32 = 0.0;
    let mut f12_38: f32 = 0.0;
    let mut a1_39: i32 = 0;
    let mut f1_38: f32 = 0.0;
    let mut f1_39: f32 = 0.0;
    let mut f1_40: f32 = 0.0;
    let mut a0_66: i32 = 0;
    let mut a1_40: i32 = 0;
    let mut a2_15: i32 = 0;
    let mut a3_8: i32 = 0;
    let mut a4_6: i32 = 0;
    let mut a5_3: i32 = 0;
    let mut f2_24: f32 = 0.0;
    let mut f0_77: f32 = 0.0;
    let mut f1_41: f32 = 0.0;
    let mut f1_42: f32 = 0.0;
    let mut f5_4: f32 = 0.0;
    let mut f0_78: f32 = 0.0;
    let mut f2_25: f32 = 0.0;
    let mut f0_79: f32 = 0.0;
    let mut f4_9: f32 = 0.0;
    let mut f2_26: f32 = 0.0;
    let mut f3_17: f32 = 0.0;
    let mut f1_43: f32 = 0.0;
    let mut f1_44: f32 = 0.0;
    let mut f0_80: f32 = 0.0;
    let mut a3_9: i32 = 0;
    let mut a1_41: i32 = 0;
    let mut a0_67: i32 = 0;
    let mut a2_16: i32 = 0;
    let mut f12_39: f32 = 0.0;
    let mut f13_13: f32 = 0.0;
    let mut a4_7: i32 = 0;
    let mut v0_63: i32 = 0;
    let mut f28_1: f32 = 0.0;
    let mut s2_5: i32 = 0;
    let mut a0_68: i32 = 0;
    let mut f30_1: f32 = 0.0;
    let mut a0_69: i32 = 0;
    let mut v1_21: i32 = 0;
    let mut v0_64: i32 = 0;
    let mut a1_42: i32 = 0;
    let mut f0_81: f32 = 0.0;
    let mut f1_45: f32 = 0.0;
    let mut f7_3: f32 = 0.0;
    let mut f1_46: f32 = 0.0;
    let mut f5_5: f32 = 0.0;
    let mut f0_82: f32 = 0.0;
    let mut f4_10: f32 = 0.0;
    let mut f2_27: f32 = 0.0;
    let mut f22_3: f32 = 0.0;
    let mut f21_4: f32 = 0.0;
    let mut f20_8: f32 = 0.0;
    let mut f2_28: f32 = 0.0;
    let mut f6_3: f32 = 0.0;
    let mut f1_47: f32 = 0.0;
    let mut f3_18: f32 = 0.0;
    let mut f2_29: f32 = 0.0;
    let mut a2_17: i32 = 0;
    let mut a1_43: i32 = 0;
    let mut a0_70: i32 = 0;
    let mut v0_65: i32 = 0;
    let mut a2_18: i32 = 0;
    let mut f20_9: f32 = 0.0;
    let mut a4_8: i32 = 0;
    let mut a0_71: i32 = 0;
    let mut a3_10: i32 = 0;
    let mut f12_40: f32 = 0.0;
    let mut a1_44: i32 = 0;
    let mut f13_14: f32 = 0.0;
    let mut v0_66: i32 = 0;
    let mut f2_30: f32 = 0.0;
    let mut f20_10: f32 = 0.0;
    let mut f1_48: f32 = 0.0;
    let mut f4_11: f32 = 0.0;
    let mut f3_19: f32 = 0.0;
    let mut f3_20: f32 = 0.0;
    let mut f2_31: f32 = 0.0;
    let mut f0_83: f32 = 0.0;
    let mut f0_84: f32 = 0.0;
    let mut f0_85: f32 = 0.0;
    let mut f0_86: f32 = 0.0;
    let mut f1_49: f32 = 0.0;
    let mut f2_32: f32 = 0.0;
    let mut f1_50: f32 = 0.0;
    let mut f2_33: f32 = 0.0;
    let mut f3_21: f32 = 0.0;
    let mut v0_67: i32 = 0;
    let mut f2_34: f32 = 0.0;
    let mut v1_22: i32 = 0;
    let mut f0_87: f32 = 0.0;
    let mut f1_51: f32 = 0.0;
    let mut v0_68: i32 = 0;
    let mut f1_52: f32 = 0.0;
    let mut f2_35: f32 = 0.0;
    let mut f12_41: f32 = 0.0;
    let mut v0_69: i32 = 0;
    let mut v1_23: i32 = 0;
    let mut f1_53: f32 = 0.0;
    let mut f3_22: f32 = 0.0;
    let mut f2_36: f32 = 0.0;
    let mut f4_12: f32 = 0.0;
    let mut f0_88: f32 = 0.0;
    let mut f13_15: f32 = 0.0;
    let mut f14_6: f32 = 0.0;
    let mut a3_11: i32 = 0;
    let mut v1_24: i32 = 0;
    let mut s0_9: i32 = 0;
    let mut s0_10: i32 = 0;
    let mut a2_19: i32 = 0;
    let mut a0_72: i32 = 0;
    let mut a1_45: i32 = 0;
    let mut a4_9: i32 = 0;
    let mut v0_70: i32 = 0;
    let mut f7_4: f32 = 0.0;
    let mut f0_89: f32 = 0.0;
    let mut f9_1: f32 = 0.0;
    let mut f3_23: f32 = 0.0;
    let mut f2_37: f32 = 0.0;
    let mut f5_6: f32 = 0.0;
    let mut f4_13: f32 = 0.0;
    let mut f6_4: f32 = 0.0;
    let mut f1_54: f32 = 0.0;
    let mut f0_90: f32 = 0.0;
    let mut f2_38: f32 = 0.0;
    let mut f1_55: f32 = 0.0;
    let mut f4_14: f32 = 0.0;
    let mut f3_24: f32 = 0.0;
    let mut f6_5: f32 = 0.0;
    let mut f0_91: f32 = 0.0;
    let mut f5_7: f32 = 0.0;
    let mut f2_39: f32 = 0.0;
    let mut f1_56: f32 = 0.0;
    let mut f0_92: f32 = 0.0;
    let mut f3_25: f32 = 0.0;
    let mut f1_57: f32 = 0.0;
    let mut v0_71: i32 = 0;
    let mut f2_40: f32 = 0.0;
    let mut f3_26: f32 = 0.0;
    let mut v1_25: i32 = 0;
    let mut v0_72: i32 = 0;
    let mut a0_73: i32 = 0;
    let mut v0_73: i32 = 0;
    let mut v1_26: i32 = 0;
    let mut a0_74: i32 = 0;
    let mut v1_27: i32 = 0;
    let mut v1_28: i32 = 0;
    let mut a0_75: i32 = 0;
    let mut a5_4: i32 = 0;
    let mut a1_46: i32 = 0;
    let mut a4_10: i32 = 0;
    let mut a0_76: i32 = 0;
    let mut a2_20: i32 = 0;
    let mut a3_12: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut v0_74: i32 = 0;
    let mut a0_77: i32 = 0;
    let mut v1_29: i32 = 0;
    let mut v1_30: i32 = 0;
    let mut v1_31: i32 = 0;
    let mut v1_32: i32 = 0;
    let mut s0_11: i32 = 0;
    let mut f2_41: f32 = 0.0;
    let mut f0_93: f32 = 0.0;
    let mut f1_58: f32 = 0.0;
    let mut f1_59: f32 = 0.0;
    let mut f2_42: f32 = 0.0;
    let mut f0_94: f32 = 0.0;
    let mut f3_27: f32 = 0.0;
    let mut f12_42: f32 = 0.0;
    let mut f13_16: f32 = 0.0;
    let mut f0_95: f32 = 0.0;
    let mut f21_5: f32 = 0.0;
    let mut f12_43: f32 = 0.0;
    let mut f13_17: f32 = 0.0;
    let mut f0_96: f32 = 0.0;
    let mut v0_75: i32 = 0;
    let mut f2_43: f32 = 0.0;
    let mut a0_78: i32 = 0;
    let mut v1_33: i32 = 0;
    let mut a0_79: i32 = 0;
    let mut a1_47: i32 = 0;
    let mut a0_80: i32 = 0;
    let mut v0_76: i32 = 0;
    let mut v1_34: i32 = 0;
    let mut s0_12: i32 = 0;
    let mut a0_81: i32 = 0;
    let mut v0_77: i32 = 0;
    let mut a0_82: i32 = 0;
    let mut a2_21: i32 = 0;
    let mut a1_48: i32 = 0;
    let mut v0_78: i32 = 0;
    let mut f1_60: f32 = 0.0;
    let mut f0_97: f32 = 0.0;
    let mut v1_35: i32 = 0;
    let mut v0_79: i32 = 0;
    let mut v1_36: i32 = 0;
    let mut v0_80: i32 = 0;
    let mut f0_98: f32 = 0.0;
    let mut f0_99: f32 = 0.0;
    let mut v0_81: i32 = 0;
    let mut bc_2: bool = false;
    let mut v0_82: i32 = 0;
    let mut a0_83: i32 = 0;
    let mut v0_83: i32 = 0;
    let mut a0_84: i32 = 0;
    'b1: {
        v0_1 = lw(g::currentplayer);
        a0_1 = sp + 16;
        fp_1 = 0;
        s2_1 = 0;
        s0_1 = 0;
        v1_1 = lw(v0_1 + 384);
        f14_1 = lwc1(v0_1 + 148);
        f12_1 = lwc1(v0_1 + 140);
        f13_1 = lwc1(v0_1 + 144);
        f15_1 = lwc1(v1_1 + 76) * lwc1(0x3ad008) / 1.8e+02_f32;
        s3_1 = lw(v1_1 + 352);
        matrixTransRotY(a0_1, f12_1, f13_1, f14_1, f15_1);
        a0_2 = sp + 80;
        a1_1 = sp + 16;
        v1_2 = lw(lw(lw(g::currentplayer) + 384) + 32);
        a2_1 = lw(v1_2 + 4) + (lw(lw(v1_2 + 92) + 20) << 6);
        sceVu0MulMatrix(a0_2, a1_1, a2_1);
        f1_1 = lwc1(0x3ad00c);
        swc1(s3_1 + 3008, lwc1(sp + 128));
        a3_1 = lw(g::lvnumplayers);
        a2_2 = lw(g::numactivechars);
        swc1(s3_1 + 3012, lwc1(sp + 132) - f1_1);
        swc1(s3_1 + 3016, lwc1(sp + 136));
        if (a3_1 + a2_2) <= 0 {
            s1_1 = lw(g::currentplayer);
        } else {
            s1_1 = lw(g::currentplayer);
            'l2: loop {
                a1_2 = lw(s3_1);
                v0_2 = lw(s0_1 * 12 + lw(s1_1 + 820));
                s2_1 = s2_1 + v0_2;
                if s0_1 != a1_2 {
                    f0_1 = lwc1(g::updateframesf);
                    if v0_2 != 0 {
                        a0_3 = 4;
                        a2_3 = s0_1;
                        f12_2 = f0_1 * lwc1(0x3ad010);
                        StatsAddChar(a0_3, a1_2, a2_3, f12_2);
                        s1_1 = lw(g::currentplayer);
                        a3_1 = lw(g::lvnumplayers);
                        a2_2 = lw(g::numactivechars);
                    }
                }
                s0_1 = s0_1 + 1;
                if s0_1 >= a3_1 + a2_2 {
                    break 'b1;
                }
            }
        }
    }
    'b4: {
        'b5: {
            if s2_1 <= 0 {
                v0_3 = lw(s1_1 + 792);
                if v0_3 != 0 {
                    v1_3 = lw(s1_1 + 792);
                    break 'b5;
                }
                a0_5 = lw(lw(s1_1 + 384) + 352);
                sw(a0_5 + 2708, lw(a0_5 + 2708) & -17);
            } else {
                a0_4 = lw(lw(s1_1 + 384) + 352);
                sw(a0_4 + 2708, lw(a0_4 + 2708) | 16);
            }
            if lw(s1_1 + 792) == 0 {
                break 'b4;
            }
            v1_3 = lw(s1_1 + 792);
        }
        if v1_3 != 60 {
            v0_4 = lw(s1_1 + 792);
        } else {
            sw(lw(lw(s1_1 + 384) + 32) + 228, 0);
            v0_4 = lw(s1_1 + 792);
        }
        sw(s1_1 + 792, v0_4 - 1);
    }
    'b8: {
        v0_5 = debugmenuGetJoyMode();
        s1_2 = lw(g::currentplayer);
        if v0_5 == 0 {
            v0_6 = lw(g::currentplayer);
            a1_3 = lw(v0_6 + 96);
            a0_6 = lw(v0_6);
            f0_2 = joyGetAxis(a0_6, a1_3);
            v0_7 = lw(g::currentplayer);
            f23_1 = -f0_2;
            a1_4 = lw(v0_7 + 92);
            a0_7 = lw(v0_7);
            joyGetAxis(a0_7, a1_4);
            s1_3 = lw(g::currentplayer);
            f2_1 = lwc1(s1_3 + 140) - lwc1(s1_3 + 344);
            f0_3 = lwc1(s1_3 + 144) - lwc1(s1_3 + 348);
            f21_1 = f2_1 * f2_1;
            f1_2 = lwc1(s1_3 + 148) - lwc1(s1_3 + 352);
            f20_1 = f1_2 * f1_2;
            s0_2 = s1_3;
            swc1(s0_2 + 376, (f21_1 + f0_3 * f0_3 + f20_1).abs().sqrt());
            s0_3 = s1_3;
            swc1(s0_3 + 380, (f21_1 + f20_1).abs().sqrt());
            v0_8 = lw(s1_3 + 292);
            f1_3 = lwc1(s1_3 + 144);
            f2_2 = lwc1(s1_3 + 148);
            f3_1 = lwc1(s1_3 + 152);
            f4_1 = lwc1(s1_3 + 156);
            f5_1 = lwc1(s1_3 + 160);
            f6_1 = lwc1(s1_3 + 276);
            f7_1 = lwc1(s1_3 + 284);
            swc1(s1_3 + 344, lwc1(s1_3 + 140));
            swc1(s1_3 + 348, f1_3);
            swc1(s1_3 + 352, f2_2);
            swc1(s1_3 + 356, f3_1);
            swc1(s1_3 + 360, f4_1);
            swc1(s1_3 + 364, f5_1);
            swc1(s1_3 + 368, f6_1);
            swc1(s1_3 + 372, f7_1);
            if v0_8 > 0 {
                sw(s1_3 + 340, v0_8);
            }
            'b11: {
                'b12: {
                    'b13: {
                        a0_8 = lw(s1_3 + 296);
                        sw(s1_3 + 292, lw(g::updateframesi));
                        if a0_8 != 4 {
                            if lw(g::padlinkedit) != 1 {
                                if a0_8 != 6 {
                                    if a0_8 == 7 {
                                        v0_13 = lw(s1_3 + 20);
                                        break 'b13;
                                    }
                                    if (lhu(lw(s1_3 + 24) + 2) & 4) == 0 {
                                        a0_10 = lw(s1_3);
                                        v0_12 = joyBut(a0_10);
                                        s1_3 = lw(g::currentplayer);
                                        v0_13 = lw(s1_3 + 20);
                                        if (v0_12 & lw(s1_3 + 56)) == 0 {
                                            break 'b13;
                                        }
                                        v0_10 = v0_13 | 1;
                                        break 'b12;
                                    }
                                    a0_9 = lw(s1_3);
                                    v0_9 = joyNewBut(a0_9);
                                    s1_3 = lw(g::currentplayer);
                                    if (v0_9 & lw(s1_3 + 56)) == 0 {
                                        v0_11 = lw(s1_3 + 296);
                                        break 'b11;
                                    }
                                    v0_10 = lw(s1_3 + 20) ^ 1;
                                    break 'b12;
                                }
                            }
                        }
                        v0_13 = lw(s1_3 + 20);
                    }
                    v0_10 = v0_13 & -2;
                }
                sw(s1_3 + 20, v0_10);
                v0_11 = lw(s1_3 + 296);
            }
            f0_4 = lwc1(g::updateframesf);
            if v0_11 != 4 {
                a0_11 = 65;
                a1_5 = lw(s1_3);
                f12_3 = f0_4 * lwc1(0x3ad014);
                StatsAdd(a0_11, a1_5, f12_3);
                s1_3 = lw(g::currentplayer);
            }
            'b16: {
                'b17: {
                    s2_2 = lw(s1_3 + 20);
                    if lw(s1_3 + 296) != 4 {
                        a0_12 = lw(s1_3);
                        v0_15 = joyNewBut(a0_12);
                        s1_3 = lw(g::currentplayer);
                        v1_4 = lw(s1_3 + 24);
                        if (v0_15 & lw(s1_3 + 44)) == 0 {
                            if (lhu(v1_4 + 2) & 2) != 0 {
                                v0_18 = lw(s1_3 + 20);
                                break 'b16;
                            }
                            a0_13 = lw(s1_3);
                            v0_17 = joyReleaseBut(a0_13);
                            s1_3 = lw(g::currentplayer);
                            if (v0_17 & lw(s1_3 + 44)) == 0 {
                                break 'b17;
                            }
                            v0_14 = lw(s1_3 + 20) & -3;
                        } else {
                            v0_16 = lw(s1_3 + 20);
                            if (lhu(v1_4 + 2) & 2) == 0 {
                                v0_14 = v0_16 | 2;
                            } else {
                                v0_14 = v0_16 ^ 2;
                            }
                        }
                    } else {
                        v0_14 = s2_2 & -3;
                    }
                    sw(s1_3 + 20, v0_14);
                }
                v0_18 = lw(s1_3 + 20);
            }
            bc_1 = v0_18 == s2_2;
            v0_19 = v0_18 & 2;
            if !bc_1 {
                if v0_19 == 0 {
                    f0_5 = lwc1(s1_3 + 264);
                    swc1(s1_3 + 268, f0_5);
                    swc1(s1_3 + 264, 6e+01_f32);
                } else {
                    swc1(s1_3 + 264, lwc1(s1_3 + 268));
                }
            }
            if (lw(s1_3 + 20) & 2) == 0 {
                f1_7 = lwc1(s1_3 + 264);
            } else {
                a0_14 = lw(s1_3);
                v0_20 = joyBut(a0_14);
                s1_4 = lw(g::currentplayer);
                f1_4 = lwc1(g::updateframesf);
                if (v0_20 & lw(s1_4 + 48)) != 0 {
                    swc1(s1_4 + 264, lwc1(s1_4 + 264) - f1_4);
                }
                a0_15 = lw(s1_4);
                v0_21 = joyBut(a0_15);
                s1_3 = lw(g::currentplayer);
                f1_5 = lwc1(g::updateframesf);
                if (v0_21 & lw(s1_3 + 52)) != 0 {
                    swc1(s1_3 + 264, lwc1(s1_3 + 264) + f1_5);
                }
                f1_6 = lwc1(s1_3 + 272);
                if lwc1(s1_3 + 264) < f1_6 {
                    swc1(s1_3 + 264, f1_6);
                }
                if 6e+01_f32 < lwc1(s1_3 + 264) {
                    swc1(s1_3 + 264, 6e+01_f32);
                }
                f0_6 = lwc1(g::updateframesf);
                if (lw(s1_3 + 20) & 2) != 0 {
                    a0_16 = 56;
                    a1_6 = lw(s3_1);
                    f12_4 = f0_6 * lwc1(0x3ad018);
                    StatsAdd(a0_16, a1_6, f12_4);
                    s1_3 = lw(g::currentplayer);
                }
                f1_7 = lwc1(s1_3 + 264);
            }
            f2_3 = lwc1(s1_3 + 260);
            if f1_7 == f2_3 {
                v1_5 = lw(s1_3 + 296);
            } else {
                f0_7 = lwc1(g::updateframesf);
                f1_8 = f1_7 - f2_3;
                f0_8 = f0_7 + f0_7;
                if f0_8 < f1_8 {
                    f1_8 = f0_8;
                }
                f0_9 = -f0_8;
                if f1_8 < f0_9 {
                    f1_8 = f0_9;
                }
                swc1(s1_3 + 260, f2_3 + f1_8);
                v1_5 = lw(s1_3 + 296);
            }
            a0_17 = 0;
            if v1_5 != 4 {
                a0_17 = ((0 as u32) < ((lw(s1_3 + 20) & 2) as u32)) as i32;
            }
            'b30: {
                sw(s1_3 + 800, a0_17);
                if a0_17 == 0 {
                    a1_9 = lw(s1_3 + 80);
                    a0_22 = lw(s1_3);
                    f0_16 = joyGetAxis(a0_22, a1_9);
                    v0_24 = lw(g::currentplayer);
                    a1_10 = lw(v0_24 + 76);
                    a0_23 = lw(v0_24);
                    f31_1 = -f0_16;
                    f0_17 = joyGetAxis(a0_23, a1_10);
                    if !(0.0_f32 < f0_17) {
                        if !(f0_17 < -0.5_f32) {
                            f22_1 = 0.0_f32;
                        } else {
                            f0_19 = f0_17 + 0.5_f32;
                            f22_1 = f0_19 + f0_19;
                        }
                    } else if !(0.5_f32 < f0_17) {
                        f22_1 = 0.0_f32;
                    } else {
                        f0_18 = f0_17 - 0.5_f32;
                        f22_1 = f0_18 + f0_18;
                    }
                    s1_6 = lw(g::currentplayer);
                    if (lhu(lw(s1_6 + 24) + 2) & 1) != 0 {
                        f22_1 = -f22_1;
                    }
                    'b33: {
                        'b34: {
                            'b35: {
                                if lw(s1_6 + 612) != 0 {
                                    v1_7 = lw(s1_6 + 24);
                                } else if lw(s1_6 + 420) != 0 {
                                    v1_7 = lw(s1_6 + 24);
                                } else {
                                    if lw(s1_6 + 296) != 7 {
                                        a1_11 = lw(s1_6 + 76);
                                        a0_24 = lw(s1_6);
                                        f0_20 = joyGetAxis(a0_24, a1_11);
                                        f1_9 = f0_20;
                                        s1_5 = lw(g::currentplayer);
                                        f2_4 = lwc1(0x3ad038);
                                        a0_25 = s1_5;
                                        v0_25 = lhu(lw(s1_5 + 24) + 2) & 1;
                                        swc1(s1_5 + 804, -f23_1 * f2_4);
                                        if v0_25 == 0 {
                                            break 'b35;
                                        }
                                        break 'b34;
                                    }
                                    v1_7 = lw(s1_6 + 24);
                                }
                                'b37: {
                                    f20_2 = lwc1(0x3ad01c);
                                    s5_1 = -1;
                                    if (lhu(v1_7 + 2) & 128) != 0 {
                                        a0_26 = s3_1;
                                        v0_26 = gunDoesAutoAim(a0_26);
                                        a3_2 = lw(g::lvnumplayers);
                                        if v0_26 != 0 {
                                            a2_4 = lw(g::numactivechars);
                                            s2_3 = 0;
                                            if (a3_2 + a2_4) > 0 {
                                                f24_1 = lwc1(0x3ad020);
                                                'l38: loop {
                                                    'b39: {
                                                        s1_7 = lw(lw(g::currentplayer) + 820) + s2_3 * 12;
                                                        s0_4 = lw(g::chrinfo) + s2_3 * 0x1210;
                                                        if lw(s1_7) != 0 {
                                                            v0_27 = lw(s0_4 + 3020);
                                                            if !(0.0_f32 < lwc1(v0_27 + 520) + lwc1(s0_4 + 12)) {
                                                                s2_3 = s2_3 + 1;
                                                                break 'b39;
                                                            }
                                                            a0_27 = sp + 16;
                                                            if f20_2 < lwc1(s1_7 + 4) {
                                                                f14_2 = lwc1(v0_27 + 56);
                                                                f12_7 = lwc1(v0_27 + 48);
                                                                f15_2 = lwc1(v0_27 + 76) * f24_1 / 1.8e+02_f32;
                                                                f13_2 = lwc1(v0_27 + 52);
                                                                matrixTransRotY(a0_27, f12_7, f13_2, f14_2, f15_2);
                                                                a0_28 = sp + 80;
                                                                a1_12 = sp + 16;
                                                                v0_28 = lw(lw(s0_4 + 3020) + 32);
                                                                a2_5 = lw(v0_28 + 4) + (lw(lw(v0_28 + 92) + 20) << 6);
                                                                sceVu0MulMatrix(a0_28, a1_12, a2_5);
                                                                a0_29 = sp + 16;
                                                                f2_5 = lwc1(sp + 132) - lwc1(0x3ad024);
                                                                v1_8 = lw(g::currentplayer);
                                                                f0_21 = lwc1(sp + 136);
                                                                swc1(sp + 160, lwc1(sp + 128));
                                                                swc1(sp + 164, f2_5);
                                                                swc1(sp + 168, f0_21);
                                                                f14_3 = lwc1(v1_8 + 148);
                                                                f12_8 = lwc1(v1_8 + 140);
                                                                f15_3 = lwc1(lw(v1_8 + 384) + 76) * f24_1 / 1.8e+02_f32;
                                                                f13_3 = lwc1(v1_8 + 144);
                                                                matrixTransRotY(a0_29, f12_8, f13_3, f14_3, f15_3);
                                                                a0_30 = sp + 80;
                                                                a1_13 = sp + 16;
                                                                v1_9 = lw(lw(lw(g::currentplayer) + 384) + 32);
                                                                a2_6 = lw(v1_9 + 4) + (lw(lw(v1_9 + 92) + 20) << 6);
                                                                sceVu0MulMatrix(a0_30, a1_13, a2_6);
                                                                a1_14 = sp + 176;
                                                                a2_7 = s1_7;
                                                                f1_10 = lwc1(sp + 164) - lwc1(sp + 132);
                                                                a0_31 = lw(g::currentplayer);
                                                                f0_22 = lwc1(sp + 168) - lwc1(sp + 136);
                                                                swc1(sp + 176, lwc1(sp + 160) - lwc1(sp + 128));
                                                                swc1(sp + 180, f1_10);
                                                                a0_32 = a0_31 + 188;
                                                                swc1(sp + 184, f0_22);
                                                                playerAutoAimCalc(a0_32, a1_14, a2_7);
                                                                f0_23 = lwc1(s1_7 + 4);
                                                                a3_2 = lw(g::lvnumplayers);
                                                                if f20_2 < f0_23 {
                                                                    f20_2 = f0_23;
                                                                    s5_1 = s2_3;
                                                                }
                                                                a2_4 = lw(g::numactivechars);
                                                            }
                                                        }
                                                        s2_3 = s2_3 + 1;
                                                    }
                                                    if s2_3 >= a3_2 + a2_4 {
                                                        break 'b37;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if s5_1 >= 0 {
                                    f20_3 = lwc1(0x3ad028);
                                    a0_33 = sp + 16;
                                    s0_5 = lw(g::chrinfo) + s5_1 * 0x1210;
                                    v0_29 = lw(s0_5 + 3020);
                                    f14_4 = lwc1(v0_29 + 56);
                                    f12_9 = lwc1(v0_29 + 48);
                                    f15_4 = lwc1(v0_29 + 76) * f20_3 / 1.8e+02_f32;
                                    f13_4 = lwc1(v0_29 + 52);
                                    matrixTransRotY(a0_33, f12_9, f13_4, f14_4, f15_4);
                                    a0_34 = sp + 80;
                                    a1_15 = sp + 16;
                                    v0_30 = lw(lw(s0_5 + 3020) + 32);
                                    a2_8 = lw(v0_30 + 4) + (lw(lw(v0_30 + 92) + 20) << 6);
                                    sceVu0MulMatrix(a0_34, a1_15, a2_8);
                                    a0_35 = sp + 16;
                                    f0_24 = lwc1(sp + 132) - lwc1(0x3ad02c);
                                    v1_10 = lw(g::currentplayer);
                                    f1_11 = lwc1(sp + 136);
                                    swc1(sp + 160, lwc1(sp + 128));
                                    swc1(sp + 164, f0_24);
                                    swc1(sp + 168, f1_11);
                                    f14_5 = lwc1(v1_10 + 148);
                                    f12_10 = lwc1(v1_10 + 140);
                                    f15_5 = lwc1(lw(v1_10 + 384) + 76) * f20_3 / 1.8e+02_f32;
                                    f13_5 = lwc1(v1_10 + 144);
                                    matrixTransRotY(a0_35, f12_10, f13_5, f14_5, f15_5);
                                    a0_36 = sp + 80;
                                    a1_16 = sp + 16;
                                    v1_11 = lw(lw(lw(g::currentplayer) + 384) + 32);
                                    a2_9 = lw(v1_11 + 4) + (lw(lw(v1_11 + 92) + 20) << 6);
                                    sceVu0MulMatrix(a0_36, a1_16, a2_9);
                                    a1_17 = sp + 192;
                                    f1_12 = lwc1(sp + 164) - lwc1(sp + 132);
                                    v0_31 = lw(g::currentplayer);
                                    f0_25 = lwc1(sp + 168) - lwc1(sp + 136);
                                    swc1(sp + 192, lwc1(sp + 160) - lwc1(sp + 128));
                                    a0_37 = lw(v0_31 + 1764);
                                    swc1(sp + 196, f1_12);
                                    swc1(sp + 200, f0_25);
                                    matrixVecRotAligned(&[a0_37 as i64, a1_17 as i64]);
                                    s1_5 = lw(g::currentplayer);
                                    v0_32 = lw(s0_5 + 3020);
                                    swc1(s1_5 + 832, lwc1(sp + 160));
                                    f3_2 = lwc1(s1_5 + 256);
                                    f2_6 = lwc1(0x3ad030);
                                    swc1(s1_5 + 836, lwc1(sp + 164));
                                    f4_2 = lwc1(s1_5 + 252);
                                    f1_13 = lwc1(sp + 168);
                                    sw(s1_5 + 828, v0_32);
                                    swc1(s1_5 + 840, f1_13);
                                    swc1(s1_5 + 804, -(lwc1(sp + 192) / lwc1(sp + 200)) / f3_2 / f2_6);
                                    swc1(s1_5 + 808, lwc1(sp + 196) / lwc1(sp + 200) / f4_2 / f2_6);
                                    break 'b30;
                                }
                                v0_33 = lw(g::currentplayer);
                                a1_18 = lw(v0_33 + 76);
                                a0_38 = lw(v0_33);
                                f0_26 = joyGetAxis(a0_38, a1_18);
                                f1_9 = f0_26;
                                s1_5 = lw(g::currentplayer);
                                f2_4 = lwc1(0x3ad034);
                                a0_25 = s1_5;
                                v0_34 = lhu(lw(s1_5 + 24) + 2) & 1;
                                swc1(s1_5 + 804, -f23_1 * f2_4);
                                if v0_34 != 0 {
                                    break 'b34;
                                }
                            }
                            f0_27 = f1_9 * lwc1(0x3ad03c);
                            break 'b33;
                        }
                        f0_27 = f1_9 * f2_4;
                    }
                    swc1(a0_25 + 808, f0_27);
                    sw(a0_25 + 828, 0);
                } else {
                    v1_6 = lw(lw(s1_3 + 384) + 352);
                    f12_5 = 0.0_f32;
                    sw(v1_6 + 2708, lw(v1_6 + 2708) | 16);
                    a0_18 = lw(s1_3);
                    joySetCentreZone(a0_18, f12_5);
                    v0_22 = lw(g::currentplayer);
                    a1_7 = lw(v0_22 + 96);
                    a0_19 = lw(v0_22);
                    f0_10 = joyGetAxis(a0_19, a1_7);
                    v0_23 = lw(g::currentplayer);
                    f23_2 = -f0_10;
                    a1_8 = lw(v0_23 + 92);
                    a0_20 = lw(v0_23);
                    f0_11 = joyGetAxis(a0_20, a1_8);
                    f12_6 = 0.25_f32;
                    a0_21 = lw(lw(g::currentplayer));
                    joySetCentreZone(a0_21, f12_6);
                    if !(0.0_f32 <= f23_2) {
                        f23_3 = -(f23_2 * f23_2);
                    } else {
                        f23_3 = f23_2 * f23_2;
                    }
                    if !(0.0_f32 <= f0_11) {
                        f21_2 = -(f0_11 * f0_11);
                    } else {
                        f21_2 = f0_11 * f0_11;
                    }
                    s1_5 = lw(g::currentplayer);
                    if (lhu(lw(s1_5 + 24) + 2) & 1) != 0 {
                        f21_2 = -f21_2;
                    }
                    swc1(s1_5 + 804, -f23_3);
                    swc1(s1_5 + 808, -f21_2);
                    if !(0.0_f32 < f23_3) {
                        f31_1 = 0.0_f32;
                        if f23_3 < -0.5_f32 {
                            f0_13 = f23_3 + 0.5_f32;
                            f31_1 = f0_13 + f0_13;
                        }
                    } else {
                        f31_1 = 0.0_f32;
                        if 0.5_f32 < f23_3 {
                            f0_12 = f23_3 - 0.5_f32;
                            f31_1 = f0_12 + f0_12;
                        }
                    }
                    if !(0.0_f32 < f21_2) {
                        f22_1 = 0.0_f32;
                        if f21_2 < -0.5_f32 {
                            f0_15 = f21_2 + 0.5_f32;
                            f22_1 = f0_15 + f0_15;
                        }
                    } else {
                        f22_1 = 0.0_f32;
                        if 0.5_f32 < f21_2 {
                            f0_14 = f21_2 - 0.5_f32;
                            f22_1 = f0_14 + f0_14;
                        }
                    }
                }
            }
            f2_7 = lwc1(s1_5 + 812);
            f3_3 = lwc1(s1_5 + 816);
            f4_3 = lwc1(0x3ad040);
            f5_2 = lwc1(g::updateframesf);
            v0_35 = lw(s1_5 + 800);
            f3_4 = f3_3 + (lwc1(s1_5 + 808) - f3_3) * f4_3 * f5_2;
            swc1(s1_5 + 812, f2_7 + (lwc1(s1_5 + 804) - f2_7) * f4_3 * f5_2);
            swc1(s1_5 + 816, f3_4);
            if v0_35 == 0 {
                a1_21 = lw(s1_5 + 68);
                a0_41 = lw(s1_5);
                f0_30 = joyGetAxis(a0_41, a1_21);
                v0_37 = lw(g::currentplayer);
                f25_1 = f0_30;
                a1_22 = lw(v0_37 + 72);
                a0_42 = lw(v0_37);
                f0_31 = joyGetAxis(a0_42, a1_22);
                f29_1 = f0_31;
            } else {
                a1_19 = lw(s1_5 + 84);
                a0_39 = lw(s1_5);
                f0_28 = joyGetAxis(a0_39, a1_19);
                v0_36 = lw(g::currentplayer);
                f25_1 = f0_28;
                a1_20 = lw(v0_36 + 88);
                a0_40 = lw(v0_36);
                f0_29 = joyGetAxis(a0_40, a1_20);
                f29_1 = f0_29;
            }
            'b47: {
                'b48: {
                    s1_8 = lw(g::currentplayer);
                    f0_32 = lwc1(g::updateframesf);
                    if (lw(s1_8 + 20) & 1) != 0 {
                        a0_43 = 54;
                        a1_23 = lw(s3_1);
                        f12_11 = f0_32 * lwc1(0x3ad044);
                        StatsAdd(a0_43, a1_23, f12_11);
                        s1_8 = lw(g::currentplayer);
                        if f25_1 == 0.0_f32 {
                            if f29_1 == 0.0_f32 {
                                f0_33 = lwc1(s1_8 + 108);
                                break 'b48;
                            }
                            s1_8 = lw(g::currentplayer);
                        }
                        swc1(s1_8 + 120, lwc1(s1_8 + 108) + lwc1(0x3ad048) - lwc1(s1_8 + 112));
                        break 'b47;
                    }
                    f0_33 = lwc1(s1_8 + 104);
                }
                swc1(s1_8 + 120, f0_33 - lwc1(s1_8 + 112));
            }
            f0_34 = lwc1(s1_8 + 120);
            f3_5 = lwc1(s1_8 + 116);
            if f0_34 == f3_5 {
                v0_38 = lw(s1_8 + 384);
            } else {
                f2_8 = f0_34 - f3_5;
                f1_14 = lwc1(g::updateframesf) * lwc1(0x3ad04c);
                if f1_14 < f2_8 {
                    f2_8 = f1_14;
                }
                f1_15 = -f1_14;
                if f2_8 < f1_15 {
                    f2_8 = f1_15;
                }
                swc1(s1_8 + 116, f3_5 + f2_8 * lwc1(0x3ad050));
                v0_38 = lw(s1_8 + 384);
            }
            'b53: {
                s0_6 = lw(v0_38 + 352);
                a0_44 = lw(s0_6 + 0x11a4);
                if a0_44 != 0 {
                    if lw(a0_44 + 4) != 2 {
                        v0_39 = lw(s1_8 + 20);
                        break 'b53;
                    }
                    specialfxFreeSpawn(a0_44);
                    sw(s0_6 + 0x11a4, 0);
                    s1_8 = lw(g::currentplayer);
                }
                v0_39 = lw(s1_8 + 20);
            }
            v1_12 = lw(s1_8 + 288);
            if (v0_39 & 1) != 0 {
                f31_1 = f31_1 * 0.75_f32;
                f25_1 = f25_1 * 0.75_f32;
                f29_1 = f29_1 * 0.75_f32;
            }
            'b56: {
                'b57: {
                    if v1_12 == 1 {
                        if !(0.0_f32 <= f25_1) {
                            f0_36 = -f25_1;
                            if !(f0_36 == 0.0_f32) {
                                v1_13 = lw(s1_8 + 296);
                                break 'b56;
                            }
                            sw(s1_8 + 288, 0);
                        } else {
                            if !(f25_1 == 0.0_f32) {
                                v1_13 = lw(s1_8 + 296);
                                break 'b56;
                            }
                            sw(s1_8 + 288, 0);
                        }
                    } else if v1_12 >= 2 {
                        if v1_12 != 2 {
                            v1_13 = lw(s1_8 + 296);
                            break 'b56;
                        }
                        if !(f22_1 == 0.0_f32) {
                            sw(s1_8 + 288, 1);
                        }
                    } else {
                        if v1_12 != 0 {
                            v1_13 = lw(s1_8 + 296);
                            break 'b56;
                        }
                        'b58: {
                            if !(f22_1 == 0.0_f32) {
                                if !(0.0_f32 <= f25_1) {
                                    f0_35 = -f25_1;
                                    if !(0.0_f32 < f0_35) {
                                        break 'b58;
                                    }
                                } else if !(0.0_f32 < f25_1) {
                                    break 'b58;
                                }
                                sw(s1_8 + 288, 1);
                                break 'b57;
                            }
                        }
                        if !(f22_1 == 0.0_f32) {
                            v1_13 = lw(s1_8 + 296);
                            break 'b56;
                        }
                        if !(0.0_f32 <= f25_1) {
                            f1_16 = -f25_1;
                            if !(0.75_f32 < f1_16) {
                                break 'b57;
                            }
                        } else if !(0.75_f32 < f25_1) {
                            v1_13 = lw(s1_8 + 296);
                            break 'b56;
                        }
                        sw(s1_8 + 288, 2);
                    }
                }
                v1_13 = lw(s1_8 + 296);
            }
            'b61: {
                'b62: {
                    if v1_13 == 6 {
                        a0_45 = lw(g::padlinkedit);
                    } else {
                        'b63: {
                            'b64: {
                                f3_6 = lwc1(g::updateframesf);
                                v0_40 = lw(s1_8 + 288);
                                swc1(s1_8 + 276, lwc1(s1_8 + 276) + f31_1 * lwc1(0x3ad054) * (lwc1(s1_8 + 260) / 6e+01_f32) * f3_6);
                                if v0_40 == 2 {
                                    if (lhu(lw(s1_8 + 24) + 2) & 8) != 0 {
                                        f0_37 = lwc1(s1_8 + 284);
                                        f1_17 = f0_37 - f0_37 * lwc1(0x3ad058) * f3_6;
                                        swc1(s1_8 + 284, f1_17);
                                        if !(0.0_f32 <= f1_17) {
                                            if !(-f1_17 < 0.5_f32) {
                                                break 'b64;
                                            }
                                        } else if !(f1_17 < 0.5_f32) {
                                            f0_38 = lwc1(s1_8 + 276);
                                            break 'b63;
                                        }
                                        sw(s1_8 + 288, 1);
                                        break 'b64;
                                    }
                                }
                                swc1(s1_8 + 284, lwc1(s1_8 + 284) + f22_1 * lwc1(0x3ad05c) * (lwc1(s1_8 + 260) / 6e+01_f32) * lwc1(g::updateframesf));
                            }
                            f0_38 = lwc1(s1_8 + 276);
                        }
                        v0_41 = s1_8;
                        if !(f0_38 < 0.0_f32) {
                            a0_45 = lw(g::padlinkedit);
                            f0_39 = lwc1(s1_8 + 276);
                        } else {
                            a0_45 = lw(g::padlinkedit);
                            'l68: loop {
                                f1_18 = lwc1(v0_41 + 276) + 3.6e+02_f32;
                                swc1(v0_41 + 276, f1_18);
                                if !(f1_18 < 0.0_f32) {
                                    break;
                                }
                            }
                            f0_39 = lwc1(s1_8 + 276);
                        }
                        'b70: {
                            v0_42 = s1_8;
                            if 3.6e+02_f32 <= f0_39 {
                                'l71: loop {
                                    f0_40 = lwc1(v0_42 + 276) - 3.6e+02_f32;
                                    swc1(v0_42 + 276, f0_40);
                                    if !(3.6e+02_f32 <= f0_40) {
                                        break 'b70;
                                    }
                                }
                            }
                        }
                        if lw(s1_8 + 296) != 7 {
                            f0_45 = lwc1(s1_8 + 284);
                            if f0_45 < -5e+01_f32 {
                                swc1(s1_8 + 284, -5e+01_f32);
                            }
                            if !(5e+01_f32 < lwc1(s1_8 + 284)) {
                                f0_44 = lwc1(s1_8 + 312);
                                break 'b61;
                            }
                            swc1(s1_8 + 284, 5e+01_f32);
                        } else {
                            v0_43 = lw(s3_1 + 0x11b4);
                            v1_14 = lw(v0_43 + 352);
                            f2_9 = lwc1(v0_43 + 76);
                            f4_4 = lwc1(v1_14 + 40) + f2_9;
                            f3_7 = lwc1(v1_14 + 44) + f2_9;
                            if lwc1(s1_8 + 284) < -1e+01_f32 {
                                swc1(s1_8 + 284, -1e+01_f32);
                            }
                            if 3e+01_f32 < lwc1(s1_8 + 284) {
                                swc1(s1_8 + 284, 3e+01_f32);
                            }
                            if f4_4 == f3_7 {
                                f0_44 = lwc1(s1_8 + 312);
                                break 'b61;
                            }
                            f0_41 = f4_4;
                            if !(f4_4 < f3_7) {
                                f0_41 = f4_4 - 3.6e+02_f32;
                            }
                            'b76: {
                                f1_19 = f3_7 + (3.6e+02_f32 - (f3_7 - f0_41)) * 0.5_f32;
                                f0_42 = lwc1(s1_8 + 276);
                                if !(f3_7 < f1_19) {
                                    if f3_7 < f0_42 {
                                        swc1(s1_8 + 276, f3_7);
                                        break 'b76;
                                    }
                                } else if !(f3_7 < f0_42) {
                                    break 'b76;
                                }
                                if f0_42 < f1_19 {
                                    swc1(s1_8 + 276, f3_7);
                                }
                            }
                            f0_43 = lwc1(s1_8 + 276);
                            if !(f1_19 < f4_4) {
                                if f0_43 < f4_4 {
                                    swc1(s1_8 + 276, f4_4);
                                    break 'b62;
                                }
                            } else if !(f0_43 < f4_4) {
                                f0_44 = lwc1(s1_8 + 312);
                                break 'b61;
                            }
                            if !(f1_19 < f0_43) {
                                f0_44 = lwc1(s1_8 + 312);
                                break 'b61;
                            }
                            swc1(s1_8 + 276, f4_4);
                        }
                    }
                }
                f0_44 = lwc1(s1_8 + 312);
            }
            f1_20 = lwc1(s1_8 + 324) + f0_44 * 0.5_f32 * lwc1(g::updateframesf);
            swc1(s1_8 + 324, f1_20);
            if 3.6e+02_f32 <= f1_20 {
                sw(s1_8 + 324, 0);
            }
            f1_21 = lwc1(s1_8 + 328) + lwc1(s1_8 + 316) * 0.5_f32 * lwc1(g::updateframesf);
            swc1(s1_8 + 328, f1_21);
            if 3.6e+02_f32 < f1_21 {
                sw(s1_8 + 328, 0);
            }
            f3_8 = lwc1(g::updateframesf);
            if !((f3_8 * 4.0_f32 / 6e+01_f32) < lwc1(s1_8 + 376)) {
                f2_11 = lwc1(s1_8 + 320);
            } else {
                f1_22 = lwc1(s1_8 + 320);
                if !(f1_22 < 1.0_f32) {
                    f2_10 = lwc1(s1_8 + 312);
                } else {
                    swc1(s1_8 + 320, f1_22 + f3_8 * lwc1(0x3ad060));
                    f2_10 = lwc1(s1_8 + 312);
                }
                f0_46 = lwc1(g::updateframesf);
                if f2_10 < 3.0_f32 {
                    swc1(s1_8 + 312, f2_10 + f0_46 * lwc1(0x3ad064));
                }
                f2_11 = lwc1(s1_8 + 320);
            }
            f0_47 = lwc1(g::updateframesf);
            if 0.5_f32 < f2_11 {
                swc1(s1_8 + 320, f2_11 - f0_47 * lwc1(0x3ad068));
            }
            f2_12 = lwc1(s1_8 + 312);
            f0_48 = lwc1(g::updateframesf);
            if 1.0_f32 < f2_12 {
                swc1(s1_8 + 312, f2_12 - f0_48 * lwc1(0x3ad06c));
            }
            'b86: {
                swc1(s1_8 + 316, lwc1(s1_8 + 312) * lwc1(0x3ad070));
                if a0_45 == 0 {
                    if lw(g::propedit) != 0 {
                        f0_51 = lwc1(s1_8 + 332);
                        break 'b86;
                    }
                    f21_3 = lwc1(0x3ad074);
                    f12_12 = lwc1(s1_8 + 324) * f21_3 / 1.8e+02_f32;
                    f20_4 = lwc1(s1_8 + 320) * 0.5_f32;
                    f0_49 = sinf(f12_12);
                    s0_7 = lw(g::currentplayer);
                    swc1(s1_8 + 332, f20_4 * f0_49);
                    f12_13 = lwc1(s0_7 + 328) * f21_3 / 1.8e+02_f32;
                    f0_50 = sinf(f12_13);
                    s1_8 = lw(g::currentplayer);
                    swc1(s0_7 + 336, lwc1(s0_7 + 320) * f0_50);
                }
                f0_51 = lwc1(s1_8 + 332);
            }
            f22_2 = lwc1(0x3ad078);
            f12_14 = (lwc1(s1_8 + 276) + f0_51) * f22_2 / 1.8e+02_f32;
            f0_52 = cosf(f12_14);
            v0_44 = lw(g::currentplayer);
            f12_15 = (lwc1(v0_44 + 276) + lwc1(v0_44 + 332)) * f22_2 / 1.8e+02_f32;
            f0_53 = sinf(f12_15);
            v0_45 = lw(g::currentplayer);
            f12_16 = (lwc1(v0_45 + 284) + lwc1(v0_45 + 336)) * f22_2 / 1.8e+02_f32;
            f0_54 = cosf(f12_16);
            v0_46 = lw(g::currentplayer);
            f12_17 = (lwc1(v0_46 + 284) + lwc1(v0_46 + 336)) * f22_2 / 1.8e+02_f32;
            f0_55 = sinf(f12_17);
            v0_47 = lw(g::currentplayer);
            f12_18 = lwc1(v0_47 + 276);
            swc1(v0_47 + 192, f0_55);
            swc1(v0_47 + 188, f0_54 * f0_53);
            f12_19 = f12_18 * f22_2 / 1.8e+02_f32;
            swc1(v0_47 + 196, f0_54 * f0_52);
            f0_56 = cosf(f12_19);
            f12_20 = lwc1(lw(g::currentplayer) + 276) * f22_2 / 1.8e+02_f32;
            f0_57 = sinf(f12_20);
            f12_21 = lwc1(lw(g::currentplayer) + 284) * f22_2 / 1.8e+02_f32;
            cosf(f12_21);
            f12_22 = lwc1(lw(g::currentplayer) + 284) * f22_2 / 1.8e+02_f32;
            sinf(f12_22);
            v0_48 = lw(g::currentplayer);
            f5_3 = lwc1(0x3ad07c);
            f2_13 = lwc1(v0_48 + 188);
            f4_5 = lwc1(v0_48 + 228);
            f0_58 = lwc1(v0_48 + 816) * f5_3 * lwc1(v0_48 + 252);
            f8_1 = lwc1(g::updateframesf);
            f1_23 = lwc1(v0_48 + 812) * f5_3 * lwc1(v0_48 + 256) * lwc1(v0_48 + 240);
            swc1(v0_48 + 212, -f0_56);
            sw(v0_48 + 216, 0);
            swc1(v0_48 + 220, f0_57);
            f7_2 = f8_1 * lwc1(0x3ad080);
            swc1(s3_1 + 2976, f2_13 + f1_23 + f0_58 * f4_5);
            swc1(s3_1 + 2980, lwc1(v0_48 + 192) + lwc1(v0_48 + 812) * f5_3 * lwc1(v0_48 + 256) * lwc1(v0_48 + 244) + lwc1(v0_48 + 816) * f5_3 * lwc1(v0_48 + 252) * lwc1(v0_48 + 232));
            swc1(s3_1 + 2984, lwc1(v0_48 + 196) + lwc1(v0_48 + 812) * f5_3 * lwc1(v0_48 + 256) * lwc1(v0_48 + 248) + lwc1(v0_48 + 816) * f5_3 * lwc1(v0_48 + 252) * lwc1(v0_48 + 236));
            f1_24 = lwc1(v0_48 + 1784);
            f0_59 = f1_24 + f7_2;
            if f0_59 < f25_1 {
                f25_1 = f0_59;
            }
            f0_60 = f1_24 - f7_2;
            if f25_1 < f0_60 {
                f25_1 = f0_60;
            }
            f3_9 = lwc1(0x3ad084);
            f2_14 = lwc1(v0_48 + 1788);
            f1_25 = f8_1 * f3_9;
            f0_61 = f2_14 + f1_25;
            if f0_61 < f29_1 {
                f29_1 = f0_61;
            }
            f0_62 = f2_14 - f1_25;
            if f29_1 < f0_62 {
                f29_1 = f0_62;
            }
            f20_5 = 0.0_f32;
            f0_63 = f29_1 * lwc1(0x3ad088);
            swc1(v0_48 + 1788, f29_1);
            f24_2 = f25_1 * f3_9 * f8_1;
            swc1(v0_48 + 1784, f25_1);
            f12_23 = f20_5;
            f23_4 = f0_63 * f8_1;
            if !(f20_5 < f24_2) {
                f20_5 = -f24_2;
            } else {
                f12_23 = f24_2;
            }
            a1_24 = lw(s3_1);
            a0_46 = 47;
            StatsAdd(a0_46, a1_24, f12_23);
            a1_25 = lw(s3_1);
            f12_24 = f20_5;
            a0_47 = 48;
            StatsAdd(a0_47, a1_25, f12_24);
            f12_25 = f23_4;
            v0_49 = fptodp(f12_25);
            a0_48 = v0_49;
            a1_26 = 0;
            v0_50 = dpcmp(a0_48, a1_26 as i64);
            a1_27 = lw(s3_1);
            if v0_50 >= 0 {
                a0_50 = 50;
                f12_27 = f23_4;
                StatsAdd(a0_50, a1_27, f12_27);
                s1_9 = lw(g::currentplayer);
            } else {
                f12_26 = -f23_4;
                a0_49 = 50;
                StatsAdd(a0_49, a1_27, f12_26);
                s1_9 = lw(g::currentplayer);
            }
            a5_1 = s1_9;
            if lw(s1_9 + 296) != 2 {
                swc1(s3_1 + 2956, lwc1(s3_1 + 0x1198));
            }
            v1_15 = lw(s1_9 + 296);
            v0_51 = ((v1_15 as u32) < (8 as u32)) as i32;
            if v1_15 != 4 {
                f1_26 = lwc1(s1_9 + 192);
                f2_15 = lwc1(s1_9 + 196);
                swc1(s1_9 + 1796, lwc1(s1_9 + 188));
                swc1(s1_9 + 1800, f1_26);
                swc1(s1_9 + 1804, f2_15);
                sw(s1_9 + 1792, 0);
                v1_15 = lw(s1_9 + 296);
                v0_51 = ((v1_15 as u32) < (8 as u32)) as i32;
            }
            'b96: {
                if v0_51 != 0 {
                    'b97: {
                        'b98: {
                            match (v1_15 << 2) >> 2 {
                                0 => {
                                    f28_1 = 0.0_f32;
                                    s2_5 = 0;
                                    a0_68 = lw(a5_1 + 824);
                                    f30_1 = f28_1;
                                    if a0_68 != 0 {
                                        a0_69 = a0_68 + 20;
                                        if (lhu(a0_68 + 6) & 2) != 0 {
                                            v1_21 = sp + 176;
                                            v0_64 = sp + 160;
                                            a1_42 = 2;
                                            'l100: loop {
                                                a1_42 = a1_42 - 1;
                                                swc1(v0_64, lwc1(a0_69) - lwc1(a0_69 - 12));
                                                v0_64 = v0_64 + 4;
                                                f0_81 = lwc1(a0_69 + 12);
                                                f1_45 = lwc1(a0_69);
                                                a0_69 = a0_69 + 4;
                                                swc1(v1_21, f0_81 - f1_45);
                                                v1_21 = v1_21 + 4;
                                                if a1_42 < 0 {
                                                    break;
                                                }
                                            }
                                            f7_3 = lwc1(sp + 164);
                                            f1_46 = lwc1(sp + 184);
                                            f5_5 = lwc1(sp + 180);
                                            f0_82 = lwc1(sp + 168);
                                            f4_10 = lwc1(sp + 176);
                                            f2_27 = lwc1(sp + 160);
                                            f22_3 = f7_3 * f1_46 - f5_5 * f0_82;
                                            f21_4 = f0_82 * f4_10 - f1_46 * f2_27;
                                            f20_8 = f2_27 * f5_5 - f4_10 * f7_3;
                                            f2_28 = (f22_3 * f22_3 + f21_4 * f21_4 + f20_8 * f20_8).abs().sqrt();
                                            if f21_4 < f2_28 * lwc1(0x3ad08c) {
                                                f28_1 = f20_8 / f2_28;
                                                f30_1 = f22_3 / f2_28;
                                                s2_5 = 1;
                                            }
                                        }
                                    }
                                    f6_3 = lwc1(g::updateframesf);
                                    if s2_5 == 0 {
                                        f1_47 = lwc1(s1_9 + 140) + f0_57 * f24_2;
                                        swc1(sp + 144, f1_47);
                                        f3_18 = lwc1(s1_9 + 144);
                                        swc1(sp + 148, f3_18);
                                        f2_29 = lwc1(s1_9 + 148) + f0_56 * f24_2;
                                        swc1(sp + 152, f2_29);
                                        swc1(sp + 144, f1_47 + lwc1(s1_9 + 212) * f23_4);
                                        swc1(sp + 148, f3_18 + lwc1(s1_9 + 216) * f23_4);
                                        swc1(sp + 152, f2_29 + lwc1(s1_9 + 220) * f23_4);
                                    } else {
                                        swc1(sp + 144, lwc1(s1_9 + 140) + f6_3 * f30_1 * 8.0_f32 / 6e+01_f32);
                                        swc1(sp + 148, lwc1(s1_9 + 144) - 0.5_f32);
                                        swc1(sp + 152, lwc1(s1_9 + 148) + f6_3 * f28_1 * 8.0_f32 / 6e+01_f32);
                                    }
                                    'b103: {
                                        a2_17 = s1_9 + 276;
                                        a1_43 = sp + 144;
                                        a0_70 = lw(s1_9);
                                        swc1(sp + 148, lwc1(sp + 148) - f6_3 * lwc1(0x3ad090));
                                        chrAddFloorMoveToPos(a0_70, a1_43, a2_17);
                                        v0_65 = lw(g::currentplayer);
                                        a2_18 = sp + 144;
                                        f20_9 = lwc1(s3_1 + 0x1198);
                                        a4_8 = 1;
                                        a0_71 = lw(v0_65);
                                        a3_10 = v0_65 + 824;
                                        f12_40 = lwc1(v0_65 + 104);
                                        a1_44 = v0_65 + 140;
                                        f13_14 = lwc1(v0_65 + 132);
                                        v0_66 = moveTestDoSlide(a0_71, a1_44, a2_18, a3_10, a4_8, f12_40, f13_14);
                                        f2_30 = lwc1(s3_1 + 0x1198);
                                        f20_10 = f20_9 - f2_30;
                                        fp_1 = v0_66;
                                        if !(0.0_f32 <= f20_10) {
                                            s1_9 = lw(g::currentplayer);
                                            if !(-f20_10 <= lwc1(0x3ad098)) {
                                                break 'b103;
                                            }
                                            swc1(sp + 148, f2_30);
                                        } else {
                                            if !(f20_10 <= lwc1(0x3ad094)) {
                                                s1_9 = lw(g::currentplayer);
                                                break 'b103;
                                            }
                                            swc1(sp + 148, f2_30);
                                        }
                                        fp_1 = fp_1 | 1;
                                        s1_9 = lw(g::currentplayer);
                                        swc1(s1_9 + 128, lwc1(s1_9 + 128) + f20_10);
                                    }
                                    if (fp_1 & 1) != 0 {
                                        f3_20 = lwc1(s3_1 + 2864);
                                    } else {
                                        sw(s1_9 + 296, 2);
                                        if s2_5 == 0 {
                                            f4_11 = lwc1(g::updateframesf);
                                            f3_19 = (f0_56 * f24_2 + lwc1(s1_9 + 220) * f23_4) / f4_11;
                                            swc1(s1_9 + 856, (f0_57 * f24_2 + lwc1(s1_9 + 212) * f23_4) / f4_11);
                                            swc1(s1_9 + 864, f3_19);
                                            sw(s3_1 + 2864, 0);
                                        } else {
                                            f1_48 = -lwc1(s1_9 + 376) / lwc1(s1_9 + 340).to_bits() as i32 as f32;
                                            swc1(s1_9 + 864, f28_1 * 8.0_f32 / 6e+01_f32);
                                            swc1(s1_9 + 856, f30_1 * 8.0_f32 / 6e+01_f32);
                                            swc1(s3_1 + 2864, f1_48);
                                        }
                                        f3_20 = lwc1(s3_1 + 2864);
                                    }
                                    if !(f3_20 < 0.0_f32) {
                                        f2_31 = lwc1(s1_9 + 124);
                                        if !(f2_31 < 0.0_f32) {
                                            f0_83 = lwc1(g::updateframesf);
                                            if !(0.0_f32 < f2_31) {
                                                break 'b98;
                                            }
                                            f0_84 = f2_31 - f0_83 * lwc1(0x3ad0a8);
                                            swc1(s1_9 + 124, f0_84);
                                            if !(f0_84 < 0.0_f32) {
                                                break 'b98;
                                            }
                                            swc1(s1_9 + 124, 0.0_f32);
                                            break 'b98;
                                        }
                                    }
                                    f0_85 = lwc1(0x3ad09c);
                                    if !(f3_20 < f0_85) {
                                        f0_86 = lwc1(g::updateframesf);
                                    } else {
                                        swc1(s3_1 + 2864, f0_85);
                                        f3_20 = f0_85;
                                        f0_86 = lwc1(g::updateframesf);
                                    }
                                    f1_49 = f3_20 + f0_86 * lwc1(0x3ad0a0);
                                    swc1(s3_1 + 2864, f1_49);
                                    f2_32 = lwc1(s1_9 + 124) + f1_49 * f0_86;
                                    swc1(s1_9 + 124, f2_32);
                                    if 0.0_f32 < f2_32 {
                                        swc1(s1_9 + 124, 0.0_f32);
                                    }
                                    f1_50 = lwc1(s1_9 + 116);
                                    f2_33 = lwc1(0x3ad0a4);
                                    f3_21 = lwc1(sp + 144);
                                    if !((f1_50 + lwc1(s1_9 + 124)) < f2_33) {
                                        break 'b97;
                                    }
                                    swc1(s1_9 + 124, f2_33 - f1_50);
                                }
                                1 => {
                                    f2_24 = lwc1(a5_1 + 300);
                                    swc1(sp + 160, f2_24);
                                    f0_77 = lwc1(a5_1 + 304);
                                    swc1(sp + 164, f0_77);
                                    f1_41 = lwc1(a5_1 + 308);
                                    swc1(sp + 168, f1_41);
                                    f1_42 = (f2_24 * f2_24 + f0_77 * f0_77 + f1_41 * f1_41).abs().sqrt();
                                    f5_4 = lwc1(g::updateframesf);
                                    f0_78 = f5_4 * lwc1(0x3ad0ac);
                                    f2_25 = f1_42;
                                    if f0_78 < f1_42 {
                                        f2_25 = f0_78;
                                    }
                                    f0_79 = f2_25 / f1_42;
                                    f4_9 = lwc1(sp + 160) * f0_79;
                                    f2_26 = lwc1(sp + 164) * f0_79;
                                    f3_17 = lwc1(sp + 168) * f0_79;
                                    swc1(sp + 160, f4_9);
                                    swc1(sp + 164, f2_26);
                                    swc1(sp + 168, f3_17);
                                    swc1(sp + 144, lwc1(s1_9 + 140) + f4_9);
                                    swc1(sp + 148, lwc1(s1_9 + 144) + f2_26);
                                    swc1(sp + 152, lwc1(s1_9 + 148) + f3_17);
                                    f1_43 = lwc1(s1_9 + 300) - f4_9;
                                    swc1(s1_9 + 300, f1_43);
                                    f1_44 = f1_43 - lwc1(sp + 160);
                                    swc1(s1_9 + 300, f1_44);
                                    swc1(s1_9 + 300, f1_44 - lwc1(sp + 160));
                                    if f0_79 == 1.0_f32 {
                                        f0_80 = lwc1(sp + 160);
                                        sw(s1_9 + 296, 2);
                                        swc1(s1_9 + 856, f0_80 / f5_4);
                                        swc1(s1_9 + 864, lwc1(sp + 168) / f5_4);
                                    }
                                    a3_9 = s1_9 + 824;
                                    a1_41 = s1_9 + 140;
                                    a0_67 = lw(s1_9);
                                    a2_16 = sp + 144;
                                    f12_39 = lwc1(s1_9 + 104);
                                    f13_13 = lwc1(s1_9 + 132);
                                    a4_7 = 1;
                                    swc1(s1_9 + 124, lwc1(s1_9 + 124) * lwc1(0x3ad0b0));
                                    v0_63 = moveTestDoSlide(a0_67, a1_41, a2_16, a3_9, a4_7, f12_39, f13_13);
                                    s1_9 = lw(g::currentplayer);
                                    fp_1 = v0_63;
                                }
                                2 => {
                                    f2_21 = lwc1(0x3ad0b8);
                                    f1_35 = lwc1(s3_1 + 2864) - lwc1(g::updateframesf) * lwc1(0x3ad0b4);
                                    swc1(s3_1 + 2864, f1_35);
                                    if f1_35 < f2_21 {
                                        swc1(s3_1 + 2864, f2_21);
                                    }
                                    a3_7 = a5_1 + 824;
                                    f2_22 = lwc1(g::updateframesf);
                                    a1_37 = a5_1 + 140;
                                    a2_14 = sp + 144;
                                    a0_64 = lw(a5_1);
                                    f3_15 = lwc1(0x3ad0bc);
                                    a4_5 = 1;
                                    swc1(sp + 144, lwc1(a5_1 + 140) + lwc1(a5_1 + 856) * f2_22);
                                    swc1(sp + 148, lwc1(a5_1 + 144) + lwc1(s3_1 + 2864) * f2_22);
                                    swc1(sp + 152, lwc1(a5_1 + 148) + lwc1(a5_1 + 864) * f2_22);
                                    f12_37 = lwc1(a5_1 + 104);
                                    f13_12 = lwc1(a5_1 + 132);
                                    swc1(a5_1 + 124, lwc1(a5_1 + 124) * f3_15);
                                    v0_61 = moveTestDoSlide(a0_64, a1_37, a2_14, a3_7, a4_5, f12_37, f13_12);
                                    v1_20 = lw(g::currentplayer);
                                    fp_1 = v0_61;
                                    f4_8 = lwc1(sp + 144);
                                    v0_62 = lw(v1_20 + 384);
                                    a0_65 = 52;
                                    swc1(v1_20 + 140, f4_8);
                                    a1_38 = lw(v0_62 + 352);
                                    f1_36 = lwc1(sp + 148);
                                    f3_16 = lwc1(v1_20 + 124);
                                    f0_76 = f1_36 + lwc1(v1_20 + 116);
                                    f2_23 = lwc1(g::updateframesf);
                                    sw(a1_38 + 2708, (lw(a1_38 + 2708) | 1) & -33);
                                    swc1(v1_20 + 144, f1_36);
                                    f1_37 = lwc1(sp + 152);
                                    f12_38 = f2_23 * lwc1(0x3ad0c0);
                                    a1_39 = lw(s3_1);
                                    swc1(v1_20 + 152, f4_8);
                                    swc1(v1_20 + 156, f0_76 + f3_16);
                                    swc1(v1_20 + 160, f1_37);
                                    swc1(v1_20 + 148, f1_37);
                                    StatsAdd(a0_65, a1_39, f12_38);
                                    if (fp_1 & 1) == 0 {
                                        s1_9 = lw(g::currentplayer);
                                        break 'b96;
                                    }
                                    'b113: {
                                        f1_38 = lwc1(s3_1 + 2956);
                                        s1_9 = lw(g::currentplayer);
                                        if !(f1_38 == lwc1(g::shakeangz_172 + 4)) {
                                            f1_39 = f1_38 - lwc1(s3_1 + 0x1198);
                                            if 2.0_f32 < f1_39 {
                                                if f1_39 < 1e+01_f32 {
                                                    swc1(s1_9 + 1780, lwc1(0x3ad0c4));
                                                    swc1(s1_9 + 1776, (f1_39 - 2.0_f32) / 12.0_f32);
                                                    break 'b113;
                                                }
                                            }
                                            s1_9 = lw(g::currentplayer);
                                            if 1e+01_f32 <= f1_39 {
                                                f1_40 = lwc1(0x3ad0c8);
                                                swc1(s1_9 + 1776, 1.0_f32);
                                                swc1(s1_9 + 1780, f1_40);
                                            }
                                        }
                                    }
                                    if !(lwc1(lw(s1_9 + 384) + 520) <= 0.0_f32) {
                                        sw(s1_9 + 296, 0);
                                        break 'b96;
                                    }
                                    a0_66 = s1_9;
                                    sw(s1_9 + 296, 4);
                                    a1_40 = 0;
                                    a2_15 = 0;
                                    a3_8 = 0;
                                    a4_6 = -1;
                                    a5_3 = 0;
                                    playerOnGroundDie(a0_66, a1_40, a2_15, a3_8, a4_6, a5_3);
                                    s1_9 = lw(g::currentplayer);
                                    break 'b96;
                                }
                                3 => {
                                    a0_63 = lw(a5_1);
                                    a3_6 = a5_1 + 824;
                                    f3_14 = lwc1(a5_1 + 140) + f0_57 * f24_2;
                                    f6_2 = lwc1(0x3ad0d0);
                                    a1_36 = a5_1 + 140;
                                    f4_7 = lwc1(g::updateframesf) * lwc1(0x3ad0cc);
                                    a2_13 = sp + 144;
                                    swc1(sp + 144, f3_14);
                                    a4_4 = 1;
                                    f1_33 = lwc1(a5_1 + 144);
                                    swc1(sp + 148, f1_33);
                                    f2_20 = lwc1(a5_1 + 148) + f0_56 * f24_2;
                                    swc1(sp + 152, f2_20);
                                    swc1(sp + 144, f3_14 + lwc1(a5_1 + 212) * f23_4);
                                    f1_34 = f1_33 + lwc1(a5_1 + 216) * f23_4;
                                    swc1(sp + 148, f1_34);
                                    f0_75 = lwc1(a5_1 + 220) * f23_4;
                                    swc1(sp + 148, f1_34 - f4_7);
                                    swc1(sp + 152, f2_20 + f0_75);
                                    f12_36 = lwc1(a5_1 + 104);
                                    f13_11 = lwc1(a5_1 + 132);
                                    swc1(a5_1 + 124, lwc1(a5_1 + 124) * f6_2);
                                    v0_60 = moveTestDoSlide(a0_63, a1_36, a2_13, a3_6, a4_4, f12_36, f13_11);
                                    fp_1 = v0_60;
                                    s1_9 = lw(g::currentplayer);
                                    if (fp_1 & 1) != 0 {
                                        sw(s1_9 + 296, 0);
                                    }
                                }
                                4 => {
                                    v1_16 = lw(a5_1 + 384);
                                    f3_13 = lwc1(0x3ad0d4);
                                    v0_54 = lw(v1_16 + 32);
                                    swc1(sp + 144, lwc1(v0_54 + 128));
                                    swc1(sp + 148, lwc1(v0_54 + 132));
                                    swc1(sp + 152, lwc1(v0_54 + 136));
                                    f12_33 = lwc1(v1_16 + 76) * f3_13 / 1.8e+02_f32;
                                    a0_55 = sp + 208;
                                    matrixRotY(a0_55, f12_33);
                                    a0_56 = sp + 208;
                                    a1_32 = sp + 144;
                                    matrixVecRotAligned(&[a0_56 as i64, a1_32 as i64]);
                                    v1_17 = lw(g::currentplayer);
                                    a2_11 = sp + 144;
                                    a3_4 = 0;
                                    v0_55 = lw(v1_17 + 384);
                                    a4_2 = 1;
                                    a0_57 = lw(v1_17);
                                    a1_33 = v0_55 + 48;
                                    f1_32 = lwc1(sp + 148);
                                    f2_19 = lwc1(sp + 152);
                                    swc1(sp + 144, lwc1(sp + 144) + lwc1(v0_55 + 48));
                                    swc1(sp + 148, f1_32 + lwc1(v0_55 + 52));
                                    swc1(sp + 152, f2_19 + lwc1(v0_55 + 56));
                                    f13_9 = lwc1(v1_17 + 132);
                                    f12_34 = lwc1(v1_17 + 104);
                                    moveTestDoSlide(a0_57, a1_33, a2_11, a3_4, a4_2, f12_34, f13_9);
                                    v1_18 = lw(g::currentplayer);
                                    a0_58 = sp + 144;
                                    v0_56 = lw(v1_18 + 384);
                                    f12_35 = lwc1(0x3ad0d8);
                                    swc1(v0_56 + 48, lwc1(sp + 144));
                                    f13_10 = f12_35;
                                    swc1(v0_56 + 52, lwc1(sp + 148));
                                    swc1(v0_56 + 56, lwc1(sp + 152));
                                    swc1(sp + 148, lwc1(sp + 148) + (lwc1(v1_18 + 116) + lwc1(v1_18 + 124)));
                                    v0_57 = moveFindRoom(a0_58, f12_35, f13_10);
                                    s1_12 = lw(g::currentplayer);
                                    a0_59 = lw(s1_12 + 384);
                                    v1_19 = lw(a0_59 + 16);
                                    sw(a0_59 + 12, v0_57);
                                    sw(a0_59 + 16, v1_19 & -33);
                                    if lw(s3_1 + 0x11b0) != 0 {
                                        playerRespawn();
                                        s1_12 = lw(g::currentplayer);
                                    }
                                    'b116: {
                                        a0_60 = lw(s1_12);
                                        v0_58 = joyNewBut(a0_60);
                                        s1_13 = lw(g::currentplayer);
                                        if (v0_58 & lw(s1_13 + 36)) != 0 {
                                            a0_61 = lw(g::curgameparms + 72);
                                            if a0_61 != 7 {
                                                if a0_61 != 6 {
                                                    a1_34 = lw(s1_13 + 384);
                                                } else {
                                                    v0_59 = enemyGetNumBases();
                                                    s1_13 = lw(g::currentplayer);
                                                    if v0_59 <= 0 {
                                                        break 'b116;
                                                    }
                                                    a1_34 = lw(s1_13 + 384);
                                                }
                                                a0_62 = 5;
                                                a2_12 = -1;
                                                a3_5 = s3_1;
                                                a1_35 = a1_34 + 48;
                                                a4_3 = 0;
                                                specialfxNewSpawn(a0_62, a1_35, a2_12, a3_5, a4_3);
                                                s1_13 = lw(g::currentplayer);
                                                sw(s3_1 + 0x11b0, 1);
                                            }
                                        }
                                    }
                                    sw(lw(lw(s1_13 + 384) + 352) + 2708, 32);
                                    playerDeathView();
                                    s1_9 = lw(g::currentplayer);
                                    break 'b96;
                                }
                                5 => {
                                    break 'b96;
                                }
                                6 => {
                                    'b118: {
                                        'b119: {
                                            s0_8 = lw(s3_1 + 0x11b4);
                                            f13_7 = lwc1(a5_1 + 276);
                                            s2_4 = lw(s0_8 + 352);
                                            f12_30 = lwc1(s2_4 + 108);
                                            f0_66 = anglediff(f12_30, f13_7);
                                            a0_54 = s0_8;
                                            f12_31 = lwc1(s2_4 + 108);
                                            a1_31 = sp + 160;
                                            f20_6 = lwc1(g::updateframesf) * lwc1(0x3ad0dc);
                                            remoteGetStandPos(a0_54, a1_31, f12_31);
                                            if !(0.0_f32 <= f0_66) {
                                                s1_11 = lw(g::currentplayer);
                                                if !(-f0_66 < f20_6) {
                                                    break 'b119;
                                                }
                                            } else {
                                                s1_11 = lw(g::currentplayer);
                                                if !(f0_66 < f20_6) {
                                                    break 'b119;
                                                }
                                            }
                                            f0_67 = lwc1(s2_4 + 108);
                                            break 'b118;
                                        }
                                        f0_68 = lwc1(s1_11 + 276);
                                        if !(f0_66 < 0.0_f32) {
                                            f0_67 = f0_68 - f20_6;
                                        } else {
                                            f0_67 = f0_68 + f20_6;
                                        }
                                    }
                                    'b121: {
                                        'b122: {
                                            swc1(s1_11 + 276, f0_67);
                                            f13_8 = lwc1(s1_11 + 284);
                                            f12_32 = -lwc1(s2_4 + 112);
                                            f0_69 = anglediff(f12_32, f13_8);
                                            f20_7 = lwc1(g::updateframesf) * lwc1(0x3ad0e0);
                                            if !(0.0_f32 <= f0_69) {
                                                if !(-f0_69 < f20_7) {
                                                    break 'b122;
                                                }
                                                f0_70 = lwc1(s2_4 + 112);
                                            } else {
                                                if !(f0_69 < f20_7) {
                                                    break 'b122;
                                                }
                                                f0_70 = lwc1(s2_4 + 112);
                                            }
                                            s1_9 = lw(g::currentplayer);
                                            f0_71 = -f0_70;
                                            break 'b121;
                                        }
                                        s1_9 = lw(g::currentplayer);
                                        f0_72 = lwc1(s1_9 + 284);
                                        if !(f0_69 < 0.0_f32) {
                                            f0_71 = f0_72 - f20_7;
                                        } else {
                                            f0_71 = f0_72 + f20_7;
                                        }
                                    }
                                    swc1(s1_9 + 284, f0_71);
                                    f1_29 = lwc1(g::updateframesf);
                                    f2_18 = lwc1(0x3ad0e4);
                                    swc1(s1_9 + 140, lwc1(sp + 160));
                                    f3_11 = lwc1(0x3ad0e8);
                                    f0_73 = lwc1(s1_9 + 124) + f1_29 * f2_18;
                                    swc1(s1_9 + 144, lwc1(sp + 164));
                                    f1_30 = lwc1(sp + 168);
                                    swc1(s1_9 + 124, f0_73);
                                    swc1(s1_9 + 148, f1_30);
                                    if f3_11 < f0_73 {
                                        swc1(s1_9 + 124, f3_11);
                                    }
                                    f3_12 = lwc1(s1_9 + 124);
                                    f0_74 = lwc1(s1_9 + 144) + lwc1(s1_9 + 116);
                                    f1_31 = lwc1(s1_9 + 148);
                                    swc1(s1_9 + 152, lwc1(s1_9 + 140));
                                    swc1(s1_9 + 160, f1_31);
                                    swc1(s1_9 + 156, f0_74 + f3_12);
                                    break 'b96;
                                }
                                7 => {
                                    s1_10 = lw(s3_1 + 0x11b4);
                                    f12_28 = lwc1(a5_1 + 276);
                                    a1_28 = sp + 176;
                                    a0_51 = s1_10;
                                    remoteGetStandPos(a0_51, a1_28, f12_28);
                                    a2_10 = sp + 176;
                                    a3_3 = 0;
                                    v0_52 = lw(g::currentplayer);
                                    a5_2 = 1;
                                    a0_52 = lw(v0_52);
                                    a4_1 = v0_52 + 824;
                                    a1_29 = v0_52 + 140;
                                    swc1(sp + 180, lwc1(sp + 180) - 1.0_f32);
                                    f12_29 = lwc1(v0_52 + 104);
                                    f13_6 = lwc1(v0_52 + 132);
                                    v0_53 = moveTest(a0_52, a1_29, a2_10, a3_3, a4_1, a5_2, f12_29, f13_6);
                                    fp_1 = v0_53;
                                    a0_53 = s1_10;
                                    if (fp_1 & 2) == 0 {
                                        s1_9 = lw(g::currentplayer);
                                    } else {
                                        a1_30 = sp + 176;
                                        f0_64 = remoteGetStandAngle(a0_53, a1_30);
                                        s1_9 = lw(g::currentplayer);
                                        swc1(s1_9 + 276, f0_64);
                                    }
                                    f2_16 = lwc1(0x3ad0f0);
                                    f1_27 = lwc1(s1_9 + 124) + lwc1(g::updateframesf) * lwc1(0x3ad0ec);
                                    swc1(s1_9 + 124, f1_27);
                                    if f2_16 < f1_27 {
                                        swc1(s1_9 + 124, f2_16);
                                    }
                                    f4_6 = lwc1(sp + 176);
                                    f1_28 = lwc1(s1_9 + 116);
                                    swc1(s1_9 + 140, f4_6);
                                    f3_10 = lwc1(s1_9 + 124);
                                    f0_65 = lwc1(sp + 180);
                                    swc1(s1_9 + 144, f0_65);
                                    f2_17 = lwc1(sp + 184);
                                    swc1(s1_9 + 152, f4_6);
                                    swc1(s1_9 + 160, f2_17);
                                    swc1(s1_9 + 148, f2_17);
                                    swc1(s1_9 + 156, f0_65 + f1_28 + f3_10);
                                    break 'b96;
                                }
                                _ => unreachable!(),
                            }
                        }
                        f3_21 = lwc1(sp + 144);
                    }
                    v0_67 = lw(s1_9 + 384);
                    swc1(s1_9 + 140, f3_21);
                    f2_34 = lwc1(sp + 148);
                    v1_22 = lw(v0_67 + 352);
                    f0_87 = f2_34 + lwc1(s1_9 + 116);
                    f1_51 = lwc1(s1_9 + 124);
                    v0_68 = lw(v1_22 + 2708);
                    swc1(s1_9 + 144, f2_34);
                    f1_52 = lwc1(sp + 152);
                    sw(v1_22 + 2708, (v0_68 | 1) & -33);
                    swc1(s1_9 + 152, f3_21);
                    swc1(s1_9 + 156, f0_87 + f1_51);
                    swc1(s1_9 + 160, f1_52);
                    swc1(s1_9 + 148, f1_52);
                }
            }
            f2_35 = lwc1(s1_9 + 128);
            f12_41 = f25_1;
            if !(f2_35 == 0.0_f32) {
                v0_69 = lw(g::updateframesi);
                v1_23 = s1_9;
                if v0_69 > 0 {
                    f1_53 = f2_35;
                    'l129: loop {
                        v0_69 = v0_69 - 1;
                        f1_53 = f1_53 - f1_53 * 0.25_f32;
                        if v0_69 == 0 {
                            break;
                        }
                    }
                    swc1(v1_23 + 128, f1_53);
                }
                'b131: {
                    'b132: {
                        if 0.0_f32 < f2_35 {
                            if lwc1(s1_9 + 128) < 0.0_f32 {
                                sw(s1_9 + 128, 0);
                                break 'b132;
                            }
                        }
                        if !(f2_35 < 0.0_f32) {
                            f3_22 = lwc1(s1_9 + 156);
                            break 'b131;
                        }
                        if !(0.0_f32 < lwc1(s1_9 + 128)) {
                            f3_22 = lwc1(s1_9 + 156);
                            break 'b131;
                        }
                        sw(s1_9 + 128, 0);
                    }
                    f3_22 = lwc1(s1_9 + 156);
                }
                f2_36 = lwc1(s1_9 + 144);
                f4_12 = lwc1(0x3ad0f4);
                if !((f3_22 + lwc1(s1_9 + 128)) < f2_36 + f4_12) {
                    f0_88 = lwc1(s1_9 + 156);
                } else {
                    swc1(s1_9 + 128, f2_36 - f3_22 + f4_12);
                    f0_88 = lwc1(s1_9 + 156);
                }
                swc1(s1_9 + 156, f0_88 + lwc1(s1_9 + 128));
                f12_41 = f25_1;
            }
            f13_15 = f29_1;
            f14_6 = f31_1;
            playerselectanim(f12_41, f13_15, f14_6);
            s1_2 = lw(g::currentplayer);
            a3_11 = 0;
            v1_24 = 0;
            s0_9 = lw(s1_2 + 796);
            s0_10 = if s0_9 == 0 { 1 } else { s0_9 };
            swc1(sp + 272, lwc1(s1_2 + 152) - lwc1(s1_2 + 356));
            swc1(sp + 276, lwc1(s1_2 + 156) - lwc1(s1_2 + 360));
            swc1(sp + 280, lwc1(s1_2 + 160) - lwc1(s1_2 + 364));
            if s0_10 != 0 {
                'l136: loop {
                    a2_19 = s0_10;
                    a0_72 = s1_2 + 356;
                    a1_45 = sp + 272;
                    a4_9 = 0;
                    v0_70 = bgPortalCrossed(a0_72, a1_45, a2_19, a3_11, a4_9);
                    v1_24 = v0_70;
                    a3_11 = s0_10;
                    s0_10 = v1_24;
                    if v1_24 == a3_11 {
                        break;
                    }
                    s1_2 = lw(g::currentplayer);
                }
                s1_2 = lw(g::currentplayer);
            }
            'b138: {
                sw(s1_2 + 164, 0);
                f7_4 = lwc1(s1_2 + 196);
                f0_89 = lwc1(s1_2 + 164);
                f9_1 = lwc1(s1_2 + 192);
                f3_23 = lwc1(s1_2 + 188);
                f2_37 = f9_1 * f0_89;
                f5_6 = lwc1(s1_2 + 156);
                f4_13 = lwc1(s1_2 + 152) + f3_23;
                f6_4 = lwc1(s1_2 + 160);
                f1_54 = f7_4 - f2_37;
                f0_90 = f3_23 * f0_89 - f7_4 * f0_89;
                sw(s1_2 + 796, v1_24);
                f2_38 = f2_37 - f3_23;
                swc1(s1_2 + 176, f4_13);
                swc1(s1_2 + 168, 1.0_f32);
                swc1(s1_2 + 244, f0_90);
                sw(s1_2 + 172, 0);
                swc1(s1_2 + 240, f1_54);
                swc1(s1_2 + 248, f2_38);
                swc1(s1_2 + 180, f5_6 + f9_1);
                swc1(s1_2 + 184, f6_4 + f7_4);
                f1_55 = 1.0_f32 / (f1_54 * f1_54 + f0_90 * f0_90 + f2_38 * f2_38).abs().sqrt();
                f4_14 = lwc1(s1_2 + 196);
                f3_24 = lwc1(s1_2 + 188);
                f6_5 = lwc1(s1_2 + 192);
                f0_91 = lwc1(s1_2 + 248) * f1_55;
                f5_7 = lwc1(s1_2 + 240) * f1_55;
                f2_39 = lwc1(s1_2 + 244) * f1_55;
                swc1(s1_2 + 248, f0_91);
                swc1(s1_2 + 240, f5_7);
                swc1(s1_2 + 244, f2_39);
                f1_56 = f4_14 * f5_7 - f3_24 * f0_91;
                f0_92 = f6_5 * f0_91 - f4_14 * f2_39;
                swc1(s1_2 + 232, f1_56);
                swc1(s1_2 + 228, f0_92);
                f3_25 = f3_24 * f2_39 - f6_5 * f5_7;
                swc1(s1_2 + 236, f3_25);
                f1_57 = 1.0_f32 / (f0_92 * f0_92 + f1_56 * f1_56 + f3_25 * f3_25).abs().sqrt();
                v0_71 = lw(s1_2 + 296);
                f2_40 = lwc1(s1_2 + 228) * f1_57;
                f3_26 = lwc1(s1_2 + 232) * f1_57;
                swc1(s1_2 + 236, lwc1(s1_2 + 236) * f1_57);
                swc1(s1_2 + 228, f2_40);
                swc1(s1_2 + 232, f3_26);
                if v0_71 != 4 {
                    v1_25 = lw(s1_2 + 384);
                    v0_72 = lw(s1_2 + 796);
                    swc1(v1_25 + 48, lwc1(s1_2 + 140));
                    sw(v1_25 + 12, v0_72);
                    a0_73 = lw(s1_2 + 296);
                    swc1(v1_25 + 52, lwc1(s1_2 + 144));
                    swc1(v1_25 + 56, lwc1(s1_2 + 148));
                    if a0_73 != 4 {
                        if a0_73 == 7 {
                            v0_73 = lw(s1_2 + 800);
                            break 'b138;
                        }
                        swc1(v1_25 + 80, lwc1(s1_2 + 276));
                    }
                }
                v0_73 = lw(s1_2 + 800);
            }
            if v0_73 == 0 {
                v1_32 = lw(s1_2 + 296);
                break 'b8;
            }
            v1_26 = lw(s1_2 + 384);
            a0_74 = lw(s3_1 + 332);
            sw(v1_26 + 16, lw(v1_26 + 16) & -33);
            if a0_74 != 0 {
                sw(a0_74 + 16, lw(a0_74 + 16) & -33);
            }
            v1_27 = lw(s3_1 + 440);
            if v1_27 == 0 {
                v1_28 = lw(s3_1 + 0x11b4);
            } else {
                sw(v1_27 + 16, lw(v1_27 + 16) & -33);
                v1_28 = lw(s3_1 + 0x11b4);
            }
            if v1_28 == 0 {
                a0_75 = lw(s1_2 + 796);
            } else {
                sw(v1_28 + 16, lw(v1_28 + 16) & -33);
                a0_75 = lw(s1_2 + 796);
            }
            a5_4 = s1_2 + 844;
            a1_46 = s1_2 + 152;
            a4_10 = s1_2 + 832;
            a0_76 = if a0_75 == 0 { 1 } else { a0_75 };
            sw(sp, 0);
            a2_20 = s3_1 + 2976;
            sw(sp + 8, 0);
            a3_12 = 1;
            a6_1 = 0;
            a7_1 = 0;
            v0_74 = bgLineTest(a0_76, a1_46, a2_20, a3_12, a4_10, a5_4, a6_1, a7_1);
            s1_2 = lw(g::currentplayer);
            a0_77 = lw(s3_1 + 332);
            v1_29 = lw(s1_2 + 384);
            sw(s1_2 + 828, v0_74);
            sw(v1_29 + 16, lw(v1_29 + 16) | 32);
            if a0_77 != 0 {
                sw(a0_77 + 16, lw(a0_77 + 16) | 32);
            }
            v1_30 = lw(s3_1 + 440);
            if v1_30 == 0 {
                v1_31 = lw(s3_1 + 0x11b4);
            } else {
                sw(v1_30 + 16, lw(v1_30 + 16) | 32);
                v1_31 = lw(s3_1 + 0x11b4);
            }
            if v1_31 == 0 {
                v1_32 = lw(s1_2 + 296);
                break 'b8;
            }
            sw(v1_31 + 16, lw(v1_31 + 16) | 32);
        }
        v1_32 = lw(s1_2 + 296);
    }
    s0_11 = lw(s1_2 + 384);
    if v1_32 == 4 {
        sw(s0_11 + 336, -1);
    } else {
        f2_41 = lwc1(s3_1 + 2976);
        swc1(sp + 288, f2_41);
        f0_93 = lwc1(s3_1 + 2980);
        swc1(sp + 292, f0_93);
        f1_58 = lwc1(s3_1 + 2984);
        swc1(sp + 296, f1_58);
        f1_59 = 1.0_f32 / (f2_41 * f2_41 + f0_93 * f0_93 + f1_58 * f1_58).abs().sqrt();
        f2_42 = lwc1(sp + 288) * f1_59;
        f0_94 = lwc1(sp + 296) * f1_59;
        f3_27 = lwc1(sp + 292) * f1_59;
        swc1(sp + 288, f2_42);
        swc1(sp + 296, f0_94);
        swc1(sp + 292, f3_27);
        f12_42 = lwc1(sp + 292);
        f13_16 = (f2_42 * f2_42 + f0_94 * f0_94).abs().sqrt();
        f0_95 = atan2f(f12_42, f13_16);
        f21_5 = lwc1(0x3ad0f8);
        swc1(lw(s0_11 + 352) + 2968, f0_95 * 1.8e+02_f32 / f21_5);
        f12_43 = lwc1(sp + 288);
        f13_17 = lwc1(sp + 296);
        f0_96 = atan2f(f12_43, f13_17);
        v0_75 = lw(s0_11 + 352);
        f2_43 = f0_96 * 1.8e+02_f32 / f21_5;
        swc1(v0_75 + 2972, f2_43);
        if !(f2_43 < 0.0_f32) {
            s1_2 = lw(g::currentplayer);
        } else {
            s1_2 = lw(g::currentplayer);
            swc1(v0_75 + 2972, f2_43 + 3.6e+02_f32);
        }
    }
    if lw(g::invisible) == 0 {
        v1_33 = lw(s1_2 + 384);
        a0_79 = lw(v1_33 + 352);
        sw(a0_79 + 28, lw(a0_79 + 28) & -2);
    } else {
        a0_78 = lw(lw(s1_2 + 384) + 352);
        sw(a0_78 + 28, lw(a0_78 + 28) | 1);
    }
    'b147: {
        a1_47 = lw(g::updateframesi);
        if lwc1(0x3ad0fc) < lwc1(s1_2 + 1776) {
            if a1_47 == 0 {
                break 'b147;
            }
            if (lhu(lw(s1_2 + 24) + 2) & 48) != 48 {
                break 'b147;
            }
            a0_80 = lw(s1_2);
            v0_76 = joyGetShockLeft(a0_80);
            v1_34 = lw(g::currentplayer);
            s0_12 = (v0_76 as f32 + lwc1(v1_34 + 1776) * 128.0_f32) as i32;
            a0_81 = lw(v1_34);
            v0_77 = joyGetShockRight(a0_81);
            a0_82 = lw(s1_2);
            a2_21 = v0_77;
            a1_48 = s0_12;
            joySetShock(a0_82, a1_48, a2_21);
            s1_2 = lw(g::currentplayer);
        }
        a1_47 = lw(g::updateframesi);
    }
    v0_78 = a1_47;
    if a1_47 > 0 {
        f1_60 = lwc1(s1_2 + 1780);
        f0_97 = lwc1(s1_2 + 1776);
        v1_35 = s1_2;
        'l150: loop {
            f0_97 = f0_97 * f1_60;
            v0_78 = v0_78 - 1;
            if v0_78 == 0 {
                break;
            }
        }
        swc1(v1_35 + 1776, f0_97);
    }
    if lw(s1_2 + 388) == 0 {
        v0_81 = lw(s1_2 + 824);
    } else {
        'b153: {
            v0_79 = camGetMode();
            s1_2 = lw(g::currentplayer);
            if v0_79 == 0 {
                f0_99 = lwc1(s1_2 + 152);
            } else {
                v1_36 = lw(s1_2 + 904);
                if v1_36 != 0 {
                    v0_80 = lw(s1_2 + 388);
                    swc1(v0_80 + 48, lwc1(v1_36 + 48));
                    swc1(v0_80 + 52, lwc1(v1_36 + 52));
                    f0_98 = lwc1(v1_36 + 56);
                    break 'b153;
                }
                f0_99 = lwc1(s1_2 + 152);
            }
            v0_80 = lw(s1_2 + 388);
            swc1(v0_80 + 48, f0_99);
            swc1(v0_80 + 52, lwc1(s1_2 + 156));
            f0_98 = lwc1(s1_2 + 160);
        }
        swc1(v0_80 + 56, f0_98);
        v0_81 = lw(s1_2 + 824);
    }
    if v0_81 != 0 {
        bc_2 = (lhu(v0_81 + 6) & 4) == 0;
        v0_82 = fp_1 & 1;
        if !bc_2 {
            if v0_82 != 0 {
                a0_83 = lw(s1_2 + 384);
                propKill(a0_83);
            }
        }
    }
    if (lw(g::curgameparms + 80) & 32) != 0 {
        if lw(g::curgameparms + 68) == 17 {
            v0_83 = lw(g::currentplayer);
            if (fp_1 & 1) != 0 {
                if lwc1(v0_83 + 144) < 2.0_f32 {
                    a0_84 = lw(v0_83 + 384);
                    propKill(a0_84);
                }
            }
        }
    }
    return;
}

/// Tick of the bullets in flight (@ 0x28a8e8): for every bullet of the array (276 bytes each) move it by its kind (straight shots, grenades, rockets, mines and the others), test what it hits, and do what the hit does (damage, sparks, smoke, decals, explosions, ricochets) or let it die.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn bulletTick() {
    let frame = Frame::new(448);
    let sp = frame.sp();
    let mut a3_1: i32 = 0;
    let mut s7_1: i32 = 0;
    let mut f3_1: f32 = 0.0;
    let mut s2_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut f1_1: f32 = 0.0;
    let mut fcc_1: bool = false;
    let mut v0_1: i32 = 0;
    let mut v1_3: i32 = 0;
    let mut f0_1: f32 = 0.0;
    let mut f0_2: f32 = 0.0;
    let mut f0_3: f32 = 0.0;
    let mut a1_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut f0_4: f32 = 0.0;
    let mut a0_2: i32 = 0;
    let mut a4_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a3_3: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut f0_5: f32 = 0.0;
    let mut f12_1: f32 = 0.0;
    let mut a0_5: i32 = 0;
    let mut a3_4: i32 = 0;
    let mut a2_4: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut a4_3: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut a6_1: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut f2_1: f32 = 0.0;
    let mut f3_2: f32 = 0.0;
    let mut f4_1: f32 = 0.0;
    let mut f5_1: f32 = 0.0;
    let mut f6_1: f32 = 0.0;
    let mut f8_1: f32 = 0.0;
    let mut f10_1: f32 = 0.0;
    let mut f3_3: f32 = 0.0;
    let mut f2_2: f32 = 0.0;
    let mut f4_2: f32 = 0.0;
    let mut f5_2: f32 = 0.0;
    let mut f6_2: f32 = 0.0;
    let mut f1_2: f32 = 0.0;
    let mut f2_3: f32 = 0.0;
    let mut s1_2: i64 = 0;
    let mut f12_2: f32 = 0.0;
    let mut v0_3: i64 = 0;
    let mut s3_1: i64 = 0;
    let mut f12_3: f32 = 0.0;
    let mut v0_4: i64 = 0;
    let mut a0_6: i64 = 0;
    let mut a1_6: i64 = 0;
    let mut v0_5: i64 = 0;
    let mut a0_7: i64 = 0;
    let mut a1_7: i64 = 0;
    let mut v0_6: i64 = 0;
    let mut a0_8: i64 = 0;
    let mut a1_8: i64 = 0;
    let mut v0_7: i64 = 0;
    let mut s0_1: i32 = 0;
    let mut a1_9: i64 = 0;
    let mut a0_9: i64 = 0;
    let mut v0_8: i64 = 0;
    let mut a0_10: i64 = 0;
    let mut f0_6: f32 = 0.0;
    let mut f12_4: f32 = 0.0;
    let mut v0_9: i64 = 0;
    let mut f1_3: f32 = 0.0;
    let mut f0_7: f32 = 0.0;
    let mut f12_5: f32 = 0.0;
    let mut v0_10: i64 = 0;
    let mut a0_11: i64 = 0;
    let mut a1_10: i64 = 0;
    let mut v0_11: i64 = 0;
    let mut a0_12: i64 = 0;
    let mut a1_11: i64 = 0;
    let mut v0_12: i64 = 0;
    let mut a0_13: i64 = 0;
    let mut a1_12: i64 = 0;
    let mut v0_13: i64 = 0;
    let mut s0_2: i32 = 0;
    let mut a1_13: i64 = 0;
    let mut a0_14: i32 = 0;
    let mut v0_14: i64 = 0;
    let mut a0_15: i64 = 0;
    let mut f0_8: f32 = 0.0;
    let mut a1_14: i32 = 0;
    let mut a0_16: i32 = 0;
    let mut a1_15: i32 = 0;
    let mut f5_3: f32 = 0.0;
    let mut f6_3: f32 = 0.0;
    let mut v0_15: i32 = 0;
    let mut f1_4: f32 = 0.0;
    let mut f4_3: f32 = 0.0;
    let mut f0_9: f32 = 0.0;
    let mut f7_1: f32 = 0.0;
    let mut f5_4: f32 = 0.0;
    let mut f1_5: f32 = 0.0;
    let mut f4_4: f32 = 0.0;
    let mut f6_4: f32 = 0.0;
    let mut f3_4: f32 = 0.0;
    let mut f1_6: f32 = 0.0;
    let mut f0_10: f32 = 0.0;
    let mut a1_16: i32 = 0;
    let mut f2_4: f32 = 0.0;
    let mut f2_5: f32 = 0.0;
    let mut f2_6: f32 = 0.0;
    let mut f0_11: f32 = 0.0;
    let mut f1_7: f32 = 0.0;
    let mut f0_12: f32 = 0.0;
    let mut f0_13: f32 = 0.0;
    let mut f1_8: f32 = 0.0;
    let mut f0_14: f32 = 0.0;
    let mut f0_15: f32 = 0.0;
    let mut a0_17: i32 = 0;
    let mut v0_16: i32 = 0;
    let mut f1_9: f32 = 0.0;
    let mut f0_16: f32 = 0.0;
    let mut f0_17: f32 = 0.0;
    let mut s0_3: i32 = 0;
    let mut v0_17: i64 = 0;
    let mut a1_17: i32 = 0;
    let mut a0_18: i32 = 0;
    let mut a3_5: i32 = 0;
    let mut a0_19: i32 = 0;
    let mut a6_2: i32 = 0;
    let mut a1_18: i32 = 0;
    let mut a2_5: i32 = 0;
    let mut a4_4: i32 = 0;
    let mut a5_2: i32 = 0;
    let mut a7_2: i32 = 0;
    let mut a0_20: i32 = 0;
    let mut f12_6: f32 = 0.0;
    let mut a0_21: i32 = 0;
    let mut f12_7: f32 = 0.0;
    let mut f12_8: f32 = 0.0;
    let mut a0_22: i32 = 0;
    let mut a1_19: i32 = 0;
    let mut a2_6: i32 = 0;
    let mut a3_6: i32 = 0;
    let mut a0_23: i32 = 0;
    let mut a3_7: i32 = 0;
    let mut a1_20: i32 = 0;
    let mut a2_7: i32 = 0;
    let mut a4_5: i32 = 0;
    let mut a0_24: i32 = 0;
    let mut v1_4: i32 = 0;
    let mut f0_18: f32 = 0.0;
    let mut f0_19: f32 = 0.0;
    let mut f1_10: f32 = 0.0;
    let mut v0_18: i32 = 0;
    let mut f3_5: f32 = 0.0;
    let mut f1_11: f32 = 0.0;
    let mut f2_7: f32 = 0.0;
    let mut a4_6: i32 = 0;
    let mut a0_25: i32 = 0;
    let mut a1_21: i32 = 0;
    let mut a2_8: i32 = 0;
    let mut a3_8: i32 = 0;
    let mut v0_19: i32 = 0;
    let mut a1_22: i32 = 0;
    let mut a0_26: i32 = 0;
    let mut f0_20: f32 = 0.0;
    let mut f2_8: f32 = 0.0;
    let mut f4_5: f32 = 0.0;
    let mut f6_5: f32 = 0.0;
    let mut f5_5: f32 = 0.0;
    let mut f7_2: f32 = 0.0;
    let mut f9_1: f32 = 0.0;
    let mut f3_6: f32 = 0.0;
    let mut f2_9: f32 = 0.0;
    let mut f4_6: f32 = 0.0;
    let mut f6_6: f32 = 0.0;
    let mut f5_6: f32 = 0.0;
    let mut a1_23: i32 = 0;
    let mut f5_7: f32 = 0.0;
    let mut f0_21: f32 = 0.0;
    let mut f4_7: f32 = 0.0;
    let mut f2_10: f32 = 0.0;
    let mut a0_27: i32 = 0;
    let mut a0_28: i32 = 0;
    let mut a1_24: i32 = 0;
    let mut a0_29: i32 = 0;
    let mut a1_25: i32 = 0;
    let mut a0_30: i32 = 0;
    let mut a1_26: i32 = 0;
    let mut a0_31: i32 = 0;
    let mut a1_27: i32 = 0;
    let mut f0_22: f32 = 0.0;
    let mut f7_3: f32 = 0.0;
    let mut f5_8: f32 = 0.0;
    let mut f1_12: f32 = 0.0;
    let mut f4_8: f32 = 0.0;
    let mut f6_7: f32 = 0.0;
    let mut f3_7: f32 = 0.0;
    let mut f1_13: f32 = 0.0;
    let mut f0_23: f32 = 0.0;
    let mut f2_11: f32 = 0.0;
    let mut f2_12: f32 = 0.0;
    let mut f2_13: f32 = 0.0;
    let mut f0_24: f32 = 0.0;
    let mut f1_14: f32 = 0.0;
    let mut f0_25: f32 = 0.0;
    let mut f0_26: f32 = 0.0;
    let mut f1_15: f32 = 0.0;
    let mut f0_27: f32 = 0.0;
    let mut f0_28: f32 = 0.0;
    let mut a0_32: i32 = 0;
    let mut f12_9: f32 = 0.0;
    let mut a0_33: i32 = 0;
    let mut a1_28: i32 = 0;
    let mut a0_34: i32 = 0;
    let mut a1_29: i32 = 0;
    let mut a2_9: i32 = 0;
    let mut a3_9: i32 = 0;
    let mut v0_20: i32 = 0;
    let mut f12_10: f32 = 0.0;
    let mut a0_35: i32 = 0;
    let mut a1_30: i32 = 0;
    let mut a1_31: i32 = 0;
    let mut a1_32: i32 = 0;
    let mut a2_10: i32 = 0;
    let mut a3_10: i32 = 0;
    let mut v0_21: i32 = 0;
    let mut f0_29: f32 = 0.0;
    let mut v1_5: i32 = 0;
    let mut f0_30: f32 = 0.0;
    let mut s0_4: i32 = 0;
    let mut v0_22: i64 = 0;
    let mut a1_33: i32 = 0;
    let mut a0_36: i32 = 0;
    let mut a3_11: i32 = 0;
    let mut a0_37: i32 = 0;
    let mut a6_3: i32 = 0;
    let mut a1_34: i32 = 0;
    let mut a2_11: i32 = 0;
    let mut a4_7: i32 = 0;
    let mut a5_3: i32 = 0;
    let mut a7_3: i32 = 0;
    let mut a0_38: i32 = 0;
    let mut a1_35: i32 = 0;
    let mut a0_39: i32 = 0;
    let mut a3_12: i32 = 0;
    let mut a1_36: i32 = 0;
    let mut a2_12: i32 = 0;
    let mut a4_8: i32 = 0;
    let mut a0_40: i32 = 0;
    let mut fcc_2: bool = false;
    let mut f0_31: f32 = 0.0;
    let mut f0_32: f32 = 0.0;
    let mut s0_5: i32 = 0;
    let mut f0_33: f32 = 0.0;
    let mut f1_16: f32 = 0.0;
    let mut f1_17: f32 = 0.0;
    let mut v1_6: i32 = 0;
    let mut a0_41: i32 = 0;
    let mut a1_37: i32 = 0;
    let mut a0_42: i32 = 0;
    let mut a1_38: i32 = 0;
    let mut a0_43: i32 = 0;
    let mut a1_39: i32 = 0;
    let mut f1_18: f32 = 0.0;
    let mut a0_44: i32 = 0;
    let mut f15_1: f32 = 0.0;
    let mut f14_1: f32 = 0.0;
    let mut f16_1: f32 = 0.0;
    let mut f12_11: f32 = 0.0;
    let mut f17_1: f32 = 0.0;
    let mut f13_1: f32 = 0.0;
    let mut a0_45: i32 = 0;
    let mut f15_2: f32 = 0.0;
    let mut a0_46: i32 = 0;
    let mut f14_2: f32 = 0.0;
    let mut f12_12: f32 = 0.0;
    let mut f15_3: f32 = 0.0;
    let mut f13_2: f32 = 0.0;
    let mut s3_2: i64 = 0;
    let mut a1_40: i32 = 0;
    let mut s1_3: i64 = 0;
    let mut a0_47: i32 = 0;
    let mut a1_41: i32 = 0;
    let mut f1_19: f32 = 0.0;
    let mut v0_23: i32 = 0;
    let mut f0_34: f32 = 0.0;
    let mut f1_20: f32 = 0.0;
    let mut f12_13: f32 = 0.0;
    let mut v0_24: i64 = 0;
    let mut f12_14: f32 = 0.0;
    let mut v0_25: i64 = 0;
    let mut a0_48: i64 = 0;
    let mut a1_42: i64 = 0;
    let mut v0_26: i64 = 0;
    let mut a0_49: i64 = 0;
    let mut a1_43: i64 = 0;
    let mut v0_27: i64 = 0;
    let mut a0_50: i64 = 0;
    let mut a1_44: i64 = 0;
    let mut v0_28: i64 = 0;
    let mut s0_6: i32 = 0;
    let mut a0_51: i64 = 0;
    let mut f0_35: f32 = 0.0;
    let mut f12_15: f32 = 0.0;
    let mut v0_29: i64 = 0;
    let mut f1_21: f32 = 0.0;
    let mut f0_36: f32 = 0.0;
    let mut f12_16: f32 = 0.0;
    let mut v0_30: i64 = 0;
    let mut a0_52: i64 = 0;
    let mut a1_45: i64 = 0;
    let mut v0_31: i64 = 0;
    let mut a0_53: i64 = 0;
    let mut a1_46: i64 = 0;
    let mut v0_32: i64 = 0;
    let mut a0_54: i64 = 0;
    let mut a1_47: i64 = 0;
    let mut v0_33: i64 = 0;
    let mut s0_7: i32 = 0;
    let mut a1_48: i64 = 0;
    let mut a0_55: i32 = 0;
    let mut v0_34: i64 = 0;
    let mut a0_56: i64 = 0;
    let mut f0_37: f32 = 0.0;
    let mut a4_9: i32 = 0;
    let mut a0_57: i32 = 0;
    let mut a1_49: i32 = 0;
    let mut a2_13: i32 = 0;
    let mut a3_13: i32 = 0;
    let mut v0_35: i32 = 0;
    let mut a1_50: i32 = 0;
    let mut a0_58: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut a0_59: i32 = 0;
    let mut a1_51: i32 = 0;
    let mut v1_7: i32 = 0;
    let mut v0_36: i32 = 0;
    let mut v0_37: i32 = 0;
    let mut f5_9: f32 = 0.0;
    let mut f4_9: f32 = 0.0;
    let mut f3_8: f32 = 0.0;
    let mut v1_8: i32 = 0;
    let mut s0_8: i32 = 0;
    let mut a1_52: i32 = 0;
    let mut a0_60: i32 = 0;
    let mut a2_14: i32 = 0;
    let mut f0_38: f32 = 0.0;
    let mut a3_14: i32 = 0;
    let mut a4_10: i32 = 0;
    let mut a5_4: i32 = 0;
    let mut a6_4: i32 = 0;
    let mut a7_4: i32 = 0;
    let mut v0_38: i32 = 0;
    let mut v1_9: i32 = 0;
    let mut v0_39: i32 = 0;
    let mut v0_40: i32 = 0;
    let mut v0_41: i32 = 0;
    let mut v0_42: i32 = 0;
    let mut v0_43: i32 = 0;
    let mut v1_10: i32 = 0;
    let mut a0_61: i32 = 0;
    let mut a1_53: i32 = 0;
    let mut s0_9: i32 = 0;
    let mut a2_15: i32 = 0;
    let mut s0_10: i32 = 0;
    let mut v1_11: i32 = 0;
    let mut s1_4: i64 = 0;
    let mut f12_17: f32 = 0.0;
    let mut v0_44: i64 = 0;
    let mut s4_2: i64 = 0;
    let mut f12_18: f32 = 0.0;
    let mut v0_45: i64 = 0;
    let mut a0_62: i64 = 0;
    let mut a1_54: i64 = 0;
    let mut v0_46: i64 = 0;
    let mut a0_63: i64 = 0;
    let mut a1_55: i64 = 0;
    let mut v0_47: i64 = 0;
    let mut a0_64: i64 = 0;
    let mut a1_56: i64 = 0;
    let mut v0_48: i64 = 0;
    let mut s0_11: i32 = 0;
    let mut a0_65: i64 = 0;
    let mut f0_39: f32 = 0.0;
    let mut f12_19: f32 = 0.0;
    let mut v0_49: i64 = 0;
    let mut f1_22: f32 = 0.0;
    let mut f0_40: f32 = 0.0;
    let mut f12_20: f32 = 0.0;
    let mut v0_50: i64 = 0;
    let mut a0_66: i64 = 0;
    let mut a1_57: i64 = 0;
    let mut v0_51: i64 = 0;
    let mut a0_67: i64 = 0;
    let mut a1_58: i64 = 0;
    let mut v0_52: i64 = 0;
    let mut a0_68: i64 = 0;
    let mut a1_59: i64 = 0;
    let mut v0_53: i64 = 0;
    let mut s0_12: i32 = 0;
    let mut a1_60: i64 = 0;
    let mut a0_69: i32 = 0;
    let mut v0_54: i64 = 0;
    let mut a0_70: i64 = 0;
    let mut f0_41: f32 = 0.0;
    let mut a1_61: i32 = 0;
    let mut v0_55: i32 = 0;
    let mut f1_23: f32 = 0.0;
    let mut f12_21: f32 = 0.0;
    let mut f13_3: f32 = 0.0;
    let mut f14_3: f32 = 0.0;
    let mut a0_71: i32 = 0;
    let mut f0_42: f32 = 0.0;
    let mut f12_22: f32 = 0.0;
    let mut f12_23: f32 = 0.0;
    let mut a0_72: i32 = 0;
    let mut f2_14: f32 = 0.0;
    let mut f3_9: f32 = 0.0;
    let mut a0_73: i32 = 0;
    let mut a1_62: i32 = 0;
    let mut a0_74: i32 = 0;
    let mut a1_63: i32 = 0;
    let mut a0_75: i32 = 0;
    let mut a1_64: i32 = 0;
    let mut v1_12: i32 = 0;
    let mut a0_76: i32 = 0;
    let mut a1_65: i32 = 0;
    let mut f1_24: f32 = 0.0;
    let mut a0_77: i32 = 0;
    let mut f0_43: f32 = 0.0;
    let mut a1_66: i32 = 0;
    let mut a0_78: i32 = 0;
    let mut a1_67: i32 = 0;
    let mut a0_79: i32 = 0;
    let mut a1_68: i32 = 0;
    let mut f6_8: f32 = 0.0;
    let mut f5_10: f32 = 0.0;
    let mut f4_10: f32 = 0.0;
    let mut f3_10: f32 = 0.0;
    let mut f0_44: f32 = 0.0;
    let mut f2_15: f32 = 0.0;
    let mut f3_11: f32 = 0.0;
    let mut f0_45: f32 = 0.0;
    let mut f0_46: f32 = 0.0;
    let mut f5_11: f32 = 0.0;
    let mut f3_12: f32 = 0.0;
    let mut f1_25: f32 = 0.0;
    let mut f2_16: f32 = 0.0;
    let mut f4_11: f32 = 0.0;
    let mut v0_56: i32 = 0;
    let mut f0_47: f32 = 0.0;
    let mut f0_48: f32 = 0.0;
    let mut f2_17: f32 = 0.0;
    let mut a1_69: i32 = 0;
    let mut a0_80: i32 = 0;
    let mut f2_18: f32 = 0.0;
    let mut f3_13: f32 = 0.0;
    let mut f0_49: f32 = 0.0;
    let mut f0_50: f32 = 0.0;
    let mut a1_70: i32 = 0;
    let mut f12_24: f32 = 0.0;
    let mut a0_81: i32 = 0;
    let mut v1_13: i32 = 0;
    let mut a0_82: i32 = 0;
    let mut a0_83: i32 = 0;
    let mut f12_25: f32 = 0.0;
    let mut a0_84: i32 = 0;
    let mut f0_51: f32 = 0.0;
    let mut v1_14: i32 = 0;
    let mut v0_57: i32 = 0;
    let mut a0_85: i32 = 0;
    let mut a1_71: i32 = 0;
    let mut a0_86: i32 = 0;
    let mut a1_72: i32 = 0;
    let mut a0_87: i32 = 0;
    let mut a1_73: i32 = 0;
    let mut a4_11: i32 = 0;
    let mut a0_88: i32 = 0;
    let mut a1_74: i32 = 0;
    let mut a2_16: i32 = 0;
    let mut a3_15: i32 = 0;
    let mut v0_58: i32 = 0;
    let mut s3_3: i32 = 0;
    let mut v1_15: i32 = 0;
    let mut a1_75: i32 = 0;
    let mut a0_89: i32 = 0;
    let mut f1_26: f32 = 0.0;
    let mut f0_52: f32 = 0.0;
    let mut v1_16: i32 = 0;
    let mut f0_53: f32 = 0.0;
    let mut f0_54: f32 = 0.0;
    let mut a1_76: i32 = 0;
    let mut a0_90: i32 = 0;
    let mut a1_77: i32 = 0;
    let mut a2_17: i32 = 0;
    let mut a3_16: i32 = 0;
    let mut a4_12: i32 = 0;
    let mut v0_59: i32 = 0;
    let mut f0_55: f32 = 0.0;
    let mut f2_19: f32 = 0.0;
    let mut f4_12: f32 = 0.0;
    let mut f6_9: f32 = 0.0;
    let mut f5_12: f32 = 0.0;
    let mut f7_4: f32 = 0.0;
    let mut f9_2: f32 = 0.0;
    let mut f3_14: f32 = 0.0;
    let mut f2_20: f32 = 0.0;
    let mut f4_13: f32 = 0.0;
    let mut f6_10: f32 = 0.0;
    let mut f5_13: f32 = 0.0;
    let mut f0_56: f32 = 0.0;
    let mut f0_57: f32 = 0.0;
    let mut a0_91: i32 = 0;
    let mut a0_92: i32 = 0;
    let mut f0_58: f32 = 0.0;
    let mut a0_93: i32 = 0;
    let mut a1_78: i32 = 0;
    let mut a1_79: i32 = 0;
    let mut a1_80: i32 = 0;
    let mut f0_59: f32 = 0.0;
    let mut f7_5: f32 = 0.0;
    let mut f5_14: f32 = 0.0;
    let mut f1_27: f32 = 0.0;
    let mut f4_14: f32 = 0.0;
    let mut f6_11: f32 = 0.0;
    let mut f3_15: f32 = 0.0;
    let mut f1_28: f32 = 0.0;
    let mut f0_60: f32 = 0.0;
    let mut v0_60: i32 = 0;
    let mut f2_21: f32 = 0.0;
    let mut v0_61: i64 = 0;
    let mut a0_94: i32 = 0;
    let mut f1_29: f32 = 0.0;
    let mut f1_30: f32 = 0.0;
    let mut v0_62: i64 = 0;
    let mut a0_95: i32 = 0;
    let mut f1_31: f32 = 0.0;
    let mut f1_32: f32 = 0.0;
    let mut v0_63: i64 = 0;
    let mut a0_96: i32 = 0;
    let mut f2_22: f32 = 0.0;
    let mut f1_33: f32 = 0.0;
    let mut f2_23: f32 = 0.0;
    let mut f12_26: f32 = 0.0;
    let mut f0_61: f32 = 0.0;
    let mut f12_27: f32 = 0.0;
    let mut v0_64: i64 = 0;
    let mut a0_97: i64 = 0;
    let mut v0_65: i64 = 0;
    let mut a0_98: i64 = 0;
    let mut a1_81: i64 = 0;
    let mut v0_66: i64 = 0;
    let mut a0_99: i64 = 0;
    let mut f0_62: f32 = 0.0;
    let mut f1_34: f32 = 0.0;
    let mut f2_24: f32 = 0.0;
    let mut a0_100: i32 = 0;
    let mut a1_82: i32 = 0;
    let mut v1_17: i32 = 0;
    let mut s0_13: i32 = 0;
    let mut a1_83: i32 = 0;
    let mut f12_28: f32 = 0.0;
    let mut a0_101: i32 = 0;
    let mut a0_102: i32 = 0;
    let mut a2_18: i32 = 0;
    let mut a3_17: i32 = 0;
    let mut v0_67: i32 = 0;
    let mut s1_5: i32 = 0;
    let mut s0_14: i32 = 0;
    let mut v0_68: i64 = 0;
    let mut a1_84: i32 = 0;
    let mut a0_103: i32 = 0;
    let mut a3_18: i32 = 0;
    let mut a1_85: i32 = 0;
    let mut a6_5: i32 = 0;
    let mut a0_104: i32 = 0;
    let mut a2_19: i32 = 0;
    let mut a4_13: i32 = 0;
    let mut a5_5: i32 = 0;
    let mut a7_5: i32 = 0;
    let mut a0_105: i32 = 0;
    let mut f12_29: f32 = 0.0;
    let mut f12_30: f32 = 0.0;
    let mut a0_106: i32 = 0;
    let mut a1_86: i32 = 0;
    let mut a2_20: i32 = 0;
    let mut a3_19: i32 = 0;
    let mut a2_21: i32 = 0;
    let mut a0_107: i32 = 0;
    let mut a1_87: i32 = 0;
    let mut a3_20: i32 = 0;
    let mut a4_14: i32 = 0;
    let mut a0_108: i32 = 0;
    let mut v0_69: i32 = 0;
    let mut f2_25: f32 = 0.0;
    let mut f0_63: f32 = 0.0;
    let mut f1_35: f32 = 0.0;
    let mut a1_88: i32 = 0;
    let mut f0_64: f32 = 0.0;
    let mut v0_70: i32 = 0;
    let mut a0_109: i32 = 0;
    let mut a4_15: i32 = 0;
    let mut a1_89: i32 = 0;
    let mut a2_22: i32 = 0;
    let mut a3_21: i32 = 0;
    let mut v0_71: i32 = 0;
    let mut a1_90: i32 = 0;
    let mut a0_110: i32 = 0;
    let mut f3_16: f32 = 0.0;
    let mut s3_4: i32 = 0;
    let mut v0_72: i64 = 0;
    let mut a1_91: i32 = 0;
    let mut s4_3: i32 = 0;
    let mut a0_111: i32 = 0;
    let mut a3_22: i32 = 0;
    let mut a0_112: i32 = 0;
    let mut a6_6: i32 = 0;
    let mut a1_92: i32 = 0;
    let mut a2_23: i32 = 0;
    let mut a4_16: i32 = 0;
    let mut a5_6: i32 = 0;
    let mut a7_6: i32 = 0;
    let mut a0_113: i32 = 0;
    let mut a1_93: i32 = 0;
    let mut a2_24: i32 = 0;
    let mut a3_23: i32 = 0;
    let mut f12_31: f32 = 0.0;
    let mut a0_114: i32 = 0;
    let mut f12_32: f32 = 0.0;
    let mut a0_115: i32 = 0;
    let mut a2_25: i32 = 0;
    let mut a1_94: i32 = 0;
    let mut a3_24: i32 = 0;
    let mut a4_17: i32 = 0;
    let mut a3_25: i32 = 0;
    let mut a0_116: i32 = 0;
    let mut a1_95: i32 = 0;
    let mut f12_33: f32 = 0.0;
    let mut a4_18: i32 = 0;
    let mut a5_7: i32 = 0;
    let mut a2_26: i32 = 0;
    let mut a6_7: i32 = 0;
    let mut a7_7: i32 = 0;
    let mut v1_18: i32 = 0;
    let mut f1_36: f32 = 0.0;
    let mut f3_17: f32 = 0.0;
    let mut f2_26: f32 = 0.0;
    let mut a0_117: i32 = 0;
    let mut a2_27: i32 = 0;
    let mut a1_96: i32 = 0;
    let mut f3_18: f32 = 0.0;
    let mut f1_37: f32 = 0.0;
    let mut f0_65: f32 = 0.0;
    let mut a0_118: i32 = 0;
    let mut f12_34: f32 = 0.0;
    let mut a0_119: i32 = 0;
    let mut a1_97: i32 = 0;
    let mut a0_120: i32 = 0;
    let mut a1_98: i32 = 0;
    let mut a2_28: i32 = 0;
    let mut a3_26: i32 = 0;
    let mut v0_73: i32 = 0;
    let mut a4_19: i32 = 0;
    let mut a1_99: i32 = 0;
    let mut a2_29: i32 = 0;
    let mut a3_27: i32 = 0;
    let mut v0_74: i32 = 0;
    let mut a1_100: i32 = 0;
    let mut a0_121: i32 = 0;
    let mut s4_4: i32 = 0;
    let mut f5_15: f32 = 0.0;
    let mut f0_66: f32 = 0.0;
    let mut f1_38: f32 = 0.0;
    let mut v0_75: i32 = 0;
    let mut s6_1: i32 = 0;
    let mut f7_6: f32 = 0.0;
    let mut f8_2: f32 = 0.0;
    let mut f6_12: f32 = 0.0;
    let mut f20_1: f32 = 0.0;
    let mut f9_3: f32 = 0.0;
    let mut f5_16: f32 = 0.0;
    let mut f3_19: f32 = 0.0;
    let mut f1_39: f32 = 0.0;
    let mut f0_67: f32 = 0.0;
    let mut f0_68: f32 = 0.0;
    let mut a0_122: i32 = 0;
    let mut v0_76: i32 = 0;
    let mut a0_123: i32 = 0;
    let mut a0_124: i32 = 0;
    let mut a1_101: i32 = 0;
    let mut s1_6: i32 = 0;
    let mut v0_77: i64 = 0;
    let mut a1_102: i32 = 0;
    let mut a0_125: i32 = 0;
    let mut a2_30: i32 = 0;
    let mut a3_28: i32 = 0;
    let mut a3_29: i32 = 0;
    let mut a0_126: i32 = 0;
    let mut a1_103: i32 = 0;
    let mut a2_31: i32 = 0;
    let mut a0_127: i32 = 0;
    let mut a2_32: i32 = 0;
    let mut a1_104: i32 = 0;
    let mut a3_30: i32 = 0;
    let mut a4_20: i32 = 0;
    let mut s3_5: i32 = 0;
    let mut f12_35: f32 = 0.0;
    let mut a0_128: i32 = 0;
    let mut a3_31: i32 = 0;
    let mut a2_33: i32 = 0;
    let mut a1_105: i32 = 0;
    let mut a4_21: i32 = 0;
    let mut a5_8: i32 = 0;
    let mut a6_8: i32 = 0;
    let mut a7_8: i32 = 0;
    let mut f12_36: f32 = 0.0;
    let mut a0_129: i32 = 0;
    let mut v0_78: i64 = 0;
    let mut f2_27: f32 = 0.0;
    let mut f2_28: f32 = 0.0;
    let mut f1_40: f32 = 0.0;
    let mut f3_20: f32 = 0.0;
    let mut a1_106: i32 = 0;
    let mut a2_34: i32 = 0;
    let mut a3_32: i32 = 0;
    let mut a0_130: i32 = 0;
    let mut a4_22: i32 = 0;
    let mut v0_79: i32 = 0;
    let mut a0_131: i32 = 0;
    let mut a4_23: i32 = 0;
    let mut a1_107: i32 = 0;
    let mut a2_35: i32 = 0;
    let mut a3_33: i32 = 0;
    let mut v0_80: i32 = 0;
    let mut a1_108: i32 = 0;
    let mut a0_132: i32 = 0;
    let mut f5_17: f32 = 0.0;
    let mut f0_69: f32 = 0.0;
    let mut f1_41: f32 = 0.0;
    let mut s4_5: i32 = 0;
    let mut f7_7: f32 = 0.0;
    let mut f8_3: f32 = 0.0;
    let mut f6_13: f32 = 0.0;
    let mut f20_2: f32 = 0.0;
    let mut f9_4: f32 = 0.0;
    let mut f5_18: f32 = 0.0;
    let mut f3_21: f32 = 0.0;
    let mut f1_42: f32 = 0.0;
    let mut f0_70: f32 = 0.0;
    let mut f0_71: f32 = 0.0;
    let mut a0_133: i32 = 0;
    let mut a1_109: i32 = 0;
    let mut s1_7: i32 = 0;
    let mut a0_134: i32 = 0;
    let mut a2_36: i32 = 0;
    let mut a1_110: i32 = 0;
    let mut s1_8: i32 = 0;
    let mut v0_81: i32 = 0;
    let mut v1_19: i32 = 0;
    let mut a2_37: i32 = 0;
    let mut a1_111: i32 = 0;
    let mut v0_82: i32 = 0;
    let mut v0_83: i32 = 0;
    let mut a0_135: i32 = 0;
    let mut a1_112: i32 = 0;
    let mut a2_38: i32 = 0;
    let mut a0_136: i32 = 0;
    let mut a1_113: i32 = 0;
    let mut a0_137: i32 = 0;
    let mut a1_114: i32 = 0;
    let mut a0_138: i32 = 0;
    let mut a2_39: i32 = 0;
    let mut a1_115: i32 = 0;
    let mut a3_34: i32 = 0;
    let mut a4_24: i32 = 0;
    let mut s3_6: i32 = 0;
    let mut f12_37: f32 = 0.0;
    let mut a0_139: i32 = 0;
    let mut a3_35: i32 = 0;
    let mut a2_40: i32 = 0;
    let mut a1_116: i32 = 0;
    let mut a4_25: i32 = 0;
    let mut a5_9: i32 = 0;
    let mut a6_9: i32 = 0;
    let mut a7_9: i32 = 0;
    let mut f12_38: f32 = 0.0;
    let mut a0_140: i32 = 0;
    let mut a0_141: i32 = 0;
    let mut a0_142: i32 = 0;
    let mut v0_84: i64 = 0;
    let mut f1_43: f32 = 0.0;
    let mut f1_44: f32 = 0.0;
    let mut f2_29: f32 = 0.0;
    let mut a1_117: i32 = 0;
    let mut f2_30: f32 = 0.0;
    let mut a2_41: i32 = 0;
    let mut a0_143: i32 = 0;
    let mut a3_36: i32 = 0;
    let mut v0_85: i32 = 0;
    let mut a0_144: i32 = 0;
    let mut f2_31: f32 = 0.0;
    let mut f3_22: f32 = 0.0;
    let mut v0_86: i32 = 0;
    let mut v1_20: i32 = 0;
    a3_1 = lw(0x3b0138);
    sw(sp + 244, sp + 208);
    sw(sp + 248, sp + 212);
    sw(sp + 252, sp + 216);
    sw(sp + 228, sp + 32);
    sw(sp + 256, sp + 220);
    s7_1 = 0;
    f3_1 = lwc1(g::updateframesf);
    sw(sp + 232, sp + 80);
    sw(sp + 236, sp + 144);
    sw(sp + 240, sp + 160);
    'l1: loop {
        s2_1 = s7_1 * 276 + a3_1;
        a2_1 = lw(s2_1);
        s1_1 = s7_1 + 1;
        if a2_1 != 0 {
            'b4: {
                'b5: {
                    a1_1 = lw(s2_1 + 108);
                    sw(sp + 224, 0);
                    if a1_1 >= 0 {
                        'b6: {
                            v1_1 = lw(s2_1 + 104);
                            if v1_1 != 1 {
                                if v1_1 != 4 {
                                    if v1_1 != 2 {
                                        f1_1 = lwc1(s2_1 + 48);
                                        break 'b4;
                                    }
                                    v1_2 = lw(a1_1 * 0x1210 + lw(g::chrinfo) + 3020);
                                    sw(sp + 224, v1_2);
                                    sw(v1_2 + 16, lw(v1_2 + 16) & -33);
                                    break 'b6;
                                }
                            }
                            fcc_1 = lwc1(s2_1 + 72) <= lwc1(0x3ad1ec);
                            v0_1 = lw(a1_1 * 1820 + lw(g::players) + 384);
                            sw(sp + 224, v0_1);
                            if !fcc_1 {
                                break 'b5;
                            }
                            sw(lw(sp + 224) + 16, lw(v0_1 + 16) & -33);
                        }
                        a2_1 = lw(s2_1);
                    }
                }
                f1_1 = lwc1(s2_1 + 48);
            }
            'b8: {
                'b9: {
                    v1_3 = a2_1 - 2;
                    swc1(sp, f1_1 * f3_1);
                    swc1(sp + 4, lwc1(s2_1 + 52) * f3_1);
                    swc1(sp + 8, lwc1(s2_1 + 56) * f3_1);
                    swc1(s2_1 + 72, lwc1(s2_1 + 72) + f3_1 / 6e+01_f32);
                    if (v1_3 as u32) < (18 as u32) {
                        'b10: {
                            'b11: {
                                'b12: {
                                    'b13: {
                                        'b14: {
                                            match (v1_3 << 2) >> 2 {
                                                0 | 1 | 2 => {
                                                    s1_1 = s7_1 + 1;
                                                    if lw(g::updateframesi) <= 0 {
                                                        break 'b8;
                                                    }
                                                    a0_109 = s2_1;
                                                    if 2.0_f32 <= lwc1(s2_1 + 72) {
                                                        break 'b10;
                                                    }
                                                    a4_23 = lw(sp + 244);
                                                    a1_107 = sp;
                                                    a2_35 = sp + 48;
                                                    a3_33 = sp + 64;
                                                    v0_80 = bulletStraightLineTest(a0_109, a1_107, a2_35, a3_33, a4_23);
                                                    a1_108 = lw(sp + 208);
                                                    if a1_108 != 0 {
                                                        a0_132 = s2_1;
                                                        bulletProcessGlasshits(a0_132, a1_108);
                                                    }
                                                    if v0_80 == 0 {
                                                        break 'b11;
                                                    }
                                                    f5_17 = lwc1(s2_1 + 48);
                                                    f0_69 = lwc1(s2_1 + 52);
                                                    f1_41 = lwc1(s2_1 + 56);
                                                    s4_5 = lw(v0_80 + 8);
                                                    f7_7 = lwc1(sp + 64);
                                                    f8_3 = lwc1(sp + 68);
                                                    f6_13 = lwc1(sp + 72);
                                                    f20_2 = f5_17 * f5_17 + f0_69 * f0_69 + f1_41 * f1_41;
                                                    f9_4 = 1.0_f32 / f20_2;
                                                    f5_18 = f5_17 * f9_4;
                                                    swc1(sp + 16, f5_18);
                                                    f3_21 = lwc1(s2_1 + 52) * f9_4;
                                                    swc1(sp + 20, f3_21);
                                                    f1_42 = lwc1(s2_1 + 56) * f9_4;
                                                    swc1(sp + 24, f1_42);
                                                    f0_70 = f5_18 * f7_7 + f3_21 * f8_3 + f1_42 * f6_13;
                                                    f0_71 = f0_70 + f0_70;
                                                    swc1(sp + 40, f1_42 - f0_71 * f6_13);
                                                    swc1(sp + 32, f5_18 - f0_71 * f7_7);
                                                    swc1(sp + 36, f3_21 - f0_71 * f8_3);
                                                    if s4_5 != 8 {
                                                        a0_134 = lw(sp + 248);
                                                        a2_36 = 0;
                                                        a1_110 = lw(sp + 252);
                                                        s1_8 = 0;
                                                        v0_81 = obGetLastHitInst(a0_134, a1_110, a2_36);
                                                        v1_19 = lw(sp + 212);
                                                        if v0_81 != 0 {
                                                            a2_37 = lw(v0_81);
                                                            a1_111 = lw(a2_37 - lw(a2_37) * 80 + v1_19 * 80 + 8);
                                                            v0_82 = lw(sp + 216);
                                                            if a1_111 != 0 {
                                                                v0_83 = lw(v0_82 * 24 + a1_111);
                                                                if v0_83 != -1 {
                                                                    s1_8 = lbu(lw(g::thistexbuf) + lw((v0_83 << 4) + lw(a2_37 + 12)) + 0x1808);
                                                                }
                                                            }
                                                        }
                                                        a0_135 = s1_8;
                                                        a1_112 = sp + 48;
                                                        ricochetsfx(a0_135, a1_112);
                                                        a2_38 = lw(sp + 228);
                                                        a0_136 = 0;
                                                        a1_113 = sp + 48;
                                                        particleNewSparks(a0_136, a1_113, a2_38);
                                                        a0_137 = 0;
                                                        a1_114 = sp + 48;
                                                        particleNewSmoke(a0_137, a1_114);
                                                        a0_138 = lw(s2_1);
                                                        a2_39 = v0_80;
                                                        a1_115 = lw(s2_1 + 100);
                                                        a3_34 = sp + 48;
                                                        a4_24 = sp + 64;
                                                        decalNewDecal(a0_138, a1_115, a2_39, a3_34, a4_24);
                                                        s1_7 = s2_1 + 24;
                                                    } else {
                                                        a0_133 = 404;
                                                        a1_109 = sp + 48;
                                                        soundStart(a0_133, a1_109);
                                                        s1_7 = s2_1 + 24;
                                                    }
                                                    s3_6 = s2_1 + 48;
                                                    f12_37 = lwc1(s2_1 + 76);
                                                    a0_139 = v0_80;
                                                    a3_35 = lw(s2_1);
                                                    a2_40 = sp + 48;
                                                    a1_116 = lw(sp + 224);
                                                    a4_25 = s1_7;
                                                    a5_9 = s3_6;
                                                    a6_9 = s2_1;
                                                    a7_9 = 0;
                                                    propDamage(a0_139, a1_116, a2_40, a3_35, a4_25, a5_9, a6_9, a7_9, f12_37);
                                                    f12_38 = lwc1(0x3ad1f0);
                                                    a0_140 = sp + 48;
                                                    bulletExplodeMines(a0_140, f12_38);
                                                    a0_141 = lw(s2_1 + 112);
                                                    if a0_141 == 0 {
                                                        a0_142 = lw(s2_1 + 116);
                                                    } else {
                                                        particleFree(a0_141);
                                                        sw(s2_1 + 112, 0);
                                                        a0_142 = lw(s2_1 + 116);
                                                    }
                                                    if a0_142 != 0 {
                                                        particleFree(a0_142);
                                                        sw(s2_1 + 116, 0);
                                                    }
                                                    v0_84 = newrnd();
                                                    if v0_84 < 0_i64 {
                                                        f1_44 = (v0_84 & 1_i64 | ((v0_84 as i32 as u32) >> 1) as i32 as i64) as i32 as f32;
                                                        f1_43 = f1_44 + f1_44;
                                                    } else {
                                                        f1_43 = v0_84 as i32 as f32;
                                                    }
                                                    f2_29 = lwc1(s2_1 + 152);
                                                    a0_109 = s2_1;
                                                    if !(f2_29 < f1_43 * 2.3283064e-10_f32) {
                                                        if s4_5 != 8 {
                                                            sw(s2_1 + 156, 1);
                                                            a1_117 = s1_7;
                                                            f2_30 = lwc1(0x3ad1f4);
                                                            a2_41 = s3_6;
                                                            swc1(s2_1 + 152, f2_29 * f2_29);
                                                            a0_143 = s2_1 + 112;
                                                            a3_36 = 15;
                                                            s1_1 = s7_1 + 1;
                                                            swc1(s2_1 + 24, lwc1(sp + 48) + lwc1(sp + 64) * f2_30);
                                                            swc1(s2_1 + 28, lwc1(sp + 52) + lwc1(sp + 68) * f2_30);
                                                            swc1(s2_1 + 32, lwc1(sp + 56) + lwc1(sp + 72) * f2_30);
                                                            swc1(s2_1 + 48, lwc1(sp + 32) * f20_2);
                                                            swc1(s2_1 + 52, lwc1(sp + 36) * f20_2);
                                                            swc1(s2_1 + 56, lwc1(sp + 40) * f20_2);
                                                            v0_85 = particleNewTracer(a0_143, a1_117, a2_41, a3_36);
                                                            sw(s2_1 + 112, v0_85);
                                                            a0_144 = s2_1;
                                                            bulletInitialiseStraightLineTest(a0_144);
                                                            sw(s2_1 + 272, 1);
                                                            break 'b12;
                                                        }
                                                        a0_109 = s2_1;
                                                    }
                                                    s1_1 = s7_1 + 1;
                                                    break 'b10;
                                                }
                                                3 | 4 | 5 => {
                                                    s1_1 = s7_1 + 1;
                                                    if lw(g::updateframesi) <= 0 {
                                                        break 'b8;
                                                    }
                                                    a0_109 = s2_1;
                                                    if 2.0_f32 <= lwc1(s2_1 + 72) {
                                                        break 'b10;
                                                    }
                                                    a4_19 = lw(sp + 244);
                                                    a1_99 = sp;
                                                    a2_29 = sp + 48;
                                                    a3_27 = sp + 64;
                                                    v0_74 = bulletStraightLineTest(a0_109, a1_99, a2_29, a3_27, a4_19);
                                                    a1_100 = lw(sp + 208);
                                                    if a1_100 != 0 {
                                                        a0_121 = s2_1;
                                                        bulletProcessGlasshits(a0_121, a1_100);
                                                    }
                                                    s4_4 = -1;
                                                    if v0_74 == 0 {
                                                        break 'b11;
                                                    }
                                                    f5_15 = lwc1(s2_1 + 48);
                                                    f0_66 = lwc1(s2_1 + 52);
                                                    f1_38 = lwc1(s2_1 + 56);
                                                    v0_75 = lw(s2_1 + 112);
                                                    s6_1 = lw(v0_74 + 8);
                                                    f7_6 = lwc1(sp + 64);
                                                    f8_2 = lwc1(sp + 68);
                                                    f6_12 = lwc1(sp + 72);
                                                    f20_1 = f5_15 * f5_15 + f0_66 * f0_66 + f1_38 * f1_38;
                                                    f9_3 = 1.0_f32 / f20_1;
                                                    f5_16 = f5_15 * f9_3;
                                                    swc1(sp + 16, f5_16);
                                                    f3_19 = lwc1(s2_1 + 52) * f9_3;
                                                    swc1(sp + 20, f3_19);
                                                    f1_39 = lwc1(s2_1 + 56) * f9_3;
                                                    swc1(sp + 24, f1_39);
                                                    f0_67 = f5_16 * f7_6 + f3_19 * f8_2 + f1_39 * f6_12;
                                                    f0_68 = f0_67 + f0_67;
                                                    swc1(sp + 40, f1_39 - f0_68 * f6_12);
                                                    swc1(sp + 32, f5_16 - f0_68 * f7_6);
                                                    swc1(sp + 36, f3_19 - f0_68 * f8_2);
                                                    if v0_75 != 0 {
                                                        a0_122 = v0_75;
                                                        s4_4 = lw(v0_75 + 928);
                                                        particleFree(a0_122);
                                                        sw(s2_1 + 112, 0);
                                                    }
                                                    v0_76 = lw(s2_1 + 116);
                                                    a0_123 = v0_76;
                                                    if v0_76 != 0 {
                                                        s4_4 = lw(v0_76 + 928);
                                                        particleFree(a0_123);
                                                        sw(s2_1 + 116, 0);
                                                    }
                                                    if s6_1 != 8 {
                                                        s1_6 = s2_1 + 24;
                                                        v0_77 = newrnd();
                                                        a1_102 = sp + 48;
                                                        a0_125 = (v0_77 & 3_i64) as i32 + 174;
                                                        a2_30 = 20;
                                                        a3_28 = 0;
                                                        soundDelayStartEx(a0_125, a1_102, a2_30, a3_28);
                                                        a3_29 = lw(sp + 228);
                                                        a0_126 = 0;
                                                        a1_103 = s4_4;
                                                        a2_31 = sp + 48;
                                                        particleNewSpriteSparks(a0_126, a1_103, a2_31, a3_29);
                                                        a0_127 = lw(s2_1);
                                                        a2_32 = v0_74;
                                                        a1_104 = lw(s2_1 + 100);
                                                        a3_30 = sp + 48;
                                                        a4_20 = sp + 64;
                                                        decalNewDecal(a0_127, a1_104, a2_32, a3_30, a4_20);
                                                    } else {
                                                        a0_124 = 404;
                                                        a1_101 = sp + 48;
                                                        soundStart(a0_124, a1_101);
                                                        s1_6 = s2_1 + 24;
                                                    }
                                                    s3_5 = s2_1 + 48;
                                                    f12_35 = lwc1(s2_1 + 76);
                                                    a0_128 = v0_74;
                                                    a3_31 = lw(s2_1);
                                                    a2_33 = sp + 48;
                                                    a1_105 = lw(sp + 224);
                                                    a4_21 = s1_6;
                                                    a5_8 = s3_5;
                                                    a6_8 = s2_1;
                                                    a7_8 = 0;
                                                    propDamage(a0_128, a1_105, a2_33, a3_31, a4_21, a5_8, a6_8, a7_8, f12_35);
                                                    f12_36 = lwc1(0x3ad1f8);
                                                    a0_129 = sp + 48;
                                                    bulletExplodeMines(a0_129, f12_36);
                                                    a0_109 = s2_1;
                                                    if !(f20_1 < 0.5_f32) {
                                                        v0_78 = newrnd();
                                                        if v0_78 < 0_i64 {
                                                            f2_28 = (v0_78 & 1_i64 | ((v0_78 as i32 as u32) >> 1) as i32 as i64) as i32 as f32;
                                                            f2_27 = f2_28 + f2_28;
                                                        } else {
                                                            f2_27 = v0_78 as i32 as f32;
                                                        }
                                                        a0_109 = s2_1;
                                                        if !(lwc1(s2_1 + 152) < f2_27 * 2.3283064e-10_f32) {
                                                            f1_40 = lwc1(sp + 64);
                                                            if s6_1 != 8 {
                                                                f3_20 = lwc1(0x3ad1fc);
                                                                swc1(s2_1 + 24, lwc1(sp + 48) + f1_40 * f3_20);
                                                                swc1(s2_1 + 28, lwc1(sp + 52) + lwc1(sp + 68) * f3_20);
                                                                swc1(s2_1 + 32, lwc1(sp + 56) + lwc1(sp + 72) * f3_20);
                                                                swc1(s2_1 + 48, lwc1(sp + 32) * f20_1);
                                                                swc1(s2_1 + 52, lwc1(sp + 36) * f20_1);
                                                                swc1(s2_1 + 56, lwc1(sp + 40) * f20_1);
                                                                if s4_4 >= 0 {
                                                                    a1_106 = s4_4;
                                                                    a2_34 = s1_6;
                                                                    a3_32 = s3_5;
                                                                    a0_130 = s2_1 + 112;
                                                                    a4_22 = 15;
                                                                    v0_79 = particleNewPulse(a0_130, a1_106, a2_34, a3_32, a4_22);
                                                                    sw(s2_1 + 112, v0_79);
                                                                }
                                                                a0_131 = s2_1;
                                                                s1_1 = s7_1 + 1;
                                                                bulletInitialiseStraightLineTest(a0_131);
                                                                break 'b12;
                                                            }
                                                            a0_109 = s2_1;
                                                        }
                                                    }
                                                    s1_1 = s7_1 + 1;
                                                    break 'b10;
                                                }
                                                6 | 7 => {
                                                    s1_1 = s7_1 + 1;
                                                    if lw(g::updateframesi) <= 0 {
                                                        break 'b8;
                                                    }
                                                    a0_109 = s2_1;
                                                    if 3.0_f32 <= lwc1(s2_1 + 72) {
                                                        break 'b10;
                                                    }
                                                    a4_15 = lw(sp + 244);
                                                    a1_89 = sp;
                                                    a2_22 = sp + 48;
                                                    a3_21 = sp + 64;
                                                    v0_71 = bulletNonStraightLineTest(a0_109, a1_89, a2_22, a3_21, a4_15);
                                                    a1_90 = lw(sp + 208);
                                                    if a1_90 != 0 {
                                                        a0_110 = s2_1;
                                                        bulletProcessGlasshits(a0_110, a1_90);
                                                    }
                                                    f3_16 = lwc1(sp);
                                                    if v0_71 == 0 {
                                                        v1_18 = 0;
                                                        f1_36 = lwc1(s2_1 + 28);
                                                        f3_17 = lwc1(s2_1 + 24) + f3_16;
                                                        f2_26 = lwc1(s2_1 + 32);
                                                        a0_117 = lw(g::updateframesi);
                                                        swc1(s2_1 + 24, f3_17);
                                                        swc1(s2_1 + 28, f1_36 + lwc1(sp + 4));
                                                        swc1(s2_1 + 32, f2_26 + lwc1(sp + 8));
                                                        if a0_117 <= 0 {
                                                            a2_27 = lw(s2_1 + 204);
                                                            a1_96 = lw(s2_1 + 96);
                                                            s1_1 = s7_1 + 1;
                                                            swc1(a1_96 + 48, f3_17);
                                                        } else {
                                                            a2_27 = lw(s2_1 + 204);
                                                            a1_96 = lw(s2_1 + 96);
                                                            s1_1 = s7_1 + 1;
                                                            f3_18 = lwc1(0x3ad200);
                                                            'l32: loop {
                                                                v1_18 = v1_18 + 1;
                                                                f1_37 = lwc1(s2_1 + 56) * f3_18;
                                                                f0_65 = lwc1(s2_1 + 52) * f3_18 + lwc1(0x3ad204);
                                                                swc1(s2_1 + 48, lwc1(s2_1 + 48) * f3_18);
                                                                swc1(s2_1 + 56, f1_37);
                                                                swc1(s2_1 + 52, f0_65);
                                                                if v1_18 >= a0_117 {
                                                                    break;
                                                                }
                                                            }
                                                            swc1(a1_96 + 48, f3_17);
                                                        }
                                                        swc1(a1_96 + 52, lwc1(s2_1 + 28));
                                                        swc1(a1_96 + 56, lwc1(s2_1 + 32));
                                                        if a2_27 != 0 {
                                                            a0_118 = a2_27;
                                                            if lw(a2_27 + 24) < 100 {
                                                                a1_97 = a1_96 + 48;
                                                                particleUpdatePos(a0_118, a1_97);
                                                                a3_1 = lw(0x3b0138);
                                                                break 'b8;
                                                            }
                                                            f12_34 = 0.0_f32;
                                                            a0_119 = a2_27;
                                                            particleUpdateRate(a0_119, f12_34);
                                                            sw(lw(s2_1 + 204) + 20, 0);
                                                        }
                                                        a0_120 = s2_1 + 204;
                                                        a1_98 = 1;
                                                        a2_28 = s2_1 + 24;
                                                        a3_26 = 60;
                                                        v0_73 = particleNewTrail(a0_120, a1_98, a2_28, a3_26);
                                                        sw(s2_1 + 204, v0_73);
                                                        a3_1 = lw(0x3b0138);
                                                        break 'b8;
                                                    }
                                                    s3_4 = s2_1 + 24;
                                                    v0_72 = newrnd();
                                                    a1_91 = sp + 48;
                                                    s4_3 = s2_1 + 48;
                                                    a0_111 = ((v0_72 as i32 as u32) % (6 as u32)) as i32 + 98;
                                                    soundStart(a0_111, a1_91);
                                                    a3_22 = lw(s2_1);
                                                    a0_112 = 0;
                                                    a6_6 = lw(sp + 224);
                                                    a1_92 = 0;
                                                    a2_23 = sp + 48;
                                                    a4_16 = s3_4;
                                                    a5_6 = s4_3;
                                                    a7_6 = v0_71;
                                                    particleNewExplosion(a0_112, a1_92, a2_23, a3_22, a4_16, a5_6, a6_6, a7_6);
                                                    a0_113 = 0;
                                                    a1_93 = 13;
                                                    a2_24 = s3_4;
                                                    a3_23 = 0;
                                                    f12_31 = 0.0_f32;
                                                    particleNewBurst(a0_113, a1_93, a2_24, a3_23, f12_31);
                                                    a0_114 = lw(s2_1 + 204);
                                                    if a0_114 == 0 {
                                                        a0_115 = lw(s2_1);
                                                    } else {
                                                        f12_32 = 0.0_f32;
                                                        particleUpdateRate(a0_114, f12_32);
                                                        a0_115 = lw(s2_1);
                                                    }
                                                    a2_25 = v0_71;
                                                    a1_94 = lw(s2_1 + 100);
                                                    a3_24 = sp + 48;
                                                    a4_17 = sp + 64;
                                                    s1_1 = s7_1 + 1;
                                                    decalNewDecal(a0_115, a1_94, a2_25, a3_24, a4_17);
                                                    a3_25 = lw(s2_1);
                                                    a0_116 = v0_71;
                                                    a1_95 = lw(sp + 224);
                                                    f12_33 = 0.0_f32;
                                                    a4_18 = s3_4;
                                                    a5_7 = s4_3;
                                                    a2_26 = sp + 48;
                                                    a6_7 = s2_1;
                                                    a7_7 = 0;
                                                    propDamage(a0_116, a1_95, a2_26, a3_25, a4_18, a5_7, a6_7, a7_7, f12_33);
                                                    a0_109 = s2_1;
                                                    break 'b10;
                                                }
                                                8 | 9 | 10 | 11 => {
                                                    'b36: {
                                                        'b37: {
                                                            a4_11 = lw(sp + 244);
                                                            if lw(g::updateframesi) > 0 {
                                                                'b39: {
                                                                    a0_88 = s2_1;
                                                                    a1_74 = sp;
                                                                    a2_16 = sp + 48;
                                                                    a3_15 = sp + 64;
                                                                    v0_58 = bulletNonStraightLineTest(a0_88, a1_74, a2_16, a3_15, a4_11);
                                                                    s3_3 = v0_58;
                                                                    v1_15 = lw(s2_1);
                                                                    if v1_15 != 10 {
                                                                        a1_75 = lw(sp + 208);
                                                                        if v1_15 != 13 {
                                                                            break 'b39;
                                                                        }
                                                                    }
                                                                    s3_3 = 0;
                                                                    a1_75 = lw(sp + 208);
                                                                }
                                                                if a1_75 == 0 {
                                                                    f1_26 = lwc1(s2_1 + 72);
                                                                } else {
                                                                    a0_89 = s2_1;
                                                                    bulletProcessGlasshits(a0_89, a1_75);
                                                                    f1_26 = lwc1(s2_1 + 72);
                                                                }
                                                                'b42: {
                                                                    if 3.0_f32 <= f1_26 {
                                                                        v1_16 = lw(s2_1);
                                                                    } else {
                                                                        f0_52 = lwc1(sp);
                                                                        if s3_3 == 0 {
                                                                            f0_53 = lwc1(sp + 4);
                                                                            if f0_52 == 0.0_f32 {
                                                                                f0_54 = lwc1(sp + 8);
                                                                                if f0_53 == 0.0_f32 {
                                                                                    if f0_54 == 0.0_f32 {
                                                                                        a1_76 = lw(s2_1 + 96);
                                                                                        break 'b42;
                                                                                    }
                                                                                }
                                                                            }
                                                                            a0_90 = s2_1;
                                                                            a1_77 = sp;
                                                                            a2_17 = sp + 48;
                                                                            a3_16 = sp + 64;
                                                                            a4_12 = 0;
                                                                            v0_59 = bulletNonStraightLineTest(a0_90, a1_77, a2_17, a3_16, a4_12);
                                                                            f0_55 = lwc1(sp + 64);
                                                                            if v0_59 == 0 {
                                                                                f0_59 = lwc1(g::updateframesf);
                                                                                f7_5 = lwc1(s2_1 + 24) + lwc1(sp);
                                                                                f5_14 = lwc1(s2_1 + 28);
                                                                                f1_27 = lwc1(0x3ad218);
                                                                                f4_14 = lwc1(s2_1 + 52);
                                                                                swc1(s2_1 + 24, f7_5);
                                                                                f6_11 = lwc1(s2_1 + 32);
                                                                                f3_15 = lwc1(s2_1 + 48);
                                                                                f1_28 = lwc1(s2_1 + 56);
                                                                                f0_60 = lwc1(0x3ad21c);
                                                                                v0_60 = lw(s2_1 + 96);
                                                                                swc1(s2_1 + 28, f5_14 + lwc1(sp + 4));
                                                                                f2_21 = lwc1(sp + 8);
                                                                                swc1(s2_1 + 56, f1_28 * f0_60);
                                                                                swc1(s2_1 + 48, f3_15 * f0_60);
                                                                                swc1(s2_1 + 52, (f4_14 + f0_59 * f1_27) * f0_60);
                                                                                swc1(s2_1 + 32, f6_11 + f2_21);
                                                                                swc1(v0_60 + 48, f7_5);
                                                                                swc1(v0_60 + 52, lwc1(s2_1 + 28));
                                                                                swc1(v0_60 + 56, lwc1(s2_1 + 32));
                                                                                v0_61 = newrnd();
                                                                                a0_94 = lw(s2_1 + 96);
                                                                                if v0_61 < 0_i64 {
                                                                                    f1_30 = (v0_61 & 1_i64 | ((v0_61 as i32 as u32) >> 1) as i32 as i64) as i32 as f32;
                                                                                    f1_29 = f1_30 + f1_30;
                                                                                } else {
                                                                                    f1_29 = v0_61 as i32 as f32;
                                                                                }
                                                                                swc1(a0_94 + 100, f1_29 * 2.3283064e-10_f32);
                                                                                v0_62 = newrnd();
                                                                                a0_95 = lw(s2_1 + 96);
                                                                                if v0_62 < 0_i64 {
                                                                                    f1_32 = (v0_62 & 1_i64 | ((v0_62 as i32 as u32) >> 1) as i32 as i64) as i32 as f32;
                                                                                    f1_31 = f1_32 + f1_32;
                                                                                } else {
                                                                                    f1_31 = v0_62 as i32 as f32;
                                                                                }
                                                                                swc1(a0_95 + 104, f1_31 * 2.3283064e-10_f32);
                                                                                v0_63 = newrnd();
                                                                                a0_96 = lw(s2_1 + 96);
                                                                                if v0_63 < 0_i64 {
                                                                                    f2_23 = (v0_63 & 1_i64 | ((v0_63 as i32 as u32) >> 1) as i32 as i64) as i32 as f32;
                                                                                    f2_22 = f2_23 + f2_23;
                                                                                    f1_33 = lwc1(a0_96 + 104);
                                                                                } else {
                                                                                    f2_22 = v0_63 as i32 as f32;
                                                                                    f1_33 = lwc1(a0_96 + 104);
                                                                                }
                                                                                f12_26 = lwc1(a0_96 + 100);
                                                                                f0_61 = f2_22 * 2.3283064e-10_f32;
                                                                                swc1(a0_96 + 108, f0_61);
                                                                                f12_27 = f12_26 * f12_26 + f1_33 * f1_33 + f0_61 * f0_61;
                                                                                v0_64 = fptodp(f12_27);
                                                                                a0_97 = v0_64;
                                                                                v0_65 = sqrt(a0_97);
                                                                                a0_98 = 0x3ff0000000000000_i64;
                                                                                a1_81 = v0_65;
                                                                                v0_66 = dpdiv(a0_98, a1_81);
                                                                                a0_99 = v0_66;
                                                                                f0_62 = dptofp(a0_99);
                                                                                a1_80 = lw(s2_1 + 96);
                                                                                f1_34 = lwc1(a1_80 + 104) * f0_62;
                                                                                f2_24 = lwc1(a1_80 + 108) * f0_62;
                                                                                swc1(a1_80 + 100, lwc1(a1_80 + 100) * f0_62);
                                                                                swc1(a1_80 + 104, f1_34);
                                                                                swc1(a1_80 + 108, f2_24);
                                                                            } else {
                                                                                'b49: {
                                                                                    'b50: {
                                                                                        f2_19 = lwc1(0x3ad208);
                                                                                        f4_12 = lwc1(s2_1 + 48);
                                                                                        f6_9 = lwc1(s2_1 + 52);
                                                                                        f5_12 = lwc1(s2_1 + 56);
                                                                                        f7_4 = lwc1(0x3ad20c);
                                                                                        f9_2 = lwc1(0x3ad210);
                                                                                        swc1(s2_1 + 24, lwc1(sp + 48) + f0_55 * f2_19);
                                                                                        swc1(s2_1 + 28, lwc1(sp + 52) + lwc1(sp + 68) * f2_19);
                                                                                        swc1(s2_1 + 32, lwc1(sp + 56) + lwc1(sp + 72) * f2_19);
                                                                                        f3_14 = lwc1(sp + 64);
                                                                                        f2_20 = (f4_12 * f3_14 + f6_9 * lwc1(sp + 68) + f5_12 * lwc1(sp + 72)) * -2.0_f32;
                                                                                        f4_13 = (f4_12 + f3_14 * f2_20) * f7_4;
                                                                                        swc1(s2_1 + 48, f4_13);
                                                                                        f6_10 = (f6_9 + lwc1(sp + 68) * f2_20) * f7_4;
                                                                                        swc1(s2_1 + 52, f6_10);
                                                                                        f5_13 = (f5_12 + lwc1(sp + 72) * f2_20) * f7_4;
                                                                                        swc1(s2_1 + 56, f5_13);
                                                                                        if !((f4_13 * f4_13 + f6_10 * f6_10 + f5_13 * f5_13) < f9_2) {
                                                                                            f0_56 = lwc1(g::updateframesf);
                                                                                            if lw(g::updateframesi) > 0 {
                                                                                                f0_57 = f0_56 * lwc1(0x3ad214);
                                                                                                a0_91 = lw(s2_1 + 212) + 1;
                                                                                                sw(s2_1 + 212, a0_91);
                                                                                                swc1(s2_1 + 52, f6_10 + f0_57);
                                                                                                if a0_91 == 2 {
                                                                                                    a0_92 = 180;
                                                                                                } else if a0_91 >= 3 {
                                                                                                    if a0_91 != 3 {
                                                                                                        a0_93 = 182;
                                                                                                        if a0_91 == 4 {
                                                                                                            a1_78 = s2_1 + 24;
                                                                                                            soundStart(a0_93, a1_78);
                                                                                                            break 'b50;
                                                                                                        }
                                                                                                        f0_58 = lwc1(s2_1 + 24);
                                                                                                        break 'b49;
                                                                                                    }
                                                                                                    a0_92 = 181;
                                                                                                } else {
                                                                                                    a0_92 = 179;
                                                                                                    if a0_91 != 1 {
                                                                                                        f0_58 = lwc1(s2_1 + 24);
                                                                                                        break 'b49;
                                                                                                    }
                                                                                                }
                                                                                                a1_79 = s2_1 + 24;
                                                                                                soundStart(a0_92, a1_79);
                                                                                                f0_58 = lwc1(s2_1 + 24);
                                                                                                break 'b49;
                                                                                            }
                                                                                        } else {
                                                                                            sw(s2_1 + 48, 0);
                                                                                            sw(s2_1 + 52, 0);
                                                                                            sw(s2_1 + 56, 0);
                                                                                        }
                                                                                    }
                                                                                    f0_58 = lwc1(s2_1 + 24);
                                                                                }
                                                                                a1_80 = lw(s2_1 + 96);
                                                                                swc1(a1_80 + 48, f0_58);
                                                                                swc1(a1_80 + 52, lwc1(s2_1 + 28));
                                                                                swc1(a1_80 + 56, lwc1(s2_1 + 32));
                                                                            }
                                                                            'b52: {
                                                                                a0_100 = lw(s2_1 + 204);
                                                                                if a0_100 != 0 {
                                                                                    if lw(a0_100 + 24) < 100 {
                                                                                        a1_82 = a1_80 + 48;
                                                                                        particleUpdatePos(a0_100, a1_82);
                                                                                        break 'b52;
                                                                                    }
                                                                                }
                                                                                v1_17 = lw(s2_1);
                                                                                if v1_17 == 11 {
                                                                                    s0_13 = 5;
                                                                                } else if v1_17 >= 12 {
                                                                                    s0_13 = 7;
                                                                                    if v1_17 == 12 {
                                                                                        s0_13 = 6;
                                                                                    }
                                                                                } else {
                                                                                    s0_13 = 7;
                                                                                    if v1_17 == 10 {
                                                                                        s0_13 = 4;
                                                                                    }
                                                                                }
                                                                                a1_83 = s0_13;
                                                                                if a0_100 != 0 {
                                                                                    f12_28 = 0.0_f32;
                                                                                    a0_101 = a0_100;
                                                                                    particleUpdateRate(a0_101, f12_28);
                                                                                    sw(lw(s2_1 + 204) + 20, 0);
                                                                                    a1_83 = s0_13;
                                                                                }
                                                                                a0_102 = s2_1 + 204;
                                                                                a2_18 = s2_1 + 24;
                                                                                a3_17 = 20;
                                                                                v0_67 = particleNewTrail(a0_102, a1_83, a2_18, a3_17);
                                                                                sw(s2_1 + 204, v0_67);
                                                                            }
                                                                            a1_76 = lw(s2_1 + 96);
                                                                            break 'b42;
                                                                        }
                                                                        v1_16 = lw(s2_1);
                                                                    }
                                                                    s1_5 = 0;
                                                                    if v1_16 == 10 {
                                                                        s1_5 = 4;
                                                                    } else if v1_16 >= 10 {
                                                                        if v1_16 == 11 {
                                                                            s1_5 = 2;
                                                                        } else if v1_16 == 12 {
                                                                            s1_5 = 3;
                                                                        }
                                                                    }
                                                                    s0_14 = s2_1 + 24;
                                                                    v0_68 = newrnd();
                                                                    a1_84 = s0_14;
                                                                    a0_103 = ((v0_68 as i32 as u32) % (6 as u32)) as i32 + 98;
                                                                    soundStart(a0_103, a1_84);
                                                                    a3_18 = lw(s2_1);
                                                                    a1_85 = s1_5;
                                                                    a6_5 = lw(sp + 224);
                                                                    a0_104 = 0;
                                                                    a2_19 = s0_14;
                                                                    a4_13 = s0_14;
                                                                    a5_5 = s2_1 + 48;
                                                                    a7_5 = s3_3;
                                                                    particleNewExplosion(a0_104, a1_85, a2_19, a3_18, a4_13, a5_5, a6_5, a7_5);
                                                                    a0_105 = lw(s2_1 + 204);
                                                                    if a0_105 != 0 {
                                                                        f12_29 = 0.0_f32;
                                                                        particleUpdateRate(a0_105, f12_29);
                                                                    }
                                                                    f12_30 = 0.0_f32;
                                                                    a0_106 = 0;
                                                                    a1_86 = 13;
                                                                    a2_20 = s0_14;
                                                                    a3_19 = 0;
                                                                    particleNewBurst(a0_106, a1_86, a2_20, a3_19, f12_30);
                                                                    a2_21 = s3_3;
                                                                    a0_107 = lw(s2_1);
                                                                    a1_87 = lw(s2_1 + 100);
                                                                    a3_20 = if a2_21 == 0 { s0_14 } else { sp + 48 };
                                                                    a4_14 = sp + 64;
                                                                    decalNewDecal(a0_107, a1_87, a2_21, a3_20, a4_14);
                                                                    a0_108 = s2_1;
                                                                    bulletFree(a0_108);
                                                                    a1_76 = lw(s2_1 + 96);
                                                                }
                                                                v0_69 = a1_76;
                                                                if a1_76 == 0 {
                                                                    s1_1 = s7_1 + 1;
                                                                    a3_1 = lw(0x3b0138);
                                                                    break 'b37;
                                                                }
                                                                f2_25 = lwc1(a1_76 + 96);
                                                                s1_1 = s7_1 + 1;
                                                                if !(0.0_f32 < f2_25) {
                                                                    a3_1 = lw(0x3b0138);
                                                                    break 'b37;
                                                                }
                                                                f0_63 = f2_25 * lwc1(0x3ad220);
                                                                f1_35 = lwc1(a1_76 + 92) + f0_63;
                                                                swc1(a1_76 + 96, f0_63);
                                                                swc1(a1_76 + 92, f1_35);
                                                                if !(3.6e+02_f32 < f1_35) {
                                                                    a3_1 = lw(0x3b0138);
                                                                } else {
                                                                    a3_1 = lw(0x3b0138);
                                                                    s1_1 = s7_1 + 1;
                                                                    a1_88 = lw(s2_1 + 108);
                                                                    'l58: loop {
                                                                        f0_64 = lwc1(v0_69 + 92) - 3.6e+02_f32;
                                                                        swc1(v0_69 + 92, f0_64);
                                                                        if !(3.6e+02_f32 < f0_64) {
                                                                            break 'b36;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            s1_1 = s7_1 + 1;
                                                        }
                                                        a1_88 = lw(s2_1 + 108);
                                                    }
                                                    if a1_88 < 0 {
                                                        break 'b8;
                                                    }
                                                    a0_17 = lw(g::chrinfo);
                                                    v0_16 = a1_88 * 0x1210;
                                                    break 'b14;
                                                }
                                                12 | 13 | 14 => {
                                                    if lw(g::updateframesi) > 0 {
                                                        'b60: {
                                                            f0_30 = lwc1(sp);
                                                            if a2_1 == 15 {
                                                                f0_30 = lwc1(sp);
                                                                if 6.0_f32 <= lwc1(s2_1 + 72) {
                                                                    s0_4 = s2_1 + 24;
                                                                    v0_22 = newrnd();
                                                                    a1_33 = s0_4;
                                                                    a0_36 = ((v0_22 as i32 as u32) % (6 as u32)) as i32 + 98;
                                                                    soundStart(a0_36, a1_33);
                                                                    a3_11 = lw(s2_1);
                                                                    a0_37 = 0;
                                                                    a6_3 = lw(sp + 224);
                                                                    a1_34 = 0;
                                                                    a2_11 = s0_4;
                                                                    a4_7 = s0_4;
                                                                    a5_3 = s2_1 + 48;
                                                                    a7_3 = 0;
                                                                    particleNewExplosion(a0_37, a1_34, a2_11, a3_11, a4_7, a5_3, a6_3, a7_3);
                                                                    a0_38 = 0;
                                                                    a1_35 = s0_4;
                                                                    particleNewSmoke(a0_38, a1_35);
                                                                    a0_39 = lw(s2_1);
                                                                    a3_12 = s0_4;
                                                                    a1_36 = lw(s2_1 + 100);
                                                                    a2_12 = 0;
                                                                    a4_8 = 0;
                                                                    decalNewDecal(a0_39, a1_36, a2_12, a3_12, a4_8);
                                                                    a0_40 = s2_1;
                                                                    bulletFree(a0_40);
                                                                    a2_1 = lw(s2_1);
                                                                    break 'b60;
                                                                }
                                                            }
                                                            fcc_2 = f0_30 == 0.0_f32;
                                                            f0_31 = lwc1(sp + 4);
                                                            if fcc_2 {
                                                                f0_32 = lwc1(sp + 8);
                                                                if f0_31 == 0.0_f32 {
                                                                    if f0_32 == 0.0_f32 {
                                                                        s0_5 = lw(s2_1 + 224);
                                                                        if s0_5 == 0 {
                                                                            break 'b60;
                                                                        }
                                                                        if lw(s0_5 + 8) != 8 {
                                                                            f1_17 = lwc1(s2_1 + 232);
                                                                        } else {
                                                                            f0_33 = lwc1(g::updateframesf);
                                                                            if (lw(lw(s0_5 + 352) + 2708) & 32) != 0 {
                                                                                f1_16 = lwc1(0x3ad288);
                                                                                sw(s2_1 + 224, 0);
                                                                                swc1(s2_1 + 52, f0_33 * f1_16);
                                                                                break 'b60;
                                                                            }
                                                                            f1_17 = lwc1(s2_1 + 232);
                                                                        }
                                                                        v1_6 = lw(s2_1 + 228);
                                                                        swc1(sp + 144, f1_17);
                                                                        swc1(sp + 148, lwc1(s2_1 + 236));
                                                                        swc1(sp + 152, lwc1(s2_1 + 240));
                                                                        swc1(sp + 160, lwc1(s2_1 + 244));
                                                                        swc1(sp + 164, lwc1(s2_1 + 248));
                                                                        swc1(sp + 168, lwc1(s2_1 + 252));
                                                                        if v1_6 >= 0 {
                                                                            a0_41 = lw(sp + 232);
                                                                            a1_37 = lw(lw(s0_5 + 32) + 4) + (v1_6 << 6);
                                                                            sceVu0CopyMatrix(a0_41, a1_37);
                                                                            a0_42 = lw(sp + 232);
                                                                            a1_38 = lw(sp + 236);
                                                                            matrixVecMulAligned(&[a0_42 as i64, a1_38 as i64]);
                                                                            a0_43 = lw(sp + 232);
                                                                            a1_39 = lw(sp + 240);
                                                                            sw(sp + 128, 0);
                                                                            sw(sp + 132, 0);
                                                                            sw(sp + 136, 0);
                                                                            matrixVecMulAligned(&[a0_43 as i64, a1_39 as i64]);
                                                                        }
                                                                        if lw(s0_5 + 140) != 2 {
                                                                            f15_2 = lwc1(s0_5 + 76);
                                                                            a0_46 = lw(sp + 232);
                                                                            f14_2 = lwc1(s0_5 + 56);
                                                                            f12_12 = lwc1(s0_5 + 48);
                                                                            f15_3 = f15_2 * lwc1(0x3ad290) / 1.8e+02_f32;
                                                                            f13_2 = lwc1(s0_5 + 52);
                                                                            matrixTransRotY(a0_46, f12_12, f13_2, f14_2, f15_3);
                                                                            a0_45 = lw(sp + 232);
                                                                        } else {
                                                                            f1_18 = lwc1(0x3ad28c);
                                                                            a0_44 = lw(sp + 232);
                                                                            f15_1 = lwc1(s0_5 + 72) * f1_18 / 1.8e+02_f32;
                                                                            f14_1 = lwc1(s0_5 + 56);
                                                                            f16_1 = lwc1(s0_5 + 76) * f1_18 / 1.8e+02_f32;
                                                                            f12_11 = lwc1(s0_5 + 48);
                                                                            f17_1 = lwc1(s0_5 + 88) * f1_18 / 1.8e+02_f32;
                                                                            f13_1 = lwc1(s0_5 + 52);
                                                                            matrixTransRotZXY(a0_44, f12_11, f13_1, f14_1, f15_1, f16_1, f17_1);
                                                                            a0_45 = lw(sp + 232);
                                                                        }
                                                                        s3_2 = ld(0x3a8b00);
                                                                        a1_40 = lw(sp + 236);
                                                                        matrixVecMulAligned(&[a0_45 as i64, a1_40 as i64]);
                                                                        s1_3 = ld(0x3a8b08);
                                                                        a0_47 = lw(sp + 232);
                                                                        a1_41 = lw(sp + 240);
                                                                        sw(sp + 128, 0);
                                                                        sw(sp + 132, 0);
                                                                        sw(sp + 136, 0);
                                                                        matrixVecMulAligned(&[a0_47 as i64, a1_41 as i64]);
                                                                        f1_19 = lwc1(sp + 144);
                                                                        v0_23 = lw(s2_1 + 96);
                                                                        swc1(s2_1 + 24, f1_19);
                                                                        swc1(v0_23 + 48, f1_19);
                                                                        f0_34 = lwc1(sp + 148);
                                                                        swc1(s2_1 + 28, f0_34);
                                                                        swc1(v0_23 + 52, f0_34);
                                                                        f1_20 = lwc1(sp + 152);
                                                                        swc1(s2_1 + 32, f1_20);
                                                                        swc1(v0_23 + 56, f1_20);
                                                                        swc1(s2_1 + 128, lwc1(sp + 160));
                                                                        swc1(s2_1 + 132, lwc1(sp + 164));
                                                                        swc1(s2_1 + 136, lwc1(sp + 168));
                                                                        f12_13 = lwc1(sp + 168);
                                                                        v0_24 = fptodp(f12_13);
                                                                        f12_14 = lwc1(sp + 164);
                                                                        v0_25 = fptodp(f12_14);
                                                                        a0_48 = v0_24;
                                                                        a1_42 = v0_25;
                                                                        v0_26 = atan2(a0_48, a1_42);
                                                                        a0_49 = v0_26;
                                                                        a1_43 = s3_2;
                                                                        v0_27 = dpmul(a0_49, a1_43);
                                                                        a0_50 = v0_27;
                                                                        a1_44 = s1_3;
                                                                        v0_28 = dpdiv(a0_50, a1_44);
                                                                        s0_6 = lw(s2_1 + 96);
                                                                        a0_51 = v0_28;
                                                                        f0_35 = dptofp(a0_51);
                                                                        swc1(s0_6 + 72, f0_35);
                                                                        f12_15 = lwc1(sp + 160);
                                                                        v0_29 = fptodp(f12_15);
                                                                        f1_21 = lwc1(sp + 164);
                                                                        f0_36 = lwc1(sp + 168);
                                                                        f12_16 = (f1_21 * f1_21 + f0_36 * f0_36).abs().sqrt();
                                                                        v0_30 = fptodp(f12_16);
                                                                        a0_52 = v0_29;
                                                                        a1_45 = v0_30;
                                                                        v0_31 = atan2(a0_52, a1_45);
                                                                        a0_53 = v0_31;
                                                                        a1_46 = s3_2;
                                                                        v0_32 = dpmul(a0_53, a1_46);
                                                                        a0_54 = v0_32;
                                                                        a1_47 = s1_3;
                                                                        v0_33 = dpdiv(a0_54, a1_47);
                                                                        s0_7 = lw(s2_1 + 96);
                                                                        a1_48 = v0_33;
                                                                        a0_55 = 0;
                                                                        v0_34 = dpsub(a0_55 as i64, a1_48);
                                                                        a0_56 = v0_34;
                                                                        f0_37 = dptofp(a0_56);
                                                                        swc1(s0_7 + 88, f0_37);
                                                                        sw(s0_7 + 76, 0);
                                                                        a2_1 = lw(s2_1);
                                                                        break 'b60;
                                                                    }
                                                                }
                                                            }
                                                            a4_9 = lw(sp + 244);
                                                            a0_57 = s2_1;
                                                            a1_49 = sp;
                                                            a2_13 = sp + 48;
                                                            a3_13 = sp + 64;
                                                            v0_35 = bulletNonStraightLineTest(a0_57, a1_49, a2_13, a3_13, a4_9);
                                                            a1_50 = lw(sp + 208);
                                                            if a1_50 != 0 {
                                                                a0_58 = s2_1;
                                                                bulletProcessGlasshits(a0_58, a1_50);
                                                            }
                                                            s4_1 = s2_1 + 24;
                                                            if v0_35 == 0 {
                                                                f0_46 = lwc1(g::updateframesf);
                                                                f5_11 = lwc1(s2_1 + 24) + lwc1(sp);
                                                                f3_12 = lwc1(s2_1 + 28);
                                                                f1_25 = lwc1(0x3ad284);
                                                                f2_16 = lwc1(s2_1 + 52);
                                                                swc1(s2_1 + 24, f5_11);
                                                                f4_11 = lwc1(s2_1 + 32);
                                                                v0_56 = lw(s2_1 + 96);
                                                                a2_1 = lw(s2_1);
                                                                swc1(s2_1 + 28, f3_12 + lwc1(sp + 4));
                                                                f0_47 = lwc1(sp + 8);
                                                                swc1(s2_1 + 52, f2_16 + f0_46 * f1_25);
                                                                swc1(s2_1 + 32, f4_11 + f0_47);
                                                                swc1(v0_56 + 48, f5_11);
                                                                f0_48 = lwc1(v0_56 + 76) + 12.0_f32;
                                                                swc1(v0_56 + 52, lwc1(s2_1 + 28));
                                                                f2_17 = lwc1(s2_1 + 32);
                                                                swc1(v0_56 + 76, f0_48);
                                                                swc1(v0_56 + 56, f2_17);
                                                            } else {
                                                                'b67: {
                                                                    a0_59 = 194;
                                                                    a1_51 = s4_1;
                                                                    soundStart(a0_59, a1_51);
                                                                    v1_7 = lw(v0_35 + 8);
                                                                    if v1_7 != 2 {
                                                                        v0_37 = lw(v0_35 + 32);
                                                                        if v1_7 != 8 {
                                                                            a0_61 = lw(sp + 256);
                                                                            a1_53 = 0;
                                                                            s0_9 = lw(v0_37);
                                                                            a2_15 = 0;
                                                                            s0_10 = s0_9 - lw(s0_9) * 80;
                                                                            obGetLastHitInst(a0_61, a1_53, a2_15);
                                                                            v1_11 = lw(sp + 220);
                                                                            sw(s2_1 + 224, v0_35);
                                                                            v1_10 = v1_11 * 80 + s0_10;
                                                                        } else {
                                                                            f5_9 = lwc1(sp);
                                                                            f4_9 = lwc1(sp + 4);
                                                                            f3_8 = lwc1(sp + 8);
                                                                            v1_8 = lw(v0_37);
                                                                            s0_8 = v1_8 - lw(v1_8) * 80;
                                                                            a1_52 = s4_1;
                                                                            a0_60 = v0_35;
                                                                            a2_14 = sp;
                                                                            f0_38 = 1.0_f32 / (f5_9 * f5_9 + f4_9 * f4_9 + f3_8 * f3_8).abs().sqrt();
                                                                            a3_14 = 0;
                                                                            a4_10 = 1;
                                                                            a5_4 = sp + 48;
                                                                            a6_4 = sp + 64;
                                                                            a7_4 = 2;
                                                                            swc1(sp + 8, f3_8 * f0_38);
                                                                            swc1(sp, f5_9 * f0_38);
                                                                            swc1(sp + 4, f4_9 * f0_38);
                                                                            chrLineTestChr(a0_60, a1_52, a2_14, a3_14, a4_10, a5_4, a6_4, a7_4);
                                                                            sw(s2_1 + 224, v0_35);
                                                                            v0_38 = chrGetLastHitPart();
                                                                            v1_9 = lb(v0_38 * 80 + s0_8 + 1);
                                                                            sw(s2_1 + 228, v1_9);
                                                                            if v1_9 >= 0 {
                                                                                break 'b67;
                                                                            }
                                                                            v0_39 = chrGetLastHitPart();
                                                                            if lb(v0_39 * 80 + s0_8 + 2) < 0 {
                                                                                v0_41 = lw(s2_1 + 228);
                                                                            } else {
                                                                                v0_40 = chrGetLastHitPart();
                                                                                sw(s2_1 + 228, lb(lb(v0_40 * 80 + s0_8 + 2) * 80 + s0_8 + 1));
                                                                                v0_41 = lw(s2_1 + 228);
                                                                            }
                                                                            if v0_41 >= 0 {
                                                                                break 'b67;
                                                                            }
                                                                            v0_42 = chrGetLastHitPart();
                                                                            if lb(v0_42 * 80 + s0_8 + 3) < 0 {
                                                                                break 'b67;
                                                                            }
                                                                            v0_43 = chrGetLastHitPart();
                                                                            v1_10 = lb(v0_43 * 80 + s0_8 + 3) * 80 + s0_8;
                                                                        }
                                                                        v0_36 = lb(v1_10 + 1);
                                                                    } else {
                                                                        v0_36 = -1;
                                                                        sw(s2_1 + 224, 0);
                                                                    }
                                                                    sw(s2_1 + 228, v0_36);
                                                                }
                                                                s1_4 = ld(0x3a8af0);
                                                                f12_17 = lwc1(sp + 72);
                                                                v0_44 = fptodp(f12_17);
                                                                s4_2 = ld(0x3a8af8);
                                                                f12_18 = lwc1(sp + 68);
                                                                v0_45 = fptodp(f12_18);
                                                                a0_62 = v0_44;
                                                                a1_54 = v0_45;
                                                                v0_46 = atan2(a0_62, a1_54);
                                                                a0_63 = v0_46;
                                                                a1_55 = s1_4;
                                                                v0_47 = dpmul(a0_63, a1_55);
                                                                a0_64 = v0_47;
                                                                a1_56 = s4_2;
                                                                v0_48 = dpdiv(a0_64, a1_56);
                                                                s0_11 = lw(s2_1 + 96);
                                                                a0_65 = v0_48;
                                                                f0_39 = dptofp(a0_65);
                                                                swc1(s0_11 + 72, f0_39);
                                                                f12_19 = lwc1(sp + 64);
                                                                v0_49 = fptodp(f12_19);
                                                                f1_22 = lwc1(sp + 68);
                                                                f0_40 = lwc1(sp + 72);
                                                                f12_20 = (f1_22 * f1_22 + f0_40 * f0_40).abs().sqrt();
                                                                v0_50 = fptodp(f12_20);
                                                                a0_66 = v0_49;
                                                                a1_57 = v0_50;
                                                                v0_51 = atan2(a0_66, a1_57);
                                                                a0_67 = v0_51;
                                                                a1_58 = s1_4;
                                                                v0_52 = dpmul(a0_67, a1_58);
                                                                a0_68 = v0_52;
                                                                a1_59 = s4_2;
                                                                v0_53 = dpdiv(a0_68, a1_59);
                                                                s0_12 = lw(s2_1 + 96);
                                                                a1_60 = v0_53;
                                                                a0_69 = 0;
                                                                v0_54 = dpsub(a0_69 as i64, a1_60);
                                                                a0_70 = v0_54;
                                                                f0_41 = dptofp(a0_70);
                                                                a1_61 = s0_12;
                                                                swc1(s0_12 + 88, f0_41);
                                                                v0_55 = lw(s2_1 + 224);
                                                                sw(a1_61 + 76, 0);
                                                                if v0_55 != 0 {
                                                                    if lw(v0_35 + 140) != 2 {
                                                                        f12_22 = lwc1(v0_35 + 76);
                                                                        f12_23 = f12_22 * lwc1(0x3ad27c) / 1.8e+02_f32;
                                                                        a0_72 = lw(sp + 232);
                                                                        matrixRotY(a0_72, f12_23);
                                                                        f0_42 = lwc1(v0_35 + 48);
                                                                    } else {
                                                                        f1_23 = lwc1(0x3ad278);
                                                                        f12_21 = lwc1(v0_35 + 72) * f1_23 / 1.8e+02_f32;
                                                                        f13_3 = lwc1(v0_35 + 76) * f1_23 / 1.8e+02_f32;
                                                                        f14_3 = lwc1(v0_35 + 88) * f1_23 / 1.8e+02_f32;
                                                                        a0_71 = lw(sp + 232);
                                                                        matrixRotZXY(a0_71, f12_21, f13_3, f14_3);
                                                                        f0_42 = lwc1(v0_35 + 48);
                                                                    }
                                                                    f2_14 = lwc1(sp + 52);
                                                                    f3_9 = lwc1(sp + 56);
                                                                    a0_73 = lw(sp + 236);
                                                                    a1_62 = lw(sp + 232);
                                                                    swc1(sp + 48, lwc1(sp + 48) - f0_42);
                                                                    swc1(sp + 52, f2_14 - lwc1(v0_35 + 52));
                                                                    swc1(sp + 56, f3_9 - lwc1(v0_35 + 56));
                                                                    sceVu0InversMatrix(a0_73, a1_62);
                                                                    a0_74 = lw(sp + 236);
                                                                    a1_63 = sp + 48;
                                                                    matrixVecMul(a0_74, a1_63);
                                                                    a0_75 = lw(sp + 236);
                                                                    a1_64 = sp + 64;
                                                                    matrixVecMul(a0_75, a1_64);
                                                                    v1_12 = lw(s2_1 + 228);
                                                                    if v1_12 >= 0 {
                                                                        a0_76 = lw(sp + 232);
                                                                        a1_65 = lw(lw(v0_35 + 32) + 4) + (v1_12 << 6);
                                                                        sceVu0CopyMatrix(a0_76, a1_65);
                                                                        f1_24 = lwc1(sp + 52) - lwc1(sp + 132);
                                                                        a0_77 = lw(sp + 236);
                                                                        f0_43 = lwc1(sp + 56) - lwc1(sp + 136);
                                                                        a1_66 = lw(sp + 232);
                                                                        swc1(sp + 48, lwc1(sp + 48) - lwc1(sp + 128));
                                                                        sw(sp + 128, 0);
                                                                        swc1(sp + 52, f1_24);
                                                                        sw(sp + 132, 0);
                                                                        swc1(sp + 56, f0_43);
                                                                        sw(sp + 136, 0);
                                                                        sceVu0InversMatrix(a0_77, a1_66);
                                                                        a0_78 = lw(sp + 236);
                                                                        a1_67 = sp + 48;
                                                                        matrixVecMul(a0_78, a1_67);
                                                                        a0_79 = lw(sp + 236);
                                                                        a1_68 = sp + 64;
                                                                        matrixVecMul(a0_79, a1_68);
                                                                    }
                                                                    f6_8 = lwc1(sp + 64);
                                                                    f5_10 = lwc1(sp + 68);
                                                                    f4_10 = lwc1(sp + 72);
                                                                    f3_10 = lwc1(sp + 48);
                                                                    f0_44 = 1.0_f32 / (f6_8 * f6_8 + f5_10 * f5_10 + f4_10 * f4_10).abs().sqrt();
                                                                    a1_61 = lw(s2_1 + 96);
                                                                    swc1(sp + 72, f4_10 * f0_44);
                                                                    swc1(sp + 64, f6_8 * f0_44);
                                                                    swc1(sp + 68, f5_10 * f0_44);
                                                                    swc1(s2_1 + 232, f3_10);
                                                                    swc1(s2_1 + 236, lwc1(sp + 52));
                                                                    swc1(s2_1 + 240, lwc1(sp + 56));
                                                                    swc1(s2_1 + 244, lwc1(sp + 64));
                                                                    swc1(s2_1 + 248, lwc1(sp + 68));
                                                                    swc1(s2_1 + 252, lwc1(sp + 72));
                                                                }
                                                                f2_15 = lwc1(0x3ad280);
                                                                a2_1 = lw(s2_1);
                                                                f3_11 = lwc1(sp + 48) + lwc1(sp + 64) * f2_15;
                                                                swc1(s2_1 + 24, f3_11);
                                                                swc1(s2_1 + 28, lwc1(sp + 52) + lwc1(sp + 68) * f2_15);
                                                                swc1(s2_1 + 32, lwc1(sp + 56) + lwc1(sp + 72) * f2_15);
                                                                swc1(s2_1 + 128, lwc1(sp + 64));
                                                                swc1(s2_1 + 132, lwc1(sp + 68));
                                                                f0_45 = lwc1(sp + 72);
                                                                sw(s2_1 + 48, 0);
                                                                swc1(s2_1 + 136, f0_45);
                                                                sw(s2_1 + 52, 0);
                                                                sw(s2_1 + 56, 0);
                                                                swc1(a1_61 + 48, f3_11);
                                                                swc1(a1_61 + 52, lwc1(s2_1 + 28));
                                                                swc1(a1_61 + 56, lwc1(s2_1 + 32));
                                                            }
                                                        }
                                                        if a2_1 == 16 {
                                                            a1_69 = lw(sp + 224);
                                                            a0_80 = s2_1;
                                                            updateProximityProps(a0_80, a1_69);
                                                            a2_1 = lw(s2_1);
                                                        }
                                                    }
                                                    'b74: {
                                                        if a2_1 == 15 {
                                                            f2_18 = lwc1(g::updateframesf);
                                                            f3_13 = lwc1(s2_1 + 124);
                                                            f0_49 = lwc1(s2_1 + 64) + f2_18 * lwc1(0x3ad294);
                                                            swc1(s2_1 + 64, f0_49);
                                                            f0_50 = f3_13 + f0_49 * f2_18;
                                                            swc1(s2_1 + 124, f0_50);
                                                            if 3.6e+02_f32 < f0_50 {
                                                                swc1(s2_1 + 124, f0_50 - 3.6e+02_f32);
                                                            }
                                                            a1_70 = 1;
                                                            f12_24 = 0.0_f32;
                                                            a0_81 = lw(lw(s2_1 + 96) + 32);
                                                            obInstSetAnim(a0_81, a1_70, f12_24);
                                                            v1_13 = lw(s2_1 + 96);
                                                            if (lw(v1_13 + 16) & 256) != 0 {
                                                                f0_51 = lwc1(s2_1 + 72);
                                                            } else {
                                                                a0_82 = lw(lw(v1_13 + 32) + 4);
                                                                sceVu0UnitMatrix(a0_82);
                                                                a0_83 = lw(lw(lw(s2_1 + 96) + 32) + 4) + 128;
                                                                sceVu0UnitMatrix(a0_83);
                                                                f12_25 = lwc1(s2_1 + 124) * lwc1(0x3ad298) / 1.8e+02_f32;
                                                                a0_84 = lw(lw(lw(s2_1 + 96) + 32) + 4) + 64;
                                                                matrixRotY(a0_84, f12_25);
                                                                f0_51 = lwc1(s2_1 + 72);
                                                            }
                                                            v1_14 = lw(s2_1 + 260);
                                                            v0_57 = v1_14 + 1;
                                                            if v1_14 < (f0_51 + f0_51 - 1.0_f32) as i32 {
                                                                sw(s2_1 + 260, v0_57);
                                                                if v0_57 < 10 {
                                                                    a0_85 = 191;
                                                                    a1_71 = s2_1 + 24;
                                                                    soundStart(a0_85, a1_71);
                                                                }
                                                                if lw(s2_1 + 260) != 10 {
                                                                    a2_1 = lw(s2_1);
                                                                    break 'b74;
                                                                }
                                                                a0_86 = 192;
                                                                a1_72 = s2_1 + 24;
                                                                soundStart(a0_86, a1_72);
                                                            }
                                                            a2_1 = lw(s2_1);
                                                        }
                                                    }
                                                    s1_1 = s7_1 + 1;
                                                    if a2_1 == 16 {
                                                        a0_87 = 193;
                                                        if lw(s2_1 + 260) >= (lwc1(s2_1 + 72) * 0.5_f32) as i32 {
                                                            a3_1 = lw(0x3b0138);
                                                            break 'b9;
                                                        }
                                                        a1_73 = s2_1 + 24;
                                                        soundStart(a0_87, a1_73);
                                                        s1_1 = s7_1 + 1;
                                                        a3_1 = lw(0x3b0138);
                                                        sw(s2_1 + 260, lw(s2_1 + 260) + 1);
                                                        break 'b8;
                                                    }
                                                }
                                                15 | 16 => {
                                                    f1_9 = 1.0_f32;
                                                    f0_16 = lwc1(s2_1 + 72);
                                                    if a2_1 == 17 {
                                                        f1_9 = lwc1(0x3ad224);
                                                    }
                                                    f0_17 = lwc1(sp);
                                                    if !(f1_9 <= f0_16) {
                                                        'b81: {
                                                            'b82: {
                                                                f0_18 = lwc1(sp + 4);
                                                                if f0_17 == 0.0_f32 {
                                                                    f0_19 = lwc1(sp + 8);
                                                                    if f0_18 == 0.0_f32 {
                                                                        if f0_19 == 0.0_f32 {
                                                                            f1_10 = lwc1(s2_1 + 60);
                                                                            v0_18 = lw(s2_1 + 96);
                                                                            f3_5 = lwc1(0x3ad248);
                                                                            swc1(v0_18 + 72, f1_10 + 9e+01_f32);
                                                                            swc1(v0_18 + 76, lwc1(s2_1 + 64));
                                                                            swc1(v0_18 + 88, lwc1(s2_1 + 68));
                                                                            f1_11 = lwc1(s2_1 + 64) * f3_5;
                                                                            f2_7 = lwc1(s2_1 + 68) * f3_5;
                                                                            swc1(s2_1 + 60, lwc1(s2_1 + 60) * f3_5);
                                                                            swc1(s2_1 + 64, f1_11);
                                                                            swc1(s2_1 + 68, f2_7);
                                                                            break 'b82;
                                                                        }
                                                                    }
                                                                }
                                                                a4_6 = lw(sp + 244);
                                                                a0_25 = s2_1;
                                                                a1_21 = sp;
                                                                a2_8 = sp + 48;
                                                                a3_8 = sp + 64;
                                                                v0_19 = bulletNonStraightLineTest(a0_25, a1_21, a2_8, a3_8, a4_6);
                                                                a1_22 = lw(sp + 208);
                                                                if a1_22 != 0 {
                                                                    a0_26 = s2_1;
                                                                    bulletProcessGlasshits(a0_26, a1_22);
                                                                }
                                                                'b85: {
                                                                    f0_20 = lwc1(sp + 64);
                                                                    if v0_19 == 0 {
                                                                        f0_22 = lwc1(g::updateframesf);
                                                                        f7_3 = lwc1(s2_1 + 24) + lwc1(sp);
                                                                        f5_8 = lwc1(s2_1 + 28);
                                                                        f1_12 = lwc1(0x3ad240);
                                                                        f4_8 = lwc1(s2_1 + 52);
                                                                        swc1(s2_1 + 24, f7_3);
                                                                        f6_7 = lwc1(s2_1 + 32);
                                                                        f3_7 = lwc1(s2_1 + 48);
                                                                        f1_13 = lwc1(s2_1 + 56);
                                                                        f0_23 = lwc1(0x3ad244);
                                                                        a1_23 = lw(s2_1 + 96);
                                                                        swc1(s2_1 + 28, f5_8 + lwc1(sp + 4));
                                                                        f2_11 = lwc1(sp + 8);
                                                                        swc1(s2_1 + 56, f1_13 * f0_23);
                                                                        swc1(s2_1 + 48, f3_7 * f0_23);
                                                                        swc1(s2_1 + 52, (f4_8 + f0_22 * f1_12) * f0_23);
                                                                        swc1(s2_1 + 32, f6_7 + f2_11);
                                                                        swc1(a1_23 + 48, f7_3);
                                                                        f2_12 = lwc1(a1_23 + 72);
                                                                        swc1(a1_23 + 52, lwc1(s2_1 + 28));
                                                                        swc1(a1_23 + 56, lwc1(s2_1 + 32));
                                                                        f2_13 = f2_12 + lwc1(s2_1 + 60);
                                                                        swc1(a1_23 + 72, f2_13);
                                                                        if f2_13 < 0.0_f32 {
                                                                            swc1(a1_23 + 72, f2_13 + 3.6e+02_f32);
                                                                        }
                                                                        f0_24 = lwc1(a1_23 + 72);
                                                                        if !(3.6e+02_f32 <= f0_24) {
                                                                            f1_14 = lwc1(a1_23 + 76);
                                                                        } else {
                                                                            swc1(a1_23 + 72, f0_24 - 3.6e+02_f32);
                                                                            f1_14 = lwc1(a1_23 + 76);
                                                                        }
                                                                        f0_25 = f1_14 + lwc1(s2_1 + 64);
                                                                        swc1(a1_23 + 76, f0_25);
                                                                        if f0_25 < 0.0_f32 {
                                                                            swc1(a1_23 + 76, f0_25 + 3.6e+02_f32);
                                                                        }
                                                                        f0_26 = lwc1(a1_23 + 76);
                                                                        if !(3.6e+02_f32 <= f0_26) {
                                                                            f1_15 = lwc1(a1_23 + 88);
                                                                        } else {
                                                                            swc1(a1_23 + 76, f0_26 - 3.6e+02_f32);
                                                                            f1_15 = lwc1(a1_23 + 88);
                                                                        }
                                                                        f0_27 = f1_15 + lwc1(s2_1 + 68);
                                                                        swc1(a1_23 + 88, f0_27);
                                                                        if f0_27 < 0.0_f32 {
                                                                            swc1(a1_23 + 88, f0_27 + 3.6e+02_f32);
                                                                        }
                                                                        f0_28 = lwc1(a1_23 + 88);
                                                                        if !(3.6e+02_f32 <= f0_28) {
                                                                            a0_32 = lw(s2_1 + 204);
                                                                            break 'b85;
                                                                        }
                                                                        swc1(a1_23 + 88, f0_28 - 3.6e+02_f32);
                                                                    } else {
                                                                        f2_8 = lwc1(0x3ad228);
                                                                        f4_5 = lwc1(s2_1 + 48);
                                                                        f6_5 = lwc1(s2_1 + 52);
                                                                        f5_5 = lwc1(s2_1 + 56);
                                                                        f7_2 = lwc1(0x3ad22c);
                                                                        f9_1 = lwc1(0x3ad230);
                                                                        swc1(s2_1 + 24, lwc1(sp + 48) + f0_20 * f2_8);
                                                                        swc1(s2_1 + 28, lwc1(sp + 52) + lwc1(sp + 68) * f2_8);
                                                                        swc1(s2_1 + 32, lwc1(sp + 56) + lwc1(sp + 72) * f2_8);
                                                                        f3_6 = lwc1(sp + 64);
                                                                        f2_9 = (f4_5 * f3_6 + f6_5 * lwc1(sp + 68) + f5_5 * lwc1(sp + 72)) * -2.0_f32;
                                                                        f4_6 = (f4_5 + f3_6 * f2_9) * f7_2;
                                                                        swc1(s2_1 + 48, f4_6);
                                                                        f6_6 = (f6_5 + lwc1(sp + 68) * f2_9) * f7_2;
                                                                        swc1(s2_1 + 52, f6_6);
                                                                        f5_6 = (f5_5 + lwc1(sp + 72) * f2_9) * f7_2;
                                                                        swc1(s2_1 + 56, f5_6);
                                                                        if !((f4_6 * f4_6 + f6_6 * f6_6 + f5_6 * f5_6) < f9_1) {
                                                                            f5_7 = lwc1(0x3ad238);
                                                                            f0_21 = lwc1(s2_1 + 68) * f5_7;
                                                                            f4_7 = lwc1(s2_1 + 60) * lwc1(0x3ad23c);
                                                                            f2_10 = lwc1(s2_1 + 64) * f5_7;
                                                                            a0_27 = lw(s2_1 + 212) + 1;
                                                                            swc1(s2_1 + 52, f6_6 + lwc1(g::updateframesf) * lwc1(0x3ad234));
                                                                            swc1(s2_1 + 60, f4_7);
                                                                            swc1(s2_1 + 64, f2_10);
                                                                            swc1(s2_1 + 68, f0_21);
                                                                            sw(s2_1 + 212, a0_27);
                                                                            if a0_27 == 2 {
                                                                                a0_31 = 180;
                                                                                a1_27 = s2_1 + 24;
                                                                                soundStart(a0_31, a1_27);
                                                                                a1_23 = lw(s2_1 + 96);
                                                                            } else if a0_27 >= 3 {
                                                                                if a0_27 == 3 {
                                                                                    a0_30 = 181;
                                                                                    a1_26 = s2_1 + 24;
                                                                                    soundStart(a0_30, a1_26);
                                                                                    a1_23 = lw(s2_1 + 96);
                                                                                } else {
                                                                                    a0_29 = 182;
                                                                                    if a0_27 == 4 {
                                                                                        a1_25 = s2_1 + 24;
                                                                                        soundStart(a0_29, a1_25);
                                                                                        a1_23 = lw(s2_1 + 96);
                                                                                    } else {
                                                                                        a1_23 = lw(s2_1 + 96);
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                a0_28 = 179;
                                                                                if a0_27 == 1 {
                                                                                    a1_24 = s2_1 + 24;
                                                                                    soundStart(a0_28, a1_24);
                                                                                    a1_23 = lw(s2_1 + 96);
                                                                                } else {
                                                                                    a1_23 = lw(s2_1 + 96);
                                                                                }
                                                                            }
                                                                        } else {
                                                                            sw(s2_1 + 48, 0);
                                                                            sw(s2_1 + 52, 0);
                                                                            sw(s2_1 + 56, 0);
                                                                            a1_23 = lw(s2_1 + 96);
                                                                            swc1(s2_1 + 60, lwc1(a1_23 + 72));
                                                                            swc1(s2_1 + 64, lwc1(a1_23 + 76));
                                                                            swc1(s2_1 + 68, lwc1(a1_23 + 88));
                                                                        }
                                                                        swc1(a1_23 + 48, lwc1(s2_1 + 24));
                                                                        swc1(a1_23 + 52, lwc1(s2_1 + 28));
                                                                        swc1(a1_23 + 56, lwc1(s2_1 + 32));
                                                                    }
                                                                    a0_32 = lw(s2_1 + 204);
                                                                }
                                                                if a0_32 == 0 {
                                                                    a0_33 = s2_1 + 204;
                                                                } else {
                                                                    if lw(a0_32 + 24) < 100 {
                                                                        a1_28 = a1_23 + 48;
                                                                        particleUpdatePos(a0_32, a1_28);
                                                                        a0_34 = lw(s2_1 + 208);
                                                                        break 'b81;
                                                                    }
                                                                    f12_9 = 0.0_f32;
                                                                    particleUpdateRate(a0_32, f12_9);
                                                                    sw(lw(s2_1 + 204) + 20, 0);
                                                                    a0_33 = s2_1 + 204;
                                                                }
                                                                a1_29 = 2;
                                                                a2_9 = s2_1 + 24;
                                                                a3_9 = 10;
                                                                v0_20 = particleNewTrail(a0_33, a1_29, a2_9, a3_9);
                                                                sw(s2_1 + 204, v0_20);
                                                            }
                                                            a0_34 = lw(s2_1 + 208);
                                                        }
                                                        'b94: {
                                                            if a0_34 == 0 {
                                                                a0_35 = s2_1 + 208;
                                                            } else {
                                                                if lw(a0_34 + 24) < 100 {
                                                                    a1_30 = lw(s2_1 + 96);
                                                                    a1_31 = a1_30 + 48;
                                                                    particleUpdatePos(a0_34, a1_31);
                                                                    break 'b94;
                                                                }
                                                                f12_10 = 0.0_f32;
                                                                particleUpdateRate(a0_34, f12_10);
                                                                sw(lw(s2_1 + 208) + 20, 0);
                                                                a0_35 = s2_1 + 208;
                                                            }
                                                            a1_32 = 3;
                                                            a2_10 = s2_1 + 24;
                                                            a3_10 = 80;
                                                            v0_21 = particleNewTrail(a0_35, a1_32, a2_10, a3_10);
                                                            sw(s2_1 + 208, v0_21);
                                                        }
                                                        v1_4 = lw(s2_1 + 108);
                                                    } else {
                                                        s0_3 = s2_1 + 24;
                                                        v0_17 = newrnd();
                                                        a1_17 = s0_3;
                                                        a0_18 = ((v0_17 as i32 as u32) % (6 as u32)) as i32 + 98;
                                                        soundStart(a0_18, a1_17);
                                                        a3_5 = lw(s2_1);
                                                        a0_19 = 0;
                                                        a6_2 = lw(sp + 224);
                                                        a1_18 = 4;
                                                        a2_5 = s0_3;
                                                        a4_4 = s0_3;
                                                        a5_2 = s2_1 + 48;
                                                        a7_2 = 0;
                                                        particleNewExplosion(a0_19, a1_18, a2_5, a3_5, a4_4, a5_2, a6_2, a7_2);
                                                        a0_20 = lw(s2_1 + 204);
                                                        if a0_20 == 0 {
                                                            a0_21 = lw(s2_1 + 208);
                                                        } else {
                                                            f12_6 = 0.0_f32;
                                                            particleUpdateRate(a0_20, f12_6);
                                                            a0_21 = lw(s2_1 + 208);
                                                        }
                                                        if a0_21 != 0 {
                                                            f12_7 = 0.0_f32;
                                                            particleUpdateRate(a0_21, f12_7);
                                                        }
                                                        f12_8 = 0.0_f32;
                                                        a0_22 = 0;
                                                        a1_19 = 13;
                                                        a2_6 = s0_3;
                                                        a3_6 = 0;
                                                        particleNewBurst(a0_22, a1_19, a2_6, a3_6, f12_8);
                                                        a0_23 = lw(s2_1);
                                                        a3_7 = s0_3;
                                                        a1_20 = lw(s2_1 + 100);
                                                        a2_7 = 0;
                                                        a4_5 = 0;
                                                        decalNewDecal(a0_23, a1_20, a2_7, a3_7, a4_5);
                                                        a0_24 = s2_1;
                                                        bulletFree(a0_24);
                                                        v1_4 = lw(s2_1 + 108);
                                                    }
                                                    if v1_4 >= 0 {
                                                        a0_17 = lw(g::chrinfo);
                                                        v0_16 = v1_4 * 0x1210;
                                                        f0_29 = lwc1(s2_1 + 24);
                                                        v1_5 = lw(s2_1);
                                                        s1_1 = s7_1 + 1;
                                                        a3_1 = lw(0x3b0138);
                                                        break 'b13;
                                                    }
                                                    s1_1 = s7_1 + 1;
                                                }
                                                17 => {
                                                    'b98: {
                                                        f0_1 = lwc1(sp);
                                                        if lw(g::updateframesi) > 0 {
                                                            'b99: {
                                                                f0_2 = lwc1(sp + 4);
                                                                if f0_1 == 0.0_f32 {
                                                                    f0_3 = lwc1(sp + 8);
                                                                    if f0_2 == 0.0_f32 {
                                                                        a1_2 = 3;
                                                                        if f0_3 == 0.0_f32 {
                                                                            a2_2 = -1;
                                                                            a3_2 = -1;
                                                                            a0_1 = lw(lw(s2_1 + 96) + 32);
                                                                            a4_1 = (lwc1(s2_1 + 264) * 127.0_f32) as i32 & 255;
                                                                            obInstSetOverrideStates(a0_1, a1_2, a2_2, a3_2, a4_1);
                                                                            f0_4 = lwc1(s2_1 + 264) - lwc1(g::updateframesf) * lwc1(0x3ad274);
                                                                            swc1(s2_1 + 264, f0_4);
                                                                            if !(f0_4 <= 0.0_f32) {
                                                                                break 'b99;
                                                                            }
                                                                            a0_2 = s2_1;
                                                                            bulletFree(a0_2);
                                                                            s2_1 = 0;
                                                                            break 'b99;
                                                                        }
                                                                    }
                                                                }
                                                                a4_2 = lw(sp + 244);
                                                                a0_3 = s2_1;
                                                                a1_3 = sp;
                                                                a2_3 = sp + 48;
                                                                a3_3 = sp + 64;
                                                                v0_2 = bulletNonStraightLineTest(a0_3, a1_3, a2_3, a3_3, a4_2);
                                                                a1_4 = lw(sp + 208);
                                                                if a1_4 != 0 {
                                                                    a0_4 = s2_1;
                                                                    bulletProcessGlasshits(a0_4, a1_4);
                                                                }
                                                                if v0_2 != 0 {
                                                                    f0_5 = lwc1(sp + 64);
                                                                    if lw(v0_2 + 8) == 8 {
                                                                        f12_1 = lwc1(s2_1 + 76);
                                                                        a0_5 = v0_2;
                                                                        a3_4 = lw(s2_1);
                                                                        a2_4 = sp + 48;
                                                                        a1_5 = lw(sp + 224);
                                                                        a4_3 = s2_1 + 24;
                                                                        a5_1 = s2_1 + 48;
                                                                        a6_1 = s2_1;
                                                                        a7_1 = 0;
                                                                        propDamage(a0_5, a1_5, a2_4, a3_4, a4_3, a5_1, a6_1, a7_1, f12_1);
                                                                        f0_5 = lwc1(sp + 64);
                                                                    }
                                                                    'b108: {
                                                                        'b109: {
                                                                            f2_1 = lwc1(0x3ad24c);
                                                                            f3_2 = lwc1(0x3ad250);
                                                                            f4_1 = lwc1(s2_1 + 48);
                                                                            f5_1 = lwc1(s2_1 + 52);
                                                                            f6_1 = lwc1(s2_1 + 56);
                                                                            f8_1 = lwc1(0x3ad254);
                                                                            swc1(s2_1 + 24, lwc1(sp + 48) + f0_5 * f2_1);
                                                                            f10_1 = lwc1(0x3ad258);
                                                                            swc1(s2_1 + 28, lwc1(sp + 52) + lwc1(sp + 68) * f2_1 + f3_2);
                                                                            swc1(s2_1 + 32, lwc1(sp + 56) + lwc1(sp + 72) * f2_1);
                                                                            f3_3 = lwc1(sp + 64);
                                                                            f2_2 = (f4_1 * f3_3 + f5_1 * lwc1(sp + 68) + f6_1 * lwc1(sp + 72)) * -2.0_f32;
                                                                            f4_2 = (f4_1 + f3_3 * f2_2) * f8_1;
                                                                            swc1(s2_1 + 48, f4_2);
                                                                            f5_2 = (f5_1 + lwc1(sp + 68) * f2_2) * 0.5_f32;
                                                                            swc1(s2_1 + 52, f5_2);
                                                                            f6_2 = (f6_1 + lwc1(sp + 72) * f2_2) * f8_1;
                                                                            swc1(s2_1 + 56, f6_2);
                                                                            if (f4_2 * f4_2 + f5_2 * f5_2 + f6_2 * f6_2) < f10_1 {
                                                                                f1_2 = lwc1(sp + 68);
                                                                                if lw(v0_2 + 8) != 8 {
                                                                                    f2_3 = lwc1(g::updateframesf);
                                                                                    if !(lwc1(0x3ad25c) < f1_2) {
                                                                                        break 'b109;
                                                                                    }
                                                                                    s1_2 = ld(0x3a8ad8);
                                                                                    f12_2 = lwc1(sp + 72);
                                                                                    v0_3 = fptodp(f12_2);
                                                                                    s3_1 = ld(0x3a8ae0);
                                                                                    f12_3 = lwc1(sp + 68);
                                                                                    v0_4 = fptodp(f12_3);
                                                                                    a0_6 = v0_3;
                                                                                    a1_6 = v0_4;
                                                                                    v0_5 = atan2(a0_6, a1_6);
                                                                                    a0_7 = v0_5;
                                                                                    a1_7 = s1_2;
                                                                                    v0_6 = dpmul(a0_7, a1_7);
                                                                                    a0_8 = v0_6;
                                                                                    a1_8 = s3_1;
                                                                                    v0_7 = dpdiv(a0_8, a1_8);
                                                                                    s0_1 = lw(s2_1 + 96);
                                                                                    a1_9 = ld(0x3a8ae8);
                                                                                    a0_9 = v0_7;
                                                                                    v0_8 = dpadd(a0_9, a1_9);
                                                                                    a0_10 = v0_8;
                                                                                    f0_6 = dptofp(a0_10);
                                                                                    swc1(s0_1 + 72, f0_6);
                                                                                    f12_4 = lwc1(sp + 64);
                                                                                    v0_9 = fptodp(f12_4);
                                                                                    f1_3 = lwc1(sp + 68);
                                                                                    f0_7 = lwc1(sp + 72);
                                                                                    f12_5 = (f1_3 * f1_3 + f0_7 * f0_7).abs().sqrt();
                                                                                    v0_10 = fptodp(f12_5);
                                                                                    a0_11 = v0_9;
                                                                                    a1_10 = v0_10;
                                                                                    v0_11 = atan2(a0_11, a1_10);
                                                                                    a0_12 = v0_11;
                                                                                    a1_11 = s1_2;
                                                                                    v0_12 = dpmul(a0_12, a1_11);
                                                                                    a0_13 = v0_12;
                                                                                    a1_12 = s3_1;
                                                                                    v0_13 = dpdiv(a0_13, a1_12);
                                                                                    s0_2 = lw(s2_1 + 96);
                                                                                    a1_13 = v0_13;
                                                                                    a0_14 = 0;
                                                                                    v0_14 = dpsub(a0_14 as i64, a1_13);
                                                                                    a0_15 = v0_14;
                                                                                    f0_8 = dptofp(a0_15);
                                                                                    swc1(s0_2 + 88, f0_8);
                                                                                    a1_14 = s0_2;
                                                                                    sw(a1_14 + 76, 0);
                                                                                    swc1(s2_1 + 264, 1.0_f32);
                                                                                    sw(s2_1 + 48, 0);
                                                                                    sw(s2_1 + 52, 0);
                                                                                    sw(s2_1 + 56, 0);
                                                                                    break 'b108;
                                                                                }
                                                                            }
                                                                            f2_3 = lwc1(g::updateframesf);
                                                                        }
                                                                        a0_16 = 136;
                                                                        a1_15 = s2_1 + 24;
                                                                        f5_3 = lwc1(s2_1 + 64);
                                                                        f6_3 = lwc1(0x3ad264);
                                                                        v0_15 = lw(s2_1 + 212);
                                                                        f1_4 = lwc1(s2_1 + 68) * f6_3;
                                                                        f4_3 = lwc1(s2_1 + 60) * lwc1(0x3ad268);
                                                                        swc1(s2_1 + 52, lwc1(s2_1 + 52) + f2_3 * lwc1(0x3ad260));
                                                                        swc1(s2_1 + 68, f1_4);
                                                                        swc1(s2_1 + 60, f4_3);
                                                                        swc1(s2_1 + 64, f5_3 * f6_3);
                                                                        sw(s2_1 + 212, v0_15 + 1);
                                                                        soundStart(a0_16, a1_15);
                                                                        a1_14 = lw(s2_1 + 96);
                                                                    }
                                                                    a3_1 = lw(0x3b0138);
                                                                    swc1(a1_14 + 48, lwc1(s2_1 + 24));
                                                                    swc1(a1_14 + 52, lwc1(s2_1 + 28));
                                                                    swc1(a1_14 + 56, lwc1(s2_1 + 32));
                                                                    break 'b98;
                                                                }
                                                                f0_9 = lwc1(g::updateframesf);
                                                                f7_1 = lwc1(s2_1 + 24) + lwc1(sp);
                                                                f5_4 = lwc1(s2_1 + 28);
                                                                f1_5 = lwc1(0x3ad26c);
                                                                f4_4 = lwc1(s2_1 + 52);
                                                                swc1(s2_1 + 24, f7_1);
                                                                f6_4 = lwc1(s2_1 + 32);
                                                                f3_4 = lwc1(s2_1 + 48);
                                                                f1_6 = lwc1(s2_1 + 56);
                                                                f0_10 = lwc1(0x3ad270);
                                                                a1_16 = lw(s2_1 + 96);
                                                                swc1(s2_1 + 28, f5_4 + lwc1(sp + 4));
                                                                f2_4 = lwc1(sp + 8);
                                                                swc1(s2_1 + 56, f1_6 * f0_10);
                                                                swc1(s2_1 + 48, f3_4 * f0_10);
                                                                swc1(s2_1 + 52, (f4_4 + f0_9 * f1_5) * f0_10);
                                                                swc1(s2_1 + 32, f6_4 + f2_4);
                                                                swc1(a1_16 + 48, f7_1);
                                                                f2_5 = lwc1(a1_16 + 72);
                                                                swc1(a1_16 + 52, lwc1(s2_1 + 28));
                                                                swc1(a1_16 + 56, lwc1(s2_1 + 32));
                                                                f2_6 = f2_5 + lwc1(s2_1 + 60);
                                                                swc1(a1_16 + 72, f2_6);
                                                                if f2_6 < 0.0_f32 {
                                                                    swc1(a1_16 + 72, f2_6 + 3.6e+02_f32);
                                                                }
                                                                f0_11 = lwc1(a1_16 + 72);
                                                                if !(3.6e+02_f32 <= f0_11) {
                                                                    f1_7 = lwc1(a1_16 + 76);
                                                                } else {
                                                                    swc1(a1_16 + 72, f0_11 - 3.6e+02_f32);
                                                                    f1_7 = lwc1(a1_16 + 76);
                                                                }
                                                                f0_12 = f1_7 + lwc1(s2_1 + 64);
                                                                swc1(a1_16 + 76, f0_12);
                                                                if f0_12 < 0.0_f32 {
                                                                    swc1(a1_16 + 76, f0_12 + 3.6e+02_f32);
                                                                }
                                                                f0_13 = lwc1(a1_16 + 76);
                                                                if !(3.6e+02_f32 <= f0_13) {
                                                                    f1_8 = lwc1(a1_16 + 88);
                                                                } else {
                                                                    swc1(a1_16 + 76, f0_13 - 3.6e+02_f32);
                                                                    f1_8 = lwc1(a1_16 + 88);
                                                                }
                                                                f0_14 = f1_8 + lwc1(s2_1 + 68);
                                                                swc1(a1_16 + 88, f0_14);
                                                                if f0_14 < 0.0_f32 {
                                                                    swc1(a1_16 + 88, f0_14 + 3.6e+02_f32);
                                                                }
                                                                f0_15 = lwc1(a1_16 + 88);
                                                                a3_1 = lw(0x3b0138);
                                                                if 3.6e+02_f32 <= f0_15 {
                                                                    swc1(a1_16 + 88, f0_15 - 3.6e+02_f32);
                                                                    break 'b98;
                                                                }
                                                            }
                                                            a3_1 = lw(0x3b0138);
                                                        }
                                                    }
                                                    if s2_1 == 0 {
                                                        break 'b9;
                                                    }
                                                    a0_17 = lw(g::chrinfo);
                                                    s1_1 = s7_1 + 1;
                                                    v0_16 = lw(s2_1 + 108) * 0x1210;
                                                    break 'b14;
                                                }
                                                _ => unreachable!(),
                                            }
                                            a3_1 = lw(0x3b0138);
                                            break 'b8;
                                        }
                                        f0_29 = lwc1(s2_1 + 24);
                                        v1_5 = lw(s2_1);
                                    }
                                    v0_70 = v0_16 + a0_17;
                                    swc1(v0_70 + 2932, f0_29);
                                    sw(v0_70 + 2944, v1_5);
                                    swc1(v0_70 + 2936, lwc1(s2_1 + 28));
                                    swc1(v0_70 + 2940, lwc1(s2_1 + 32));
                                    break 'b8;
                                }
                                sw(s2_1 + 72, 0);
                                a3_1 = lw(0x3b0138);
                                break 'b8;
                            }
                            s1_1 = s7_1 + 1;
                            f2_31 = lwc1(s2_1 + 28);
                            f3_22 = lwc1(s2_1 + 32);
                            a3_1 = lw(0x3b0138);
                            swc1(s2_1 + 24, lwc1(s2_1 + 24) + lwc1(sp));
                            swc1(s2_1 + 28, f2_31 + lwc1(sp + 4));
                            swc1(s2_1 + 32, f3_22 + lwc1(sp + 8));
                            break 'b8;
                        }
                        bulletFree(a0_109);
                        a3_1 = lw(0x3b0138);
                        break 'b8;
                    }
                }
                s1_1 = s7_1 + 1;
            }
            v0_86 = lw(sp + 224);
            v1_20 = lw(sp + 224);
            if v0_86 == 0 {
                f3_1 = lwc1(g::updateframesf);
            } else {
                f3_1 = lwc1(g::updateframesf);
                sw(v1_20 + 16, lw(v0_86 + 16) | 32);
            }
        }
        s7_1 = s1_1;
        if s7_1 >= 50 {
            break;
        }
    }
    return;
}
