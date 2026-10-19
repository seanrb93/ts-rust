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
