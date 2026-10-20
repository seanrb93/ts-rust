//! Opening an XPM picture, keeping an attached prop on its parent, and finding the best place on a bone.

use super::particle_spawners::random_as_float;
use crate::*;

/// Read the header of the XPM picture in the big buffer into the texture record `record` and copy its pixels to the download buffer (+20): the width and height (+0, +2) and the
/// colour count are read by `sscanf` in the form the picture `kind` (2 to 7) needs (the kinds with a palette keep the number of colours at +37 and the type 0 to 3 at +36; type 1
/// builds the palette at 0x1fedb00); the width rounded up to a power of 2 is kept at +4; rows are copied and padded with 127 on the right, or with a mip map (type flag 2) each
/// level is copied with the size shifted down and padded with 15, every level on a 128-byte boundary (@ 0x2b9180).
pub fn OpenXPM(_unused: i32, record: i32, kind: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let shifted = kind - 2;
    if (shifted as u32) < 6 {
        match shifted {
            0 | 2 => {
                let source = lw(g::bigbuffer) + 2;
                sw(sp + 12, 4);
                sscanf(&[source as i64, 0x3a9b40, record as i64, (record + 2) as i64, (sp | 4) as i64]);
                sb(record + 36, 0);
            }
            1 => {
                sw(sp + 12, 0);
            }
            3 => {
                let source = lw(g::bigbuffer);
                sscanf(&[
                    source as i64,
                    0x3a9b50,
                    record as i64,
                    (record + 2) as i64,
                    (sp | 4) as i64,
                    sp as i64,
                    (sp | 8) as i64,
                ]);
                sw(sp + 12, 1);
                sb(record + 36, 1);
            }
            4 => {
                let source = lw(g::bigbuffer);
                sw(sp + 12, 4);
                sscanf(&[source as i64, 0x3a9b68, record as i64, (record + 2) as i64, (sp | 4) as i64, (record + 37) as i64]);
                let colours = lbu(record + 37);
                sb(record + 36, 2);
                sb(record + 37, colours - 48);
            }
            _ => {
                let source = lw(g::bigbuffer);
                sw(sp + 12, 1);
                sscanf(&[
                    source as i64,
                    0x3a9b80,
                    record as i64,
                    (record + 2) as i64,
                    (sp | 4) as i64,
                    sp as i64,
                    (sp | 8) as i64,
                    (record + 37) as i64,
                ]);
                let colours = lbu(record + 37);
                sb(record + 36, 3);
                sb(record + 37, colours - 48);
            }
        }
    } else {
        sw(sp + 12, 0);
    }
    let power = PowerOf2(lhu(record));
    let buffer = lw(g::bigbuffer);
    sw(record + 4, 1 << (power & 31));
    let mut data = FindRawData(buffer);
    let mut flags = lbu(record + 36);
    if (flags & 1) != 0 {
        sw(record + 28, 0x1fedb00);
        memset(0x1fedb00, 63, 1024);
        MakeCSM1Palette(data, lw(record + 28));
        flags = lbu(record + 36);
        data += lw(sp + 8) << 2;
    }
    let download = lw(g::downloadbuffer);
    sw(record + 20, download);
    if (flags & 2) == 0 {
        let mut source = data;
        let mut to = download;
        let mut row = 0;
        if lhu(record + 2) != 0 {
            let mut width = lhu(record);
            loop {
                memcpy(to, source, width * lw(sp + 12));
                let size = lw(sp + 12);
                let mut used = lhu(record) * size;
                source += lhu(record) * size;
                to += lhu(record) * size;
                let rows = row + 1;
                if (used as u32) >= ((lw(record + 4) * size) as u32) {
                } else {
                    loop {
                        sb(to, 127);
                        used += 1;
                        to += 1;
                        if (used as u32) >= ((lw(record + 4) * lw(sp + 12)) as u32) {
                            break;
                        }
                    }
                }
                row = rows;
                let high = lhu(record + 2);
                if (row as u32) >= (high as u32) {
                    break;
                }
                width = lhu(record);
            }
        }
    } else {
        let wide = lhu(record);
        let power = PowerOf2(wide);
        let mut source = data;
        if wide == 1 << (power & 31) {
            PowerOf2(lhu(record + 2));
            source = data;
        }
        let mut to = lw(record + 20);
        let mut level = 0;
        if lbu(record + 37) != 0 {
            loop {
                let mut rows = 0;
                let height = lhu(record + 2) >> (level & 31);
                let padded = ((lw(record + 4) as u32) >> (level & 31)) as i32;
                let mut line = (lhu(record) >> (level & 31)) * lw(sp + 12);
                let next = level + 1;
                if height != 0 {
                    loop {
                        memcpy(to, source, line);
                        rows += 1;
                        source += line;
                        to += line;
                        if (rows as u32) >= (height as u32) {
                            break;
                        }
                    }
                }
                let full = padded * lw(sp + 12);
                if (line as u32) < (full as u32) {
                    loop {
                        sb(to, 15);
                        line += 1;
                        to += 1;
                        if (line as u32) >= (full as u32) {
                            break;
                        }
                    }
                }
                level = next;
                to = (to + 127) & -128;
                if (level as u32) >= (lbu(record + 37) as u32) {
                    break;
                }
            }
        }
    }
}

