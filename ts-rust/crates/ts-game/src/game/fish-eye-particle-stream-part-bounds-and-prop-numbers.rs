//! The fish-eye screen effect, the drawing of a stream of particles, the bounds of the polygons of a model part, and the number of the prop for a place in a level.

use crate::*;

/// Draw the fish-eye effect over the window with the screen copied as a texture: for each line of the window (strength `strength` and ratio `ratio`) a line is drawn with its ends
/// pulled in by the square root of a sine of its height, and then for each column the same with the top and bottom pulled in by the sine of an arc cosine (@ 0x2962d0).
pub fn ipFishEye(strength: f32, ratio: f32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut pull = (1.0_f32 - strength) * lwc1(0x3ad478) + lwc1(0x3ad47c);
    if lw(g::lvnumplayers) >= 3 {
        pull *= 0.5_f32;
    }
    ipSetCurScreenAsTexture();
    dlSetBlend(0);
    dlSetDitherMatrix(0);
    let mut y = lw(g::window + 20);
    if y < lw(g::window + 24) + 1 {
        let window = g::window;
        loop {
            let angle = ((y as f32 * 1.8e+02_f32).pdiv(lw(window + 32) as f32) * lwc1(0x3ad480)).pdiv(1.8e+02_f32);
            let wave = sinf(angle);
            let right = lw(window + 4);
            let bottom = lw(window + 24);
            let next = y + 1;
            let left = lw(g::window);
            let right_edge = right + 1;
            let shift = (wave.abs().psqrt() * pull) as i32;
            sw(sp, 0x7f7f7f7f);
            dlTextureRectangle(left, bottom, right_edge, bottom + 1, left << 4, y << 4, right_edge << 4, next << 4);
            let bottom = lw(window + 24);
            let left = lw(g::window);
            let right_edge = lw(window + 4) + 1;
            sw(sp, 0x7f7f7f7f);
            dlTextureRectangle(left, y, right_edge, next, shift + (left << 4), bottom << 4, (right_edge << 4) - shift, (bottom + 1) << 4);
            y = next;
            if y >= lw(window + 24) + 1 {
                break;
            }
        }
    }
    let mut x = lw(g::window);
    if x < lw(g::window + 4) + 1 {
        let window = g::window;
        let mut first = lw(g::window);
        loop {
            let width = lw(window + 16);
            let across = fptodp((first + width - x) as f32);
            let mut distance = across;
            if dpcmp(distance, 0_i64) < 0 {
                distance = dpsub(0_i64, distance);
            }
            let next = x + 1;
            let ratio_now = dptofp(dpmul(dpdiv(distance, fptodp(width as f32)), fptodp(ratio)));
            let middle = lw(window + 36) as f32;
            let wave = sinf(acosf(ratio_now));
            let right = lw(window + 4);
            let top = lw(window + 20);
            let bottom = lw(window + 24) + 1;
            let shift = (middle * wave) as i32;
            sw(sp, 0x7f7f7f7f);
            dlTextureRectangle(right, top, right + 1, bottom, x << 4, top << 4, next << 4, bottom << 4);
            let right = lw(window + 4);
            let centre = lw(window + 36);
            sw(sp, 0x7f7f7f7f);
            dlTextureRectangle(x, centre - shift, next, centre + shift, right << 4, lw(window + 20) << 4, (right + 1) << 4, (lw(window + 24) + 1) << 4);
            x = next;
            first = lw(g::window);
            if x >= lw(window + 4) + 1 {
                break;
            }
        }
    }
}

