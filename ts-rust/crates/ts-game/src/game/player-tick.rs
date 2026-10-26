//! The tick of the player: the sticks and buttons turn into a movement, which is made by the mode the player is in.
//!
//! The record of the player (`currentplayer`) has, among other things (the offsets are in bytes): the number of the pad +0, the flags +20 (1 run, 2 aim), the gun +24 (its flags at +2: 1 look inverted, 2 aim button toggles, 4 run button toggles, 8 holds the pitch when it turns, 128 auto-aim), the masks of the buttons +36 to +56, the axes of the sticks +68 to +96, the heights of the eyes +104 to +124, the position +140, the position and heights shown +152, the unit vector the player looks along +188, the right vector +212, the frame of the view +228 to +248, the field of view +252 to +272, the angle round and up +276 and +284, the stance +288, the mode +296 (0 on the ground, 1 pushed, 2 falling, 3 swimming, 4 dead, 5 nothing, 6 going into a remote prop, 7 riding one), the body prop +384, the camera prop +388, the room +796, 1 while aiming +800, the speeds of the looking +804 to +816, the polygon of the ground +824, the auto-aim target +828, the rumble +1776 and +1780, and the limits of the sticks +1784 and +1788. The character of the body is at +352 of the prop.

use crate::*;

/// The time of the frame, as the game counts it.
fn frames() -> f32 {
    lwc1(g::updateframesf)
}

/// The part of a stick axis outside of the dead zone of 0.5 (twice its size), or 0.
fn stick_dead_zone(x: f32) -> f32 {
    if !(0.0_f32 < x) {
        if !(x < -0.5_f32) {
            0.0_f32
        } else {
            let part = ps2float::add(x, 0.5_f32);
            ps2float::add(part, part)
        }
    } else if !(0.5_f32 < x) {
        0.0_f32
    } else {
        let part = ps2float::sub(x, 0.5_f32);
        ps2float::add(part, part)
    }
}

/// The square of `x` with its sign.
fn signed_square(x: f32) -> f32 {
    if !(0.0_f32 <= x) {
        -ps2float::mul(x, x)
    } else {
        ps2float::mul(x, x)
    }
}

/// Whether the size of `x` is more than `limit`.
fn size_over(x: f32, limit: f32) -> bool {
    if !(0.0_f32 <= x) {
        limit < -x
    } else {
        limit < x
    }
}

/// Make the pose of the prop `prop` at the position `(x, y, z)` (turned by the angle of the prop times `scale` / 180): the matrix of the turn is made at `sp+16` and the pose at `sp+80`, whose position is then at `sp+128`.
fn pose_matrix(sp: i32, prop: i32, x: f32, y: f32, z: f32, scale: f32) {
    matrixTransRotY(sp + 16, x, y, z, ps2float::mul(lwc1(prop + 76), scale).pdiv(1.8e+02_f32));
    let model = lw(prop + 32);
    sceVu0MulMatrix(sp + 80, sp + 16, lw(model + 4) + (lw(lw(model + 92) + 20) << 6));
}

/// Put the position of the head of the target prop `target` (`drop` lower) at `sp+160` and the vector from the head of the player's prop to it at `out`.
fn vector_to_target(sp: i32, p: i32, target: i32, scale: f32, drop: f32, out: i32) {
    pose_matrix(sp, target, lwc1(target + 48), lwc1(target + 52), lwc1(target + 56), scale);
    let height = ps2float::sub(lwc1(sp + 132), drop);
    let depth = lwc1(sp + 136);
    swc1(sp + 160, lwc1(sp + 128));
    swc1(sp + 164, height);
    swc1(sp + 168, depth);
    pose_matrix(sp, lw(p + 384), lwc1(p + 140), lwc1(p + 144), lwc1(p + 148), scale);
    let up = ps2float::sub(lwc1(sp + 164), lwc1(sp + 132));
    let along = ps2float::sub(lwc1(sp + 168), lwc1(sp + 136));
    swc1(out, ps2float::sub(lwc1(sp + 160), lwc1(sp + 128)));
    swc1(out + 4, up);
    swc1(out + 8, along);
}

/// Look for the character the auto-aim aims at: the one seen by the player (the records at +820, 12 bytes each, the weight of the aim at +4) that is alive and has the best weight. Gives its number, or -1.
fn find_auto_aim_target(sp: i32, p: i32, chr: i32, gun: i32) -> i32 {
    let mut best = lwc1(0x3ad01c);
    let mut target = -1;
    if lhu(gun + 2) & 128 == 0 || autoaim_off_x() {
        return target;
    }
    if gunDoesAutoAim(chr) == 0 {
        return target;
    }
    let mut index = 0;
    if lw(g::lvnumplayers) + lw(g::numactivechars) > 0 {
        let scale = lwc1(0x3ad020);
        loop {
            let seen = lw(lw(g::currentplayer) + 820) + index * 12;
            let character = lw(g::chrinfo) + index * 0x1210;
            if lw(seen) != 0 {
                let prop = lw(character + 3020);
                if 0.0_f32 < ps2float::add(lwc1(prop + 520), lwc1(character + 12)) && best < lwc1(seen + 4) {
                    vector_to_target(sp, p, prop, scale, lwc1(0x3ad024), sp + 176);
                    playerAutoAimCalc(lw(g::currentplayer) + 188, sp + 176, seen);
                    let weight = lwc1(seen + 4);
                    if best < weight {
                        best = weight;
                        target = index;
                    }
                }
            }
            index += 1;
            if index >= lw(g::lvnumplayers) + lw(g::numactivechars) {
                break;
            }
        }
    }
    target
}

/// Aim the player at the character `target`: keep it as the target with the position it is at, and set the speeds of the looking that point the view at it.
fn aim_at_target(sp: i32, p: i32, target: i32) {
    let character = lw(g::chrinfo) + target * 0x1210;
    vector_to_target(sp, p, lw(character + 3020), lwc1(0x3ad028), lwc1(0x3ad02c), sp + 192);
    matrixVecRotAligned(&[lw(p + 1764) as i64, (sp + 192) as i64]);
    let target_prop = lw(character + 3020);
    swc1(p + 832, lwc1(sp + 160));
    let width = lwc1(p + 256);
    let limit = lwc1(0x3ad030);
    swc1(p + 836, lwc1(sp + 164));
    let height = lwc1(p + 252);
    let depth = lwc1(sp + 168);
    sw(p + 828, target_prop);
    swc1(p + 840, depth);
    swc1(p + 804, ((-((lwc1(sp + 192)).pdiv(lwc1(sp + 200)))).pdiv(width)).pdiv(limit));
    swc1(p + 808, (((lwc1(sp + 196)).pdiv(lwc1(sp + 200))).pdiv(height)).pdiv(limit));
}

/// Set the speeds of the looking for the sticks when the player is not aiming and there is no target: `look_input` is the stick for up and down, `scale` the speed.
fn set_free_look(p: i32, look_input: f32, scale: f32) {
    let axis = joyGetAxis(lw(lw(g::currentplayer)), lw(lw(g::currentplayer) + 76));
    swc1(p + 804, ps2float::mul(-look_input, scale));
    if lhu(lw(p + 24) + 2) & 1 == 0 {
        swc1(p + 808, ps2float::mul(axis, lwc1(0x3ad03c)));
    } else {
        swc1(p + 808, ps2float::mul(axis, scale));
    }
    sw(p + 828, 0);
}

