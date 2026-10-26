//! Function: the tick of a remote prop (a turret or a mounted gun that sweeps for players, and that a player may take over).

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    random_as_float() * 2.3283064e-10_f32
}

/// Turn an angle in degrees into radians with the value of pi at `pi_at`.
fn radians(degrees: f32, pi: f32) -> f32 {
    (degrees * pi).pdiv(1.8e+02_f32)
}

/// Move the angle at `at` towards `target` by `step`, but not past it.
fn step_toward(at: i32, target: f32, step: f32) {
    let value = lwc1(at);
    let passed;
    if !(value < target) {
        let moved = value - step;
        passed = moved < target;
        swc1(at, moved);
    } else {
        let moved = value + step;
        passed = target < moved;
        swc1(at, moved);
    }
    if passed {
        swc1(at, target);
    }
}

/// Move the angle at `value_at` by its speed (at `speed_at`) for the frames that passed, turning it round when it comes to the limits at `min_at` and `max_at`.
fn bounce(mind: i32, value_at: i32, speed_at: i32, min_at: i32, max_at: i32) {
    let speed = lwc1(mind + speed_at);
    let min = lwc1(mind + min_at);
    let moved = lwc1(mind + value_at) + speed * lwc1(g::updateframesf);
    swc1(mind + value_at, moved);
    if moved < min {
        swc1(mind + value_at, min);
        swc1(mind + speed_at, -speed);
    }
    let max = lwc1(mind + max_at);
    if max < lwc1(mind + value_at) {
        let speed_now = lwc1(mind + speed_at);
        swc1(mind + value_at, max);
        swc1(mind + speed_at, -speed_now);
    }
}

/// Put the identity in every matrix of the bones of the prop.
fn reset_matrices(a0: i32) {
    let state = lw(a0 + 32);
    let mut n = 0;
    if lw(lw(state) + 4) > 0 {
        let mut base = lw(state + 4);
        loop {
            sceVu0UnitMatrix(base + (n << 6));
            n += 1;
            let state_now = lw(a0 + 32);
            if n >= lw(lw(state_now) + 4) {
                break;
            }
            base = lw(state_now + 4);
        }
    }
}

