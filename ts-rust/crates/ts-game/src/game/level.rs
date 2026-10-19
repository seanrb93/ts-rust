//! Accessors of the loaded level (`docs/level-formats.md` describes the files): rooms, portals, pad links.

use crate::*;

/// A room is 44 bytes of the level's room table.
fn room(n: i32) -> i32 {
    n * 44 + lw(g::levelrooms)
}

/// Address of the floors of room `n` (@ 0x2575f0).
pub fn bgGetFloors(n: i32) -> i32 {
    lw(room(n) + 8)
}

/// Address of the walls of room `n` (@ 0x257608).
pub fn bgGetWalls(n: i32) -> i32 {
    lw(room(n) + 12)
}

/// Address of the glass of room `n` (@ 0x257620).
pub fn bgGetGlass(n: i32) -> i32 {
    lw(room(n) + 16)
}

/// A value from the first player's table for room `n`: 20 bytes a room (@ 0x256fa0).
pub fn bgRoomGetLevel(n: i32) -> i32 {
    lw(n * 20 + lw((lw(lw(g::currentplayer)) << 2) + g::roomdata))
}

fn portal(n: i32) -> i32 {
    lw((n << 2) + lw(g::levelportals))
}

/// Close a portal (bit 0 of its flags): nothing is seen through it (@ 0x257778).
pub fn bgPortalDisable(n: i32) {
    let p = portal(n);
    sh(p + 22, lhu(p + 22) | 1);
}

/// Open a portal again (@ 0x257798).
pub fn bgPortalEnable(n: i32) {
    let p = portal(n);
    sh(p + 22, lhu(p + 22) & 0xfffe);
}

/// Disable a link between pads, so the bots do not use it (@ 0x264e68).
pub fn linkDisable(n: i32) {
    let l = n * 20 + lw(g::levellinks);
    sw(l, lw(l) & -2);
}

/// Enable a link between pads (@ 0x264e48).
pub fn linkEnable(n: i32) {
    let l = n * 20 + lw(g::levellinks);
    sw(l, lw(l) | 1);
}