/// The sticks while the player is not aiming: gives the turn and the look (up and down) for the movement. The speeds of the looking are set, to the auto-aim target if there is one.
fn read_free_sticks(sp: i32, p: i32, chr: i32, look_input: f32) -> (f32, f32) {
    let pad = lw(p);
    let turn = -joyGetAxis(pad, lw(p + 80));
    let mut look = stick_dead_zone(joyGetAxis(lw(lw(g::currentplayer)), lw(lw(g::currentplayer) + 76)));
    if lhu(lw(p + 24) + 2) & 1 != 0 {
        look = -look;
    }
    if lw(p + 612) == 0 && lw(p + 420) == 0 && lw(p + 296) != 7 {
        set_free_look(p, look_input, lwc1(0x3ad038));
        return (turn, look);
    }
    let target = find_auto_aim_target(sp, p, chr, lw(p + 24));
    if target >= 0 {
        aim_at_target(sp, p, target);
        return (turn, look);
    }
    set_free_look(p, look_input, lwc1(0x3ad034));
    (turn, look)
}

/// The sticks while the player is aiming: the squares of the looking sticks give the speeds of the looking, and the turn and the look for the movement.
fn read_aiming_sticks(p: i32) -> (f32, f32) {
    let flags_at = lw(lw(p + 384) + 352) + 2708;
    sw(flags_at, lw(flags_at) | 16);
    joySetCentreZone(lw(p), 0.0_f32);
    let up_down = -joyGetAxis(lw(lw(g::currentplayer)), lw(lw(g::currentplayer) + 96));
    let left_right = joyGetAxis(lw(lw(g::currentplayer)), lw(lw(g::currentplayer) + 92));
    joySetCentreZone(lw(lw(g::currentplayer)), 0.25_f32);
    let up_down = signed_square(up_down);
    let mut left_right = signed_square(left_right);
    if lhu(lw(p + 24) + 2) & 1 != 0 {
        left_right = -left_right;
    }
    swc1(p + 804, -up_down);
    swc1(p + 808, -left_right);
    (stick_dead_zone(up_down), stick_dead_zone(left_right))
}

/// Keep the angle round of a player riding a remote prop in the arc from `low` to `high` of the prop.
fn clamp_to_arc(p: i32, low: f32, high: f32) {
    let start = if low < high { low } else { ps2float::sub(low, 360.0_f32) };
    let middle = ps2float::add(high, ps2float::mul(ps2float::sub(360.0_f32, ps2float::sub(high, start)), 0.5_f32));
    let angle = lwc1(p + 276);
    let mut limit = true;
    if !(high < middle) {
        if high < angle {
            swc1(p + 276, high);
            limit = false;
        }
    } else if !(high < angle) {
        limit = false;
    }
    if limit && angle < middle {
        swc1(p + 276, high);
    }
    let angle = lwc1(p + 276);
    if !(middle < low) {
        if angle < low {
            swc1(p + 276, low);
            return;
        }
    } else if !(angle < low) {
        return;
    }
    if !(middle < angle) {
        return;
    }
    swc1(p + 276, low);
}

/// What the mode of the player gives for the rest of the tick.
struct Motion {
    /// The distance to go forward and to the side in this frame.
    forward: f32,
    side: f32,
    /// The sine and the cosine of the angle round.
    sin_yaw: f32,
    cos_yaw: f32,
}

/// The mode 0 (on the ground): move to the new position (kept at `sp+144`), down a steep polygon of the ground if the player stands on one, and fall if the ground is not there. Gives the flags of the move test.
fn move_on_ground(sp: i32, p: i32, chr: i32, m: &Motion) -> i32 {
    let mut slide_z = 0.0_f32;
    let mut slide_x = 0.0_f32;
    let mut steep = false;
    let ground = lw(p + 824);
    if ground != 0 {
        let corner = ground + 20;
        if lhu(ground + 6) & 2 != 0 {
            for axis in 0..3 {
                swc1(sp + 160 + axis * 4, ps2float::sub(lwc1(corner + axis * 4), lwc1(corner + axis * 4 - 12)));
                swc1(sp + 176 + axis * 4, ps2float::sub(lwc1(corner + 12 + axis * 4), lwc1(corner + axis * 4)));
            }
            let a_y = lwc1(sp + 164);
            let b_z = lwc1(sp + 184);
            let b_y = lwc1(sp + 180);
            let a_z = lwc1(sp + 168);
            let b_x = lwc1(sp + 176);
            let a_x = lwc1(sp + 160);
            let normal_x = ps2float::sub(ps2float::mul(a_y, b_z), ps2float::mul(b_y, a_z));
            let normal_y = ps2float::sub(ps2float::mul(a_z, b_x), ps2float::mul(b_z, a_x));
            let normal_z = ps2float::sub(ps2float::mul(a_x, b_y), ps2float::mul(b_x, a_y));
            let length = ps2float::add(ps2float::add(ps2float::mul(normal_x, normal_x), ps2float::mul(normal_y, normal_y)), ps2float::mul(normal_z, normal_z)).abs().psqrt();
            if normal_y < ps2float::mul(length, lwc1(0x3ad08c)) {
                slide_z = (normal_z).pdiv(length);
                slide_x = (normal_x).pdiv(length);
                steep = true;
            }
        }
    }
    let dt = frames();
    if !steep {
        let x = ps2float::add(lwc1(p + 140), m.sin_yaw * m.forward);
        swc1(sp + 144, x);
        let y = lwc1(p + 144);
        swc1(sp + 148, y);
        let z = ps2float::add(lwc1(p + 148), m.cos_yaw * m.forward);
        swc1(sp + 152, z);
        swc1(sp + 144, ps2float::add(x, ps2float::mul(lwc1(p + 212), m.side)));
        swc1(sp + 148, ps2float::add(y, ps2float::mul(lwc1(p + 216), m.side)));
        swc1(sp + 152, ps2float::add(z, ps2float::mul(lwc1(p + 220), m.side)));
    } else {
        swc1(sp + 144, ps2float::add(lwc1(p + 140), ps2float::mul(ps2float::mul(dt, slide_x), 8.0_f32).pdiv(60.0_f32)));
        swc1(sp + 148, ps2float::sub(lwc1(p + 144), 0.5_f32));
        swc1(sp + 152, ps2float::add(lwc1(p + 148), ps2float::mul(ps2float::mul(dt, slide_z), 8.0_f32).pdiv(60.0_f32)));
    }
    swc1(sp + 148, ps2float::sub(lwc1(sp + 148), ps2float::mul(dt, lwc1(0x3ad090))));
    chrAddFloorMoveToPos(lw(p), sp + 144, p + 276);
    let floor_before = lwc1(chr + 0x1198);
    let mut flags = moveTestDoSlide(lw(p), p + 140, sp + 144, p + 824, 1, lwc1(p + 104), lwc1(p + 132));
    let floor_after = lwc1(chr + 0x1198);
    let step = ps2float::sub(floor_before, floor_after);
    // The player follows a floor that moved a little.
    if !(0.0_f32 <= step) {
        if -step <= lwc1(0x3ad098) {
            swc1(sp + 148, floor_after);
            flags |= 1;
            swc1(p + 128, ps2float::add(lwc1(p + 128), step));
        }
    } else if step <= lwc1(0x3ad094) {
        swc1(sp + 148, floor_after);
        flags |= 1;
        swc1(p + 128, ps2float::add(lwc1(p + 128), step));
    }
    let mut drop;
    if flags & 1 != 0 {
        drop = lwc1(chr + 2864);
    } else {
        // Not on the ground: fall (the mode 2) with the speed that was moving the player.
        sw(p + 296, 2);
        if !steep {
            let dt = frames();
            let speed_z = (ps2float::add(m.cos_yaw * m.forward, ps2float::mul(lwc1(p + 220), m.side))).pdiv(dt);
            swc1(p + 856, (ps2float::add(m.sin_yaw * m.forward, ps2float::mul(lwc1(p + 212), m.side))).pdiv(dt));
            swc1(p + 864, speed_z);
            sw(chr + 2864, 0);
        } else {
            let fall = (-lwc1(p + 376)).pdiv(ticks_of(lw(p + 340)));
            swc1(p + 864, ps2float::mul(slide_z, 8.0_f32).pdiv(60.0_f32));
            swc1(p + 856, ps2float::mul(slide_x, 8.0_f32).pdiv(60.0_f32));
            swc1(chr + 2864, fall);
        }
        drop = lwc1(chr + 2864);
    }
    // The drop of the head after a fall settles.
    if !(drop < 0.0_f32) {
        let bounce = lwc1(p + 124);
        if !(bounce < 0.0_f32) {
            let dt = frames();
            if 0.0_f32 < bounce {
                let settled = ps2float::sub(bounce, ps2float::mul(dt, lwc1(0x3ad0a8)));
                swc1(p + 124, settled);
                if settled < 0.0_f32 {
                    swc1(p + 124, 0.0_f32);
                }
            }
            return flags;
        }
    }
    let limit = lwc1(0x3ad09c);
    let dt;
    if drop < limit {
        swc1(chr + 2864, limit);
        drop = limit;
        dt = frames();
    } else {
        dt = frames();
    }
    let speed = ps2float::add(drop, ps2float::mul(dt, lwc1(0x3ad0a0)));
    swc1(chr + 2864, speed);
    let bounce = ps2float::add(lwc1(p + 124), ps2float::mul(speed, dt));
    swc1(p + 124, bounce);
    if 0.0_f32 < bounce {
        swc1(p + 124, 0.0_f32);
    }
    let eye = lwc1(p + 116);
    let lowest = lwc1(0x3ad0a4);
    if ps2float::add(eye, lwc1(p + 124)) < lowest {
        swc1(p + 124, ps2float::sub(lowest, eye));
    }
    flags
}

