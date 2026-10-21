//! Functions: damaging a prop or a character, and creating a special effect group.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    random_as_float() * 2.3283064e-10_f32
}

/// An unsigned 32 bit number as a float.
fn uint_to_float(value: i32) -> f32 {
    if value < 0 {
        let half = (value & 1 | ((value as u32) >> 1) as i32) as f32;
        half + half
    } else {
        value as f32
    }
}

/// Damage the prop `a0` by `f12` (@ 0x271998), hit by `a1` at the point `a2` with the bullet `a3` (from the gun `a6`, `a4`, `a5` and `a7` being the details the hit kept in
/// its own frame): a prop that is not a character only takes the damage and calls its own handler, and a character loses health after the scaling of the game mode,
/// the handicap, the armour and the part that was hit, reacts, plays a sound, scores for the attacker, and dies when it has none left.
pub fn propDamage(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, f12: f32) {
    let frame = Frame::new(256);
    let sp = frame.sp();
    let mut f21_1 = f12;
    let f24_1 = f21_1;
    sw(sp + 16, a2);
    sw(sp + 20, a4);
    sw(sp + 24, a5);
    sw(sp + 28, a7);
    if lw(g::updateframesi) == 0 {
        return;
    }
    if a0 != 0 {
        if lw(a0 + 8) == 8 {
            let a0_1 = lw(a0 + 352);
            if lh(a0_1 + 4) == 31 {
                if lw(a0_1 + 2716) == 0x2000000 {
                    if lw(a0_1 + 2728) == 0 {
                        return;
                    }
                }
            }
        }
    }
    let v0_3;
    'b3: {
        if a1 != 0 {
            if lw(a1 + 8) != 8 {
                v0_3 = lw(a0 + 16);
                break 'b3;
            }
            let v1_1 = lw(sp + 16);
            let v0_2 = lw(a1 + 352);
            swc1(v0_2 + 2932, lwc1(v1_1));
            swc1(v0_2 + 2936, lwc1(v1_1 + 4));
            swc1(v0_2 + 2940, lwc1(v1_1 + 8));
            if a3 >= 0 {
                sw(v0_2 + 2944, a3);
            }
        }
        v0_3 = lw(a0 + 16);
    }
    let v1_2;
    if (v0_3 & 16) != 0 {
        v1_2 = lw(a0 + 8);
    } else {
        if (lw(sp + 28) & 2) == 0 {
            return;
        }
        v1_2 = lw(a0 + 8);
    }
    if v1_2 != 8 {
        let v0_27 = lw(a0 + 4);
        if ((v0_27 - 201) as u32) >= (2 as u32) {
            swc1(a0 + 532, lwc1(a0 + 532) + f21_1);
        } else {
            swc1(a0 + 532, lwc1(a0 + 532) + f21_1 * lwc1(0x3acd64));
        }
        let v0_28 = lw(a0 + 400);
        let a2_11 = lw(sp + 16);
        if v0_28 != 0 {
            let a4_3 = lw(sp + 20);
            let a5_3 = lw(sp + 24);
            call_indirect(v0_28, [a0, a1, a2_11, a3, a4_3, a5_3, a6, a7], [f21_1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        }
        return;
    }
    let mut f23_1 = 1.0_f32;
    let s4_1 = lw(a0 + 356);
    let s5_1 = lw(a0 + 352);
    let mut s1_1 = 0;
    if a1 != 0 {
        if lw(a1 + 8) == v1_2 {
            s1_1 = lw(a1 + 352);
        }
    }
    let a1_1;
    'b8: {
        'b9: {
            let v1_3 = g::curgameparms;
            if (lw(v1_3 + 80) & 32) != 0 {
                if lw(v1_3 + 72) != 6 {
                    let v0_4 = challengeGetSetup();
                    f21_1 = f21_1 * lwc1((lw(s5_1) << 4) + lw(v0_4 + 48) + 4);
                    if s1_1 == 0 {
                        break 'b9;
                    }
                    let v0_5 = challengeGetSetup();
                    f21_1 = f21_1 * lwc1((lw(s1_1) << 4) + lw(v0_5 + 48) + 8);
                }
            }
            if s1_1 == 0 {
                a1_1 = lw(s5_1);
                break 'b8;
            }
            if a6 == 0 {
                if a3 != 1 {
                    a1_1 = lw(s5_1);
                    break 'b8;
                }
            }
            if a3 != 1 {
                let f22_1 = 1.0_f32;
                StatsAddGun(2, lw(s1_1), lw(s1_1 + 260), f22_1);
                StatsAddChar(3, lw(s5_1), lw(s1_1), f22_1);
                let f0_1 = StatsGet(9, lw(s1_1));
                let f0_2 = StatsGet(8, lw(s1_1));
                if f0_1 < f0_2 {
                    StatsAdd(9, lw(s1_1), f22_1);
                }
                StatsAddChar(2, lw(s1_1), lw(s5_1), f22_1);
            } else {
                let f20_1 = 1.0_f32;
                StatsAdd(14, lw(s1_1), f20_1);
                StatsAdd(28, lw(s5_1), f20_1);
            }
            if a6 == 0 {
                a1_1 = lw(s5_1);
                break 'b8;
            }
            if lw(a6 + 156) == 0 {
                a1_1 = lw(s5_1);
                break 'b8;
            }
            StatsAdd(61, lw(s1_1), 1.0_f32);
        }
        a1_1 = lw(s5_1);
    }
    StatsAdd(26, a1_1, 1.0_f32);
    if 0.0_f32 < lwc1(s5_1 + 2900) {
        let v1_4 = lw(sp + 28);
        if ((lw(g::curgameparms + 72) - 6) as u32) >= (2 as u32) {
            if (v1_4 & 2) == 0 {
                soundStart(217, a0 + 48);
                return;
            }
        }
    }
    if s4_1 != 0 {
        if lw(s4_1 + 296) != 4 {
            sw(s5_1 + 256, a1);
        }
    }
    let mut v0_6 = a3 - 1;
    if a3 < 8 {
        animHitReact(a0);
        v0_6 = a3 - 1;
    }
    let f1_1;
    'b17: {
        sw(sp + 32, 0);
        if (v0_6 as u32) < (7 as u32) {
            let mut s0_1;
            let mut v0_8 = 0;
            let mut v0_9 = 0;
            'b19: {
                'b20: {
                    let v0_7 = chrGetLastHitPart();
                    let a2_4 = g::curgameparms;
                    s0_1 = lw((v0_7 << 2) + lw(lw(lw(a0 + 32) + 92) + 32));
                    if (lw(a2_4 + 80) & 32) != 0 {
                        let a0_14 = lw(a2_4 + 68);
                        v0_8 = s0_1 - 1;
                        if a0_14 != 1 {
                            break 'b20;
                        }
                        v0_9 = ((v0_8 as u32) < (17 as u32)) as i32;
                        if a1 == 0 {
                            break 'b19;
                        }
                        v0_8 = s0_1 - 1;
                        if lw(a1 + 356) == 0 {
                            break 'b20;
                        }
                        v0_9 = ((v0_8 as u32) < (17 as u32)) as i32;
                        if s4_1 != 0 {
                            break 'b19;
                        }
                        v0_8 = s0_1 - 1;
                        if lw(lw(a0 + 352) + 8) != a0_14 {
                            break 'b20;
                        }
                        v0_9 = ((v0_8 as u32) < (17 as u32)) as i32;
                        if s1_1 == 0 {
                            break 'b19;
                        }
                        v0_8 = s0_1 - 1;
                        if lw(s1_1 + 260) != 0 {
                            break 'b20;
                        }
                        if ((s0_1 - 2) as u32) < (2 as u32) {
                            s0_1 = 11;
                        } else if s0_1 == 7 {
                            s0_1 = 11;
                        } else {
                            v0_8 = s0_1 - 1;
                            if s0_1 != 0 {
                                break 'b20;
                            }
                            s0_1 = 11;
                        }
                    }
                    v0_8 = s0_1 - 1;
                }
                v0_9 = ((v0_8 as u32) < (17 as u32)) as i32;
            }
            let bc_1 = v0_9 == 0;
            let v0_10 = ((s0_1 as u32) < (18 as u32)) as i32;
            if !bc_1 {
                'b22: {
                    if v0_10 != 0 {
                        let mut v1_5 = 0;
                        let mut v0_11 = 0;
                        'b23: {
                            'b24: {
                                match (s0_1 << 2) >> 2 {
                                    0 | 1 | 2 => {
                                        v0_11 = 1;
                                        break 'b23;
                                    }
                                    3 | 4 | 5 | 6 => {
                                        v1_5 = 3;
                                        break 'b24;
                                    }
                                    7 | 8 | 9 | 10 => {
                                        v0_11 = 4;
                                    }
                                    11 => {
                                        sw(sp + 32, 2);
                                        let mut v1_6 = 0;
                                        if lw(sp + 24) == 0 {
                                            sw(sp + 24, g::bd_363);
                                        }
                                        f23_1 = 5.0_f32;
                                        if a1 != 0 {
                                            if lw(a1 + 356) != 0 {
                                                let a1_12 = lw(sp + 24);
                                                if s4_1 == 0 {
                                                    v1_6 = headPropNew(a0, a1_12);
                                                }
                                            }
                                        }
                                        if v1_6 == 0 {
                                            break 'b22;
                                        }
                                        'b28: {
                                            let a1_13 = lw(a0 + 352);
                                            if lw(a1_13 + 8) == 1 {
                                                sw(a0 + 520, 0);
                                                if s1_1 == 0 {
                                                    break 'b28;
                                                }
                                                StatsKillMade(s1_1, a1_13);
                                                if lw(g::curgameparms + 72) == 6 {
                                                    if a1 != 0 {
                                                        if lw(a1 + 8) == 8 {
                                                            let v1_7 = lw(a1 + 352);
                                                            sw(v1_7 + 248, lw(v1_7 + 248) + 10);
                                                        }
                                                    }
                                                }
                                            }
                                            if s1_1 != 0 {
                                                let f20_2 = 1.0_f32;
                                                StatsAdd(23, lw(s1_1), f20_2);
                                                if lw(s1_1 + 260) == 0 {
                                                    StatsAdd(24, lw(s1_1), f20_2);
                                                }
                                            }
                                        }
                                        let f20_3 = 1.0_f32;
                                        StatsAdd(29, lw(s5_1), f20_3);
                                        if s1_1 == 0 {
                                            break 'b22;
                                        }
                                        if lw(s1_1 + 260) != 0 {
                                            break 'b22;
                                        }
                                        StatsAdd(30, lw(s5_1), f20_3);
                                        break 'b22;
                                    }
                                    12 | 13 | 14 => {
                                        v1_5 = 5;
                                        break 'b24;
                                    }
                                    _ => {
                                        v0_11 = 6;
                                    }
                                }
                                f23_1 = lwc1(0x3acd2c);
                                break 'b23;
                            }
                            f23_1 = lwc1(0x3acd28);
                            sw(sp + 32, v1_5);
                            break 'b22;
                        }
                        sw(sp + 32, v0_11);
                    }
                }
                if a6 == 0 {
                    f1_1 = lwc1(a0 + 520);
                    break 'b17;
                }
                if s1_1 != 0 {
                    if a3 != 1 {
                        StatsBodyPartHit(s1_1, s5_1, s0_1);
                    }
                }
            }
        }
        f1_1 = lwc1(a0 + 520);
    }
    if f1_1 <= 0.0_f32 {
        if lw(lw(a0 + 352) + 8) != 1 {
            if a6 == 0 {
                return;
            }
            if s1_1 == 0 {
                return;
            }
            if lw(s1_1 + 260) == 0 {
                return;
            }
            StatsAdd(12, lw(s1_1), 1.0_f32);
            return;
        }
    }
    let mut v0_13;
    'b31: {
        if s4_1 != 0 {
            let v1_8 = lw(sp + 32);
            if (lw(g::curgameparms + 80) & 0x10000) != 0 {
                v0_13 = lw(g::invincible);
                if v1_8 == 2 {
                    break 'b31;
                }
                let v0_14 = bulletTypeExplosive(a3);
                v0_13 = lw(g::invincible);
                if v0_14 != 0 {
                    break 'b31;
                }
                f21_1 = f21_1 * lwc1((lw(lw(s4_1 + 24) + 92) << 2) + g::handicapmods_364);
            }
        }
        v0_13 = lw(g::invincible);
    }
    let a0_24 = g::curgameparms;
    if v0_13 != 0 {
        if s4_1 != 0 {
            f21_1 = 0.0_f32;
        }
    }
    if lw(a0_24 + 72) == 7 {
        let mut f9_1 = 1.0_f32;
        let f0_9;
        'b35: {
            let mut f8_1 = f9_1;
            let mut f7_1 = f9_1;
            if s4_1 != 0 {
                let v1_9 = lw(a0_24 + 76);
                if v1_9 == 0 {
                    f8_1 = lwc1(0x3acd30);
                }
                if v1_9 == 1 {
                    f8_1 = lwc1(0x3acd34);
                }
                let f1_2 = lwc1(a0 + 520);
                if v1_9 == 2 {
                    f8_1 = lwc1(0x3acd38);
                }
                f7_1 = f1_2 / 5.0_f32;
                let v0_15 = g::curgameparms;
                if v1_9 == 0 {
                    let f0_3 = lwc1(0x3acd3c);
                    if f7_1 < f0_3 {
                        f7_1 = f0_3;
                    }
                }
                let v1_10 = lw(v0_15 + 76);
                let v0_16 = g::curgameparms;
                if v1_10 == 1 {
                    let f0_4 = lwc1(0x3acd40);
                    if f7_1 < f0_4 {
                        f7_1 = f0_4;
                    }
                }
                if lw(v0_16 + 76) == 2 {
                    let f0_5 = lwc1(0x3acd44);
                    if f7_1 < f0_5 {
                        f7_1 = f0_5;
                    }
                }
                if a1 != 0 {
                    if lw(a1 + 356) == 0 {
                        let f1_3 = lwc1(a1 + 48) - lwc1(a0 + 48);
                        let f0_6 = lwc1(a1 + 56);
                        let f2_1 = lwc1(a1 + 52) - lwc1(a0 + 52);
                        let f5_1 = lwc1(a0 + 56);
                        let f6_1 = 1e+02_f32;
                        swc1(sp, f1_3);
                        let f0_7 = f0_6 - f5_1;
                        swc1(sp + 4, f2_1);
                        let f3_1 = f1_3 * f1_3 + f2_1 * f2_1 + f0_7 * f0_7;
                        swc1(sp + 8, f0_7);
                        if f3_1 < f6_1 {
                            let f0_8 = f6_1 - f3_1;
                            f9_1 = f9_1 + (f0_8 + f0_8) / (f8_1 * f6_1);
                        }
                    }
                }
                if s4_1 != 0 {
                    f0_9 = f8_1 * f7_1;
                    break 'b35;
                }
            }
            if a1 == 0 {
                f0_9 = f8_1 * f7_1;
            } else {
                if lw(a1 + 356) != 0 {
                    let v1_11 = lw(g::curgameparms + 76);
                    if v1_11 == 0 {
                        f8_1 = 5.0_f32;
                    }
                    if v1_11 == 1 {
                        f8_1 = 3.0_f32;
                    }
                    if v1_11 != 2 {
                        f0_9 = f8_1 * f7_1;
                        break 'b35;
                    }
                    f8_1 = 1.75_f32;
                }
                f0_9 = f8_1 * f7_1;
            }
        }
        f21_1 = f21_1 * (f0_9 * f9_1);
    }
    let mut f21_2 = f21_1 * f23_1;
    if s4_1 != 0 {
        let a0_25 = g::curgameparms;
        if lw(a0_25 + 72) == 7 {
            let v1_12 = lw(a0_25 + 76);
            let mut f0_10 = 1.0_f32;
            if v1_12 == 0 {
                f0_10 = lwc1(0x3acd48);
            }
            if v1_12 == 1 {
                f0_10 = lwc1(0x3acd4c);
            }
            if v1_12 == 2 {
                f0_10 = lwc1(0x3acd50);
            }
            if f0_10 < f21_2 {
                f21_2 = f0_10;
            }
        }
    }
    let f2_2;
    'b51: {
        if s1_1 == 0 {
            f2_2 = lwc1(s5_1 + 12);
        } else {
            if s1_1 != s5_1 {
                let a0_26 = lw(g::curgameparms + 80);
                if (a0_26 & 1) == 0 {
                    f2_2 = lwc1(s5_1 + 12);
                    break 'b51;
                }
                if lw(s1_1 + 24) != lw(s5_1 + 24) {
                    f2_2 = lwc1(s5_1 + 12);
                    break 'b51;
                }
                if (a0_26 & 8) == 0 {
                    f21_2 = 0.0_f32;
                }
                let f20_4 = 1.0_f32;
                StatsAdd(25, lw(s1_1), f20_4);
                StatsAdd(31, lw(s5_1), f20_4);
            }
            f2_2 = lwc1(s5_1 + 12);
        }
    }
    if 0.0_f32 < f2_2 {
        let mut f1_4 = f21_2;
        if f2_2 < f21_2 {
            f1_4 = f2_2;
        }
        f21_2 = f21_2 - f1_4;
        swc1(s5_1 + 12, f2_2 - f1_4);
    }
    let f1_5;
    if !(0.0_f32 < f21_2) {
        f1_5 = lwc1(a0 + 520);
    } else if a1 == 0 {
        f1_5 = lwc1(a0 + 520);
    } else if lw(a1 + 8) != 8 {
        f1_5 = lwc1(a0 + 520);
    } else if (lw(g::curgameparms + 80) & 4) == 0 {
        f1_5 = lwc1(a0 + 520);
    } else {
        f21_2 = lwc1(a0 + 520);
        f1_5 = f21_2;
    }
    let f0_11;
    if lw(g::curgameparms + 72) != 4 {
        f0_11 = f1_5 - f21_2;
    } else if a0 != lw(g::escort) {
        f0_11 = f1_5 - f21_2;
    } else {
        f21_2 = f21_2 * lwc1(0x3acd54);
        f0_11 = f1_5 - f21_2;
    }
    let f0_12;
    'b58: {
        let v1_13 = lw(lw(a0 + 352) + 8);
        swc1(a0 + 520, f0_11);
        if v1_13 == 1 {
            let a0_29 = g::curgameparms;
            if lw(a0_29 + 72) != 7 {
                f0_12 = lwc1(a0 + 520);
                break 'b58;
            }
            if lw(a0_29 + 76) == 0 {
                sw(a0 + 520, 0);
            }
        }
        f0_12 = lwc1(a0 + 520);
    }
    if f0_12 < 0.0_f32 {
        swc1(a0 + 520, 0.0_f32);
    }
    if s4_1 != 0 {
        let a1_22 = g::hudparms;
        let a0_30 = lw(s4_1) * 352 + a1_22;
        sw(a0_30 + 268, lw(a0_30 + 268) + (f21_2 * 6e+02_f32) as i32);
        let v1_14 = lw(s4_1) * 352 + a1_22;
        if lw(v1_14 + 268) >= 601 {
            sw(v1_14 + 268, 600);
        }
        if a3 != -1 {
            let v0_17 = bulletTypeExplosive(a3);
            if v0_17 == 0 {
                let v1_15 = a3 << 3;
                let v0_18 = g::dualshockbulletdamage;
                swc1(s4_1 + 1776, lwc1(v1_15 + v0_18));
                swc1(s4_1 + 1780, lwc1(v0_18 + v1_15 + 4));
            } else {
                swc1(s4_1 + 1780, lwc1(0x3acd5c));
                swc1(s4_1 + 1776, f24_1 * 5.0_f32);
            }
        } else {
            let f0_13 = if !(f24_1 < 0.25_f32) { 1.0_f32 } else { f24_1 * 4.0_f32 };
            swc1(s4_1 + 1776, f0_13);
            swc1(s4_1 + 1780, lwc1(0x3acd58));
        }
    }
    let mut f1_6;
    if a3 != 1 {
        f1_6 = lwc1(a0 + 520);
    } else {
        let v0_19 = newrnd();
        let a1_23 = lw(sp + 16);
        soundStart((v0_19 & 3_i64) as i32 + 207, a1_23);
        f1_6 = lwc1(a0 + 520);
    }
    'b65: {
        let a1_24 = lw(g::gameframe);
        let a0_33;
        let a1_25;
        if !(0.0_f32 < f1_6) {
            let lo_1 = lh(s5_1 + 4) * 44;
            sw(g::lastreactionframenum, a1_24);
            let s1_3 = lw(lo_1 + (g::chrs + 4));
            let mut s0_3 = 0;
            'b67: {
                if lh(s1_3 + 36) != 0 {
                    s0_3 = s0_3 + 1;
                    loop {
                        let v0_21 = (s0_3 < 6) as i32;
                        if lh(s1_3 + 36 + (s0_3 << 1)) == 0 {
                            break 'b67;
                        }
                        if v0_21 == 0 {
                            break 'b67;
                        }
                        s0_3 = s0_3 + 1;
                    }
                }
            }
            if s0_3 <= 0 {
                break 'b65;
            }
            let v0_22 = newrnd();
            let hi_2 = ((v0_22 as i32 as u32) % (s0_3 as u32)) as i32;
            if s0_3 == 0 {
                trap("break");
            }
            a1_25 = lw(sp + 16);
            a0_33 = lh(s1_3 + (hi_2 << 1) + 36);
        } else {
            if (a1_24 - lw(g::lastreactionframenum)) < 61 {
                break 'b65;
            }
            let v1_16 = lh(s5_1 + 4);
            sw(g::lastreactionframenum, a1_24);
            let s1_2 = lw(v1_16 * 44 + (g::chrs + 4));
            let mut s0_2 = 0;
            'b70: {
                if lh(s1_2) != 0 {
                    let mut v1_17 = s1_2 + 2;
                    loop {
                        s0_2 = s0_2 + 1;
                        if lh(v1_17) == 0 {
                            break 'b70;
                        }
                        if s0_2 >= 18 {
                            break 'b70;
                        }
                        v1_17 = v1_17 + 2;
                    }
                }
            }
            if s0_2 <= 0 {
                break 'b65;
            }
            let v0_20 = newrnd();
            let hi_1 = ((v0_20 as i32 as u32) % (s0_2 as u32)) as i32;
            if s0_2 == 0 {
                trap("break");
            }
            a1_25 = lw(sp + 16);
            a0_33 = lh(s1_2 + (hi_1 << 1));
        }
        soundDelayStartEx(a0_33, a1_25, 20, 0);
        f1_6 = lwc1(a0 + 520);
    }
    'b73: {
        if !(f1_6 == 0.0_f32) {
            let v0_26 = lw(s5_1);
            if v0_26 < lw(g::lvnumplayers) {
                return;
            }
            if a1 == 0 {
                return;
            }
            if lw(lw(a0 + 352) + 8) == 1 {
                if f1_6 < lwc1(0x3acd60) {
                    break 'b73;
                }
            }
            enemyInformHit(a0, a1);
            return;
        }
        'b75: {
            if a1 != 0 {
                if lw(a1 + 8) == 8 {
                    let s1_4 = lw(a1 + 352);
                    let mut a1_26 = lw(a1 + 356);
                    let a0_34 = s1_4 + 32 + (lw(s5_1) << 1);
                    sh(a0_34, lhu(a0_34) + 1);
                    let s0_4 = g::curgameparms;
                    'b78: {
                        if a1_26 != 0 {
                            if lw(g::curgameparms + 72) == 3 {
                                hudShowScore(lw(a1_26), 184);
                                a1_26 = lw(a1 + 356);
                            }
                            if a1_26 == 0 {
                                break 'b78;
                            }
                            if (lw(a1_26 + 20) & 2) == 0 {
                                break 'b78;
                            }
                            StatsAdd(68, lw(a1 + 4), 1.0_f32);
                        }
                    }
                    if lw(s0_4 + 72) == 3 {
                        if (lw(s0_4 + 80) & 0x8000) != 0 {
                            let a0_37 = lw(s1_4 + 24);
                            let v0_23 = teamGetScore(a0_37);
                            if v0_23 == lw(s0_4 + 92) - 1 {
                                let s1_5;
                                if (lw(s0_4 + 80) & 1) == 0 {
                                    let a1_30 = lw(s1_4 + 16);
                                    let a0_39 = 0x3af940;
                                    let a2_8 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x120c);
                                    s1_5 = vafmt(&[a0_39 as i64, a1_30 as i64, a2_8 as i64]);
                                } else {
                                    let v1_18 = lw((lw(g::cur_language) << 2) + g::languages);
                                    let a0_38 = 0x3a7fc8;
                                    let a3_2 = lw(v1_18 + 0x1208);
                                    let a1_29 = lw((lh((lw(s1_4 + 24) << 3) + (g::storydata + 64)) << 2) + v1_18);
                                    let a2_7 = lw(v1_18 + 0x1204);
                                    s1_5 = vafmt(&[a0_38 as i64, a1_29 as i64, a2_7 as i64, a3_2 as i64]);
                                }
                                'b82: {
                                    let mut s0_5 = 0;
                                    if lw(g::lvnumplayers) > 0 {
                                        let mut a0_40 = s0_5;
                                        loop {
                                            hudMessage(a0_40, s1_5);
                                            s0_5 = s0_5 + 1;
                                            a0_40 = s0_5;
                                            if s0_5 >= lw(g::lvnumplayers) {
                                                break 'b82;
                                            }
                                        }
                                    }
                                }
                                soundStart(200, 0);
                            }
                        }
                    }
                }
                if a1 != 0 {
                    if lw(a1 + 8) == 8 {
                        break 'b75;
                    }
                }
            }
            let a0_42 = s5_1 + 32 + (lw(s5_1) << 1);
            sh(a0_42, lhu(a0_42) + 1);
        }
        let a3_3 = lw(sp + 20);
        if s4_1 != 0 {
            let a4_1 = lw(sp + 24);
            let a5_1 = lw(sp + 32);
            let a6_1 = lw(sp + 28);
            playerDie(s4_1, a1, a3, a3_3, a4_1, a5_1, a6_1);
            return;
        }
    }
    let a3_4 = lw(sp + 20);
    let a4_2 = lw(sp + 24);
    let a5_2 = lw(sp + 32);
    let a6_2 = lw(sp + 28);
    enemyDie(a0, a1, a3, a3_4, a4_2, a5_2, a6_2);
}

