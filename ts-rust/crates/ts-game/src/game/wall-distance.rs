//! The distance of a point from a wall.

use crate::*;

/// The squared distance of the point `pos` (x at +0, z at +8) from the wall `wall` (a segment from (+4, +12) to (+20, +28)); a constant of the game
/// (0x3af9d0) when the point is on the open side.  When `out` is not 0 and the distance is below `limit` (squared), the unit normal of the wall goes to
/// `out` (two floats) (@ 0x27cb68).
pub fn DistFromWallSq(pos: i32, wall: i32, out: i32, _unused: f32, limit: f32) -> f32 {
    let x0 = lwc1(wall + 4);
    let z0 = lwc1(wall + 12);
    let dx = lwc1(wall + 20) - x0;
    let dz = lwc1(wall + 28) - z0;
    let vx = lwc1(pos) - x0;
    let vz = lwc1(pos + 8) - z0;
    if (dz * vx + -dx * vz) < 0.0 {
        return lwc1(0x3af9d0);
    }
    let along = vx * dx + vz * dz;
    let length2 = dx * dx + dz * dz;
    let (mut rx, mut rz) = (vx, vz);
    if 0.0 < along {
        if !(length2 <= along) {
            let t = (along).pdiv(length2);
            rz = vz - t * dz;
            rx = vx - t * dx;
        } else {
            rx = vx - dx;
            rz = vz - dz;
        }
    }
    let d2 = rx * rx + rz * rz;
    if out != 0 {
        if !(d2 < limit) {
            return d2;
        }
        let length = length2.abs().psqrt();
        swc1(out, (dz).pdiv(length));
        swc1(out + 4, (-dx).pdiv(length));
    }
    d2
}

/// The smallest squared distance (below 4) of the point `pos` from the walls of room `room`; 0 when the room has no walls, 4 when none is nearer (@ 0x2ae240).
pub fn specialfxCalcDistSqFromWall(pos: i32, room: i32) -> f32 {
    let mut nearest = 4.0_f32;
    let walls = bgGetWalls(room);
    if walls == 0 {
        return 0.0;
    }
    let mut wall = lw(walls);
    let mut at = walls + 4;
    if wall == 0 {
        return nearest;
    }
    loop {
        let d2 = DistFromWallSq(pos, wall, 0, lwc1(0x3ad74c), 4.0);
        if d2 < nearest {
            nearest = d2;
        }
        wall = lw(at);
        at += 4;
        if wall == 0 {
            break;
        }
    }
    nearest
}
