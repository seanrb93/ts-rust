//! Moving a point in the pad and prop editors, and drawing the sound envelope.

use crate::*;

/// Move the point `p` (x, y, z at +0, +4, +8) with the d-pad of the first controller: left / right (0x1000 / 0x4000) change x, or y when the button 1024 is held,
/// up / down (0x2000 / 0x8000) change z, by constants of the game; only when a button was newly pressed or button 8 is held (@ 0x265c40).
pub fn editTranslate(p: i32) {
    let shift = joyBut(0) & 1024;
    let active = joyNewBut(0) != 0 || joyBut(0) & 8 != 0;
    if !active {
        return;
    }
    if joyBut(0) & 0x1000 != 0 && shift == 0 {
        swc1(p, ps2float::sub(lwc1(p), lwc1(0x3acba8)));
    }
    if joyBut(0) & 0x4000 != 0 && shift == 0 {
        swc1(p, ps2float::add(lwc1(p), lwc1(0x3acbac)));
    }
    if joyBut(0) & 0x2000 != 0 {
        swc1(p + 8, ps2float::add(lwc1(p + 8), lwc1(0x3acbb0)));
    }
    if joyBut(0) & 0x8000 != 0 {
        swc1(p + 8, ps2float::sub(lwc1(p + 8), lwc1(0x3acbb4)));
    }
    if joyBut(0) & 0x1000 != 0 && shift != 0 {
        swc1(p + 4, ps2float::add(lwc1(p + 4), lwc1(0x3acbb8)));
    }
    if joyBut(0) & 0x4000 != 0 && shift != 0 {
        swc1(p + 4, ps2float::sub(lwc1(p + 4), lwc1(0x3acbbc)));
    }
}

/// Draw the sound envelope box (x, y, width, height halfwords): a frame, and a line at a height set by the current sound (@ 0x206d18).
pub fn envelope_draw(b: i32) {
    let x = lhu(b);
    let y = lhu(b + 2);
    let x1 = x + lhu(b + 4);
    let y1 = y + lhu(b + 6);
    dlFillRectangle(x, y, x1, y1, 0x3480);
    let left = x + 1;
    dlFillRectangle(x, y, x, y1, 0x385c80);
    dlFillRectangle(x1, y, x1, y1, 0x385c80);
    let right = x1 - 1;
    dlFillRectangle(left, y, right, y, 0x385c80);
    dlFillRectangle(left, y1, right, y1, 0x385c80);
    let sound = lw(g::snd + 4);
    let height = lhu(b + 6);
    let line = lhu(b + 2) + height - (height * lh(sound + 6)).wrapping_div(15);
    dlDrawLine(x, line, x1, line, 0x64501e80);
}