/// Start the special effect group `a7` of the kind `a0` (@ 0x2a9888): `a1` is where it is, `a2` and `a3` the two other points it may keep (the first point when they are
/// not given), `a4` and `a5` its details; depending on the kind it makes the prop (a spark, a flash, the fire, a smoke trail) and the particles that go with it. Gives the
/// group, or 0 when the prop could not be made.
pub fn specialfxCreate(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32) -> i32 {
    let frame = Frame::new(256);
    let sp = frame.sp();
    let scale = 2.3283064e-10_f32;
    let t4_1 = g::fxgroups;
    let a1_1 = a7 * 92 + (t4_1 + 8);
    sw(a7 * 92 + t4_1, a0);
    sw(t4_1 + a7 * 92 + 4, a4);
    sw(a1_1, a5);
    if a5 < 0 {
        sw(a1_1, 0);
    }
    let v0_1 = t4_1 + a7 * 92;
    swc1(v0_1 + 12, lwc1(a1));
    swc1(v0_1 + 16, lwc1(a1 + 4));
    swc1(v0_1 + 20, lwc1(a1 + 8));
    let f1_1;
    if a2 == 0 {
        swc1(v0_1 + 24, lwc1(a1));
        swc1(v0_1 + 28, lwc1(a1 + 4));
        f1_1 = lwc1(a1 + 8);
    } else {
        swc1(v0_1 + 24, lwc1(a2));
        swc1(v0_1 + 28, lwc1(a2 + 4));
        f1_1 = lwc1(a2 + 8);
    }
    swc1(v0_1 + 32, f1_1);
    let group_at = a7 * 92 + g::fxgroups;
    let f1_2;
    if a3 == 0 {
        swc1(group_at + 36, lwc1(a1));
        swc1(group_at + 40, lwc1(a1 + 4));
        f1_2 = lwc1(a1 + 8);
    } else {
        swc1(group_at + 36, lwc1(a3));
        swc1(group_at + 40, lwc1(a3 + 4));
        f1_2 = lwc1(a3 + 8);
    }
    let result;
    'b4: {
        'b5: {
            swc1(group_at + 44, f1_2);
            let v1_2 = g::fxgroups;
            let a0_4 = lw(a7 * 92 + v1_2);
            sw(v1_2 + a7 * 92 + 48, 0);
            if (a0_4 as u32) < (12 as u32) {
                let mut a1_7 = 0;
                let mut f1_8 = 0.0_f32;
                let mut f0_11 = 0.0_f32;
                'b6: {
                    'b7: {
                        let mut s2_4 = 0;
                        let mut s3_3 = 0;
                        let mut f0_19 = 0.0_f32;
                        let mut f2_4 = 0.0_f32;
                        let mut s1_4 = 0;
                        let mut a0_12 = 0;
                        let mut a1_8 = 0;
                        let mut f12_4 = 0.0_f32;
                        let mut f13_1 = 0.0_f32;
                        let mut f14_1 = 0.0_f32;
                        'b8: {
                            match (a0_4 << 2) >> 2 {
                                0 => {
                                    f1_8 = random_as_float();
                                    a1_7 = a7 * 92 + (g::fxgroups + 52);
                                    f0_11 = scale;
                                    break 'b6;
                                }
                                1 => {
                                    memset(sp + 16, 0, 12);
                                    let a0_29 = a5 + 304;
                                    let f14_7 = lwc1(a1 + 8);
                                    let f12_10 = lwc1(a1);
                                    let f15_7 = 0.0_f32;
                                    let f13_7 = lwc1(a1 + 4) + 1.0_f32 + lwc1(0x3ad660);
                                    let v0_34 = specialPropNew(a0_29, -1, f12_10, f13_7, f14_7, f15_7);
                                    let s0_7 = g::fxgroups;
                                    sw(s0_7 + a7 * 92 + 56, v0_34);
                                    if v0_34 == 0 {
                                        break 'b7;
                                    }
                                    let s0_8 = a7 * 92 + (s0_7 + 72);
                                    let v0_35 = particleNewManual(s0_8, 11, sp + 16);
                                    sw(s0_8, v0_35);
                                    let mut s4_1 = 0;
                                    if v0_35 != 0 {
                                        let mut f1_10;
                                        'b12: {
                                            'b13: {
                                                if a5 == 1 {
                                                    f1_10 = 1e+02_f32;
                                                    s4_1 = g::scratchbuffer + 13023;
                                                    swc1(sp + 36, f1_10);
                                                    swc1(sp + 40, 3e+01_f32);
                                                } else {
                                                    if a5 < 2 {
                                                        if a5 == 0 {
                                                            s4_1 = g::scratchbuffer + 13055;
                                                            swc1(sp + 32, 127.0_f32);
                                                            swc1(sp + 36, 6e+01_f32);
                                                            swc1(sp + 40, 3e+01_f32);
                                                            break 'b13;
                                                        }
                                                        break 'b12;
                                                    }
                                                    if a5 == 2 {
                                                        f1_10 = 3e+01_f32;
                                                        s4_1 = 0x7f3f2f;
                                                        swc1(sp + 36, f1_10);
                                                        swc1(sp + 40, 127.0_f32);
                                                    } else {
                                                        if a5 != 3 {
                                                            break 'b12;
                                                        }
                                                        f1_10 = 3e+01_f32;
                                                        s4_1 = 0x3f7f3f;
                                                        swc1(sp + 40, f1_10);
                                                        swc1(sp + 36, 127.0_f32);
                                                    }
                                                }
                                                swc1(sp + 32, f1_10);
                                            }
                                        }
                                        let mut s1_7 = 0;
                                        if lw(lw(g::fxgroups + a7 * 92 + 72) + 24) > 0 {
                                            loop {
                                                let f12_11 = lwc1(sp + 32) - random_as_float() * scale * 1e+01_f32;
                                                let v0_37 = fptoui(f12_11);
                                                let f12_12 = lwc1(sp + 36) - random_as_float() * scale * 1e+01_f32;
                                                let v0_39 = fptoui(f12_12);
                                                let f12_13 = lwc1(sp + 40) - random_as_float() * scale * 1e+01_f32;
                                                let v0_41 = fptoui(f12_13);
                                                let a0_31 = lw(a7 * 92 + (g::fxgroups + 72)) + 992 + s1_7 * 12;
                                                let f1_11 = random_as_float();
                                                swc1(a0_31, (f1_11 * scale - 0.5_f32) * lwc1(0x3ad664));
                                                let a0_32 = lw(a7 * 92 + (g::fxgroups + 72)) + 996 + s1_7 * 12;
                                                let f1_13 = random_as_float();
                                                swc1(a0_32, (f1_13 * scale - 0.5_f32) * lwc1(0x3ad668));
                                                let a3_2 = lw(a7 * 92 + (g::fxgroups + 72)) + 1000 + s1_7 * 12;
                                                let f2_6 = random_as_float();
                                                let a1_28 = s1_7 << 2;
                                                let a0_33 = a7 * 92 + (g::fxgroups + 72);
                                                let a2_10 = v0_37 | v0_39 << 8 | v0_41 << 16;
                                                swc1(a3_2, (f2_6 * scale - 0.5_f32) * lwc1(0x3ad66c));
                                                sw(lw(a0_33) + a1_28 + 2592, 0);
                                                let a0_34 = lw(a0_33) + 2192 + a1_28;
                                                swc1(a0_34, uint_to_float(a2_10));
                                                s1_7 = s1_7 + 1;
                                                if s1_7 >= lw(lw(a7 * 92 + (g::fxgroups + 72)) + 24) {
                                                    break;
                                                }
                                            }
                                        }
                                        let s1_8 = g::fxgroups;
                                        let s3_5 = lw(s1_8 + a7 * 92 + 72);
                                        let s2_7 = s3_5 + 928;
                                        sceVu0UnitMatrix(s2_7);
                                        let v0_45 = s1_8 + a7 * 92;
                                        let f2_8 = 1.0_f32;
                                        let f1_15 = lwc1(0x3ad670);
                                        swc1(s2_7 + 48, lwc1(v0_45 + 12));
                                        swc1(s2_7 + 52, lwc1(v0_45 + 16) + f2_8 + f1_15);
                                        swc1(s2_7 + 56, lwc1(v0_45 + 20));
                                        swc1(s3_5 + 928, f2_8);
                                        swc1(s2_7 + 40, f2_8);
                                        swc1(s2_7 + 20, f2_8);
                                    }
                                    let s0_9 = a7 * 92 + (g::fxgroups + 76);
                                    let v0_46 = particleNewManual(s0_9, 12, sp + 16);
                                    sw(s0_9, v0_46);
                                    if v0_46 == 0 {
                                        break 'b5;
                                    }
                                    'b25: {
                                        let mut s1_9 = 0;
                                        if lw(v0_46 + 24) > 0 {
                                            let f21_4 = scale;
                                            let mut s2_8 = 0;
                                            let f20_7 = 0.5_f32;
                                            let f22_2 = lwc1(0x3ad674);
                                            loop {
                                                let a1_30 = lw(s0_9) + 992 + s2_8;
                                                swc1(a1_30, (random_as_float() * f21_4 - f20_7) * f22_2);
                                                let a1_31 = lw(s0_9) + 996 + s2_8;
                                                swc1(a1_31, (random_as_float() * f21_4 - f20_7) * lwc1(0x3ad678));
                                                let a1_32 = lw(s0_9) + 1000 + s2_8;
                                                let f0_40 = random_as_float() * f21_4;
                                                let a0_37 = s1_9 << 2;
                                                swc1(a1_32, (f0_40 - f20_7) * f22_2);
                                                sw(lw(s0_9) + a0_37 + 2592, 0);
                                                let a0_38 = lw(s0_9) + 2192 + a0_37;
                                                swc1(a0_38, uint_to_float(s4_1));
                                                s1_9 = s1_9 + 1;
                                                s2_8 = s2_8 + 12;
                                                if s1_9 >= lw(lw(s0_9) + 24) {
                                                    break 'b25;
                                                }
                                            }
                                        }
                                    }
                                    let s1_10 = g::fxgroups;
                                    s3_3 = lw(s1_10 + a7 * 92 + 76);
                                    s2_4 = s3_3 + 928;
                                    sceVu0UnitMatrix(s2_4);
                                    let v0_50 = s1_10 + a7 * 92;
                                    f2_4 = 1.0_f32;
                                    let f1_16 = lwc1(0x3ad67c);
                                    s1_4 = v0_50;
                                    swc1(s2_4 + 48, lwc1(v0_50 + 12));
                                    f0_19 = lwc1(v0_50 + 16) + f2_4 + f1_16;
                                    break 'b8;
                                }
                                2 | 3 => {
                                    memset(sp, 0, 12);
                                    let f14_6 = lwc1(a1 + 8);
                                    let f12_9 = lwc1(a1);
                                    let f15_6 = 0.0_f32;
                                    let f13_6 = lwc1(a1 + 4);
                                    let v0_26 = specialPropNew(294, -1, f12_9, f13_6, f14_6, f15_6);
                                    let a1_20 = g::fxgroups;
                                    sw(a1_20 + a7 * 92 + 56, v0_26);
                                    if v0_26 == 0 {
                                        result = 0;
                                        break 'b4;
                                    }
                                    let v0_28 = lw(v0_26 + 352);
                                    if a6 != 0 {
                                        swc1(v0_28 + 76, lwc1(a6));
                                        swc1(v0_28 + 80, lwc1(a6 + 4));
                                        swc1(v0_28 + 84, lwc1(a6 + 8));
                                        swc1(v0_28 + 88, lwc1(a6 + 12));
                                        swc1(v0_28 + 92, lwc1(a6 + 16));
                                        swc1(v0_28 + 96, lwc1(a6 + 20));
                                    }
                                    if lw(a7 * 92 + a1_20) != 2 {
                                        break 'b5;
                                    }
                                    let s0_6 = a7 * 92 + (a1_20 + 72);
                                    let v0_29 = particleNewManual(s0_6, 9, sp);
                                    sw(s0_6, v0_29);
                                    if v0_29 == 0 {
                                        break 'b5;
                                    }
                                    'b32: {
                                        let mut s1_5 = 0;
                                        if lw(v0_29 + 24) > 0 {
                                            let f20_6 = scale;
                                            let mut s2_5 = 0;
                                            let f21_3 = 0.5_f32;
                                            loop {
                                                let a1_22 = lw(s0_6) + 992 + s2_5;
                                                let f0_22 = random_as_float() * f20_6 - f21_3;
                                                swc1(a1_22, f0_22 + f0_22);
                                                let a1_23 = lw(s0_6) + 996 + s2_5;
                                                let f0_23 = random_as_float() * f20_6;
                                                swc1(a1_23, f0_23 + f0_23);
                                                let a1_24 = lw(s0_6) + 1000 + s2_5;
                                                let f0_25 = random_as_float() * f20_6;
                                                let a0_26 = s1_5 << 2;
                                                let f1_9 = lwc1(0x3ad650);
                                                s1_5 = s1_5 + 1;
                                                let f0_27 = f0_25 - f21_3;
                                                swc1(a1_24, f0_27 + f0_27);
                                                sw(lw(s0_6) + a0_26 + 2592, 0);
                                                swc1(lw(s0_6) + a0_26 + 2192, f1_9);
                                                s2_5 = s2_5 + 12;
                                                if s1_5 >= lw(lw(s0_6) + 24) {
                                                    break 'b32;
                                                }
                                            }
                                        }
                                    }
                                    let s1_6 = g::fxgroups;
                                    let s3_4 = lw(s1_6 + a7 * 92 + 72);
                                    let s2_6 = s3_4 + 928;
                                    sceVu0UnitMatrix(s2_6);
                                    let v0_33 = s1_6 + a7 * 92;
                                    let f2_5 = lwc1(0x3ad654);
                                    swc1(s2_6 + 48, lwc1(v0_33 + 12));
                                    let f3_1 = 13.0_f32;
                                    swc1(s2_6 + 52, lwc1(v0_33 + 16) + 1.0_f32 - f2_5);
                                    swc1(s2_6 + 56, lwc1(v0_33 + 20));
                                    swc1(s3_4 + 928, f3_1);
                                    swc1(s2_6 + 40, f3_1);
                                    swc1(s2_6 + 20, 7.0_f32);
                                    break 'b5;
                                }
                                4 => {
                                    a0_12 = 295;
                                    a1_8 = -1;
                                    f14_1 = lwc1(a1 + 8);
                                    f13_1 = lwc1(a1 + 4) + 1.0_f32;
                                    f12_4 = lwc1(a1);
                                }
                                5 => {
                                    a0_12 = a5 + 296;
                                    a1_8 = -1;
                                    f14_1 = lwc1(a1 + 8);
                                    f13_1 = lwc1(a1 + 4) + 1.0_f32;
                                    f12_4 = lwc1(a1);
                                }
                                6 => {
                                    memset(sp, 0, 12);
                                    let s0_4 = a5 + 296;
                                    let f21_1 = 1.0_f32;
                                    let f12_5 = lwc1(a1);
                                    let f13_2 = lwc1(a1 + 4) + f21_1;
                                    let f14_2 = lwc1(a1 + 8);
                                    let f15_1 = 0.0_f32;
                                    let s6_1 = g::fxgroups;
                                    let mut s3_2 = 0;
                                    let v0_15 = specialPropNew(s0_4, -1, f12_5, f13_2, f14_2, f15_1);
                                    sw(s6_1 + a7 * 92 + 56, v0_15);
                                    if v0_15 == 0 {
                                        break 'b7;
                                    }
                                    swc1(v0_15 + 72, 0.0_f32);
                                    let f20_2 = 9e+01_f32;
                                    let f12_6 = lwc1(a1);
                                    let f13_3 = lwc1(a1 + 4) + f21_1;
                                    let f14_3 = lwc1(a1 + 8);
                                    let v0_16 = specialPropNew(s0_4, -1, f12_6, f13_3, f14_3, f20_2);
                                    sw(s6_1 + a7 * 92 + 60, v0_16);
                                    if v0_16 == 0 {
                                        break 'b7;
                                    }
                                    swc1(v0_16 + 72, f20_2);
                                    let f20_3 = 1.8e+02_f32;
                                    let f12_7 = lwc1(a1);
                                    let f13_4 = lwc1(a1 + 4) + f21_1;
                                    let f14_4 = lwc1(a1 + 8);
                                    let v0_17 = specialPropNew(s0_4, -1, f12_7, f13_4, f14_4, f20_3);
                                    sw(s6_1 + a7 * 92 + 64, v0_17);
                                    if v0_17 == 0 {
                                        break 'b7;
                                    }
                                    swc1(v0_17 + 72, f20_3);
                                    let f20_4 = 2.7e+02_f32;
                                    let f14_5 = lwc1(a1 + 8);
                                    let f13_5 = lwc1(a1 + 4) + f21_1;
                                    let f12_8 = lwc1(a1);
                                    let v0_18 = specialPropNew(s0_4, -1, f12_8, f13_5, f14_5, f20_4);
                                    sw(s6_1 + a7 * 92 + 68, v0_18);
                                    if v0_18 == 0 {
                                        break 'b7;
                                    }
                                    swc1(v0_18 + 72, f20_4);
                                    let s0_5 = a7 * 92 + (s6_1 + 72);
                                    let v0_19 = particleNewManual(s0_5, 10, sp);
                                    sw(s0_5, v0_19);
                                    if v0_19 == 0 {
                                        break 'b5;
                                    }
                                    'b37: {
                                        if a5 == 1 {
                                            s3_2 = g::scratchbuffer + 13023;
                                        } else if a5 >= 2 {
                                            if a5 == 2 {
                                                s3_2 = 0x7f4f2f;
                                            } else {
                                                if a5 != 3 {
                                                    break 'b37;
                                                }
                                                s3_2 = g::scratchbuffer + 21167;
                                            }
                                        } else {
                                            if a5 != 0 {
                                                break 'b37;
                                            }
                                            s3_2 = g::scratchbuffer + 8959;
                                        }
                                    }
                                    let v0_20 = g::fxgroups;
                                    'b39: {
                                        let a0_19 = a7 * 92 + (v0_20 + 72);
                                        let mut s1_2 = 0;
                                        if lw(lw(a0_19) + 24) > 0 {
                                            let f20_5 = scale;
                                            let f22_1 = 0.5_f32;
                                            let mut s2_3 = 0;
                                            let f21_2 = lwc1(0x3ad658);
                                            loop {
                                                let a1_15 = lw(a0_19) + 992 + s2_3;
                                                swc1(a1_15, (random_as_float() * f20_5 - f22_1) * f21_2);
                                                let a1_16 = lw(a0_19) + 996 + s2_3;
                                                let f0_14 = random_as_float() * f20_5;
                                                swc1(a1_16, f0_14 + f0_14);
                                                let a1_17 = lw(a0_19) + 1000 + s2_3;
                                                let f0_16 = random_as_float() * f20_5;
                                                let a0_20 = s1_2 << 2;
                                                swc1(a1_17, (f0_16 - f22_1) * f21_2);
                                                sw(lw(a0_19) + a0_20 + 2592, 0);
                                                let a0_21 = lw(a0_19) + 2192 + a0_20;
                                                swc1(a0_21, uint_to_float(s3_2));
                                                s1_2 = s1_2 + 1;
                                                s2_3 = s2_3 + 12;
                                                if s1_2 >= lw(lw(a0_19) + 24) {
                                                    break 'b39;
                                                }
                                            }
                                        }
                                    }
                                    let s1_3 = g::fxgroups;
                                    s3_3 = lw(s1_3 + a7 * 92 + 72);
                                    s2_4 = s3_3 + 928;
                                    sceVu0UnitMatrix(s2_4);
                                    let v0_24 = s1_3 + a7 * 92;
                                    f2_4 = lwc1(0x3ad65c);
                                    s1_4 = v0_24;
                                    swc1(s2_4 + 48, lwc1(v0_24 + 12));
                                    f0_19 = lwc1(v0_24 + 16) + 1.0_f32 - f2_4;
                                    break 'b8;
                                }
                                7 => {
                                    f14_1 = lwc1(a1 + 8);
                                    a0_12 = a5 + 300;
                                    f12_4 = lwc1(a1);
                                    a1_8 = -1;
                                    f13_1 = lwc1(a1 + 4);
                                }
                                8 => {
                                    let s0_3 = g::fxgroups + 72;
                                    let s2_2 = a7 * 92 + s0_3;
                                    let a2_3 = a7 * 92 + (s0_3 - 60);
                                    let v0_11 = particleNewTrail(s2_2, 14, a2_3, 60);
                                    sw(s2_2, v0_11);
                                    let f1_4 = random_as_float();
                                    swc1(a7 * 92 + (s0_3 + 8), (f1_4 * scale - 0.5_f32) * 1e+01_f32);
                                    let f2_2 = random_as_float();
                                    swc1(a7 * 92 + (g::fxgroups + 84), f2_2 * scale + 1.0_f32);
                                    let f1_6 = random_as_float();
                                    a1_7 = a7 * 92 + (g::fxgroups + 88);
                                    let f0_10 = f1_6 * scale;
                                    f1_8 = 1e+01_f32;
                                    f0_11 = f0_10 - 0.5_f32;
                                    break 'b6;
                                }
                                9 => {
                                    memset(sp + 32, 0, 12);
                                    let s0_1 = a7 * 92 + (g::fxgroups + 72);
                                    let v0_3 = particleNewManual(s0_1, 16, sp + 32);
                                    sw(s0_1, v0_3);
                                    if v0_3 == 0 {
                                        break 'b5;
                                    }
                                    'b48: {
                                        'b49: {
                                            'b50: {
                                                if a5 != 1 {
                                                    if a5 >= 2 {
                                                        if a5 == 2 {
                                                            let f1_3 = 3e+01_f32;
                                                            let f0_1 = 127.0_f32;
                                                            swc1(sp + 52, f1_3);
                                                            swc1(sp + 56, f0_1);
                                                            swc1(sp + 48, f1_3);
                                                            break 'b49;
                                                        }
                                                        if a5 != 3 {
                                                            break 'b49;
                                                        }
                                                    } else if a5 != 0 {
                                                        break 'b48;
                                                    }
                                                    swc1(sp + 48, 127.0_f32);
                                                    swc1(sp + 52, 6e+01_f32);
                                                    swc1(sp + 56, 3e+01_f32);
                                                    break 'b49;
                                                }
                                                let f1_3 = 1e+02_f32;
                                                let f0_1 = 3e+01_f32;
                                                swc1(sp + 52, f1_3);
                                                swc1(sp + 56, f0_1);
                                                swc1(sp + 48, f1_3);
                                                break 'b50;
                                            }
                                        }
                                    }
                                    'b52: {
                                        let a1_4 = g::loadbuf + 576;
                                        let mut s0_2 = 0;
                                        if lw(lw(g::fxgroups + a7 * 92 + 72) + 24) > 0 {
                                            loop {
                                                let f20_1 = s0_2 as f32 / lw(lw(a1_4 - 0x3d98 + a7 * 92 + 72) + 24) as f32;
                                                let f12_1 = lwc1(sp + 48) - random_as_float() * scale * 1e+01_f32;
                                                let v0_5 = fptoui(f12_1);
                                                let f12_2 = lwc1(sp + 52) - random_as_float() * scale * 1e+01_f32;
                                                let v0_7 = fptoui(f12_2);
                                                let f12_3 = lwc1(sp + 56) - random_as_float() * scale * 1e+01_f32;
                                                let v0_9 = fptoui(f12_3);
                                                let a1_5 = a7 * 92 + (g::fxgroups + 72);
                                                let a6_1 = v0_5 | v0_7 << 8 | v0_9 << 16;
                                                let a4_1 = g::g_SinTable;
                                                let f5_1 = lwc1(0x3ad688);
                                                let a5_1 = s0_2 << 2;
                                                let a0_8 = (f20_1 * f20_1 * f20_1 * 3.6e+02_f32 * lwc1(0x3ad680) / 1.8e+02_f32 * lwc1(0x3ad684) + 0.5_f32) as i32;
                                                swc1(lw(a1_5) + s0_2 * 12 + 992, lwc1(((a0_8 + 512 & 2047) << 2) + a4_1) * f5_1);
                                                sw(lw(a1_5) + s0_2 * 12 + 996, 0);
                                                swc1(lw(a1_5) + s0_2 * 12 + 1000, lwc1(((a0_8 & 2047) << 2) + a4_1) * f5_1);
                                                swc1(lw(a1_5) + a5_1 + 2592, (1.0_f32 - f20_1) * 6e+01_f32);
                                                let a0_9 = lw(a1_5) + 2192 + a5_1;
                                                swc1(a0_9, uint_to_float(a6_1));
                                                s0_2 = s0_2 + 1;
                                                if s0_2 >= lw(lw(a7 * 92 + (g::fxgroups + 72)) + 24) {
                                                    break 'b52;
                                                }
                                            }
                                        }
                                    }
                                    let s1_1 = g::fxgroups;
                                    let s3_1 = lw(s1_1 + a7 * 92 + 72);
                                    let s2_1 = s3_1 + 928;
                                    sceVu0UnitMatrix(s2_1);
                                    let v0_10 = s1_1 + a7 * 92;
                                    let f2_1 = lwc1(0x3ad68c);
                                    swc1(s2_1 + 48, lwc1(v0_10 + 12));
                                    swc1(s2_1 + 52, lwc1(v0_10 + 16));
                                    swc1(s2_1 + 56, lwc1(v0_10 + 20));
                                    swc1(s3_1 + 928, f2_1);
                                    swc1(s2_1 + 40, f2_1);
                                    swc1(s2_1 + 20, 1.0_f32);
                                    break 'b5;
                                }
                                _ => {
                                    sw(a7 * 92 + (g::fxgroups + 52), 0);
                                    break 'b5;
                                }
                            }
                            let f15_5 = 0.0_f32;
                            let v0_25 = specialPropNew(a0_12, a1_8, f12_4, f13_1, f14_1, f15_5);
                            sw(a7 * 92 + (g::fxgroups + 56), v0_25);
                            if v0_25 != 0 {
                                break 'b5;
                            }
                            break 'b7;
                        }
                        swc1(s2_4 + 52, f0_19);
                        swc1(s2_4 + 56, lwc1(s1_4 + 20));
                        swc1(s3_3 + 928, f2_4);
                        swc1(s2_4 + 40, f2_4);
                        swc1(s2_4 + 20, f2_4);
                        break 'b5;
                    }
                    result = 0;
                    break 'b4;
                }
                swc1(a1_7, f1_8 * f0_11);
            }
        }
        result = a7 * 92 + g::fxgroups;
    }
    result
}
