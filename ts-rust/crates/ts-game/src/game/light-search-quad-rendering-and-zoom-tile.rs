//! Finding the two nearest lights of an object, drawing a screen-aligned quad, and the tick of the zoomed tile of the map maker.

use crate::*;

/// Find the two lights nearest to the place `at` for the object instance `instance` and put their records (a pointer to the light, then the difference to it in x, y, z and the
/// distance squared) at `*first` and `*second` (the records are at 0x1fc63c8 and 0x1fc63e0, with a third at 0x1fc63f8 to work in): the two lights found last time (halfwords at +544 and
/// +546 of the instance's record) are kept if they are both within 20 (400 squared) and the instance is not due for a new search (every eighth tick, by its number); else every
/// light is looked at (those that face away, by the vector at +128, count as far) and the two nearest within 20 are kept (@ 0x25a7d0).
pub fn bglightGet(instance: i32, at: i32, first: i32, second: i32) {
    let ticks = lw(g::updateframesi);
    let mut nearest = 0x1fc63c8;
    let mut scratch = 0x1fc63f8;
    let mut next = 0x1fc63e0;
    let mut frame = lw(g::gameframe);
    let number = ((instance - lw(g::obinstances)).wrapping_mul(-0x3e7063e7)) >> 3;
    if ticks > 0 {
        if ticks == 0 {
            trap("break");
        }
        frame = frame.wrapping_div(ticks);
    }
    sw(0x1fc63e0, 0);
    let mut search = 1;
    sw(0x1fc63c8, 0);
    if (number & 7) != frame & 7 {
        let record = lw(instance + 244);
        let second_index = lh(record + 546);
        let first_index = lh(record + 544);
        search = 0;
        if first_index >= 0 {
            let x = lwc1(at);
            let light = lw((first_index << 2) + 0x1fc5f18);
            let light_x = lwc1(light + 68);
            sw(0x1fc63c8, light);
            let dx = ps2float::sub(x, light_x);
            swc1(nearest + 4, dx);
            let dy = ps2float::sub(lwc1(at + 4), lwc1(light + 72));
            swc1(nearest + 8, dy);
            let dz = ps2float::sub(lwc1(at + 8), lwc1(light + 76));
            swc1(nearest + 12, dz);
            let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
            swc1(nearest + 16, distance);
            if 4e+02_f32 <= distance {
                search = 1;
            }
        }
        if second_index >= 0 {
            let x = lwc1(at);
            let light = lw((second_index << 2) + 0x1fc5f18);
            let light_x = lwc1(light + 68);
            sw(next, light);
            let dx = ps2float::sub(x, light_x);
            swc1(next + 4, dx);
            let dy = ps2float::sub(lwc1(at + 4), lwc1(light + 72));
            swc1(next + 8, dy);
            let dz = ps2float::sub(lwc1(at + 8), lwc1(light + 76));
            swc1(next + 12, dz);
            let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
            swc1(next + 16, distance);
            if 4e+02_f32 <= distance {
                search = 1;
            }
        }
    }
    if search != 0 {
        let record = lw(instance + 244);
        sw(nearest, 0);
        let mut i = 0;
        let count = lw(g::numbglights);
        sh(record + 546, -1);
        sw(next, 0);
        sh(record + 544, -1);
        if count > 0 {
            loop {
                let x = lwc1(at);
                let light = lw((i << 2) + 0x1fc5f18);
                let light_x = lwc1(light + 68);
                sw(scratch, light);
                let dx = ps2float::sub(x, light_x);
                swc1(scratch + 4, dx);
                let dy = ps2float::sub(lwc1(at + 4), lwc1(light + 72));
                swc1(scratch + 8, dy);
                let dz = ps2float::sub(lwc1(at + 8), lwc1(light + 76));
                swc1(scratch + 12, dz);
                let distance;
                if lh(light + 6) != 0 && ps2float::add(ps2float::add(ps2float::mul(dx, lwc1(light + 128)), ps2float::mul(dy, lwc1(light + 132))), ps2float::mul(dz, lwc1(light + 136))) <= 0.0_f32 {
                    distance = lwc1(g::numbglights + 4);
                } else {
                    let (a, b, c) = (lwc1(scratch + 4), lwc1(scratch + 8), lwc1(scratch + 12));
                    distance = ps2float::add(ps2float::add(ps2float::mul(a, a), ps2float::mul(b, b)), ps2float::mul(c, c));
                }
                swc1(scratch + 16, distance);
                let distance = lwc1(scratch + 16);
                if !(distance < 4e+02_f32) {
                    i += 1;
                } else {
                    'placed: {
                        let old_nearest = nearest;
                        if lw(nearest) != 0 && !(distance < lwc1(nearest + 16)) {
                            let other = lw(next);
                            let old_next = next;
                            if other != 0 && !(distance < lwc1(next + 16)) {
                                i += 1;
                                break 'placed;
                            }
                            sh(record + 546, i);
                            next = scratch;
                            scratch = old_next;
                            i += 1;
                            break 'placed;
                        }
                        nearest = scratch;
                        scratch = old_nearest;
                        if lw(next) != 0 && !(lwc1(old_nearest + 16) < lwc1(next + 16)) {
                            sh(record + 544, i);
                            i += 1;
                            break 'placed;
                        }
                        let old_next = next;
                        next = scratch;
                        sh(record + 546, lhu(record + 544));
                        scratch = old_next;
                        sh(record + 544, i);
                        i += 1;
                    }
                }
                if i >= count {
                    break;
                }
            }
        }
    }
    sw(first, nearest);
    sw(second, next);
}

