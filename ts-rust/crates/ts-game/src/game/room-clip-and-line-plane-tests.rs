//! The screen box of a room, the distance of a line segment to a plane, and a line against a polygon.

use crate::*;

/// Work out the box of the screen (left, top, right, bottom) in which a room is seen by the current player, from the boxes of the portals that lead to rooms nearer the
/// player (a lower number in the player's room data, 20 bytes a room); if there are none, the portals of the room that lead to nearer rooms are marked with flag 1 at +44;
/// a room not in view takes the whole window. The four numbers are written to +4, +8, +12 and +16 of the room's data (@ 0x257b80).
pub fn bgRoomCalcClip(room: i32) {
    let player = lw(g::currentplayer);
    let list = lw(room * 44 + lw(g::levelrooms) + 4);
    let count = lw(list);
    let (left, top, right, bottom);
    let first_index;
    if lw(room * 20 + lw((lw(player) << 2) + g::roomdata)) != 0 {
        let (mut l, mut t, mut r, mut b) = (0x2710, 0x2710, -1, -1);
        let mut deepest = -1;
        if count > 0 {
            let portals = lw(g::levelportals);
            let boxes = lw(g::portaldata);
            for k in 0..count {
                let index = lw((k << 2) + list + 4);
                let rec = index * 48 + boxes;
                let portal = lw((index << 2) + portals);
                if lw(rec + 44) == 0 {
                    let mut other = lw(portal);
                    if other == room {
                        other = lw(portal + 4);
                    }
                    let rooms = lw((lw(player) << 2) + g::roomdata);
                    let current = l;
                    if lw(other * 20 + rooms) < lw(room * 20 + rooms) {
                        let v = lw(rec + 24);
                        if v < current {
                            l = v;
                        }
                        let v = lw(rec + 28);
                        if v < t {
                            t = v;
                        }
                        let v = lw(rec + 32);
                        if !(deepest >= v) {
                            r = v;
                            deepest = v;
                        }
                        let v = lw(rec + 36);
                        if b < v {
                            b = v;
                        }
                    }
                }
            }
        }
        if deepest >= 0 {
            first_index = lw(player);
        } else {
            if count > 0 {
                let portals = lw(g::levelportals);
                let boxes = lw(g::portaldata);
                for k in 0..count {
                    let index = lw((k << 2) + list + 4);
                    let rec = index * 48 + boxes;
                    let portal = lw((index << 2) + portals);
                    if lw(rec + 44) == 0 {
                        let mut other = lw(portal);
                        if other == room {
                            other = lw(portal + 4);
                        }
                        let rooms = lw((lw(player) << 2) + g::roomdata);
                        if lw(other * 20 + rooms) < lw(room * 20 + rooms) {
                            continue;
                        }
                        sw(rec + 44, 1);
                    }
                }
            }
            first_index = lw(player);
        }
        left = l;
        top = t;
        right = r;
        bottom = b;
    } else {
        let window = g::window;
        let a2 = lw(window + 24);
        let v1 = lw(window + 20);
        let a0 = lw(window + 4);
        left = lw(g::window);
        top = v1;
        right = a0;
        bottom = a2;
        first_index = lw(player);
    }
    let table = g::roomdata;
    sw(room * 20 + lw((first_index << 2) + table) + 4, left);
    sw(room * 20 + lw((lw(player) << 2) + table) + 8, top);
    sw(room * 20 + lw((lw(player) << 2) + table) + 12, right);
    sw(room * 20 + lw((lw(player) << 2) + table) + 16, bottom);
}

