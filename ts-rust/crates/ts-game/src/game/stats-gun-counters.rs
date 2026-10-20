//! The per-gun counters of the match statistics.

use crate::*;

/// The place of the counter of gun `gun` of the combination `combo` for the player `player`, or None when the player (up to 14; the first 4 in the story modes 6 and 7 or with
/// flag 32, else those in the game) or the combination (under 33) is not valid: records of 1368 bytes a player, 20 bytes a combination, a float a gun (@ the three functions below).
fn gun_counter(gun: i32, player: i32, combo: i32) -> Option<i32> {
    if player < 14 {
        let params = g::curgameparms;
        let known;
        if ((lw(params + 72) - 6) as u32) >= 2 && (lw(params + 80) & 32) == 0 {
            known = player < lw(g::lvnumplayers) + lw(g::numactivechars);
        } else {
            known = player < 4;
        }
        if known && combo < 33 {
            return Some((gun << 2) + combo * 20 + player * 1368 + (g::matchstats + 276));
        }
    }
    None
}

/// Set the counter of gun `gun` for the player `player` and the combination `combo` to `value` (@ 0x224110).
pub fn StatsSetGun(gun: i32, player: i32, combo: i32, value: f32) {
    if let Some(at) = gun_counter(gun, player, combo) {
        swc1(at, value);
    }
}

/// Add `value` to the counter (@ 0x2241a8).
pub fn StatsAddGun(gun: i32, player: i32, combo: i32, value: f32) {
    if let Some(at) = gun_counter(gun, player, combo) {
        swc1(at, lwc1(at) + value);
    }
}

/// The counter, or 0 for one that is not valid (@ 0x223f78).
pub fn StatsGetGun(gun: i32, player: i32, combo: i32) -> f32 {
    match gun_counter(gun, player, combo) {
        Some(at) => lwc1(at),
        None => 0.0_f32,
    }
}
