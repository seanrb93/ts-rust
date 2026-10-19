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

/// Draw a slider at (`x`, `y`): a frame, the filled part for `value` of `range`, and the rest, with the sizes taken from the window (width +12, height +32)
/// scaled by constants of the game; the unfilled part is darkened unless `hide_rest` is set (@ 0x22d0e0).
pub fn draw_slider(x: i32, y: i32, value: i32, range: i32, hide_rest: i32) {
    let scale_x = lwc1(0x3ac984);
    let window = g::window;
    let width = lw(window + 12) as f32;
    let bottom = y + (lw(window + 32) as f32 * lwc1(0x3ac988)) as i32;
    let right = x + (width * scale_x) as i32;
    let inner_bottom = bottom - 1;
    let inner_right = right - 1;
    let split = x + (value as f32 * scale_x / range as f32 * width) as i32;
    dlFillRectangle(x - 2, y - 1, right + 1, bottom, 64);
    dlFillRectangle(x, y, split - 1, inner_bottom, 0x287f147f);
    dlFillRectangle(split, y, inner_right, inner_bottom, 0x2840147f);
    if hide_rest != 0 {
        return;
    }
    dlFillRectangle(x, y, inner_right, inner_bottom, 64);
}

/// Draw a meter at (`x`, `y`), `width` by `height`: a frame, the part up to the fraction `fraction_a` in `colour`, and from there to `fraction_b` in a
/// colour whose strength is the byte `strength` (when it is positive), the rest dark (@ 0x2b2c10).
pub fn hudMeterGfx(x: i32, y: i32, width: i32, height: i32, colour: i32, strength: i32, fraction_a: f32, fraction_b: f32) {
    let w = width as f32;
    let bottom = y + height;
    let half = 0.5_f32;
    let right = x + width;
    let first = x + (w * fraction_a) as i32;
    let second = x + (w * fraction_b) as i32;
    let byte = (strength << 24) >> 24;
    dlFillRectangleFloat(79, (x - 1) as f32, y as f32 - half, right as f32, bottom as f32 - half);
    let inner_bottom = bottom - 1;
    dlFillRectangle(x, y, first, inner_bottom, colour);
    let mut from = first;
    if first < second {
        let b = byte & 255;
        if byte > 0 {
            dlFillRectangle(first, y, second, inner_bottom, b << 24 | 127 | (b << 8 | b << 16));
            from = second;
        }
    }
    dlFillRectangle(from, y, right - 1, bottom - 1, 0x28282864);
}

/// Draw a textured rectangle given in floats: the colour 0xrrggbbaa, texture coordinates (`u0`, `v0`) - (`u1`, `v1`) (plus 8), positions (`x0`, `y0`) - (`x1`, `y1`) in 16ths of
/// a pixel; the depth test is switched off for it (@ 0x2b8df8).
pub fn dlTextureRectangleFloat(u0: i32, v0: i32, u1: i32, v1: i32, colour: i32, x0: f32, y0: f32, x1: f32, y1: f32) {
    let packet = memdbAlloc(80);
    let previous = dlSetZB(0);
    sb(lw(g::gfxtag) + 3, 48);
    let begin = lw(g::gfxtag);
    let scale = 16.0_f32;
    sw(begin + 4, g::texrect_begin);
    sh(begin, 2);
    sw(g::gfxtag, begin + 16);
    sb(begin + 16 + 3, 48);
    sw(packet, ((colour as u32) >> 24) as i32);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    // the words of the packet and the stores that come between them are in the order of the original (the packet may overlap what they write)
    let first = [colour >> 16 & 255, colour >> 8 & 255, colour & 255, u0 + 8, v0 + 8, 0, 0];
    for (i, w) in first.iter().enumerate() {
        sw(packet + 4 + 4 * i as i32, *w);
    }
    sh(tag, 5);
    sw(packet + 32, (x0 * scale) as i32 + 0x6c00);
    sw(g::gfxtag, tag + 16);
    let rest = [
        (y0 * scale) as i32 + 0x7900,
        -1,
        0,
        u1 + 8,
        v1 + 8,
        0,
        0,
        (x1 * scale) as i32 + 0x6c00,
        (y1 * scale) as i32 + 0x7900,
        -1,
        0,
    ];
    for (i, w) in rest.iter().enumerate() {
        sw(packet + 36 + 4 * i as i32, *w);
    }
    sb(tag + 16 + 3, 48);
    let end = lw(g::gfxtag);
    sw(end + 4, g::texrect_end);
    sh(end, 1);
    sw(g::gfxtag, end + 16);
    dlSetZB(previous);
}

/// Draw a triangle strip of `count` vertices from `points` (a colour word, x and y as integers, and for a textured strip `textured` u and v: 12 or 20 bytes each) (@ 0x2b8a10).
pub fn dlDrawTriangleStrip(points: i32, count: i32, textured: i32) {
    let per_vertex = if textured != 0 { 3 } else { 2 };
    let quadwords = count * per_vertex + 2;
    let packet = memdbAlloc(quadwords << 4);
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    sw(packet, (quadwords - 1) << 16 | 0x6c008000);
    sw(tag + 4, packet);
    sw(packet + 4, count + 0x8000);
    sw(packet + 8, if textured != 0 { 0x30ae4000 } else { 0x20264000 });
    sw(packet + 12, if textured != 0 { 1299 } else { 81 });
    sw(packet + 16, 0);
    sh(tag, quadwords);
    sw(g::gfxtag, tag + 16);
    let mut cursor = packet + 20;
    let mut src = points;
    let scale = 16.0_f32;
    for _ in 0..count.max(0) {
        let colour = lw(src);
        let x = lw(src + 4) as f32;
        let y = lw(src + 8) as f32;
        src += 12;
        if textured != 0 {
            let u = lw(src);
            let v = lw(src + 4);
            src += 8;
            sw(cursor, u + 8);
            sw(cursor + 4, v + 8);
            sw(cursor + 8, 0);
            sw(cursor + 12, 0);
            cursor += 16;
        }
        sw(cursor, ((colour as u32) >> 24) as i32);
        sw(cursor + 4, ((colour as u32) >> 16) as i32 & 255);
        sw(cursor + 8, ((colour as u32) >> 8) as i32 & 255);
        sw(cursor + 12, colour & 255);
        sw(cursor + 16, (x * scale) as i32 + 0x6c00);
        sw(cursor + 20, (y * scale) as i32 + 0x7900);
        sw(cursor + 24, -1);
        sw(cursor + 28, 0);
        cursor += 32;
    }
    sw(cursor, 0x140007fc);
    sw(cursor + 4, 0x11000000);
    sw(cursor + 8, 0);
}

/// Make the screen the texture of the next drawing: a packet that sets the texture registers for the frame buffer at page `page` (@ 0x295dd0).
pub fn ipSetScreenAsTexture(page: i32) {
    let low = page << 5 | 0x28228000;
    push_packet(
        112,
        0x6c068000,
        7,
        &[
            0x8005, 0x108f4000, 14, 0, 0, 0, 63, 0, 127, 127, 59, 0, 0, 0, 20, 0, low, 2, 6, 0, 0x9fc00a, 892, 8, 0, 0x140007fc, 0x11000000, 0,
        ],
    );
}
