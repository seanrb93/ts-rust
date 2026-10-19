//! The centre of a part of a model, and the boxes of the rooms of a level.

use crate::*;

/// Work out the centre of the box that holds the vertices of the first list of chunks of bone `bone` of the model `model`, in the place the bone's matrix puts them (the matrix of
/// the bone itself, or of its parent or of its other parent by the vertex's weight byte when flag byte 0 of the bone is set) and write it to `out` (@ 0x285770).
pub fn FindPartCenterPos(model: i32, bone: i32, out: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let skeleton = lw(model);
    let bones = skeleton - lw(skeleton) * 80;
    let this = bones + bone * 80;
    let parent_a_index = lb(this + 2);
    let parent_a = if parent_a_index < 0 { 0 } else { bones + parent_a_index * 80 };
    let parent_b_index = lb(this + 3);
    let parent_b = if parent_b_index < 0 { 0 } else { parent_b_index * 80 + bones };
    let mut chunk = lw(this + 8);
    let arrays = this + 20;
    let (mut low, mut high) = ([0.0_f32; 3], [0.0_f32; 3]);
    if chunk != 0 {
        let big = lwc1(0x3afa3c);
        let small = lwc1(0x3afa40);
        low = [big; 3];
        high = [small; 3];
        if lw(chunk + 20) >= 0 {
            let mut first = lw(chunk + 12);
            loop {
                let mut k = 0;
                let mut count = lw(chunk + 16);
                let mut weights = lw(arrays + 16) + (first << 2);
                let mut vertex = lw(arrays + 4) + first * 12;
                if count <= 0 {
                    chunk += 24;
                } else {
                    let next_chunk = chunk + 24;
                    loop {
                        swc1(sp + 32, lwc1(vertex));
                        swc1(sp + 36, lwc1(vertex + 4));
                        swc1(sp + 40, lwc1(vertex + 8));
                        let matrix_bone;
                        let mut transform = true;
                        if lb(this) != 0 {
                            let weight = lbu(weights);
                            if weight == 1 {
                                matrix_bone = lb(parent_b + 1);
                            } else if weight >= 2 {
                                if weight != 2 {
                                    transform = false;
                                    matrix_bone = 0;
                                } else {
                                    matrix_bone = lb(this + 1);
                                }
                            } else {
                                matrix_bone = lb(parent_a + 1);
                            }
                        } else {
                            matrix_bone = lb(this + 1);
                        }
                        if transform {
                            matrixVecMul(lw(model + 4) + (matrix_bone << 6), sp + 32);
                            count = lw(chunk + 16);
                        }
                        let v = [lwc1(sp + 32), lwc1(sp + 36), lwc1(sp + 40)];
                        for axis in 0..3 {
                            if v[axis] < low[axis] {
                                low[axis] = v[axis];
                            }
                        }
                        for axis in 0..3 {
                            if high[axis] < v[axis] {
                                high[axis] = v[axis];
                            }
                        }
                        k += 1;
                        vertex += 12;
                        weights += 4;
                        if k >= count {
                            break;
                        }
                    }
                    chunk = next_chunk;
                }
                if lw(chunk + 20) < 0 {
                    break;
                }
                first = lw(chunk + 12);
            }
        }
    }
    let half = 0.5_f32;
    let x = low[0] + (high[0] - low[0]) * half;
    let y = low[1] + (high[1] - low[1]) * half;
    let z = low[2] + (high[2] - low[2]) * half;
    swc1(out, x);
    swc1(out + 4, y);
    swc1(out + 8, z);
}

/// Widen the box at `rec` (the smallest at +0, the largest at +12) to take in the point at `p` (floats) (the bound read again after it is written, as in the game).
fn widen_box(rec: i32, p: i32) {
    for axis in 0..3 {
        let at = 4 * axis;
        let mut v = lwc1(p + at);
        if v < lwc1(rec + at) {
            swc1(rec + at, v);
            v = lwc1(p + at);
        }
        if lwc1(rec + 12 + at) < v {
            swc1(rec + 12 + at, v);
        }
    }
}

/// Widen the bounds of one axis (the smallest at `low_at`, the largest at `high_at`): the first candidate for the smallest, then the first for the largest, then the second of each.
fn widen_axis(low_at: i32, high_at: i32, low_a: i32, high_a: i32, low_b: i32, high_b: i32) {
    let mut low = lwc1(low_at);
    let v = lwc1(low_a);
    if v < low {
        swc1(low_at, v);
        low = v;
    }
    let mut high = lwc1(high_at);
    let w = lwc1(high_a);
    if high < w {
        swc1(high_at, w);
        high = w;
    }
    let v = lwc1(low_b);
    if v < low {
        swc1(low_at, v);
    }
    let w = lwc1(high_b);
    if high < w {
        swc1(high_at, w);
    }
}

/// Work out the two boxes of each room of the level (records of 48 bytes in `roombb` after the first): the first (smallest at +0, largest at +12) from the vertices of the
/// polygons of the room's first list (polygons with a count and vertices of 12 bytes from +8), the second (at +24 and +36) from the boxes of the room's second list (@ 0x255d70).
pub fn bgRoomInitBB() {
    let rooms = lw(g::bgNumRooms);
    let mut room = 0;
    if rooms > 0 {
        let levelrooms = lw(g::levelrooms);
        let boxes = lw(g::roombb);
        loop {
            let high = lwc1(0x3aea40);
            let next = room + 1;
            let low = lwc1(g::levelvolumes + 4);
            let record = levelrooms + (room * 44 + 44);
            let rec = boxes + (room * 48 + 48);
            let list_b = lw(record + 12);
            let list_a = lw(record + 8);
            let first_b = lw(list_b);
            let first_a = lw(list_a);
            for k in 0..3 {
                swc1(rec + 4 * k, low);
                swc1(rec + 12 + 4 * k, high);
                swc1(rec + 24 + 4 * k, low);
                swc1(rec + 36 + 4 * k, high);
            }
            if first_a != 0 {
                let mut index = 0;
                let mut offset = 0;
                loop {
                    if lw(lw(offset + list_a)) == 0 {
                        index += 1;
                    } else {
                        let following = index + 1;
                        let mut k = 0;
                        loop {
                            let polygon = lw(offset + list_a);
                            widen_box(rec, polygon + (k * 12 + 8));
                            k += 1;
                            if (k as u32) >= (lw(polygon) as u32) {
                                break;
                            }
                        }
                        index = following;
                    }
                    offset = index << 2;
                    if lw(offset + list_a) == 0 {
                        break;
                    }
                }
            }
            if first_b != 0 {
                let mut index = 0;
                let mut at = list_b;
                loop {
                    let item = lw(at);
                    widen_axis(rec + 24, rec + 36, item + 4, item + 4, item + 20, item + 20);
                    widen_axis(rec + 28, rec + 40, item + 8, item + 16, item + 24, item + 32);
                    widen_axis(rec + 32, rec + 44, item + 12, item + 12, item + 28, item + 28);
                    index += 1;
                    at = (index << 2) + list_b;
                    if lw((index << 2) + list_b) == 0 {
                        break;
                    }
                }
            }
            room = next;
            if room >= rooms {
                break;
            }
        }
    }
}
