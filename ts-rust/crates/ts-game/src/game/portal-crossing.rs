//! Which room a ray from a point leaves through.

use crate::*;

/// Find the portal of room `room` through which the ray from `position` along `direction` leaves (not the one to room `skip`, not one the ray's start is behind): the ray
/// must reach the plane of the portal after its start and (unless `unbounded` is not 0) at no more than its whole length, and meet the plane inside the polygon of the portal.
/// Returns the room on the other side, or `room` if there is none or the direction is 0 (@ 0x258cf0).
pub fn bgPortalCrossed(position: i32, direction: i32, room: i32, skip: i32, unbounded: i32) -> i32 {
    let list = lw(room * 44 + lw(g::levelrooms) + 4);
    if lwc1(direction) == 0.0_f32 && lwc1(direction + 4) == 0.0_f32 && lwc1(direction + 8) == 0.0_f32 {
        return room;
    }
    let n = lw(list);
    let mut count = 0;
    if n > 0 {
        let mut at = list + 4;
        loop {
            let portal = lw((lw(at) << 2) + lw(g::levelportals));
            let mut other = lw(portal);
            if other == room {
                other = lw(portal + 4);
            }
            if other == skip {
                count += 1;
            } else if bgPortalBackFaceTest(room, position, portal) != 0 {
                count += 1;
            } else {
                let plane = portal + 8;
                let nx = lwc1(portal + 8);
                let ny = lwc1(plane + 4);
                let dx = lwc1(direction);
                let dy = lwc1(direction + 4);
                let nz = lwc1(plane + 8);
                let dz = lwc1(direction + 8);
                let facing = nx * dx + ny * dy + nz * dz;
                let offset = nx * lwc1(portal + 24) + ny * lwc1(portal + 28) + nz * lwc1(portal + 32);
                if !(facing == 0.0_f32) {
                    let sx = lwc1(position);
                    let sy = lwc1(position + 4);
                    let sz = lwc1(position + 8);
                    let t = (offset - (nx * sx + ny * sy + nz * sz)) / facing;
                    let mut skip_portal = false;
                    if t <= 0.0_f32 {
                        skip_portal = true;
                    } else if 1.0_f32 < t && unbounded == 0 {
                        skip_portal = true;
                    }
                    if !skip_portal {
                        let corners = lhu(portal + 20) as i32;
                        let mut index = 0;
                        let mut outside = false;
                        let hz = sz + t * dz;
                        let hx = sx + t * dx;
                        let hy = sy + t * dy;
                        let signed_corners = (corners << 16) >> 16;
                        let mut previous = portal + (signed_corners * 12 + 12);
                        let (mut cx, mut cy, mut cz) = (0.0_f32, 0.0_f32, 0.0_f32);
                        if signed_corners > 0 {
                            loop {
                                let corner = portal + (index * 12 + 24);
                                let ex = lwc1(corner) - lwc1(previous);
                                let ey = lwc1(corner + 4) - lwc1(previous + 4);
                                let ez = lwc1(corner + 8) - lwc1(previous + 8);
                                let wx = lwc1(corner) - hx;
                                let wy = lwc1(corner + 4) - hy;
                                let new_z = ex * wy - ey * wx;
                                let wz = lwc1(corner + 8) - hz;
                                let new_y = ez * wx - ex * wz;
                                let new_x = ey * wz - ez * wy;
                                if index > 0 && (cx * new_x + cy * new_y + cz * new_z) < 0.0_f32 {
                                    outside = true;
                                }
                                index += 1;
                                cx = new_x;
                                cy = new_y;
                                cz = new_z;
                                previous = corner;
                                if index >= signed_corners || outside {
                                    break;
                                }
                            }
                        }
                        if !outside {
                            return other;
                        }
                    }
                }
                count += 1;
            }
            at += 4;
            if count >= n {
                break;
            }
        }
    }
    room
}
