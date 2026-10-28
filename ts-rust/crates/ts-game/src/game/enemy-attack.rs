//! Function: the attack of a computer character (picking the state of the attack, moving, and the melee swipe).

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// The size of a float as a double.
fn dp_abs(x: f32) -> i64 {
    let value = fptodp(x);
    if dpcmp(value, 0_i64) < 0 {
        dpsub(0_i64, value)
    } else {
        value
    }
}

/// How the tick of the attack ends: it simply stops, it clears the timer of the state, or (when the target is dead) it first makes the character lose interest.
enum Flow {
    Return,
    Reset,
    Dead,
}

/// Check the target (@ the end of the attack): when it is not a character, or not dead, the tick stops, else the character loses interest.
fn check_target(mind: i32, target: i32) -> Flow {
    if lw(target + 8) != 8 {
        return Flow::Return;
    }
    if lw(lw(target + 352) + 2708) != 32 {
        return Flow::Return;
    }
    Flow::Dead
}

/// The end of the states when the state ends the same way: with the target flag `has_target` set the target is checked, else the state 8 and 15 go on to check it and
/// any other state is replaced by state 8.
fn finish_with_flag(mind: i32, has_target: i32) -> Flow {
    if has_target != 0 {
        return check_target(mind, lw(mind + 2788));
    }
    finish_with_state(mind, lw(mind + 2728))
}

/// See `finish_with_flag`: the state at the end of the tick is `state`.
fn finish_with_state(mind: i32, state: i32) -> Flow {
    if state != 15 {
        if state == 8 {
            return check_target(mind, lw(mind + 2788));
        }
        sw(mind + 2728, 8);
        return Flow::Reset;
    }
    check_target(mind, lw(mind + 2788))
}

/// Go to the state `new_state` and finish the tick.
fn go_to(mind: i32, new_state: i32) -> Flow {
    let has_target = lw(mind + 2772);
    sw(mind + 2728, new_state);
    finish_with_flag(mind, has_target)
}

/// Finish the tick with nothing more to do.
fn stay(mind: i32) -> Flow {
    finish_with_flag(mind, lw(mind + 2772))
}

/// Finish the tick after the animation that began: flags are replaced by `flags`.
fn set_flags_and_stay(mind: i32, flags: i32) -> Flow {
    let has_target = lw(mind + 2772);
    sw(mind + 2704, flags);
    finish_with_flag(mind, has_target)
}

/// Put the position of the target in the place the character walks to (+2796).
fn aim_at_target(mind: i32, target: i32) {
    swc1(mind + 2796, lwc1(target + 48));
    swc1(mind + 2800, lwc1(target + 52));
    swc1(mind + 2804, lwc1(target + 56));
}

/// Squared distance between two points.
fn squared_distance(a: i32, b: i32) -> f32 {
    let dx = ps2float::sub(lwc1(a), lwc1(b));
    let dy = ps2float::sub(lwc1(a + 4), lwc1(b + 4));
    let dz = ps2float::sub(lwc1(a + 8), lwc1(b + 8));
    ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz))
}

/// The state 4 and 6 end: when the aim animation `anim` is playing the character may stop (state 7) after a while or a big change of height; else it starts the animation.
fn aim_animation(a0: i32, mind: i32, anim: i32, speed: f32) -> Flow {
    enemySetMoveMode(a0, 512);
    if curAnim(a0, anim) != 0 {
        let height = lwc1(mind + 2780);
        if !(5.0_f32 < height) {
            if height < -2e+01_f32 {
                sw(mind + 2892, 0);
            } else if !(3.0_f32 < lwc1(mind + 2892)) {
                return stay(mind);
            } else {
                sw(mind + 2892, 0);
            }
        } else {
            sw(mind + 2892, 0);
        }
        sw(mind + 2728, 7);
        enemySetMoveMode(a0, 8);
        return stay(mind);
    }
    setAnim(a0, anim, 0.25_f32, speed);
    let flags = lw(mind + 2704);
    sw(mind + 2892, 0);
    set_flags_and_stay(mind, flags | 6)
}

