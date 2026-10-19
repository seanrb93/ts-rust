//! Checking whether the map maker object being placed overlaps another.

use crate::*;

/// Whether the object in the hand of the map maker (`usedobj`, and the objects after it) can be put at the mouse position: each tile object (kind 1) is placed on the grid
/// (cells of 2^21 units) at the mouse position and must lie within 0x2800000 of the origin; every cell it fills (a byte not 0 in the layer of the tile) must be free (by `mmCellBits`).
/// If a cell is not free, the level and flag 1024 are kept in `warnlevel` and `editflags` to flash it. Returns 1 for a place that is free (also with no object), else 0 (@ 0x245650).
pub fn mmCheckPlacement() -> i32 {
    let saved_level = lw(g::editlevel);
    let mouse_x = lh(lw(g::front) + 34) - lw(g::w_x);
    let scale_x = lw(g::scale_wtomx);
    let moved_x = FMul(mouse_x << 16, scale_x);
    let scale_y = lw(g::scale_wtomy);
    let mouse_y = lh(lw(g::front) + 38) - lw(g::w_y);
    let at_x = lw(g::m_x) + moved_x as i32 + 0x100000;
    let moved_y = FMul(mouse_y << 16, scale_y);
    let mut object = lw(g::usedobj);
    let at_y = lw(g::m_y) + moved_y as i32 + 0x100000;
    if object != 0 {
        let mut kind = lw(object);
        loop {
            if kind != 1 {
                object = lw(object + 56);
            } else {
                let x = (at_x + lw(object + 12)) & -0x200000;
                let tile = lw(object + (lw(object + 24) << 2) + 36);
                let y = (at_y + lw(object + 16)) & -0x200000;
                if x < -0x2800000 {
                    sw(g::editlevel, saved_level);
                    return 0;
                }
                if 0x2800000 < x.wrapping_add(lw(tile) << 21) || y < -0x2800000 || 0x2800000 < y.wrapping_add(lw(tile + 4) << 21) {
                    sw(g::editlevel, saved_level);
                    return 0;
                }
                let mut layer = 0;
                if lw(tile + 8) > 0 {
                    let layers = tile + 12;
                    loop {
                        let mut width = lw(tile);
                        let mut cell = lw(layers + (layer << 2));
                        let mut column = 0;
                        let mut row = 0;
                        let mut left = width * lw(tile + 4) - 1;
                        sw(g::editlevel, saved_level + lw(object + 20) + layer);
                        if left != -1 {
                            loop {
                                if lbu(cell) != 0 {
                                    let free = mmCellBits(x + (column << 21), y + (row << 21));
                                    if free != 0 {
                                        let flags = lw(g::editflags) | 1024;
                                        sw(g::warnlevel, lw(g::editlevel));
                                        sw(g::editflags, flags);
                                        sw(g::editlevel, saved_level);
                                        return 0;
                                    }
                                    width = lw(tile);
                                }
                                column += 1;
                                left -= 1;
                                if column == width {
                                    column = 0;
                                    row += 1;
                                }
                                cell += 1;
                                if left == -1 {
                                    break;
                                }
                            }
                        }
                        layer += 1;
                        if layer >= lw(tile + 8) {
                            break;
                        }
                    }
                }
                object = lw(object + 56);
            }
            if object == 0 {
                break;
            }
            kind = lw(object);
        }
    }
    sw(g::editlevel, saved_level);
    1
}
