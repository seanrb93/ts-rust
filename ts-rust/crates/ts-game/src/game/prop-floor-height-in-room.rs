//! The floor height under a point from the props of a room.

use crate::*;

/// The height of the highest floor under the point `point` among the props of a room (`room`, or any with kind 256) that have flag 32 and a list of floors at +480 (two lists, the
/// first of the 0x3b00e8 props at 0x1fd2f60, the second of the 0x3b00ec props at 0x1fd3410): by `moveFindFloorHeight`, whose item and the prop are written to `out_item` and
/// `out_prop` (when not 0) (@ 0x2714e0).
pub fn propFindFloorHeightRoom(room: i32, point: i32, out_item: i32, out_prop: i32) -> f32 {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let mut pass = 0;
    let mut best_item = 0;
    let mut best_prop = 0;
    let mut best = lwc1(0x3af93c);
    loop {
        let (count, table) = if pass != 0 { (lw(0x3b00ec), 0x1fd3410) } else { (lw(0x3b00e8), 0x1fd2f60) };
        if count > 0 {
            let mut i = 0;
            loop {
                let prop = lw((i << 2) + table);
                if prop != 0 && lw(prop + 16) & 32 != 0 && (lw(prop + 12) == room || lw(prop + 8) == 256) {
                    let floors = lw(prop + 480);
                    if floors != 0 {
                        let height = moveFindFloorHeight(floors, 0, point, sp);
                        if best < height {
                            best_item = lw(sp);
                            best = height;
                            best_prop = prop;
                        }
                    }
                }
                i += 1;
                if i >= count {
                    break;
                }
            }
        }
        pass += 1;
        if pass >= 2 {
            break;
        }
    }
    if out_item != 0 {
        sw(out_item, best_item);
    }
    if out_prop != 0 {
        sw(out_prop, best_prop);
    }
    best
}
