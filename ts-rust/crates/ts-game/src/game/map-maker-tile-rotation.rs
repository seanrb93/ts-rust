//! Making the three turned copies of a map maker tile.

use crate::*;

/// Make the three copies of the tile `tile` turned by a quarter each, one after the other (each linked from the one before at +112; the last link cleared): the width and
/// height are swapped, each layer's bytes are turned (the bits of each byte rotated too), the 80 bytes at +32 are copied, and each plug (5 bytes at +32: the kind, which turns
/// 1 to 2 to 3 to 4 to 1, then the place, turned by the tile's size) is moved to its turned place (@ 0x248fa0).
pub fn mmMakeTileRotations(tile: i32) {
    let mut source = tile;
    let mut turn = 1;
    let mut new;
    loop {
        new = mmAllocTile(lw(source), lw(source + 4), lw(source + 8));
        sw(new, lw(source + 4));
        sw(new + 4, lw(source));
        sw(new + 8, lw(source + 8));
        let layers = lw(source + 8);
        let after = turn + 1;
        let corner = new + 32;
        let from = source + 32;
        let end = source + 96;
        let plug_count;
        if layers <= 0 {
            plug_count = lw(source + 28);
        } else {
            let mut width = lw(source);
            let layers_now = layers;
            let mut layer = 0;
            let mut offset = layer << 2;
            let mut layers_left = layers_now;
            loop {
                let mut x = 0;
                let source_layer = lw(source + 12 + offset);
                let mut destination = lw(new + 12 + offset);
                let next_layer = layer + 1;
                if width > 0 {
                    let mut height = lw(source + 4);
                    loop {
                        let mut y = height - 1;
                        let next_x = x + 1;
                        if y >= 0 {
                            let mut row = y * width;
                            loop {
                                y -= 1;
                                let c = lbu(source_layer + (row + x));
                                sb(destination, c << 1 & 15 | c & 240 | (((c & 8) as u32) >> 3) as i32);
                                destination += 1;
                                if y < 0 {
                                    break;
                                }
                                row = y * lw(source);
                            }
                            width = lw(source);
                        }
                        x = next_x;
                        if x >= width {
                            break;
                        }
                        height = lw(source + 4);
                    }
                    layers_left = lw(source + 8);
                }
                layer = next_layer;
                offset = layer << 2;
                if layer >= layers_left {
                    break;
                }
            }
            plug_count = lw(source + 28);
        }
        sw(new + 28, plug_count);
        let mut src = from;
        let mut dst = corner;
        for _ in 0..2 {
            let b = ld(src + 8);
            let c = ld(src + 16);
            let d = ld(src + 24);
            sd(dst, ld(src));
            sd(dst + 8, b);
            sd(dst + 16, c);
            sd(dst + 24, d);
            src += 32;
            dst += 32;
        }
        debug_assert!(src == end);
        let b = ld(src + 8);
        sd(dst, ld(src));
        sd(dst + 8, b);
        let mut plug = 0;
        if lw(source + 28) > 0 {
            let y_at = source + 34;
            let x_at = source + 33;
            let mut offset = plug << 2;
            loop {
                let at = offset + plug;
                let kind = lbu(from + at);
                let mut next_kind = kind + 1;
                if (kind as u32) >= 4 {
                    next_kind = 1;
                }
                sb(corner + at, next_kind);
                let placed = lbu(source + 4) - lbu(y_at + at);
                let to = new + 33 + at;
                sb(to, placed);
                let mut moved = x_at + at;
                if lbu(from + at) & 1 == 0 {
                    sb(to, placed - lbu(source + at + 36));
                    moved = x_at + at;
                }
                plug += 1;
                sb(new + 34 + at, lbu(moved));
                offset = plug << 2;
                if plug >= lw(source + 28) {
                    break;
                }
            }
        }
        sw(source + 112, new);
        turn = after;
        source = new;
        if turn >= 4 {
            break;
        }
    }
    sw(new + 112, 0);
}
