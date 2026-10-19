//! A look-at matrix from two points, transforming a model, the head of a character, and the floor of a room.

use crate::*;

/// Move all the vertices of the model `skeleton` (both lists of chunks) by the matrix `matrix` (@ 0x260490).
pub fn obTransform(skeleton: i32, matrix: i32) {
    partTransform(skeleton, 0, 0, matrix);
    partTransform(skeleton, 0, 1, matrix);
}

/// Make the view matrix `m` that looks from the point `eye` at the point `target` with the up direction `up` (points of three floats); the last float goes to `matrixLook` on the stack (@ 0x2b5208).
pub fn matrixLookAt(m: i32, eye: i32, target: i32, up: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let ex = lwc1(eye);
    let ez = lwc1(eye + 8);
    let ey = lwc1(eye + 4);
    let dx = lwc1(target) - ex;
    let dz = lwc1(target + 8) - ez;
    let dy = lwc1(target + 4) - ey;
    let uy = lwc1(up + 4);
    let ux = lwc1(up);
    swc1(sp, lwc1(up + 8));
    matrixLook(m, ex, ey, ez, dx, dy, dz, ux, uy);
}

/// The prop a character's head uses: -1 unless the cheat 0x4000 is on or `head` is one of the heads 40 to 49, 60, 61 and 64 (@ 0x2886a0).
pub fn chrGetChrHeadProp(chr: i32, head: i32) -> i32 {
    if (lw(g::allowedcheats) & 0x4000 & (lw(g::unlockedcheats) & lw(g::selectedcheats))) == 0 {
        let offset = head - 40;
        if (offset as u32) < 25 {
            match offset {
                0..=9 | 20 | 21 | 24 => {}
                _ => return -1,
            }
        } else {
            return -1;
        }
    }
    chrGetChrHeadPropUse(chr)
}

/// The height of the floor under the point `point` in the room `room`, the higher of the room's own floors and its props' floors; the floor item is written to `out_item` and the
/// prop (0 for the room's own) to `out_prop`, when not 0 (@ 0x27c798).
pub fn moveFindFloorRoom(room: i32, point: i32, out_item: i32, out_prop: i32) -> f32 {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let floors = bgGetFloors(room);
    let props = lw(g::roomprops);
    sw(sp, 0);
    let mask = lhu(lw(lw((room << 2) + props) + 32) + 292);
    let mut best = moveFindFloorHeight(floors, mask, point, sp);
    let prop_height = propFindFloorHeightRoom(room, point, sp | 4, sp | 8);
    let found = lw(sp + 4);
    if !(best < prop_height) {
        if out_prop != 0 {
            sw(out_prop, 0);
        }
    } else {
        best = prop_height;
        sw(sp, found);
        if out_prop != 0 {
            sw(out_prop, lw(sp + 8));
        }
    }
    let item = lw(sp);
    if out_item != 0 {
        sw(out_item, item);
    }
    best
}