/// The mode 1 (pushed): move by the push (at +300), a part of it each frame, and fall when the push is spent. Gives the flags of the move test.
fn move_pushed(sp: i32, p: i32) -> i32 {
    swc1(sp + 160, lwc1(p + 300));
    swc1(sp + 164, lwc1(p + 304));
    swc1(sp + 168, lwc1(p + 308));
    let push_x = lwc1(sp + 160);
    let push_y = lwc1(sp + 164);
    let push_z = lwc1(sp + 168);
    let size = ps2float::add(ps2float::add(ps2float::mul(push_x, push_x), ps2float::mul(push_y, push_y)), ps2float::mul(push_z, push_z)).abs().psqrt();
    let dt = frames();
    let limit = ps2float::mul(dt, lwc1(0x3ad0ac));
    let mut part = size;
    if limit < size {
        part = limit;
    }
    let fraction = (part).pdiv(size);
    let step_x = ps2float::mul(lwc1(sp + 160), fraction);
    let step_y = ps2float::mul(lwc1(sp + 164), fraction);
    let step_z = ps2float::mul(lwc1(sp + 168), fraction);
    swc1(sp + 160, step_x);
    swc1(sp + 164, step_y);
    swc1(sp + 168, step_z);
    swc1(sp + 144, ps2float::add(lwc1(p + 140), step_x));
    swc1(sp + 148, ps2float::add(lwc1(p + 144), step_y));
    swc1(sp + 152, ps2float::add(lwc1(p + 148), step_z));
    let left = ps2float::sub(lwc1(p + 300), step_x);
    swc1(p + 300, left);
    let left = ps2float::sub(left, lwc1(sp + 160));
    swc1(p + 300, left);
    swc1(p + 300, ps2float::sub(left, lwc1(sp + 160)));
    if fraction == 1.0_f32 {
        let speed_x = lwc1(sp + 160);
        sw(p + 296, 2);
        swc1(p + 856, (speed_x).pdiv(dt));
        swc1(p + 864, (lwc1(sp + 168)).pdiv(dt));
    }
    swc1(p + 124, ps2float::mul(lwc1(p + 124), lwc1(0x3ad0b0)));
    moveTestDoSlide(lw(p), p + 140, sp + 144, p + 824, 1, lwc1(p + 104), lwc1(p + 132))
}

/// The mode 2 (falling): the speed down grows, the player moves, and when it lands it takes the fall (a rumble by how far it fell) or dies. Gives the flags of the move test.
fn move_falling(sp: i32, p: i32, chr: i32) -> i32 {
    let floor = lwc1(0x3ad0b8);
    let speed = ps2float::sub(lwc1(chr + 2864), ps2float::mul(frames(), lwc1(0x3ad0b4)));
    swc1(chr + 2864, speed);
    if speed < floor {
        swc1(chr + 2864, floor);
    }
    let dt = frames();
    let damping = lwc1(0x3ad0bc);
    swc1(sp + 144, ps2float::add(lwc1(p + 140), ps2float::mul(lwc1(p + 856), dt)));
    swc1(sp + 148, ps2float::add(lwc1(p + 144), ps2float::mul(lwc1(chr + 2864), dt)));
    swc1(sp + 152, ps2float::add(lwc1(p + 148), ps2float::mul(lwc1(p + 864), dt)));
    swc1(p + 124, ps2float::mul(lwc1(p + 124), damping));
    let flags = moveTestDoSlide(lw(p), p + 140, sp + 144, p + 824, 1, lwc1(p + 104), lwc1(p + 132));
    let x = lwc1(sp + 144);
    let body = lw(p + 384);
    swc1(p + 140, x);
    let character = lw(body + 352);
    let y = lwc1(sp + 148);
    let bounce = lwc1(p + 124);
    let head = ps2float::add(y, lwc1(p + 116));
    let dt = frames();
    sw(character + 2708, (lw(character + 2708) | 1) & -33);
    swc1(p + 144, y);
    let z = lwc1(sp + 152);
    swc1(p + 152, x);
    swc1(p + 156, ps2float::add(head, bounce));
    swc1(p + 160, z);
    swc1(p + 148, z);
    StatsAdd(52, lw(chr), ps2float::mul(dt, lwc1(0x3ad0c0)));
    if flags & 1 == 0 {
        return flags;
    }
    // Landed: how far did it fall?
    let started = lwc1(chr + 2956);
    if !(started == lwc1(g::shakeangz_172 + 4)) {
        let fallen = ps2float::sub(started, lwc1(chr + 0x1198));
        if 2.0_f32 < fallen && fallen < 10.0_f32 {
            swc1(p + 1780, lwc1(0x3ad0c4));
            swc1(p + 1776, (ps2float::sub(fallen, 2.0_f32)).pdiv(12.0_f32));
        } else if 10.0_f32 <= fallen {
            swc1(p + 1776, 1.0_f32);
            swc1(p + 1780, lwc1(0x3ad0c8));
        }
    }
    if !(lwc1(lw(p + 384) + 520) <= 0.0_f32) {
        sw(p + 296, 0);
        return flags;
    }
    sw(p + 296, 4);
    playerOnGroundDie(p, 0, 0, 0, -1, 0);
    flags
}

/// The mode 3 (swimming): move by the sticks, sinking a little. Gives the flags of the move test.
fn move_swimming(sp: i32, p: i32, m: &Motion) -> i32 {
    let x = ps2float::add(lwc1(p + 140), m.sin_yaw * m.forward);
    let sink = ps2float::mul(frames(), lwc1(0x3ad0cc));
    swc1(sp + 144, x);
    let y = lwc1(p + 144);
    swc1(sp + 148, y);
    let z = ps2float::add(lwc1(p + 148), m.cos_yaw * m.forward);
    swc1(sp + 152, z);
    swc1(sp + 144, ps2float::add(x, ps2float::mul(lwc1(p + 212), m.side)));
    let y = ps2float::add(y, ps2float::mul(lwc1(p + 216), m.side));
    swc1(sp + 148, y);
    let along = ps2float::mul(lwc1(p + 220), m.side);
    swc1(sp + 148, ps2float::sub(y, sink));
    swc1(sp + 152, ps2float::add(z, along));
    swc1(p + 124, ps2float::mul(lwc1(p + 124), lwc1(0x3ad0d0)));
    let flags = moveTestDoSlide(lw(p), p + 140, sp + 144, p + 824, 1, lwc1(p + 104), lwc1(p + 132));
    if flags & 1 != 0 {
        sw(p + 296, 0);
    }
    flags
}

