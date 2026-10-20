//! The drawing of a tile of the map maker's map.

use crate::*;

/// Draw the tile `tile` (the width and height in cells at +0 and +4, the number of levels at +8, the cells of each level through the table from +12, the number of doors at +28 and
/// the doors from +32, 5 bytes each: the direction, x, y, level and size) of the object `object` at (`x`, `y`) in the map maker's fixed point (0x200000 to a cell) in the colours of
/// the draw mode `mode` (1 to 5 lit in different colours; 6 and more plain): all the cells that exist are filled, then the walls of the edit level's cells (a wall is drawn where
/// the cell flag says there is one and no neighbour has it), then the doors that are on that level (a door already drawn as a plug of the cell is left out), and for the edit modes
/// 1 and 2 the items of the object or a marker box in the middle of the tile (@ 0x24a008).
pub fn mmDrawTile(x: i32, y: i32, tile: i32, mode: i32, object: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    sw(sp + 8, mode);
    let kind = mode - 1;
    sw(sp, x);
    sw(sp + 4, y);
    sw(sp + 12, object);
    let fill;
    let edge;
    if (kind as u32) >= 6 {
        fill = 0x64646464;
        edge = 0x2828285a;
    } else {
        let flash;
        let (high, low);
        match kind {
            0 => {
                flash = lbu(g::quickflash);
                high = 0x507f5000;
                low = 0x1e321e00;
            }
            1 => {
                flash = lbu(g::flashalpha);
                high = 0x3c7f3c00;
                low = 0x143c1400;
            }
            2 => {
                flash = lbu(g::quickflash);
                high = 0x7f3c3c00;
                low = 0x3c141400;
            }
            3 => {
                flash = lbu(g::quickflash);
                high = 0x7f0000 | 0x7f00;
                low = 0x3f3f00;
            }
            4 => {
                flash = lbu(g::flashalpha);
                high = 0x647f0000 | 0x7f00;
                low = 0x3f3f00;
            }
            _ => {
                fill = 0x283c3c5a;
                edge = 0x3c3c3c;
                return mm_draw_tile_rest(sp, x, y, tile, fill, edge);
            }
        }
        fill = flash | high;
        edge = flash | low;
    }
    mm_draw_tile_rest(sp, x, y, tile, fill, edge);
}

