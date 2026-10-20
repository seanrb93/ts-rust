//! The message of the HUD, the animation matrices of a prop, and the centre of a prop.

use crate::*;

/// Draw the HUD message of the player `player` (the text at +284 of the player's 352-byte record, its time at +348, which counts down to 0 and is where nothing is drawn): the lines
/// are placed from the bottom of the window up by their count, the text slides in from the left over the first 30 ticks and out over the 30 ticks from 150, a line starting with ~ is
/// in the other colour and one starting with ` is moved down by half a line (@ 0x2b0a30).
pub fn hudMessageDraw(player: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let record = player * 352 + g::hudparms;
    let text = record + 284;
    if lw(record + 348) == 0 {
        return;
    }
    let dither = dlSetDitherMatrix(2);
    sw(sp + 4, dither);
    let players = ilinkGetNumLocalPlayers();
    let font = textSelect(if players < 3 { 2 } else { 1 });
    sw(sp, font);
    let mut lines = 1;
    let time;
    let flags;
    let current_font;
    if lbu(record + 284) == 0 {
        time = lw(record + 348);
        flags = lbu(g::fontflags);
        current_font = lw(g::curfont);
    } else {
        time = lw(record + 348);
        flags = lbu(g::fontflags);
        current_font = lw(g::curfont);
        let mut i = 0;
        loop {
            let character = lbu(text + i);
            i += 1;
            lines = if (character ^ 10) == 0 { lines + 1 } else { lines };
            if lbu(text + i) == 0 {
                break;
            }
        }
    }
    let window = g::window;
    let height = lw(window + 32);
    let bottom = lw(window + 24);
    let line_height = if (flags & 1) == 0 { lw(current_font + 12) } else { lw(current_font + 12) >> 1 };
    let mut y = bottom - ((if -1 < height { height } else { height + 3 }) >> 2) - line_height * lines;
    let offset;
    if time >= 30 {
        let past = time - 150;
        let mut slide = 0;
        if (past as u32) < 30 {
            let half = 0x3fe0000000000000_i64;
            let fraction = past as f32 / 3e+01_f32;
            let turned = dpmul(dpmul(fptodp(fraction), ld(0x3a9a28)), half);
            let index = dptoli(dpadd(dpmul(turned, ld(0x3a9a30)), half));
            slide = (-((1.0_f32 - lwc1((((index + 512) & 2047) << 2) + g::g_SinTable)) * lw(g::window + 12) as f32)) as i32;
        }
        offset = slide;
    } else {
        let half = 0x3fe0000000000000_i64;
        let fraction = time as f32 / 3e+01_f32;
        let turned = dpmul(dpmul(fptodp(fraction), ld(0x3a9a18)), half);
        let index = dptoli(dpadd(dpmul(turned, ld(0x3a9a20)), half));
        offset = ((1.0_f32 - lwc1(((index & 2047) << 2) + g::g_SinTable)) * lw(g::window + 12) as f32) as i32;
    }
    let total = strlen(text);
    let mut at = 0;
    let mut line = text + at;
    loop {
        let end = strchr(line, 10);
        let mut colour = 0x64806480;
        let mut outline = 0x280060;
        if at >= total {
            exit(0);
        }
        if end != 0 {
            sb(end, 0);
        }
        let first = lbu(line);
        let mut next = first;
        if first != 126 {
            if first == 96 {
                let raw = lw(lw(g::curfont) + 12);
                let sign = ((raw as u32) >> 31) as i32;
                let height = if (lbu(g::fontflags) & 1) == 0 { raw } else { raw >> 1 };
                y += (height + sign) >> 1;
                at += 1;
                line = text + at;
                next = lbu(line);
            }
        } else {
            at += 1;
            colour = 0x40787880;
            outline = 0x5080;
            line = text + at;
            next = lbu(line);
        }
        if next != 0 {
            let width = textWidth(line);
            let x = lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1) + offset;
            textOutline(x, y, line, colour, outline);
        }
        let skip = at + 1;
        let size = strlen(line);
        at = skip + size;
        if (lbu(g::fontflags) & 1) == 0 {
            y += lw(lw(g::curfont) + 12);
        } else {
            y += lw(lw(g::curfont) + 12) >> 1;
        }
        if end != 0 {
            sb(end, 10);
        }
        let all = strlen(text);
        line = text + at;
        if (at as u32) >= (all as u32) {
            break;
        }
    }
    textSelect(lw(sp));
    dlSetDitherMatrix(lw(sp + 4));
}