/// Draw the particles of the stream group `group` (kind at +4, the count at +24, the age of each at +3332, the places from +932, spins at +3732) with the transformation
/// `matrix`: each live particle (age below the kind's life at +12) has its place worked out from its start and speed, moved into the screen by the matrix and, when it is inside
/// the clip volume, drawn as a quad (two sines of its spin for the corners, its size from the age, its alpha fading in and out) by `particleRender` (@ 0x2a02d0).
pub fn particleStreamGfx(group: i32, matrix: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let mut i = 0;
    let rate = (6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260));
    let info = lw(group + 4) * 104 + g::particleinfos;
    dlSelectTextureKick(lw((lw(info) << 4) + (g::particletextures + 12)));
    dlSetDitherMatrix(2);
    let mut count = lw(group + 24);
    if count > 0 {
        loop {
            let at4 = i << 2;
            let age = lwc1(group + 12) - lwc1(group + 3332 + at4);
            if !(0.0_f32 <= age) {
                i += 1;
            } else {
                let life = lwc1(info + 12);
                let spin_at = group + at4;
                if age < life {
                    let fraction = (age).pdiv(life);
                    let fade_in = lwc1(info + 24);
                    let mut alpha = 1.0_f32;
                    let spin = (lwc1(spin_at + 3732) * age * lwc1(0x3ad574)).pdiv(1.8e+02_f32);
                    let colour = lw(info + 32);
                    if !(age < fade_in) {
                        let fade_out = lwc1(info + 28);
                        if (life - fade_out) < age {
                            alpha = ((life - age)).pdiv(fade_out);
                        }
                    } else {
                        alpha = (age).pdiv(fade_in);
                    }
                    let place = group + i * 12;
                    let x = lwc1(place + 932) + lwc1(info + 56) * age;
                    let y = lwc1(place + 936) + lwc1(info + 60) * age;
                    swc1(sp + 8, lwc1(place + 940) + lwc1(info + 64) * age);
                    swc1(sp, x);
                    swc1(sp + 4, y);
                    matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 16) as i64]);
                    let w = lwc1(sp + 28);
                    let sx = lwc1(sp + 16);
                    let minus_w = -w;
                    if !(minus_w <= sx) {
                        count = lw(group + 24);
                    } else {
                        let sy = lwc1(sp + 20);
                        let mut inside = false;
                        if sx <= w {
                            if !(minus_w <= sy) {
                                count = lw(group + 24);
                                i += 1;
                                if i >= count {
                                    break;
                                }
                                continue;
                            }
                            let sz = lwc1(sp + 24);
                            if sy <= w {
                                if !(minus_w <= sz) {
                                    count = lw(group + 24);
                                    i += 1;
                                    if i >= count {
                                        break;
                                    }
                                    continue;
                                }
                                if sz <= w {
                                    inside = true;
                                }
                            }
                        }
                        if inside {
                            let sz = lwc1(sp + 24);
                            let screen = g::matchstats + 6488 - 0x6610;
                            let one = 1.0_f32;
                            let inverse = (one).pdiv(w);
                            let half = 0.5_f32;
                            let depth = sz * inverse;
                            let table = g::g_SinTable;
                            let depth_scale = lwc1(0x3ad57c);
                            let angle = (spin * lwc1(0x3ad578) + half) as i32;
                            let ndc_x = sx * inverse;
                            let ndc_y = sy * inverse;
                            let size = (lwc1(info + 40)).pdiv(depth) * rate;
                            let sine = lwc1(((angle & 2047) << 2) + table);
                            let cosine = lwc1(((angle + 512 & 2047) << 2) + table);
                            let px = lw(screen + 16) as f32 * ndc_x;
                            let particle_texture = (lw(info) << 4) + g::particletextures;
                            let name = lw(particle_texture + 4);
                            let py = (0 - lw(screen + 36)) as f32 * ndc_y;
                            let half_w = (size * (fraction * lwc1(info + 48) + one)) as i32 as f32;
                            let name2 = lw(particle_texture + 8);
                            let half_h = (size * (fraction * lwc1(info + 44) + one)) as i32 as f32;
                            swc1(sp + 20, ndc_y);
                            swc1(sp + 24, depth);
                            swc1(sp + 16, ndc_x);
                            let w_sin = half_w * sine;
                            let h_cos = half_h * cosine;
                            let h_half = h_cos * half;
                            let cx = lw(screen + 8) as f32 + px;
                            let cy = lw(screen + 28) as f32 + py;
                            let w_half = w_sin * half;
                            swc1(sp, cx);
                            swc1(sp + 4, cy);
                            let colour_alpha = colour | ((alpha * 4e+01_f32) as i32) << 24;
                            swc1(sp + 80, cx - h_cos);
                            swc1(sp + 84, cy - w_half);
                            swc1(sp + 64, cx - w_sin);
                            let z = depth * depth_scale + 32768.0_f32 + 1e+02_f32;
                            swc1(sp + 68, cy + h_half);
                            swc1(sp + 32, cx + h_cos);
                            swc1(sp + 36, cy + w_half);
                            swc1(sp + 88, z);
                            swc1(sp + 48, cx + w_sin);
                            swc1(sp + 52, cy - h_half);
                            swc1(sp + 8, z);
                            swc1(sp + 40, z);
                            swc1(sp + 56, z);
                            swc1(sp + 72, z);
                            particleRender(name, name2, colour_alpha, sp + 32, sp + 48, sp + 64, sp + 80);
                        }
                        count = lw(group + 24);
                    }
                }
                i += 1;
            }
            if i >= count {
                break;
            }
        }
    }
}