/// Draw a quad facing the screen: the matrix `matrix` is copied, and a packet with the four corners (the three vectors of `corners` and the fourth made of them), the texture
/// coordinates from (`u`, `v`) to (`u` + 1, `v` + 1), and the four words `a5`, `a6`, `a7` and the colour on the caller's stack, is added to the display list with the texture
/// of the number `texture` (@ 0x2aaff8).
pub fn quadRenderAligned(texture: i32, matrix: i32, corners: i32, _a3: i32, _a4: i32, a5: i32, a6: i32, a7: i32, u: f32, v: f32) {
    let frame = Frame::new(176);
    let tail = lw(frame.sp() + 176);
    let copy = memdbAlloc(64);
    let packet = memdbAlloc(144);
    sceVu0CopyMatrix(copy, matrix);
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    let next = tag + 16;
    sw(tag + 4, g::vif_load_mmtx_36c1a0);
    sw(g::gfxtag, next);
    sh(tag, 1);
    sb(next + 3, 48);
    let tag = lw(g::gfxtag);
    let next = tag + 16;
    sw(tag + 4, copy);
    sw(g::gfxtag, next);
    let load = 0x68048000;
    sh(tag, 4);
    sb(next + 3, 48);
    let tag = lw(g::gfxtag);
    let u0 = ps2float::add(u, 0.0_f32);
    let one = 1.0_f32;
    let next = tag + 16;
    sw(tag + 4, g::vif_entry_loadm_36c1b0);
    sw(g::gfxtag, next);
    let v0 = ps2float::add(v, 0.0_f32);
    sh(tag, 1);
    let u1 = ps2float::add(u, one);
    sb(next + 3, 48);
    let v1 = ps2float::add(v, one);
    sw(packet, 0x6c018000);
    let tag = lw(g::gfxtag);
    let first = lwc1(corners);
    sw(tag + 4, packet);
    let after = tag + 16;
    let mut at = packet + 4;
    sh(tag, 9);
    sw(at, 0x8004);
    at += 4;
    sw(g::gfxtag, after);
    sw(at, 0x302e4000);
    at += 4;
    sw(at, 1042);
    at += 4;
    sw(at, 0);
    at += 4;
    sw(at, lw(g::ucode_loadxyzw) | load);
    at += 4;
    swc1(at, first);
    for offset in [4, 8, 16, 20, 24, 32, 36, 40, 48, 52, 56] {
        at += 4;
        swc1(at, lwc1(corners + offset));
    }
    at += 4;
    let load_st = lw(g::ucode_loadst);
    sw(at, load_st | load);
    for value in [u0, v0, one, u1, v0, one, u0, v1, one, u1, v1, one] {
        at += 4;
        swc1(at, value);
    }
    at += 4;
    let load_rgba = lw(g::ucode_loadrgba);
    sw(at, load_rgba | 0x6e048000);
    let buffers = lw(g::thistexbuf);
    at += 4;
    sw(at, a5);
    let entries = lw(buffers + 4);
    at += 4;
    sw(at, a6);
    at += 4;
    sw(at + 4, tail);
    let texture_word = lw((if -1 >= texture { 0 } else { texture }) * 40 + entries + 32);
    sw(at, a7);
    sb(after + 3, 48);
    let tag = lw(g::gfxtag);
    let next = tag + 16;
    sw(tag + 4, texture_word);
    sw(g::gfxtag, next);
    sh(tag, 8);
    sb(next + 3, 48);
    let tag = lw(g::gfxtag);
    sw(tag + 4, g::startucode);
    sh(tag, 1);
    sw(g::gfxtag, tag + 16);
}