/// Advance the animation of the prop `prop` (its state at +96, the animation from the table `pAnimations`) and make its bone matrices: the time (+4 of the state) goes on by the
/// frames times the speed (+44) over 60, looping (flag 1 of the animation) or stopping at the end, with +56 set when the animation went round; a looping animation also moves the
/// travelled place (+20) by the difference of its last and first frames; the matrices of the bones are then the keyframes (`getKeyFrames`) blended by the part gone (quaternion
/// slerp for the turn, linear for the place) and put in the global form; with flag 2 the movement of the root since the last tick is kept at +116 to +136, else cleared (@ 0x2143b0).
pub fn calPropMatrices(prop: i32, scale: f32) {
    let frame = Frame::new(400);
    let sp = frame.sp();
    let state = prop + 96;
    let anim = lw((lw(prop + 96) << 2) + g::pAnimations);
    let length = lwc1(anim + 16);
    if 0.0_f32 < length {
        let step = lwc1(g::updateframesf) * lwc1(state + 44);
        let time = lwc1(state + 4);
        sw(state + 56, 0);
        let looping = lw(anim + 12) & 1;
        let moved = time + step / 6e+01_f32;
        swc1(state + 4, moved);
        if looping == 0 {
            if length < moved {
                swc1(state + 4, length);
                sw(state + 56, 1);
            }
        } else if length < moved {
            loop {
                let now = lwc1(state + 4);
                sw(state + 56, 1);
                let back = now - length;
                swc1(state + 4, back);
                if !(length < back) {
                    break;
                }
            }
        }
        let mut now = lwc1(state + 4);
        let wrapped;
        if !(now < 0.0_f32) {
            wrapped = lw(state + 56);
        } else {
            let mut up;
            loop {
                up = now + length;
                now = up;
                if !(up < 0.0_f32) {
                    break;
                }
            }
            swc1(state + 4, up);
            sw(state + 56, 1);
            wrapped = lw(state + 56);
        }
        if wrapped != 0 && (lw(anim + 12) & 1) != 0 {
            extractAnimData(prop, anim, 0, lw(anim + 4) - 1, sp + 176, scale);
            extractAnimData(prop, anim, 0, 0, sp + 208, scale);
            let y = lwc1(sp + 212) + lwc1(state + 24) - lwc1(sp + 180);
            let z = lwc1(sp + 216) + lwc1(state + 28) - lwc1(sp + 184);
            swc1(state + 20, lwc1(sp + 208) + lwc1(state + 20) - lwc1(sp + 176));
            swc1(state + 24, y);
            swc1(state + 28, z);
        }
    }
    let model = lw(prop + 244);
    let flags = lw(model + 16);
    if (flags & 256) != 0 || (flags & 0xf000000) == 0 {
        return;
    }
    getKeyFrames(prop, state, sp + 240, sp + 244);
    let after = lw(sp + 244);
    let mut before = lw(sp + 240);
    let blend;
    let count;
    if before == after {
        blend = 0.0_f32;
        count = lw(anim + 8);
    } else {
        let times = lw(anim + 20);
        let from = lw((before << 2) + times) as f32 / 6e+01_f32;
        blend = (lwc1(state + 4) - from) / (lw((after << 2) + times) as f32 / 6e+01_f32 - from);
        count = lw(anim + 8);
    }
    let mut i = 0;
    if count > 0 {
        loop {
            extractAnimData(prop, anim, i, before, sp + 112, scale);
            let after_now = lw(sp + 244);
            extractAnimData(prop, anim, i, after_now, sp + 144, scale);
            i += 1;
            let a1 = lwc1(sp + 128);
            let a2 = lwc1(sp + 132);
            let a3 = lwc1(sp + 136);
            let b0 = lwc1(sp + 156);
            let b1 = lwc1(sp + 160);
            let b2 = lwc1(sp + 164);
            let b3 = lwc1(sp + 168);
            swc1(sp + 16, lwc1(sp + 124));
            swc1(sp + 20, a1);
            swc1(sp + 24, a2);
            swc1(sp + 28, a3);
            swc1(sp + 32, b0);
            swc1(sp + 36, b1);
            swc1(sp + 40, b2);
            swc1(sp + 44, b3);
            quaternionSlerp(&[(sp + 16) as i64, (sp + 32) as i64, sp as i64, blend.to_bits() as i64]);
            quaternionToMatrix(lw(prop + 4), sp);
            let x0 = lwc1(sp + 112);
            swc1(lw(prop + 4) + 48, x0 + (lwc1(sp + 144) - x0) * blend);
            let y0 = lwc1(sp + 116);
            swc1(lw(prop + 4) + 52, y0 + (lwc1(sp + 148) - y0) * blend);
            let z0 = lwc1(sp + 120);
            swc1(lw(prop + 4) + 56, z0 + (lwc1(sp + 152) - z0) * blend);
            before = lw(sp + 240);
            if i >= lw(anim + 8) {
                break;
            }
        }
    }
    calGlobalMatrices(prop, lw(prop + 4), 0, -1);
    if (lw(anim + 12) & 2) == 0 {
        sw(prop + 136, 0);
        sw(prop + 128, 0);
        sw(prop + 132, 0);
    } else {
        let a = lwc1(prop + 112);
        let b = lwc1(prop + 104);
        let c = lwc1(prop + 108);
        let d = lwc1(prop + 120);
        let move_x = b - lwc1(prop + 116);
        let e = lwc1(prop + 124);
        swc1(prop + 124, a);
        swc1(prop + 128, move_x);
        swc1(prop + 116, b);
        swc1(prop + 132, c - d);
        swc1(prop + 136, a - e);
        swc1(prop + 120, c);
    }
}