/// Work out the bounds of the polygons of the part `part` (the chunk list at +8 for the kind 0 and at +12 for the others, ended by a chunk with a negative flag at +20; the
/// points at +20 or +44): memory for the bounds (36 bytes a chunk) is kept at +76 (the game exits without it); each chunk's bounds come from `obCalcChunkBounds`, and the polygons of
/// the chunks are merged in three passes (`obMergeChunkPolys`: count, then the list of 2-byte entries, then the merged data) (@ 0x25e640).
pub fn obCalcPartBounds(part: i32, kind: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let chunks;
    let points;
    if kind == 0 {
        chunks = lw(part + 8);
        points = part + 20;
    } else {
        chunks = lw(part + 12);
        points = part + 44;
    }
    let mut count = 0;
    let mut chunk = chunks;
    while chunk != 0 {
        if lw(chunk + 20) < 0 {
            break;
        }
        count += 1;
        chunk += 24;
    }
    if count == 0 {
        sw(part + 76, 0);
        return;
    }
    let memory = memAlloc(count * 36 + 15 & -16, 1);
    if memory != 0 {
        sw(part + 76, memory);
    } else {
        exit(1);
    }
    let mut i = 0;
    if count > 0 {
        loop {
            let offset = i * 36;
            i += 1;
            sw(offset + memory + 24, kind);
            if i >= count {
                break;
            }
        }
    }
    let mut chunk = chunks;
    let mut bounds = memory;
    let mut memory = memory;
    if lw(chunk + 20) >= 0 {
        loop {
            let this = chunk;
            let out = bounds;
            chunk += 24;
            obCalcChunkBounds(points, this, out);
            bounds += 36;
            if lw(chunk + 20) < 0 {
                break;
            }
        }
        memory = lw(part + 76);
    }
    let mut left = count;
    if count > 0 {
        let mut at = memory + 28;
        loop {
            sw(at, -1);
            left -= 1;
            at += 36;
            if left == 0 {
                break;
            }
        }
    }
    sw(sp, 0);
    let mut chunk = chunks;
    sw(sp + 12, 0);
    sw(sp + 20, 0);
    let mut bounds = memory;
    if lw(chunk + 20) >= 0 {
        loop {
            obMergeChunkPolys(sp, points, chunk, bounds);
            chunk += 24;
            bounds += 36;
            if lw(chunk + 20) < 0 {
                break;
            }
        }
    }
    let size = lw(sp + 12) << 1;
    sw(sp, 1);
    let mut chunk = chunks;
    let table = memAlloc(size + 15 & -16, 0);
    let flag = lw(chunk + 20);
    let end = table + lw(sp + 12);
    sw(sp + 4, table);
    sw(sp + 8, end);
    let mut bounds = lw(part + 76);
    if flag >= 0 {
        loop {
            obMergeChunkPolys(sp, points, chunk, bounds);
            chunk += 24;
            bounds += 36;
            if lw(chunk + 20) < 0 {
                break;
            }
        }
    }
    let data_size = lw(sp + 20);
    sw(sp, 2);
    let data = memAlloc(data_size, 1);
    let mut chunk = chunks;
    sw(sp + 20, 0);
    sw(sp + 16, data);
    let mut bounds = lw(part + 76);
    if lw(chunk + 20) >= 0 {
        loop {
            obMergeChunkPolys(sp, points, chunk, bounds);
            chunk += 24;
            bounds += 36;
            if lw(chunk + 20) < 0 {
                break;
            }
        }
    }
    memFree(lw(sp + 4));
}