/// Keep the prop `prop`, which is attached to the prop at +188 (the matrix number at +192), on its parent: when the parent's model has animation and flag 128 is not set, its first
/// matrix is the parent's matrix of that number turned by the attach kind (+8: 1 or 2048 none, 32 and 16 by the fixed turn of the bits at +196, 64 by the turn at 0x3accf8) and the
/// other matrices are cleared; the place, angles and the saved place (+564) follow the parent's, and the rocket-like kinds of 32 get their root matrix turned by a random angle
/// about the place of the bone. A parent without animation gives the cleared matrices and the parent's place raised by 0.75 (@ 0x26ff10).
pub fn propAttachUpdate(prop: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let mut parent = lw(prop + 188);
    let flags = lw(lw(parent + 244) + 16);
    let model;
    let mut animated = false;
    if (flags & 128) != 0 {
        model = lw(prop + 32);
    } else if (flags & 0xf000000) == 0 {
        model = lw(prop + 32);
    } else {
        let matrices = lw(parent + 4);
        if matrices != 0 {
            animated = true;
            model = 0;
        } else {
            model = lw(prop + 32);
        }
    }
    if animated {
        let own = lw(prop + 32);
        let first = lw(own + 4);
        sceVu0CopyMatrix(first, matrices_of(parent, prop));
        let kind = lw(prop + 8);
        // None leaves the matrix at sp as it was, as the original does for the kinds it does not know.
        let mut turn: Option<[f32; 5]> = None;
        let mut unit = false;
        if kind != 32 {
            if kind >= 33 {
                if kind == 64 {
                    turn = Some([lwc1(0x3accf8), lwc1(0x3accfc), lwc1(0x3acd00), lwc1(0x3acd04), lwc1(0x3acd08)]);
                } else if kind == 2048 {
                    unit = true;
                }
            } else if kind == 1 {
                unit = true;
            } else if kind == 16 {
                turn = attach_turn(lw(prop + 196));
            }
        } else {
            turn = attach_turn(lw(prop + 196));
        }
        if unit {
            sceVu0UnitMatrix(sp);
        } else if let Some(values) = turn {
            matrixTransRotZXY(sp, values[0], values[1], values[2], values[3], values[4], 0.0_f32);
        }
        let own = lw(prop + 32);
        let mut count = 1;
        let root = lw(own + 4);
        sceVu0MulMatrix(root, root, sp);
        let mut own = lw(prop + 32);
        let source = lw(lw(prop + 188) + 244);
        let mut walk = own;
        let bones = lw(lw(own) + 4);
        swc1(prop + 48, lwc1(source + 48));
        swc1(prop + 52, lwc1(source + 52));
        swc1(prop + 56, lwc1(source + 56));
        swc1(prop + 72, lwc1(source + 72));
        swc1(prop + 76, lwc1(source + 76));
        swc1(prop + 88, lwc1(source + 88));
        if count < bones {
            loop {
                let at = lw(walk + 4) + (count << 6);
                count += 1;
                sceVu0UnitMatrix(at);
                own = lw(prop + 32);
                walk = own;
                if count >= lw(lw(own) + 4) {
                    break;
                }
            }
        }
        let rooted = lw(own + 4);
        let kind = lw(prop + 8);
        swc1(prop + 564, lwc1(rooted + 48));
        swc1(prop + 568, lwc1(rooted + 52));
        swc1(prop + 572, lwc1(rooted + 56));
        if kind == 32 {
            let number = lw(prop + 4);
            if ((number - 374) as u32) >= 5 && number != 385 {
                let skeleton = lw(own);
                let bone_count = lw(skeleton);
                let bones = skeleton - bone_count * 80;
                if lb(bones + 7) > 0 && bone_count >= 2 {
                    let place = lw(bones + 72);
                    let matrix = rooted + 64;
                    matrixTrans(sp + 128, lwc1(place), lwc1(place + 4), lwc1(place + 8));
                    sceVu0MulMatrix(matrix, matrix, sp + 128);
                    let r = random_as_float();
                    let angle = r * 2.3283064e-10_f32 * 3.6e+02_f32 * lwc1(0x3acd0c) / 1.8e+02_f32;
                    matrixRotZXY(sp + 128, 0.0_f32, 0.0_f32, angle);
                    sceVu0MulMatrix(matrix, matrix, sp + 128);
                    matrixTrans(sp + 128, -lwc1(place), -lwc1(place + 4), -lwc1(place + 8));
                    sceVu0MulMatrix(matrix, matrix, sp + 128);
                }
            }
        }
        parent = lw(prop + 188);
        sw(prop + 12, lw(lw(parent + 244) + 12));
        return;
    }
    let mut i = 0;
    let mut current = model;
    if lw(lw(current) + 4) > 0 {
        let mut matrices = lw(current + 4);
        loop {
            let at = matrices + (i << 6);
            i += 1;
            sceVu0UnitMatrix(at);
            current = lw(prop + 32);
            if i >= lw(lw(current) + 4) {
                break;
            }
            matrices = lw(current + 4);
        }
        parent = lw(prop + 188);
    }
    let source = lw(parent + 244);
    let x = lwc1(source + 48);
    let word = lw(source + 12);
    swc1(prop + 48, x);
    sw(prop + 12, word);
    let y = lwc1(source + 52) + 0.75_f32;
    swc1(prop + 52, y);
    let z = lwc1(source + 56);
    swc1(prop + 564, x);
    swc1(prop + 572, z);
    swc1(prop + 568, y);
    swc1(prop + 56, z);
}

