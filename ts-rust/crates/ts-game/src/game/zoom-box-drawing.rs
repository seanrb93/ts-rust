//! Drawing the frame of a zoom box.

use crate::*;

/// Draw a box from (`x0`, `y0`) to (`x1`, `y1`): the inside filled with `fill` (when not 0) and a frame of eight pieces of the texture at 0x3aff18 (four corners and four sides, each
/// of 128 by 128 texture units) 8 pixels wide and 4 high round it, in the colour `colour` (@ 0x22ed18).
pub fn zoomboxDraw(x0: i32, y0: i32, x1: i32, y1: i32, fill: i32, colour: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let right = x1 + 8;
    let left = x0 - 8;
    let bottom = y1 + 4;
    let top = y0 - 4;
    let inner_left = left + 8;
    let inner_top = top + 4;
    let inner_right = right - 8;
    let inner_bottom = bottom - 4;
    if fill != 0 {
        dlFillRectangle(inner_left, inner_top, inner_right, inner_bottom, fill);
    }
    dlSelectTextureKick(lw(0x3aff18));
    sw(sp, colour);
    dlTextureRectangle(left, top, inner_left, inner_top, 0, 0, 128, 128);
    sw(sp, colour);
    dlTextureRectangle(inner_right, top, right, inner_top, 256, 0, 384, 128);
    sw(sp, colour);
    dlTextureRectangle(left, inner_bottom, inner_left, bottom, 0, 256, 128, 384);
    sw(sp, colour);
    dlTextureRectangle(inner_right, inner_bottom, right, bottom, 256, 256, 384, 384);
    sw(sp, colour);
    dlTextureRectangle(inner_left, top, inner_right, inner_top, 128, 0, 256, 128);
    sw(sp, colour);
    dlTextureRectangle(inner_left, inner_bottom, inner_right, bottom, 128, 256, 256, 384);
    sw(sp, colour);
    dlTextureRectangle(left, inner_top, inner_left, inner_bottom, 0, 128, 128, 256);
    sw(sp, colour);
    dlTextureRectangle(inner_right, inner_top, right, inner_bottom, 256, 128, 384, 256);
}
