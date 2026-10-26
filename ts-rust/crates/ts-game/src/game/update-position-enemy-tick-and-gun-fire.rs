//! Functions: moving a character and updating its speed, the tick of a computer character (which state function runs by its state) and firing a gun (making its bullet).

use super::particle_spawners::random_as_float;
use crate::*;

/// Move the character `a0` (@ 0x2bbcf8) by the velocity of its animation (3 floats at +128 of its model data), on its way to the pad it goes to or on the floor, and bring its
/// speed (+2876) towards the speed it wants. A character that is dropping (the flag 0x1000 at +2708) falls and may die when it lands.
pub fn updatePos(a0: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let data = lw(a0 + 32);
    let mind = lw(a0 + 352);
    let velocity_x = lwc1(data + 128);
    let flags = lw(mind + 2708);
    let number = ((mind.wrapping_sub(lw(g::chrinfo))).wrapping_mul(-0x795b3d1f)) >> 4;
    swc1(sp + 64, velocity_x);
    let velocity_y = lwc1(data + 132);
    swc1(sp + 68, velocity_y);
    let velocity_z = lwc1(data + 136);
    swc1(sp + 72, velocity_z);
    'b2: {
        if (flags & 0x1000) == 0 {
            if velocity_x == 0.0_f32 && velocity_y == 0.0_f32 && velocity_z == 0.0_f32 {
                break 'b2;
            }
            let room_flag;
            'b5: {
                let chr_flags = lw(a0 + 16);
                let mind_flags;
                if (chr_flags & 0xf000000) != 0 {
                    mind_flags = lw(mind + 2704);
                } else {
                    let state = lw(mind + 2716);
                    mind_flags = lw(mind + 2704);
                    if (state & 0x68c000) == 0 {
                        sw(mind + 2704, mind_flags | 512);
                        if (state & 768) == 0 {
                            if (state & 0x1084c) == 0 {
                                break 'b2;
                            }
                            if lw(mind + 680) < 0 {
                                break 'b2;
                            }
                            let y = lwc1(a0 + 52);
                            let dx = lwc1(mind + 2796) - lwc1(a0 + 48);
                            let z = lwc1(a0 + 56);
                            let lift = lwc1(0x3ad810);
                            swc1(sp + 96, dx);
                            let dy = lwc1(mind + 2800) + lift - y;
                            swc1(sp + 100, dy);
                            let dz = lwc1(mind + 2804) - z;
                            swc1(sp + 104, dz);
                            let vx = lwc1(sp + 64);
                            let vy = lwc1(sp + 68);
                            let vz = lwc1(sp + 72);
                            let mut scale = ps2float::mul(ps2float::div(1.0_f32, ps2float::sqrt(dx * dx + dy * dy + dz * dz)), ps2float::sqrt(vx * vx + vy * vy + vz * vz));
                            let step_x = lwc1(sp + 96);
                            if scale < lwc1(0x3ad814) {
                                scale = 0.0_f32;
                            }
                            let move_x = step_x * scale;
                            let move_z = lwc1(sp + 104) * scale;
                            let new_y = lwc1(a0 + 52) + lwc1(sp + 100) * scale;
                            let new_z = lwc1(a0 + 56) + move_z;
                            swc1(a0 + 48, lwc1(a0 + 48) + move_x);
                            swc1(a0 + 52, new_y);
                            swc1(a0 + 56, new_z);
                            swc1(mind + 0x1198, new_y);
                            room_flag = 0;
                            break 'b5;
                        }
                        swc1(mind + 0x1198, lwc1(a0 + 52));
                        room_flag = 1;
                        break 'b5;
                    }
                }
                if (mind_flags & 512) != 0 {
                    enemySetRoom(a0, 1);
                }
                let after;
                'b9: {
                    let turn = (lwc1(a0 + 76) * lwc1(0x3ad7fc)).pdiv(1.8e+02_f32);
                    matrixRotY(sp, turn);
                    matrixVecRotAligned(&[sp as i64, (sp + 64) as i64]);
                    let data_now = lw(a0 + 32);
                    let z = lwc1(a0 + 56) + lwc1(sp + 72);
                    let animation = lw(data_now + 96) - 566;
                    swc1(sp + 80, lwc1(a0 + 48) + lwc1(sp + 64));
                    swc1(sp + 88, z);
                    if (animation as u32) >= (41 as u32) {
                        swc1(sp + 84, lwc1(mind + 0x1198));
                    } else {
                        let mut y = lwc1(a0 + 52);
                        let floor = lwc1(mind + 0x1198);
                        let mut rise = lwc1(sp + 68);
                        if (y - 1.0_f32) < floor {
                            let low = floor - lwc1(0x3ad800);
                            swc1(a0 + 52, low);
                            y = low;
                            rise = lwc1(sp + 68);
                        }
                        let target = y + rise;
                        swc1(sp + 84, target);
                        let floor = lwc1(mind + 0x1198);
                        if !(target < floor) {
                            after = lw(data_now + 96);
                            break 'b9;
                        }
                        swc1(sp + 84, floor);
                    }
                    after = lw(data_now + 96);
                }
                if ((after - 524) as u32) < (2 as u32) {
                    let floor = lwc1(mind + 0x1198);
                    if lwc1(a0 + 52) < floor {
                        swc1(a0 + 52, floor);
                    }
                }
                let mut mind_flags;
                'b13: {
                    let x_now;
                    'b14: {
                        chrAddFloorMoveToPos(number, sp + 80, a0 + 76);
                        let at = a0 + 48;
                        let player = lw(g::currentplayer);
                        swc1(mind + 2956, lwc1(mind + 0x1198));
                        let result = moveTestDoSlide(number, at, sp + 80, 0, 1, lwc1(0x3ad804), lwc1(player + 132));
                        mind_flags = lw(mind + 2704);
                        if (result & 1) == 0 {
                            if lwc1(0x3ad808) < lwc1(mind + 2956) - lwc1(mind + 0x1198) {
                                x_now = lwc1(sp + 80);
                                if (mind_flags & 0x2000) != 0 {
                                    break 'b14;
                                }
                                'b16: {
                                    let y = lwc1(a0 + 52) - lwc1(0x3ad80c);
                                    let z = lwc1(sp + 88);
                                    let flags = lw(mind + 2708) | 0x1000;
                                    swc1(a0 + 48, lwc1(sp + 80));
                                    swc1(a0 + 52, y);
                                    swc1(a0 + 56, z);
                                    sw(mind + 2708, flags);
                                    setAnim(a0, 469, 0.25_f32, 1.0_f32);
                                    if !(7.0_f32 < lwc1(mind + 2956) - lwc1(mind + 0x1198)) {
                                        sw(mind + 2864, 0);
                                    } else {
                                        let hard = lw(mind + 2708) & 32;
                                        swc1(lw(a0 + 32) + 164, 1.0_f32);
                                        if hard != 0 {
                                            let level = lw(g::lvlevelnum);
                                            if level != 24 {
                                                if level != 5 {
                                                    sw(mind + 2864, 0);
                                                    break 'b16;
                                                }
                                            }
                                            if lw(g::updateframesi) > 0 {
                                                soundStart(205, at);
                                            }
                                        }
                                        sw(mind + 2864, 0);
                                    }
                                }
                                mind_flags = lw(mind + 2704);
                                break 'b13;
                            }
                        }
                        x_now = lwc1(sp + 80);
                    }
                    let y_now = lwc1(sp + 84);
                    let z_now = lwc1(sp + 88);
                    swc1(a0 + 48, x_now);
                    swc1(a0 + 52, y_now);
                    swc1(a0 + 56, z_now);
                }
                sw(mind + 2704, mind_flags & -513);
                room_flag = 0;
            }
            enemySetRoom(a0, room_flag);
        } else {
            let lowest = lwc1(0x3ad7f0);
            let speed = lwc1(mind + 2864) - lwc1(g::updateframesf) * lwc1(0x3ad7ec);
            swc1(mind + 2864, speed);
            if speed < lowest {
                swc1(mind + 2864, lowest);
            }
            sw(sp + 64, 0);
            sw(sp + 68, 0);
            let mut size = fptodp(lwc1(mind + 2876));
            if dpcmp(size, 0_i64) < 0 {
                size = dpsub(0_i64, size);
            }
            let room_flag;
            let at = a0 + 48;
            'b21: {
                let flags_now;
                'b22: {
                    let sum = dpadd(size, ld(0x3a9cb0));
                    let product = dpmul(sum, ld(0x3a9cb8));
                    let frames = fptodp(lwc1(g::updateframesf));
                    let product = dpmul(product, frames);
                    let down = dpdiv(product, 0x404e000000000000_i64);
                    let down = dptofp(down);
                    let turn = (lwc1(a0 + 76) * lwc1(0x3ad7f4)).pdiv(1.8e+02_f32);
                    swc1(sp + 72, down);
                    matrixRotY(sp, turn);
                    matrixVecRotAligned(&[sp as i64, (sp + 64) as i64]);
                    let y = lwc1(a0 + 52);
                    let frames = lwc1(g::updateframesf);
                    let z = lwc1(a0 + 56);
                    let fall = lwc1(sp + 72);
                    swc1(sp + 80, lwc1(a0 + 48) + lwc1(sp + 64));
                    let player = lw(g::currentplayer);
                    let radius = lwc1(0x3ad7f8);
                    let drop = lwc1(mind + 2864) * frames;
                    swc1(sp + 88, z + fall);
                    swc1(sp + 84, y + drop);
                    let result = moveTestDoSlide(number, at, sp + 80, 0, 1, radius, lwc1(player + 132));
                    if (result & 1) == 0 {
                        if curAnim(a0, 469) != 0 {
                            if curAnimComplete(lw(a0 + 32)) != 0 {
                                flags_now = lw(mind + 2708);
                                break 'b22;
                            }
                        }
                        room_flag = 0;
                        break 'b21;
                    }
                    flags_now = lw(mind + 2708);
                }
                sw(mind + 2708, flags_now & -0x1001);
                if (flags_now & 32) != 0 {
                    if lw(g::updateframesi) > 0 {
                        soundStart(206, at);
                    }
                    chrSelectDeathAnim(a0, -1, 0, 0, 0, 0);
                    let state = lw(mind + 2716);
                    let flags = lw(mind + 2704) | 0x2000;
                    sw(mind + 2716, 16);
                    sw(mind + 2704, flags);
                    sw(mind + 2720, state);
                    sw(mind + 2892, 0);
                    return;
                }
                enemyCheckValidPad(lw(mind));
                room_flag = 1;
            }
            enemySetRoom(a0, room_flag);
            sw(mind + 2704, lw(mind + 2704) & -513);
            let y = lwc1(sp + 84);
            let z = lwc1(sp + 88);
            swc1(a0 + 48, lwc1(sp + 80));
            swc1(a0 + 52, y);
            swc1(a0 + 56, z);
        }
    }
    let state = lw(mind + 2716);
    if state != 0x400000 {
        if state != 0x200000 {
            rotateChr(a0);
        }
    }
    let flags = lw(mind + 2708);
    if (flags & 1027) == 0 {
        if (flags & 128) == 0 {
            if (flags & 8) != 0 {
                let speed = lwc1(mind + 2876);
                let reached;
                if !(0.0_f32 < speed) {
                    if !(speed < 0.0_f32) {
                        return;
                    }
                    let faster = speed + lwc1(mind + 2868);
                    reached = 0.0_f32 < faster;
                    swc1(mind + 2876, faster);
                } else {
                    let slower = speed - lwc1(mind + 2868);
                    reached = slower < 0.0_f32;
                    swc1(mind + 2876, slower);
                }
                if reached {
                    swc1(mind + 2876, 0.0_f32);
                }
            }
        } else {
            let top = if (flags & 16) == 0 { lwc1(mind + 2856) } else { lwc1(mind + 2860) };
            swc1(mind + 2872, top);
            let top = lwc1(mind + 2872);
            let slower = lwc1(mind + 2876) - lwc1(mind + 2868);
            swc1(mind + 2876, slower);
            if top < slower {
                swc1(mind + 2876, top);
            }
            let bottom = -top;
            if lwc1(mind + 2876) < bottom {
                swc1(mind + 2876, bottom);
            }
        }
    } else {
        let top;
        let mut speed;
        if (flags & 2) == 0 {
            let moved = lwc1(mind + 2876) + lwc1(mind + 2868);
            speed = moved;
            swc1(mind + 2876, moved);
            top = lwc1(mind + 2872);
        } else if (flags & 64) != 0 {
            speed = lwc1(mind + 2876);
            if !(0.0_f32 < speed) {
                if !(speed < 0.0_f32) {
                    top = lwc1(mind + 2872);
                } else {
                    let faster = speed + lwc1(mind + 2868);
                    let reached = 0.0_f32 < faster;
                    swc1(mind + 2876, faster);
                    speed = 0.0_f32;
                    if !reached {
                        speed = faster;
                    } else {
                        swc1(mind + 2876, 0.0_f32);
                    }
                    top = lwc1(mind + 2872);
                }
            } else {
                let slower = speed - lwc1(mind + 2868);
                let reached = slower < 0.0_f32;
                swc1(mind + 2876, slower);
                speed = 0.0_f32;
                if !reached {
                    speed = slower;
                } else {
                    swc1(mind + 2876, 0.0_f32);
                }
                top = lwc1(mind + 2872);
            }
        } else {
            let side = checkSide(a0, mind + 2820, 0.0_f32);
            let current = lwc1(mind + 2876);
            let moved = if side != 0 { current - lwc1(mind + 2868) } else { current + lwc1(mind + 2868) };
            speed = moved;
            swc1(mind + 2876, moved);
            top = lwc1(mind + 2872);
        }
        let bottom;
        if !(top < speed) {
            bottom = -top;
        } else {
            swc1(mind + 2876, top);
            speed = top;
            bottom = -top;
        }
        if speed < bottom {
            swc1(mind + 2876, bottom);
        }
    }
}