/// The matrix of the parent of `prop` that the prop follows: the parent's matrices (+4) at the number in the prop's +192 (64 bytes each).
fn matrices_of(parent: i32, prop: i32) -> i32 {
    lw(parent + 4) + (lw(prop + 192) << 6)
}

/// The turn (a place and three angles, five floats) an attached prop is given by the bits `bits` of its +196: bit 1 the first set of constants (at 0x3accd0), bit 2 the second (at
/// 0x3acce4), neither none.
fn attach_turn(bits: i32) -> Option<[f32; 5]> {
    if (bits & 1) == 0 {
        if (bits & 2) == 0 {
            return None;
        }
        Some([lwc1(0x3acce4), lwc1(0x3acce8), lwc1(0x3accec), lwc1(0x3accf0), lwc1(0x3accf4)])
    } else {
        Some([lwc1(0x3accd0), lwc1(0x3accd4), lwc1(0x3accd8), lwc1(0x3accdc), lwc1(0x3acce0)])
    }
}

/// Find the best place for the point a shot hit on the bone `bone` of the model `model`: the points of the bone's list (+8) are turned into a frame that looks along the shot (from
/// `from` to `to`, three floats each) by the bone matrices (the point's table at +16 of the bone's records telling which of the bone, its parent and the other one moves it), the box
/// they fill is found, and its middle, with the front and back of the box, is written to `front` and `back` (three floats each) in the space of the bone (@ 0x285348).
pub fn FindBestBonePos(model: i32, bone: i32, from: i32, to: i32, front: i32, back: i32) {
    let frame = Frame::new(464);
    let sp = frame.sp();
    let skeleton = lw(model);
    let count = lw(skeleton);
    sw(sp + 272, front);
    sw(sp + 276, back);
    let bones = skeleton - count * 80;
    let record = bones + bone * 80;
    let first_parent = lb(record + 2);
    if first_parent < 0 {
        sw(sp + 280, 0);
    } else {
        sw(sp + 280, bones + first_parent * 80);
    }
    let second_parent = lb(record + 3);
    if second_parent < 0 {
        sw(sp + 284, 0);
    } else {
        sw(sp + 284, bones + second_parent * 80);
    }
    let mut list = lw(record + 8);
    sw(sp + 288, record + 20);
    let low_x;
    if list == 0 {
        sw(sp + 296, sp + 32);
        sw(sp + 292, sp + 96);
        low_x = lwc1(sp);
    } else {
        let dz = lwc1(to + 8) - lwc1(from + 8);
        let dy = lwc1(to + 4) - lwc1(from + 4);
        let high = lwc1(g::s_PossibleLastHitPart_3afa30 + 4);
        let dx = lwc1(to) - lwc1(from);
        let low = lwc1(0x3afa38);
        swc1(sp, high);
        swc1(sp + 16, low);
        swc1(sp + 240, dx);
        swc1(sp + 8, high);
        swc1(sp + 4, high);
        swc1(sp + 24, low);
        swc1(sp + 20, low);
        swc1(sp + 244, dy);
        swc1(sp + 248, dz);
        let pitch = atan2f(dz, dy);
        sw(sp + 296, sp + 32);
        matrixRotX(sp + 32, -pitch);
        matrixVecMul(lw(sp + 296), sp + 240);
        let yaw = atan2f(lwc1(sp + 240), lwc1(sp + 244));
        sw(sp + 292, sp + 96);
        matrixRotZ(sp + 96, yaw);
        if lw(list + 20) >= 0 {
            let mut start = lw(list + 12);
            loop {
                let lists = lw(sp + 288);
                let mut i = 0;
                let mut point = lw(lists + 4) + start * 12;
                let mut table = lw(lists + 16) + (start << 2);
                if lw(list + 16) <= 0 {
                    list += 24;
                } else {
                    let next = list + 24;
                    loop {
                        swc1(sp + 256, lwc1(point));
                        swc1(sp + 260, lwc1(point + 4));
                        swc1(sp + 264, lwc1(point + 8));
                        let mut matrix = None;
                        if lb(record) != 0 {
                            let which = lbu(table);
                            matrix = match which {
                                0 => Some(lb(lw(sp + 280) + 1)),
                                1 => Some(lb(lw(sp + 284) + 1)),
                                2 => Some(lb(record + 1)),
                                _ => None,
                            };
                        } else {
                            matrix = Some(lb(record + 1));
                        }
                        if let Some(number) = matrix {
                            matrixVecMul(lw(model + 4) + (number << 6), sp + 256);
                        }
                        matrixVecMul(sp + 32, sp + 256);
                        matrixVecMul(sp + 96, sp + 256);
                        let x = lwc1(sp + 256);
                        if x < lwc1(sp) {
                            swc1(sp, x);
                        }
                        let y = lwc1(sp + 260);
                        if y < lwc1(sp + 4) {
                            swc1(sp + 4, y);
                        }
                        let z = lwc1(sp + 264);
                        if z < lwc1(sp + 8) {
                            swc1(sp + 8, z);
                        }
                        if lwc1(sp + 16) < x {
                            swc1(sp + 16, x);
                        }
                        if lwc1(sp + 20) < y {
                            swc1(sp + 20, y);
                        }
                        if lwc1(sp + 24) < z {
                            swc1(sp + 24, z);
                        }
                        i += 1;
                        point += 12;
                        table += 4;
                        if i >= lw(list + 16) {
                            break;
                        }
                    }
                    list = next;
                }
                if lw(list + 20) < 0 {
                    break;
                }
                start = lw(list + 12);
            }
        }
        low_x = lwc1(sp);
    }
    let half = 0.5_f32;
    let low_z = lwc1(sp + 8);
    let low_y = lwc1(sp + 4);
    let inverse_a = lw(sp + 292);
    let middle_x = low_x + (lwc1(sp + 16) - low_x) * half;
    let half_z = (lwc1(sp + 24) - low_z) * half;
    let half_y = (lwc1(sp + 20) - low_y) * half;
    swc1(lw(sp + 276), middle_x);
    let front_at = lw(sp + 272);
    let middle_z = low_z + half_z;
    swc1(front_at, middle_x);
    sw(front_at + 4, 0);
    let back_at = lw(sp + 276);
    swc1(back_at + 4, 1.0_f32);
    swc1(back_at + 8, middle_z);
    let front_at = lw(sp + 272);
    swc1(sp + 164, low_y + half_y);
    swc1(front_at + 8, middle_z);
    swc1(sp + 160, middle_x);
    swc1(sp + 168, middle_z);
    sceVu0InversMatrix(sp + 176, inverse_a);
    matrixVecMul(sp + 176, lw(sp + 272));
    matrixVecMul(sp + 176, lw(sp + 276));
    sceVu0InversMatrix(sp + 176, lw(sp + 296));
    matrixVecMul(sp + 176, lw(sp + 272));
    matrixVecMul(sp + 176, lw(sp + 276));
}
