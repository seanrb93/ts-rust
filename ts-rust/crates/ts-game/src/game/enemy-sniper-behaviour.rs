//! The behaviour of an enemy that snipes from where it stands.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// What the enemy `enemy` does as a sniper, by its state at +2728 of its record: with more than one player it takes the co-op target (+2788), and without a target nothing is done.
/// State 0: move mode 8, to state 2 for a random time. State 1: mode 4, to state 2 after its time. State 2: mode 8 (flag 4 of +2708 cleared), to state 3 with flag 2 of +2704
/// after its time. State 3: back to state 2 for a random time of 1 to 3, mode 8 while flag 2 is set. State 4: to state 1 or, by chance (0x3ada38), state 2. State 5: the target
/// becomes the first base that is not whole (health under 1) and the state 0. State 6: after 2 seconds with a new random player as the target, to state 0. In the mode 6 a target
/// that has fallen puts the enemy in state 6 and one that stands puts it in state 5 (@ 0x2ca6a8).
pub fn enemySnipe1(enemy: i32) {
    let record = lw(enemy + 352);
    if lw(g::lvnumplayers) >= 2 {
        let target = checkCoopTarget(enemy);
        sw(record + 2788, target);
    }
    let target = lw(record + 2788);
    if target == 0 {
        return;
    }
    let seen = lw(record + 2772);
    if seen != 0 && (lw(record + 2704) & 0x4000) != 0 {
        swc1(record + 2796, lwc1(target + 48));
        swc1(record + 2800, lwc1(target + 52));
        swc1(record + 2804, lwc1(target + 56));
    }
    let state = lw(record + 2728);
    'state: {
        if (state as u32) < 7 {
            let base_time;
            let extra;
            match state {
                0 => {
                    enemySetMoveMode(enemy, 8);
                    sw(record + 2728, 2);
                    let r = random_unit();
                    let scale = lwc1(record + 2964);
                    sw(record + 2892, 0);
                    swc1(record + 2964, ps2float::mul(scale, 0.25_f32));
                    swc1(record + 2732, ps2float::add(ps2float::add(r, r), 0.5_f32));
                    break 'state;
                }
                1 => {
                    enemySetMoveMode(enemy, 4);
                    if !(lwc1(record + 2732) < lwc1(record + 2892)) {
                        break 'state;
                    }
                    sw(record + 2728, 2);
                    let r = random_unit();
                    sw(record + 2892, 0);
                    extra = lwc1(0x3ada34);
                    base_time = ps2float::mul(r, 0.5_f32);
                }
                2 => {
                    sw(record + 2708, lw(record + 2708) & -5);
                    enemySetMoveMode(enemy, 8);
                    if !(lwc1(record + 2732) < lwc1(record + 2892)) {
                        break 'state;
                    }
                    let flags = lw(record + 2704);
                    sw(record + 2728, 3);
                    sw(record + 2704, flags | 2);
                    break 'state;
                }
                3 => {
                    if (lw(record + 2704) & 2) != 0 {
                        enemySetMoveMode(enemy, 8);
                        break 'state;
                    }
                    sw(record + 2728, 2);
                    let r = random_unit();
                    extra = 1.0_f32;
                    sw(record + 2892, 0);
                    base_time = ps2float::add(r, r);
                }
                4 => {
                    if (lw(record + 2704) & 2) != 0 {
                        break 'state;
                    }
                    let r = random_unit();
                    if !(r < lwc1(0x3ada38)) {
                        sw(record + 2728, 1);
                        let r = random_unit();
                        extra = 1.0_f32;
                        base_time = r;
                        sw(record + 2892, 0);
                    } else {
                        enemySetMoveMode(enemy, 8);
                        sw(record + 2728, 2);
                        let r = random_unit();
                        extra = 1.0_f32;
                        sw(record + 2892, 0);
                        base_time = ps2float::add(r, r);
                    }
                }
                5 => {
                    let count = lw(g::numBases);
                    let mut index = 0;
                    if count > 0 {
                        let bases = g::bases;
                        while index < count && 1.0_f32 <= lwc1(lw(bases + (index << 3) + 4) + 532) {
                            index += 1;
                        }
                    }
                    let offset = index << 3;
                    sw(record + 2728, 0);
                    sw(record + 2788, lw(g::bases + offset + 4));
                    return;
                }
                _ => {
                    if !(2.0_f32 < lwc1(record + 2892)) {
                        if seen == 0 {
                            return;
                        }
                        sw(record + 2728, 0);
                        return;
                    }
                    if seen != 0 {
                        sw(record + 2728, 0);
                        return;
                    }
                    let r = newrnd();
                    let players = lw(g::lvnumplayers);
                    if players == 0 {
                        trap("break");
                    }
                    let list = lw(g::players);
                    sw(record + 2728, 0);
                    sw(record + 2788, lw(((r as i32 as u32) % (players as u32)) as i32 * 1820 + list + 384));
                    return;
                }
            }
            swc1(record + 2732, base_time + extra);
        }
    }
    let mode = lw(g::curgameparms + 72);
    if mode == 6 {
        if !(1.0_f32 <= lwc1(lw(record + 2788) + 532)) {
            let now = lw(record + 2728);
            if now != mode && now != 1 && (lw(record + 2708) & 4) == 0 && lw(record + 2772) == 0 {
                sw(record + 2728, mode);
                sw(record + 2892, 0);
            }
        } else {
            sw(record + 2892, 0);
            sw(record + 2728, 5);
        }
    }
}
