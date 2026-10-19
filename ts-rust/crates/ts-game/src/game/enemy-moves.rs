//! The movement mode of an enemy.

use crate::*;

/// Set an enemy's move mode (the flags at +2708 of its record): each mode sets its own bit and clears the bits of the modes it excludes; mode 32
/// keeps only flag 0x1000, mode 512 remembers the old flags at +2712.  Other numbers do nothing (@ 0x2c82f8).
pub fn enemySetMoveMode(enemy: i32, mode: i32) {
    let data = lw(enemy + 352);
    // (the flags are only read for a mode the function knows)
    let flags = || lw(data + 2708);
    match mode {
        32 => sw(data + 2708, flags() & 0x1000 | 32),
        512 => {
            let old = flags();
            sw(data + 2712, old);
            sw(data + 2708, old | 512);
        }
        1024 => sw(data + 2708, (flags() | 1024) & -2574),
        2048 => sw(data + 2708, (flags() | 2048) & -513),
        128 => sw(data + 2708, (flags() | 128) & -3594),
        4 => sw(data + 2708, (flags() | 4) & -513),
        8 => sw(data + 2708, (flags() | 8) & -3714),
        1 => sw(data + 2708, (flags() | 1) & -3725),
        _ => {}
    }
}
