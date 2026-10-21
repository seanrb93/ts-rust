//! Functions: the tick of the prop of a character, moving the parts of a gun, putting a computer character into the level and the move test of a character.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    random_as_float() * 2.3283064e-10_f32
}

/// Set the animation state of the character prop `chr` (the word at +344) and return it.
fn set_anim_state(chr: i32, state: i32) -> i32 {
    sw(chr + 344, state);
    state
}

/// Choose the new animation state of a computer character `chr` (with the mind `mind`) (the word at +344, 0 to 9): a character on foot, moving or standing and with a gun that is
/// held in one hand or both, may keep its state, take a new one or take one at random. `riding` is the flag at +484 of the mind. Returns the state.
fn npc_anim_state(chr: i32, mind: i32, riding: i32) -> i32 {
    let keep = || lw(chr + 344);
    let gun_flags = || lw(lw(mind + 376) * 400 + (g::guns + 28));
    if riding == 0 {
        let moving = lw(mind + 332);
        let aiming = lw(mind + 440);
        if moving == 0 && aiming == 0 {
            return set_anim_state(chr, 0);
        }
        if moving != 0 && aiming != 0 {
            let gun = gun_flags();
            let flags = lw(mind + 2708);
            if (gun & 2) != 0 {
                let both_hands = gun & 4;
                if (flags & 16) == 0 {
                    if both_hands == 0 {
                        let state = keep();
                        if (state as u32) < (2 as u32) {
                            return state;
                        }
                        if state == 6 {
                            return state;
                        }
                        if state == 9 {
                            return state;
                        }
                        let random = random_unit();
                        if random < 0.25_f32 {
                            return set_anim_state(chr, 0);
                        }
                        if random < 0.5_f32 {
                            return set_anim_state(chr, 1);
                        }
                        if !(random < 0.75_f32) {
                            return set_anim_state(chr, 9);
                        }
                    }
                    return set_anim_state(chr, 6);
                } else if both_hands == 0 {
                    let state = keep();
                    if state == 6 {
                        return state;
                    }
                    if state == 9 {
                        return state;
                    }
                    if state == 5 {
                        return state;
                    }
                    let random = random_unit();
                    if !(random < lwc1(0x3acc90)) {
                        if random < lwc1(0x3acc94) {
                            return set_anim_state(chr, 9);
                        }
                        return set_anim_state(chr, 5);
                    }
                }
                return set_anim_state(chr, 6);
            }
            if (flags & 16) == 0 {
                let state = keep();
                if state < 2 {
                    return state;
                }
                let random = random_unit();
                if 0.5_f32 < random {
                    return set_anim_state(chr, 0);
                }
                return set_anim_state(chr, 1);
            }
            let state = keep();
            if state == 2 {
                return state;
            }
            if state == 5 {
                return state;
            }
            let random = random_unit();
            if lwc1(0x3acc98) < random {
                return set_anim_state(chr, 2);
            }
            return set_anim_state(chr, 5);
        }
        let gun = gun_flags();
        let flags = lw(mind + 2708);
        if (gun & 2) != 0 {
            let both_hands = gun & 4;
            if (flags & 16) == 0 {
                if both_hands == 0 {
                    let state = keep();
                    if state == 1 {
                        return state;
                    }
                    if state == 3 {
                        return state;
                    }
                    let random = random_unit();
                    if random < 0.5_f32 {
                        return set_anim_state(chr, 1);
                    }
                }
                return set_anim_state(chr, 3);
            }
            if both_hands == 0 {
                let state = keep();
                if state == 2 {
                    return state;
                }
                if state == 4 {
                    return state;
                }
                if lw(mind + 2716) == 0x100000 {
                    return set_anim_state(chr, 4);
                }
                let random = random_unit();
                if 0.5_f32 < random {
                    return set_anim_state(chr, 2);
                }
            }
            return set_anim_state(chr, 4);
        }
        if (flags & 16) == 0 {
            let state = keep();
            if (state as u32) < (2 as u32) {
                return state;
            }
            let random = random_unit();
            if 0.5_f32 < random {
                return set_anim_state(chr, 0);
            }
            return set_anim_state(chr, 1);
        }
        return set_anim_state(chr, 2);
    }
    if lw(mind + 440) != 0 {
        let flags = lw(mind + 2708);
        if (gun_flags() & 2) == 0 {
            if (flags & 16) == 0 {
                let state = keep();
                if ((state - 7) as u32) < (2 as u32) {
                    return state;
                }
                let random = random_unit();
                if 0.5_f32 < random {
                    return set_anim_state(chr, 7);
                }
            }
        } else {
            let state = keep();
            if (flags & 16) == 0 {
                if state >= 7 {
                    return state;
                }
                let random = random_unit();
                if random < lwc1(0x3acc88) {
                    return set_anim_state(chr, 7);
                }
                if !(random < lwc1(0x3acc8c)) {
                    return set_anim_state(chr, 9);
                }
            } else {
                if ((state - 8) as u32) < (2 as u32) {
                    return state;
                }
                let random = random_unit();
                if !(0.5_f32 < random) {
                    return set_anim_state(chr, 9);
                }
            }
        }
        return set_anim_state(chr, 8);
    }
    set_anim_state(chr, 7)
}

