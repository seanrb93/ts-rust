//! Positions in the map maker: centring the objects and finding the plug of a tile that touches a point.

use crate::*;

/// Move all the map maker's objects (a list through +56 from `usedobj`, if its first is of kind 1) so that the middle of their bounding box is the origin,
/// rounded to whole tiles (2^21 units) (@ 0x245ba8).
pub fn mmCenterFlyingObj() {
    let head = lw(g::usedobj);
    if head == 0 || lw(head) != 1 {
        return;
    }
    let (mut min_y, mut min_x) = (0x7fffffff, 0x7fffffff);
    let (mut max_y, mut max_x) = (-0x80000000_i32, -0x80000000_i32);
    let mut object = head;
    loop {
        let x = lw(object + 12);
        let y = lw(object + 16);
        let tile = lw(object + (lw(object + 24) << 2) + 36);
        min_x = if x < min_x { x } else { min_x };
        min_y = if y < min_y { y } else { min_y };
        object = lw(object + 56);
        let far_y = y.wrapping_add(lw(tile + 4) << 21);
        let far_x = x.wrapping_add(lw(tile) << 21);
        max_y = if max_y < far_y { far_y } else { max_y };
        max_x = if max_x < far_x { far_x } else { max_x };
        if object == 0 {
            break;
        }
    }
    let center_y = (max_y.wrapping_add(min_y) >> 1) & -0x200000;
    let center_x = (max_x.wrapping_add(min_x) >> 1) & -0x200000;
    let mut object = head;
    while object != 0 {
        let y = lw(object + 16) - center_y;
        sw(object + 12, lw(object + 12) - center_x);
        sw(object + 16, y);
        object = lw(object + 56);
    }
}

/// Find the plug (5 bytes: kind at +0, the position at +1, +2, +3, a value at +4) of the tile of `object` that matches the plug `plug` and lies at (`x`, `y`, `z`) in tiles;
/// while the object is being dragged (flag 16) its position is taken from the mouse. 0 if there is none (@ 0x249e80).
pub fn mmFindPlug(object: i32, plug: i32, x: i32, y: i32, z: i32) -> i32 {
    let tile = lw(object + (lw(object + 24) << 2) + 36);
    let count = lw(tile + 28);
    let mut at = tile + 32;
    let (px, py, pz);
    if lw(g::editflags) & 16 == 0 {
        pz = lw(object + 20);
        px = lw(object + 12) >> 21;
        py = lw(object + 16) >> 21;
    } else {
        let front = lw(g::front);
        let scale_x = lw(g::scale_wtomx);
        let scale_y = lw(g::scale_wtomy);
        let mouse_x = (lh(front + 34) - lw(g::w_x)) << 16;
        let mouse_y = (lh(front + 38) - lw(g::w_y)) << 16;
        px = (lw(object + 12) + (lw(g::m_x) + FMul(mouse_x, scale_x) as i32 + 0x100000)) >> 21;
        pz = lw(g::editlevel) + lw(object + 20);
        py = (lw(object + 16) + (lw(g::m_y) + FMul(mouse_y, scale_y) as i32 + 0x100000)) >> 21;
    }
    let mut left = count - 1;
    if left != -1 {
        let kind = lbu(plug);
        loop {
            if (lbu(at) ^ kind) & 1 != 0 || px + lbu(at + 1) != x {
                left -= 1;
            } else {
                left -= 1;
                if py + lbu(at + 2) == y && pz + lbu(at + 3) == z && lbu(at + 4) == lbu(plug + 4) {
                    return at;
                }
            }
            at += 5;
            if left == -1 {
                break;
            }
        }
    }
    0
}

/// The cell of the map maker at (`x`, `y`): the object under the point (or, while an object is carried, the one under the mouse, flag 0x800000), and the byte of its tile
/// at that cell; 0 when there is none.  Flag 16 says the cell came from the mouse (@ 0x2452e8).
pub fn mmCellBits(x: i32, y: i32) -> i32 {
    let object = mmCheckObject(x, y);
    let flags = lw(g::editflags);
    if object != 0 {
        let tile = lw(object + (lw(object + 24) << 2) + 36);
        let cells = lw(tile + ((lw(g::editlevel) - lw(object + 20)) << 2) + 12);
        let index = ((y - lw(object + 16)) >> 21) * lw(tile) + ((x - lw(object + 12)) >> 21);
        sw(g::editflags, lw(g::editflags) & -17);
        sw(g::mmcellobject, object);
        return lbu(cells + index);
    }
    if flags & 0x800000 != 0 {
        let front = lw(g::front);
        if lw(g::usedobj) != 0 {
            let scale_x = lw(g::scale_wtomx);
            let mouse_x = (lh(front + 34) - lw(g::w_x)) << 16;
            let moved_x = FMul(mouse_x, scale_x);
            let scale_y = lw(g::scale_wtomy);
            let view_x = lw(g::m_x) + moved_x as i32 + 0x100000;
            let mouse_y = (lh(lw(g::front) + 38) - lw(g::w_y)) << 16;
            let moved_y = FMul(mouse_y, scale_y);
            let mut candidate = lw(g::usedobj);
            let view_y = lw(g::m_y) + moved_y as i32 + 0x100000;
            while candidate != 0 {
                if lw(candidate) == 1 {
                    let level = 0 - lw(candidate + 20);
                    let tile = lw(candidate + (lw(candidate + 24) << 2) + 36);
                    if (level as u32) < (lw(tile + 8) as u32) {
                        let width = lw(tile);
                        let dx = x - (view_x + lw(candidate + 12) & -0x200000);
                        let height = lw(tile + 4) << 21;
                        let top = view_y + lw(candidate + 16) & -0x200000;
                        if (dx as u32) < ((width << 21) as u32) {
                            let dy = y - top;
                            if (dy as u32) < (height as u32) {
                                let cells = lw(tile + (level << 2) + 12);
                                sw(g::editflags, lw(g::editflags) | 16);
                                let index = (dy >> 21) * width + (dx >> 21);
                                sw(g::mmcellobject, candidate);
                                return lbu(cells + index);
                            }
                        }
                    }
                }
                candidate = lw(candidate + 56);
            }
        }
    }
    sw(g::mmcellobject, 0);
    0
}