/// The mode 4 (dead): the view sinks to the floor; a button brings the player back (or makes a new spawn in the games that have them).
fn move_dead(sp: i32, p: i32, chr: i32) {
    let body = lw(p + 384);
    let model = lw(body + 32);
    swc1(sp + 144, lwc1(model + 128));
    swc1(sp + 148, lwc1(model + 132));
    swc1(sp + 152, lwc1(model + 136));
    matrixRotY(sp + 208, ps2float::mul(lwc1(body + 76), lwc1(0x3ad0d4)).pdiv(1.8e+02_f32));
    matrixVecRotAligned(&[(sp + 208) as i64, (sp + 144) as i64]);
    let height = lwc1(sp + 148);
    let depth = lwc1(sp + 152);
    swc1(sp + 144, ps2float::add(lwc1(sp + 144), lwc1(body + 48)));
    swc1(sp + 148, ps2float::add(height, lwc1(body + 52)));
    swc1(sp + 152, ps2float::add(depth, lwc1(body + 56)));
    moveTestDoSlide(lw(p), body + 48, sp + 144, 0, 1, lwc1(p + 104), lwc1(p + 132));
    swc1(body + 48, lwc1(sp + 144));
    let limit = lwc1(0x3ad0d8);
    swc1(body + 52, lwc1(sp + 148));
    swc1(body + 56, lwc1(sp + 152));
    swc1(sp + 148, ps2float::add(lwc1(sp + 148), ps2float::add(lwc1(p + 116), lwc1(p + 124))));
    let room = moveFindRoom(sp + 144, limit, limit);
    let flags = lw(body + 16);
    sw(body + 12, room);
    sw(body + 16, flags & -33);
    if lw(chr + 0x11b0) != 0 {
        playerRespawn();
    }
    if joyNewBut(lw(p)) & lw(p + 36) != 0 {
        let kind = lw(g::curgameparms + 72);
        if kind != 7 {
            let spawn = if kind != 6 {
                true
            } else {
                enemyGetNumBases() > 0
            };
            if spawn {
                specialfxNewSpawn(5, lw(p + 384) + 48, -1, chr, 0);
                sw(chr + 0x11b0, 1);
            }
        }
    }
    sw(lw(lw(p + 384) + 352) + 2708, 32);
    playerDeathView();
}

/// The step of an angle `angle` to the angle `wanted` that is at most `step`: gives the new angle, which is `wanted` if it is near.
fn turn_toward(angle: f32, difference: f32, wanted: f32, step: f32) -> f32 {
    if !(0.0_f32 <= difference) {
        if !(-difference < step) {
            return if !(difference < 0.0_f32) { ps2float::sub(angle, step) } else { ps2float::add(angle, step) };
        }
    } else if !(difference < step) {
        return if !(difference < 0.0_f32) { ps2float::sub(angle, step) } else { ps2float::add(angle, step) };
    }
    wanted
}

/// The mode 6 (going into a remote prop): turn to the way the prop is stood at and go to its stand.
fn move_into_prop(sp: i32, p: i32, chr: i32) {
    let prop = lw(chr + 0x11b4);
    let yaw = lwc1(p + 276);
    let info = lw(prop + 352);
    let wanted_yaw = lwc1(info + 108);
    let yaw_difference = anglediff(wanted_yaw, yaw);
    let yaw_step = ps2float::mul(frames(), lwc1(0x3ad0dc));
    remoteGetStandPos(prop, sp + 160, lwc1(info + 108));
    swc1(p + 276, turn_toward(lwc1(p + 276), yaw_difference, lwc1(info + 108), yaw_step));
    let pitch = lwc1(p + 284);
    let pitch_difference = anglediff(-lwc1(info + 112), pitch);
    let pitch_step = ps2float::mul(frames(), lwc1(0x3ad0e0));
    let new_pitch = if !(0.0_f32 <= pitch_difference) {
        if -pitch_difference < pitch_step {
            -lwc1(info + 112)
        } else {
            let now = lwc1(p + 284);
            if !(pitch_difference < 0.0_f32) { ps2float::sub(now, pitch_step) } else { ps2float::add(now, pitch_step) }
        }
    } else if pitch_difference < pitch_step {
        -lwc1(info + 112)
    } else {
        let now = lwc1(p + 284);
        if !(pitch_difference < 0.0_f32) { ps2float::sub(now, pitch_step) } else { ps2float::add(now, pitch_step) }
    };
    swc1(p + 284, new_pitch);
    let dt = frames();
    let k = lwc1(0x3ad0e4);
    swc1(p + 140, lwc1(sp + 160));
    let limit = lwc1(0x3ad0e8);
    let bounce = ps2float::add(lwc1(p + 124), ps2float::mul(dt, k));
    swc1(p + 144, lwc1(sp + 164));
    let z = lwc1(sp + 168);
    swc1(p + 124, bounce);
    swc1(p + 148, z);
    if limit < bounce {
        swc1(p + 124, limit);
    }
    let bounce = lwc1(p + 124);
    let head = ps2float::add(lwc1(p + 144), lwc1(p + 116));
    let z = lwc1(p + 148);
    swc1(p + 152, lwc1(p + 140));
    swc1(p + 160, z);
    swc1(p + 156, ps2float::add(head, bounce));
}

/// The mode 7 (riding a remote prop): the player is where the prop puts it, and turns with it. Gives the flags of the move test.
fn move_on_prop(sp: i32, p: i32, chr: i32) -> i32 {
    let prop = lw(chr + 0x11b4);
    remoteGetStandPos(prop, sp + 176, lwc1(p + 276));
    swc1(sp + 180, ps2float::sub(lwc1(sp + 180), 1.0_f32));
    let flags = moveTest(lw(p), p + 140, sp + 176, 0, p + 824, 1, lwc1(p + 104), lwc1(p + 132));
    if flags & 2 != 0 {
        let angle = remoteGetStandAngle(prop, sp + 176);
        swc1(p + 276, angle);
    }
    let limit = lwc1(0x3ad0f0);
    let bounce = ps2float::add(lwc1(p + 124), ps2float::mul(frames(), lwc1(0x3ad0ec)));
    swc1(p + 124, bounce);
    if limit < bounce {
        swc1(p + 124, limit);
    }
    let x = lwc1(sp + 176);
    let eye = lwc1(p + 116);
    swc1(p + 140, x);
    let bounce = lwc1(p + 124);
    let y = lwc1(sp + 180);
    swc1(p + 144, y);
    let z = lwc1(sp + 184);
    swc1(p + 152, x);
    swc1(p + 160, z);
    swc1(p + 148, z);
    swc1(p + 156, ps2float::add(ps2float::add(y, eye), bounce));
    flags
}

