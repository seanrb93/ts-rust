//! Display list packets that draw shapes: a cleared rectangle and a line.

use crate::*;

/// Write a GIF packet to the display list: `size` bytes from the debug allocator, a first word `header`, then `words` (the rest of the packet),
/// with a DMA tag of `quadwords` 16-byte units pointing at it.
pub(super) fn push_packet(size: i32, header: i32, quadwords: i32, words: &[i32]) {
    let packet = memdbAlloc(size);
    sb(lw(g::gfxtag) + 3, 48);
    sw(packet, header);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    for (i, w) in words.iter().enumerate() {
        sw(packet + 4 + 4 * i as i32, *w);
    }
    sw(g::gfxtag, tag + 16);
    sh(tag, quadwords);
}

/// Clear the rectangle (x0, y0) - (x1, y1) of the frame buffer to a colour (a sprite with the colour, byte-swapped, and a depth of 1) (@ 0x2b8178).
pub fn dlClearFB(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32) {
    push_packet(
        80,
        0x6c048000,
        5,
        &[
            0x8001,
            0x30274000,
            3822,
            0,
            ((colour as u32) >> 24) as i32 | colour << 24 | ((colour >> 8 & 255) << 16 | colour >> 8 & 0xff00),
            0x3f800000,
            1,
            0,
            (x0 << 4) + 0x6c00 & 0xffff | ((y0 << 4) + 0x7900) << 16,
            0,
            5,
            0,
            (x1 << 4) + 0x6c10 & 0xffff | ((y1 << 4) + 0x7910) << 16,
            0,
            5,
            0,
            0x140007fc,
            0x11000000,
            0,
        ],
    );
}

/// Draw a line from (x0, y0) to (x1, y1) in a colour 0xrrggbbaa (@ 0x2b8898).
pub fn dlDrawLine(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32) {
    push_packet(
        80,
        0x6c048000,
        5,
        &[
            0x8001,
            0x3020c000,
            1301,
            0,
            (x0 << 4) + 0x6c00,
            (y0 << 4) + 0x7900,
            -1,
            0,
            ((colour as u32) >> 24) as i32,
            colour >> 16 & 255,
            colour >> 8 & 255,
            colour & 255,
            (x1 << 4) + 0x6c00,
            (y1 << 4) + 0x7900,
            -1,
            0,
            0x140007fc,
            0x11000000,
            0,
        ],
    );
}

/// Fill the rectangle (x0, y0) - (x1, y1) with a colour (@ 0x2b8420).
pub fn dlFillRectangle(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32) {
    push_packet(
        80,
        0x6c048000,
        5,
        &[
            0x8003,
            0x10274000,
            14,
            0,
            ((colour as u32) >> 24) as i32 | colour << 24 | ((colour >> 8 & 255) << 16 | colour >> 8 & 0xff00),
            0x3f800000,
            1,
            0,
            (x0 << 4) + 0x6c00 & 0xffff | ((y0 << 4) + 0x7900) << 16,
            -1,
            5,
            0,
            (x1 << 4) + 0x6c10 & 0xffff | ((y1 << 4) + 0x7910) << 16,
            -1,
            5,
            0,
            0x140007fc,
            0x11000000,
            0,
        ],
    );
}

/// Fill a rectangle given in floats (16 sub-pixels to a pixel) with a colour 0xrrggbbaa (@ 0x2b85c8).
pub fn dlFillRectangleFloat(colour: i32, x0: f32, y0: f32, x1: f32, y1: f32) {
    let scale = 16.0_f32;
    push_packet(
        80,
        0x6c048000,
        5,
        &[
            0x8001,
            0x30274000,
            1361,
            0,
            ((colour as u32) >> 24) as i32,
            colour >> 16 & 255,
            colour >> 8 & 255,
            colour & 255,
            (x0 * scale) as i32 + 0x6c00,
            (y0 * scale) as i32 + 0x7900,
            -1,
            0,
            (x1 * scale) as i32 + 0x6c00,
            (y1 * scale) as i32 + 0x7900,
            -1,
            0,
            0x140007fc,
            0x11000000,
            0,
        ],
    );
}

