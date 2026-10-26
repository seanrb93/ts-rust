//! Functions: the ambush states of a character waiting behind a wall (left and right), the ambush state of a crouching character and the handler of the waves of Last Stand.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// Turn the character `a0` (angles at +76, +80, +84) half a turn and bring them to the range of one turn.
fn ambush_half_turn(a0: i32) {
    let turn = ps2float::add(lwc1(a0 + 76), 1.8e+02_f32);
    let other = ps2float::add(lwc1(a0 + 80), 1.8e+02_f32);
    swc1(a0 + 76, turn);
    swc1(a0 + 80, other);
    swc1(a0 + 84, turn);
    if 3.6e+02_f32 < turn {
        swc1(a0 + 76, ps2float::sub(turn, 3.6e+02_f32));
    }
    let other = lwc1(a0 + 80);
    if 3.6e+02_f32 < other {
        swc1(a0 + 80, ps2float::sub(other, 3.6e+02_f32));
    }
}

/// The character `a0` (with its mind `mind`) stops waiting behind the wall: it turns, runs towards its target and looks at it.
fn ambush_come_out(a0: i32, mind: i32) {
    setAnim(a0, 3, 0.0_f32, 1.0_f32);
    let flags = lw(mind + 2704);
    let mode = lw(mind + 2716);
    swc1(mind + 2892, 0.0_f32);
    sw(mind + 2720, mode);
    sw(mind + 2716, 2);
    sw(mind + 2704, flags | 6);
    enemySetMoveMode(a0, 1);
    let target = lw(mind + 2788);
    swc1(mind + 2796, lwc1(target + 48));
    swc1(mind + 2800, lwc1(target + 52));
    swc1(mind + 2804, lwc1(target + 56));
    rotateChr(a0);
}

