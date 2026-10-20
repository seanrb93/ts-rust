//! The special particles drawn as quads, and the fading of the characters that spawn.

use crate::*;

/// Draw the special particles `points` (a list of `count` points of three floats, with their colours in `colours`): each point is put on the screen by the player's matrices and,
/// when it is inside the view, drawn as a square of the size 500 over the distance (scaled by the player's zoom) as a packet of 176 bytes; `a3` and `a4` are the texture
/// coordinates' shifts (@ 0x2a8b48).
pub fn specialParticlesGfx(points: i32, colours: i32, count: i32, shift_a: i32, shift_b: i32) {
    let frame = Frame::new(320);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    let mut i = 0;
    let zoom = 6e+01_f32 / lwc1(player + 260);
    sceVu0MulMatrix(sp, lw(player + 1760), lw(player + 1764));
    if count <= 0 {
        return;
    }
    loop {
        matrixVec3Mul4(sp, i * 12 + points, sp + 80);
        let w = lwc1(sp + 92);
        let x = lwc1(sp + 80);
        let low = -w;
        let y = lwc1(sp + 84);
        let z = lwc1(sp + 88);
        let outside = x < low || w < x || y < low || w < y || z < low || w < z;
        let window = g::window;
        let across = lw(window + 16) as f32;
        let inverse = 1.0_f32 / w;
        let scale = lwc1(0x3ad644);
        let centre_x = lw(window + 8) as f32;
        let centre_y = lw(window + 28) as f32;
        let down = (0 - lw(window + 36)) as f32;
        let screen_x = x * inverse;
        let screen_y = y * inverse;
        let size = (zoom * 5e+02_f32 / z) as i32;
        let depth = z * inverse;
        swc1(sp + 80, screen_x);
        swc1(sp + 84, screen_y);
        swc1(sp + 88, depth);
        let px = centre_x + across * screen_x;
        let py = centre_y + down * screen_y;
        let half = ((size + ((size as u32) >> 31) as i32) >> 1) as f32;
        let full = size as f32;
        swc1(sp + 64, px);
        swc1(sp + 68, py);
        let z_out = depth * scale + 32768.0_f32;
        let bottom = py + half;
        let right = px + full;
        let left = px - full;
        swc1(sp + 152, z_out);
        let top = py - half;
        swc1(sp + 148, bottom);
        swc1(sp + 144, right);
        swc1(sp + 128, left);
        swc1(sp + 116, top);
        swc1(sp + 72, z_out);
        swc1(sp + 96, left);
        swc1(sp + 100, top);
        swc1(sp + 104, z_out);
        swc1(sp + 112, right);
        swc1(sp + 120, z_out);
        swc1(sp + 132, bottom);
        swc1(sp + 136, z_out);
        if !outside {
            let packet = memdbAlloc(176);
            sb(lw(g::gfxtag) + 3, 48);
            sw(packet, 0x6c0a8000);
            let tag = lw(g::gfxtag);
            sw(tag + 4, packet);
            let corner = |x_at: i32, y_at: i32| -> i32 {
                (((lwc1(x_at) as i32) << 4) + 0x6c00 & 0xffff) | (((lwc1(y_at) as i32) << 4) + 0x7900) << 16
            };
            let high = shift_b << 20;
            let low_shift = shift_a << 4;
            let words: [i32; 41] = [
                0x8001,
                -0x6f51c000,
                -0x11111112,
                14,
                lw((i << 2) + colours),
                0x3f800000,
                1,
                0,
                0,
                0,
                3,
                0,
                corner(sp + 96, sp + 100),
                lwc1(sp + 104) as i32,
                5,
                0,
                high,
                0,
                3,
                0,
                corner(sp + 128, sp + 132),
                lwc1(sp + 120) as i32,
                5,
                0,
                low_shift,
                0,
                3,
                0,
                corner(sp + 112, sp + 116),
                lwc1(sp + 136) as i32,
                5,
                0,
                low_shift | high,
                0,
                3,
                0,
                corner(sp + 144, sp + 148),
                lwc1(sp + 152) as i32,
                5,
                0,
                0x140007fc,
            ];
            let mut put = packet + 4;
            for word in words {
                sw(put, word);
                put += 4;
            }
            sw(put, 0x11000000);
            sw(put + 4, 0);
            sw(g::gfxtag, tag + 16);
            sh(tag, 11);
        }
        i += 1;
        if i >= count {
            break;
        }
    }
}