/// Draw a box outline of a one pixel line in `border` colour, filled with `fill` when it is not 0 (@ 0x22eae0).
pub fn guiDrawBox(x0: i32, y0: i32, x1: i32, y1: i32, border: i32, fill: i32) {
    if fill != 0 {
        dlFillRectangle(x0, y0, x1, y1, fill);
    }
    let left = x0 + 1;
    dlFillRectangle(x0, y0, left, y1, border);
    let right = x1 - 1;
    dlFillRectangle(right, y0, x1, y1, border);
    dlFillRectangle(left, y0, right, y0, border);
    dlFillRectangle(left, y1, right, y1, border);
}

/// Invert the picture inside the main window: a packet that sets the blending (@ 0x2961f0), then a filled rectangle over the window.
pub fn ipInvertScreen() {
    push_packet(
        48,
        0x6c028000,
        3,
        &[0x8001, 0x108f4000, 14, 0, 132, 0, 66, 0, 0x140007fc, 0x11000000, 0],
    );
    let window = g::window;
    dlFillRectangle(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24), -129);
}

/// Draw the bar of an amount (the fraction `amount` of the width `width` filled) at (`x`, `y`), `height` high: filled in a colour that goes from green to red and flashes when low,
/// the rest in a dark one (@ 0x2b1328).
pub fn hudAmountBarGfx(x: i32, y: i32, width: i32, height: i32, amount: f32) {
    let level = (amount * 128.0_f32) as i32;
    let filled_to = x + (amount * width as f32) as i32;
    let colour = if level < 41 && (lw(g::tickframenum) & 16) == 0 {
        0x64646460
    } else {
        (128 - level) << 24 | ((level >> 1) + 64 & 255) << 16 | 96
    };
    let bottom = y + height - 1;
    dlFillRectangle(x, y, filled_to - 1, bottom, colour);
    dlFillRectangle(filled_to, y, x + width - 1, bottom, 64);
}

/// Fill the colour box of the map maker's colour dialog (the box is 24 bytes: x, y, width, height halfwords and the colour number at +8, from 64);
/// the chosen colour gets a grey frame (@ 0x251e70).
pub fn draw_colourbox(b: i32) {
    let number = lhu(b + 8);
    let mut index = number - 64;
    if lw(g::curlightcol) == number - 64 {
        let (x, y) = (lhu(b), lhu(b + 2));
        dlFillRectangle(x - 2, y - 1, x + lhu(b + 4) + 2, y + lhu(b + 6) + 1, 0x7f7f7f80);
        index = lhu(b + 8) - 64;
    }
    let (x, y) = (lhu(b), lhu(b + 2));
    let colour = lw((index << 2) + 0x1fc4040);
    dlFillRectangle(x, y, x + lhu(b + 4), y + lhu(b + 6), colour);
}

/// Draw the rectangle (x0, y0) - (x1, y1) of the map maker's world in a colour: moved by the view and scaled to the window with fixed point numbers, then
/// filled as floats (@ 0x249340).
pub fn mmDrawRect(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32) {
    let view_x = lw(g::m_x);
    let window_x = lw(g::w_x) as f32;
    let scale_x = lw(g::scale_mtowx);
    let window_y = lw(g::w_y) as f32;
    let (dx1, view_y) = (x1 - view_x, lw(g::m_y));
    let scale_y = lw(g::scale_mtowy);
    let (dy1, dy0, dx0) = (y1 - view_y, y0 - view_y, x0 - view_x);
    let unit = 1.5258789e-05_f32;
    let fx0 = window_x + FMul(dx0, scale_x) as i32 as f32 * unit;
    let fy1 = window_y + FMul(dy1, scale_y) as i32 as f32 * unit;
    let fx1 = window_x + FMul(dx1, scale_x) as i32 as f32 * unit;
    let fy0 = window_y + FMul(dy0, scale_y) as i32 as f32 * unit;
    dlFillRectangleFloat(colour, fx0, fy0, fx1, fy1);
}

/// Shade the rest of a menu page that is not the top one: fills the rectangle kept at 0x1fc39f0 (x0, y0, x1, y1) with a translucent colour (@ 0x24eef8).
pub fn menu_pageDrawAfter(page: i32) {
    let front = lw(g::front);
    if ((page - lw(front + 4)) >> 4) == lw(front + 20) - 1 {
        return;
    }
    let rect = 0x1fc39f0;
    dlFillRectangle(lw(rect), lw(rect + 4), lw(rect + 8), lw(rect + 12), 0x2840);
}