/// The number of the prop for the place `id` of a level: a non-negative number is the prop itself; the negative numbers stand for the props of the game mode (-34 to -32 the ones a
/// level made in the map maker has by its game kind, -33 and -32 the base and its team by the level; -31 and -30 353; -25 to -21 themselves; -9 to -6 the props of the players
/// of the game, from 337; -5 to -1 the props of the guns of the level's combinations); 324 in the challenge of kind 7 is 334; no prop at all is the most negative number (@ 0x227c20).
pub fn lvGetPropNum(id: i32) -> i32 {
    let parms = g::curgameparms;
    if (lw(parms + 80) & 32) != 0 && lw(parms + 68) == 7 && id == 324 {
        return 334;
    }
    let level = lw(g::lvlevelnum);
    if id >= 0 {
        return id;
    }
    let mut index = id + 33;
    if level == 100 {
        let info = mmfileGetInfo(lw(g::mmtempbuffer));
        let kind = lhu(info + 8);
        if id == -33 {
            if (kind as u32) < 5 {
                return match kind {
                    0 => 241,
                    1 => 357,
                    2 => 355,
                    _ => 359,
                };
            }
        } else if id >= -32 {
            if id == -32 && (kind as u32) < 5 {
                return match kind {
                    0 => 240,
                    1 => 356,
                    2 => 354,
                    _ => 358,
                };
            }
        } else if id == -34 && (kind as u32) < 5 {
            return match kind {
                0 => 324,
                1 => 333,
                2 => 335,
                3 => 334,
                _ => 337,
            };
        }
        index = id + 33;
    }
    if (index as u32) < 33 {
        match index {
            0 | 1 => {
                let offset = lw(g::lvlevelnum) - 2;
                if (offset as u32) < 26 {
                    match offset {
                        0 | 6 | 14 | 20 | 21 | 24 => return if id == -32 { 354 } else { 355 },
                        3 | 8 | 11 | 13 | 16 | 19 | 22 => return if id == -32 { 356 } else { 357 },
                        _ => {}
                    }
                }
                return if id == -32 { 358 } else { 359 };
            }
            2 | 3 => return 353,
            8..=12 => return id,
            24..=27 => {
                let parms = g::curgameparms;
                let players = lw(parms + 12);
                let team = -5 - id;
                let mut i = 0;
                if players > 0 {
                    if lbu(parms + 16) != team {
                        i += 1;
                        loop {
                            if i >= players {
                                break;
                            }
                            if lbu(i + (parms + 16)) == team {
                                break;
                            }
                            i += 1;
                        }
                    }
                    if i < players {
                        let parms = g::curgameparms;
                        if i < lw(parms + 12) {
                            return team + 337;
                        }
                    }
                }
            }
            28..=32 => {
                let list = lvGetGunComboNums();
                let gun = lh((lw((!id << 2) + list) << 3) + (GUNCOMBOS + 4));
                if gun == -1 {
                    return -0x80000000;
                }
                return lw(gun * 400 + (g::guns + 8));
            }
            _ => {}
        }
    }
    -0x80000000
}
