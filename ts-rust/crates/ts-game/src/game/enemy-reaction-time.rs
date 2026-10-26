//! How long an enemy waits before it reacts.

use super::particle_spawners::random_as_float;
use crate::*;

/// How long, in seconds, a computer character waits to react. In the story mode (game mode 7) by its difficulty setting 0 to 2 a random time from half of a constant to one and a half of it,
/// the constants being the game's; otherwise by the skill of its kind (a byte of the game parameters, 0 to 4) a random time on top of a base, one pair of constants for each;
/// 0 for any other skill (@ 0x2c15f8).
pub fn enemyReactTime(chr: i32) -> f32 {
    let params = g::curgameparms;
    if lw(params + 72) == 7 {
        let mode = lw(params + 76);
        if mode == 0 || mode == 1 || mode == 2 {
            let r = random_as_float();
            let base = if mode == 0 {
                0.5_f32
            } else if mode == 1 {
                lwc1(0x3ad8c0)
            } else {
                lwc1(0x3ad8c4)
            };
            let unit = ps2float::mul(r, 2.3283064e-10_f32);
            return ps2float::add(ps2float::mul(unit, base), base);
        }
    }
    let state = lw(chr + 352);
    let skill = lbu(lw(state) + (g::curgameparms + 49));
    if (skill as u32) >= 5 {
        return 0.0_f32;
    }
    let r = random_as_float();
    let unit = ps2float::mul(r, 2.3283064e-10_f32);
    match skill {
        0 => ps2float::add(unit, 1.75_f32),
        1 => ps2float::add(ps2float::mul(unit, lwc1(0x3ad8c8)), lwc1(0x3ad8cc)),
        2 => ps2float::add(ps2float::mul(unit, lwc1(0x3ad8d0)), lwc1(0x3ad8d4)),
        3 => ps2float::add(ps2float::mul(unit, lwc1(0x3ad8d8)), lwc1(0x3ad8dc)),
        4 => ps2float::add(ps2float::mul(unit, lwc1(0x3ad8e0)), lwc1(0x3ad8e4)),
        _ => unreachable!(),
    }
}
