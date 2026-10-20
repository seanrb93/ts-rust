//! The choice of the animation of the player on the player select screen.

use crate::*;

/// Start the animation `anim` of `chr` with `blend` and `extra` unless it is playing.
fn start_unless_playing(chr: i32, anim: i32, blend: f32, extra: f32) {
    if curAnim(chr, anim) == 0 {
        setAnim(chr, anim, blend, extra);
    }
}

/// Whether the animation `anim` of `chr` is playing in its first slot and the character has the second slot in use as well (the original asks these in this order).
fn slot1_with_slot2(chr: i32, anim: i32) -> bool {
    curAnimSlot1(chr, anim) != 0 && lw(lw(chr + 32) + 152) != 0
}

/// Set the animation of the current player's character from the movement input `f12` (side) and `f13` (forward) and the turn `f14` (@ 0x283f60): a player in the mode 4 is
/// still; in the mode 7 the character turns on the spot towards the angle at +276 of the player (the animations 466 and 467 turn left and right, 468 stands) at the speed of the
/// turn; in the others, a mode 2 player high above the ground falls (469); the speed of the input smooths the walk rate at +2960, and when the input is strong the character
/// walks in the direction of the input (the angle at +224 from the arctangent of the input; the forward animations 6, 7, 10, 11 or the running ones 116 to 119 by the weapon
/// flag, the sideways ones 4, 5, 8, 9 or 114, 115, 120, 121) with its speed set from the root movement of the animation; when the input is weak the character stands, turning
/// on the spot (12 and 13, or 122 and 123 with the flag) towards the angle it faces beyond the limits -25 and 65 degrees, or standing (3 or 113).
pub fn playerselectanim(f12: f32, f13: f32, f14: f32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    let s1 = lw(player + 384);
    let mode = lw(player + 296);
    let s0 = lw(s1 + 352);
    if mode == 4 {
        sw(s0 + 2968, 0);
        let facing = lwc1(s1 + 76);
        swc1(s1 + 80, facing);
        swc1(s0 + 2972, facing);
        return;
    }
    if mode == 7 {
        let mut speed = f14 * 2.5_f32;
        if !(0.0_f32 <= speed) {
            speed = -speed;
        }
        'turn: {
            'b27: {
                'b28: {
                    let diff = anglediff(lwc1(player + 276), lwc1(s1 + 76));
                    let anim;
                    let blend;
                    if !(diff < 0.0_f32) {
                        if !(0.0_f32 < diff) {
                            if curAnim(s1, 468) == 0 {
                                setAnim(s1, 468, lwc1(0x3ad108), 1.0_f32);
                            }
                            curAnimSpeed(lw(s1 + 32), 1.0_f32);
                            break 'turn;
                        }
                        if curAnim(s1, 466) != 0 {
                            if curAnimSlot1(s1, 466) == 0 {
                                break 'b27;
                            }
                            if lw(lw(s1 + 32) + 152) == 0 {
                                break 'b28;
                            }
                            if curAnimSlot2(s1, 466) != 0 {
                                break 'b27;
                            }
                        }
                        blend = lwc1(0x3ad104);
                        anim = 466;
                    } else {
                        if curAnim(s1, 467) != 0 {
                            if curAnimSlot1(s1, 467) == 0 {
                                break 'b27;
                            }
                            if lw(lw(s1 + 32) + 152) == 0 {
                                break 'b28;
                            }
                            if curAnimSlot2(s1, 467) != 0 {
                                break 'b27;
                            }
                        }
                        blend = lwc1(0x3ad100);
                        anim = 467;
                    }
                    setAnim(s1, anim, blend, 1.0_f32);
                }
            }
            curAnimSpeed(lw(s1 + 32), speed);
        }
        'b33: {
            'b34: {
                'b35: {
                    if curAnimSlot1(s1, 466) != 0 {
                        if lw(lw(s1 + 32) + 152) != 0 {
                            break 'b35;
                        }
                    }
                    if curAnimSlot1(s1, 467) == 0 {
                        break 'b33;
                    }
                    if lw(lw(s1 + 32) + 152) == 0 {
                        break 'b34;
                    }
                }
                if curAnimSlot2(s1, 466) != 0 {
                    break 'b33;
                }
                if curAnimSlot2(s1, 467) != 0 {
                    break 'b33;
                }
                if curAnim(s1, 468) == 0 {
                    setAnim(s1, 468, lwc1(0x3ad10c), 1.0_f32);
                }
                curAnimSpeed(lw(s1 + 32), 1.0_f32);
            }
        }
        let angle = lwc1(lw(g::currentplayer) + 276);
        swc1(s1 + 84, angle);
        swc1(s1 + 80, angle);
        return;
    }
    if mode == 2 {
        if 1.0_f32 < lwc1(player + 144) - lwc1(s0 + 0x1198) {
            if curAnim(s1, 469) != 0 {
                if curAnimSlot1(s1, 469) == 0 {
                    return;
                }
                if lw(lw(s1 + 32) + 152) == 0 {
                    return;
                }
                if curAnimSlot2(s1, 469) != 0 {
                    return;
                }
            }
            setAnim(s1, 469, lwc1(0x3ad110), 1.5_f32);
            return;
        }
    }
    let strength = (f12 * 8.0_f32 * f12 * 8.0_f32 + f13 * 7.0_f32 * f13 * 7.0_f32).abs().sqrt();
    let unit = 113.0_f32.abs().sqrt();
    if !((unit * 0.5_f32) < strength) {
        let rate = lwc1(s0 + 2960);
        swc1(s0 + 2960, rate + (strength - unit * 0.5_f32) / unit * lwc1(0x3ad118) * rate);
    } else {
        let rate = lwc1(s0 + 2960);
        swc1(s0 + 2960, rate + (strength - unit * 0.5_f32) / unit * lwc1(0x3ad114) * (1.0_f32 - rate));
    }
    if !(strength < lwc1(0x3ad11c)) {
        let mut side = fptodp(f12);
        if dpcmp(side, 0_i64) < 0 {
            side = dpsub(0_i64, side);
        }
        let mut forward = fptodp(f13 * lwc1(0x3ad150));
        if dpcmp(forward, 0_i64) < 0 {
            forward = dpsub(0_i64, forward);
        }
        let root_axis;
        if dpcmp(side, forward) <= 0 {
            let mut player_now = lw(g::currentplayer);
            'b8: {
                if f12 == 0.0_f32 {
                    swc1(player_now + 224, 0.0_f32);
                } else {
                    'b9: {
                        let slope = fptodp(-f13 / f12);
                        let arc = atan(slope);
                        player_now = lw(g::currentplayer);
                        let degrees = dptofp(dpdiv(dpmul(arc, ld(0x3a8840)), ld(0x3a8848)));
                        swc1(player_now + 224, degrees);
                        if !(f12 < 0.0_f32) {
                            if !(0.0_f32 < f13) {
                                break 'b9;
                            }
                        } else if 0.0_f32 < f13 {
                            break 'b9;
                        }
                        swc1(player_now + 224, degrees + 9e+01_f32);
                        break 'b8;
                    }
                    let degrees = lwc1(player_now + 224);
                    swc1(player_now + 224, degrees - 9e+01_f32);
                }
            }
            swc1(s1 + 80, lwc1(player_now + 276));
            swc1(s1 + 84, lwc1(player_now + 276) + lwc1(player_now + 224));
            let flags = lw(player_now + 20);
            let fast = 3.0_f32 < strength;
            let (plain, armed, plain_blend, armed_blend) = if !(0.0_f32 < f13) {
                if !fast {
                    (11, 116, 0x3ad190, 0x3ad18c)
                } else {
                    (7, 118, 0x3ad188, 0x3ad184)
                }
            } else if !fast {
                (10, 117, 0x3ad180, 0x3ad17c)
            } else {
                (6, 119, 0x3ad178, 0x3ad174)
            };
            if (flags & 1) == 0 {
                start_unless_playing(s1, plain, lwc1(plain_blend), 1.0_f32);
            } else {
                start_unless_playing(s1, armed, lwc1(armed_blend), 1.0_f32);
            }
            root_axis = 0;
        } else {
            let slope = fptodp(-f13 / f12);
            let arc = atan(slope);
            let player_now = lw(g::currentplayer);
            let degrees = dptofp(dpdiv(dpmul(arc, ld(0x3a8830)), ld(0x3a8838)));
            swc1(player_now + 224, degrees);
            swc1(s1 + 80, lwc1(player_now + 276));
            swc1(s1 + 84, lwc1(player_now + 276) + lwc1(player_now + 224));
            let flags = lw(player_now + 20);
            let fast = 4.0_f32 < strength;
            let (plain, armed, plain_blend, armed_blend) = if !(0.0_f32 < f12) {
                if !fast {
                    (9, 121, 0x3ad170, 0x3ad16c)
                } else {
                    (5, 115, 0x3ad168, 0x3ad164)
                }
            } else if !fast {
                (8, 120, 0x3ad160, 0x3ad15c)
            } else {
                (4, 114, 0x3ad158, 0x3ad154)
            };
            if (flags & 1) == 0 {
                start_unless_playing(s1, plain, lwc1(plain_blend), 1.0_f32);
            } else {
                start_unless_playing(s1, armed, lwc1(armed_blend), 1.0_f32);
            }
            root_axis = 2;
        }
        extractAnimRoot(s1, sp, sp | 4, sp | 8, root_axis);
        let moved = lwc1(sp + 4) - lwc1(sp);
        let mut size = fptodp(moved);
        if dpcmp(size, 0_i64) < 0 {
            size = dpsub(0_i64, size);
        }
        let distance = dptofp(size);
        if distance == 0.0_f32 {
            curAnimSpeed(lw(s1 + 32), strength + 0.75_f32);
        } else {
            let speed = strength * lwc1(sp + 8) / (distance * 6e+01_f32) / lwc1(0x3ad194);
            curAnimSpeed(lw(s1 + 32), speed);
        }
        return;
    }
    'b12: {
        let diff = anglediff(lwc1(s1 + 80), lwc1(s1 + 76));
        if !(65.0_f32 < diff) {
            if !(diff < -25.0_f32) {
                if curAnim(s1, 12) == 0 && curAnim(s1, 13) == 0 && curAnim(s1, 122) == 0 && curAnim(s1, 123) == 0 {
                    if (lw(lw(g::currentplayer) + 20) & 1) == 0 {
                        start_unless_playing(s1, 3, lwc1(0x3ad144), 1.0_f32);
                    } else {
                        start_unless_playing(s1, 113, lwc1(0x3ad140), 1.0_f32);
                    }
                }
            } else {
                'b13: {
                    if curAnim(s1, 12) == 0 {
                        if curAnim(s1, 122) == 0 {
                            break 'b13;
                        }
                    }
                    if slot1_with_slot2(s1, 12) {
                        if curAnimSlot2(s1, 12) == 0 {
                            break 'b13;
                        }
                    }
                    if curAnimSlot1(s1, 122) == 0 {
                        break 'b12;
                    }
                    if lw(lw(s1 + 32) + 152) == 0 {
                        break 'b12;
                    }
                    if curAnimSlot2(s1, 122) != 0 {
                        break 'b12;
                    }
                }
                if (lw(lw(g::currentplayer) + 20) & 1) == 0 {
                    setAnim(s1, 12, lwc1(0x3ad138), lwc1(0x3ad13c));
                } else {
                    setAnim(s1, 122, lwc1(0x3ad130), lwc1(0x3ad134));
                }
            }
        } else {
            'b16: {
                if curAnim(s1, 13) == 0 {
                    if curAnim(s1, 123) == 0 {
                        break 'b16;
                    }
                }
                if slot1_with_slot2(s1, 13) {
                    if curAnimSlot2(s1, 13) == 0 {
                        break 'b16;
                    }
                }
                if curAnimSlot1(s1, 123) == 0 {
                    break 'b12;
                }
                if lw(lw(s1 + 32) + 152) == 0 {
                    break 'b12;
                }
                if curAnimSlot2(s1, 123) != 0 {
                    break 'b12;
                }
            }
            if (lw(lw(g::currentplayer) + 20) & 1) == 0 {
                setAnim(s1, 13, lwc1(0x3ad128), lwc1(0x3ad12c));
            } else {
                setAnim(s1, 123, lwc1(0x3ad120), lwc1(0x3ad124));
            }
        }
    }
    'b19: {
        'b20: {
            if slot1_with_slot2(s1, 12) {
                break 'b20;
            }
            if slot1_with_slot2(s1, 122) {
                break 'b20;
            }
            if slot1_with_slot2(s1, 13) {
                break 'b20;
            }
            if curAnimSlot1(s1, 123) == 0 {
                break 'b19;
            }
            if lw(lw(s1 + 32) + 152) == 0 {
                break 'b19;
            }
        }
        if curAnimSlot2(s1, 12) == 0 && curAnimSlot2(s1, 122) == 0 && curAnimSlot2(s1, 13) == 0 && curAnimSlot2(s1, 123) == 0 {
            if (lw(lw(g::currentplayer) + 20) & 1) == 0 {
                start_unless_playing(s1, 3, lwc1(0x3ad14c), 1.0_f32);
            } else {
                start_unless_playing(s1, 113, lwc1(0x3ad148), 1.0_f32);
            }
        }
    }
    let rate;
    if curAnim(s1, 113) != 0 {
        rate = lwc1(s0 + 2960);
    } else {
        if curAnim(s1, 3) == 0 {
            return;
        }
        rate = lwc1(s0 + 2960);
    }
    curAnimSpeed(lw(s1 + 32), rate + rate + 1.0_f32);
}
