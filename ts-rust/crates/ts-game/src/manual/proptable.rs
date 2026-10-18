//! The game's table of props (`proptable`: 713 entries of 28 bytes) and the table of the walls and floors of props (`PropWallsFloors`: 16 bytes
//! for each) hold an entry for each room of a level (the room `n` is the prop number 512 + n), so a level could have 200 rooms at the most.  Both
//! are moved to the port's memory (`tools/patches.py`, GLOBAL), with entries for 1100 props: a level of a mod can have up to 480 rooms.
//!
//! The lists of the props of the rooms (`propGfxRoom`: they held rooms up to 213) are moved too.
//!
//! The entries added after the game's are copies of its last (a room's prop), each with a place of its own for what the entry points at.

use crate::*;

const BASE: i32 = 0x240_0000;
pub const PROPTABLE: i32 = BASE + 0x11_0000;
const PROP_SLOTS: i32 = BASE + 0x13_0000;
pub const PWFTAB: i32 = BASE + 0x14_0000;
const GAME_ENTRIES: i32 = 713;
pub const MAX_ENTRIES: i32 = 1100;
/// the props of each room, for the drawing (`propGfxRoom`): a count and 100 props for each room and for each of the special numbers 205 to 212
pub const ROOM_PROP_COUNTS: i32 = BASE + 0x15_0000;
pub const ROOM_PROP_LISTS: i32 = BASE + 0x16_0000;

/// Make the tables in the port's memory (once, before anything runs).
pub fn prop_table_init() {
    for i in 0..GAME_ENTRIES * 7 {
        sw(PROPTABLE + 4 * i, lw(g::proptable + 4 * i));
    }
    for k in GAME_ENTRIES..MAX_ENTRIES {
        for w in 0..7 {
            sw(PROPTABLE + 28 * k + 4 * w, lw(g::proptable + 28 * (GAME_ENTRIES - 1) + 4 * w));
        }
        sw(PROPTABLE + 28 * k + 8, PROP_SLOTS + 8 * k);
    }
    for i in 0..GAME_ENTRIES * 4 {
        sw(PWFTAB + 4 * i, lw(g::PropWallsFloors + 4 * i));
    }
}
