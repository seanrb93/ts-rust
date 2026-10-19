//! Small pieces that do not belong to a larger system yet.

use crate::*;

/// Fill in a 24-byte box record for the menus: four signed 16-bit numbers, a flags word, a byte, and a value at +12 (@ 0x22cec0).
pub fn makebox(b: i32, x0: i32, y0: i32, x1: i32, y1: i32, flags: i32, kind: i32, value: i32) {
    memset(b, 0, 24);
    sw(b + 12, value);
    sh(b, x0);
    sh(b + 2, y0);
    sh(b + 4, x1);
    sh(b + 6, y1);
    sh(b + 8, flags);
    sh(b + 10, kind & 255);
}

/// Whether a player can use a weapon in the left hand: not when the player carries a one-handed-only setup (@ 0x2950b8).
pub fn gunLeftHandAble(player: i32) -> i32 {
    (lw(player + 484) == 0) as i32
}

/// Whether a player can use a weapon in the right hand (@ 0x2950c8).
pub fn gunRightHandAble(player: i32) -> i32 {
    if lw(player + 484) == 0 {
        return 1;
    }
    ((lw(g::curgameparms + 80) & 2048) == 0) as i32
}
