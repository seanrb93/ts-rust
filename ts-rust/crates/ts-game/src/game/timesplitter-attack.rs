//! The attack of the Timesplitter enemy.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// The attack of a Timesplitter, by the stage kept at +2728 of its state: 0 picks the next stage (3 or 4, at random) and sets move mode 8; 1 plays the animations 549, then 553
/// (with a sound), then 533 or 534, and goes to stage 2; 2 waits for flag 2 of +2704 to go, then picks stage 3 or 4 at random; 3 and 4 move (move modes 1 and 4, or 1 and
/// 2048) and play animation 559; after stages 0 and 2 the stage goes back to 1 when the target is nearer than 9 (@ 0x2c38f0).
pub fn enemyTimeSplitterAttack(chr: i32) {
    let state = lw(chr + 352);
    let target = lw(state + 2788);
    let dx = ps2float::sub(lwc1(chr + 48), lwc1(target + 48));
    let dy = ps2float::sub(lwc1(chr + 52), lwc1(target + 52));
    let dz = ps2float::sub(lwc1(chr + 56), lwc1(target + 56));
    let tx = lwc1(target + 48);
    let stage = lw(state + 2728);
    swc1(state + 2796, tx);
    swc1(state + 2800, lwc1(target + 52));
    swc1(state + 2804, lwc1(target + 56));
    if (stage as u32) >= 5 {
        return;
    }
    match stage {
        0 => {
            enemySetMoveMode(chr, 8);
            swc1(state + 2872, lwc1(state + 2856));
            let next = if 0.5_f32 < random_unit() { 4 } else { 3 };
            sw(state + 2728, next);
            return;
        }
        1 => {
            enemySetMoveMode(chr, 512);
            if curAnim(chr, 549) == 0 {
                if curAnim(chr, 553) == 0 {
                    let mut busy = false;
                    if curAnim(chr, 533) != 0 || curAnim(chr, 534) != 0 {
                        busy = true;
                    }
                    if busy && curAnimComplete(lw(chr + 32)) != 0 {
                        setAnim(chr, 549, 0.25_f32, 1.0_f32);
                        return;
                    }
                    if curAnim(chr, 533) != 0 {
                        return;
                    }
                    if curAnim(chr, 534) != 0 {
                        return;
                    }
                    let flags = lw(state + 2704);
                    sw(state + 2728, 2);
                    sw(state + 2704, flags | 2);
                    return;
                }
            }
            if curAnim(chr, 549) != 0 && curAnimComplete(lw(chr + 32)) != 0 {
                setAnim(chr, 553, 0.25_f32, 1.0_f32);
                if frames_gate() <= 0.0_f32 {
                    return;
                }
                soundDelayStartEx(170, chr + 48, 65, 0);
                return;
            }
            if curAnim(chr, 553) == 0 {
                return;
            }
            if curAnimComplete(lw(chr + 32)) == 0 {
                return;
            }
            let flags = lw(state + 2704);
            sw(state + 2728, 2);
            sw(state + 2704, flags | 2);
            return;
        }
        2 => {
            if (lw(state + 2704) & 2) == 0 {
                let next = if !(0.5_f32 < random_unit()) { 3 } else { 4 };
                sw(state + 2728, next);
                sw(state + 2704, lw(state + 2704) & -3);
                return;
            }
            enemySetMoveMode(chr, 8);
            return;
        }
        3 | 4 => {
            let extra_mode = if stage == 3 { 4 } else { 2048 };
            if (lw(state + 2708) & 2052) != 0 {
                enemySetMoveMode(chr, 1);
                enemySetMoveMode(chr, extra_mode);
            } else if curAnim(chr, 559) != 0 {
                if curAnimComplete(lw(chr + 32)) != 0 {
                    enemySetMoveMode(chr, 1);
                    enemySetMoveMode(chr, extra_mode);
                }
            } else {
                enemySetMoveMode(chr, 512);
                setAnim(chr, 559, 0.25_f32, 1.0_f32);
            }
        }
        _ => unreachable!(),
    }
    if !(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt() < 9.0_f32) {
        return;
    }
    sw(state + 2728, 1);
}
