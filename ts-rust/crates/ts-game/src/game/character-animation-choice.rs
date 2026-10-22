//! Function: choosing the animation of a character from what it is doing, and the speed it plays at.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    random_as_float() * 2.3283064e-10_f32
}

/// Whether the character plays any of the animations.
fn playing_any(chr: i32, anims: &[i32]) -> bool {
    for anim in anims {
        if curAnim(chr, *anim) != 0 {
            return true;
        }
    }
    false
}

/// Start the animation `anim` blended in over `blend` unless it is playing already.
fn play_unless_playing(chr: i32, anim: i32, blend: f32) {
    if curAnim(chr, anim) == 0 {
        setAnim(chr, anim, blend, 1.0_f32);
    }
}

/// The animation of a character that stands still (the end of the standing logic): when it was fired at it may flinch (animation 463, or 462 / 464 / 460 by the weapon it
/// holds); otherwise it takes the idle animation 3, or the aiming one 113 when it aims (a different blend when it has bases).
fn idle_animation(a0: i32, mind: i32, blend: f32, flinch_chance: f32, aim_blend: f32) {
    if (lw(mind + 2708) & 4) == 0 {
        if enemyFiredAt(a0) != 0 && !playing_any(a0, &[460, 462, 463, 464]) {
            if random_unit() < flinch_chance {
                let mut anim = 463;
                if lw(mind + 484) == 0 {
                    if lw(mind + 440) != 0 {
                        let gun = lw(mind + 376);
                        anim = if (lw(gun * 400 + (g::guns + 28)) & 2) == 0 { 462 } else { 464 };
                    } else {
                        anim = 460;
                    }
                }
                setAnim(a0, anim, blend, 1.0_f32);
                return;
            }
        }
        if playing_any(a0, &[3, 460, 462, 463, 464]) {
            return;
        }
        setAnim(a0, 3, blend, 1.0_f32);
        return;
    }
    if (lw(mind + 2716) & 0x80000) != 0 {
        if curAnimBases(a0, 113, 0, 0, 0) == 0 {
            setAnim(a0, 113, aim_blend, 1.0_f32);
            return;
        }
    }
    if curAnim(a0, 113) != 0 {
        return;
    }
    setAnim(a0, 113, blend, 1.0_f32);
}

/// Whether the animation `anim` is on the first slot while the second slot is in use (a blend is going on).
fn on_first_slot_blending(a0: i32, anim: i32) -> bool {
    curAnimSlot1(a0, anim) != 0 && lw(lw(a0 + 32) + 152) != 0
}

/// Choose the animation of a character that walks or runs (not standing, not stunned): by its speed (forward or back, slow or fast), whether it aims, and the flags that
/// say it moves sideways.
fn choose_moving_animation(a0: i32, mind: i32, flags: i32) {
    let previous = lw(mind + 2712);
    let speed = lwc1(mind + 2876);
    let top = lwc1(mind + 2860);
    let aiming = (flags & 4) != 0;
    if (flags & 2) == 0 {
        let blend = if (previous & 2) != 0 { 0.0_f32 } else { 0.25_f32 };
        if 0.0_f32 <= speed {
            if !(top < speed) {
                if !aiming {
                    if enemyFiredAt(a0) != 0 && !playing_any(a0, &[444, 445, 446]) {
                        if random_unit() < lwc1(0x3ad848) {
                            let armed = lw(mind + 440);
                            swc1(mind + 2876, lwc1(mind + 2860));
                            if armed == 0 {
                                setAnim(a0, 444, blend, lwc1(0x3ad850));
                            } else {
                                setAnim(a0, 446, blend, lwc1(0x3ad84c));
                            }
                            return;
                        }
                    }
                    if playing_any(a0, &[8, 444, 445, 446]) {
                        if curAnimComplete(lw(a0 + 32)) == 0 {
                            return;
                        }
                    }
                    setAnim(a0, 8, blend, 1.0_f32);
                    return;
                }
                play_unless_playing(a0, 120, blend);
                return;
            }
            if !aiming {
                if curAnim(a0, 4) != 0 {
                    if curAnimComplete(lw(a0 + 32)) == 0 {
                        return;
                    }
                    if lw(mind + 2776) >= 0 && (lw(mind + 2704) & 8) != 0 && 25.0_f32 < lwc1(mind + 2948) {
                        if lwc1(0x3ad844) < random_unit() {
                            setAnim(a0, 443, 0.0_f32, 1.0_f32);
                            swc1(mind + 2948, 0.0_f32);
                            sw(mind + 2704, lw(mind + 2704) & -9);
                            return;
                        }
                    }
                    setAnim(a0, 4, 0.0_f32, 1.0_f32);
                    return;
                }
                if lw(mind + 2776) >= 0 && (lw(mind + 2704) & 8) != 0 && lw(mind + 2716) != 2 && 25.0_f32 < lwc1(mind + 2948) {
                    if lwc1(0x3ad840) < random_unit() {
                        setAnim(a0, 443, blend, 1.0_f32);
                        sw(mind + 2948, 0);
                        sw(mind + 2704, lw(mind + 2704) & -9);
                        return;
                    }
                }
                setAnim(a0, 4, blend, 1.0_f32);
                return;
            }
            play_unless_playing(a0, 114, blend);
            return;
        }
        if speed < -top {
            play_unless_playing(a0, if aiming { 115 } else { 5 }, blend);
            return;
        }
        play_unless_playing(a0, if aiming { 121 } else { 9 }, blend);
        return;
    }
    let blend = if (previous & 1) != 0 { 0.0_f32 } else { 0.25_f32 };
    if !(0.0_f32 < speed) {
        if !(speed < -top) {
            play_unless_playing(a0, if aiming { 116 } else { 11 }, blend);
        } else {
            play_unless_playing(a0, if aiming { 118 } else { 7 }, blend);
        }
    } else if !(top < speed) {
        play_unless_playing(a0, if aiming { 117 } else { 10 }, blend);
    } else {
        play_unless_playing(a0, if aiming { 119 } else { 6 }, blend);
    }
}