/// Rotate the move in the vector at `sp` by the heading and set the place at +2820 to go to, then test the move.
fn heading_move(a0: i32, mind: i32, sp: i32, scratch: i32, heading: f32, reach: f32) -> i32 {
    matrixRotY(scratch, heading);
    matrixVecRotAligned(&[scratch as i64, sp as i64]);
    let player = lw(g::currentplayer);
    let index = (mind - lw(g::chrinfo)).wrapping_mul(-0x795b3d1f);
    swc1(mind + 2820, ps2float::add(lwc1(sp), lwc1(a0 + 48)));
    swc1(mind + 2824, ps2float::add(lwc1(sp + 4), lwc1(a0 + 52)));
    swc1(mind + 2828, ps2float::add(lwc1(sp + 8), lwc1(a0 + 56)));
    moveTest(index >> 4, a0 + 48, mind + 2820, 0, 0, 0, reach, lwc1(player + 132))
}

/// The attack state machine for a character that can shoot or fight (the states in +2728): 0 starts, 1 and 2 close in on the target or keep the range, 3 waits to aim,
/// 4 / 6 aim at it, 5 shoots, 7 picks the next thing to do (shoot, throw, roll, dodge...), 8 gives up, 9 looks for a better pad, 10 goes to it, 11 and 12 decide to dodge,
/// 13 dodges, 14 and 16 roll, 15 and 17 play the roll animations, 18 fights in melee.
fn attack_state(a0: i32, mind: i32, sp: i32, kind: i32, state: i32, dist: f32) -> Flow {
    let weapon = lw(mind + 376) * 12;
    match state {
        0 => {
            if 5.0_f32 < lwc1(mind + 2912) {
                enemyPlayActivationSound(a0);
            }
            if lw(mind + 8) != 1 {
                sw(mind + 2728, 1);
                return stay(mind);
            }
            go_to(mind, 18)
        }
        1 => {
            let target = lw(mind + 2788);
            if dpcmp(dp_abs(ps2float::sub(lwc1(a0 + 52), lwc1(target + 52))), 0x3ff8000000000000_i64) <= 0 {
                if dist < lwc1(weapon + (g::firerange + 4)) {
                    return go_to(mind, 2);
                }
                swc1(mind + 2876, lwc1(mind + 2856));
                enemySetMoveMode(a0, 1);
                aim_at_target(mind, lw(mind + 2788));
                return stay(mind);
            }
            sw(mind + 2728, 2);
            enemySetMoveMode(a0, 1);
            stay(mind)
        }
        2 => {
            let target = lw(mind + 2788);
            aim_at_target(mind, target);
            let max_range = lwc1(weapon + (g::firerange + 4));
            if !(dist < ps2float::mul(ps2float::add(max_range, lwc1(weapon + g::firerange)), 0.5_f32)) {
                if dist < max_range {
                    enemySetMoveMode(a0, 1);
                    if !(enemyReactTime(a0) < lwc1(mind + 2912)) {
                        return stay(mind);
                    }
                    let flags = lw(mind + 2704);
                    sw(mind + 2892, 0);
                    return set_flags_and_stay(mind, flags | 2);
                }
                if dpcmp(dp_abs(ps2float::sub(lwc1(a0 + 52), lwc1(target + 52))), 0x3ff8000000000000_i64) <= 0 {
                    return go_to(mind, 1);
                }
            }
            enemySetMoveMode(a0, 8);
            if !(enemyReactTime(a0) < lwc1(mind + 2912)) {
                return stay(mind);
            }
            sw(mind + 2892, 0);
            sw(mind + 2728, 3);
            stay(mind)
        }
        3 => {
            enemySetMoveMode(a0, 8);
            if dpcmp(dp_abs(lwc1(mind + 2780)), 0x4024000000000000_i64) >= 0 {
                return stay(mind);
            }
            let flags = lw(mind + 2704);
            sw(mind + 2728, 7);
            sw(mind + 2892, 0);
            sw(mind + 2704, flags | 2);
            stay(mind)
        }
        4 => aim_animation(a0, mind, 473, 1.0_f32),
        5 => {
            enemySetMoveMode(a0, 4);
            let flags = lw(mind + 2704);
            if (flags & 2) != 0 {
                sw(mind + 2704, flags | 6);
            }
            if 3.5_f32 < lwc1(mind + 2892) {
                let flags2 = lw(mind + 2708);
                sw(mind + 2892, 0);
                sw(mind + 2728, 1);
                sw(mind + 2708, flags2 & -5);
            }
            if dist < ps2float::sub(lwc1(weapon + g::firerange), 2.0_f32) {
                sw(mind + 2728, 12);
                return stay(mind);
            }
            stay(mind)
        }
        6 => {
            if (lw(lw(mind + 376) * 400 + (GUNS_TABLE + 28)) & 8) != 0 {
                return aim_animation(a0, mind, 449, lwc1(0x3ad8ec));
            }
            go_to(mind, 3)
        }
        7 => {
            if !(ps2float::add(lwc1(weapon + (g::firerange + 4)), 1.0_f32) < dist) {
                if !(dist < ps2float::sub(lwc1(weapon + g::firerange), 2.0_f32)) {
                    enemySetMoveMode(a0, 8);
                } else {
                    sw(mind + 2728, 12);
                }
            } else {
                sw(mind + 2728, 1);
            }
            if !(enemyReactTime(a0) < lwc1(mind + 2892)) {
                return stay(mind);
            }
            sw(mind + 2892, 0);
            let roll = random_unit();
            let skill = lwc1(mind + 2832);
            if roll < ps2float::add(ps2float::mul(skill, lwc1(0x3ad8f0)), lwc1(0x3ad8f4)) {
                if ps2float::add(lwc1(weapon + g::firerange), 1.0_f32) < dist {
                    if !(skill < 0.5_f32) {
                        return go_to(mind, 3);
                    }
                    if lw(mind + 332) != 0 {
                        return go_to(mind, 3);
                    }
                    return go_to(mind, 6);
                }
            }
            if roll < 0.5_f32 {
                return go_to(mind, 3);
            }
            if dpcmp(fptodp(roll), ld(0x3a9dd0)) >= 0 {
                if dpcmp(fptodp(roll), ld(0x3a9dd8)) >= 0 {
                    if (lw(lw(mind + 376) * 400 + (GUNS_TABLE + 28)) & 1) == 0 {
                        return go_to(mind, 3);
                    }
                    if lw(mind + 332) != 0 {
                        return go_to(mind, 3);
                    }
                    return go_to(mind, 4);
                }
                if skill < 0.5_f32 {
                    return go_to(mind, 14);
                }
            } else if skill < 0.5_f32 {
                return go_to(mind, 16);
            }
            go_to(mind, 3)
        }
        8 => {
            if kind != 2 {
                enemySetMoveMode(a0, 8);
            }
            let flags = lw(mind + 2708);
            if (flags & 256) != 0 {
                sw(mind + 2708, flags & -257);
            }
            let has_target = lw(mind + 2772);
            if !(lwc1(0x3ad8f8) < lwc1(mind + 2892)) {
                if has_target == 0 {
                    return finish_with_state(mind, lw(mind + 2728));
                }
            } else if has_target == 0 {
                let saved = lw(mind + 2716);
                sw(mind + 2716, 256);
                sw(mind + 2720, saved);
                sw(mind + 2728, 0);
                return Flow::Reset;
            }
            let kind_now = lw(mind + 8);
            if kind_now != 1 {
                if kind_now != 2 {
                    sw(mind + 2728, 11);
                    return finish_with_flag(mind, has_target);
                }
                sw(mind + 2728, 0);
                return finish_with_flag(mind, has_target);
            }
            sw(mind + 2728, 18);
            finish_with_flag(mind, has_target)
        }
        9 => {
            let pad = lw(mind + 680);
            let count = findpadslinked2pad(pad, sp + 16, 20);
            if count > 0 {
                let mut n = 0;
                let mut entry = sp + 16;
                let mut found = false;
                loop {
                    let current = enemyGetTargetPad(a0);
                    let candidate = lw(entry);
                    if current != candidate {
                        let in_front = checkInFront(a0, lw(g::levelpads) + candidate * 28 + 12, 0.0_f32);
                        let pads = lw(g::levelpads);
                        if in_front != 0 {
                            let at = lw(entry) * 28 + pads;
                            let range = lwc1(lw(mind + 376) * 12 + g::firerange);
                            swc1(mind + 2796, lwc1(at + 12));
                            swc1(mind + 2800, lwc1(at + 16));
                            swc1(mind + 2804, lwc1(at + 20));
                            let dx = ps2float::sub(lwc1(a0 + 48), lwc1(at + 12));
                            let dy = ps2float::sub(lwc1(a0 + 52), lwc1(at + 16));
                            let dz = ps2float::sub(lwc1(a0 + 56), lwc1(at + 20));
                            if ps2float::mul(range, range) < ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) {
                                found = true;
                                break;
                            }
                        }
                    }
                    n += 1;
                    entry += 4;
                    if n >= count {
                        break;
                    }
                }
                if found {
                    let flags = lw(mind + 2704);
                    sw(mind + 2728, 10);
                    sw(mind + 2704, flags & -3);
                }
            }
            if lw(mind + 2728) == 10 {
                return stay(mind);
            }
            if 0.5_f32 < random_unit() {
                return go_to(mind, 14);
            }
            go_to(mind, 16)
        }
        10 => {
            enemySetMoveMode(a0, 1);
            swc1(mind + 2872, lwc1(mind + 2856));
            let dx = ps2float::sub(lwc1(a0 + 48), lwc1(mind + 2796));
            let dy = ps2float::sub(lwc1(a0 + 52), lwc1(mind + 2800));
            let dz = ps2float::sub(lwc1(a0 + 56), lwc1(mind + 2804));
            if !(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) < lwc1(0x3ad8fc)) {
                return stay(mind);
            }
            let flags = lw(mind + 2704);
            sw(mind + 2728, 11);
            let has_target = lw(mind + 2772);
            sw(mind + 2704, flags | 6);
            finish_with_flag(mind, has_target)
        }
        11 => {
            if lwc1(weapon + (g::firerange + 4)) < dist {
                return go_to(mind, 1);
            }
            go_to(mind, 2)
        }
        12 => {
            if random_unit() < lwc1(0x3ad900) {
                if !(0.5_f32 < random_unit()) {
                    return go_to(mind, 16);
                }
                if lwc1(mind + 2832) < 0.5_f32 {
                    return go_to(mind, 14);
                }
                return go_to(mind, 16);
            }
            let target = lw(mind + 2788);
            if lw(target + 8) == 8 {
                let data = lw(target + 352);
                if lw(data + 2716) == 2 {
                    if lw(data + 2728) == 13 {
                        return go_to(mind, 3);
                    }
                }
            }
            let dy = ps2float::sub(lwc1(a0 + 52), lwc1(target + 52));
            let dx = ps2float::sub(lwc1(a0 + 48), lwc1(target + 48));
            let dz = ps2float::sub(lwc1(a0 + 56), lwc1(target + 56));
            if dpcmp(dp_abs(dy), ld(0x3a9de0)) > 0 {
                return go_to(mind, 3);
            }
            let weapon_now = lw(mind + 376);
            let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
            let range = weapon_now * 12 + g::firerange;
            let player = lw(g::currentplayer);
            let reach = lwc1(0x3ad904);
            let index = (mind - lw(g::chrinfo)).wrapping_mul(-0x795b3d1f);
            swc1(mind + 2820, ps2float::add(ps2float::mul(ps2float::mul(dx, inverse), ps2float::add(lwc1(range), 2.0_f32)), lwc1(target + 48)));
            swc1(mind + 2824, ps2float::add(ps2float::mul(ps2float::mul(dy, inverse), ps2float::add(lwc1(range), 2.0_f32)), lwc1(target + 52)));
            swc1(mind + 2828, ps2float::add(ps2float::mul(ps2float::mul(dz, inverse), ps2float::add(lwc1(range), 2.0_f32)), lwc1(target + 56)));
            moveTest(index >> 4, a0 + 48, mind + 2820, 0, 0, 0, reach, lwc1(player + 132));
            let travel = squared_distance(a0 + 48, mind + 2820);
            if lwc1(0x3ad908) < travel {
                sw(mind + 2728, 13);
                sw(mind + 2892, 0);
                swc1(mind + 2732, travel.abs().psqrt());
                return stay(mind);
            }
            go_to(mind, 3)
        }
        13 => {
            if enemyReactTime(a0) < lwc1(mind + 2892) {
                sw(mind + 2704, lw(mind + 2704) & -3);
            }
            swc1(mind + 2872, lwc1(mind + 2860));
            enemySetMoveMode(a0, 128);
            let z = lwc1(mind + 2828);
            let x = lwc1(mind + 2820);
            let y = lwc1(mind + 2824);
            swc1(mind + 2804, z);
            swc1(mind + 2796, x);
            swc1(mind + 2800, y);
            let dz = ps2float::sub(lwc1(a0 + 56), z);
            let dx = ps2float::sub(lwc1(a0 + 48), x);
            if ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dz, dz)) < lwc1(0x3ad90c) {
                return go_to(mind, 1);
            }
            if lw(mind + 0x1194) != 0 {
                return go_to(mind, 1);
            }
            if dpcmp(dp_abs(lwc1(mind + 2780)), 0x4034000000000000_i64) <= 0 {
                return stay(mind);
            }
            go_to(mind, 1)
        }
        14 => {
            let model = lw(lw(lw(a0 + 32) + 88) + 12);
            if model != 0 {
                extractAnimRootData(a0, 611, sp + 416, sp + 420, sp + 424, 0);
            } else {
                extractAnimRootData(a0, 608, sp + 416, sp + 420, sp + 424, 0);
            }
            let heading = ps2float::mul(lwc1(a0 + 80), lwc1(0x3ad910)).pdiv(1.8e+02_f32);
            sw(sp + 4, 0);
            sw(sp + 8, 0);
            swc1(sp, ps2float::sub(lwc1(sp + 420), lwc1(sp + 416)));
            let hit = heading_move(a0, mind, sp, sp + 96, heading, lwc1(0x3ad914));
            if (hit & 2) != 0 {
                return go_to(mind, 3);
            }
            let turn = anglediff(lwc1(a0 + 76), lwc1(a0 + 80));
            if dpcmp(dp_abs(turn), 0x402e000000000000_i64) >= 0 {
                return go_to(mind, 3);
            }
            let flags = lw(mind + 2704) & -3;
            sw(mind + 2728, 15);
            sw(mind + 2704, flags);
            stay(mind)
        }
        15 => {
            if !playing_roll(a0) {
                let state_data = lw(a0 + 32);
                let heading = lwc1(a0 + 80);
                sw(mind + 2708, lw(mind + 2708) | 256);
                let model = lw(state_data + 88);
                swc1(a0 + 76, heading);
                if lw(model + 12) != 0 {
                    setAnim(a0, 612, 0.0_f32, 1.0_f32);
                } else {
                    setAnim(a0, 609, 0.0_f32, 1.0_f32);
                }
                return stay(mind);
            }
            if curAnimComplete(lw(a0 + 32)) == 0 {
                return stay(mind);
            }
            let flags = lw(mind + 2708);
            sw(mind + 2728, 3);
            sw(mind + 2892, 0);
            sw(mind + 2708, flags & -257);
            stay(mind)
        }
        16 => {
            extractAnimRootData(a0, 447, sp + 428, sp + 432, sp + 436, 0);
            let heading = ps2float::mul(lwc1(a0 + 80), lwc1(0x3ad918)).pdiv(1.8e+02_f32);
            sw(sp + 4, 0);
            sw(sp + 8, 0);
            swc1(sp, ps2float::sub(lwc1(sp + 432), lwc1(sp + 428)));
            let hit = heading_move(a0, mind, sp, sp + 160, heading, lwc1(0x3ad91c));
            if (hit & 2) != 0 {
                return go_to(mind, 3);
            }
            let turn = anglediff(lwc1(a0 + 76), lwc1(a0 + 80));
            if dpcmp(dp_abs(turn), 0x402e000000000000_i64) >= 0 {
                return go_to(mind, 3);
            }
            go_to(mind, 17)
        }
        17 => {
            if curAnim(a0, 447) == 0 && curAnim(a0, 448) == 0 {
                let gun = lw(mind + 376) * 400;
                let heading = lwc1(a0 + 80);
                sw(mind + 2708, lw(mind + 2708) | 256);
                swc1(a0 + 76, heading);
                if (lw(gun + (GUNS_TABLE + 28)) & 2) == 0 {
                    setAnim(a0, 448, 0.0_f32, 1.0_f32);
                } else {
                    setAnim(a0, 447, 0.0_f32, 1.0_f32);
                }
                return stay(mind);
            }
            if curAnimComplete(lw(a0 + 32)) == 0 {
                return stay(mind);
            }
            let flags = lw(mind + 2704);
            let flags2 = lw(mind + 2708) & -257;
            sw(mind + 2728, 5);
            sw(mind + 2892, 0);
            sw(mind + 2708, flags2);
            sw(mind + 2704, flags | 6);
            enemySetMoveMode(a0, 4);
            stay(mind)
        }
        _ => melee_attack(a0, mind, sp, dist),
    }
}

