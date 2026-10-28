//! What an enemy does while it stands at a window, and the choice of its death animation.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// What the enemy `enemy` does at a window, by its state at +2728 of its record: with a random chance (1 in 4 while in state 1, the constant at 0x3ada3c otherwise) it is fired at
/// (`enemyFiredAt`), and with more than one player it takes the co-op target. State 0: move mode 8, to state 2 for a random time. State 1: mode 4 with flags 2 and 4 of +2704 cleared,
/// to state 2 for a time of 0.5 to 1 plus the constant at 0x3ada40 after its time. State 2: mode 8, to state 1 (mode 4) when fired at, to state 3 with flags 2 and 4 of +2704 set after
/// its time. State 3: back to state 1 (mode 4) when fired at or by chance (0x3ada44), else to state 2, for a random time of 1 to 3 (@ 0x2cab98).
pub fn enemyWindow(enemy: i32) {
    let record = lw(enemy + 352);
    let mut chance = lwc1(0x3ada3c);
    let mut fired = 0;
    if lw(record + 2728) == 1 {
        chance = 0.25_f32;
    }
    let roll = random_unit();
    let mut players = lw(g::lvnumplayers);
    if roll < chance {
        fired = enemyFiredAt(enemy);
        players = lw(g::lvnumplayers);
    }
    let state;
    if players < 2 {
        state = lw(record + 2728);
    } else {
        let target = checkCoopTarget(enemy);
        sw(record + 2788, target);
        state = lw(record + 2728);
    }
    if state == 1 {
        sw(record + 2704, lw(record + 2704) & -3 & -5);
        enemySetMoveMode(enemy, 4);
        if !(lwc1(record + 2732) < lwc1(record + 2892)) {
            return;
        }
        sw(record + 2728, 2);
        let r = random_unit();
        sw(record + 2892, 0);
        swc1(record + 2732, ps2float::add(ps2float::mul(r, 0.5_f32), lwc1(0x3ada40)));
        return;
    }
    if state < 2 {
        if state == 0 {
            enemySetMoveMode(enemy, 8);
            sw(record + 2728, 2);
            let r = random_unit();
            let scale = lwc1(record + 2964);
            sw(record + 2892, 0);
            swc1(record + 2964, ps2float::mul(scale, 0.25_f32));
            swc1(record + 2732, ps2float::add(ps2float::add(r, r), 0.5_f32));
        }
        return;
    }
    if state == 2 {
        sw(record + 2708, lw(record + 2708) & -5);
        enemySetMoveMode(enemy, 8);
        if fired != 0 {
            enemySetMoveMode(enemy, 4);
            sw(record + 2728, 1);
            let r = random_unit();
            sw(record + 2892, 0);
            swc1(record + 2732, ps2float::add(ps2float::add(r, r), 1.0_f32));
        }
        if !(lwc1(record + 2732) < lwc1(record + 2892)) {
            return;
        }
        let flags = lw(record + 2704);
        sw(record + 2728, 3);
        sw(record + 2704, flags | 6);
        return;
    }
    if state != 3 {
        return;
    }
    if fired == 0 {
        if (lw(record + 2704) & 2) != 0 {
            enemySetMoveMode(enemy, 8);
            return;
        }
        let r = random_unit();
        if !(r < lwc1(0x3ada44)) {
            sw(record + 2728, 2);
        } else {
            enemySetMoveMode(enemy, 4);
            sw(record + 2728, 1);
        }
    } else {
        enemySetMoveMode(enemy, 4);
        sw(record + 2728, 1);
    }
    let r = random_unit();
    sw(record + 2892, 0);
    swc1(record + 2732, ps2float::add(ps2float::add(r, r), 1.0_f32));
}

/// Start the death animation `anim` of `chr`: 21 more when the character's model type (+12 of the thing at +88 of its model) is 1; animation 599 is started at double speed
/// (the end of `chrSelectDeathAnim`).
fn start_death_animation(chr: i32, anim: i32) {
    let model = lw(chr + 32);
    let anim = if (lw(lw(model + 88) + 12) ^ 1) == 0 { anim + 21 } else { anim };
    if anim != 599 {
        setAnim(chr, anim, 0.0_f32, 1.0_f32);
    } else {
        setAnim(chr, 599, 0.0_f32, 2.0_f32);
    }
}

