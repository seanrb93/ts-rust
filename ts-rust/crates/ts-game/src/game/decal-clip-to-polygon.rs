//! Clipping a decal's quad against the edges of a polygon.

use crate::*;

/// Clip the quad of the decal `decal` (four corners of 16-byte points from +0; the corners picked by `corner_a` and `corner_b` (0 to 3) give the directions of the texture) against the
/// polygon of `edges` vertices (their numbers in the byte list `list`, the vertices from `vertices`, 12 bytes each, the plane's normal at `normal`): each edge cuts the polygon
/// being clipped (two buffers of up to 8 points, used in turn) by the plane through the edge along the normal, Sutherland-Hodgman style; the edges that cut it are written to
/// `record` (pairs of vertex numbers ended by -1, when it is not 0, up to 8). The points left are written as the decal's corners with the texture coordinates (at +128) from their
/// distance to the two texture directions over the length of the direction; the number of corners is set at +200 and 1 is returned when there are more than two (@ 0x2a2710).
pub fn decalClipToPoly(
    decal: i32,
    normal: i32,
    corner_a: i32,
    corner_b: i32,
    vertices: i32,
    list: i32,
    edges: i32,
    record: i32,
) -> i32 {
    let frame = Frame::new(640);
    let sp = frame.sp();
    let mut count: i32 = 4;
    let mut out_buffer = sp + 96;
    let spare = out_buffer;
    let mut in_buffer = sp;
    let mut recorded: i32 = 0;
    sw(sp + 320, vertices);
    sw(sp + 324, list);
    sw(sp + 328, record);
    let x0 = lwc1(decal);
    let y0 = lwc1(decal + 4);
    let (ax, ay, az);
    if corner_a != 1 && corner_a != 3 {
        ay = lwc1(decal + 52);
        ax = x0 - lwc1(decal + 48);
        az = lwc1(decal + 56);
    } else {
        ay = lwc1(decal + 20);
        ax = x0 - lwc1(decal + 16);
        az = lwc1(decal + 24);
    }
    let dy = y0 - ay;
    let z0 = lwc1(decal + 8);
    let dz = z0 - az;
    let length_a = ax * ax + dy * dy + dz * dz;
    let (bx, by, bz);
    if corner_b != 1 && corner_b != 3 {
        bx = lwc1(decal + 48);
        by = lwc1(decal + 52);
        bz = lwc1(decal + 56);
    } else {
        bx = lwc1(decal + 16);
        by = lwc1(decal + 20);
        bz = lwc1(decal + 24);
    }
    let across_x = x0 - bx;
    let across_y = y0 - by;
    let across_z = z0 - bz;
    sw(sp + 332, decal + 4);
    sw(sp + 356, corner_a << 4);
    sw(sp + 360, corner_b << 4);
    sw(sp + 344, decal + 8);
    sw(sp + 336, corner_a + 3);
    sw(sp + 348, corner_a + 6);
    sw(sp + 340, corner_b + 3);
    sw(sp + 352, corner_b + 6);
    for i in 0..4 {
        let offset = i << 4;
        let at = (((i << 1) + i) << 2) + in_buffer;
        swc1(at, lwc1(decal + offset));
        swc1(at + 4, lwc1(lw(sp + 332) + offset));
        swc1(at + 8, lwc1(lw(sp + 344) + offset));
    }
    let mut edge = 0;
    if edges > 0 {
        let mut record_at = (recorded << 3) + lw(sp + 328);
        loop {
            let next_edge = edge + 1;
            if edges == 0 {
                trap("break");
            }
            let mut first = true;
            let nx = lwc1(normal);
            let ny = lwc1(normal + 4);
            let nz = lwc1(normal + 8);
            let start_at = lw(sp + 320) + lbu(lw(sp + 324) + edge) * 12;
            let mut px = lwc1(start_at);
            let mut pz = lwc1(start_at + 8);
            let mut py = lwc1(start_at + 4);
            let end_at = lw(sp + 320) + lbu(lw(sp + 324) + next_edge.wrapping_rem(edges)) * 12;
            let ez = lwc1(end_at + 8) - pz;
            let ex = lwc1(end_at) - px;
            let ey = lwc1(end_at + 4) - py;
            swc1(sp + 280, ez);
            swc1(sp + 272, ex);
            swc1(sp + 276, ey);
            let cy = ez * nx - ex * nz;
            let cx = ey * nz - ez * ny;
            swc1(sp + 260, cy);
            swc1(sp + 256, cx);
            let cz = ex * ny - ey * nx;
            swc1(sp + 264, cz);
            let unit = 1.0_f32 / (cx * cx + cy * cy + cz * cz).abs().sqrt();
            let mut produced: i32 = 0;
            let last = (count - 1) * 12 + in_buffer;
            let plane_x = lwc1(sp + 256) * unit;
            let plane_z = lwc1(sp + 264) * unit;
            let plane_y = lwc1(sp + 260) * unit;
            swc1(sp + 256, plane_x);
            swc1(sp + 264, plane_z);
            swc1(sp + 260, plane_y);
            let rx = lwc1(last) - px;
            swc1(sp + 288, rx);
            let ry = lwc1(last + 4) - py;
            swc1(sp + 292, ry);
            let rz = lwc1(last + 8) - pz;
            swc1(sp + 296, rz);
            let mut previous_inside = 1;
            if !((rx * plane_x + ry * plane_y + rz * plane_z) <= 0.0_f32) {
                previous_inside = 0;
            }
            let mut index = 0;
            let mut previous = last;
            let swap_in;
            let swap_out;
            if (count as i64) <= 0_i64 {
                swap_in = in_buffer;
                swap_out = out_buffer;
                edge = next_edge;
            } else {
                swap_in = in_buffer;
                swap_out = out_buffer;
                loop {
                    let mx = lwc1(sp + 256);
                    let my = lwc1(sp + 260);
                    let mz = lwc1(sp + 264);
                    let mut inside = 1;
                    let current = index * 12 + in_buffer;
                    let rx = lwc1(current) - px;
                    swc1(sp + 288, rx);
                    let ry = lwc1(current + 4) - py;
                    swc1(sp + 292, ry);
                    let rz = lwc1(current + 8) - pz;
                    swc1(sp + 296, rz);
                    if !((rx * mx + ry * my + rz * mz) <= 0.0_f32) {
                        inside = 0;
                    }
                    'vertex: {
                        let mut recording = true;
                        if previous_inside == 0 {
                            if inside == 0 {
                                recording = false;
                            } else {
                                let dx = lwc1(current) - lwc1(previous);
                                swc1(sp + 304, dx);
                                let dy = lwc1(current + 4) - lwc1(previous + 4);
                                swc1(sp + 308, dy);
                                let dz = lwc1(current + 8) - lwc1(previous + 8);
                                swc1(sp + 312, dz);
                                let prev_x = lwc1(previous);
                                let t = (mx * px + my * py + mz * pz - (mx * prev_x + my * lwc1(previous + 4) + mz * lwc1(previous + 8)))
                                    / (mx * dx + my * dy + mz * dz);
                                let slot = produced * 12 + out_buffer;
                                swc1(slot, prev_x + t * dx);
                                swc1(slot + 4, lwc1(previous + 4) + t * lwc1(sp + 308));
                                swc1(slot + 8, lwc1(previous + 8) + t * lwc1(sp + 312));
                                let slot = (produced + 1) * 12 + out_buffer;
                                swc1(slot, lwc1(current));
                                swc1(slot + 4, lwc1(current + 4));
                                swc1(slot + 8, lwc1(current + 8));
                                produced += 2;
                                if lw(sp + 328) == 0 {
                                    recording = false;
                                }
                            }
                        } else if inside != 0 {
                            let slot = produced * 12 + out_buffer;
                            produced += 1;
                            swc1(slot, lwc1(current));
                            swc1(slot + 4, lwc1(current + 4));
                            swc1(slot + 8, lwc1(current + 8));
                            recording = false;
                        } else {
                            let dx = lwc1(current) - lwc1(previous);
                            swc1(sp + 304, dx);
                            let slot = produced * 12 + out_buffer;
                            let dy = lwc1(current + 4) - lwc1(previous + 4);
                            swc1(sp + 308, dy);
                            let dz = lwc1(current + 8) - lwc1(previous + 8);
                            swc1(sp + 312, dz);
                            let prev_x = lwc1(previous);
                            let t = (mx * px + my * py + mz * pz - (mx * prev_x + my * lwc1(previous + 4) + mz * lwc1(previous + 8)))
                                / (mx * dx + my * dy + mz * dz);
                            swc1(slot, prev_x + t * dx);
                            swc1(slot + 4, lwc1(previous + 4) + t * lwc1(sp + 308));
                            swc1(slot + 8, lwc1(previous + 8) + t * lwc1(sp + 312));
                            produced += 1;
                            if lw(sp + 328) == 0 {
                                recording = false;
                            }
                        }
                        if recording && first {
                            if !(recorded < 8) {
                                index += 1;
                                break 'vertex;
                            }
                            sw(record_at, edge);
                            first = false;
                            sw(record_at + 4, next_edge.wrapping_rem(edges));
                            recorded += 1;
                            record_at += 8;
                        }
                        index += 1;
                    }
                    previous_inside = inside;
                    previous = current;
                    if (index as i64) >= count as i64 {
                        break;
                    }
                    px = lwc1(start_at);
                    py = lwc1(start_at + 4);
                    pz = lwc1(start_at + 8);
                }
                edge = next_edge;
            }
            count = produced;
            in_buffer = if swap_in != sp { sp } else { spare };
            out_buffer = if swap_out != sp { sp } else { spare };
            if edge >= edges {
                break;
            }
        }
    }
    let a_index = lw(sp + 340);
    let b_index = lw(sp + 336);
    let mut n = 0;
    let base_z = lw(sp + 344);
    let b_pick = if -1 < b_index { b_index } else { lw(sp + 348) };
    let base_y = lw(sp + 332);
    let top_z = base_z + lw(sp + 360);
    let a_offset = lw(sp + 356);
    sw(sp + 352, if -1 < a_index { a_index } else { lw(sp + 352) });
    let v0 = (b_index - ((b_pick >> 2) << 2)) << 4;
    let v1 = (a_index - ((lw(sp + 352) >> 2) << 2)) << 4;
    let b_offset = lw(sp + 360);
    let f11 = lwc1(base_y + v0);
    let f0 = lwc1(base_z + v0);
    let f1 = lwc1(decal + a_offset);
    let f2 = lwc1(base_y + a_offset);
    let f3 = lwc1(base_z + a_offset);
    let f4 = lwc1(decal + v1);
    let f5 = lwc1(lw(sp + 332) + v1);
    let f7 = lwc1(decal + b_offset);
    let f8 = lwc1(lw(sp + 332) + b_offset);
    let f9 = lwc1(top_z);
    let f6 = lwc1(base_z + v1);
    swc1(sp + 192, lwc1(decal + v0));
    swc1(sp + 196, f11);
    swc1(sp + 200, f0);
    swc1(sp + 204, f1);
    swc1(sp + 208, f2);
    swc1(sp + 212, f3);
    swc1(sp + 224, f4);
    swc1(sp + 228, f5);
    swc1(sp + 232, f6);
    swc1(sp + 236, f7);
    swc1(sp + 240, f8);
    swc1(sp + 244, f9);
    let mut count_reg = count as u128;
    let mut recorded_reg = recorded as u128;
    if (count as i64) > 0 {
        sw(sp + 372, decal + 128);
        sw(sp + 364, sp + 192);
        sw(sp + 368, sp + 204);
        loop {
            let sign = ((n as u32) >> 31) as i32;
            let vertex;
            if (n & 1) == 0 {
                let half = (n + sign) >> 1;
                if (count_reg as i64) == 0 {
                    trap("break");
                }
                vertex = (count_reg as i32 - half).wrapping_rem(count_reg as i32) * 12 + in_buffer;
            } else {
                vertex = (((n + sign) >> 1) + 1) * 12 + in_buffer;
            }
            let at16 = n << 4;
            let base_y_now = lw(sp + 332);
            swc1(decal + at16, lwc1(vertex));
            let at8 = n << 3;
            let u_at = lw(sp + 372) + at8;
            swc1(base_y_now + at16, lwc1(vertex + 4));
            let z_at = lw(sp + 344) + at16;
            swc1(z_at, lwc1(vertex + 8));
            sq(sp + 416, count_reg);
            sq(sp + 432, recorded_reg);
            let along_a = HitTestDistancePointToLineSegmentSquared(vertex, lw(sp + 364), lw(sp + 368), 0);
            let saved_count = lq(sp + 416);
            let saved_recorded = lq(sp + 432);
            swc1(u_at, (along_a / length_a).abs().sqrt());
            sq(sp + 416, saved_count);
            sq(sp + 432, saved_recorded);
            let v_at = decal + 132 + at8;
            let along_b = HitTestDistancePointToLineSegmentSquared(vertex, sp + 224, sp + 236, 0);
            count_reg = lq(sp + 416);
            recorded_reg = lq(sp + 432);
            swc1(v_at, (along_b / (across_x * across_x + across_y * across_y + across_z * across_z)).abs().sqrt());
            let mut u = lwc1(u_at);
            if u < 0.0_f32 {
                swc1(u_at, 0.0_f32);
                u = 0.0_f32;
            }
            if 1.0_f32 < u {
                swc1(u_at, 1.0_f32);
            }
            let mut v = lwc1(v_at);
            if v < 0.0_f32 {
                swc1(v_at, 0.0_f32);
                v = 0.0_f32;
            }
            if 1.0_f32 < v {
                swc1(v_at, 1.0_f32);
            }
            n += 1;
            if (n as i64) >= count_reg as i64 {
                break;
            }
        }
    }
    sw(decal + 200, count_reg as i32);
    let record_now = lw(sp + 328);
    let room = ((recorded_reg as i64) < 8_i64) as i32;
    if record_now != 0 {
        let at = (recorded_reg as i32) << 3;
        if room != 0 {
            sw(at + record_now, -1);
        }
    }
    (2 < lw(decal + 200)) as i32
}