/// Whether any of the roll animations (608 to 613) plays.
fn playing_roll(a0: i32) -> bool {
    for anim in [608, 609, 610, 611, 612, 613] {
        if curAnim(a0, anim) != 0 {
            return true;
        }
    }
    false
}

/// State 18: a character fighting in melee. Without a gun it makes a swipe (animations 526 to 529) and hurts whatever the hand sweeps through; with a gun it comes close
/// and strikes after a short time.
fn melee_attack(a0: i32, mind: i32, sp: i32, dist: f32) -> Flow {
    let target = lw(mind + 2788);
    swc1(mind + 2872, lwc1(mind + 2856));
    let armed = lw(mind + 440);
    aim_at_target(mind, target);
    if armed == 0 {
        if (curAnim(a0, 526) != 0 || curAnim(a0, 527) != 0 || curAnim(a0, 528) != 0 || curAnim(a0, 529) != 0) && (lw(a0 + 16) & 128) == 0 {
            let mut hit = 0;
            matrixTransRotY(sp + 224, lwc1(a0 + 48), lwc1(a0 + 52), lwc1(a0 + 56), ps2float::mul(lwc1(a0 + 76), lwc1(0x3ad920)).pdiv(1.8e+02_f32));
            let state_data = lw(a0 + 32);
            sceVu0MulMatrix(sp + 288, sp + 224, lw(state_data + 4) + (lw(lw(state_data + 92) + 20) << 6));
            let f1_2 = lwc1(sp + 340);
            let f0_5 = lwc1(sp + 344);
            swc1(sp + 352, lwc1(sp + 336));
            swc1(sp + 356, f1_2);
            swc1(sp + 360, f0_5);
            let bone;
            if curAnim(a0, 527) != 0 {
                bone = 24;
            } else {
                bone = if curAnim(a0, 528) == 0 { 28 } else { 24 };
            }
            let state_data = lw(a0 + 32);
            sceVu0MulMatrix(sp + 288, sp + 224, lw(state_data + 4) + (lw(lw(state_data + 92) + bone) << 6));
            let f4_2 = lwc1(sp + 352);
            let f5_2 = lwc1(sp + 356);
            let f6_2 = lwc1(sp + 360);
            let f5_3 = ps2float::add(f5_2, ps2float::mul(ps2float::sub(lwc1(sp + 340), f5_2), 1.5_f32));
            let f6_3 = ps2float::add(f6_2, ps2float::mul(ps2float::sub(lwc1(sp + 344), f6_2), 1.5_f32));
            swc1(sp + 368, ps2float::add(f4_2, ps2float::mul(ps2float::sub(lwc1(sp + 336), f4_2), 1.5_f32)));
            swc1(sp + 372, f5_3);
            swc1(sp + 376, f6_3);
            let first_frame = lwc1(mind + 2920) == 0.0_f32 && lwc1(mind + 2924) == 0.0_f32 && lwc1(mind + 2928) == 0.0_f32;
            if !first_frame {
                hit = propSwipeTest(a0, sp + 352, mind + 2920, sp + 368, sp + 384, 0);
            }
            if hit != 0 {
                let dx = ps2float::sub(lwc1(sp + 368), lwc1(sp + 352));
                let dz = lwc1(sp + 376);
                let dy = ps2float::sub(lwc1(sp + 372), lwc1(sp + 356));
                let base_z = lwc1(sp + 360);
                swc1(sp + 400, dx);
                let dz = ps2float::sub(dz, base_z);
                swc1(sp + 404, dy);
                swc1(sp + 408, dz);
                let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
                let nx = ps2float::mul(lwc1(sp + 400), inverse);
                let ny = ps2float::mul(lwc1(sp + 404), inverse);
                swc1(sp + 408, ps2float::mul(lwc1(sp + 408), inverse));
                swc1(sp + 400, nx);
                swc1(sp + 404, ny);
                propDamage(hit, a0, sp + 384, 1, sp + 368, sp + 400, 0, 0, lwc1(0x3ad924));
            }
            let has_target = lw(mind + 2772);
            swc1(mind + 2920, lwc1(sp + 368));
            swc1(mind + 2924, lwc1(sp + 372));
            swc1(mind + 2928, lwc1(sp + 376));
            return finish_with_flag(mind, has_target);
        }
        if dist < 2.5_f32 {
            if dpcmp(dp_abs(lwc1(mind + 2780)), 0x4034000000000000_i64) < 0 {
                enemySetMoveMode(a0, 8);
                if frames_gate() > 0.0_f32 {
                    let pick = newrnd();
                    soundStart((pick & 1_i64) as i32 + 171, a0 + 48);
                }
                let roll = random_unit();
                if !(roll < 0.25_f32) {
                    if !(roll < 0.5_f32) {
                        setAnim(a0, 529, 0.0_f32, 1.0_f32);
                    } else {
                        setAnim(a0, 527, 0.0_f32, 1.0_f32);
                    }
                } else {
                    setAnim(a0, 526, 0.0_f32, 1.0_f32);
                }
                return stay(mind);
            }
        }
        enemySetMoveMode(a0, 1);
        return stay(mind);
    }
    if !(5.0_f32 < dist) {
        enemySetMoveMode(a0, 8);
    } else {
        enemySetMoveMode(a0, 1);
    }
    let timer = lwc1(mind + 2892);
    let has_target = lw(mind + 2772);
    if !(2.0_f32 < timer) {
        return finish_with_flag(mind, has_target);
    }
    if has_target == 0 {
        return finish_with_state(mind, lw(mind + 2728));
    }
    let flags = lw(mind + 2704);
    sw(mind + 2892, 0);
    sw(mind + 2704, flags | 2);
    finish_with_flag(mind, has_target)
}

