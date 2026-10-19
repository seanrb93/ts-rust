//! Display list packets that draw shapes: a cleared rectangle and a line.

use crate::*;

/// Write a GIF packet to the display list: `size` bytes from the debug allocator, a first word `header`, then `words` (the rest of the packet),
/// with a DMA tag of `quadwords` 16-byte units pointing at it.
fn push_packet(size: i32, header: i32, quadwords: i32, words: &[i32]) {
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