/// The tick of the prop `a0` of a character (@ 0x26ddc0): the glow of its hit colour, the statistics of the time it has been aiming, in a gun fight and in the lead, and then
/// the choice of its animation state and the tick of the mind of a computer character.
pub fn chrPropTick(a0: i32) {
    let mind = lw(a0 + 352);
    let glow_time = lwc1(mind + 2900);
    let frame_time = lwc1(g::updateframesf);
    if 0.0_f32 < glow_time {
        swc1(mind + 2900, glow_time - frame_time / 6e+01_f32);
    }
    if lw(mind + 0x11a8) == 0 {
        let mut amount = lwc1(mind + 2900) / 2.5_f32;
        if 1.0_f32 < amount {
            amount = 1.0_f32;
        }
        if !(0.0_f32 < amount) {
            obInstSetOverrideCol(lw(a0 + 32), 0, 0, 0, 0.0_f32);
        } else {
            let strength = amount.abs().sqrt();
            let wave = (lwc1((((lw(g::gameframe) as f32 * 6.0_f32 / 6e+01_f32 * lwc1(0x3acc74) + 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) + 1.0_f32) * 0.5_f32 * 0.5_f32 + 0.5_f32;
            let hold = lw(mind + 0x11a4);
            let glow = wave * strength;
            if hold == 0 {
                let inst = lw(a0 + 32);
                obInstSetOverrideCol(inst, lw(inst + 316), lw(inst + 320), lw(inst + 324), glow);
            } else {
                let team = lw(hold + 28);
                if team >= 0 {
                    let (red, green, blue);
                    if team == 1 {
                        red = 255;
                        green = 255;
                        blue = 64;
                    } else if team >= 2 {
                        if team == 2 {
                            red = 64;
                            green = 64;
                        } else {
                            red = 64;
                            green = if team == 3 { 255 } else { 64 };
                        }
                        blue = 255;
                    } else if team == 0 {
                        red = 255;
                        green = 64;
                        blue = 64;
                    } else {
                        red = 64;
                        green = 64;
                        blue = 255;
                    }
                    obInstSetOverrideCol(lw(a0 + 32), red, green, blue, glow);
                }
            }
        }
    }
    'stat: {
        let mut frame_time = lwc1(g::updateframesf);
        if lw(mind + 260) == 0 {
            let punching = gunIsPunching(mind);
            frame_time = lwc1(g::updateframesf);
            if punching == 0 {
                break 'stat;
            }
        }
        StatsAddGun(3, lw(mind), lw(mind + 260), frame_time * lwc1(0x3acc78));
    }
    let frame_time = lwc1(g::updateframesf);
    if lw(mind + 484) != 0 {
        StatsAdd(36, lw(mind), frame_time * lwc1(0x3acc7c));
    }
    let rank = gameGetRankTeam(lw(mind + 24));
    let players = lw(g::lvnumplayers);
    let level;
    if rank != 0 {
        let mut level_now = lw(g::lvlevelnum);
        if rank == players + lw(g::numactivechars) - 1 {
            StatsAdd(41, lw(mind), lwc1(g::updateframesf) * lwc1(0x3acc84));
            level_now = lw(g::lvlevelnum);
        }
        level = level_now;
    } else {
        StatsAdd(40, lw(mind), lwc1(g::updateframesf) * lwc1(0x3acc80));
        level = lw(g::lvlevelnum);
    }
    if level == 102 {
        sw(a0 + 344, 0);
        return;
    }
    if lw(mind + 8) != 0 {
        enemyTick(a0);
        return;
    }
    let state = lw(a0 + 344);
    let players = lw(g::lvnumplayers);
    sw(a0 + 348, state);
    if lw(mind) >= players {
        if state >= 20 {
            sw(a0 + 344, state - 20);
        }
        let riding = lw(mind + 484);
        let mut new_state = npc_anim_state(a0, mind, riding);
        let data = lw(a0 + 32);
        let model = lw(data + 88);
        if model != 0 {
            if lw(model + 12) == 1 {
                new_state += 20;
                sw(a0 + 344, new_state);
            }
        }
        let previous = lw(a0 + 348);
        if new_state != previous {
            let animation = lw(data + 96) - 3;
            if (animation as u32) < (440 as u32) {
                setAnim(a0, animation.wrapping_rem(11) + 3, lwc1(0x3acc9c), lwc1(data + 140));
            }
        }
        enemyTick(a0);
        return;
    }
    'set: {
        if lw(mind + 484) == 0 {
            let moving = lw(mind + 332);
            let aiming = lw(mind + 440);
            if moving == 0 {
                if aiming == 0 {
                    sw(a0 + 344, 0);
                    break 'set;
                }
            } else if aiming != 0 {
                'zero: {
                    let gun = lw(lw(mind + 376) * 400 + (g::guns + 28));
                    if (gun & 2) != 0 {
                        if (gun & 4) == 0 {
                            if (lw(mind + 2708) & 16) == 0 {
                                break 'zero;
                            }
                        }
                        sw(a0 + 344, 6);
                        break 'set;
                    }
                    if (lw(mind + 2708) & 16) != 0 {
                        sw(a0 + 344, 5);
                        break 'set;
                    }
                }
                sw(a0 + 344, 0);
                break 'set;
            }
            let flags = lw(mind + 2708);
            if (lw(lw(mind + 376) * 400 + (g::guns + 28)) & 2) == 0 {
                if (flags & 16) == 0 {
                    sw(a0 + 344, 1);
                } else {
                    sw(a0 + 344, 2);
                }
            } else if (flags & 16) == 0 {
                sw(a0 + 344, 3);
            } else {
                sw(a0 + 344, 4);
            }
        } else {
            'seven: {
                if lw(mind + 440) != 0 {
                    let gun = lw(lw(mind + 376) * 400 + (g::guns + 28));
                    if (gun & 2) != 0 {
                        if (gun & 4) == 0 {
                            if (lw(mind + 2708) & 16) == 0 {
                                break 'seven;
                            }
                        }
                        sw(a0 + 344, 9);
                        break 'set;
                    }
                    if (lw(mind + 2708) & 16) != 0 {
                        sw(a0 + 344, 8);
                        break 'set;
                    }
                }
            }
            sw(a0 + 344, 7);
        }
    }
    if lw(lw(lw(a0 + 32) + 88) + 12) == 1 {
        sw(a0 + 344, lw(a0 + 344) + 20);
    }
}

