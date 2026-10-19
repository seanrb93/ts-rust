//! Choosing the map maker objects inside the lasso.

use crate::*;

/// Mark the tile objects that the lasso (from the corner kept in `lasso_x` and `lasso_y` to the mouse) touches with flag 1, and count them in `lassocount`: only tiles of the level
/// being edited are looked at, and an object is marked if one of the cells it fills (a byte not 0 in the tile's layer) is inside the lasso, or the whole tile is. A lasso smaller
/// than 0x40000 on both sides does nothing; otherwise flag 32 is set in `editflags` (@ 0x2458a8).
pub fn mmFindLassoed() {
    let scale_x = lw(g::scale_wtomx);
    let mouse_x = (lh(lw(g::front) + 34) - lw(g::w_x)) << 16;
    let moved_x = FMul(mouse_x, scale_x);
    let scale_y = lw(g::scale_wtomy);
    let mut right = lw(g::m_x) + moved_x as i32;
    let mouse_y = (lh(lw(g::front) + 38) - lw(g::w_y)) << 16;
    let moved_y = FMul(mouse_y, scale_y);
    let corner_x = lw(g::lasso_x);
    let mut bottom = lw(g::m_y) + moved_y as i32;
    let left;
    if right >= corner_x {
        left = corner_x;
    } else {
        left = right;
        right = corner_x;
    }
    let corner_y = lw(g::lasso_y);
    let width = right - left;
    let top;
    if bottom >= corner_y {
        top = corner_y;
    } else {
        top = bottom;
        bottom = corner_y;
    }
    let height = bottom - top;
    if 0x3ffff >= width && 0x3ffff >= height {
        return;
    }
    let mut object = lw(g::mapobjects);
    let flags = lw(g::editflags) | 32;
    sw(g::lassocount, 0);
    sw(g::editflags, flags);
    if object != 0 {
        let level = lw(g::editlevel);
        let mut object_flags = lw(object + 4);
        loop {
            let kind = lw(object);
            sw(object + 4, object_flags & -2);
            if kind == 1 {
                let tile = lw(object + (lw(object + 24) << 2) + 36);
                if ((level - lw(object + 20)) as u32) < (lw(tile + 8) as u32) {
                    let x = lw(object + 12);
                    if x < right {
                        let y = lw(object + 16);
                        if y < bottom {
                            let w = lw(tile);
                            let h = lw(tile + 4);
                            let x_size = w << 21;
                            let x_end = x + x_size;
                            let y_size = h << 21;
                            if x_end >= left {
                                let y_end = y + y_size;
                                if !(y_end < top) {
                                    let mut found = false;
                                    if x >= left && !(y < top) && x_end < right && y_end < bottom {
                                        found = true;
                                    }
                                    if !found {
                                        let first_column = if x >= left { 0 } else { (left - x) >> 21 };
                                        let last_column = if right >= x + x_size { w - 1 } else { (right - x) >> 21 };
                                        let first_row = if y >= top { 0 } else { (top - y) >> 21 };
                                        let last_row = if bottom >= y + y_size { h - 1 } else { (bottom - y) >> 21 };
                                        let mut row = first_row;
                                        if !(last_row < row) {
                                            let mut column = first_column;
                                            'rows: loop {
                                                if last_column < column {
                                                    row += 1;
                                                } else {
                                                    let cells = lw(tile + 12);
                                                    let mut at = row * w;
                                                    loop {
                                                        let cell = lbu(cells + (at + column));
                                                        column += 1;
                                                        if cell != 0 {
                                                            found = true;
                                                            break 'rows;
                                                        }
                                                        at = row * w;
                                                        if last_column < column {
                                                            break;
                                                        }
                                                    }
                                                    row += 1;
                                                }
                                                column = first_column;
                                                if last_row < row {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if found {
                                        let count = lw(g::lassocount);
                                        sw(object + 4, lw(object + 4) | 1);
                                        sw(g::lassocount, count + 1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            object = lw(object + 56);
            if object == 0 {
                break;
            }
            object_flags = lw(object + 4);
        }
    }
}