/// Everything the tick does for a player that is not stopped by the debug menu: the buttons and sticks, the movement and the frame of the view. Gives the flags of the move test of the movement.
fn update_player(sp: i32, p: i32, chr: i32) -> i32 {
    let mut flags_fp = 0;
    let look_input = -joyGetAxis(lw(p), lw(p + 96));
    joyGetAxis(lw(p), lw(p + 92));

    // The speed the player moved at, and the place it was.
    let dx = ps2float::sub(lwc1(p + 140), lwc1(p + 344));
    let dy = ps2float::sub(lwc1(p + 144), lwc1(p + 348));
    let dz = ps2float::sub(lwc1(p + 148), lwc1(p + 352));
    let dx2 = ps2float::mul(dx, dx);
    let dz2 = ps2float::mul(dz, dz);
    swc1(p + 376, ps2float::add(ps2float::add(dx2, ps2float::mul(dy, dy)), dz2).abs().psqrt());
    swc1(p + 380, ps2float::add(dx2, dz2).abs().psqrt());
    let frames_before = lw(p + 292);
    for (to, from) in [(344, 140), (348, 144), (352, 148), (356, 152), (360, 156), (364, 160), (368, 276), (372, 284)] {
        swc1(p + to, lwc1(p + from));
    }
    if ticks_of(frames_before) > 0.0_f32 {
        sw(p + 340, frames_before);
    }

    // The run button.
    let mode = lw(p + 296);
    store_update_ticks(p + 292);
    if mode == 4 || lw(g::padlinkedit) == 1 || mode == 6 || mode == 7 {
        sw(p + 20, lw(p + 20) & -2);
    } else if lhu(lw(p + 24) + 2) & 4 == 0 {
        if joyBut(lw(p)) & lw(p + 56) != 0 {
            sw(p + 20, lw(p + 20) | 1);
        } else {
            sw(p + 20, lw(p + 20) & -2);
        }
    } else if joyNewBut(lw(p)) & lw(p + 56) != 0 {
        sw(p + 20, lw(p + 20) ^ 1);
    }
    let dt = frames();
    if lw(p + 296) != 4 {
        StatsAdd(65, lw(p), ps2float::mul(dt, lwc1(0x3ad014)));
    }

    // The aim button.
    let flags_before = lw(p + 20);
    if lw(p + 296) != 4 {
        let gun = lw(p + 24);
        if joyNewBut(lw(p)) & lw(p + 44) != 0 {
            let flags = lw(p + 20);
            sw(p + 20, if lhu(gun + 2) & 2 == 0 { flags | 2 } else { flags ^ 2 });
        } else if lhu(lw(p + 24) + 2) & 2 == 0 && joyReleaseBut(lw(p)) & lw(p + 44) != 0 {
            sw(p + 20, lw(p + 20) & -3);
        }
    } else {
        sw(p + 20, flags_before & -3);
    }
    let flags_after = lw(p + 20);
    if flags_after != flags_before {
        if flags_after & 2 == 0 {
            swc1(p + 268, lwc1(p + 264));
            swc1(p + 264, 60.0_f32);
        } else {
            swc1(p + 264, lwc1(p + 268));
        }
    }

    // The zoom, and the field of view that follows it.
    let wanted;
    if lw(p + 20) & 2 == 0 {
        wanted = lwc1(p + 264);
    } else {
        let dt = frames();
        if joyBut(lw(p)) & lw(p + 48) != 0 {
            swc1(p + 264, ps2float::sub(lwc1(p + 264), dt));
        }
        let dt = frames();
        if joyBut(lw(p)) & lw(p + 52) != 0 {
            swc1(p + 264, ps2float::add(lwc1(p + 264), dt));
        }
        let least = lwc1(p + 272);
        if lwc1(p + 264) < least {
            swc1(p + 264, least);
        }
        if 60.0_f32 < lwc1(p + 264) {
            swc1(p + 264, 60.0_f32);
        }
        let dt = frames();
        if lw(p + 20) & 2 != 0 {
            StatsAdd(56, lw(chr), ps2float::mul(dt, lwc1(0x3ad018)));
        }
        wanted = lwc1(p + 264);
    }
    let shown = lwc1(p + 260);
    if wanted != shown {
        let limit = ps2float::add(frames(), frames());
        let mut step = ps2float::sub(wanted, shown);
        if limit < step {
            step = limit;
        }
        let least = -limit;
        if step < least {
            step = least;
        }
        swc1(p + 260, ps2float::add(shown, step));
    }

    // The sticks.
    let aiming = lw(p + 296) != 4 && lw(p + 20) & 2 != 0;
    sw(p + 800, aiming as i32);
    let (mut turn, mut look) = if !aiming { read_free_sticks(sp, p, chr, look_input) } else { read_aiming_sticks(p) };
    let smooth = lwc1(0x3ad040);
    let dt = frames();
    let side_before = lwc1(p + 812);
    let look_before = lwc1(p + 816);
    let look_smoothed = ps2float::add(look_before, ps2float::mul(ps2float::mul(ps2float::sub(lwc1(p + 808), look_before), smooth), dt));
    swc1(p + 812, ps2float::add(side_before, ps2float::mul(ps2float::mul(ps2float::sub(lwc1(p + 804), side_before), smooth), dt)));
    swc1(p + 816, look_smoothed);
    let (mut walk, mut strafe) = if !aiming {
        (joyGetAxis(lw(p), lw(p + 68)), joyGetAxis(lw(lw(g::currentplayer)), lw(lw(g::currentplayer) + 72)))
    } else {
        (joyGetAxis(lw(p), lw(p + 84)), joyGetAxis(lw(lw(g::currentplayer)), lw(lw(g::currentplayer) + 88)))
    };

    // The height of the eyes: lower when the player does not run.
    let dt = frames();
    if lw(p + 20) & 1 != 0 {
        StatsAdd(54, lw(chr), ps2float::mul(dt, lwc1(0x3ad044)));
        if walk == 0.0_f32 && strafe == 0.0_f32 {
            swc1(p + 120, ps2float::sub(lwc1(p + 108), lwc1(p + 112)));
        } else {
            swc1(p + 120, ps2float::sub(ps2float::add(lwc1(p + 108), lwc1(0x3ad048)), lwc1(p + 112)));
        }
    } else {
        swc1(p + 120, ps2float::sub(lwc1(p + 104), lwc1(p + 112)));
    }
    let wanted_height = lwc1(p + 120);
    let height = lwc1(p + 116);
    if wanted_height != height {
        let mut step = ps2float::sub(wanted_height, height);
        let limit = ps2float::mul(frames(), lwc1(0x3ad04c));
        if limit < step {
            step = limit;
        }
        let least = -limit;
        if step < least {
            step = least;
        }
        swc1(p + 116, ps2float::add(height, ps2float::mul(step, lwc1(0x3ad050))));
    }
    let character = lw(lw(p + 384) + 352);
    let spawn = lw(character + 0x11a4);
    if spawn != 0 && lw(spawn + 4) == 2 {
        specialfxFreeSpawn(spawn);
        sw(character + 0x11a4, 0);
    }
    let stance = lw(p + 288);
    if lw(p + 20) & 1 != 0 {
        turn = ps2float::mul(turn, 0.75_f32);
        walk = ps2float::mul(walk, 0.75_f32);
        strafe = ps2float::mul(strafe, 0.75_f32);
    }

    // The stance: 0 standing, 1 walking, 2 running (for the sticks).
    if stance == 1 {
        if walk == 0.0_f32 {
            sw(p + 288, 0);
        }
    } else if stance == 2 {
        if !(look == 0.0_f32) {
            sw(p + 288, 1);
        }
    } else if stance == 0 {
        if !(look == 0.0_f32) {
            if size_over(walk, 0.0_f32) {
                sw(p + 288, 1);
            }
        } else if size_over(walk, 0.75_f32) {
            sw(p + 288, 2);
        }
    }

    // Turn and look.
    if lw(p + 296) != 6 {
        let dt = frames();
        let stance = lw(p + 288);
        swc1(p + 276, ps2float::add(lwc1(p + 276), ps2float::mul(ps2float::mul(look_scale(ps2float::mul(turn, lwc1(0x3ad054))), ((lwc1(p + 260)).pdiv(60.0_f32))), dt)));
        if stance == 2 && lhu(lw(p + 24) + 2) & 8 != 0 {
            let pitch = lwc1(p + 284);
            let eased = ps2float::sub(pitch, ps2float::mul(ps2float::mul(pitch, lwc1(0x3ad058)), dt));
            swc1(p + 284, eased);
            let small = if !(0.0_f32 <= eased) { -eased < 0.5_f32 } else { eased < 0.5_f32 };
            if small {
                sw(p + 288, 1);
            }
        } else {
            swc1(p + 284, ps2float::add(lwc1(p + 284), ps2float::mul(ps2float::mul(look_scale(ps2float::mul(look, lwc1(0x3ad05c))), ((lwc1(p + 260)).pdiv(60.0_f32))), frames())));
        }
        if lwc1(p + 276) < 0.0_f32 {
            loop {
                let wrapped = ps2float::add(lwc1(p + 276), 360.0_f32);
                swc1(p + 276, wrapped);
                if !(wrapped < 0.0_f32) {
                    break;
                }
            }
        }
        if 360.0_f32 <= lwc1(p + 276) {
            loop {
                let wrapped = ps2float::sub(lwc1(p + 276), 360.0_f32);
                swc1(p + 276, wrapped);
                if !(360.0_f32 <= wrapped) {
                    break;
                }
            }
        }
        if lw(p + 296) != 7 {
            if lwc1(p + 284) < -50.0_f32 {
                swc1(p + 284, -50.0_f32);
            }
            if 50.0_f32 < lwc1(p + 284) {
                swc1(p + 284, 50.0_f32);
            }
        } else {
            let prop = lw(chr + 0x11b4);
            let info = lw(prop + 352);
            let base = lwc1(prop + 76);
            let low = ps2float::add(lwc1(info + 40), base);
            let high = ps2float::add(lwc1(info + 44), base);
            if lwc1(p + 284) < -10.0_f32 {
                swc1(p + 284, -10.0_f32);
            }
            if 30.0_f32 < lwc1(p + 284) {
                swc1(p + 284, 30.0_f32);
            }
            if low != high {
                clamp_to_arc(p, low, high);
            }
        }
    }

    // The shake of the walk.
    let pad_linked = lw(g::padlinkedit);
    let bob = lwc1(p + 312);
    let dt = frames();
    let phase = ps2float::add(lwc1(p + 324), ps2float::mul(ps2float::mul(bob, 0.5_f32), dt));
    swc1(p + 324, phase);
    if 360.0_f32 <= phase {
        sw(p + 324, 0);
    }
    let phase = ps2float::add(lwc1(p + 328), ps2float::mul(ps2float::mul(lwc1(p + 316), 0.5_f32), frames()));
    swc1(p + 328, phase);
    if 360.0_f32 < phase {
        sw(p + 328, 0);
    }
    let dt = frames();
    let amount;
    if !((ps2float::mul(dt, 4.0_f32).pdiv(60.0_f32)) < lwc1(p + 376)) {
        amount = lwc1(p + 320);
    } else {
        let size = lwc1(p + 320);
        if size < 1.0_f32 {
            swc1(p + 320, ps2float::add(size, ps2float::mul(dt, lwc1(0x3ad060))));
        }
        let speed = lwc1(p + 312);
        let dt = frames();
        if speed < 3.0_f32 {
            swc1(p + 312, ps2float::add(speed, ps2float::mul(dt, lwc1(0x3ad064))));
        }
        amount = lwc1(p + 320);
    }
    let dt = frames();
    if 0.5_f32 < amount {
        swc1(p + 320, ps2float::sub(amount, ps2float::mul(dt, lwc1(0x3ad068))));
    }
    let speed = lwc1(p + 312);
    let dt = frames();
    if 1.0_f32 < speed {
        swc1(p + 312, ps2float::sub(speed, ps2float::mul(dt, lwc1(0x3ad06c))));
    }
    swc1(p + 316, ps2float::mul(lwc1(p + 312), lwc1(0x3ad070)));
    if pad_linked == 0 && lw(g::propedit) == 0 {
        let scale = lwc1(0x3ad074);
        let sideways = ps2float::mul(lwc1(p + 320), 0.5_f32);
        swc1(p + 332, ps2float::mul(sideways, sinf(ps2float::mul(lwc1(p + 324), scale).pdiv(1.8e+02_f32))));
        swc1(p + 336, ps2float::mul(lwc1(p + 320), sinf(ps2float::mul(lwc1(p + 328), scale).pdiv(1.8e+02_f32))));
    }

    // The directions the player looks along and moves by.
    let scale = lwc1(0x3ad078);
    let bob_yaw = lwc1(p + 332);
    let cos_look_yaw = cosf(ps2float::mul(ps2float::add(lwc1(p + 276), bob_yaw), scale).pdiv(1.8e+02_f32));
    let sin_look_yaw = sinf(ps2float::mul(ps2float::add(lwc1(p + 276), lwc1(p + 332)), scale).pdiv(1.8e+02_f32));
    let cos_look_pitch = cosf(ps2float::mul(ps2float::add(lwc1(p + 284), lwc1(p + 336)), scale).pdiv(1.8e+02_f32));
    let sin_look_pitch = sinf(ps2float::mul(ps2float::add(lwc1(p + 284), lwc1(p + 336)), scale).pdiv(1.8e+02_f32));
    let yaw = lwc1(p + 276);
    swc1(p + 192, sin_look_pitch);
    swc1(p + 188, cos_look_pitch * sin_look_yaw);
    swc1(p + 196, cos_look_pitch * cos_look_yaw);
    let cos_yaw = cosf(ps2float::mul(yaw, scale).pdiv(1.8e+02_f32));
    let sin_yaw = sinf(ps2float::mul(lwc1(p + 276), scale).pdiv(1.8e+02_f32));
    cosf(ps2float::mul(lwc1(p + 284), scale).pdiv(1.8e+02_f32));
    sinf(ps2float::mul(lwc1(p + 284), scale).pdiv(1.8e+02_f32));
    let sway = lwc1(0x3ad07c);
    let look_x = lwc1(p + 188);
    let previous_y = lwc1(p + 228);
    let sway_look = ps2float::mul(ps2float::mul(lwc1(p + 816), sway), lwc1(p + 252));
    let dt = frames();
    let sway_side = ps2float::mul(ps2float::mul(ps2float::mul(lwc1(p + 812), sway), lwc1(p + 256)), lwc1(p + 240));
    swc1(p + 212, -cos_yaw);
    sw(p + 216, 0);
    swc1(p + 220, sin_yaw);
    let step = ps2float::mul(dt, lwc1(0x3ad080));
    swc1(chr + 2976, ps2float::add(ps2float::add(look_x, sway_side), ps2float::mul(sway_look, previous_y)));
    swc1(
        chr + 2980,
        ps2float::add(ps2float::add(lwc1(p + 192), ps2float::mul(ps2float::mul(ps2float::mul(lwc1(p + 812), sway), lwc1(p + 256)), lwc1(p + 244))), ps2float::mul(ps2float::mul(ps2float::mul(lwc1(p + 816), sway), lwc1(p + 252)), lwc1(p + 232))),
    );
    swc1(
        chr + 2984,
        ps2float::add(ps2float::add(lwc1(p + 196), ps2float::mul(ps2float::mul(ps2float::mul(lwc1(p + 812), sway), lwc1(p + 256)), lwc1(p + 248))), ps2float::mul(ps2float::mul(ps2float::mul(lwc1(p + 816), sway), lwc1(p + 252)), lwc1(p + 236))),
    );

    // The sticks move by no more than the limits that follow them, and make the distances for this frame.
    let walk_limit = lwc1(p + 1784);
    if ps2float::add(walk_limit, step) < walk {
        walk = ps2float::add(walk_limit, step);
    }
    if walk < ps2float::sub(walk_limit, step) {
        walk = ps2float::sub(walk_limit, step);
    }
    let speed_scale = lwc1(0x3ad084);
    let strafe_limit = lwc1(p + 1788);
    let strafe_step = ps2float::mul(dt, speed_scale);
    if ps2float::add(strafe_limit, strafe_step) < strafe {
        strafe = ps2float::add(strafe_limit, strafe_step);
    }
    if strafe < ps2float::sub(strafe_limit, strafe_step) {
        strafe = ps2float::sub(strafe_limit, strafe_step);
    }
    let mut backward = 0.0_f32;
    let side_speed = ps2float::mul(strafe, lwc1(0x3ad088));
    swc1(p + 1788, strafe);
    let forward = ps2float::mul(ps2float::mul(walk, speed_scale), dt);
    swc1(p + 1784, walk);
    let mut forwards_stat = backward;
    let side = ps2float::mul(side_speed, dt);
    if !(backward < forward) {
        backward = -forward;
    } else {
        forwards_stat = forward;
    }
    StatsAdd(47, lw(chr), forwards_stat);
    StatsAdd(48, lw(chr), backward);
    let side_dp = fptodp(side);
    let side_sign = dpcmp(side_dp, 0);
    if side_sign >= 0 {
        StatsAdd(50, lw(chr), side);
    } else {
        StatsAdd(50, lw(chr), -side);
    }
    let motion = Motion { forward, side, sin_yaw, cos_yaw };

    // The movement of the mode.
    if lw(p + 296) != 2 {
        swc1(chr + 2956, lwc1(chr + 0x1198));
    }
    let mut mode = lw(p + 296);
    if mode != 4 {
        let up = lwc1(p + 192);
        let along = lwc1(p + 196);
        swc1(p + 1796, lwc1(p + 188));
        swc1(p + 1800, up);
        swc1(p + 1804, along);
        sw(p + 1792, 0);
        mode = lw(p + 296);
    }
    if (mode as u32) < 8 {
        let mut settle = false;
        match mode {
            0 => {
                flags_fp = move_on_ground(sp, p, chr, &motion);
                settle = true;
            }
            1 => {
                flags_fp = move_pushed(sp, p);
                settle = true;
            }
            2 => flags_fp = move_falling(sp, p, chr),
            3 => {
                flags_fp = move_swimming(sp, p, &motion);
                settle = true;
            }
            4 => move_dead(sp, p, chr),
            5 => {}
            6 => move_into_prop(sp, p, chr),
            _ => flags_fp = move_on_prop(sp, p, chr),
        }
        if settle {
            let x = lwc1(sp + 144);
            let character = lw(lw(p + 384) + 352);
            swc1(p + 140, x);
            let y = lwc1(sp + 148);
            let head = ps2float::add(y, lwc1(p + 116));
            let bounce = lwc1(p + 124);
            let character_flags = lw(character + 2708);
            swc1(p + 144, y);
            let z = lwc1(sp + 152);
            sw(character + 2708, (character_flags | 1) & -33);
            swc1(p + 152, x);
            swc1(p + 156, ps2float::add(head, bounce));
            swc1(p + 160, z);
            swc1(p + 148, z);
        }
    }

    // The player is lifted out of the ground that pushed it up (the offset at +128 eases away).
    let lift = lwc1(p + 128);
    if !(lift == 0.0_f32) {
        let ticks = lw(g::updateframesi);
        if ticks > 0 {
            let mut eased = lift;
            let mut left = ticks;
            loop {
                left -= 1;
                eased = ps2float::sub(eased, ps2float::mul(eased, 0.25_f32));
                if left == 0 {
                    break;
                }
            }
            swc1(p + 128, eased);
        }
        if 0.0_f32 < lift && lwc1(p + 128) < 0.0_f32 {
            sw(p + 128, 0);
        } else if lift < 0.0_f32 && 0.0_f32 < lwc1(p + 128) {
            sw(p + 128, 0);
        }
        let head = lwc1(p + 156);
        let floor = lwc1(p + 144);
        let margin = lwc1(0x3ad0f4);
        if ps2float::add(head, lwc1(p + 128)) < ps2float::add(floor, margin) {
            swc1(p + 128, ps2float::add(ps2float::sub(floor, head), margin));
        }
        let head = lwc1(p + 156);
        swc1(p + 156, ps2float::add(head, lwc1(p + 128)));
    }
    playerselectanim(walk, strafe, turn);

    // The portals the player crossed give the room.
    let mut room = lw(p + 796);
    let mut next = if room == 0 { 1 } else { room };
    swc1(sp + 272, ps2float::sub(lwc1(p + 152), lwc1(p + 356)));
    swc1(sp + 276, ps2float::sub(lwc1(p + 156), lwc1(p + 360)));
    swc1(sp + 280, ps2float::sub(lwc1(p + 160), lwc1(p + 364)));
    let mut previous = 0;
    room = 0;
    if next != 0 {
        loop {
            room = bgPortalCrossed(p + 356, sp + 272, next, previous, 0);
            previous = next;
            next = room;
            if room == previous {
                break;
            }
        }
    }

    // The frame of the view of the player.
    sw(p + 164, 0);
    let look_z = lwc1(p + 196);
    let zero = lwc1(p + 164);
    let look_y = lwc1(p + 192);
    let look_x = lwc1(p + 188);
    let y_zero = ps2float::mul(look_y, zero);
    let top = lwc1(p + 156);
    let right_x = ps2float::add(lwc1(p + 152), look_x);
    let behind = lwc1(p + 160);
    let up_x = ps2float::sub(look_z, y_zero);
    let up_y = ps2float::sub(ps2float::mul(look_x, zero), ps2float::mul(look_z, zero));
    sw(p + 796, room);
    let up_z = ps2float::sub(y_zero, look_x);
    swc1(p + 176, right_x);
    swc1(p + 168, 1.0_f32);
    swc1(p + 244, up_y);
    sw(p + 172, 0);
    swc1(p + 240, up_x);
    swc1(p + 248, up_z);
    swc1(p + 180, ps2float::add(top, look_y));
    swc1(p + 184, ps2float::add(behind, look_z));
    let length = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(up_x, up_x), ps2float::mul(up_y, up_y)), ps2float::mul(up_z, up_z)).abs().psqrt());
    let look_z = lwc1(p + 196);
    let look_x = lwc1(p + 188);
    let look_y = lwc1(p + 192);
    let unit_z = ps2float::mul(lwc1(p + 248), length);
    let unit_x = ps2float::mul(lwc1(p + 240), length);
    let unit_y = ps2float::mul(lwc1(p + 244), length);
    swc1(p + 248, unit_z);
    swc1(p + 240, unit_x);
    swc1(p + 244, unit_y);
    let side_x = ps2float::sub(ps2float::mul(look_z, unit_x), ps2float::mul(look_x, unit_z));
    let side_y = ps2float::sub(ps2float::mul(look_y, unit_z), ps2float::mul(look_z, unit_y));
    swc1(p + 232, side_x);
    swc1(p + 228, side_y);
    let side_z = ps2float::sub(ps2float::mul(look_x, unit_y), ps2float::mul(look_y, unit_x));
    swc1(p + 236, side_z);
    let length = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(side_y, side_y), ps2float::mul(side_x, side_x)), ps2float::mul(side_z, side_z)).abs().psqrt());
    let mode = lw(p + 296);
    let basis_y = ps2float::mul(lwc1(p + 228), length);
    let basis_x = ps2float::mul(lwc1(p + 232), length);
    swc1(p + 236, ps2float::mul(lwc1(p + 236), length));
    swc1(p + 228, basis_y);
    swc1(p + 232, basis_x);
    if mode != 4 {
        let body = lw(p + 384);
        let room = lw(p + 796);
        swc1(body + 48, lwc1(p + 140));
        sw(body + 12, room);
        let mode = lw(p + 296);
        swc1(body + 52, lwc1(p + 144));
        swc1(body + 56, lwc1(p + 148));
        if mode != 4 && mode != 7 {
            swc1(body + 80, lwc1(p + 276));
        }
    }
    if lw(p + 800) == 0 {
        return flags_fp;
    }

    // While the player aims, the line from the eye to where the sights point is tested for what it hits (the props of the player itself are left out of it).
    let body = lw(p + 384);
    let gun = lw(chr + 332);
    sw(body + 16, lw(body + 16) & -33);
    if gun != 0 {
        sw(gun + 16, lw(gun + 16) & -33);
    }
    let second = lw(chr + 440);
    if second != 0 {
        sw(second + 16, lw(second + 16) & -33);
    }
    let remote = lw(chr + 0x11b4);
    if remote != 0 {
        sw(remote + 16, lw(remote + 16) & -33);
    }
    let room = lw(p + 796);
    sw(sp, 0);
    sw(sp + 8, 0);
    let hit = bgLineTest(if room == 0 { 1 } else { room }, p + 152, chr + 2976, 1, p + 832, p + 844, 0, 0);
    let body = lw(p + 384);
    let gun = lw(chr + 332);
    sw(p + 828, hit);
    sw(body + 16, lw(body + 16) | 32);
    if gun != 0 {
        sw(gun + 16, lw(gun + 16) | 32);
    }
    let second = lw(chr + 440);
    if second != 0 {
        sw(second + 16, lw(second + 16) | 32);
    }
    let remote = lw(chr + 0x11b4);
    if remote != 0 {
        sw(remote + 16, lw(remote + 16) | 32);
    }
    flags_fp
}