/// Start a turning animation (12 / 122 turning one way, 13 / 123 the other; the second of each is for aiming) unless the turn is playing already: `opposite` is the turn the
/// other way, which must not be playing, and the turn is left alone while it is on the first slot with the second one in use.
fn start_turn(a0: i32, mind: i32, blend: f32, opposite: i32, turn: i32, turn_aiming: i32) {
    if curAnim(a0, opposite) != 0 {
        return;
    }
    let mut start = false;
    if curAnim(a0, turn) == 0 && curAnim(a0, turn_aiming) == 0 {
        start = true;
    }
    if !start {
        if on_first_slot_blending(a0, turn) && curAnimSlot2(a0, turn) == 0 {
            start = true;
        }
    }
    if !start {
        if curAnimSlot1(a0, turn_aiming) == 0 {
            return;
        }
        if lw(lw(a0 + 32) + 152) == 0 {
            return;
        }
        if curAnimSlot2(a0, turn_aiming) != 0 {
            return;
        }
    }
    let anim = if (lw(mind + 2708) & 4) == 0 { turn } else { turn_aiming };
    setAnim(a0, anim, blend, 1.0_f32);
}

/// Set the animation of the character `a0` (@ 0x2bd008) from its state: a character with a one-shot animation keeps it until it ends; zombies and time splitters have
/// their own choices; the others take a hit reaction, a walk or run by speed, or the standing and turning animations; the speed of the animation is then set from the
/// movement of its root.
pub fn setAnimation(a0: i32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mind = lw(a0 + 352);
    let flags = lw(mind + 2708);
    if (flags & 512) != 0 {
        if curAnimComplete(lw(a0 + 32)) != 0 {
            if (lw(lw((lw(lw(a0 + 32) + 96) << 2) + g::pAnimations) + 12) & 1) != 0 {
                sw(mind + 2708, lw(mind + 2712));
            }
        }
        return;
    }
    if lw(mind + 8) == 1 {
        setZombieAnimation(a0);
        return;
    }
    if (flags & 0x1020) != 0 {
        return;
    }
    if lw(mind + 8) == 2 {
        setTimeSplitterAnimation(a0);
        return;
    }
    if (flags & 256) != 0 {
        if curAnimComplete(lw(a0 + 32)) != 0 {
            sw(mind + 2708, lw(mind + 2708) & -257);
        }
        return;
    }
    'choice: {
        if (flags & 1024) != 0 {
            if playing_any(a0, &[450, 452, 453, 454]) {
                break 'choice;
            }
            if lw(mind + 440) == 0 {
                setAnim(a0, 452, 0.25_f32, 1.0_f32);
            } else if !(random_unit() < 0.5_f32) {
                setAnim(a0, 454, 0.25_f32, 1.0_f32);
            } else {
                setAnim(a0, 453, 0.25_f32, 1.0_f32);
            }
            break 'choice;
        }
        if (flags & 8) == 0 {
            choose_moving_animation(a0, mind, flags);
            break 'choice;
        }
        let blend = if (lw(mind + 2712) & 2) != 0 { 0.0_f32 } else { 0.25_f32 };
        let turn = anglediff(lwc1(a0 + 80), lwc1(a0 + 76));
        'standing: {
            if !(45.0_f32 < turn) {
                if !(turn < -45.0_f32) {
                    if playing_any(a0, &[12, 13, 122, 123]) {
                        break 'standing;
                    }
                    if !(lwc1(mind + 2876) <= lwc1(lw(a0 + 32) + 140) * 0.25_f32 * 6e+01_f32 * lwc1(mind + 2868)) {
                        break 'standing;
                    }
                    idle_animation(a0, mind, blend, lwc1(0x3ad830), lwc1(0x3ad82c));
                } else {
                    start_turn(a0, mind, blend, 13, 12, 122);
                }
            } else {
                start_turn(a0, mind, blend, 12, 13, 123);
            }
        }
        let mut anim_now = None;
        'second: {
            if on_first_slot_blending(a0, 12) || on_first_slot_blending(a0, 122) || on_first_slot_blending(a0, 13) {
                // fall to the second slot checks
            } else {
                if curAnimSlot1(a0, 123) == 0 {
                    anim_now = Some(lw(lw(a0 + 32) + 96));
                    break 'second;
                }
                if lw(lw(a0 + 32) + 152) == 0 {
                    anim_now = Some(lw(lw(a0 + 32) + 96));
                    break 'second;
                }
            }
            if curAnimSlot2(a0, 12) == 0 && curAnimSlot2(a0, 122) == 0 && curAnimSlot2(a0, 13) == 0 && curAnimSlot2(a0, 123) == 0 {
                let state = lw(a0 + 32);
                if !(lwc1(mind + 2876) <= lwc1(state + 140) * 0.25_f32 * 6e+01_f32 * lwc1(mind + 2868)) {
                    anim_now = Some(lw(state + 96));
                    break 'second;
                }
                idle_animation(a0, mind, blend, lwc1(0x3ad838), lwc1(0x3ad834));
            }
        }
        let anim_now = match anim_now {
            Some(anim) => anim,
            None => lw(lw(a0 + 32) + 96),
        };
        if anim_now < 443 {
            break 'choice;
        }
        if curAnimComplete(lw(a0 + 32)) == 0 {
            break 'choice;
        }
        if (lw(mind + 2708) & 4) == 0 {
            play_unless_playing(a0, 3, blend);
            break 'choice;
        }
        if (lw(mind + 2716) & 0x80000) != 0 {
            if curAnimBases(a0, 113, 0, 0, 0) == 0 {
                setAnim(a0, 113, lwc1(0x3ad83c), 1.0_f32);
                break 'choice;
            }
        }
        play_unless_playing(a0, 113, blend);
    }
    let flags_now = lw(mind + 2708);
    if (flags_now & 2) == 0 {
        extractAnimRoot(a0, sp, sp | 4, sp | 8, 2);
    } else {
        extractAnimRoot(a0, sp, sp | 4, sp | 8, 0);
    }
    let rate = lwc1(mind + 2784);
    let f20_4 = if rate == lwc1(0x3ad854) {
        1.0_f32
    } else {
        let mut size = fptodp(rate);
        if dpcmp(size, 0_i64) < 0 {
            size = dpsub(0_i64, size);
        }
        let ratio = dpdiv(size, ld(0x3a9ce0));
        let scaled = dpmul(ratio, ld(0x3a9ce8));
        dptofp(dpsub(0x3ff0000000000000_i64, scaled))
    };
    let mut travelled = fptodp(lwc1(sp + 4) - lwc1(sp));
    if dpcmp(travelled, 0_i64) < 0 {
        travelled = dpsub(0_i64, travelled);
    }
    let distance = dptofp(travelled);
    if !playing_any(a0, &[12, 13, 122, 123]) && !(distance == 0.0_f32) && !(lwc1(mind + 2876) == 0.0_f32) {
        let factor = fptodp(f20_4);
        let mut speed = fptodp(lwc1(mind + 2876));
        if dpcmp(speed, 0_i64) < 0 {
            speed = dpsub(0_i64, speed);
        }
        let step = fptodp(lwc1(sp + 8));
        let moved = dpmul(speed, step);
        let per_frame = fptodp(distance * 6e+01_f32);
        let ratio = dpdiv(moved, per_frame);
        let result = dpdiv(dpmul(factor, ratio), ld(0x3a9cf0));
        curAnimSpeed(lw(a0 + 32), dptofp(result));
    } else {
        curAnimSpeed(lw(a0 + 32), 1.0_f32);
    }
    sw(mind + 2712, lw(mind + 2708));
}
