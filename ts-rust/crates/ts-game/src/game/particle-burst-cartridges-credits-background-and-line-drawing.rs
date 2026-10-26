//! Drawing the particles of a burst, the cartridges that fly out of guns, the tunnel of the credits, and a line in the world.

use crate::*;

/// Add the tag of a display list to the end of the list, pointing at `data` with `count` quadwords (the byte 3 of the tag before it marks it as the end of the packet).
fn push_tag(data: i32, count: i32) {
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    sw(tag + 4, data);
    sw(g::gfxtag, tag + 16);
    sh(tag, count);
}

/// Draw the particles of the burst group `group` (kind at +4, the count at +24, the age at +12, flag 1 at +8: the places are in the group's frame or already the world's, from
/// +928 and +2528 speeds, spins at +3728) with the transformation `matrix`: like `particleStreamGfx` but all the particles have the age of the group (@ 0x29fe20).
pub fn particleBurstGfx(group: i32, matrix: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let mut i = 0;
    let rate = (6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260));
    let info = lw(group + 4) * 104 + g::particleinfos;
    dlSelectTextureKick(lw((lw(info) << 4) + (g::particletextures + 12)));
    dlSetDitherMatrix(2);
    let mut count = lw(group + 24);
    if count > 0 {
        let one = 1.0_f32;
        let window = g::window;
        let half = 0.5_f32;
        let table = g::g_SinTable;
        let mut offset = 0;
        let mut place_at = group + 928;
        loop {
            let age = lwc1(group + 12);
            if !(0.0_f32 <= age) {
                i += 1;
            } else {
                let life = lwc1(info + 12);
                if age < life {
                    let fade_in = lwc1(info + 24);
                    let fraction = (age).pdiv(life);
                    let mut alpha = 1.0_f32;
                    let spin = (lwc1(group + (i << 2) + 3728) * age * lwc1(0x3ad568)).pdiv(1.8e+02_f32);
                    let colour = lw(info + 32);
                    let flags;
                    if !(age < fade_in) {
                        let fade_out = lwc1(info + 28);
                        flags = lw(group + 8);
                        if (life - fade_out) < age {
                            alpha = ((life - age)).pdiv(fade_out);
                        }
                    } else {
                        alpha = (age).pdiv(fade_in);
                        flags = lw(group + 8);
                    }
                    let w;
                    if (flags & 1) == 0 {
                        let place = group + offset;
                        let speed = group + i * 12;
                        let x = lwc1(place + 928) + lwc1(speed + 2528) * age;
                        let y = lwc1(place + 932) + lwc1(speed + 2532) * age;
                        swc1(sp + 8, lwc1(place + 936) + lwc1(speed + 2536) * age);
                        swc1(sp, x);
                        swc1(sp + 4, y);
                        matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 16) as i64]);
                        w = lwc1(sp + 28);
                    } else {
                        matrixVec3Mul4Aligned(&[matrix as i64, place_at as i64, (sp + 16) as i64]);
                        w = lwc1(sp + 28);
                    }
                    let sx = lwc1(sp + 16);
                    let minus_w = -w;
                    let mut go = false;
                    if !(minus_w <= sx) {
                        count = lw(group + 24);
                    } else {
                        let sy = lwc1(sp + 20);
                        let mut skip = false;
                        if sx <= w {
                            if !(minus_w <= sy) {
                                skip = true;
                            } else {
                                let sz = lwc1(sp + 24);
                                if sy <= w {
                                    if !(minus_w <= sz) {
                                        skip = true;
                                    } else if sz <= w {
                                        go = true;
                                    }
                                }
                            }
                        }
                        let _ = skip;
                        count = lw(group + 24);
                    }
                    if go {
                        let sy = lwc1(sp + 20);
                        let sz = lwc1(sp + 24);
                        let inverse = (one).pdiv(w);
                        let particle_texture = lw(info) << 4;
                        let depth = sz * inverse;
                        let depth_scale = lwc1(0x3ad570);
                        let ndc_x = sx * inverse;
                        let ndc_y = sy * inverse;
                        let angle = (spin * lwc1(0x3ad56c) + half) as i32;
                        let screen_h = (0 - lw(window + 36)) as f32;
                        let name = lw(particle_texture + (g::particletextures + 4));
                        let size = (lwc1(info + 40)).pdiv(depth) * rate;
                        let sine = lwc1(((angle & 2047) << 2) + table);
                        let name2 = lw(particle_texture + (g::particletextures + 8));
                        let cosine = lwc1(((angle + 512 & 2047) << 2) + table);
                        let screen_w = lw(window + 16) as f32;
                        let half_h = (size * (fraction * lwc1(info + 44) + one)) as i32;
                        let half_w = (size * (fraction * lwc1(info + 48) + one)) as i32 as f32;
                        let centre_x = lw(window + 8) as f32;
                        swc1(sp + 16, ndc_x);
                        swc1(sp + 20, ndc_y);
                        let w_sin = half_w * sine;
                        swc1(sp + 24, depth);
                        let h_cos = half_h as f32 * cosine;
                        let cx = centre_x + screen_w * ndc_x;
                        let cy = lw(window + 28) as f32 + screen_h * ndc_y;
                        let w_half = w_sin * half;
                        let h_half = h_cos * half;
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
                }
                i += 1;
            }
            offset += 16;
            place_at += 16;
            if i >= count {
                break;
            }
        }
    }
}