/// The rest of `mmDrawTile`, with the colours chosen: `fill` for the walls and `edge` for the cells.
fn mm_draw_tile_rest(sp: i32, _x: i32, _y: i32, tile: i32, fill: i32, edge: i32) {
    let width = lw(tile);
    let mut index = 0;
    let height = lw(tile + 4);
    let mut across = width;
    let mut cell_x = lw(sp);
    let mut cell_y = lw(sp + 4);
    let mut cells = lw(tile + 12);
    let total = width * height;
    sw(sp + 28, tile + 12);
    let mut width_now = width;
    let mut height_now = height;
    if total > 0 {
        loop {
            let value;
            if across != 0 {
                value = lbu(cells);
            } else {
                across = lw(tile);
                cell_x = lw(sp);
                cell_y += 0x200000;
                value = lbu(cells);
            }
            if value != 0 {
                mmDrawRect(cell_x, cell_y, cell_x + 0x200000, cell_y + 0x200000, edge);
            }
            index += 1;
            cell_x += 0x200000;
            cells += 1;
            across -= 1;
            if index >= total {
                break;
            }
        }
        width_now = lw(tile);
        height_now = lw(tile + 4);
    }
    let mut across = width_now;
    let level = lw(g::editlevel) - lw(lw(sp + 12) + 20);
    sw(sp + 16, height_now);
    let mut cell_x = lw(sp);
    let levels = lw(tile + 8);
    let count = across * height_now;
    let mut cell_y = lw(sp + 4);
    let mut cells = lw(lw(sp + 28) + ((if (level as u32) >= (levels as u32) { levels - 1 } else { level }) << 2));
    'walls: {
        if count > 0 {
            let mut remaining = count;
            sw(sp + 20, 0x50000);
            sw(sp + 24, 0x1a0000);
            sw(sp + 20, 0x55555);
            sw(sp + 24, 0x1aaaab);
            loop {
                let flags = lbu(cells);
                if across == 0 {
                    cell_y += 0x200000;
                    across = lw(tile);
                    cell_x = lw(sp);
                    sw(sp + 16, lw(sp + 16) - 1);
                }
                'cell: {
                    if flags != 0 {
                        let mut mask = flags & 2;
                        'top: {
                            if (flags & 1) == 0 {
                                if cell_y != lw(sp + 4) && lbu(cells - lw(tile)) != 0 {
                                    break 'top;
                                }
                                let thickness = lw(sp + 20);
                                mmDrawRect(cell_x, cell_y, cell_x + 0x200000, cell_y + thickness, fill);
                                mask = flags & 2;
                            }
                        }
                        let mut right_mask = flags & 4;
                        'right: {
                            if mask == 0 {
                                if across != 1 && lbu(cells + 1) != 0 {
                                    break 'right;
                                }
                                let start = lw(sp + 24);
                                mmDrawRect(cell_x + start, cell_y, cell_x + 0x200000, cell_y + 0x200000, fill);
                                right_mask = flags & 4;
                            }
                        }
                        let mut bottom_mask = flags & 8;
                        'bottom: {
                            if right_mask == 0 {
                                if lw(sp + 16) != 1 && lbu(cells + lw(tile)) != 0 {
                                    break 'bottom;
                                }
                                let start = lw(sp + 24);
                                mmDrawRect(cell_x, cell_y + start, cell_x + 0x200000, cell_y + 0x200000, fill);
                                bottom_mask = flags & 8;
                            }
                        }
                        if bottom_mask != 0 {
                            remaining -= 1;
                            break 'cell;
                        }
                        let thickness = lw(sp + 20);
                        if across != lw(tile) && lbu(cells - 1) != 0 {
                            remaining -= 1;
                            break 'cell;
                        }
                        mmDrawRect(cell_x, cell_y, cell_x + thickness, cell_y + 0x200000, fill);
                    }
                    remaining -= 1;
                }
                cells += 1;
                across -= 1;
                cell_x += 0x200000;
                if remaining == 0 {
                    break 'walls;
                }
            }
        }
    }
    let doors = lw(tile + 28);
    let mut done = 0;
    if doors > 0 {
        let half = 0x2aaaa;
        let behind = -0x2aaaa;
        let mut door = tile + 32;
        loop {
            'door: {
                let door_x = lw(sp) + (lbu(door + 1) << 21);
                let door_y = lw(sp + 4) + (lbu(door + 2) << 21);
                if (lw(g::editlevel) - lw(lw(sp + 12) + 20)) != lbu(door + 3) {
                    done += 1;
                    break 'door;
                }
                let colour = if (lbu(door + 4) ^ 1) == 0 { 0x781e1e64 } else { 0xb07864 };
                let direction = lbu(door);
                let plug_level = lw(lw(sp + 12) + 20) + lbu(door + 3);
                let size = lbu(door + 4) << 21;
                match direction {
                    1 => {
                        let bits = mmCellBits(door_x, door_y - 0x200000);
                        if (bits & 4) != 0 && mmFindPlug(lw(g::mmcellobject), door, door_x >> 21, door_y >> 21, plug_level) != 0 {
                            done += 1;
                            break 'door;
                        }
                        mmDrawRect(door_x + 0x80000, door_y, door_x + size - 0x80000, door_y + half, colour);
                    }
                    2 => {
                        let bits = mmCellBits(door_x, door_y);
                        if (bits & 8) != 0 && mmFindPlug(lw(g::mmcellobject), door, door_x >> 21, door_y >> 21, plug_level) != 0 {
                            done += 1;
                            break 'door;
                        }
                        mmDrawRect(door_x + behind, door_y + 0x80000, door_x, door_y + size - 0x80000, colour);
                    }
                    3 => {
                        let bits = mmCellBits(door_x, door_y);
                        if (bits & 1) != 0 && mmFindPlug(lw(g::mmcellobject), door, door_x >> 21, door_y >> 21, plug_level) != 0 {
                            done += 1;
                            break 'door;
                        }
                        mmDrawRect(door_x + 0x80000, door_y + behind, door_x + size - 0x80000, door_y, colour);
                    }
                    4 => {
                        let bits = mmCellBits(door_x - 0x200000, door_y);
                        if (bits & 2) != 0 && mmFindPlug(lw(g::mmcellobject), door, door_x >> 21, door_y >> 21, plug_level) != 0 {
                            done += 1;
                            break 'door;
                        }
                        mmDrawRect(door_x, door_y + 0x80000, door_x + half, door_y + size - 0x80000, colour);
                    }
                    _ => {
                        done += 1;
                        break 'door;
                    }
                }
                done += 1;
            }
            door += 5;
            if done >= lw(tile + 28) {
                break;
            }
        }
    }
    if lw(lw(g::front) + 20) == 0 {
        let editing = lw(g::editmode);
        if editing == 1 {
            let object = lw(sp + 12);
            if lw(object + 52) != 0 {
                mmDrawItems(object);
                return;
            }
        }
        if editing == 2 && lw(sp + 8) != 6 {
            let top = lw(sp + 4) + ((lw(tile + 4) << 21) >> 1) - 0x80000;
            let left = lw(sp) + ((lw(tile) << 21) >> 1) - 0x80000;
            mmDrawRect(left, top, left + 0x100000, top + 0x100000, 128);
            let inner = lw(lw(sp + 12) + 32);
            mmDrawRect(left + 0x20000, top + 0x20000, left + 0xe0000, top + 0xe0000, inner);
        }
    }
}