/// Tick of the player (@ 0x281248): read the sticks and buttons, turn and move the player by the mode they are in, keep the camera vectors, the room, the sight line when zoomed, and the rumble.
pub fn playerTick() {
    let frame = Frame::new(560);
    let sp = frame.sp();
    let p = lw(g::currentplayer);
    let body = lw(p + 384);
    let chr = lw(body + 352);

    // The pose of the head, and the other characters that see the player.
    pose_matrix(sp, body, lwc1(p + 140), lwc1(p + 144), lwc1(p + 148), lwc1(0x3ad008));
    let drop = lwc1(0x3ad00c);
    swc1(chr + 3008, lwc1(sp + 128));
    swc1(chr + 3012, ps2float::sub(lwc1(sp + 132), drop));
    swc1(chr + 3016, lwc1(sp + 136));
    let mut seen_total = 0;
    let mut index = 0;
    if lw(g::lvnumplayers) + lw(g::numactivechars) > 0 {
        loop {
            let own = lw(chr);
            let seen = lw(index * 12 + lw(p + 820));
            seen_total += seen;
            if index != own && seen != 0 {
                StatsAddChar(4, own, index, ps2float::mul(frames(), lwc1(0x3ad010)));
            }
            index += 1;
            if index >= lw(g::lvnumplayers) + lw(g::numactivechars) {
                break;
            }
        }
    }
    let timer = lw(p + 792);
    if seen_total > 0 {
        let character = lw(lw(p + 384) + 352);
        sw(character + 2708, lw(character + 2708) | 16);
    } else if timer == 0 {
        let character = lw(lw(p + 384) + 352);
        sw(character + 2708, lw(character + 2708) & -17);
    }
    if lw(p + 792) != 0 {
        if lw(p + 792) == 60 {
            sw(lw(lw(p + 384) + 32) + 228, 0);
        }
        sw(p + 792, lw(p + 792) - 1);
    }

    let mut fp = 0;
    if debugmenuGetJoyMode() == 0 {
        fp = update_player(sp, p, chr);
    }

    // The direction the player looks along, as the angles the character keeps.
    let body = lw(p + 384);
    if lw(p + 296) == 4 {
        sw(body + 336, -1);
    } else {
        let x = lwc1(chr + 2976);
        swc1(sp + 288, x);
        let y = lwc1(chr + 2980);
        swc1(sp + 292, y);
        let z = lwc1(chr + 2984);
        swc1(sp + 296, z);
        let length = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
        let unit_x = ps2float::mul(lwc1(sp + 288), length);
        let unit_z = ps2float::mul(lwc1(sp + 296), length);
        let unit_y = ps2float::mul(lwc1(sp + 292), length);
        swc1(sp + 288, unit_x);
        swc1(sp + 296, unit_z);
        swc1(sp + 292, unit_y);
        let across = ps2float::add(ps2float::mul(unit_x, unit_x), ps2float::mul(unit_z, unit_z)).abs().psqrt();
        let pitch = atan2f(lwc1(sp + 292), across);
        let radians = lwc1(0x3ad0f8);
        swc1(lw(body + 352) + 2968, ps2float::mul(pitch, 1.8e+02_f32).pdiv(radians));
        let round = atan2f(lwc1(sp + 288), lwc1(sp + 296));
        let character = lw(body + 352);
        let degrees = ps2float::mul(round, 1.8e+02_f32).pdiv(radians);
        swc1(character + 2972, degrees);
        if degrees < 0.0_f32 {
            swc1(character + 2972, ps2float::add(degrees, 3.6e+02_f32));
        }
    }
    let character_flags = lw(lw(p + 384) + 352) + 28;
    if lw(g::invisible) == 0 {
        sw(character_flags, lw(character_flags) & -2);
    } else {
        sw(character_flags, lw(character_flags) | 1);
    }

    // The rumble of the pad: stronger the bigger the shake of the view.
    let ticks = lw(g::updateframesi);
    if lwc1(0x3ad0fc) < lwc1(p + 1776) && ticks != 0 && lhu(lw(p + 24) + 2) & 48 == 48 {
        let left = joyGetShockLeft(lw(p));
        let strength = ps2float::add(left as f32, ps2float::mul(lwc1(p + 1776), 128.0_f32)) as i32;
        let right = joyGetShockRight(lw(p));
        joySetShock(lw(p), strength, right);
    }
    let ticks = lw(g::updateframesi);
    if ticks > 0 {
        let decay = lwc1(p + 1780);
        let mut shake = lwc1(p + 1776);
        let mut left = ticks;
        loop {
            shake = ps2float::mul(shake, decay);
            left -= 1;
            if left == 0 {
                break;
            }
        }
        swc1(p + 1776, shake);
    }

    // The camera prop follows the player (or the prop it is watching).
    if lw(p + 388) != 0 {
        let camera = lw(p + 388);
        let watched = if camGetMode() != 0 { lw(p + 904) } else { 0 };
        if watched != 0 {
            swc1(camera + 48, lwc1(watched + 48));
            swc1(camera + 52, lwc1(watched + 52));
            swc1(camera + 56, lwc1(watched + 56));
        } else {
            let camera = lw(p + 388);
            swc1(camera + 48, lwc1(p + 152));
            swc1(camera + 52, lwc1(p + 156));
            swc1(camera + 56, lwc1(p + 160));
        }
    }
    let ground = lw(p + 824);
    if ground != 0 && lhu(ground + 6) & 4 != 0 && fp & 1 != 0 {
        propKill(lw(p + 384));
    }
    if lw(g::curgameparms + 80) & 32 != 0 && lw(g::curgameparms + 68) == 17 && fp & 1 != 0 && lwc1(p + 144) < 2.0_f32 {
        propKill(lw(p + 384));
    }
}