/// The tick of the zoomed tile of the map maker: the tile shown (number `zoomtilenum` of 21, its prop from `tileprops`) is put at the middle and turned by `zoomtileroty`; with
/// flags 3 of `zoomtileflags` the pads of the tile (records of 24 bytes at 0x1fc4108; the count at 0x3b0078) are put in their places by the tile's turn, those facing the player
/// being turned on by the frame, and the circle prop is put on the aimed pad; the flexible camera is set at the tilt and distance of the zoom (@ 0x2506c0).
pub fn zoomtileTick() {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let number = lw(g::zoomtilenum);
    if (number as u32) >= 21 {
        return;
    }
    let at = number << 2;
    let prop = lw(at + g::tileprops);
    let depth = lw(lw(at + g::tiles) + 8) as f32;
    let flags = lw(g::zoomtileflags);
    sw(sp, 0);
    sw(sp + 8, 0);
    sw(prop + 140, 1);
    let prop_flags = lw(prop + 16);
    let turn = lwc1(g::zoomtileroty);
    swc1(sp + 4, -depth);
    sw(prop + 48, 0);
    sw(prop + 52, 0);
    sw(prop + 56, 0);
    let zero = lwc1(prop + 48);
    let x = lwc1(sp);
    sw(prop + 16, prop_flags | 0x1000001);
    swc1(prop + 48, ps2float::add(x, zero));
    swc1(prop + 52, ps2float::add(lwc1(sp + 4), zero));
    let z = lwc1(sp + 8);
    swc1(prop + 76, turn);
    swc1(prop + 56, ps2float::add(z, zero));
    if (flags & 3) != 0 {
        matrixRotY(sp + 32, ps2float::mul(turn, lwc1(0x3aca5c)).pdiv(1.8e+02_f32));
        let mut i = 0;
        let tile_x = lwc1(sp);
        let tile = lw((lw(g::zoomtilenum) << 2) + g::tiles);
        let tile_z = lwc1(sp + 8);
        let tile_y = ps2float::add(lwc1(sp + 4), zero);
        let size_z = lw(tile + 4) as f32;
        let pads = lw(0x3b0078);
        let size_x = lw(tile) as f32;
        swc1(sp + 20, zero);
        swc1(sp + 4, tile_y);
        let offset_z = ps2float::mul(-size_z, 2.5_f32);
        let offset_x = ps2float::mul(-size_x, 2.5_f32);
        swc1(sp + 24, offset_z);
        swc1(sp + 16, offset_x);
        swc1(sp + 8, ps2float::add(tile_z, offset_z));
        swc1(sp, ps2float::add(tile_x, offset_x));
        if pads > 0 {
            loop {
                let pad_record = i * 24 + 0x1fc4108;
                let pad = lw(pad_record);
                let pad_prop = lw(i * 24 + 0x1fc410c);
                let count;
                if (lw(pad + 4) & 16) == 0 {
                    sw(pad_prop + 16, lw(pad_prop + 16) & -2);
                    count = lw(0x3b0078);
                } else {
                    let place = pad_prop + 48;
                    swc1(pad_prop + 48, lwc1(pad + 12));
                    swc1(place + 4, lwc1(pad + 16));
                    swc1(place + 8, lwc1(pad + 20));
                    swc1(pad_prop + 48, ps2float::add(lwc1(pad_prop + 48), lwc1(sp)));
                    let depth = lwc1(place + 8);
                    swc1(place + 4, ps2float::add(lwc1(place + 4), lwc1(sp + 4)));
                    swc1(place + 8, ps2float::add(depth, lwc1(sp + 8)));
                    matrixVecMul(sp + 32, place);
                    let angle = lwc1(g::zoomtileroty);
                    let new_flags = lw(pad_prop + 16) | 0x1000001;
                    let aimed = lw(g::aimpadinfo);
                    swc1(pad_prop + 76, angle);
                    sw(pad_prop + 16, new_flags);
                    let turned = if pad_record != aimed {
                        ps2float::add(angle, 45.0_f32)
                    } else {
                        ps2float::add(angle, lw(g::tickframenum).wrapping_rem(360) as f32)
                    };
                    swc1(pad_prop + 76, turned);
                    count = lw(0x3b0078);
                    if 3.6e+02_f32 <= turned {
                        loop {
                            let less = ps2float::sub(lwc1(pad_prop + 76), 3.6e+02_f32);
                            swc1(pad_prop + 76, less);
                            if !(3.6e+02_f32 <= less) {
                                break;
                            }
                        }
                    }
                    let record = i * 24 + 0x1fc4108;
                    swc1(record + 12, lwc1(place));
                    swc1(record + 16, lwc1(place + 4));
                    swc1(record + 20, lwc1(place + 8));
                }
                i += 1;
                if i >= count {
                    break;
                }
            }
        }
        let aimed = lw(g::aimpadinfo);
        let circle = lw(g::circleprop);
        if aimed == 0 {
            sw(circle + 16, lw(circle + 16) & -2);
        } else {
            let mut angle = lwc1(g::zoomtileroty);
            let circle_flags = lw(circle + 16);
            swc1(circle + 76, angle);
            sw(circle + 16, circle_flags | 0x1000001);
            if 3.6e+02_f32 <= angle {
                loop {
                    angle = ps2float::sub(angle, 3.6e+02_f32);
                    if !(3.6e+02_f32 <= angle) {
                        break;
                    }
                }
                swc1(circle + 76, angle);
            }
            swc1(circle + 48, lwc1(aimed + 12));
            swc1(circle + 52, lwc1(aimed + 16));
            swc1(circle + 56, lwc1(aimed + 20));
        }
    }
    let camera = g::flexcam + 12;
    let tilt = ps2float::mul(lwc1(g::zoomtiletilt), lwc1(0x3aca60));
    let scale = lwc1(0x3aca64);
    sw(g::flexcam, 0);
    let distance = lwc1(g::zoomtiledist);
    sw(g::flexcam + 12, 0);
    sw(camera + 8, 0);
    sw(camera + 4, 0);
    let angle = ps2float::add(ps2float::mul((tilt).pdiv(1.8e+02_f32), scale), 0.5_f32) as i32;
    let up = ps2float::mul(lwc1(((angle + 512 & 2047) << 2) + g::g_SinTable), distance);
    swc1(g::flexcam + 4, ps2float::mul(lwc1(((angle & 2047) << 2) + g::g_SinTable), distance));
    swc1(g::flexcam + 8, up);
}
