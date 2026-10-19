//! Drawing and measuring in the map maker's world.

use crate::*;

/// Draw a line between two points of the map maker's world: each coordinate is moved by the view (`m_x`, `m_y`), scaled to the window by a fixed
/// point factor and put in the window (@ 0x249450).
pub fn mmDrawLine(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32) {
    let view_x = lw(g::m_x);
    let scale_x = lw(g::scale_mtowx);
    let view_y = lw(g::m_y);
    let scale_y = lw(g::scale_mtowy);
    let (dx1, dy1, dy0, dx0) = (x1 - view_x, y1 - view_y, y0 - view_y, x0 - view_x);
    let window_x = lw(g::w_x);
    let window_y = lw(g::w_y);
    let wx1 = window_x + ((FMul(dx1, scale_x) as i32) >> 16);
    let wy1 = window_y + ((FMul(dy1, scale_y) as i32) >> 16);
    let wx0 = window_x + ((FMul(dx0, scale_x) as i32) >> 16);
    let wy0 = window_y + ((FMul(dy0, scale_y) as i32) >> 16);
    dlDrawLine(wx0, wy0, wx1, wy1, colour);
}

/// Find the bounds of all the map maker's objects (a list through +56; an object of kind 1 covers its tile's width and height in 2^21 units): the words
/// `min_x`, `max_x`, `min_y`, `max_y` get them, `max_w` and `max_h` the size, and `min_w` and `min_h` are 128 (@ 0x249530).
pub fn mmFindBoundaries() {
    let (mut low_x, mut low_y) = (0x7fffffff, 0x7fffffff);
    let (mut high_x, mut high_y) = (-0x80000000_i32, -0x80000000_i32);
    sw(g::min_y, low_y);
    sw(g::min_x, low_x);
    sw(g::max_y, high_y);
    sw(g::max_x, high_x);
    let mut object = lw(g::mapobjects);
    while object != 0 {
        let x = lw(object + 12);
        let mut far_y = lw(object + 16);
        let mut far_x = x;
        if lw(object) == 1 {
            let tile = lw(object + (lw(object + 24) << 2) + 36);
            far_y = far_y.wrapping_add(lw(tile + 4) << 21);
            far_x = x.wrapping_add(lw(tile) << 21);
        }
        if x < low_x {
            low_x = x;
            sw(g::min_x, x);
        }
        if high_x < far_x {
            high_x = far_x;
            sw(g::max_x, far_x);
        }
        let y = lw(object + 16);
        if y < low_y {
            low_y = y;
            sw(g::min_y, y);
        }
        if high_y < far_y {
            high_y = far_y;
            sw(g::max_y, far_y);
        }
        object = lw(object + 56);
    }
    sw(g::max_w, high_x.wrapping_sub(low_x));
    sw(g::max_h, high_y.wrapping_sub(low_y));
    sw(g::min_h, 128);
    sw(g::min_w, 128);
}
