//! The walls and floors of a prop, as the game first made them.

use crate::*;

/// Stretch the box at sp (lowest) and sp+16 (highest) over the points `points` of the lists of vertices starting at `list` (records of 24 bytes ended by a negative +20; the first
/// point of a list at +12, the count at +16).
fn grow_box(sp: i32, mut list: i32, points: i32) {
    let mut first = lw(list + 12);
    loop {
        let count = lw(list + 16);
        let mut i = 0;
        let mut point = first * 12 + points;
        if count <= 0 {
            list += 24;
        } else {
            let next = list + 24;
            loop {
                let step = i + 1;
                let after = point + 12;
                let mut at = point;
                let mut high = sp + 16;
                let mut low = sp;
                let mut left = 2;
                loop {
                    let mut value = lwc1(at);
                    let current_high;
                    if !(value < lwc1(low)) {
                        current_high = lwc1(high);
                    } else {
                        swc1(low, value);
                        value = lwc1(at);
                        current_high = lwc1(high);
                    }
                    if current_high < value {
                        swc1(high, value);
                    }
                    high += 4;
                    low += 4;
                    left -= 1;
                    at += 4;
                    if left < 0 {
                        break;
                    }
                }
                i = step;
                point = after;
                if i >= count {
                    break;
                }
            }
            list = next;
        }
        if lw(list + 20) < 0 {
            break;
        }
        first = lw(list + 12);
    }
}