/// The tick of the cartridge groups (the groups at `cartridge_groups`, records of 96 bytes for 10 cartridges, the oldest at the index +960 and the count at +964): each cartridge
/// is a prop thrown out of a gun; one that has not started is set going (its animation, its flags); one in flight moves by its speed and falls and turns, slows by the air each
/// tick, and when it reaches the floor it bounces at half speed (or stops, with its speed kept for the next). A cartridge older than 100 ticks is deleted (@ 0x295a00).
pub fn cartridgesTick() {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let groups = g::cartridge_groups;
    let mut group = 0;
    let drag = lwc1(0x3ad46c);
    let half = 0.5_f32;
    let thousand = 1e+03_f32;
    let to_radians = lwc1(0x3ad470);
    let half_turn = 1.8e+02_f32;
    let mut count = lw(groups + 964);
    loop {
        let mut done = 0;
        sw(sp + 64, count);
        let mut head = lw(groups + 960);
        if count <= 0 {
            group += 1;
        } else {
            let next_group = group + 1;
            loop {
                let record = head * 96 + groups;
                let next_head = head + 1;
                let mut prop = lw(record + 64);
                head = if next_head >= 10 { 0 } else { next_head };
                let shown = prop;
                if lw(prop + 12) == 0 {
                    sw(prop + 16, lw(prop + 16) & -2);
                }
                let mut age = lw(record + 80);
                let ticks;
                if age < 100 {
                    let spin_z;
                    'moved: {
                        let flying = lw(prop + 16);
                        let step = lwc1(g::updateframesf);
                        if flying != 0 {
                            let speed_y = lwc1(prop + 64);
                            let y = lwc1(prop + 52) + speed_y * step;
                            let z = lwc1(prop + 56) + lwc1(prop + 68) * step;
                            let roll = lwc1(prop + 72);
                            let gravity = step * lwc1(0x3ad474);
                            swc1(prop + 48, lwc1(prop + 48) + lwc1(prop + 60) * step);
                            swc1(prop + 52, y);
                            swc1(prop + 56, z);
                            let spin = lwc1(prop + 88);
                            ticks = lw(g::updateframesi);
                            swc1(prop + 72, roll + lwc1(record + 68) * step);
                            let turn = lwc1(record + 76);
                            swc1(prop + 64, speed_y - gravity);
                            swc1(prop + 88, spin + turn * step);
                            done += 1;
                            if ticks > 0 {
                                let mut k = 0;
                                loop {
                                    k += 1;
                                    let slow_z = lwc1(shown + 68) * drag;
                                    swc1(shown + 60, lwc1(shown + 60) * drag);
                                    swc1(shown + 68, slow_z);
                                    if k >= ticks {
                                        break;
                                    }
                                }
                            }
                            let height = lwc1(shown + 52);
                            let floor = lwc1(record + 84);
                            if !(height <= floor) {
                                spin_z = lwc1(prop + 88);
                                break 'moved;
                            }
                            let vy = lwc1(prop + 64);
                            let vx = lwc1(prop + 60) * half;
                            let vz = lwc1(prop + 68) * half;
                            swc1(prop + 52, floor);
                            let bounce = vy * -0.5_f32;
                            swc1(prop + 60, vx);
                            swc1(prop + 68, vz);
                            swc1(prop + 64, bounce);
                            if dpcmp(fptodp(bounce), ld(g::cartridge_table + 16)) >= 0 {
                                swc1(record + 68, lwc1(shown + 60) * thousand);
                                swc1(record + 76, lwc1(shown + 68) * thousand);
                            } else {
                                sw(shown + 68, 0);
                                sw(shown + 64, 0);
                                sw(shown + 60, 0);
                                sw(record + 68, 0);
                                sw(record + 72, 0);
                                sw(record + 76, 0);
                            }
                        } else {
                            obInstSetAnim(lw(prop + 32), 1, 0.0_f32);
                            done += 1;
                            prop = lw(record + 64);
                            sw(prop + 16, lw(record + 88) | 0x40000000);
                            age = lw(record + 80);
                            ticks = lw(g::updateframesi);
                        }
                        spin_z = lwc1(prop + 88);
                    }
                    let roll = lwc1(prop + 72);
                    sw(record + 80, age + ticks);
                    matrixRotZXY(sp, (roll * to_radians).pdiv(half_turn), 0.0_f32, (spin_z * to_radians).pdiv(half_turn));
                    sceVu0MulMatrix(lw(lw(lw(record + 64) + 32) + 4), sp, record);
                    let player = lw(g::currentplayer);
                    swc1(lw(lw(lw(record + 64) + 32) + 4) + 48, 0.0_f32);
                    swc1(lw(lw(lw(record + 64) + 32) + 4) + 52, 0.0_f32);
                    swc1(lw(lw(lw(record + 64) + 32) + 4) + 56, 0.0_f32);
                    sw(lw(record + 64) + 12, lw(lw(player + 384) + 12));
                } else {
                    let oldest = lw(groups + 960);
                    sw(groups + 964, lw(groups + 964) - 1);
                    let old_prop = lw(oldest * 96 + groups + 64);
                    let mut following = oldest + 1;
                    if old_prop != 0 {
                        propDelete(old_prop);
                        let oldest = lw(groups + 960);
                        sw(oldest * 96 + groups + 64, 0);
                        following = oldest + 1;
                    }
                    sw(groups + 960, following);
                    done += 1;
                    if following >= 10 {
                        sw(groups + 960, 0);
                    }
                }
                if done >= lw(sp + 64) {
                    break;
                }
            }
            group = next_group;
        }
        if group > 0 {
            break;
        }
        count = lw(groups + 964);
    }
}