/// The tick of a computer character `a0` (@ 0x2bb470): its clocks go on, its name of state (+2736, 30 letters) is set and the function of its state (+2716) runs: asleep, awake,
/// patrolling, attacking, defending, dying, spawning, following, the ambush states and so on.
pub fn enemyTick(a0: i32) {
    let mind = lw(a0 + 352);
    if lw(g::numactivechars) == 0 {
        return;
    }
    let state = lw(mind + 2716);
    if state == 0 {
        let prop = lw(mind + 3020);
        sw(prop + 16, lw(prop + 16) & -0xf000001 & -33);
        strncpy(mind + 2736, 0x3afd00, 30);
        return;
    }
    let step = (lwc1(g::updateframesf)).pdiv(6e+01_f32);
    let f4 = lwc1(mind + 2952);
    let f5 = lwc1(mind + 2948);
    let f2 = lwc1(mind + 2896) + step;
    let f0 = lwc1(mind + 2908) + step;
    let f3 = lwc1(mind + 2912) + step;
    swc1(mind + 2892, lwc1(mind + 2892) + step);
    swc1(mind + 2896, f2);
    swc1(mind + 2908, f0);
    swc1(mind + 2912, f3);
    swc1(mind + 2952, f4 + step);
    swc1(mind + 2948, f5 + step);
    if state != 0x2000 {
        if state != 0x20000 {
            swc1(mind + 2904, lwc1(mind + 2904) + step);
        }
    }
    let frames = lwc1(g::updateframesf);
    if state != 16 {
        StatsAdd(65, lw(mind), frames * lwc1(0x3ad7e8));
    }
    let hold = lw(mind + 0x11a4);
    if hold != 0 {
        if lw(hold + 8) != 0 {
            updatePos(a0);
            return;
        }
    }
    if lw(mind + 2716) != 16 {
        checkProgress(a0);
    }
    let mut state;
    'b6: {
        enemyCheckSpawnfx(mind);
        state = lw(mind + 2716);
        let before = state;
        let flags;
        if before != 0x20000 {
            'b8: {
                let flags_a = lw(mind + 2704);
                let flags_b = lw(mind + 2708);
                if (flags_a & 2) != 0 {
                    if (flags_b & 16) == 0 {
                        let set = flags_b | 16;
                        if (1.0_f32 - lwc1(mind + 2832)) < lwc1(mind + 2952) {
                            sw(mind + 2952, 0);
                            sw(mind + 2708, set);
                        }
                    }
                    break 'b8;
                }
                if (flags_b & 16) != 0 {
                    if (lwc1(mind + 2832) * 5.0_f32) < lwc1(mind + 2952) {
                        sw(mind + 2952, 0);
                        sw(mind + 2708, flags_b & -17);
                    }
                }
            }
            let current;
            if (before & 0x180000) != 0 {
                current = lw(mind + 2708);
            } else {
                let playing = curAnim(a0, 449);
                state = lw(mind + 2716);
                if playing == 0 {
                    if state != 4 {
                        break 'b6;
                    }
                    let flags_c = lw(mind + 2708);
                    if (flags_c & 8) == 0 {
                        break 'b6;
                    }
                    sw(mind + 2708, flags_c | 16);
                    break 'b6;
                }
                current = lw(mind + 2708);
            }
            flags = current | 16;
        } else {
            flags = lw(mind + 2708) & -17;
        }
        sw(mind + 2708, flags);
    }
    let name = mind + 2736;
    // the function of the state; `settle` is true when the animation of the character is set after it
    let mut settle = false;
    match state {
        0x2000 => {
            strncpy(name, 0x3afd40, 30);
            enemySleep(a0);
        }
        0x100000 => {
            strncpy(name, 0x3afd50, 30);
            updatePos(a0);
            enemySnipe1(a0);
            settle = true;
        }
        0x1000000 => {
            strncpy(name, 0x3afd58, 30);
            updatePos(a0);
            enemyWindow(a0);
            settle = true;
        }
        0x2000000 => {
            strncpy(name, 0x3a9c80, 30);
            if lw(mind + 2728) > 0 {
                updatePos(a0);
            }
            enemyPillarAlien(a0);
        }
        0x4000000 => {
            strncpy(name, 0x3afd60, 30);
            updatePos(a0);
            enemyNoRoute(a0);
            settle = true;
        }
        0x200000 => {
            strncpy(name, 0x3a9ca0, 30);
            updatePos(a0);
            enemyAmbushWallRight(a0);
        }
        0x400000 => {
            strncpy(name, 0x3a9c90, 30);
            updatePos(a0);
            enemyAmbushWallLeft(a0);
        }
        0x10000 => {
            strncpy(name, 0x3afd38, 30);
            updatePos(a0);
            enemyPatrol(a0);
            settle = true;
        }
        0x20000 => {
            strncpy(name, 0x3afd48, 30);
            updatePos(a0);
            enemyAwake(a0);
        }
        0x80000 => {
            strncpy(name, 0x3a9c70, 30);
            updatePos(a0);
            enemyAmbushCrouch(a0);
            settle = true;
        }
        0x4000 => {
            strncpy(name, 0x3a9c60, 30);
            updatePos(a0);
            enemyAmbushRight(a0);
        }
        0x8000 => {
            strncpy(name, 0x3a9c50, 30);
            updatePos(a0);
            enemyAmbushLeft(a0);
        }
        64 => {
            strncpy(name, 0x3afd30, 30);
            updatePos(a0);
            findweapon(a0);
            settle = true;
        }
        512 => {
            strncpy(name, 0x3a9c30, 30);
            updatePos(a0);
            findmypad(a0);
            settle = true;
        }
        1024 => {
            strncpy(name, 0x3a9c40, 30);
            robotAI(a0);
        }
        2048 => {
            strncpy(name, 0x3afd18, 30);
            updatePos(a0);
            followTarget(a0);
            settle = true;
        }
        128 => {
            strncpy(name, 0x3a9c10, 30);
            updatePos(a0);
            selectTarget(a0);
            settle = true;
        }
        256 => {
            strncpy(name, 0x3a9c20, 30);
            updatePos(a0);
            findbearings(a0);
            settle = true;
        }
        8 => {
            strncpy(name, 0x3a9c00, 30);
            updatePos(a0);
            trackTarget(a0);
            settle = true;
        }
        16 => {
            strncpy(name, 0x3afd20, 30);
            updatePos(a0);
            dying(a0);
            settle = true;
        }
        32 => {
            strncpy(name, 0x3afd28, 30);
            spawn(a0);
        }
        2 => {
            strncpy(name, 0x3afd08, 30);
            updatePos(a0);
            setAnimation(a0);
            attack(a0);
        }
        4 => {
            strncpy(name, 0x3afd10, 30);
            updatePos(a0);
            setAnimation(a0);
            enemyDefend(a0);
        }
        _ => {
            strncpy(name, 0x3afd68, 30);
        }
    }
    if settle {
        setAnimation(a0);
    }
    let now = lw(mind + 2716);
    if now != lw(mind + 2720) {
        if now == 2 || now == 0x4000000 {
            sw(mind + 2728, 0);
        }
        sw(mind + 2720, now);
    }
}