/// Make the walls and floors of the prop of kind `kind` (1, 4, 128, 256 for props 310 and 311, and 512) and number `number` of the prop table: memory for the number of boxes the kind
/// needs (1, 2 for prop 310, or one for each bone of kind 4) is taken from the level area, and for each box the points of the bones it holds (all of them, or by the kind: prop
/// 310 splits at the sixth bone, kind 4 takes the bone of its own number) are measured, and the box's walls and floor (five faces of 36 bytes with their corners and the plane
/// records of 56 bytes) are filled in from the measure; the tables of the walls (`PWFTAB`) get the starts (@ 0x2677a8).
pub fn propCalculateWallsFloorsOriginal(kind: i32, number: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let table = (number << 4) + PWFTAB;
    let skeleton = lw(number * 28 + (PROPTABLE + 4));
    sw(sp + 36, skeleton);
    let bone_count = lw(skeleton);
    sw(sp + 40, skeleton - bone_count * 80);
    let boxes;
    if kind != 128 {
        if kind >= 129 {
            if kind != 256 {
                if kind != 512 {
                    return;
                }
                boxes = 1;
            } else if number != 310 {
                if number != 311 {
                    return;
                }
                boxes = 1;
            } else {
                boxes = 2;
            }
        } else if kind == 1 {
            boxes = 1;
        } else if kind != 4 {
            return;
        } else {
            boxes = bone_count;
        }
    } else {
        boxes = 1;
    }
    sw(sp + 32, boxes);
    let by_four = boxes << 2;
    let by_sixteen = boxes << 4;
    let heads = (by_four + 1) << 2;
    sw(sp + 48, by_sixteen);
    let counts = (boxes + 1) << 2;
    sw(sp + 44, by_four);
    let size = heads + 15 + (by_sixteen + boxes * 144) + (by_four + counts + boxes * 56) & -16;
    let memory = memAlloc(size, 1);
    let mut box_number = 0;
    sw(table, memory);
    let mut next = memory + heads;
    sw(table + 4, next);
    next += lw(sp + 48);
    sw(table + 8, next);
    next += counts;
    sw(table + 12, next);
    next += lw(sp + 44);
    if lw(sp + 32) <= 0 {
        let second = lw(table + 8);
        sw(sp + 56, memory);
        sw(sp + 52, second);
    } else {
        let second = lw(table + 8);
        sw(sp + 56, memory);
        sw(sp + 52, second);
        loop {
            let a1 = lw(sp + 52);
            let faces = lw(sp + 56) + (box_number << 4);
            sw(faces, next);
            let record_at = a1 + (box_number << 2);
            let mut at = next + 36;
            sw(faces + 4, at);
            at += 36;
            sw(faces + 8, at);
            at += 36;
            sw(faces + 12, at);
            at += 36;
            sw(record_at, at);
            next = at + 56;
            'fill: {
                let zero_base;
                if kind != 128 {
                    if kind >= 129 {
                        if kind != 256 {
                            if kind != 512 {
                                break 'fill;
                            }
                            zero_base = lw(table + 4);
                        } else if number == 310 {
                            let base = lw(table + 4);
                            if box_number != 0 {
                                let top = lw(table + 12);
                                sw(base + 28, 6);
                                sw(base + 16, 6);
                                sw(base + 20, 6);
                                sw(base + 24, 6);
                                sw(top + 4, 6);
                                break 'fill;
                            }
                            let top = lw(table + 12);
                            sw(base + 12, 1);
                            sw(base, 1);
                            sw(base + 4, 1);
                            sw(base + 8, 1);
                            sw(top, 1);
                            break 'fill;
                        } else if number != 311 {
                            break 'fill;
                        } else {
                            zero_base = lw(table + 4);
                        }
                    } else if kind != 1 {
                        if kind == 4 {
                            let base = lw(table + 4);
                            let top = lw(table + 12);
                            let value = if (lw(lw(sp + 36)) ^ 1) == 0 { 0 } else { box_number - 1 };
                            sw(base + 12, value);
                            sw(base, value);
                            sw(base + 4, value);
                            sw(base + 8, value);
                            sw(top, value);
                            break 'fill;
                        }
                        break 'fill;
                    } else {
                        zero_base = lw(table + 4);
                    }
                } else {
                    zero_base = lw(table + 4);
                }
                let top = lw(table + 12);
                sw(zero_base + 12, 0);
                sw(zero_base, 0);
                sw(zero_base + 4, 0);
                sw(zero_base + 8, 0);
                sw(top, 0);
            }
            let mut j = 0;
            let high = lwc1(0x3af900);
            let low = lwc1(0x3af904);
            let bones = lw(lw(sp + 36));
            swc1(sp, high);
            swc1(sp + 16, low);
            swc1(sp + 8, high);
            swc1(sp + 4, high);
            swc1(sp + 24, low);
            swc1(sp + 20, low);
            let after = lw(faces + 12);
            let next_box = box_number + 1;
            let record = lw(record_at);
            let face_a = lw(faces);
            let face_b = lw(faces + 4);
            let face_c = lw(faces + 8);
            if bones > 0 {
                loop {
                    let selected = if kind != 128 {
                        if kind >= 129 {
                            if kind != 256 {
                                kind == 512
                            } else if number != 310 {
                                number == 311
                            } else if j < 6 {
                                box_number == 0
                            } else {
                                box_number == 1
                            }
                        } else if kind != 1 {
                            kind == 4 && j == box_number
                        } else {
                            true
                        }
                    } else {
                        true
                    };
                    if selected {
                        let bone = lw(sp + 40) + j * 80;
                        let first = lw(bone + 8);
                        if first != 0 && lw(first + 20) >= 0 {
                            grow_box(sp, first, lw(bone + 24));
                        }
                        let second = lw(bone + 12);
                        if second != 0 && lw(second + 20) >= 0 {
                            grow_box(sp, second, lw(bone + 48));
                        }
                    }
                    j += 1;
                    if !(j < bones) {
                        break;
                    }
                }
            }
            let min_y = lwc1(sp + 4);
            sh(after, 0);
            swc1(after + 8, min_y);
            box_number = next_box;
            swc1(face_c + 8, min_y);
            let total = lw(sp + 32);
            swc1(face_b + 8, min_y);
            swc1(face_a + 8, min_y);
            sh(face_c, 0);
            let min_y = lwc1(sp + 4);
            sh(face_b, 0);
            swc1(after + 24, min_y);
            swc1(face_c + 24, min_y);
            swc1(face_b + 24, min_y);
            swc1(face_a + 24, min_y);
            sh(face_a, 0);
            let max_y = lwc1(sp + 20);
            swc1(after + 16, max_y);
            swc1(face_c + 16, max_y);
            swc1(face_b + 16, max_y);
            swc1(face_a + 16, max_y);
            let max_y = lwc1(sp + 20);
            swc1(after + 32, max_y);
            swc1(face_c + 32, max_y);
            swc1(face_b + 32, max_y);
            swc1(face_a + 32, max_y);
            let min_x = lwc1(sp);
            swc1(face_b + 4, min_x);
            swc1(face_a + 20, min_x);
            let min_z = lwc1(sp + 8);
            swc1(face_b + 12, min_z);
            swc1(face_a + 28, min_z);
            let max_x = lwc1(sp + 16);
            swc1(face_c + 4, max_x);
            swc1(face_b + 20, max_x);
            let min_z = lwc1(sp + 8);
            swc1(face_c + 12, min_z);
            swc1(face_b + 28, min_z);
            sw(record, 4);
            let max_x = lwc1(sp + 16);
            sh(record + 6, 16);
            swc1(after + 4, max_x);
            swc1(face_c + 20, max_x);
            sh(record + 4, 0);
            let max_z = lwc1(sp + 24);
            swc1(after + 12, max_z);
            swc1(face_c + 28, max_z);
            let min_x = lwc1(sp);
            swc1(face_a + 4, min_x);
            swc1(after + 20, min_x);
            let max_z = lwc1(sp + 24);
            swc1(face_a + 12, max_z);
            swc1(after + 28, max_z);
            swc1(record + 8, lwc1(sp));
            swc1(record + 12, lwc1(sp + 20));
            swc1(record + 16, lwc1(sp + 8));
            swc1(record + 20, lwc1(sp));
            swc1(record + 24, lwc1(sp + 20));
            swc1(record + 28, lwc1(sp + 24));
            swc1(record + 32, lwc1(sp + 16));
            swc1(record + 36, lwc1(sp + 20));
            swc1(record + 40, lwc1(sp + 24));
            swc1(record + 44, lwc1(sp + 16));
            swc1(record + 48, lwc1(sp + 20));
            swc1(record + 52, lwc1(sp + 8));
            if box_number >= total {
                break;
            }
        }
    }
    let heads_end = lw(sp + 56);
    sw(lw(sp + 48) + heads_end, 0);
    sw(lw(sp + 44) + lw(sp + 52), 0);
}
