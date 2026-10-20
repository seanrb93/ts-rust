//! What an enemy does while it attacks an object.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    random_as_float() * 2.3283064e-10_f32
}

/// Where the state machine of `enemyAttackObject` goes on to.
enum Next {
    /// To the end test, with the value of the seen flag (+2772).
    Tail(i32),
    /// To the state test (go to state 8 unless there already).
    State,
    /// To the clearing of the time only.
    Clear,
}

/// The part of the states 4 and 6 once the animation is playing: the sideways drift `drift` of the target decides to go to state 7 (the time cleared) when it is far to either
/// side, or when the time is past 3; else nothing is done.
fn drift_check(enemy: i32, record: i32, drift: f32) -> Next {
    if !(5.0_f32 < drift) {
        if drift < -2e+01_f32 {
            sw(record + 2892, 0);
        } else {
            if !(3.0_f32 < lwc1(record + 2892)) {
                return Next::Tail(lw(record + 2772));
            }
            sw(record + 2892, 0);
        }
    } else {
        sw(record + 2892, 0);
    }
    sw(record + 2728, 7);
    enemySetMoveMode(enemy, 8);
    Next::Tail(lw(record + 2772))
}

/// What the enemy `enemy` does when it attacks an object (its target at +2788), by its state at +2728 of its record: state 0 to 1, state 1 to 2 when near (within 15) else it
/// goes to the target, state 2 shoots at it, state 3 stands, states 4 and 6 play an attack animation (473, 449) and then check the target's sideways drift, state 5 retreats for
/// 3.5 seconds, state 7 chooses the next state by chance, state 8 waits and ends (going to state 256 of the main record), state 9 chooses between 1 and 2; the enemy is put in
/// state 8 when nothing sees it any more (@ 0x2c3368).
pub fn enemyAttackObject(enemy: i32) {
    let record = lw(enemy + 352);
    let target = lw(record + 2788);
    let dx = lwc1(enemy + 48) - lwc1(target + 48);
    let dy = lwc1(enemy + 52) - lwc1(target + 52);
    let dz = lwc1(enemy + 56) - lwc1(target + 56);
    let distance = (dx * dx + dy * dy + dz * dz).abs().sqrt();
    'first: {
        if lw(record + 2772) != 0 {
            let kind = lw(record + 8);
            let flags;
            'choose: {
                if kind == 0 && 1.0_f32 < lwc1(record + 2912) {
                    flags = lw(record + 2704);
                    break 'choose;
                }
                if kind != 1 {
                    break 'first;
                }
                if !(2.0_f32 < lwc1(record + 2912)) {
                    break 'first;
                }
                flags = lw(record + 2704);
            }
            sw(record + 2704, flags | 2);
        }
    }
    updateNPCPadPos(enemy);
    let state = lw(record + 2728);
    let set_state = |new_state: i32| -> Next {
        let seen = lw(record + 2772);
        sw(record + 2728, new_state);
        Next::Tail(seen)
    };
    let set_flags = |new_flags: i32| -> Next {
        let seen = lw(record + 2772);
        sw(record + 2704, new_flags);
        Next::Tail(seen)
    };
    let next = 'machine: {
        if (state as u32) < 10 {
            match state {
                0 => break 'machine set_state(1),
                1 => {
                    if distance < 15.0_f32 {
                        break 'machine set_state(2);
                    }
                    swc1(record + 2876, lwc1(record + 2856));
                    enemySetMoveMode(enemy, 1);
                    let goal = lw(record + 2788);
                    let limit = lwc1(0x3ad928);
                    let wait = lwc1(record + 2908);
                    swc1(record + 2796, lwc1(goal + 48));
                    swc1(record + 2800, lwc1(goal + 52));
                    swc1(record + 2804, lwc1(goal + 56));
                    if !(wait < limit) {
                        break 'machine Next::Tail(lw(record + 2772));
                    }
                    break 'machine set_state(5);
                }
                2 => {
                    let goal = lw(record + 2788);
                    swc1(record + 2872, lwc1(record + 2860));
                    swc1(record + 2796, lwc1(goal + 48));
                    swc1(record + 2800, lwc1(goal + 52));
                    swc1(record + 2804, lwc1(goal + 56));
                    if !(distance < 9.0_f32) {
                        if !(distance < 15.0_f32) {
                            break 'machine set_state(1);
                        }
                        enemySetMoveMode(enemy, 1);
                        let limit = lwc1(0x3ad92c);
                        let flags;
                        if limit < lwc1(record + 2892) {
                            flags = lw(record + 2704);
                        } else {
                            if !(limit < lwc1(record + 2912)) {
                                break 'machine Next::Tail(lw(record + 2772));
                            }
                            flags = lw(record + 2704);
                        }
                        sw(record + 2892, 0);
                        break 'machine set_flags(flags | 2);
                    }
                    sw(record + 2892, 0);
                    sw(record + 2728, 3);
                    break 'machine Next::Tail(lw(record + 2772));
                }
                3 => {
                    enemySetMoveMode(enemy, 8);
                    let mut drift = fptodp(lwc1(record + 2780));
                    if dpcmp(drift, 0_i64) < 0 {
                        drift = dpsub(0_i64, drift);
                    }
                    if dpcmp(drift, 0x4024000000000000_i64) >= 0 {
                        break 'machine Next::Tail(lw(record + 2772));
                    }
                    let flags = lw(record + 2704);
                    sw(record + 2728, 7);
                    sw(record + 2892, 0);
                    sw(record + 2704, flags | 2);
                    break 'machine Next::Tail(lw(record + 2772));
                }
                4 | 6 => {
                    enemySetMoveMode(enemy, 512);
                    let (animation, speed) = if state == 4 { (473, 1.0_f32) } else { (449, lwc1(0x3ad930)) };
                    if curAnim(enemy, animation) != 0 {
                        break 'machine drift_check(enemy, record, lwc1(record + 2780));
                    }
                    setAnim(enemy, animation, 0.25_f32, speed);
                    let flags = lw(record + 2704);
                    sw(record + 2892, 0);
                    break 'machine set_flags(flags | 6);
                }
                5 => {
                    enemySetMoveMode(enemy, 4);
                    let flags = lw(record + 2704);
                    if (flags & 2) != 0 {
                        sw(record + 2704, flags | 6);
                    }
                    if !(3.5_f32 < lwc1(record + 2892)) {
                        break 'machine Next::Tail(lw(record + 2772));
                    }
                    let other = lw(record + 2708);
                    sw(record + 2892, 0);
                    sw(record + 2728, 1);
                    sw(record + 2708, other & -5);
                    break 'machine Next::Tail(lw(record + 2772));
                }
                7 => {
                    if !(16.0_f32 < distance) {
                        enemySetMoveMode(enemy, 8);
                    } else {
                        sw(record + 2728, 1);
                    }
                    if !(lwc1(0x3ad934) < lwc1(record + 2892)) {
                        break 'machine Next::Tail(lw(record + 2772));
                    }
                    sw(record + 2892, 0);
                    let r = random_unit();
                    if r < lwc1(0x3ad938) && 1e+01_f32 < distance {
                        break 'machine set_state(6);
                    }
                    if r < 0.5_f32 {
                        break 'machine set_state(3);
                    }
                    break 'machine set_state(4);
                }
                8 => {
                    enemySetMoveMode(enemy, 8);
                    let flags = lw(record + 2708);
                    if (flags & 256) != 0 {
                        sw(record + 2708, flags & -257);
                    }
                    let seen = lw(record + 2772);
                    if !(0.5_f32 < lwc1(record + 2892)) {
                        if seen == 0 {
                            break 'machine Next::State;
                        }
                    } else if seen == 0 {
                        let previous = lw(record + 2716);
                        sw(record + 2716, 256);
                        sw(record + 2720, previous);
                        sw(record + 2728, 0);
                        break 'machine Next::Clear;
                    }
                    sw(record + 2728, 9);
                    break 'machine Next::Tail(seen);
                }
                _ => {
                    break 'machine set_state(if 15.0_f32 < distance { 1 } else { 2 });
                }
            }
        }
        Next::Tail(lw(record + 2772))
    };
    match next {
        Next::Tail(seen) => {
            if seen != 0 {
                return;
            }
        }
        Next::State => {}
        Next::Clear => {
            sw(record + 2892, 0);
            return;
        }
    }
    if lw(record + 2728) == 8 {
        return;
    }
    sw(record + 2728, 8);
    sw(record + 2892, 0);
}