/// Fire the gun `a1` of the character `a0` (@ 0x2906c8): a bullet of the kind of the gun is made (`a3` picks the second shot of the gun) at the place `a4` with the data of
/// the shot from `a2`, and by the kind of the bullet (2 to 19) it gets its look: tracers and pulses for the beams, props for the rockets, grenades, mines and the like.
pub fn gunFire(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let gun_number = lw(a1);
    sw(sp, a3);
    let mind = lw(a0 + 352);
    sw(sp + 4, a6);
    sw(sp + 8, a7);
    let gun = gun_number * 400 + g::guns;
    let shot = if a3 == 0 { gun + 52 } else { gun + 188 };
    let kind = lw(shot);
    let bullet = bulletNew(kind);
    if ((a5 - 3) as u32) >= (2 as u32) {
        sw(mind + 2912, 0);
        sw(mind + 2916, kind);
        swc1(mind + 2920, lwc1(a0 + 48));
        swc1(mind + 2924, lwc1(a0 + 52));
        swc1(mind + 2928, lwc1(a0 + 56));
    }
    let one = 1.0_f32;
    sw(bullet + 16, 0);
    swc1(bullet + 12, one);
    swc1(bullet + 20, one);
    swc1(bullet + 264, one);
    sw(bullet + 72, 0);
    swc1(bullet + 84, one);
    swc1(bullet + 88, one);
    swc1(bullet + 92, one);
    sw(bullet, kind);
    let saved = lw(sp + 4);
    swc1(bullet + 76, lwc1(shot + 20));
    sw(bullet + 108, saved);
    let f0 = lwc1(shot + 24);
    sw(bullet + 104, a5);
    swc1(bullet + 80, f0);
    sw(bullet + 156, 0);
    let room = lw(a0 + 12);
    swc1(bullet + 152, lwc1(shot + 16));
    sw(bullet + 100, room);
    let x = lwc1(a4);
    sw(bullet + 112, 0);
    swc1(bullet + 48, x);
    sw(bullet + 116, 0);
    let y = lwc1(a4 + 4);
    sw(bullet + 204, 0);
    swc1(bullet + 52, y);
    sw(bullet + 208, 0);
    let z = lwc1(a4 + 8);
    sw(bullet + 120, 0);
    sw(bullet + 144, 0);
    swc1(bullet + 56, z);
    sw(bullet + 140, 0);
    sw(bullet + 60, 0);
    sw(bullet + 256, -1);
    let saved = lw(sp + 8);
    sw(bullet + 64, 0);
    sw(bullet + 272, saved);
    sw(bullet + 68, 0);
    sw(bullet + 124, 0);
    sw(bullet + 212, 0);
    sw(bullet + 216, 1);
    sw(bullet + 220, a0);
    sw(bullet + 224, 0);
    sw(bullet + 228, -1);
    sw(bullet + 260, 0);
    sw(bullet + 268, 0);
    if a5 != 1 {
        swc1(bullet + 24, lwc1(a1 + 72));
        swc1(bullet + 28, lwc1(a1 + 76));
        swc1(bullet + 32, lwc1(a1 + 80));
    } else {
        if (lw(lw(g::currentplayer) + 20) & 2) != 0 {
            sw(bullet + 268, a5);
        }
        let gun_now = lw(a1);
        swc1(bullet + 24, lwc1(a2 + 16));
        swc1(bullet + 28, lwc1(a2 + 20));
        swc1(bullet + 32, lwc1(a2 + 24));
        if gun_now == 18 {
            let mut limit = one;
            if lw(sp) == 0 {
                limit = lwc1(0x3ad330);
            }
            let charge = lwc1(bullet + 72) + (((lw(a2 + 152) as f32)).pdiv(6e+01_f32) - lwc1(0x3ad334));
            swc1(bullet + 72, charge);
            if limit <= charge {
                sw(bullet + 48, 0);
                sw(bullet + 52, 0);
                sw(bullet + 56, 0);
            }
        }
    }
    let kind = lw(bullet);
    let direction_y = lwc1(bullet + 28);
    let index = kind - 2;
    let direction_z = lwc1(bullet + 32);
    swc1(bullet + 36, lwc1(bullet + 24));
    swc1(bullet + 40, direction_y);
    swc1(bullet + 44, direction_z);
    if (index as u32) < (18 as u32) {
        let player_bit = |count: i32| count << (lw(lw(g::currentplayer)) & 31);
        match index {
            0 | 1 | 2 => {
                let half = 0.5_f32;
                let size = lwc1(0x3ad338);
                let owner = lw(bullet + 104);
                swc1(bullet + 16, half);
                swc1(bullet + 20, size);
                swc1(bullet + 12, half);
                let at = bullet + 48;
                let mask = if owner != 1 { 15 } else { 15 - player_bit(owner) };
                let tracer = particleNewTracer(bullet + 112, a1 + 72, at, mask);
                sw(bullet + 112, tracer);
                let owner = lw(bullet + 104);
                if owner == 1 {
                    let tracer = particleNewTracer(bullet + 116, a2 + 16, at, player_bit(owner));
                    sw(bullet + 116, tracer);
                }
            }
            3 | 4 | 5 => {
                let one = 1.0_f32;
                let colour = match kind {
                    5 => 0,
                    6 => 1,
                    _ => 2,
                };
                if kind == 5 {
                    sw(bullet + 16, 0);
                    swc1(bullet + 84, one);
                    swc1(bullet + 12, one);
                    sw(bullet + 20, 0);
                    sw(bullet + 88, 0);
                    sw(bullet + 92, 0);
                } else if kind == 6 {
                    sw(bullet + 12, 0);
                    swc1(bullet + 88, one);
                    swc1(bullet + 16, one);
                    sw(bullet + 20, 0);
                    sw(bullet + 84, 0);
                    sw(bullet + 92, 0);
                } else {
                    sw(bullet + 12, 0);
                    swc1(bullet + 92, one);
                    sw(bullet + 16, 0);
                    swc1(bullet + 20, one);
                    sw(bullet + 84, 0);
                    sw(bullet + 88, 0);
                }
                let owner = lw(bullet + 104);
                let at = bullet + 48;
                let mask = if owner != 1 { 15 } else { 15 - player_bit(owner) };
                let pulse = particleNewPulse(bullet + 112, colour, a1 + 72, at, mask);
                sw(bullet + 112, pulse);
                let owner = lw(bullet + 104);
                if owner == 1 {
                    let pulse = particleNewPulse(bullet + 116, colour, a2 + 16, at, player_bit(owner));
                    sw(bullet + 116, pulse);
                }
            }
            6 | 7 => {
                let width = lwc1(0x3ad33c);
                let height = lwc1(0x3ad340);
                swc1(bullet + 16, width);
                swc1(bullet + 20, height);
                swc1(bullet + 12, width);
                let z = lwc1(a1 + 80);
                let turn = lwc1(a0 + 76) + 1.8e+02_f32;
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let prop = propNew(223, x, y, z, turn);
                let flags = lw(prop + 16);
                sw(bullet + 96, prop);
                sw(prop + 16, flags | 0x40000000);
            }
            8 | 9 | 10 | 11 => {
                match kind {
                    12 => {
                        let width = lwc1(0x3ad354);
                        let height = lwc1(0x3ad358);
                        swc1(bullet + 16, width);
                        swc1(bullet + 20, height);
                        swc1(bullet + 12, width);
                    }
                    13 => {
                        let size = lwc1(0x3ad35c);
                        swc1(bullet + 20, size);
                        swc1(bullet + 12, size);
                        swc1(bullet + 16, size);
                    }
                    _ => {
                        // kind 10 is first given the size below and then, like kind 11, the later one
                        if kind == 10 {
                            let height = lwc1(0x3ad348);
                            swc1(bullet + 12, lwc1(0x3ad344));
                            swc1(bullet + 16, height);
                            swc1(bullet + 20, 0.5_f32);
                        }
                        let height = lwc1(0x3ad34c);
                        let width = lwc1(0x3ad350);
                        swc1(bullet + 20, height);
                        swc1(bullet + 12, width);
                        swc1(bullet + 16, height);
                    }
                }
                let prop_kind = if kind == 10 || kind == 13 { 224 } else { 225 };
                let z = lwc1(a1 + 80);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let prop = propNew(prop_kind, x, y, z, 0.0_f32);
                let inst = lw(prop + 32);
                sw(bullet + 96, prop);
                propSetMtxRoll(prop, inst, bullet + 48, 35.0_f32);
                let made = lw(bullet + 96);
                sw(made + 16, lw(made + 16) | 0x40000000);
            }
            12 => {
                let width = lwc1(0x3ad368);
                let length = lwc1(0x3ad36c);
                swc1(bullet + 20, width);
                swc1(bullet + 16, length);
                swc1(bullet + 12, width);
                let mut newest = 0.0_f32;
                let z = lwc1(a1 + 80);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let prop = propNew(230, x, y, z, newest);
                let list = mind + 0x11c0;
                let flags = lw(prop + 16) | 0x40000000;
                sw(prop + 140, 2);
                sw(prop + 16, flags);
                let mut oldest = -1;
                sw(bullet + 96, prop);
                let mut placed = false;
                let mut n = 0;
                let mut at = list;
                let mut entry = lw(at);
                loop {
                    if entry == 0 {
                        sw(at, bullet);
                        placed = true;
                    } else {
                        let age = lwc1(entry + 72);
                        if newest < age {
                            newest = age;
                            oldest = n;
                        }
                    }
                    n += 1;
                    at += 4;
                    if n >= 20 {
                        break;
                    }
                    if placed {
                        break;
                    }
                    entry = lw(at);
                }
                if !placed {
                    if oldest >= 0 {
                        let slot = list + (oldest << 2);
                        let old = lw(slot);
                        sw(slot, bullet);
                        sw(old + 108, -1);
                    }
                }
            }
            13 => {
                let size = lwc1(0x3ad370);
                swc1(bullet + 20, size);
                swc1(bullet + 12, size);
                swc1(bullet + 16, size);
                let z = lwc1(a1 + 80);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let prop = propNew(231, x, y, z, 0.0_f32);
                let flags = lw(prop + 16);
                sw(bullet + 96, prop);
                sw(prop + 140, 2);
                sw(prop + 16, flags | 0x40000000);
            }
            14 => {
                let width = lwc1(0x3ad374);
                let length = lwc1(0x3ad378);
                swc1(bullet + 20, width);
                swc1(bullet + 12, length);
                swc1(bullet + 16, width);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let z = lwc1(a1 + 80);
                let first = propNew(227, x, y, z, 0.0_f32);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let z = lwc1(a1 + 80);
                sw(first + 16, lw(first + 16) | 0x40000000);
                sw(bullet + 96, first);
                sw(first + 140, 2);
                let second = propNew(228, x, y, z, 0.0_f32);
                let z = lwc1(a1 + 80);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                sw(second + 16, lw(second + 16) | 0x40000000);
                sw(second + 140, 2);
                sw(bullet + 144, second);
                let third = propNew(229, x, y, z, 0.0_f32);
                let flags = lw(third + 16);
                sw(third + 140, 2);
                sw(bullet + 140, third);
                sw(third + 16, flags | 0x40000000);
            }
            15 | 16 => {
                let width = lwc1(0x3ad360);
                let length = lwc1(0x3ad364);
                swc1(bullet + 16, width);
                swc1(bullet + 20, length);
                swc1(bullet + 12, width);
                let z = lwc1(a1 + 80);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let prop = propNew(232, x, y, z, 0.0_f32);
                let flags = lw(prop + 16) & -33;
                sw(bullet + 96, prop);
                sw(prop + 140, 2);
                sw(prop + 16, flags);
                let random = random_as_float();
                sw(bullet + 64, 0);
                swc1(bullet + 60, (random * 2.3283064e-10_f32 - 0.5_f32) * 7e+01_f32);
                let random = random_as_float();
                swc1(bullet + 68, (random * 2.3283064e-10_f32 - 0.5_f32) * 7e+01_f32);
            }
            _ => {
                // kind 19
                sw(bullet + 12, 0);
                sw(bullet + 16, 0);
                sw(bullet + 20, 0);
                let w = lwc1(bullet + 12);
                let z = lwc1(a1 + 80);
                let x = lwc1(a1 + 72);
                let y = lwc1(a1 + 76);
                let prop = propNew(233, x, y, z, w);
                let flags = lw(prop + 16);
                sw(bullet + 96, prop);
                sw(prop + 140, 2);
                sw(prop + 16, flags | 0x40000000);
                let random = random_as_float();
                sw(bullet + 64, 0);
                swc1(bullet + 60, (random * 2.3283064e-10_f32 - 0.5_f32) * 7e+01_f32);
                let random = random_as_float();
                swc1(bullet + 68, (random * 2.3283064e-10_f32 - 0.5_f32) * 7e+01_f32);
                let height = lwc1(bullet + 52);
                let raised = if lw(sp) != 0 { height + lwc1(0x3ad380) } else { height + lwc1(0x3ad37c) };
                swc1(bullet + 52, raised);
            }
        }
    }
}