/// Draw the tunnel of the credits' background: the camera matrix is made from the camera's place, direction and the roll about z (the first player's matrix at +1760 is multiplied
/// with it); then for each tunnel section (a list from `tunnelDataHead`, each with 3 rings of 12 strips) the strips are drawn with their points, normals and colours as packets
/// in the display list, the first two rings blended, textured by `tunneltexnum` (@ 0x2afb98).
pub fn creditsBgGfx() {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let mut section = lw(g::tunnelDataHead);
    dlSetDitherMatrix(1);
    let up = g::creditscamup;
    let up_z = lwc1(up + 8);
    let up_y = lwc1(up + 4);
    let player_matrix = lw(lw(g::players) + 1760);
    swc1(sp + 128, lwc1(g::creditscamup));
    swc1(sp + 132, up_y);
    swc1(sp + 136, up_z);
    let matrix = memdbAlloc(64);
    let roll = (lwc1(g::creditscamrotz) * lwc1(0x3ad77c)).pdiv(1.8e+02_f32);
    sw(g::creditsmatrix, matrix);
    matrixRotZ(sp + 64, roll);
    matrixVecMul(sp + 64, sp + 128);
    matrixLookAt(lw(g::creditsmatrix), g::creditscampos, g::creditscamdir, sp + 128);
    sceVu0MulMatrix(sp, player_matrix, lw(g::creditsmatrix));
    dlSetDitherMatrix(1);
    if section == 0 {
        return;
    }
    loop {
        let mut ring = 2;
        let colours = section + 0x1200;
        let normals = section + 2304;
        loop {
            dlSetBlend(if (ring as u32) < 2 { 1 } else { 0 });
            let previous = ring - 1;
            let mut points = ring * 768 + section;
            let mut strip = 0;
            let mut rgba = ring * 192 + colours;
            let texture_at = (ring << 2) + g::tunneltexnum;
            let mut stnorm = ring * 768 + normals;
            loop {
                push_tag(g::credits_vif_load_mmtx, 1);
                push_tag(lw(g::creditsmatrix), 4);
                push_tag(g::credits_vif_entry_loadm, 1);
                if strip < 2 {
                    let texture = lw(texture_at);
                    let packet = lw((if -1 >= texture { 0 } else { texture }) * 40 + lw(lw(g::thistexbuf) + 4) + 32);
                    push_tag(packet, 8);
                }
                strip += 1;
                push_tag(g::credits_vif_load_giftags, 1);
                push_tag(g::credits_giftags, 1);
                push_tag(g::credits_vif_load_xyzw, 1);
                push_tag(points, 4);
                points += 64;
                push_tag(g::credits_vif_load_stnorm, 1);
                push_tag(stnorm, 4);
                stnorm += 64;
                push_tag(g::credits_vif_load_rgba, 1);
                push_tag(rgba, 1);
                rgba += 16;
                push_tag(g::startucode + 16, 1);
                if strip >= 12 {
                    break;
                }
            }
            ring = previous;
            if ring < 0 {
                break;
            }
        }
        section = lw(section + 0x1440);
        if section == 0 {
            break;
        }
    }
}

