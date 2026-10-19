//! Match statistics and awards, and the cheat selection.

use crate::*;

const PLAYER_STATS_SIZE: i32 = 1368;

/// Whether `n` is a slot of the statistics table that is in play: below 14, and either one of the players or characters in the match, or
/// (in the two modes 6 and 7 and when the game parameters' flag 32 is set) one of the four players (shared by all `Stats...` functions).
pub(super) fn stats_slot_in_play(n: i32) -> bool {
    if n >= 14 {
        return false;
    }
    let params = g::curgameparms;
    if ((lw(params + 72) - 6) as u32) >= 2 && (lw(params + 80) & 32) == 0 {
        n < lw(g::lvnumplayers) + lw(g::numactivechars)
    } else {
        n < 4
    }
}

/// Set statistic `stat` of slot `slot` (@ 0x223de0).
pub fn StatsSet(stat: i32, slot: i32, value: f32) {
    if stats_slot_in_play(slot) {
        swc1((stat << 2) + slot * PLAYER_STATS_SIZE + g::matchstats, value);
    }
}

/// Add to statistic `stat` of slot `slot` (@ 0x223e60).
pub fn StatsAdd(stat: i32, slot: i32, value: f32) {
    if stats_slot_in_play(slot) {
        let at = (stat << 2) + slot * PLAYER_STATS_SIZE + g::matchstats;
        swc1(at, lwc1(at) + value);
    }
}

/// Read statistic `stat` of slot `slot` (@ 0x223ee8).
pub fn StatsGet(stat: i32, slot: i32) -> f32 {
    if stats_slot_in_play(slot) {
        lwc1((stat << 2) + slot * PLAYER_STATS_SIZE + g::matchstats)
    } else {
        0.0
    }
}

/// Set a statistic of `slot` about another character `other` (kills by whom, ...): 24 bytes for each other, from +936 (@ 0x224250).
pub fn StatsSetChar(stat: i32, slot: i32, other: i32, value: f32) {
    if stats_slot_in_play(slot) && stats_slot_in_play(other) {
        swc1((stat << 2) + other * 24 + slot * PLAYER_STATS_SIZE + (g::matchstats + 936), value);
    }
}

/// Clear the statistics of one life (a record of 396 bytes: counters, then a table of words and a table of bytes) (@ 0x224428).
pub fn StatsLifeReset(life: i32) {
    for at in (0..=60).step_by(4) {
        sw(life + at, 0);
    }
    for at in (64..=196).step_by(4) {
        sw(life + at, 0);
    }
    sw(life + 344, 0);
    sw(life + 336, 0);
    sw(life + 340, 0);
    for at in 348..=395 {
        sb(life + at, 0);
    }
}

/// An award of tier `tier` (0 to 3) in place `place` (0 to 2); 0 outside (@ 0x224b20).
pub fn StatsGetAward(tier: i32, place: i32) -> i32 {
    if tier < 4 && place < 3 {
        lw((place << 2) + tier * 12 + g::awards)
    } else {
        0
    }
}

/// How many places of a tier have an award (@ 0x224b60).
pub fn StatsGetNumAwards(tier: i32) -> i32 {
    if tier >= 4 {
        return 0;
    }
    (0..3).filter(|&p| lw(tier * 12 + g::awards + 4 * p) != 0).count() as i32
}

/// The cheats the character select screen uses (@ 0x2255a8).
pub fn cheatsChrSelect() -> i32 {
    0x2007
}

/// No cheats selected or allowed (@ 0x225670).
pub fn cheatsMake() {
    sw(g::selectedcheats, 0);
    sw(g::allowedcheats, 0);
}

/// The unlocked cheats (of 15) whose definition (12 bytes each, flags at +4) has `flag` (a bit mask over the cheats).
fn unlocked_cheats_with(flag: i32) -> i32 {
    let unlocked = lw(g::unlockedcheats);
    let mut mask = 0;
    for i in 0..15 {
        if (unlocked >> i) & 1 != 0 && lw(g::cheatdefs + 4 + 12 * i) & flag != 0 {
            mask |= 1 << i;
        }
    }
    mask
}

/// Unlocked cheats that work in Arcade (flag 2) (@ 0x2255b0).
pub fn cheatsArcade() -> i32 {
    unlocked_cheats_with(2)
}

/// Unlocked cheats that work in Story (flag 1) (@ 0x225610).
pub fn cheatsStory() -> i32 {
    unlocked_cheats_with(1)
}

/// The text of a cheat menu item in the current language, or 0 for none (@ 0x231290).
pub fn cheatOption(item: i32) -> i32 {
    let language = lw(g::cur_language);
    if item == 0 {
        0
    } else {
        lw(lw((language << 2) + g::languages) + 536)
    }
}

/// A cheat menu item (40 bytes, from 0x1fc2d98) was switched; its word at +0 is the new state.  Cheats 0-1, 6-7 and 8-12 are groups of which
/// only one can be on (@ 0x239718).
pub fn cheatOnChange(item: i32) {
    let index = (item - 0x1fc2d98).wrapping_mul(-0x33333333) >> 3;
    let bit = 1 << (index & 31);
    let mut selected = lw(g::selectedcheats);
    if bit & 3 != 0 {
        selected &= !3;
    } else if bit & 192 != 0 {
        selected &= !192;
    } else if bit & 0x1f00 != 0 {
        selected &= !0x1f00;
    }
    if lw(item) == 0 {
        sw(g::selectedcheats, selected & !bit);
    } else {
        sw(g::selectedcheats, selected | bit);
    }
}

/// How many of the four teams are allowed (@ 0x2353b0).
pub fn numAllowedTeams() -> i32 {
    (0..4).filter(|&i| lbu(0x1fc1234 + i) != 0).count() as i32
}
