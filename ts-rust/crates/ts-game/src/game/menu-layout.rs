//! Placing the items of a menu, and the map maker's colour dialog.

use crate::*;

/// A position along one axis from a mode: 0 the start (`base`), 1 / 3 the near / far edge of the page (`near`, `far`), 2 their middle; modes 4 to 6 are the
/// same but with the item (of size `size`) centred on that edge, and 7 to 9 with it ending there; 0 for modes from 10 (@ 0x20cb50).
pub fn menuitemCalcPos(mode: i32, base: i32, size: i32, near: i32, far: i32) -> i32 {
    if (mode as u32) >= 10 {
        return 0;
    }
    let middle = (near + far) / 2 + base;
    match mode {
        0 => base,
        1 => near + base,
        2 => middle,
        3 => far + base,
        4 => near + base - size / 2,
        5 => middle - size / 2,
        6 => far + base - size / 2,
        7 => near + base - size,
        8 => middle - size,
        _ => far + base - size,
    }
}

/// Make the colour dialog of the map maker: a box (24 bytes) for the dialog and two for the sliders, put into the window; returns 3 (the number of boxes) (@ 0x251768).
pub fn makecolourdialog(boxes: i32) -> i32 {
    let window_x = lw(g::w_x);
    let window_y = lw(g::w_y);
    makebox(boxes, 54, 158, 260, 10, 0, 1, 0x24fb00);
    let texts = (lw(g::cur_language) << 2) + g::languages;
    sw(boxes + 20, g::nt_center);
    let y = ((window_y + 55) << 16) >> 16;
    sw(boxes + 16, lw(lw(texts) + 3492));
    let second = boxes + 24;
    makebox(second, ((window_x + 64) << 16) >> 16, y, 192, 96, 1, 0, 0x2512b0);
    let third = second + 24;
    makebox(third, ((window_x + 266) << 16) >> 16, y, 30, 96, 2, 0, 0x251600);
    ((third + 24 - boxes).wrapping_mul(-0x55555555)) >> 3
}