/// Draw a line from `from` to `to` in the world (points of three floats) in the colour `colour` (0xaarrggbb): a quad of 1 wide each side made of the points, in the current player's
/// view (the matrix at +1764 of the player), with the z test turned off and the clip tag off for it; nothing when the player has no matrix (@ 0x2c79a8).
pub fn drawLine(from: i32, to: i32, colour: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let packet = memdbAlloc(112);
    if lw(lw(g::currentplayer) + 1764) == 0 {
        return;
    }
    let old = dlSetZB(3);
    dlSetClipTag(0);
    let view = memdbAlloc(80);
    matrixTrans(sp, 0.0_f32, 0.0_f32, 0.0_f32);
    matrixRotY(sp + 64, 0.0_f32);
    sceVu0MulMatrix(sp + 128, sp, sp + 64);
    sceVu0MulMatrix(sp + 192, lw(lw(g::currentplayer) + 1764), sp + 128);
    sceVu0CopyMatrix(sp + 192, lw(lw(g::currentplayer) + 1764));
    sb(lw(g::gfxtag) + 3, 48);
    sw(view, 0x1000404);
    let tag = lw(g::gfxtag);
    sw(tag + 4, view);
    let next = tag + 16;
    let mut at = view + 4;
    sh(tag, 5);
    sw(at, 0x6c048004);
    at += 4;
    sw(g::gfxtag, next);
    let half = lwc1(0x3ad9b4);
    let alpha_shifted = ((colour as u32) >> 8) as i32;
    for k in 0..16 {
        sw(at, lw(sp + 192 + k * 4));
        at += 4;
    }
    sw(at + 4, 0x11000000);
    sw(at, 0x140007a8);
    sb(next + 3, 48);
    sw(packet, 0x6c018000);
    let tag = lw(g::gfxtag);
    let x0 = lwc1(from);
    sw(tag + 4, packet);
    let mut at = packet + 4;
    sw(at, 0x8004);
    at += 4;
    sh(tag, 7);
    sw(at, 0x30264000);
    at += 4;
    sw(g::gfxtag, tag + 16);
    sw(at, 1042);
    at += 4;
    sw(at, 0);
    at += 4;
    sw(at, 0x6c048018);
    let one = 1.0_f32;
    let corners = [
        (x0 + half, lwc1(from + 4), lwc1(from + 8)),
        (lwc1(from) - half, lwc1(from + 4), lwc1(from + 8)),
        (lwc1(to) + half, lwc1(to + 4), lwc1(to + 8)),
        (lwc1(to) - half, lwc1(to + 4), lwc1(to + 8)),
    ];
    for (x, y, z) in corners {
        at += 4;
        swc1(at, x);
        at += 4;
        swc1(at, y);
        at += 4;
        swc1(at, z);
        at += 4;
        swc1(at, one);
    }
    at += 4;
    sw(at, 0x6e04c058);
    at += 4;
    let mut left = 3;
    loop {
        sw(at, ((colour as u32) >> 24) as i32 | colour << 24 | ((alpha_shifted & 255) << 16 | alpha_shifted & 0xff00));
        left -= 1;
        at += 4;
        if left < 0 {
            break;
        }
    }
    sw(at, 0x14000000);
    dlSetClipTag(1);
    dlSetZB(old);
}