/// The shared part of the left and the right wall ambush. `idle` is the animation of waiting, `rise` the animation of the rise out of the wall (state 3), `constants` the three
/// constants of the side (the divisor of the angle, the start of the window of the shot and the chance of staying) and `leaves` is true for the left side, which stops at once when
/// the character has gone back to its pad.
fn ambush_wall(a0: i32, idle: i32, rise: i32, constants: [i32; 3], leaves: bool) {
    let mind = lw(a0 + 352);
    enemySetMoveMode(a0, 8);
    let state = lw(mind + 2728);
    swc1(mind + 2796, lwc1(a0 + 48));
    swc1(mind + 2800, lwc1(a0 + 52));
    let z = lwc1(a0 + 56);
    sw(mind + 2780, 0);
    swc1(mind + 2804, z);
    let pad_number;
    let at;
    if (state as u32) < (5 as u32) {
        match state {
            0 => {
                if curAnim(a0, idle) == 0 {
                    let height = lwc1(a0 + 52);
                    let angle = lwc1(lw(mind + 680) * 28 + lw(g::levelpads) + 24);
                    swc1(a0 + 84, angle);
                    swc1(mind + 2972, angle);
                    swc1(a0 + 76, angle);
                    swc1(a0 + 80, angle);
                    setAnim(a0, idle, 0.0_f32, 0.0_f32);
                    let x = lwc1(a0 + 48);
                    swc1(a0 + 52, height);
                    swc1(mind + 2820, x);
                    swc1(mind + 2824, lwc1(a0 + 52));
                    swc1(mind + 2828, lwc1(a0 + 56));
                }
                if enemyTriggered(a0) != 0 {
                    sw(mind + 2728, 1);
                }
                pad_number = lw(mind + 680);
                at = lw(mind + 2788) + 48;
            }
            1 => {
                let target = lw(mind + 2788);
                let difference = ps2float::sub(lwc1(a0 + 52), lwc1(target + 52));
                let mut height = fptodp(difference);
                if dpcmp(height, 0_i64) < 0 {
                    height = dpsub(0_i64, height);
                }
                if dpcmp(height, 0x4000000000000000_i64) < 0 {
                    setAnim(a0, idle, 0.0_f32, 1.0_f32);
                    let pads = lw(g::levelpads);
                    let pad_offset = lw(mind + 680) * 28;
                    let target = lw(mind + 2788);
                    sw(mind + 2728, 2);
                    let pad = pad_offset + pads;
                    let z = ps2float::sub(lwc1(pad + 20), lwc1(target + 56));
                    let x = ps2float::sub(lwc1(pad + 12), lwc1(target + 48));
                    let mut angle = ps2float::mul(atan2f(x, z), 1.8e+02_f32).pdiv(lwc1(constants[0]));
                    while angle < 0.0_f32 {
                        angle = ps2float::add(angle, 3.6e+02_f32);
                    }
                    while 3.6e+02_f32 <= angle {
                        angle = ps2float::sub(angle, 3.6e+02_f32);
                    }
                    swc1(a0 + 84, angle);
                    swc1(mind + 2972, angle);
                    swc1(a0 + 76, angle);
                    swc1(a0 + 80, angle);
                    return;
                }
                pad_number = lw(mind + 680);
                at = target + 48;
            }
            2 => {
                let progress = lwc1(lw(a0 + 32) + 100);
                if lwc1(constants[1]) < progress {
                    if progress < 1.0_f32 {
                        let flags = lw(mind + 2704);
                        if (flags & 2) == 0 {
                            sw(mind + 2704, flags | 6);
                        }
                    }
                }
                if curAnimComplete(lw(a0 + 32)) == 0 {
                    return;
                }
                let target = lw(mind + 2788);
                let dx = ps2float::sub(lwc1(a0 + 48), lwc1(target + 48));
                let dy = ps2float::sub(lwc1(a0 + 52), lwc1(target + 52));
                let dz = ps2float::sub(lwc1(a0 + 56), lwc1(target + 56));
                if checkInFrontOfPad(lw(mind + 680), target + 48, 0.0_f32) == 0 {
                    let random = random_unit();
                    if !(random < lwc1(constants[2])) {
                        if !(6.0_f32 < ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt()) {
                            ambush_half_turn(a0);
                            ambush_come_out(a0, mind);
                            return;
                        }
                    }
                    sw(mind + 2728, 3);
                    return;
                }
                ambush_half_turn(a0);
                ambush_come_out(a0, mind);
                return;
            }
            3 => {
                if (lw(mind + 2704) & 2) != 0 {
                    if curAnim(a0, 456) != 0 {
                        return;
                    }
                    if checkInFrontOfPad(lw(mind + 680), lw(mind + 2788) + 48, 0.0_f32) == 0 {
                        return;
                    }
                    ambush_half_turn(a0);
                    ambush_come_out(a0, mind);
                    return;
                }
                setAnim(a0, rise, 0.0_f32, 1.0_f32);
                swc1(mind + 2892, 0.0_f32);
                let random = random_unit();
                let wait;
                if !(0.25_f32 < random) {
                    let random = random_unit();
                    wait = ps2float::add(ps2float::add(random, random), 2.0_f32);
                } else {
                    let random = random_unit();
                    wait = ps2float::add(random, 1.0_f32);
                }
                swc1(mind + 2732, wait);
                sw(mind + 2728, 4);
                return;
            }
            _ => {
                // state 4: waiting for the time to end, then back to the pad
                if lwc1(mind + 2732) < lwc1(mind + 2892) {
                    let complete = curAnimComplete(lw(a0 + 32));
                    let frames = running_x();
                    if complete != 0 {
                        if frames > 0 {
                            soundStart(138, 0);
                        }
                        let pads = lw(g::levelpads);
                        let pad_offset = lw(mind + 680) * 28;
                        sw(mind + 2728, 1);
                        let pad = pad_offset + pads;
                        swc1(a0 + 48, lwc1(pad + 12));
                        swc1(a0 + 52, lwc1(pad + 16));
                        swc1(a0 + 56, lwc1(pad + 20));
                        if leaves {
                            return;
                        }
                    }
                }
                if curAnimComplete(lw(a0 + 32)) == 0 {
                    return;
                }
                let pad_number = lw(mind + 680);
                let at = lw(mind + 2788) + 48;
                let pad = pad_number * 28 + lw(g::levelpads);
                swc1(a0 + 48, lwc1(pad + 12));
                swc1(a0 + 52, lwc1(pad + 16));
                swc1(a0 + 56, lwc1(pad + 20));
                if checkInFrontOfPad(pad_number, at, 0.0_f32) == 0 {
                    return;
                }
                ambush_come_out(a0, mind);
                return;
            }
        }
    } else {
        return;
    }
    if checkInFrontOfPad(pad_number, at, 0.0_f32) != 0 {
        setAnim(a0, 3, 0.0_f32, 1.0_f32);
        let mode = lw(mind + 2716);
        sw(mind + 2716, 2);
        swc1(mind + 2892, 0.0_f32);
        sw(mind + 2720, mode);
    }
}