/// The signed distance from the line segment `from` to `to` to the plane through `plane_point` with the normal `normal`: when both ends are on one side the nearer end's
/// distance (negative on the back) is returned and the nearer end and the foot of its perpendicular are written to `near_end` and `foot` (when not 0); when the segment
/// crosses the plane 0 is returned and the point where it crosses is written to both (@ 0x20af20).
pub fn HitTestDistanceLineSegmentToPlane(from: i32, to: i32, plane_point: i32, normal: i32, near_end: i32, foot: i32) -> f32 {
    let py = lwc1(plane_point + 4);
    let from_y = lwc1(from + 4);
    let mut side_from = 0;
    let px = lwc1(plane_point);
    let mut side_to = 0;
    let from_x = lwc1(from);
    let pz = lwc1(plane_point + 8);
    let ny = lwc1(normal + 4);
    let from_z = lwc1(from + 8);
    let nx = lwc1(normal);
    let to_x = lwc1(to);
    let to_y = lwc1(to + 4);
    let nz = lwc1(normal + 8);
    let rx = to_x - px;
    let ry = to_y - py;
    let to_z = lwc1(to + 8);
    let rz = to_z - pz;
    let d_from = (from_x - px) * nx + (from_y - py) * ny + (from_z - pz) * nz;
    let d_to = rx * nx + ry * ny + rz * nz;
    let mut done = false;
    if d_from <= 0.0_f32 && d_to <= 0.0_f32 {
        if d_to < d_from {
            side_from = 1;
        } else {
            side_to = 1;
        }
        done = true;
    }
    if !done && 0.0_f32 <= d_from && 0.0_f32 <= d_to {
        if !(d_from < d_to) {
            side_to = 1;
        } else {
            side_from = 1;
        }
    }
    if side_from == 0 {
        if side_to == 0 {
            if near_end != 0 || foot != 0 {
                let mut a = d_from;
                if !(0.0_f32 <= a) {
                    a = -a;
                }
                let mut b = d_from;
                if !(0.0_f32 <= b) {
                    b = -b;
                }
                let total = if !(0.0_f32 <= d_to) { a - d_to } else { a + d_to };
                let t = (b).pdiv(total);
                let x = from_x + t * (to_x - from_x);
                let y = from_y + t * (to_y - from_y);
                let z = from_z + t * (to_z - from_z);
                if foot != 0 {
                    swc1(foot + 8, z);
                    swc1(foot, x);
                    swc1(foot + 4, y);
                }
                if near_end != 0 {
                    swc1(near_end, x);
                    swc1(near_end + 8, z);
                    swc1(near_end + 4, y);
                }
            }
            0.0_f32
        } else {
            if foot != 0 {
                swc1(foot, to_x - d_to * nx);
                swc1(foot + 4, lwc1(to + 4) - d_to * lwc1(normal + 4));
                swc1(foot + 8, lwc1(to + 8) - d_to * lwc1(normal + 8));
            }
            if near_end != 0 {
                swc1(near_end, lwc1(to));
                swc1(near_end + 4, lwc1(to + 4));
                swc1(near_end + 8, lwc1(to + 8));
            }
            d_to
        }
    } else {
        if foot != 0 {
            swc1(foot, from_x - d_from * nx);
            swc1(foot + 4, lwc1(from + 4) - d_from * lwc1(normal + 4));
            swc1(foot + 8, lwc1(from + 8) - d_from * lwc1(normal + 8));
        }
        if near_end != 0 {
            swc1(near_end, lwc1(from));
            swc1(near_end + 4, lwc1(from + 4));
            swc1(near_end + 8, lwc1(from + 8));
        }
        d_from
    }
}

/// Whether the ray from `start` along `direction` hits the polygon whose corners (`count` of them, by the bytes at `order` as numbers of the vertices of 12 bytes at
/// `vertices`) lie in the plane with the normal `normal`: the ray must run against the normal and reach the plane in front of its start, and the hit point must be on the inside
/// of every edge. The hit point goes to `hit` and the normal to `normal_out` (when not 0) (@ 0x209b08).
pub fn hittestLinePoly(start: i32, direction: i32, vertices: i32, order: i32, count: i32, normal: i32, hit: i32, normal_out: i32) -> i32 {
    let nx = lwc1(normal);
    let dx = lwc1(direction);
    let ny = lwc1(normal + 4);
    let dy = lwc1(direction + 4);
    let dz = lwc1(direction + 8);
    let nz = lwc1(normal + 8);
    let facing = nx * dx + ny * dy + nz * dz;
    if !(0.0_f32 <= facing) {
        let sx = lwc1(start);
        let first = lbu(order) * 12;
        let sy = lwc1(start + 4);
        let sz = lwc1(start + 8);
        let corner = vertices + lbu(order) * 12;
        let t = ((nx * lwc1(corner) + ny * lwc1(corner + 4) + nz * lwc1(corner + 8) - (nx * sx + ny * sy + nz * sz))).pdiv(facing);
        if !(t < 0.0_f32) {
            let first_corner = vertices + first;
            let mut k = 1;
            let pz = sz + t * dz;
            let px = sx + t * dx;
            let py = sy + t * dy;
            let last = lbu(count + order - 1) * 12 + vertices;
            let ex = lwc1(first_corner) - lwc1(last);
            let ey = lwc1(first_corner + 4) - lwc1(last + 4);
            let mut previous = first_corner;
            let ez = lwc1(first_corner + 8) - lwc1(last + 8);
            let wx = lwc1(first_corner) - px;
            let wy = lwc1(first_corner + 4) - py;
            let wz = lwc1(first_corner + 8) - pz;
            let mut cz = ex * wy - ey * wx;
            let mut cy = ez * wx - ex * wz;
            let mut cx = ey * wz - ez * wy;
            if k < count {
                loop {
                    let next = lbu(order + k) * 12 + vertices;
                    let ex = lwc1(next) - lwc1(previous);
                    let ey = lwc1(next + 4) - lwc1(previous + 4);
                    let ez = lwc1(next + 8) - lwc1(previous + 8);
                    let wx = lwc1(next) - px;
                    let wy = lwc1(next + 4) - py;
                    let wz = lwc1(next + 8) - pz;
                    let new_z = ex * wy - ey * wx;
                    let new_y = ez * wx - ex * wz;
                    let new_x = ey * wz - ez * wy;
                    if (cx * new_x + cy * new_y + cz * new_z) < 0.0_f32 {
                        return 0;
                    }
                    k += 1;
                    cx = new_x;
                    cy = new_y;
                    previous = next;
                    cz = new_z;
                    if k >= count {
                        break;
                    }
                }
            }
            if hit != 0 {
                swc1(hit, px);
                swc1(hit + 8, pz);
                swc1(hit + 4, py);
            }
            if normal_out == 0 {
                return 1;
            }
            swc1(normal_out, lwc1(normal));
            swc1(normal_out + 4, lwc1(normal + 4));
            swc1(normal_out + 8, lwc1(normal + 8));
            return 1;
        }
        return 0;
    }
    0
}