/// One of the animations 568 to 580 from the random number `r` (nine choices; 572 and up are picked again from the same number, 580 becomes 581).
fn pick_death_animation(r: i64) -> i32 {
    let first = ((r as i32 as u32) % 9) as i32 + 568;
    let second = if first >= 572 { ((r as i32 as u32) % 9) as i32 + 572 } else { first };
    if (second ^ 580) == 0 { second + 1 } else { second }
}

/// Choose and start the death animation of `chr` (`from` is the direction of the hit, `at` and `dir` the place and direction of the shot as three floats, `kind` the kind of
/// the hit and `flag` 1 for a hit from the side): a character that is state 1 (+8 of its record) lies down (animation 577 or 579 when not already); with `from` -1 animation 471;
/// on a pad with flag 2, 570, 571 or 578; hits from the side of the kinds 8 to 18 give 572 to 575; the kinds 3 to 6 give 582 to 585; other hits pick by `pick_death_animation`
/// (566 or 567 when the character does not see the shot coming) with 576 to 578 and, at 580, a chance of 1 in 5 (@ 0x288a30).
pub fn chrSelectDeathAnim(chr: i32, from: i32, at: i32, dir: i32, kind: i32, flag: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let record = lw(chr + 352);
    // (BlueRobot has no death animation: those of the people would turn him upside down; he is left standing until he is removed)
    #[cfg(feature = "extensions")]
    if lhu(record + 4) == 79 {
        return;
    }
    if lw(record + 8) == 1 {
        if curAnim(chr, 577) == 0 {
            if curAnim(chr, 579) == 0 {
                let r = random_unit();
                if !(0.5_f32 < r) {
                    setAnim(chr, 579, 0.0_f32, 1.0_f32);
                } else {
                    setAnim(chr, 577, 0.0_f32, 1.0_f32);
                }
            }
        }
        return;
    }
    if from == -1 {
        setAnim(chr, 471, 0.0_f32, 1.0_f32);
        return;
    }
    let pad = lw(record + 680);
    let anim;
    if (lw(pad * 28 + lw(g::levelpads) + 4) & 2) != 0 {
        let mut chosen = 570;
        let r = newrnd();
        let part = ((r as i32 as u32) % 3) as i32;
        if part != 0 {
            chosen = if (part ^ 1) == 0 { 571 } else { 578 };
        }
        start_death_animation(chr, chosen);
        return;
    }
    if ((from - 8) as u32) < 11 && flag == 1 {
        let mut chosen = 572;
        let r = newrnd() & 3;
        if r != 0 {
            chosen = 573;
            if r != flag as i64 {
                chosen = if (r ^ 2) == 0 { 574 } else { 575 };
            }
        }
        start_death_animation(chr, chosen);
        return;
    }
    if kind != 5 && kind != 3 && kind != 6 && kind != 4 {
        if from <= 0 {
            let r = newrnd();
            start_death_animation(chr, pick_death_animation(r));
            return;
        }
        let x = ps2float::sub(lwc1(at), ps2float::mul(lwc1(dir), 1e+02_f32));
        let y = ps2float::sub(lwc1(at + 4), ps2float::mul(lwc1(dir + 4), 1e+02_f32));
        swc1(sp + 8, ps2float::sub(lwc1(at + 8), ps2float::mul(lwc1(dir + 8), 1e+02_f32)));
        swc1(sp, x);
        swc1(sp + 4, y);
        let mut chosen;
        let mut seen;
        if checkInFront(chr, sp, 0.0_f32) == 0 {
            let r = newrnd();
            seen = lw(chr + 356);
            chosen = if (r & 1) == 0 { 566 } else { 567 };
        } else {
            let r = newrnd();
            seen = lw(chr + 356);
            chosen = pick_death_animation(r);
            if seen != 0 && (chosen == 568 || chosen == 579) {
                let r = newrnd();
                seen = lw(chr + 356);
                chosen = ((r as i32 as u32) % 3) as i32 + 576;
            }
        }
        if seen != 0 {
            start_death_animation(chr, chosen);
            return;
        }
        if lw(record + 2708) != 1 {
            start_death_animation(chr, chosen);
            return;
        }
        if !(lwc1(record + 2860) < lwc1(record + 2876)) {
            start_death_animation(chr, chosen);
            return;
        }
        let r = newrnd();
        if ((r as i32 as u32) % 5) as i32 == 0 {
            chosen = 580;
        }
        start_death_animation(chr, chosen);
        return;
    }
    anim = match kind {
        4 => 584,
        5 => 583,
        6 => 585,
        _ => 582,
    };
    start_death_animation(chr, anim);
}