/// Fade the characters that spawn: for each of the 14 spawn groups (records of 32 bytes from `spawngroups`, the kind at +0, the state at +4, a flag at +8, the character at +12, the
/// time at +16 and a flag at +24) the time goes on by the frames over 60; while it is under the fade-out time of the kind the character fades out (its override states are set with
/// the opacity falling), then while it is under the fade-in time it fades in; once the locked time of the kind (halved in modes 6 and 7) is past, the character's override states
/// are cleared and the group's effect ended (a kind 5 character deleted, a kind 6 after 4) (@ 0x2abea0).
pub fn spawnfxTick() {
    let step = lwc1(g::updateframesf) / 6e+01_f32;
    let mut i = 0;
    let mut offset = i << 5;
    loop {
        'body: {
            let groups = g::spawngroups;
            let kind = lw(offset + groups);
            let group = offset;
            if kind == -1 {
                break 'body;
            }
            let mut fade_out = lwc1((kind << 2) + g::spawnfadeouttime);
            if !(lw(groups + offset + 24) == 0 && lw(g::lvnumplayers) < 3) {
                fade_out *= lwc1(0x3ad6d0);
            }
            let groups = g::spawngroups;
            let fade_in;
            if !(lw(groups + group + 24) == 0 && lw(g::lvnumplayers) < 3) && lw(offset + groups) == 6 {
                fade_in = lwc1(g::spawnfadeintime + 24) * lwc1(0x3ad6d4);
            } else {
                fade_in = lwc1((lw(group + groups) << 2) + g::spawnfadeintime);
            }
            let groups = g::spawngroups;
            let record = group + groups;
            let mut locked = lwc1((lw(record) << 2) + g::spawnlockedtime);
            if ((lw(g::curgameparms + 72) - 6) as u32) < 2 {
                locked *= 0.5_f32;
            }
            let time_at = offset + (groups + 16);
            let flag_at = offset + (groups + 8);
            let time = lwc1(time_at);
            let still_locked = (time < locked) as i32;
            let now = time + step;
            sw(flag_at, still_locked);
            swc1(time_at, now);
            if now < fade_out {
                let rec = groups + offset;
                let object = lw(rec + 12);
                sw(rec + 4, 0);
                if object != 0 {
                    let model = lw(object + 32);
                    if model == 0 {
                        break 'body;
                    }
                    let ratio = now / fade_out;
                    let mut mode = -1;
                    if ratio <= 1.0_f32 {
                        let opacity = (127.0_f32 - ratio * 127.0_f32) as i32 & 255;
                        let rec = groups + offset;
                        if (opacity as u32) < 44 {
                            mode = if lw(rec + 24) != 0 { 2 } else { mode };
                        }
                        obInstSetOverrideStates(model, 3, -1, mode, opacity);
                    }
                }
                break 'body;
            }
            let groups_now = g::spawngroups;
            if 0.0_f32 <= now && now < fade_in {
                let rec = groups + offset;
                let object = lw(rec + 12);
                sw(rec + 4, 1);
                if object == 0 {
                    break 'body;
                }
                if lw(object + 32) == 0 {
                    break 'body;
                }
                let kind = lw(record);
                let ratio = now / fade_in;
                let mut mode = -1;
                let value = if (kind as u32) >= 5 {
                    ratio * ratio * ratio * 127.0_f32
                } else {
                    match kind {
                        0 | 3 | 4 => ratio * 97.0_f32 + 3e+01_f32,
                        _ => ratio * ratio * 127.0_f32,
                    }
                };
                let opacity = value as i32 & 255;
                if (opacity as u32) < 44 {
                    mode = if lw(group + (g::spawngroups + 24)) != 0 { 2 } else { mode };
                }
                let object_at = group + (g::spawngroups + 12);
                obInstSetOverrideStates(lw(lw(object_at) + 32), 3, -1, mode, opacity);
                let second = lw(lw(object_at) + 536);
                if second == 0 {
                    break 'body;
                }
                obInstSetOverrideStates(lw(second + 32), 3, -1, -1, opacity);
                break 'body;
            }
            let time_now = lwc1(groups_now + group + 16);
            if !(fade_in <= time_now) {
                break 'body;
            }
            if !(fade_out <= time_now) {
                break 'body;
            }
            let rec = groups_now + offset;
            if locked <= time_now {
                let object = lw(rec + 12);
                sw(rec + 4, 2);
                if object != 0 {
                    let model = lw(object + 32);
                    if model != 0 {
                        obInstSetOverrideStates(model, -1, -1, -1, 0);
                    }
                }
                let groups = g::spawngroups;
                let object_at = group + (groups + 12);
                let second = lw(lw(object_at) + 536);
                if second != 0 {
                    obInstSetOverrideStates(lw(second + 32), -1, -1, -1, 0);
                }
                let spawn = offset + groups;
                let kind = lw(spawn);
                if kind == 5 {
                    let object = lw(object_at);
                    let mut deleted = false;
                    if object != 0 && lw(object + 8) == 0x1000 {
                        propDeleteCorpse(object);
                        sw(object_at, 0);
                        deleted = true;
                    }
                    if !deleted {
                        propDelete(lw(g::spawngroups + group + 12));
                    }
                    let groups = g::spawngroups;
                    let spawn = group + groups;
                    sw(groups + group + 12, 0);
                    specialfxFreeSpawn(spawn);
                    break 'body;
                }
                if kind != 6 {
                    break 'body;
                }
                if !(4.0_f32 < lwc1(groups + offset + 16)) {
                    break 'body;
                }
                specialfxFreeSpawn(spawn);
            }
        }
        i += 1;
        offset = i << 5;
        if i >= 14 {
            break;
        }
    }
}