/// The part of the attack that only goes on while the character stands on a pad next to its target: it decides whether to move on to another pad.
fn pad_attack(a0: i32, mind: i32) -> Flow {
    let target = lw(mind + 2788);
    sw(mind + 2728, 0);
    let height = dp_abs(ps2float::sub(lwc1(a0 + 52), lwc1(target + 52)));
    if dpcmp(height, 0x3ff8000000000000_i64) <= 0 {
        let wanted = enemyGetTargetPad(a0);
        sw(mind + 684, wanted);
        let next = findNextPad(mind + 680);
        if next == -2 {
            return Flow::Dead;
        }
        if next != -1 {
            enemySetMoveMode(a0, 1);
            let pads = lw(g::levelpads);
            swc1(mind + 2872, lwc1(mind + 2860));
            let limit = lwc1(0x3ad8e8);
            let at = next * 28 + pads;
            swc1(mind + 2796, lwc1(at + 12));
            swc1(mind + 2800, lwc1(at + 16));
            swc1(mind + 2804, lwc1(at + 20));
            let dx = ps2float::sub(lwc1(a0 + 48), lwc1(at + 12));
            let dy = ps2float::sub(lwc1(a0 + 52), lwc1(at + 16));
            let dz = ps2float::sub(lwc1(a0 + 56), lwc1(at + 20));
            if !(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) < limit) {
                return check_target(mind, lw(mind + 2788));
            }
            let left = lw(mind + 1688);
            sw(mind + 680, next);
            sw(mind + 1688, left - 1);
            enemySetMoveMode(a0, 8);
            let target_pad = lw(mind + 684);
            sw(mind + 2704, lw(mind + 2704) | 6);
            if target_pad != wanted {
                sw(mind + 1688, 0);
                sw(mind + 2692, 0);
            }
            return check_target(mind, lw(mind + 2788));
        }
        if ps2float::add(ps2float::add(lwc1(mind + 2832), 0.5_f32), 3.0_f32) < lwc1(mind + 2912) {
            sw(mind + 2704, lw(mind + 2704) | 6);
        }
        enemySetMoveMode(a0, 8);
        return Flow::Return;
    }
    enemySetMoveMode(a0, 8);
    if !(ps2float::mul(ps2float::add(lwc1(mind + 2832), 0.5_f32), 3.0_f32) < lwc1(mind + 2912)) {
        return check_target(mind, lw(mind + 2788));
    }
    if lw(mind + 2772) == 0 {
        return check_target(mind, lw(mind + 2788));
    }
    sw(mind + 2704, lw(mind + 2704) | 6);
    check_target(mind, lw(mind + 2788))
}