/// The ambush state of a character waiting behind a wall on its left (@ 0x2c9248).
pub fn enemyAmbushWallLeft(a0: i32) {
    ambush_wall(a0, 455, 456, [0x3ad9ec, 0x3ad9f0, 0x3ad9f4], true);
}

/// The ambush state of a character waiting behind a wall on its right (@ 0x2c8a28).
pub fn enemyAmbushWallRight(a0: i32) {
    ambush_wall(a0, 457, 458, [0x3ad9e0, 0x3ad9e4, 0x3ad9e8], false);
}

/// The ambush state of a crouching character `a0` (@ 0x2c9b68): it keeps its place on its pad, waits, shoots, throws a grenade or stands, by the state at +2728 (0 to 6).
pub fn enemyAmbushCrouch(a0: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mind = lw(a0 + 352);
    let alone = (lw(g::lvnumplayers) < 2) as i32;
    let pad = lw(mind + 680) * 28 + lw(g::levelpads);
    swc1(a0 + 48, lwc1(pad + 12));
    swc1(a0 + 56, lwc1(pad + 20));
    if alone == 0 {
        sw(mind + 2788, checkCoopTarget(a0));
    }
    let mut target = lw(mind + 2788);
    let target_x = lwc1(target + 48);
    let dx = ps2float::sub(target_x, lwc1(a0 + 48));
    let dy = ps2float::sub(lwc1(target + 52), lwc1(a0 + 52));
    let dz = ps2float::sub(lwc1(target + 56), lwc1(a0 + 56));
    let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
    if lw(mind + 2772) != 0 {
        if (lw(mind + 2704) & 0x4000) != 0 {
            swc1(mind + 2796, target_x);
            swc1(mind + 2800, lwc1(target + 52));
            swc1(mind + 2804, lwc1(target + 56));
        }
    }
    let state = lw(mind + 2728);
    'tail: {
        if (state as u32) < (7 as u32) {
            // the wait to set when the state ends without an animation: `timer_base + timer_random`
            let timer_random;
            let timer_base;
            'wait: {
                'state_set: {
                    'leave: {
                        let anim;
                        let blend;
                        let extra;
                        match state {
                            0 => {
                                enemySetMoveMode(a0, 4);
                                sw(mind + 2728, 1);
                                let random = random_unit();
                                sw(mind + 2892, 0);
                                target = lw(mind + 2788);
                                swc1(mind + 2732, ps2float::add(ps2float::add(random, random), 0.5_f32));
                                swc1(mind + 2820, lwc1(a0 + 80));
                                break 'tail;
                            }
                            1 => {
                                enemySetMoveMode(a0, 4);
                                if !(lwc1(mind + 2732) < lwc1(mind + 2892)) {
                                    target = lw(mind + 2788);
                                    break 'tail;
                                }
                                if enemyFiredAt(a0) != 0 {
                                    target = lw(mind + 2788);
                                    break 'tail;
                                }
                                sw(mind + 2728, 2);
                                let random = random_unit();
                                let range = lwc1(0x3ad9f8);
                                sw(mind + 2892, 0);
                                timer_base = lwc1(0x3ad9fc);
                                target = lw(mind + 2788);
                                timer_random = ps2float::mul(random, range);
                                break 'wait;
                            }
                            2 => {
                                sw(mind + 2708, lw(mind + 2708) & -5);
                                enemySetMoveMode(a0, 8);
                                if !(lwc1(mind + 2732) < lwc1(mind + 2892)) {
                                    target = lw(mind + 2788);
                                    break 'tail;
                                }
                                let sees;
                                if (lw(mind + 2704) & 1024) != 0 {
                                    sees = lw(mind + 2772);
                                } else if !(1e+02_f32 < distance) {
                                    sees = lw(mind + 2772);
                                } else {
                                    let random = random_unit();
                                    if random < lwc1(0x3ada00) {
                                        target = lw(mind + 2788);
                                        sw(mind + 2728, 3);
                                        break 'tail;
                                    }
                                    sees = lw(mind + 2772);
                                }
                                if sees == 0 {
                                    break 'leave;
                                }
                                let flags = lw(mind + 2704);
                                sw(mind + 2728, 5);
                                sw(mind + 2704, flags | 6);
                                let random = random_unit();
                                let range = lwc1(0x3ada04);
                                sw(mind + 2892, 0);
                                timer_base = lwc1(0x3ada08);
                                target = lw(mind + 2788);
                                timer_random = ps2float::mul(random, range);
                                break 'wait;
                            }
                            3 => {
                                let word = lw(0x3a9ee0);
                                sd(sp, ld(0x3a9ed8));
                                sw(sp + 8, word);
                                enemySetMoveMode(a0, 512);
                                if curAnim(a0, 459) != 0 {
                                    if curAnimComplete(lw(a0 + 32)) != 0 {
                                        target = lw(mind + 2788);
                                        break 'tail;
                                    }
                                    let x = lwc1(sp);
                                    if !(lwc1(0x3ada10) < lwc1(lw(a0 + 32) + 100)) {
                                        break 'leave;
                                    }
                                    let y = lwc1(sp + 4);
                                    let z = lwc1(sp + 8);
                                    let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
                                    let angle = ps2float::mul(lwc1(a0 + 80), lwc1(0x3ada14)).pdiv(1.8e+02_f32);
                                    swc1(sp + 8, ps2float::mul(z, scale));
                                    swc1(sp, ps2float::mul(x, scale));
                                    swc1(sp + 4, ps2float::mul(y, scale));
                                    matrixRotY(sp + 16, angle);
                                    matrixVecRotAligned(&[(sp + 16) as i64, sp as i64]);
                                    npcThrowGrenade(a0, sp, lw(mind + 2788));
                                    break 'state_set;
                                }
                                anim = 459;
                                blend = 0.25_f32;
                                extra = lwc1(0x3ada0c);
                            }
                            4 => {
                                if curAnimComplete(lw(a0 + 32)) == 0 {
                                    break 'leave;
                                }
                                sw(mind + 2728, 1);
                                let random = random_unit();
                                timer_base = 3.0_f32;
                                target = lw(mind + 2788);
                                timer_random = ps2float::mul(random, timer_base);
                                break 'wait;
                            }
                            5 => {
                                'settled: {
                                    if (lw(mind + 2704) & 2) == 0 {
                                        if lwc1(mind + 2732) < lwc1(mind + 2892) {
                                            let random = random_unit();
                                            if dpcmp(fptodp(random), ld(0x3a9f10)) >= 0 {
                                                if dpcmp(fptodp(random), ld(0x3a9f18)) < 0 {
                                                    if (lw(mind + 2704) & 0x8000) == 0 {
                                                        sw(mind + 2728, 6);
                                                        break 'settled;
                                                    }
                                                }
                                                enemySetMoveMode(a0, 4);
                                                sw(mind + 2728, 1);
                                                let random = random_unit();
                                                sw(mind + 2892, 0);
                                                swc1(mind + 2732, ps2float::add(ps2float::add(random, random), 0.5_f32));
                                                break 'settled;
                                            }
                                            let wait = lwc1(0x3ada18);
                                            sw(mind + 2728, 2);
                                            swc1(mind + 2732, wait);
                                            sw(mind + 2892, 0);
                                            break 'settled;
                                        }
                                    }
                                    enemySetMoveMode(a0, 8);
                                }
                                if lw(mind + 2772) != 0 {
                                    target = lw(mind + 2788);
                                    break 'tail;
                                }
                                enemySetMoveMode(a0, 4);
                                sw(mind + 2728, 1);
                                let random = random_unit();
                                timer_base = 0.5_f32;
                                let flags = lw(mind + 2704);
                                sw(mind + 2892, 0);
                                target = lw(mind + 2788);
                                timer_random = ps2float::add(random, random);
                                sw(mind + 2704, flags & -3);
                                break 'wait;
                            }
                            _ => {
                                // state 6: the standing shot, by the animation
                                if curAnim(a0, 477) == 0 {
                                    if curAnim(a0, 478) == 0 {
                                        enemySetMoveMode(a0, 512);
                                        if (lw(lw(mind + 376) * 400 + (g::guns + 28)) & 2) == 0 {
                                            blend = lwc1(0x3ada20);
                                            extra = 1.0_f32;
                                            anim = 477;
                                        } else {
                                            blend = lwc1(0x3ada1c);
                                            extra = 1.0_f32;
                                            anim = 478;
                                        }
                                        setAnim(a0, anim, blend, extra);
                                        break 'leave;
                                    }
                                }
                                if curAnim(a0, 477) == 0 {
                                    if curAnim(a0, 478) == 0 {
                                        break 'leave;
                                    }
                                }
                                if curAnim(a0, 477) != 0 {
                                    let progress = lwc1(lw(a0 + 32) + 100);
                                    if lwc1(0x3ada24) < progress {
                                        if progress < lwc1(0x3ada28) {
                                            let flags = lw(mind + 2704);
                                            if (flags & 2) == 0 {
                                                sw(mind + 2704, flags | 6);
                                            }
                                        }
                                    }
                                }
                                if curAnim(a0, 478) != 0 {
                                    let progress = lwc1(lw(a0 + 32) + 100);
                                    if 0.5_f32 < progress {
                                        if progress < 1.5_f32 {
                                            let flags = lw(mind + 2704);
                                            if (flags & 2) == 0 {
                                                sw(mind + 2704, flags | 6);
                                            }
                                        }
                                    }
                                }
                                if curAnimComplete(lw(a0 + 32)) == 0 {
                                    target = lw(mind + 2788);
                                    break 'tail;
                                }
                                let random = random_unit();
                                let scale = lwc1(0x3ada2c);
                                sw(mind + 2892, 0);
                                let base = lwc1(0x3ada30);
                                sw(mind + 2728, 2);
                                swc1(mind + 2732, ps2float::add(ps2float::mul(random, scale), base));
                                anim = 113;
                                blend = 0.25_f32;
                                extra = 1.0_f32;
                            }
                        }
                        setAnim(a0, anim, blend, extra);
                        break 'leave;
                    }
                    target = lw(mind + 2788);
                    break 'tail;
                }
                // a grenade has been thrown: wait in state 4
                target = lw(mind + 2788);
                sw(mind + 2728, 4);
                break 'tail;
            }
            swc1(mind + 2732, timer_random + timer_base);
        }
    }
    let angle_back = lwc1(mind + 2820);
    swc1(mind + 2824, lwc1(a0 + 80));
    swc1(a0 + 80, angle_back);
    let threshold = lwc1(mind + 2700);
    let in_front = checkInFront(a0, target + 48, threshold);
    let restore;
    'settle: {
        if in_front == 0 {
            if lw(mind + 2772) != 0 {
                let mode = lw(mind + 2716);
                sw(mind + 2716, 8);
                sw(mind + 2720, mode);
                sw(mind + 2892, 0);
                restore = lwc1(mind + 2824);
                break 'settle;
            }
        }
        if !(distance < 2e+01_f32) {
            if lw(mind + 2728) == 6 {
                restore = lwc1(mind + 2824);
                break 'settle;
            }
            if !(distance < 65.0_f32) {
                restore = lwc1(mind + 2824);
                break 'settle;
            }
            if (lw(mind + 2704) & 0x8000) == 0 {
                sw(mind + 2728, 6);
            }
            restore = lwc1(mind + 2824);
            break 'settle;
        }
        let mode = lw(mind + 2716);
        sw(mind + 2716, 8);
        sw(mind + 2720, mode);
        sw(mind + 2892, 0);
        restore = lwc1(mind + 2824);
    }
    swc1(a0 + 80, restore);
}

