//! The screen boxes of the portals of a room, and the drawing of the rings of the blasts.

use crate::*;

/// Find the part of the screen that each portal of the room `room` shows (its box at +24 to +36 of the portal data, in screen coordinates), through the matrix `matrix`: the
/// corners of the portal are moved by the matrix and given their out codes, a portal outside any plane is flagged (+44), the rest is clipped (`bgPortalPlaneClip`) against the near
/// plane and the four edges of the room's own box (kept at sp and sp+16 from the player's room data and in the floats 0x3b0088 to 0x3b0094), put on the screen, and its box taken
/// and clamped to the room's box; the number of portals seen is kept at sp+400 (@ 0x258108).
pub fn bgPortalCalcPos(room: i32, matrix: i32) {
    let frame = Frame::new(592);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    sw(sp + 384, room);
    let room_data = g::roomdata;
    let first = room * 44;
    let second = room * 20;
    let table = lw((lw(player) << 2) + room_data);
    let rooms = lw(g::levelrooms);
    sw(sp + 388, matrix);
    sw(sp + 400, 0);
    let entry = second + table;
    let list = lw(first + rooms + 4);
    sw(sp + 392, list);
    let valid = lw(entry + 12);
    sw(sp + 396, lw(list));
    if valid < 0 {
        return;
    }
    let window = g::window;
    let count = lw(sp + 396);
    let centre_x = lw(window + 8) as f32;
    let left = lw(entry + 4) - 1;
    sw(sp, left);
    let mut index = 0;
    let centre_y = lw(window + 28) as f32;
    let scale_x = lw(window + 16) as f32;
    let scale_y = lw(window + 36) as f32;
    let top = lw(second + lw((lw(player) << 2) + room_data) + 8) - 1;
    sw(sp + 4, top);
    let now = lw(player);
    swc1(0x3b0088, (left as f32 - centre_x) / scale_x);
    let right = lw(second + lw((now << 2) + room_data) + 12) + 1;
    sw(sp + 16, right);
    swc1(0x3b0090, (top as f32 - centre_y) / scale_y);
    let bottom = lw(second + lw((lw(player) << 2) + room_data) + 16) + 1;
    swc1(0x3b008c, (right as f32 - centre_x) / scale_x);
    sw(sp + 20, bottom);
    swc1(0x3b0094, (bottom as f32 - centre_y) / scale_y);
    if count <= 0 {
        return;
    }
    let minus_one = -1.0_f32;
    sw(sp + 404, second);
    let one = 1.0_f32;
    let mut player = player;
    loop {
        let portal_number = lw((index << 2) + lw(sp + 392) + 4);
        let portal = lw((portal_number << 2) + lw(g::levelportals));
        let data = lw(g::portaldata) + portal_number * 48;
        let mut other = lw(portal);
        if other == lw(sp + 384) {
            other = lw(portal + 4);
        }
        'portal: {
            let rooms_at = lw((lw(player) << 2) + g::roomdata);
            sw(0x3b0098, 0x70000000);
            let offset = lw(sp + 404);
            sw(0x3b009c, 20);
            let next = index + 1;
            if lw(other * 20 + rooms_at) < lw(offset + rooms_at) {
                index = next;
                break 'portal;
            }
            let points = lh(portal + 20);
            sw(data + 28, 0x2710);
            sw(data + 36, -1);
            let mut outside = -1;
            let mut last_count = lhu(portal + 20);
            let mut k = 0;
            sw(data + 24, 0x2710);
            sw(data + 32, -1);
            if points > 0 {
                loop {
                    let view = lw(sp + 388);
                    let vertex = lw(0x3b0098) + k * 20;
                    sw(vertex + 16, 0);
                    matrixVec3Mul4(view, portal + (k * 12 + 24), vertex);
                    bgPortalCalcOutCode(vertex);
                    let code = lw(vertex + 16);
                    sw(sp + 32 + (k << 2), vertex);
                    k += 1;
                    outside &= code;
                    last_count = lhu(portal + 20);
                    if k >= lh(portal + 20) {
                        break;
                    }
                }
            }
            let number = ((last_count << 16) as i32) >> 16;
            let first_vertex = lw(sp + 32);
            sw(sp + 112, number);
            sw(sp + 32 + (number << 2), first_vertex);
            if outside != 0 {
                sw(data + 44, 1);
                index = next;
                break 'portal;
            }
            bgPortalPlaneClip(sp + 32, sp + 128, 3, 16, minus_one, 0.0_f32);
            bgPortalPlaneClip(sp + 128, sp + 32, 0, 4, minus_one, lwc1(0x3b0088));
            bgPortalPlaneClip(sp + 32, sp + 128, 0, 8, minus_one, lwc1(0x3b008c));
            bgPortalPlaneClip(sp + 128, sp + 32, 1, 1, one, lwc1(0x3b0090));
            bgPortalPlaneClip(sp + 32, sp + 128, 1, 2, one, lwc1(0x3b0094));
            let clipped = lw(sp + 208);
            if clipped == 0 {
                sw(data + 44, 1);
                index = next;
                break 'portal;
            }
            let mut j = 0;
            if clipped > 0 {
                loop {
                    let vertex = lw(sp + 128 + (j << 2));
                    let at = j << 3;
                    let w = lwc1(vertex + 12);
                    let x_at = sp + 224 + at;
                    let y_at = sp + 228 + at;
                    let y = lw(window + 28) as f32 - lw(window + 36) as f32 * (lwc1(vertex + 4) / w);
                    swc1(x_at, f32::from_bits((lw(window + 8) as f32 + lw(window + 16) as f32 * (lwc1(vertex) / w)) as i32 as u32));
                    swc1(y_at, f32::from_bits(y as i32 as u32));
                    let value = lw(x_at);
                    if value < lw(data + 24) {
                        sw(data + 24, value);
                    }
                    let value = lw(y_at);
                    if value < lw(data + 28) {
                        sw(data + 28, value);
                    }
                    let value = lw(x_at);
                    if lw(data + 32) < value {
                        sw(data + 32, value);
                    }
                    let value = lw(y_at);
                    if lw(data + 36) < value {
                        sw(data + 36, value);
                    }
                    j += 1;
                    if j >= lw(sp + 208) {
                        break;
                    }
                }
            }
            let max_x = lw(data + 32);
            let room_left = lw(sp);
            if max_x >= 0 {
                if max_x >= room_left {
                    let max_y = lw(data + 36);
                    let room_top = lw(sp + 4);
                    if max_y >= room_top {
                        let min_x = lw(data + 24);
                        if lw(sp + 16) >= min_x {
                            let min_y = lw(data + 28);
                            if lw(sp + 20) >= min_y {
                                let mut above = (min_y < room_top) as i32;
                                if min_x < room_left {
                                    sw(data + 24, room_left);
                                    above = (min_y < lw(sp + 4)) as i32;
                                }
                                if above != 0 {
                                    sw(data + 28, lw(sp + 4));
                                }
                                let room_right = lw(sp + 16);
                                if room_right < max_x {
                                    sw(data + 32, room_right);
                                }
                                let room_bottom = lw(sp + 20);
                                if room_bottom < max_y {
                                    sw(data + 36, room_bottom);
                                }
                                sw(sp + 400, lw(sp + 400) + 1);
                                index = next;
                                break 'portal;
                            }
                        }
                    }
                }
                sw(data + 44, 1);
            }
            index = next;
        }
        player = lw(g::currentplayer);
        if index >= lw(sp + 396) {
            break;
        }
    }
}