/// Turn the bones about the pivot: move the pivot to the origin, rotate by the three angles of the prop, and move it back (each step on every bone of the group).
fn pivot_rotate(bones: &[i32], scratch: i32, pivot: i32, pi: f32, mind: i32) {
    matrixTrans(scratch, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
    for bone in bones {
        sceVu0MulMatrix(*bone, *bone, scratch);
    }
    matrixRotZXY(scratch, radians(lwc1(mind + 12), pi), radians(lwc1(mind + 20), pi), radians(lwc1(mind + 28), pi));
    for bone in bones {
        sceVu0MulMatrix(*bone, *bone, scratch);
    }
    matrixTrans(scratch, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
    for bone in bones {
        sceVu0MulMatrix(*bone, *bone, scratch);
    }
}

/// Spin the bone about a point, a random amount (the flash of the gun): move to the point, turn, and move back.
fn flash_spin(bone: i32, scratch: i32, point: i32, spin: f32) {
    matrixTrans(scratch, lwc1(point), lwc1(point + 4), lwc1(point + 8));
    sceVu0MulMatrix(bone, bone, scratch);
    matrixRotZXY(scratch, 0.0_f32, 0.0_f32, spin);
    sceVu0MulMatrix(bone, bone, scratch);
    matrixTrans(scratch, -lwc1(point), -lwc1(point + 4), -lwc1(point + 8));
    sceVu0MulMatrix(bone, bone, scratch);
}

/// The pivot rotated by the heading of the prop, in the vector at `out`.
fn rotated_pivot(a0: i32, sp: i32, pivot: i32, out: i32, pi: f32) {
    swc1(out, lwc1(pivot));
    matrixRotY(sp + 16, radians(lwc1(a0 + 76), pi));
    swc1(out + 4, lwc1(pivot + 4));
    swc1(out + 8, lwc1(pivot + 8));
    matrixVecRotAligned(&[(sp + 16) as i64, out as i64]);
}

/// State 1: sweep to and fro between the limits, and look for a player in reach that can be seen.
fn state_sweep(a0: i32, mind: i32, sp: i32, pivot: i32) {
    bounce(mind, 12, 56, 32, 36);
    bounce(mind, 20, 60, 40, 44);
    bounce(mind, 28, 64, 48, 52);
    swc1(mind + 16, lwc1(mind + 12));
    swc1(mind + 24, lwc1(mind + 20));
    let pi = lwc1(0x3acc3c);
    rotated_pivot(a0, sp, pivot, sp + 96, pi);
    let f22_1 = lwc1(a0 + 52) + lwc1(sp + 96) - lwc1(mind + 72);
    let tangent = tanf(radians(lwc1(mind + 12), pi));
    let distance = (f22_1).pdiv(tangent);
    sw(sp + 80, 0);
    swc1(sp + 88, 1.0_f32);
    sw(sp + 84, 0);
    matrixRotY(sp + 16, ((lwc1(mind + 20) + lwc1(a0 + 76)) * pi).pdiv(1.8e+02_f32));
    matrixVecRotAligned(&[(sp + 16) as i64, (sp + 80) as i64]);
    let x = lwc1(sp + 80) * distance;
    let base_x = lwc1(a0 + 48);
    let y = lwc1(sp + 84) * distance;
    let rotated_x = lwc1(sp + 96);
    let z = lwc1(sp + 88) * distance;
    swc1(sp + 80, x);
    swc1(sp + 84, y);
    swc1(sp + 88, z);
    let players = lw(g::lvnumplayers);
    swc1(mind + 84, lwc1(mind + 72));
    swc1(mind + 80, base_x + rotated_x + x);
    swc1(mind + 88, lwc1(a0 + 56) + lwc1(sp + 104) + lwc1(sp + 88));
    if players > 0 {
        let mut n = 0;
        loop {
            let player = lw(n * 1820 + lw(g::players) + 384);
            let data = lw(player + 352);
            if (lw(data + 2708) & 32) == 0 {
                let dx = lwc1(mind + 80) - lwc1(player + 48);
                let dy = lwc1(mind + 84) - lwc1(player + 52);
                let reach = lwc1(mind + 92);
                let dz = lwc1(mind + 88) - lwc1(player + 56);
                if (dx * dx + dy * dy + dz * dz) < reach * reach {
                    let team = lw(mind + 8);
                    let same_team = team != 0 && team == lw(data + 24);
                    if !same_team {
                        remoteGetPivotePos(a0, sp + 128);
                        let f2_6 = lwc1(sp + 132);
                        let f4_3 = lwc1(sp + 136);
                        swc1(sp + 112, lwc1(player + 48) - lwc1(sp + 128));
                        swc1(sp + 116, lwc1(player + 52) + 1.0_f32 - f2_6);
                        let f1_8 = lwc1(player + 56);
                        sw(sp, 0);
                        sw(sp + 8, 0);
                        swc1(sp + 120, f1_8 - f4_3);
                        let blocked = bgLineTest(lw(mind + 120), sp + 128, sp + 112, 0, 0, 0, 0, -3);
                        if blocked == 0 {
                            sw(mind + 116, 0);
                            sw(mind + 4, 2);
                            sw(mind + 96, player);
                        }
                    }
                }
            }
            n += 1;
            if n >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
}

/// State 2: turn towards the player that was seen; when it is aimed at (and for the time kept at +116) fire, and give up (state 3) after a while or when the player is dead
/// or out of sight. Gives whether to fire.
fn state_track(a0: i32, mind: i32, sp: i32, pivot: i32, model: i32) -> bool {
    let pi = lwc1(0x3acc40);
    rotated_pivot(a0, sp, pivot, sp + 96, pi);
    let target = lw(mind + 96);
    let base_z = lwc1(a0 + 56);
    let dx = lwc1(target + 48) - (lwc1(a0 + 48) + lwc1(sp + 96));
    let rotated_z = lwc1(sp + 104);
    let rotated_y = lwc1(a0 + 52) + lwc1(sp + 100);
    swc1(sp + 80, dx);
    let dy = lwc1(target + 52) + 1.0_f32 - rotated_y;
    swc1(sp + 84, dy);
    let dz = lwc1(target + 56) - (base_z + rotated_z);
    swc1(sp + 88, dz);
    let inverse = (1.0_f32).pdiv((dx * dx + dy * dy + dz * dz).abs().psqrt());
    let heading = (lwc1(a0 + 76) * pi).pdiv(1.8e+02_f32);
    let nx = lwc1(sp + 80) * inverse;
    let ny = lwc1(sp + 84) * inverse;
    swc1(sp + 88, lwc1(sp + 88) * inverse);
    swc1(sp + 80, nx);
    swc1(sp + 84, ny);
    matrixRotY(sp + 16, -heading);
    matrixVecRotAligned(&[(sp + 16) as i64, (sp + 80) as i64]);
    let side = lwc1(sp + 80);
    let mut pitch;
    let mut exact;
    if lb(model + 7) != 1 {
        let forward = lwc1(sp + 88);
        let up = lwc1(sp + 84);
        let flat = (side * side + forward * forward).abs().psqrt();
        pitch = -((atan2f(up, flat) * 1.8e+02_f32).pdiv(pi));
        exact = true;
        if pitch < 0.0_f32 {
            pitch = pitch + 3.6e+02_f32;
        }
    } else {
        pitch = lwc1(mind + 12);
        exact = false;
    }
    let mut yaw = (atan2f(lwc1(sp + 80), lwc1(sp + 88)) * 1.8e+02_f32).pdiv(lwc1(0x3acc44));
    if yaw < -9e+01_f32 {
        yaw = yaw + 3.6e+02_f32;
    }
    let max_pitch = lwc1(mind + 36);
    if max_pitch < pitch {
        pitch = max_pitch;
        exact = false;
    }
    let min_pitch = lwc1(mind + 32);
    if pitch < min_pitch {
        pitch = min_pitch;
        exact = false;
    }
    let max_yaw = lwc1(mind + 44);
    if max_yaw < yaw {
        yaw = max_yaw;
        exact = false;
    }
    let current_yaw = lwc1(mind + 20);
    let min_yaw = lwc1(mind + 40);
    if yaw < min_yaw {
        yaw = min_yaw;
        exact = false;
    }
    let yaw_error = anglediff(yaw, current_yaw);
    let mut size = fptodp(yaw_error);
    if dpcmp(size, 0_i64) < 0 {
        size = dpsub(0_i64, size);
    }
    if dpcmp(size, 0x4014000000000000_i64) < 0 {
        step_toward(mind + 12, pitch, 4.0_f32);
    }
    step_toward(mind + 20, yaw, 4.0_f32);
    let timer = lw(mind + 116);
    let frames = lw(g::updateframesi);
    let mut fire = false;
    if exact && timer.wrapping_rem(100) < 31 {
        if yaw == lwc1(mind + 20) {
            if pitch == lwc1(mind + 12) {
                fire = true;
            }
        }
    }
    let timer_now = timer + frames;
    sw(mind + 116, timer_now);
    if timer_now < 300 {
        return fire;
    }
    let target_data = lw(lw(mind + 96) + 352);
    remoteGetPivotePos(a0, sp + 144);
    let target_now = lw(mind + 96);
    let f2_9 = lwc1(sp + 148);
    let dead = lw(target_data + 2708) & 32;
    let f4_5 = lwc1(sp + 152);
    swc1(sp + 112, lwc1(target_now + 48) - lwc1(sp + 144));
    swc1(sp + 116, lwc1(target_now + 52) + 1.0_f32 - f2_9);
    swc1(sp + 120, lwc1(target_now + 56) - f4_5);
    if dead == 0 {
        sw(sp, 0);
        sw(sp + 8, 0);
        let blocked = bgLineTest(lw(mind + 120), sp + 144, sp + 112, 0, 0, 0, 0, -3);
        if blocked == 0 {
            sw(mind + 116, 0);
            return fire;
        }
    }
    sw(mind + 4, 3);
    fire
}

/// State 3: bring the aim back to where it was when the sweep stopped (yaw first, then pitch) and go back to sweeping (state 1).
fn state_return(mind: i32) {
    let yaw_target = lwc1(mind + 24);
    let yaw = lwc1(mind + 20);
    let pitch_target = lwc1(mind + 16);
    if yaw_target == yaw {
        step_toward(mind + 12, pitch_target, 2.0_f32);
    }
    step_toward(mind + 20, yaw_target, 2.0_f32);
    if lwc1(mind + 12) == pitch_target && lwc1(mind + 20) == yaw_target {
        sw(mind + 4, 1);
    }
}

/// State 4: wait for a local player near the prop to press the use button; then take them in (state 5) and aim the prop where they look.
fn state_wait(a0: i32, mind: i32, sp: i32) {
    let mut n = 0;
    loop {
        if n >= ilinkGetNumLocalPlayers() {
            break;
        }
        let info = lw(g::chrinfo) + n * 0x1210;
        let player = lw(info + 3020);
        let data = lw(player + 356);
        let buttons = if lw(lw(g::front) + 20) != 0 { 0 } else { joyNewBut(lw(data)) };
        if (buttons & lw(data + 36)) == 0 {
            n += 1;
        } else if lw(data + 296) != 0 {
            n += 1;
        } else if lw(g::updateframesi) <= 0 {
            n += 1;
        } else {
            let dx = lwc1(player + 48) - lwc1(a0 + 48);
            let dy = lwc1(player + 52) - lwc1(a0 + 52);
            let dz = lwc1(player + 56) - lwc1(a0 + 56);
            if (dx * dx + dy * dy + dz * dz) < lwc1(0x3acc48) {
                remoteGetPivotePos(a0, sp + 112);
                soundStart(218, data + 140);
                let z = lwc1(sp + 120);
                let own_z = lwc1(data + 148);
                let x = lwc1(sp + 112) - lwc1(data + 140);
                sw(mind + 4, 5);
                let relative_z = z - own_z;
                sw(mind + 96, player);
                sw(data + 296, 6);
                let facing = atan2f(x, relative_z);
                sw(info + 0x11b4, a0);
                let yaw = (facing * 1.8e+02_f32).pdiv(lwc1(0x3acc4c));
                swc1(mind + 108, yaw);
                if yaw < 0.0_f32 {
                    swc1(mind + 108, yaw + 3.6e+02_f32);
                }
                let tilt = -lwc1(data + 284);
                swc1(mind + 112, tilt);
                if tilt < -1e+01_f32 {
                    swc1(mind + 112, -1e+01_f32);
                } else if 3e+01_f32 < tilt {
                    swc1(mind + 112, 3e+01_f32);
                }
                let weapon = lw(info + 260);
                if weapon == lw(info + 264) {
                    sw(info + 264, weapon);
                }
                sw(info + 260, 0);
                gunSetPlayerZoom(data, lw(lw(mind + 100)));
            }
            n += 1;
        }
    }
}

/// Give the player that was on the prop their weapon back and put the prop in state 4.
fn release_player(mind: i32, chr: i32, player: i32) {
    sw(chr + 0x11b4, 0);
    sw(mind + 4, 4);
    sw(player + 296, 0);
    let weapon = lw(chr + 264);
    if weapon != 0 {
        sw(chr + 260, weapon);
    }
    sw(chr + 264, 0);
}

/// State 5: the player gets into the prop: turn the prop towards where they stand and tilt it level with them; when both are done and they stand in place, go on to state 6.
fn state_enter(a0: i32, mind: i32, sp: i32) {
    let rider = lw(mind + 96);
    let player = lw(rider + 356);
    let chr = lw(rider + 352);
    let buttons = if lw(lw(g::front) + 20) != 0 { 0 } else { joyNewBut(lw(player)) };
    if lw(chr + 0x11b4) != 0 && (lw(chr + 2708) & 32) == 0 {
        if (buttons & lw(player + 36)) == 0 {
            let mut yaw_done = false;
            let mut pitch_done = false;
            let mut in_place = false;
            let target_yaw = lwc1(mind + 20) + lwc1(a0 + 76);
            let step = lwc1(g::updateframesf) * 4.0_f32;
            let yaw_error = anglediff(lwc1(mind + 108), target_yaw);
            remoteGetStandPos(a0, sp + 112, lwc1(mind + 108));
            let stand_y = lwc1(sp + 116);
            if lwc1(rider + 48) == lwc1(sp + 112) {
                let stand_z = lwc1(sp + 120);
                if lwc1(rider + 52) == stand_y {
                    if lwc1(rider + 56) == stand_z {
                        if lwc1(player + 276) == lwc1(mind + 108) {
                            if lwc1(player + 284) == -lwc1(mind + 112) {
                                in_place = true;
                            }
                        }
                    }
                }
            }
            let new_yaw;
            let near = if !(0.0_f32 <= yaw_error) { -yaw_error < step } else { yaw_error < step };
            if near {
                yaw_done = true;
                new_yaw = lwc1(mind + 108) - lwc1(a0 + 76);
            } else {
                let yaw = lwc1(mind + 20);
                new_yaw = if !(yaw_error < 0.0_f32) { yaw - step } else { yaw + step };
            }
            swc1(mind + 20, new_yaw);
            let pitch_error = anglediff(lwc1(mind + 112), lwc1(mind + 12));
            let near = if !(0.0_f32 <= pitch_error) { -pitch_error < step } else { pitch_error < step };
            let new_pitch;
            if near {
                pitch_done = true;
                new_pitch = lwc1(mind + 112);
            } else {
                let pitch = lwc1(mind + 12);
                new_pitch = if !(pitch_error < 0.0_f32) { pitch - step } else { pitch + step };
            }
            swc1(mind + 12, new_pitch);
            if !yaw_done || !pitch_done || !in_place {
                return;
            }
            sw(mind + 4, 6);
            sw(player + 296, 7);
            return;
        }
        release_player(mind, chr, player);
        return;
    }
    sw(mind + 4, 4);
}

/// State 6: the player rides the prop: it follows where they look, until they let go (state 4); every frame counts in their stats.
fn state_ridden(a0: i32, mind: i32) {
    let rider = lw(mind + 96);
    let player = lw(rider + 356);
    let chr = lw(rider + 352);
    let buttons = if lw(lw(g::front) + 20) != 0 { 0 } else { joyNewBut(lw(player)) };
    if lw(chr + 0x11b4) != 0 && (lw(chr + 2708) & 32) == 0 {
        if (buttons & lw(player + 36)) == 0 {
            swc1(mind + 20, lwc1(chr + 2972) - lwc1(a0 + 76));
            swc1(mind + 12, -lwc1(chr + 2968));
        } else {
            release_player(mind, chr, player);
            soundStart(219, player + 140);
        }
    } else {
        sw(mind + 4, 4);
    }
    let frames = lwc1(g::updateframesf);
    StatsAdd(63, lw(player), frames * lwc1(0x3acc50));
}

/// Tick of a remote prop (@ 0x26c440): the state at +4 says what it does (1 sweeps for players, 2 aims at one and fires, 3 goes back to the sweep, 4 waits to be used, 5
/// takes a player in, 6 is ridden by the player); afterwards the matrices of its bones are made from the angles of the state, with a flash of the gun when it fired.
pub fn remotePropTick(a0: i32) {
    let frame = Frame::new(368);
    let sp = frame.sp();
    let mut fire = false;
    let a0_1 = lw(a0 + 32);
    let mind = lw(a0 + 352);
    let state = lw(mind + 4);
    sh(a0_1 + 292, lhu(a0_1 + 292) & 0xfffc);
    if state == 0 || (lw(a0 + 16) & 1) == 0 {
        reset_matrices(a0);
        return;
    }
    let chain = lw(mind + 100);
    let header = lw(a0_1);
    sw(chain + 68, lw(chain + 68) + lw(g::updateframesi));
    let model = header - lw(header) * 80;
    let bone_group = if lw(a0 + 4) != 425 {
        if (lb(model + 7) ^ 1) == 0 { 2 } else { 1 }
    } else {
        3
    };
    let pivot = remoteGetPivoteOffset(a0);
    match lw(mind + 4) {
        1 => state_sweep(a0, mind, sp, pivot),
        2 => fire = state_track(a0, mind, sp, pivot, model),
        3 => state_return(mind),
        4 => state_wait(a0, mind, sp),
        5 => state_enter(a0, mind, sp),
        6 => state_ridden(a0, mind),
        _ => {}
    }
    reset_matrices(a0);
    let bones = lw(lw(a0 + 32) + 4);
    let scratch = sp + 16;
    pivot_rotate(&[bones + (bone_group << 6)], scratch, pivot, lwc1(0x3acc54), mind);
    if lw(a0 + 4) != 425 {
        if bone_group == 1 {
            let bone = lw(lw(a0 + 32) + 4) + 128;
            pivot_rotate(&[bone], scratch, pivot, lwc1(0x3acc54), mind);
            let vector = lw(model + 72);
            let spin = (random_unit() * 3.6e+02_f32 * lwc1(0x3acc70)).pdiv(1.8e+02_f32);
            flash_spin(bone, scratch, vector, spin);
        }
    } else {
        let frames = lw(g::updateframesi);
        if frames > 0 {
            let mut a = lwc1(mind + 124);
            let mut b = lwc1(mind + 128);
            let mut c = lwc1(mind + 132);
            let mut d = lwc1(mind + 136);
            let mut n = frames;
            loop {
                a = a * 0.75_f32;
                n -= 1;
                b = b * 0.75_f32;
                c = c * lwc1(0x3acc58);
                d = d * lwc1(0x3acc5c);
                if n == 0 {
                    break;
                }
            }
            swc1(mind + 136, d);
            swc1(mind + 132, c);
            swc1(mind + 128, b);
            swc1(mind + 124, a);
        }
        if lwc1(0x3acc60) < lwc1(mind + 136) && frames != 0 {
            let rider = lw(mind + 96);
            if rider != 0 {
                let player = lw(rider + 356);
                if player != 0 {
                    let id = lw(player);
                    if (lhu(lw(id * 1820 + lw(g::players) + 24) + 2) & 80) == 80 {
                        let left = joyGetShockLeft(id);
                        let right = joyGetShockRight(id);
                        joySetShock(id, left, right + (lwc1(mind + 136) * 128.0_f32) as i32);
                    }
                }
            }
        }
        let barrels = [256, 320, 384, 448, 512].map(|offset| lw(lw(a0 + 32) + 4) + offset);
        pivot_rotate(&barrels, scratch, pivot, lwc1(0x3acc64), mind);
        let recoil = lwc1(0x3acc68);
        let kicks = [(0, lwc1(mind + 132)), (1, lwc1(mind + 128)), (2, lwc1(mind + 124)), (3, lwc1(mind + 124) * recoil), (4, lwc1(mind + 128) * recoil)];
        for (n, amount) in kicks {
            matrixTrans(scratch, 0.0_f32, 0.0_f32, -amount);
            sceVu0MulMatrix(barrels[n], barrels[n], scratch);
        }
        let first = lw(lw(a0 + 32) + 4) + 64;
        pivot_rotate(&[first], scratch, pivot, lwc1(0x3acc64), mind);
        flash_spin(first, scratch, lw(model + 72) + 12, random_unit() * 3.6e+02_f32);
        let second = lw(lw(a0 + 32) + 4) + 128;
        pivot_rotate(&[second], scratch, pivot, lwc1(0x3acc6c), mind);
        flash_spin(second, scratch, lw(model + 72) + 24, random_unit() * 3.6e+02_f32);
    }
    if !fire {
        return;
    }
    gunRemoteFire(a0, lw(mind + 100), 0, 0, 3, 0);
}
