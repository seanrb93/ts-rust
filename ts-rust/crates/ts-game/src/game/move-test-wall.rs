//! Testing a move against the walls of a list.

use crate::*;

/// Test a character's move against the walls of the list `walls` (a null-ended list of pointers to walls: the kind in a halfword at +0, the end points at (+4, +12) and
/// (+20, +28) in x and z, the bottom and top at +8 and +16) that have a kind bit of `mask` (or kind 0): `swept` is the box of the move (six floats), `pos` the character's position,
/// `motion` its move in x and z, `radius` and `height` its size. A wall that is already touching (the character's box overlaps its box and `DistFromWallSq` finds it nearer than the
/// radius) or is hit by the move (`hittestMoveCircleToLine`, the cheapest of them) is recorded: the part of the move before it is written to `out_part` (1 if none), the
/// normal of the move's hit to `out_normal` and the distance to `out_distance` (the square root, when not 0), and the normal of the nearest wall to `out_wall_normal`
/// (the eleventh argument, on the caller's stack, just above this function's frame) (@ 0x27cce0).
pub fn MoveTestWall(walls: i32, mask: i32, swept: i32, pos: i32, motion: i32, out_part: i32, out_normal: i32, out_distance: i32, radius: f32, height: f32) {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let mut nearest2 = radius * radius;
    let mut wall = lw(walls);
    let out_wall_normal = lw(sp + 352);
    let mut best_part = 1.0_f32;
    if wall != 0 {
        let mut at = walls;
        let mut kind = lhu(wall);
        loop {
            if kind == 0 || (kind & mask) != 0 {
                let x1 = lwc1(wall + 4);
                let x2 = lwc1(wall + 20);
                let (x_min, x_max) = if !(x1 < x2) { (x2, x1) } else { (x1, x2) };
                let y_min = lwc1(wall + 8);
                let y_max = lwc1(wall + 16);
                let z1 = lwc1(wall + 12);
                let z2 = lwc1(wall + 28);
                let (z_min, z_max) = if !(z1 < z2) { (z2, z1) } else { (z1, z2) };
                let mut moving = true;
                let mut still_hit: Option<(f32, f32)> = None;
                let mut done = false;
                if lwc1(motion) == 0.0_f32 && lwc1(motion + 4) == 0.0_f32 {
                    moving = false;
                    let x = lwc1(pos);
                    let y = lwc1(pos + 4);
                    let z = lwc1(pos + 8);
                    if x_min < x + radius
                        && (x - radius) < x_max
                        && y_min < y + height
                        && y < y_max
                        && z_min < z + radius
                        && (z - radius) < z_max
                    {
                        let d2 = DistFromWallSq(pos, wall, sp + 80, height, nearest2);
                        let nx = lwc1(sp + 80);
                        if d2 < nearest2 {
                            nearest2 = d2;
                            still_hit = Some((nx, lwc1(sp + 84)));
                        }
                    }
                    done = true;
                }
                if moving && !done {
                    if x_min < lwc1(swept + 12)
                        && lwc1(swept) < x_max
                        && y_min < lwc1(swept + 16)
                        && lwc1(swept + 4) < y_max
                        && z_min < lwc1(swept + 20)
                        && lwc1(swept + 8) < z_max
                    {
                        swc1(sp, lwc1(wall + 28));
                        let part = hittestMoveCircleToLine(
                            sp + 80,
                            lwc1(pos),
                            lwc1(pos + 8),
                            radius,
                            lwc1(motion),
                            lwc1(motion + 4),
                            lwc1(wall + 4),
                            lwc1(wall + 12),
                            lwc1(wall + 20),
                        );
                        if !(1.0_f32 < part) {
                            if part < best_part {
                                best_part = part;
                                let d2 = DistFromWallSq(pos, wall, sp + 112, height, nearest2);
                                let nx = lwc1(sp + 80);
                                swc1(sp + 20, lwc1(sp + 84));
                                swc1(sp + 16, nx);
                                if d2 < nearest2 {
                                    let first = lwc1(sp + 112);
                                    nearest2 = d2;
                                    still_hit = Some((first, lwc1(sp + 116)));
                                }
                            }
                        } else {
                            let d2 = DistFromWallSq(pos, wall, sp + 96, height, nearest2);
                            let first = lwc1(sp + 96);
                            if d2 < nearest2 {
                                nearest2 = d2;
                                still_hit = Some((first, lwc1(sp + 100)));
                            }
                        }
                    }
                }
                if let Some((a, b)) = still_hit {
                    swc1(sp + 32, a);
                    swc1(sp + 36, b);
                }
            }
            at += 4;
            wall = lw(at);
            if wall == 0 {
                break;
            }
            kind = lhu(wall);
        }
    }
    if out_part != 0 {
        swc1(out_part, best_part);
    }
    let first = lwc1(sp + 16);
    if out_normal != 0 {
        let second = lwc1(sp + 20);
        swc1(out_normal, first);
        swc1(out_normal + 4, second);
    }
    if out_distance != 0 {
        swc1(out_distance, nearest2.abs().psqrt());
    }
    let wall_first = lwc1(sp + 32);
    if out_wall_normal != 0 {
        let wall_second = lwc1(sp + 36);
        swc1(out_wall_normal, wall_first);
        swc1(out_wall_normal + 4, wall_second);
    }
}