/// Stretch the box at sp+80 (lowest) and sp+96 (highest) over the points `points` of the lists of vertices starting at `list` (records of 24 bytes ended by a negative +20; the first
/// point of a list at +12, the count at +16).
fn grow_box(sp: i32, mut list: i32, points: i32) {
    let mut first = lw(list + 12);
    loop {
        let count = lw(list + 16);
        let mut i = 0;
        let mut point = first * 12 + points;
        if count <= 0 {
            list += 24;
        } else {
            let next = list + 24;
            loop {
                let step = i + 1;
                let after = point + 12;
                let mut at = point;
                let mut high = sp + 96;
                let mut low = sp + 80;
                let mut left = 2;
                loop {
                    let mut value = lwc1(at);
                    let current_high;
                    if !(value < lwc1(low)) {
                        current_high = lwc1(high);
                    } else {
                        swc1(low, value);
                        value = lwc1(at);
                        current_high = lwc1(high);
                    }
                    if current_high < value {
                        swc1(high, value);
                    }
                    high += 4;
                    low += 4;
                    left -= 1;
                    at += 4;
                    if left < 0 {
                        break;
                    }
                }
                i = step;
                point = after;
                if i >= count {
                    break;
                }
            }
            list = next;
        }
        if lw(list + 20) < 0 {
            break;
        }
        first = lw(list + 12);
    }
}

/// Find the box that holds the points of the bones' two lists (+8 with the points at +24, +12 with the points at +48) of the prop `prop`'s model, and from it the centre of the prop
/// (half way, kept in the table at 0x1fd0df0 by the prop's number at +4, and turned about y by the prop's angle at +76 to +508), its half-height (+524) and its radius (the smaller
/// of the half width and half depth, +528, doubled for prop 310) (@ 0x267f08).
pub fn propCalculateCentrePos(prop: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let mut i = 0;
    let high = lwc1(0x3af908);
    let low = lwc1(0x3af90c);
    let skeleton = lw(lw(prop + 32));
    let count = lw(skeleton);
    swc1(sp + 80, high);
    swc1(sp + 96, low);
    swc1(sp + 88, high);
    swc1(sp + 84, high);
    swc1(sp + 104, low);
    swc1(sp + 100, low);
    let number = lw(prop + 4);
    let min_x;
    if count <= 0 {
        min_x = lwc1(sp + 80);
    } else {
        loop {
            let bone = i * 80 + (skeleton - count * 80);
            let first = lw(bone + 8);
            let second;
            let next = i + 1;
            if first != 0 && lw(first + 20) >= 0 {
                let points = lw(bone + 24);
                second = lw(bone + 12);
                grow_box(sp, first, points);
            } else {
                second = lw(bone + 12);
            }
            i = next;
            if second != 0 && lw(second + 20) >= 0 {
                let points = lw(bone + 44 + 4);
                grow_box(sp, second, points);
            }
            if !(i < count) {
                break;
            }
        }
        min_x = lwc1(sp + 80);
    }
    let min_y = lwc1(sp + 84);
    let table = 0x1fd0df0;
    let min_z = lwc1(sp + 88);
    let half = 0.5_f32;
    let at = table + number * 12;
    let degrees = lwc1(0x3acbcc);
    let centre_x = min_x + (lwc1(sp + 96) - min_x) * half;
    let centre_y = min_y + (lwc1(sp + 100) - min_y) * half;
    let centre_z = min_z + (lwc1(sp + 104) - min_z) * half;
    swc1(number * 12 + table, centre_x);
    swc1(at + 4, centre_y);
    swc1(at + 8, centre_z);
    swc1(sp + 64, centre_x);
    let angle = lwc1(prop + 76);
    swc1(sp + 68, centre_y);
    swc1(sp + 72, centre_z);
    matrixRotY(sp, angle * degrees / 1.8e+02_f32);
    matrixVecMulAligned(&[sp as i64, (sp + 64) as i64]);
    let depth = lwc1(sp + 104) - lwc1(sp + 88);
    let turned_y = lwc1(sp + 68);
    let width = (lwc1(sp + 96) - lwc1(sp + 80)) * half;
    let turned_z = lwc1(sp + 72);
    swc1(prop + 508, lwc1(sp + 64));
    let depth_half = depth * half;
    swc1(prop + 512, turned_y);
    swc1(prop + 516, turned_z);
    if !(depth_half < width) {
        swc1(prop + 528, depth_half);
    } else {
        swc1(prop + 528, width);
    }
    let kind = lw(prop + 4);
    swc1(prop + 524, (lwc1(sp + 100) - lwc1(sp + 84)) * 0.5_f32);
    if kind == 310 {
        let radius = lwc1(prop + 528);
        swc1(prop + 528, radius + radius);
    }
}
