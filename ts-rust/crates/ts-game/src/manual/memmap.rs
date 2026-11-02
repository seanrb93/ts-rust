//! Small tables that the port's extensions keep in the game's memory instead of in the program's own: what the extensions remember while a game is played (who owns an added
//! weapon pair, whose swipe is the left fist's, when a bot may punch again) is part of the state of the game, so it has to be put back when the game goes back some ticks (a
//! network game with rollback, and the pictures made between its ticks), which only what is in the game's memory is.
//!
//! A table is `ENTRIES` records of 16 bytes: a key (an address of the game; 0 is empty, 1 is a place that was emptied) and a value of 8 bytes.

use crate::*;

pub const ENTRIES: i32 = 64;
pub const TABLE_BYTES: i32 = ENTRIES * 16;
const EMPTY: i32 = 0;
const GONE: i32 = 1;

fn start(key: i32) -> i32 {
    (((key as u32) >> 4).wrapping_mul(0x9e37_79b1) >> 20) as i32 % ENTRIES
}

/// The place of `key` in the table at `table`.
fn find(table: i32, key: i32) -> Option<i32> {
    let first = start(key);
    for i in 0..ENTRIES {
        let at = table + 16 * ((first + i) % ENTRIES);
        match lw(at) {
            k if k == key => return Some(at),
            EMPTY => return None,
            _ => {}
        }
    }
    None
}

pub fn mm_get(table: i32, key: i32) -> Option<u64> {
    find(table, key).map(|at| (lw(at + 4) as u32 as u64) | ((lw(at + 8) as u32 as u64) << 32))
}

pub fn mm_set(table: i32, key: i32, value: u64) {
    let at = match find(table, key) {
        Some(at) => at,
        None => {
            let first = start(key);
            let mut free = None;
            for i in 0..ENTRIES {
                let at = table + 16 * ((first + i) % ENTRIES);
                if matches!(lw(at), EMPTY | GONE) {
                    free = Some(at);
                    break;
                }
            }
            let Some(at) = free else { return };
            sw(at, key);
            at
        }
    };
    sw(at + 4, value as u32 as i32);
    sw(at + 8, (value >> 32) as u32 as i32);
}

pub fn mm_remove(table: i32, key: i32) {
    if let Some(at) = find(table, key) {
        sw(at, GONE);
    }
}

pub fn mm_clear(table: i32) {
    memset(table, 0, TABLE_BYTES);
}