/// Draw the rings of the blasts (`numrings` of them, records of 36 bytes from `br`): each ring is turned about its axis by its speed (`matrixroll`), grows with its time (the outer
/// radius at +0, the inner at +4 from three times the time) and when its time is past 1 all the rings are ended; a strip of 20 pairs of points (22.5 degrees apart, inner and
/// outer, put round the player's place by the turned matrix) is made and sent as a packet of 1936 bytes with the colour 0x1e7f2824 (@ 0x2a72f0).
pub fn blastringGfx() {
    let frame = Frame::new(848);
    let sp = frame.sp();
    let players = lw(g::players);
    let mut ring = 0;
    let rings = lw(g::numrings);
    swc1(sp + 512, lwc1(players + 152));
    swc1(sp + 516, lwc1(players + 156));
    swc1(sp + 520, lwc1(players + 160));
    if rings <= 0 {
        return;
    }
    sw(sp + 640, sp + 560);
    sw(sp + 628, sp + 528);
    sw(sp + 636, sp + 544);
    sw(sp + 624, sp + 512);
    sw(sp + 652, sp + 12);
    sw(sp + 644, sp + 4);
    sw(sp + 648, sp + 8);
    loop {
        let offset = ring * 36;
        let base = g::br + 24;
        let angle_at = ring * 36 + (base - 8);
        matrixroll(lw(sp + 640), ring * 36 + base, lwc1(angle_at));
        let full = 3.6e+02_f32;
        let angle = lwc1(angle_at) + lwc1(base + offset - 4) * lwc1(g::tickframesf);
        swc1(angle_at, angle);
        let frames;
        if !(full < angle) {
            frames = lwc1(g::updateframesf);
            sw(sp + 632, ring + 1);
        } else {
            frames = lwc1(g::updateframesf);
            sw(sp + 632, ring + 1);
            loop {
                let at = ring * 36 + (g::br + 16);
                let back = lwc1(at) - full;
                swc1(at, back);
                if !(full < back) {
                    break;
                }
            }
        }
        let offset = ring * 36;
        let one = 1.0_f32;
        let rings_at = g::br;
        let time_at = ring * 36 + (rings_at + 12);
        let time = lwc1(time_at) + frames / 6e+01_f32;
        swc1(time_at, time);
        if one < time {
            sw(g::numrings, 0);
        }
        let inner = time * 3.0_f32;
        let outer_at = offset + (rings_at + 4);
        let mut turn = 0.0_f32;
        let mut pair = 0;
        swc1(offset + rings_at, inner);
        let mut step = 0;
        let matrix = lw(sp + 640);
        let player_at = lw(sp + 624);
        swc1(outer_at, inner + one);
        let inner_vec = lw(sp + 628);
        let outer_vec = lw(sp + 636);
        loop {
            let radians = turn * lwc1(0x3ad620);
            turn += 22.5_f32;
            let radians = radians / 1.8e+02_f32;
            let cosine = cosf(radians);
            let sine = sinf(radians);
            let outer_radius = lwc1(outer_at);
            let inner_radius = lwc1(offset + g::br);
            sw(sp + 532, 0);
            sw(sp + 548, 0);
            swc1(sp + 552, outer_radius * sine);
            swc1(sp + 544, outer_radius * cosine);
            swc1(sp + 536, inner_radius * sine);
            swc1(sp + 528, inner_radius * cosine);
            matrixVecRotAligned(&[matrix as i64, lw(sp + 628) as i64]);
            matrixVecRotAligned(&[matrix as i64, lw(sp + 636) as i64]);
            let next_step = step + 1;
            let next_pair = pair + 2;
            let mut inner_at = inner_vec;
            let mut outer_at_vec = outer_vec;
            let mut left = 2;
            let mut first_at = pair * 12 + sp;
            let mut second_at = pair * 12 + 12 + sp;
            let mut player_value = player_at;
            loop {
                let inner_value = lwc1(inner_at);
                left -= 1;
                inner_at += 4;
                swc1(first_at, lwc1(player_value) + inner_value);
                first_at += 4;
                let player_now = lwc1(player_value);
                let outer_value = lwc1(outer_at_vec);
                player_value += 4;
                outer_at_vec += 4;
                swc1(second_at, player_now + outer_value);
                second_at += 4;
                if left < 0 {
                    break;
                }
            }
            step = next_step;
            pair = next_pair;
            if step >= 20 {
                break;
            }
        }
        let mut source = lw(sp + 652);
        let mut from = sp;
        let mut left = 2;
        let mut first_at = pair * 12 + sp;
        let mut second_at = (pair + 1) * 12 + sp;
        loop {
            let value = lwc1(from);
            left -= 1;
            from += 4;
            swc1(first_at, value);
            first_at += 4;
            let other = lwc1(source);
            source += 4;
            swc1(second_at, other);
            second_at += 4;
            if left < 0 {
                break;
            }
        }
        let packet = memdbAlloc(1936);
        let colour = 0x1e7f2824;
        dlSetClipTag(0);
        sb(lw(g::gfxtag) + 3, 48);
        sw(packet, 0x6c148000);
        let tag = lw(g::gfxtag);
        sw(tag + 4, packet);
        let mut put = packet + 4;
        sh(tag, 121);
        sw(g::gfxtag, tag + 16);
        for k in 0..20 {
            let at = put + 16 * k;
            sw(at, if k == 19 { 0x8004 } else { 4 });
            sw(at + 4, 0x30264000);
            sw(at + 8, 1042);
            sw(at + 12, 0);
        }
        put += 320;
        let second = lw(sp + 648);
        sw(put, 0x6c508018);
        let mut first_vec = sp;
        put += 4;
        let mut count = 19;
        let mut q = put + 4;
        let mut third_vec = lw(sp + 644);
        let mut second_vec = second;
        loop {
            put += 64;
            count -= 1;
            swc1(q - 4, lwc1(first_vec));
            swc1(q, lwc1(third_vec));
            let a = lwc1(second_vec);
            swc1(q + 8, 1.0_f32);
            swc1(q + 4, a);
            swc1(q + 12, lwc1(first_vec + 12));
            swc1(q + 16, lwc1(third_vec + 12));
            let b = lwc1(second_vec + 12);
            swc1(q + 24, 1.0_f32);
            swc1(q + 20, b);
            swc1(q + 28, lwc1(first_vec + 24));
            swc1(q + 32, lwc1(third_vec + 24));
            let c = lwc1(second_vec + 24);
            swc1(q + 40, 1.0_f32);
            swc1(q + 36, c);
            let d = lwc1(first_vec + 36);
            first_vec += 24;
            swc1(q + 44, d);
            let e = lwc1(third_vec + 36);
            third_vec += 24;
            swc1(q + 48, e);
            let f = lwc1(second_vec + 36);
            second_vec += 24;
            swc1(q + 52, f);
            swc1(q + 56, 1.0_f32);
            q += 64;
            if count < 0 {
                break;
            }
        }
        sw(put, 0x6e50c058);
        let mut colours = put + 4;
        let mut v = colours + 4;
        let mut left = 19;
        loop {
            sw(v - 4, colour);
            colours += 16;
            sw(v + 4, colour);
            left -= 1;
            sw(v, colour);
            sw(v + 8, colour);
            v += 16;
            if left < 0 {
                break;
            }
        }
        sw(colours, 0x14000000);
        ring = lw(sp + 632);
        dlSetClipTag(1);
        if ring >= lw(g::numrings) {
            break;
        }
    }
}