/// Update the moving parts of the gun `a0` of the kind `a1` (@ 0x2911e0): its time and shots, its sway and recoil, the shock of a pad, the speed of its parts (slides,
/// spins) and then the matrices of the parts of its model.
pub fn gunUpdateParts(a0: i32, a1: i32) {
    let frame = Frame::new(368);
    let sp = frame.sp();
    let mut n = 0;
    let mut owner = lw(a0 + 12);
    let gun = lw(a1) * 400 + g::guns;
    let shot = if lw(a0 + 52) == 0 { gun + 52 } else { gun + 188 };
    if lw(lw(lw(owner + 32)) + 4) > 0 {
        loop {
            let offset = n << 6;
            n += 1;
            sceVu0UnitMatrix(lw(lw(owner + 32) + 4) + offset);
            owner = lw(a0 + 12);
            if n >= lw(lw(lw(owner + 32)) + 4) {
                break;
            }
        }
    }
    let time = lw(a0 + 152);
    let frames = lw(g::updateframesi);
    if time != -1 {
        sw(a0 + 152, time + frames);
    }
    let mut kind;
    let mut frames_now;
    'fire: {
        kind = lw(a1);
        if lw(a0 + 156) == 0 {
            if lw(a0 + 152) >= lw(gun + 388) {
                frames_now = lw(g::updateframesi);
                if kind != 18 {
                    if frames_now <= 0 {
                        break 'fire;
                    }
                    playerFireShots(a0);
                    kind = lw(a1);
                    owner = lw(a0 + 12);
                }
            }
        }
        frames_now = lw(g::updateframesi);
    }
    let sin_table = g::g_SinTable;
    let phase_scale = lwc1(0x3ad388);
    let phase = lw(g::gameframe) as f32 / 2e+01_f32;
    let half = 0.5_f32;
    let step = lwc1(g::updateframesf);
    let owner_y = lwc1(owner + 52);
    let owner_z = lwc1(owner + 56);
    let z_sway = (((phase * lwc1(0x3ad38c) * phase_scale + half) as i32 & 2047) << 2) + sin_table;
    swc1(
        owner + 48,
        lwc1(owner + 48)
            + (lwc1(a0 + 68) + lwc1((((phase * lwc1(0x3ad384) * phase_scale + half) as i32 & 2047) << 2) + sin_table) * lwc1(shot + 88) * lwc1(a0 + 120) + lwc1(a0 + 92) * lwc1(a0 + 116)),
    );
    swc1(
        owner + 52,
        owner_y + (lwc1(a0 + 72) + lwc1((((phase * phase_scale + half) as i32 & 2047) << 2) + sin_table) * lwc1(shot + 88) * lwc1(a0 + 120) + lwc1(a0 + 96) * lwc1(a0 + 116)),
    );
    swc1(owner + 56, owner_z + (lwc1(a0 + 76) + lwc1(z_sway) * lwc1(shot + 88) * lwc1(a0 + 120) + lwc1(a0 + 100) * lwc1(a0 + 116)));
    swc1(a0 + 84, lwc1(a0 + 84) + lwc1(a0 + 88) * step);
    if kind == 23 {
        if lw(a0 + 152) >= 9 {
            sw(a0 + 84, 0);
        }
    }
    if lwc1(0x3ad390) < lwc1(a0 + 188) {
        let player = lw(g::currentplayer);
        if frames_now != 0 {
            if (lhu(lw(player + 24) + 2) & 80) == 80 {
                let left = joyGetShockLeft(lw(player));
                let right = joyGetShockRight(lw(lw(g::currentplayer)));
                joySetShock(lw(player), left, right + (lwc1(a0 + 188) * 128.0_f32) as i32);
                kind = lw(a1);
                frames_now = lw(g::updateframesi);
            }
        }
    }
    let time = lw(a0 + 152);
    if frames_now > 0 {
        let mut n = 0;
        loop {
            n += 1;
            let decay = lwc1(0x3ad394);
            let x = lwc1(a0 + 68);
            let y = lwc1(a0 + 72);
            let z = lwc1(a0 + 76);
            let spin = lwc1(a0 + 88);
            let slide = lwc1(a0 + 116);
            let pull = lwc1(a0 + 120);
            swc1(a0 + 60, lwc1(a0 + 60) * (lwc1(shot + 68) * decay));
            let shock = lwc1(a0 + 188);
            swc1(a0 + 68, x * (lwc1(shot + 68) * decay));
            swc1(a0 + 72, y * (lwc1(shot + 68) * decay));
            swc1(a0 + 76, z * (lwc1(shot + 68) * decay));
            swc1(a0 + 88, spin * (lwc1(shot + 84) * decay));
            swc1(a0 + 116, slide * (lwc1(shot + 100) * decay));
            swc1(a0 + 120, pull * (lwc1(shot + 92) * decay));
            swc1(a0 + 188, shock * (lwc1(shot + 132) * decay));
            if n >= frames_now {
                break;
            }
        }
    }
    'part: {
        'cartridge: {
            let value;
            'moved: {
                'range: {
                    if kind != 20 {
                        let power;
                        if time != -1 {
                            power = lwc1(shot + 76);
                        } else {
                            if lw(a0 + 56) != 0 {
                                break 'range;
                            }
                            power = lwc1(shot + 76);
                        }
                        let frames = lwc1(g::updateframesf);
                        let damped = powf(power, frames);
                        value = lwc1(a0 + 80) * damped;
                        break 'moved;
                    }
                }
                let age = time as f32;
                let eject_start;
                if !(25.0_f32 <= age) {
                    eject_start = 33.0_f32;
                } else {
                    eject_start = 33.0_f32;
                    let frames = lwc1(g::updateframesf);
                    if age < eject_start {
                        value = lwc1(a0 + 80) + lwc1(shot + 72) * frames;
                        break 'moved;
                    }
                }
                if eject_start < age {
                    if lw(a0 + 164) == 0 {
                        fireCartridges(a0, a1, gun, shot);
                        sw(a0 + 164, 1);
                        break 'cartridge;
                    }
                }
                let age = time as f32;
                if !(37.0_f32 < age) {
                    break 'part;
                }
                let frames = lwc1(g::updateframesf);
                if !(age < 5e+01_f32) {
                    break 'part;
                }
                value = lwc1(a0 + 80) - lwc1(shot + 72) * frames;
            }
            swc1(a0 + 80, value);
        }
    }
    let frames = lwc1(g::updateframesf);
    'slide: {
        let y = lwc1(a0 + 96) + lwc1(a0 + 108) * frames;
        let mut mode = lw(a0 + 124);
        let z = lwc1(a0 + 100) + lwc1(a0 + 112) * frames;
        swc1(a0 + 92, lwc1(a0 + 92) + lwc1(a0 + 104) * frames);
        swc1(a0 + 96, y);
        swc1(a0 + 100, z);
        if mode != 1 {
            let damped = powf(lwc1(shot + 112), frames);
            swc1(a0 + 128, lwc1(a0 + 128) * damped);
        } else {
            let moved = lwc1(a0 + 128) + lwc1(shot + 104) * frames;
            swc1(a0 + 128, moved);
            let limit = lwc1(shot + 108);
            let reached = if !(lwc1(shot + 104) < 0.0_f32) { limit < moved } else { moved < limit };
            if reached {
                swc1(a0 + 128, limit);
                sw(a0 + 124, 2);
                mode = 2;
            }
            if lw(gun + 392) < mode {
                sw(a0 + 124, 0);
            }
            let kind_now = lw(a1);
            if kind_now != 23 {
                if kind_now != 9 {
                    break 'slide;
                }
            }
            if lw(a0 + 56) != 0 {
                if lw(a0 + 152) != -1 {
                    break 'slide;
                }
                sw(a0 + 124, 1);
            }
        }
    }
    let mode = lw(a0 + 124);
    'turn: {
        if mode != 2 {
            if mode == 1 {
                break 'turn;
            }
            let damped = powf(lwc1(shot + 124), lwc1(g::updateframesf));
            swc1(a0 + 132, lwc1(a0 + 132) * damped);
        } else {
            let moved = lwc1(a0 + 132) + lwc1(shot + 116) * lwc1(g::updateframesf);
            swc1(a0 + 132, moved);
            let limit = lwc1(shot + 120);
            let reached = if !(lwc1(shot + 116) < 0.0_f32) { limit < moved } else { moved < limit };
            if !reached {
                break 'turn;
            }
            swc1(a0 + 132, limit);
            sw(a0 + 124, 0);
        }
    }
    let bone = lw(gun + 368);
    let barrel_bone;
    if bone < 0 {
        barrel_bone = lw(gun + 352);
    } else {
        let shift = lwc1(a0 + 80);
        let matrix = lw(lw(lw(a0 + 12) + 32) + 4) + (bone << 6);
        matrixTrans(matrix, 0.0_f32, 0.0_f32, shift);
        if lw(a1) != 5 {
            barrel_bone = lw(gun + 352);
        } else {
            let shift = lwc1(a0 + 80);
            let model = lw(lw(a0 + 12) + 32);
            let header = lw(model);
            let minus_twelve = -12.0_f32;
            let first = lw(model + 4) + 320;
            let scale = lwc1(0x3ad398);
            let full = 1.8e+02_f32;
            let skeleton = header - lw(header) * 80;
            let pivot = lw(skeleton + 72);
            let at = pivot + 12;
            matrixTrans(first, lwc1(pivot + 12), lwc1(at + 4), lwc1(at + 8) + shift);
            matrixRotZXY(sp, 0.0_f32, shift * 9e+01_f32 * minus_twelve * scale / full, 0.0_f32);
            sceVu0MulMatrix(first, first, sp);
            matrixTrans(sp, -lwc1(pivot + 12), -lwc1(at + 4), -lwc1(at + 8));
            sceVu0MulMatrix(first, first, sp);
            let pivot = lw(skeleton + 72);
            let at = pivot + 24;
            let second = lw(lw(lw(a0 + 12) + 32) + 4) + 256;
            matrixTrans(second, lwc1(pivot + 24), lwc1(at + 4), lwc1(at + 8) + lwc1(a0 + 80));
            matrixRotZXY(sp, 0.0_f32, shift * -9e+01_f32 * minus_twelve * scale / full, 0.0_f32);
            sceVu0MulMatrix(second, second, sp);
            matrixTrans(sp, -lwc1(pivot + 24), -lwc1(at + 4), -lwc1(at + 8));
            sceVu0MulMatrix(second, second, sp);
            barrel_bone = lw(gun + 352);
        }
    }
    let second_bone;
    'bone: {
        if barrel_bone >= 0 {
            let offset = barrel_bone * 12;
            let model = lw(lw(a0 + 12) + 32);
            let header = lw(model);
            let matrix = lw(model + 4) + (lw(gun + 372) << 6);
            let pivot = lw(header - lw(header) * 80 + 72) + offset;
            matrixTrans(matrix, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
            matrixRotZXY(sp, 0.0_f32, 0.0_f32, lwc1(a0 + 84) * lwc1(0x3ad39c) / 1.8e+02_f32);
            sceVu0MulMatrix(matrix, matrix, sp);
            matrixTrans(sp, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
            sceVu0MulMatrix(matrix, matrix, sp);
            let kind_now = lw(a1);
            if kind_now != 11 {
                if kind_now != 13 {
                    second_bone = lw(gun + 356);
                    break 'bone;
                }
            }
            let mut n = 2;
            loop {
                let target = lw(lw(lw(a0 + 12) + 32) + 4) + (n << 6);
                n += 1;
                sceVu0CopyMatrix(target, matrix);
                if n >= 10 {
                    break;
                }
            }
        }
        second_bone = lw(gun + 356);
    }
    if second_bone >= 0 {
        let model = lw(lw(a0 + 12) + 32);
        let header = lw(model);
        let matrix = lw(model + 4) + (lw(gun + 376) << 6);
        let pivot = lw(header - lw(header) * 80 + 72) + second_bone * 12;
        matrixTrans(matrix, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
        matrixRotZXY(sp + 64, lwc1(a0 + 128) * lwc1(0x3ad3a0) / 1.8e+02_f32, 0.0_f32, 0.0_f32);
        sceVu0MulMatrix(matrix, matrix, sp + 64);
        matrixTrans(sp + 64, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
        sceVu0MulMatrix(matrix, matrix, sp + 64);
    }
    let third_bone = lw(gun + 360);
    if third_bone >= 0 {
        let model = lw(lw(a0 + 12) + 32);
        let header = lw(model);
        let matrix = lw(model + 4) + (lw(gun + 380) << 6);
        let pivot = lw(header - lw(header) * 80 + 72) + third_bone * 12;
        matrixTrans(matrix, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
        matrixRotZXY(sp + 128, lwc1(a0 + 132) * lwc1(0x3ad3a4) / 1.8e+02_f32, 0.0_f32, 0.0_f32);
        sceVu0MulMatrix(matrix, matrix, sp + 128);
        matrixTrans(sp + 128, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
        sceVu0MulMatrix(matrix, matrix, sp + 128);
    }
    let kind_now = lw(a1);
    if kind_now == 18 {
        let owner = lw(a0 + 12);
        swc1(owner + 88, lwc1(owner + 88) - 5e+01_f32);
    }
    if kind_now == 25 {
        let owner = lw(a0 + 12);
        let moved = lwc1(owner + 72) - 3e+01_f32;
        swc1(owner + 88, lwc1(owner + 88) - 5e+01_f32);
        swc1(owner + 72, moved);
    }
}

/// Put the computer character `a0` into the level (@ 0x2c4288): on its first time it gets a pad (and its first state by its kind), on a later time it is respawned as it was;
/// then its position, speeds and reactions are set up, it is moved to the floor and the effect of the spawn is started.
pub fn spawn(a0: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let mind = lw(a0 + 352);
    sw(sp + 32, ((mind.wrapping_sub(lw(g::chrinfo))).wrapping_mul(-0x795b3d1f)) >> 4);
    let combos = lvGetGunComboNums();
    let spawn_fx = lw(mind + 0x11a4);
    if spawn_fx != 0 {
        specialfxFreeSpawn(spawn_fx);
        let group = lw(mind + 0x11a8);
        sw(mind + 0x11a4, 0);
        if group != 0 {
            specialfxFreePtr(group);
            sw(mind + 0x11a8, 0);
        }
    }
    let at = a0 + 48;
    let old_flags = lw(mind + 2704);
    'pre: {
        if (old_flags & 16) == 0 {
            chrRespawn(mind);
            chrClearGuns(mind);
            sw(mind + 2788, 0);
            let mode = lw(g::curgameparms + 72);
            if mode != 7 {
                let mut escorted = false;
                if mode == 4 {
                    if a0 == lw(g::escort) {
                        let route = lw(lw(g::setup) + 48);
                        sw(mind + 2720, lw(mind + 2716));
                        sw(mind + 2716, 0x10000);
                        let first = lw(route);
                        sw(mind + 2892, 0);
                        let pad = padNumFromExtref(lw(first));
                        sw(mind + 680, pad);
                        sw(mind + 2788, 0);
                        escorted = true;
                    }
                }
                if !escorted {
                    let mut tries = 0;
                    while tries < 10 {
                        let pad = lvGetRandomAiSpawn(mind);
                        sw(mind + 680, pad);
                        if campingPad(pad) == 0 {
                            break;
                        }
                        tries += 1;
                    }
                    campingPad(lw(mind + 680));
                    let kind = lw(mind + 8);
                    sw(mind + 2788, 0);
                    if kind >= 3 {
                        if kind == 3 {
                            let state = lw(mind + 2716);
                            sw(mind + 376, 0);
                            sw(mind + 268, 0);
                            sw(mind + 2892, 0);
                            sw(mind + 260, 1);
                            sw(mind + 2720, state);
                            sw(mind + 2716, 1024);
                            let random = random_unit();
                            swc1(mind + 2892, random);
                        }
                    } else if kind > 0 {
                        let state = lw(mind + 2716);
                        sw(mind + 2716, 128);
                        sw(mind + 2720, state);
                        sw(mind + 2892, 0);
                    } else if kind == 0 {
                        setAnim(a0, 3, 0.0_f32, 2.0_f32);
                        let state = lw(mind + 2716);
                        swc1(mind + 2892, 0.0_f32);
                        sw(mind + 2720, state);
                        sw(mind + 2716, 64);
                    }
                }
                let parms = g::curgameparms;
                if (lw(parms + 80) & 32) != 0 {
                    if lw(parms + 68) == 24 {
                        sw(mind + 2716, 0x2000);
                    }
                }
                break 'pre;
            }
            let target = suggestTarget(a0, lw(lw(g::players) + 384), 0);
            let setup = lw(g::setup);
            let players = lw(g::lvnumplayers);
            let number = lw(mind);
            let inside = (number < lw(setup + 44) + players) as i32;
            sw(mind + 2788, target);
            if inside == 0 {
                sw(mind + 2716, 0x2000);
                sw(mind + 2724, 8);
                break 'pre;
            }
            let entry = lw(setup + 40) + (number - players) * 24;
            if lvCheckCreateFlags(lw(entry + 8)) == 0 {
                enemyDeleteNPC(a0);
                return;
            }
            let pad = padNumFromExtref(lw(entry));
            sw(mind + 680, pad);
            let other = padNumFromExtref(lw(entry + 4));
            sw(mind + 2696, other);
            let gun = lw(entry + 16);
            if gun != -1 {
                sw(mind + 260, lw((gun << 2) + combos));
            }
            let kind_flags = lw(entry + 12);
            if (kind_flags & 0x40000) == 0 {
                sw(mind + 2716, 0x2000);
                sw(mind + 2724, lw(entry + 12));
            } else {
                sw(mind + 2724, 0);
                sw(mind + 2716, kind_flags & -0x40001);
            }
            let mut extra = lw(entry + 20);
            if extra == 0 {
                break 'pre;
            }
            if (lw(entry + 12) & 0x10000) != 0 {
                break 'pre;
            }
            let mut bit = extra & 2;
            if (extra & 1) != 0 {
                sw(mind + 2704, lw(mind + 2704) & -33);
                extra = lw(entry + 20);
                bit = extra & 2;
            }
            let mut bit_four = extra & 4;
            if bit != 0 {
                sw(mind + 2704, lw(mind + 2704) & -257);
                extra = lw(entry + 20);
                bit_four = extra & 4;
            }
            let bit_eight;
            if bit_four != 0 {
                sw(mind + 2704, lw(mind + 2704) | 1024);
                bit_eight = lw(entry + 20) & 8;
            } else {
                bit_eight = extra & 8;
            }
            if bit_eight != 0 {
                sw(mind + 2704, lw(mind + 2704) | 0x8000);
            }
        } else {
            let gun = lw(mind + 260);
            let mut kind_flags = lw(mind + 2724);
            let target = lw(mind + 2788);
            let hold = lw(mind + 2768);
            let state = lw(mind + 2728);
            chrRespawn(mind);
            sw(mind + 2704, lw(mind + 2704) | 16);
            let players = lw(g::players);
            if lw(g::curgameparms + 72) != 6 {
                let new_target = suggestTarget(a0, lw(players + 384), 0);
                sw(mind + 2788, new_target);
            } else {
                sw(mind + 2788, target);
            }
            sw(mind + 260, gun);
            if (kind_flags & 0x40000) == 0 {
                sw(mind + 2724, kind_flags);
                sw(mind + 2716, 0x2000);
            } else {
                sw(mind + 2724, 0);
                kind_flags &= -0x40001;
                sw(mind + 2716, kind_flags);
            }
            if hold != -1 {
                break 'pre;
            }
            if kind_flags != 0x10000 {
                sw(mind + 2696, state);
            } else {
                sw(mind + 2728, state);
            }
            sw(mind + 2704, 16);
            if (old_flags & 1) == 0 {
                sw(mind + 2704, 48);
            }
            if (old_flags & 2) == 0 {
                sw(mind + 2704, lw(mind + 2704) | 256);
            } else {
                sw(mind + 2704, lw(mind + 2704) & -257);
            }
            if (old_flags & 4) != 0 {
                sw(mind + 2704, lw(mind + 2704) | 1024);
            }
            if (old_flags & 8) != 0 {
                sw(mind + 2704, lw(mind + 2704) | 0x8000);
            }
        }
    }
    let kind = lw(mind + 2724);
    sw(mind + 684, -1);
    sw(mind + 1688, 0);
    sw(mind + 2692, 0);
    swc1(a0 + 520, 1.0_f32);
    let register_pad = |sp_pad: i32| {
        let n = lw(g::numtriggerpads);
        let listed = lw(mind + 2696);
        sw(mind + 2708, sp_pad);
        sw((n << 3) + 0x1fedf00, listed);
        let number = lw(mind);
        sw(g::numtriggerpads, n + 1);
        sw((n << 3) + 0x1fedf00 + 4, number);
    };
    if kind == 0x80000 {
        register_pad(12);
        setAnim(a0, 113, 0.0_f32, 1.0_f32);
    } else {
        'b23: {
            let register;
            if (0x80000 as u32) < (kind as u32) {
                if kind != 0x400000 {
                    if (0x400000 as u32) < (kind as u32) {
                        if kind == 0x2000000 {
                            sw(mind + 260, 0);
                            sw(mind + 2708, 8);
                            break 'b23;
                        }
                        sw(mind + 2708, 1);
                        break 'b23;
                    }
                    if kind != 0x200000 {
                        sw(mind + 2708, 1);
                        break 'b23;
                    }
                }
                register = true;
            } else if kind != 0x4000 {
                if kind != 0x8000 {
                    sw(mind + 2708, 1);
                    break 'b23;
                }
                register = true;
            } else {
                register = true;
            }
            if register {
                register_pad(1);
            }
        }
    }
    let pad = lw(mind + 680) * 28 + lw(g::levelpads);
    let speed = lwc1(mind + 2856);
    let parms = g::curgameparms;
    swc1(mind + 2956, lwc1(0x3afd70));
    swc1(mind + 2872, speed);
    let raise = lwc1(0x3ad93c);
    let x = lwc1(pad + 12);
    sw(mind + 2720, 0);
    swc1(a0 + 48, x);
    sw(mind + 2768, 0);
    let y = lwc1(pad + 16);
    sw(mind + 2772, 0);
    sw(mind + 3004, 0);
    let mode = lw(parms + 72);
    swc1(a0 + 52, y + raise);
    swc1(a0 + 56, lwc1(pad + 20));
    let angle = lwc1(pad + 24);
    swc1(a0 + 84, angle);
    swc1(a0 + 76, angle);
    swc1(a0 + 80, angle);
    sw(mind + 2968, 0);
    swc1(mind + 2972, lwc1(a0 + 76));
    swc1(mind + 2988, lwc1(a0 + 48));
    swc1(mind + 2992, lwc1(a0 + 52));
    let z = lwc1(a0 + 56);
    swc1(mind + 2960, 1.0_f32);
    swc1(mind + 2996, z);
    if mode != 7 {
        let random = random_unit();
        let number = lw(mind);
        let level = (4 - lbu(parms + number + 49)) as f32;
        let parms2 = g::curgameparms;
        let skill = lw(parms2 + 108) as f32 / 4e+01_f32;
        let index = number - lw(g::lvnumplayers);
        let c1 = lwc1(0x3ad95c);
        let c2 = skill * lwc1(0x3ad94c) + lwc1(0x3ad954);
        swc1(mind + 2832, (level + random + lwc1(0x3ad950)) * lwc1(0x3ad958));
        swc1(mind + 2836, index as f32 / lw(parms2 + 8) as f32 * c2 + ((1.0_f32 - skill) * c1 + lwc1(0x3ad960)));
    } else {
        if lw(parms + 76) == 0 {
            let random = random_unit();
            swc1(mind + 2832, (random + 2.0_f32) * lwc1(0x3ad940));
        }
        if lw(parms + 76) == 1 {
            let random = random_unit();
            swc1(mind + 2832, (random + 1.0_f32) * lwc1(0x3ad944));
        }
        if lw(parms + 76) == 2 {
            let random = random_unit();
            swc1(mind + 2832, (random + 0.0_f32) * lwc1(0x3ad948));
        }
        swc1(mind + 2836, 0.5_f32);
    }
    let mut do_spawn = 0;
    let mut effect = -1;
    let mut colour = -1;
    'b33: {
        let radius = lwc1(0x3ad964);
        swc1(mind + 2964, lwc1(mind + 2964) * (lwc1(mind + 2832) + 0.25_f32));
        let y = lwc1(a0 + 52);
        let z = lwc1(a0 + 56);
        swc1(sp, lwc1(a0 + 48));
        swc1(sp + 4, y);
        swc1(sp + 8, z);
        let height = lwc1(lw(g::currentplayer) + 132);
        moveTest(lw(sp + 32), at, sp, sp + 16, 0, 1, radius, height);
        moveTest(lw(sp + 32), sp, sp + 16, 0, 0, 1, radius, lwc1(lw(g::currentplayer) + 132));
        let x = lwc1(sp + 16);
        let y = lwc1(sp + 20);
        let z = lwc1(sp + 24);
        swc1(a0 + 112, x);
        swc1(a0 + 116, y);
        swc1(a0 + 120, z);
        swc1(a0 + 48, x);
        swc1(a0 + 52, y);
        swc1(a0 + 56, z);
        let floor = moveFindFloor(at, 0, 0, radius, lwc1(lw(g::currentplayer) + 132));
        swc1(mind + 2956, floor);
        enemySetRoom(a0, 1);
        if lw(g::curgameparms + 72) != 7 {
            do_spawn = ((lw(mind + 0x11ac) as u32) < (1 as u32)) as i32;
        } else {
            let level = lw(g::lvlevelnum);
            if level != 9 {
                if level != 27 {
                    let kind = lw(mind + 8);
                    if ((kind - 1) as u32) >= (2 as u32) {
                        break 'b33;
                    }
                    let space = specialfxIsSpace(3, at, lw(a0 + 12));
                    do_spawn = 1;
                    colour = 2;
                    effect = if space != 0 { 3 } else { 0 };
                    break 'b33;
                }
            }
            do_spawn = 1;
            effect = 1;
            colour = 2;
        }
    }
    let flags;
    if do_spawn == 0 {
        flags = lw(mind + 2704);
    } else {
        if isRoomVisibleByAnyPlayer(lw(a0 + 12)) != 0 {
            let mut chosen = effect;
            if effect == -1 {
                chosen = specialfxGetBestSpawnType(lw(sp + 32), at, lw(a0 + 12));
            }
            let made = specialfxNewSpawn(chosen, at, colour, mind, 0);
            sw(mind + 0x11a4, made);
        }
        flags = lw(mind + 2704);
    }
    sw(mind + 0x11ac, 0);
    sw(mind + 2704, flags & -0x3001);
}

/// Record the nearest hit of one of the tests of `moveTest`: when the swept fraction `fraction` (the float at `fraction_at`) is smaller than `best_fraction` the normal
/// at `normal_at` is kept in `sp+64`, and when the distance at `distance_at` is smaller than `best_distance` the normal at `wall_at` is kept in `sp+176`.
fn move_test_keep(sp: i32, distance_at: i32, wall_at: i32, fraction_at: i32, normal_at: i32, best_distance: &mut f32, best_fraction: &mut f32, kind: Option<i32>) {
    let distance = lwc1(distance_at);
    let wall_x = lwc1(wall_at);
    if distance < *best_distance {
        *best_distance = distance;
        let wall_z = lwc1(wall_at + 4);
        swc1(sp + 176, wall_x);
        swc1(sp + 180, wall_z);
    }
    let fraction = lwc1(fraction_at);
    let normal_x = lwc1(normal_at);
    if fraction < *best_fraction {
        *best_fraction = fraction;
        let normal_z = lwc1(normal_at + 4);
        if let Some(found) = kind {
            sw(sp + 344, found);
        }
        swc1(sp + 64, normal_x);
        swc1(sp + 68, normal_z);
    }
}

/// Move the character number `a0` from `a1` to `a2` (@ 0x27d0c0) against the walls, floors and props of the rooms it crosses, `a3` getting the place it ends at (or, without it,
/// `a2` is changed), `a4` the floor item under it, and `a5` allowing the flags of the character to be changed. `f12` is the height and `f13` the radius. Returns the
/// bits of what was hit: 1 a floor, 2 a hit, 4 a wall in the way, 8 a prop in the way.
pub fn moveTest(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, f12: f32, f13: f32) -> i32 {
    let frame = Frame::new(560);
    let sp = frame.sp();
    sw(sp + 312, a0);
    let mut nearest_prop = f13;
    let start_y = lwc1(a1 + 4);
    let end_y = lwc1(a2 + 4);
    let record = lw(g::chrinfo) + a0 * 0x1210;
    sw(sp + 336, record);
    sw(sp + 316, a3);
    let mut nearest_wall = 1.0_f32;
    let on_floor = lw(record + 0x1194);
    sw(sp + 320, a4);
    sw(sp + 324, a5);
    sw(sp + 332, 0);
    sw(sp + 344, -1);
    sw(sp + 340, 1);
    let end_x = lwc1(a2);
    let start_x = lwc1(a1);
    let end_z = lwc1(a2 + 8);
    let start_z = lwc1(a1 + 8);
    if on_floor == 0 && end_x == start_x && end_y == start_y && end_z == start_z {
        sw(sp + 340, 0);
    }
    swc1(sp + 16, end_x - start_x);
    swc1(sp + 20, end_z - start_z);
    let (low_x, high_x) = if !(start_x < end_x) { (end_x - f13, start_x + f13) } else { (start_x - f13, end_x + f13) };
    swc1(sp + 32, low_x);
    swc1(sp + 44, high_x);
    swc1(sp + 36, start_y);
    swc1(sp + 48, start_y + f12);
    let (low_z, high_z) = if !(start_z < end_z) { (end_z - f13, start_z + f13) } else { (start_z - f13, end_z + f13) };
    swc1(sp + 40, low_z);
    swc1(sp + 52, high_z);
    sw(sp + 348, sp + 80);
    if lw(sp + 340) != 0 {
        let rooms = bgWallBBIntersection(sp + 32, sp + 80, 20);
        sw(sp + 328, rooms);
        if rooms > 0 {
            let mut n = 0;
            loop {
                let room_at = (n << 2) + sp + 80;
                let walls = bgGetWalls(lw(room_at));
                let mask = lhu(lw(lw((lw(room_at) << 2) + lw(g::roomprops)) + 32) + 292);
                sw(sp, sp + 208);
                MoveTestWall(walls, mask, sp + 32, a1, sp + 16, sp + 288, sp + 192, sp + 292, f13, f12);
                move_test_keep(sp, sp + 292, sp + 208, sp + 288, sp + 192, &mut nearest_prop, &mut nearest_wall, None);
                n += 1;
                if n >= lw(sp + 328) {
                    break;
                }
            }
        }
    }
    let list = lw(sp + 348);
    let floors = bgFloorBBIntersectionXZ(sp + 32, list, 20);
    sw(sp + 328, floors);
    if floors > 0 {
        let mut n = 0;
        let mut moving = lw(sp + 340);
        loop {
            let list = lw(sp + 348);
            let offset = n << 2;
            if moving != 0 {
                sw(sp, sp + 240);
                sw(sp + 8, 0);
                let found = propMoveTestRoom(lw(sp + 312), lw(list + offset), sp + 32, a1, sp + 16, sp + 296, sp + 224, sp + 300, f13, f12);
                move_test_keep(sp, sp + 300, sp + 240, sp + 296, sp + 224, &mut nearest_prop, &mut nearest_wall, Some(found));
            }
            sw(sp, sp + 240);
            sw(sp + 8, 1);
            let found = propMoveTestRoom(lw(sp + 312), lw(lw(sp + 348) + offset), sp + 32, a1, sp + 16, sp + 296, sp + 224, sp + 300, f13, f12);
            move_test_keep(sp, sp + 300, sp + 240, sp + 296, sp + 224, &mut nearest_prop, &mut nearest_wall, Some(found));
            n += 1;
            moving = lw(sp + 340);
            if n >= lw(sp + 328) {
                break;
            }
        }
    }
    let one = 1.0_f32;
    let flags_after;
    'result: {
        if !(nearest_wall < one) {
            if !(nearest_prop < f13) {
                let player_flag = lw(sp + 324);
                let record = lw(sp + 336);
                if player_flag != 0 {
                    sw(record + 0x1194, 0);
                }
                let out = lw(sp + 316);
                flags_after = lw(sp + 340);
                if out == 0 {
                    break 'result;
                }
                swc1(out, lwc1(a2));
                swc1(out + 4, lwc1(a2 + 4));
                swc1(out + 8, lwc1(a2 + 8));
                break 'result;
            }
        }
        let player_flag = lw(sp + 324);
        sw(sp + 332, 2);
        if player_flag != 0 {
            sw(lw(sp + 336) + 0x1194, 1);
        }
        if nearest_wall < one {
            sw(sp + 332, 6);
        }
        let hit = lw(sp + 332);
        if nearest_prop < f13 {
            sw(sp + 332, hit | 8);
        }
        let hit = lw(sp + 332);
        if lw(sp + 316) == 0 {
            'fixed: {
                let mut bits = lw(sp + 332);
                if (lw(sp + 332) & 12) == 12 {
                    if (lwc1(sp + 64) * lwc1(sp + 176) + lwc1(sp + 68) * lwc1(sp + 180)) <= 0.0_f32 {
                        swc1(a2, lwc1(a1));
                        swc1(a2 + 4, lwc1(a1 + 4));
                        swc1(a2 + 8, lwc1(a1 + 8));
                        break 'fixed;
                    }
                    bits = lw(sp + 332);
                }
                let step_x = lwc1(sp + 16);
                let step_z;
                let along;
                if (bits & 8) == 0 {
                    step_z = lwc1(sp + 20);
                    along = nearest_wall * step_x;
                } else {
                    step_z = lwc1(sp + 20);
                    if (step_x * lwc1(sp + 176) + step_z * lwc1(sp + 180)) < 0.0_f32 {
                        swc1(a2, lwc1(a1));
                        swc1(a2 + 4, lwc1(a1 + 4));
                        swc1(a2 + 8, lwc1(a1 + 8));
                        break 'fixed;
                    }
                    along = nearest_wall * step_x;
                }
                swc1(a2, lwc1(a1) + along);
                swc1(a2 + 4, lwc1(a1 + 4));
                swc1(a2 + 8, lwc1(a1 + 8) + nearest_wall * step_z);
            }
        } else {
            let sides = hit & 12;
            let mut slide = 1;
            if sides == 8 {
                let wall_x = lwc1(sp + 176);
                let wall_z = lwc1(sp + 180);
                if (lwc1(sp + 16) * wall_x + lwc1(sp + 20) * wall_z) < 0.0_f32 {
                    swc1(sp + 64, wall_x);
                    nearest_wall = 0.0_f32;
                    swc1(sp + 68, wall_z);
                }
            }
            let normal_x = lwc1(sp + 64);
            if sides == 0 {
                swc1(sp + 256, 0.0_f32);
                swc1(sp + 260, 0.0_f32);
            } else {
                let normal_z = lwc1(sp + 68);
                let negative = -normal_x;
                let step_z = lwc1(sp + 20);
                let product = lwc1(sp + 16) * normal_z;
                swc1(sp + 256, normal_z);
                let mut dot = product + step_z * negative;
                swc1(sp + 260, negative);
                if dot < 0.0_f32 {
                    swc1(sp + 260, normal_x);
                    dot = -dot;
                    swc1(sp + 256, -normal_z);
                }
                let ex = lwc1(sp + 256);
                let ez = lwc1(sp + 260);
                let size = ex * ex + ez * ez;
                if !(0.0_f32 < size) {
                    swc1(sp + 256, 0.0_f32);
                    swc1(sp + 260, 0.0_f32);
                } else {
                    let scale = (1.0_f32 - nearest_wall) * dot / size;
                    swc1(sp + 260, ez * scale);
                    swc1(sp + 256, ex * scale);
                }
            }
            'corner: {
                let bits;
                if sides != 12 {
                    bits = lw(sp + 332);
                } else {
                    let wall_x = lwc1(sp + 176);
                    let wall_z = lwc1(sp + 180);
                    let slide_x = lwc1(sp + 256);
                    if (lwc1(sp + 64) * wall_x + lwc1(sp + 68) * wall_z) <= 0.0_f32 {
                        let out = lw(sp + 316);
                        if !((slide_x * wall_x + lwc1(sp + 260) * wall_z) <= 0.0_f32) {
                            swc1(sp + 272, 0.0_f32);
                            swc1(sp + 276, 0.0_f32);
                            break 'corner;
                        }
                        let x = lwc1(a1);
                        slide = 0;
                        swc1(out, x);
                        swc1(a2, x);
                        let y = lwc1(a1 + 4);
                        swc1(out + 4, y);
                        swc1(a2 + 4, y);
                        let z = lwc1(a1 + 8);
                        swc1(out + 8, z);
                        swc1(a2 + 8, z);
                        break 'corner;
                    }
                    bits = lw(sp + 332);
                }
                let frames = lwc1(g::updateframesf);
                if (bits & 8) == 0 {
                    swc1(sp + 272, 0.0_f32);
                    swc1(sp + 276, 0.0_f32);
                } else {
                    let wall_x = lwc1(sp + 176);
                    let wall_z = lwc1(sp + 180);
                    let towards = (lwc1(sp + 16) * wall_x + lwc1(sp + 20) * wall_z) < 0.0_f32;
                    let push = (f13 - nearest_prop) / f13 * 4.0_f32 * frames / 6e+01_f32 + lwc1(0x3acfa8);
                    swc1(sp + 276, push * wall_z);
                    swc1(sp + 272, push * wall_x);
                    if towards {
                        nearest_wall = 0.0_f32;
                    }
                }
            }
            let bits = lw(sp + 332);
            if slide != 0 {
                let step_x = lwc1(sp + 16);
                let along_z;
                if (bits & 4) == 0 {
                    let push_z = lwc1(sp + 276);
                    let step = nearest_wall * lwc1(sp + 20);
                    swc1(a2, lwc1(a1) + nearest_wall * step_x + lwc1(sp + 272));
                    swc1(a2 + 4, lwc1(a1 + 4));
                    along_z = lwc1(a1 + 8) + step + push_z;
                } else {
                    let factor = lwc1(0x3acfac);
                    let push_z = lwc1(sp + 276);
                    let step = nearest_wall * lwc1(sp + 20);
                    let wall = lwc1(sp + 68) * factor;
                    swc1(a2, lwc1(a1) + nearest_wall * step_x + lwc1(sp + 64) * factor + lwc1(sp + 272));
                    swc1(a2 + 4, lwc1(a1 + 4));
                    along_z = lwc1(a1 + 8) + step + wall + push_z;
                }
                swc1(a2 + 8, along_z);
                moveTest(lw(sp + 312), a1, a2, 0, 0, 0, f12, f13);
                let out = lw(sp + 316);
                let slide_z = lwc1(sp + 260);
                swc1(out, lwc1(a2) + lwc1(sp + 256));
                swc1(out + 4, lwc1(a2 + 4));
                swc1(out + 8, lwc1(a2 + 8) + slide_z);
            }
        }
        flags_after = lw(sp + 340);
    }
    'glass: {
        let player_flag = lw(sp + 324);
        if flags_after != 0 {
            let rooms = lw(sp + 328);
            if player_flag != 0 {
                let mut at = lw(sp + 348);
                if rooms > 0 {
                    let mut left = rooms;
                    loop {
                        bgGlassMoveTestRoom(lw(sp + 312), lw(at), a1, a2, f12, f13);
                        left -= 1;
                        at += 4;
                        if left == 0 {
                            break 'glass;
                        }
                    }
                }
            }
        }
    }
    let out = lw(sp + 316);
    let end = if out == 0 { a2 } else { out };
    let same_x = lwc1(end) == lwc1(a1);
    sw(sp + 340, 1);
    if same_x {
        if lwc1(end + 8) == lwc1(a1 + 8) {
            sw(sp + 340, 0);
        }
    }
    'b37: {
        let hit_bits;
        'b38: {
            if lw(sp + 340) != 0 {
                swc1(end + 4, start_y);
            } else {
                let record = lw(sp + 336);
                let floor = lwc1(record + 0x1198);
                if floor <= lwc1(a1 + 4) {
                    swc1(end + 4, floor);
                    let item = lw(sp + 320);
                    if lw(sp + 320) != 0 {
                        sw(item, lw(record + 0x119c));
                    }
                    if lwc1(end + 4) < end_y {
                        swc1(end + 4, end_y);
                        break 'b37;
                    }
                    hit_bits = lw(sp + 332);
                    break 'b38;
                }
                swc1(end + 4, start_y);
            }
            let floor = moveFindFloor(end, sp + 304, sp + 308, lwc1(0x3acfb0), f13);
            swc1(end + 4, floor);
            let found = lw(sp + 304);
            if lw(sp + 324) != 0 {
                let other = lw(sp + 308);
                let record = lw(sp + 336);
                swc1(record + 0x1198, floor);
                sw(record + 0x119c, found);
                sw(record + 0x11a0, other);
            }
            let found = lw(sp + 304);
            if lw(sp + 320) != 0 {
                sw(lw(sp + 320), found);
            }
            hit_bits = lw(sp + 332);
            if lwc1(end + 4) < end_y {
                swc1(end + 4, end_y);
                break 'b37;
            }
        }
        sw(sp + 332, hit_bits | 1);
    }
    let wall_z = lwc1(sp + 68);
    let prop = lw(sp + 344);
    swc1(0x3b0118, lwc1(sp + 64));
    swc1(0x3b011c, wall_z);
    let prop_now = lw(sp + 344);
    if prop < 0 {
        sw(lw(sp + 336) + 0x11b8, 0);
    } else {
        let record = lw(sp + 336);
        sw(record + 0x11bc, prop_now);
        sw(record + 0x11b8, 1);
    }
    lw(sp + 332)
}