/// The handler of Last Stand (@ 0x2cc958): the waves of the level are taken one after the other. A new character of the wave comes in at its time, and when a wave has been
/// beaten the score of the wave is given to the players and the next wave starts. Between waves (while the wave's time has not ended) it also brings a character of the
/// set from `lvGetRandomLastStandNPC`.
pub fn enemyAIFutureLastStandHandler() {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut score_base = 0;
    if lw(g::curgameparms + 72) != 6 {
        return;
    }
    let combos = lvGetGunComboNums();
    if lwc1(0x3b01c0) == lwc1(0x3ada7c) {
        let waves = lw(0x3b01e0);
        let time = if lw(0x3b01b0) <= 0 { 3.0_f32 } else { lwc1(waves) };
        swc1(0x3b01c0, time);
    }
    let mut frames = lwc1(g::updateframesf);
    if lwc1(0x3b01cc) == lwc1(0x3ada80) {
        let spawn = lw(0x3b01e4);
        let range = ps2float::sub(lwc1(spawn + 8), lwc1(spawn + 4));
        let random = random_unit();
        swc1(0x3b01cc, ps2float::add(ps2float::mul(range, random), lwc1(spawn + 4)));
        frames = lwc1(g::updateframesf);
    }
    let step = (frames).pdiv(6e+01_f32);
    let active = lw(g::numactivechars);
    swc1(0x3b01c4, ps2float::add(lwc1(0x3b01c4), step));
    let mut alive = 0;
    let wave_count = lw(0x3b01b0);
    if active > 0 {
        let players = lw(g::lvnumplayers);
        let chrinfo = lw(g::chrinfo);
        let mut n = 0;
        loop {
            let offset = (n + players) * 0x1210;
            n += 1;
            alive = if lw(offset + chrinfo + 2716) != 0 { alive + 1 } else { alive };
            if n >= active {
                break;
            }
        }
    }
    let limit = lw(0x3b01dc);
    'wave: {
        if wave_count > 0 {
            let time = lwc1(0x3b01c0);
            if alive < limit {
                let time_left = ps2float::sub(time, step);
                let gap = ps2float::sub(lwc1(0x3b01c8), step);
                swc1(0x3b01c0, time_left);
                swc1(0x3b01c8, gap);
                if time_left <= 0.0_f32 {
                    let mut group = lw(0x3b01b8);
                    if gap <= 0.0_f32 {
                        'next: {
                            let wave = lw(0x3b01d4);
                            if group >= 0 {
                                let entry = lvGetRandomLastStandNPC(lw(wave * 216 + lw(0x3b01e0) + lw(0x3b01b4) * 20 + 12));
                                group = lw(0x3b01b8);
                                if group == 0 {
                                    let count = lw(0x3b01bc);
                                    if lw(0x3b01b4) == 0 {
                                        let language = lw(g::cur_language);
                                        if count == 0 {
                                            let text = vafmt(&[0x3afd80, lw(lw((language << 2) + g::languages) + 428) as i64, lw(0x3b01d8) as i64]);
                                            if frames_gate() > 0.0_f32 {
                                                soundStart(220, 0);
                                            }
                                            let players = lw(g::lvnumplayers);
                                            if group < players {
                                                let mut player = 0;
                                                let mut offset = 0;
                                                loop {
                                                    if lw(offset + lw(g::players) + 296) != 4 {
                                                        hudMessage(player, text);
                                                    }
                                                    player += 1;
                                                    offset += 1820;
                                                    if player >= lw(g::lvnumplayers) {
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if entry != 0 {
                                    let slot = lw(entry + 16);
                                    let gun = if slot == -1 { -1 } else { lw((slot << 2) + combos) };
                                    lvGetChrTypeNumFromSetup(lw(entry + 8), sp, sp | 4);
                                    let chr = enemyCreateNPC(lw(entry), lw(sp), lw(sp + 4), gun, lw(entry + 12) | 0x40000, -1, -1);
                                    let wave = lw(0x3b01d4);
                                    let entry_number;
                                    let waves;
                                    if lw(g::numBases) <= 0 {
                                        let random = newrnd();
                                        let players = lw(g::lvnumplayers);
                                        let chrinfo = lw(g::chrinfo);
                                        if players == 0 {
                                            trap("break");
                                        }
                                        let record = chr * 0x1210 + chrinfo;
                                        sw(record + 2788, lw(((random as i32 as u32) % (players as u32)) as i32 * 1820 + lw(g::players) + 384));
                                        if players >= 2 {
                                            let target = checkCoopTarget(lw(record + 3020));
                                            sw(chr * 0x1210 + lw(g::chrinfo) + 2788, target);
                                        }
                                        waves = lw(0x3b01e0);
                                        entry_number = lw(0x3b01b4);
                                    } else {
                                        entry_number = lw(0x3b01b4);
                                        waves = lw(0x3b01e0);
                                        sw(chr * 0x1210 + lw(g::chrinfo) + 2788, lw((lw(wave * 216 + waves + entry_number * 20 + 16) << 3) + (g::bases + 4)));
                                    }
                                    let wave = lw(0x3b01d4);
                                    let wave_record = wave * 216 + waves;
                                    let entry_record = wave_record + entry_number * 20;
                                    let number = lw(entry_record + 8);
                                    let made = lw(0x3b01bc) + 1;
                                    sw(0x3b01bc, made);
                                    alive += 1;
                                    if made < number {
                                        group = lw(0x3b01b8);
                                        swc1(0x3b01c8, lwc1(entry_record + 20));
                                        break 'next;
                                    }
                                    let next_gap = lwc1(wave_record + entry_number * 20 + 24);
                                    let next_entry = entry_number + 1;
                                    let more = next_entry < lw(wave_record + 208);
                                    sw(0x3b01bc, 0);
                                    swc1(0x3b01c8, next_gap);
                                    sw(0x3b01b4, next_entry);
                                    if !more {
                                        let groups = lw(wave_record + 4);
                                        group = lw(0x3b01b8) + 1;
                                        sw(0x3b01b4, 0);
                                        sw(0x3b01b8, group);
                                        if group < groups {
                                            break 'next;
                                        }
                                        group = -1;
                                        sw(0x3b01b8, group);
                                        break 'next;
                                    }
                                }
                                group = lw(0x3b01b8);
                            }
                        }
                        if group != -1 || alive > 0 {
                            break 'wave;
                        }
                        // the whole wave has been beaten: give its score
                        let language;
                        let bonus;
                        let wave = lw(0x3b01d4);
                        if lw(g::numBases) <= 0 {
                            let record = lwc1(wave * 216 + lw(0x3b01e0) + 212);
                            let elapsed = lwc1(0x3b01c4);
                            language = lw(g::cur_language);
                            if 0.0_f32 < record && 0.0_f32 < elapsed {
                                let seconds = ps2float::mul((record).pdiv(elapsed), 1e+03_f32) as i32;
                                let number = lw(0x3b01d8);
                                bonus = if seconds <= 0 { 0 } else { seconds * number };
                            } else {
                                bonus = 0;
                            }
                        } else {
                            let bases = lw(g::numBases);
                            language = lw(g::cur_language);
                            let number = lw(0x3b01d8);
                            let mut n = 0;
                            loop {
                                let base = lw((n << 3) + (g::bases + 4));
                                n += 1;
                                score_base = score_base + ps2float::mul(ps2float::sub(1.0_f32, lwc1(base + 532)), 1e+03_f32) as i32;
                                if n >= bases {
                                    break;
                                }
                            }
                            bonus = score_base * number;
                        }
                        let strings = lw((language << 2) + g::languages);
                        let text = vafmt(&[
                            0x3a9fb8,
                            lw(strings + 428) as i64,
                            lw(0x3b01d8) as i64,
                            lw(strings + 432) as i64,
                            lw(strings + 436) as i64,
                            bonus as i64,
                            lw(strings + 440) as i64,
                            0x2f0000,
                        ]);
                        if frames_gate() > 0.0_f32 {
                            soundStart(220, 0);
                        }
                        let players = lw(g::lvnumplayers);
                        if players > 0 {
                            let mut player = 0;
                            let mut all = lw(g::players);
                            loop {
                                let offset = player * 1820;
                                if lw(player * 1820 + all + 296) == 4 && lw(g::numBases) <= 0 {
                                    player += 1;
                                } else {
                                    hudMessage(player, text);
                                    all = lw(g::players);
                                    let chr = lw(lw(offset + all + 384) + 352);
                                    sw(chr + 248, lw(chr + 248) + bonus);
                                    player += 1;
                                }
                                if player >= players {
                                    break;
                                }
                            }
                        }
                        let wave = lw(0x3b01d4);
                        let waves = lw(0x3b01b0);
                        let mut next = wave + 1;
                        sw(0x3b01d8, lw(0x3b01d8) + 1);
                        sw(0x3b01b8, 0);
                        sw(0x3b01d4, next);
                        if next >= waves {
                            next = wave;
                            sw(0x3b01d4, next);
                        }
                        let all_waves = lw(0x3b01e0);
                        sw(0x3b01c4, 0);
                        let first_time = lwc1(next * 216 + all_waves);
                        sw(0x3b01c8, 0);
                        swc1(0x3b01c0, first_time);
                    }
                }
            }
        }
    }
    let spawn = lw(0x3b01e4);
    let spawn_number = lw(spawn);
    let group = lw(0x3b01b8);
    if spawn_number >= 0 {
        let time = lwc1(0x3b01c0);
        if group >= 0 {
            let mut timer = lwc1(0x3b01cc);
            if !(time <= 0.0_f32) {
                if lw(0x3b01b0) > 0 {
                    return;
                }
                timer = lwc1(0x3b01cc);
            }
            timer = ps2float::sub(timer, step);
            swc1(0x3b01cc, timer);
            if timer < 0.0_f32 {
                timer = 0.0_f32;
                swc1(0x3b01cc, timer);
            }
            let limit = lw(0x3b01dc);
            if timer == 0.0_f32 {
                if alive < limit {
                    let entry = lvGetRandomLastStandNPC(spawn_number);
                    if entry != 0 {
                        let gun = lw((lw(entry + 16) << 2) + combos);
                        lvGetChrTypeNumFromSetup(lw(entry + 8), sp | 8, sp | 12);
                        let chr = enemyCreateNPC(lw(entry), lw(sp + 8), lw(sp + 12), gun, lw(entry + 12) | 0x40000, -1, -1);
                        let random = newrnd();
                        let bases = lw(g::numBases);
                        if bases == 0 {
                            trap("break");
                        }
                        let mut base = ((random as i32 as u32) % (bases as u32)) as i32;
                        sw(0x3b01d0, base);
                        let chrinfo;
                        if base >= bases {
                            let offset = (base - bases) * 1820;
                            chrinfo = lw(g::chrinfo);
                            let record = chr * 0x1210 + chrinfo;
                            let alone = (lw(g::lvnumplayers) < 2) as i32;
                            sw(record + 2788, lw(offset + lw(g::players) + 384));
                            if alone == 0 {
                                let target = checkCoopTarget(lw(record + 3020));
                                let chrinfo = lw(g::chrinfo);
                                sw(chr * 0x1210 + chrinfo + 2788, target);
                            }
                        } else {
                            chrinfo = lw(g::chrinfo);
                            if 1.0_f32 <= lwc1(lw((base << 3) + (g::bases + 4)) + 532) {
                                base += 1;
                                loop {
                                    sw(0x3b01d0, base);
                                    if base >= bases {
                                        sw(0x3b01d0, 0);
                                        base = 0;
                                    }
                                    if !(1.0_f32 <= lwc1(lw((base << 3) + (g::bases + 4)) + 532)) {
                                        break;
                                    }
                                    base += 1;
                                }
                            }
                            sw(chr * 0x1210 + chrinfo + 2788, lw((base << 3) + (g::bases + 4)));
                        }
                        let frames = running_x();
                        swc1(0x3b01cc, lwc1(0x3ada84));
                        sw(chr * 0x1210 + chrinfo + 2772, 1);
                        if frames <= 0 {
                            return;
                        }
                        soundStart(221, 0);
                        return;
                    }
                }
                swc1(0x3b01cc, lwc1(0x3ada88));
            }
        }
    }
}
