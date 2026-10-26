//! Turning the triangle strips of a chunk of a model into polygons, and merging them.

use crate::*;

/// Turn the triangle strips of the chunk `chunk` (the first strip at +4 and the first vertex at +12 of its record, in the strip table and the vertices of the model `source`)
/// into triangles (polygons of 12 bytes: the count of 3 at +0, the three vertex numbers as the bytes of the list at +8, the normal as three shorts at +2, +4 and +6), and merge
/// the triangles into larger polygons with `obMergePolys` until none join; the polygons are put in the new record `dest` (the number of them at +28 and the array at +32,
/// their lists of vertex numbers after the array) taken from the output pools of `model` (+16 is the pool, +20 how much of it is used, +4 and +8 two buffers to work in, +12 the
/// size needed, found out when +0 is 0 and only counted; +24 and +28 the number of polygons and of indices made).
///
/// Two lists of polygons are kept in records on the stack (at `sp` and `sp + 48`) and used turn about: the merged result of one pass is the source of the next (@ 0x25dec0).
pub fn obMergeChunkPolys(model: i32, source: i32, chunk: i32, dest: i32) {
    let frame = Frame::new(480);
    let sp = frame.sp();
    let mut source_record = sp;
    let mut result_record = sp + 48;
    let mut buffer = 0;
    let mut pass = 0;
    sw(model + 24, 0);
    loop {
        let mut ordinal = 0;
        sw(model + 28, 0);
        let mut strip = lw(source) + (lw(chunk + 4) << 4);
        let mut vertex = lw(source + 4) + lw(chunk + 12) * 12;
        let scale = lwc1(0x3acb54);
        loop {
            let mut header = lw(strip);
            let mut triangles_left = (header & 0x7fff) - 3;
            let mut odd = ((header as u32) >> 16) as i32 & 1;
            if triangles_left != -1 {
                loop {
                    let next_vertex = vertex + 12;
                    let next_ordinal = ordinal + 1;
                    let count;
                    if pass != 1 {
                        count = lw(model + 24);
                    } else {
                        let far = vertex + 24;
                        let second = if odd != 0 { far } else { vertex + 12 };
                        let third = if odd == 0 { far } else { next_vertex };
                        let after = ordinal + 2;
                        let second_number = if odd != 0 { after } else { next_ordinal };
                        let third_number = if odd == 0 { after } else { next_ordinal };
                        let e1x = lwc1(second) - lwc1(vertex);
                        let e1y = lwc1(second + 4) - lwc1(vertex + 4);
                        let e1z = lwc1(second + 8) - lwc1(vertex + 8);
                        let e2x = lwc1(third) - lwc1(vertex);
                        let e2y = lwc1(third + 4) - lwc1(vertex + 4);
                        let nz = e1x * e2y - e1y * e2x;
                        let e2z = lwc1(third + 8) - lwc1(vertex + 8);
                        let ny = e1z * e2x - e1x * e2z;
                        let nx = e1y * e2z - e1z * e2y;
                        let inv = (1.0_f32).pdiv((nx * nx + ny * ny + nz * nz).abs().psqrt());
                        let polygons = lw(result_record + 32);
                        let record = lw(model + 24) * 12 + polygons;
                        let nz = nz * inv;
                        let nx = nx * inv;
                        let ny = ny * inv;
                        sw(record + 8, lw(polygons + 8) + lw(model + 28));
                        sb(record, 3);
                        sb(lw(lw(model + 24) * 12 + lw(result_record + 32) + 8), ordinal);
                        sb(lw(lw(model + 24) * 12 + lw(result_record + 32) + 8) + 1, second_number);
                        sb(lw(lw(model + 24) * 12 + lw(result_record + 32) + 8) + 2, third_number);
                        count = lw(model + 24);
                        let x_short = (nx * scale) as i32;
                        let y_short = (ny * scale) as i32;
                        let at = count * 12 + lw(result_record + 32);
                        sh(at + 6, (nz * scale) as i32);
                        sh(at + 2, x_short);
                        sh(at + 4, y_short);
                    }
                    let indices = lw(model + 28);
                    sw(model + 24, count + 1);
                    ordinal = next_ordinal;
                    vertex = next_vertex;
                    sw(model + 28, indices + 3);
                    triangles_left -= 1;
                    odd = 1 - odd;
                    if triangles_left == -1 {
                        break;
                    }
                }
                header = lw(strip);
            }
            ordinal += 2;
            vertex += 24;
            if (header & 0x8000) != 0 {
                break;
            }
            strip += 16;
        }
        let made = lw(model + 24);
        if made == 0 {
            return;
        }
        if pass != 0 {
            pass += 1;
        } else {
            if lw(model) == 0 {
                let needed = made * 12 + lw(model + 28) + 15 & -16;
                if lw(model + 12) < needed {
                    sw(model + 12, needed);
                }
                return;
            }
            sw(result_record + 28, made);
            buffer = lw(model + 4);
            let count = lw(model + 24);
            sw(result_record + 32, buffer);
            sw(buffer + 8, count * 12 + buffer);
            pass += 1;
        }
        if pass >= 2 {
            break;
        }
        sw(model + 24, 0);
    }
    // merge the polygons over and over, the result of each pass the source of the next
    loop {
        let mut other = lw(model + 4);
        if buffer == other {
            other = lw(model + 8);
        }
        let next_buffer = other;
        let next_source = if source_record != sp { sp } else { sp + 48 };
        let source_count = lw(next_source + 28);
        result_record = if result_record != sp { sp } else { sp + 48 };
        sw(model + 24, 0);
        sw(model + 28, 0);
        let vertex_base = lw(source + 4);
        let first_vertex = lw(chunk + 12) * 12;
        sw(result_record + 32, next_buffer);
        sw(next_buffer + 8, source_count * 12 + next_buffer);
        let merged = obMergePolys(model, result_record, next_source, next_source, vertex_base + first_vertex);
        buffer = next_buffer;
        source_record = next_source;
        if merged == 0 {
            break;
        }
    }
    let final_record = if source_record != sp { sp } else { sp + 48 };
    if lw(model) != 1 {
        let polygons_size = lw(model + 24) * 12;
        let used = lw(model + 20);
        let array = lw(model + 16) + used;
        let total = lw(model + 24) * 12 + lw(model + 28);
        sw(dest + 32, array);
        sw(array + 8, array + polygons_size);
        sw(model + 20, used + (total + 15 & -16));
        sw(dest + 28, lw(final_record + 28));
        sw(model + 28, 0);
        let mut k = 0;
        if lw(final_record + 28) > 0 {
            let mut from = lw(final_record + 32);
            loop {
                sb(k * 12 + lw(dest + 32), lbu(k * 12 + from));
                from = lw(final_record + 32);
                let to_array = lw(dest + 32);
                let from_record = k * 12 + from;
                let offset = lw(model + 28);
                let to_record = k * 12 + to_array;
                let first = lw(to_array + 8);
                sh(to_record + 2, lhu(from_record + 2));
                let y = lhu(from_record + 4);
                sw(to_record + 8, first + offset);
                sh(to_record + 4, y);
                sh(to_record + 6, lhu(from_record + 6));
                sw(model + 28, offset + lbu(from_record));
                let mut m = 0;
                let mut to_array = to_array;
                if lbu(from_record) != 0 {
                    loop {
                        let byte = lbu(lw(k * 12 + from + 8) + m);
                        let at = lw(k * 12 + to_array + 8) + m;
                        m += 1;
                        sb(at, byte);
                        from = lw(final_record + 32);
                        if m >= lbu(k * 12 + from) {
                            break;
                        }
                        to_array = lw(dest + 32);
                    }
                }
                k += 1;
                if k >= lw(final_record + 28) {
                    break;
                }
            }
        }
    } else {
        sw(model + 20, lw(model + 20) + (lw(model + 24) * 12 + lw(model + 28) + 15 & -16));
    }
}