/// The attack of the character `a0` (@ 0x2c18d0): a character attacking a prop or an object calls its own attack; otherwise this notes where the target is and how far, decides
/// whether the character has spotted it (the flag 2 of +2704), and runs the state machine of the attack (the state at +2728).
pub fn attack(a0: i32) {
    let frame = Frame::new(624);
    let sp = frame.sp();
    let mind = lw(a0 + 352);
    let target = lw(mind + 2788);
    if lw(target + 8) != 8 {
        enemyAttackObject(a0);
        return;
    }
    swc1(mind + 2796, lwc1(target + 48));
    swc1(mind + 2800, lwc1(target + 52));
    swc1(mind + 2804, lwc1(target + 56));
    let dx = ps2float::sub(lwc1(a0 + 48), lwc1(target + 48));
    let dy = ps2float::sub(lwc1(a0 + 52), lwc1(target + 52));
    let dz = ps2float::sub(lwc1(a0 + 56), lwc1(target + 56));
    let dist = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
    if lw(mind + 2772) != 0 {
        let kind = lw(mind + 8);
        if kind == 0 || kind == 1 {
            if enemyReactTime(a0) < lwc1(mind + 2912) {
                let flags = lw(mind + 2704);
                sw(mind + 2704, flags | 2);
            }
        }
    }
    let pad = lw(mind + 680);
    let flow;
    if (lw(pad * 28 + lw(g::levelpads) + 4) & 2) != 0 && lw(mind + 2772) != 0 {
        flow = pad_attack(a0, mind);
    } else {
        updateNPCPadPos(a0);
        let kind = lw(mind + 8);
        let state = lw(mind + 2728);
        if kind == 2 && state != 8 {
            enemyTimeSplitterAttack(a0);
            sw(mind + 2892, 0);
            flow = stay(mind);
        } else if (state as u32) < (19 as u32) {
            flow = attack_state(a0, mind, sp, kind, state, dist);
        } else {
            flow = stay(mind);
        }
    }
    match flow {
        Flow::Return => {}
        Flow::Dead => {
            let saved = lw(mind + 2716);
            sw(mind + 2716, 128);
            sw(mind + 2720, saved);
            sw(mind + 2892, 0);
        }
        Flow::Reset => {
            sw(mind + 2892, 0);
        }
    }
}
