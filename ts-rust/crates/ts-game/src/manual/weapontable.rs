//! The table of weapon combinations in the port's memory.
//!
//! The game keeps its 33 combinations (`guncombos`: rank, left gun, right gun, name; 8 bytes each) and the order the weapons are cycled
//! in (`gunsinventory`) in its data, with other data right behind them, so they cannot grow there.  The generated code reads them through
//! `GUNCOMBOS` and `GUNSINV` (`tools/patches.py`, GLOBAL), which are places in the port's memory with room for 64 combinations.

use crate::*;

/// the places of the tables in the port's memory
pub const GUNCOMBOS: i32 = 0x240_0000 + 0x40000;
pub const GUNSINV: i32 = 0x240_0000 + 0x40400;
/// room for this many combinations
pub const MAX_COMBOS: i32 = 64;
/// the game's own combinations
pub const GAME_COMBOS: i32 = 33;

/// Copy the game's tables to their new places (once, before anything runs).
pub fn weapon_table_init() {
    for i in 0..GAME_COMBOS * 2 {
        sw(GUNCOMBOS + 4 * i, lw(g::guncombos + 4 * i));
    }
    for i in 0..GAME_COMBOS {
        sw(GUNSINV + 4 * i, lw(g::gunsinventory + 4 * i));
    }
}
