//! Reading animation data, colouring the parts of a model, and the colours and textures of bullet decals.

use crate::*;

/// The data of an animation segment: position (3 floats) and rotation (4 floats) into `out` for frame `frame` of segment `segment` (32-byte records from +32 of the animation; the kind
/// at +4, the data at +20), by the layout of the kind (0, 4, 8 or 2); the position is then scaled (@ 0x215280).
pub fn extractAnimData(chr: i32, anim: i32, segment: i32, frame: i32, out: i32, scale: f32) {
    let mut scale = scale;
    let entry = lw(anim + 32) + (segment << 5);
    let kind = lw(entry + 4);
    let copy7 = |src: i32| {
        for at in [0, 4, 8, 12, 16, 20, 24] {
            swc1(out + at, lwc1(src + at));
        }
    };
    match kind {
        2 => {
            let list = lw(entry + 20);
            let rotation = (frame << 4) + list;
            swc1(out, lwc1(list));
            swc1(out + 4, lwc1(list + 4));
            swc1(out + 8, lwc1(list + 8));
            swc1(out + 12, lwc1(rotation + 12));
            swc1(out + 16, lwc1(rotation + 16));
            swc1(out + 20, lwc1(rotation + 20));
            swc1(out + 24, lwc1(rotation + 24));
        }
        4 => {
            if frame != 0 {
                let list = lw(entry + 20);
                let at = (frame - 1) * 12 + list;
                swc1(out, lwc1(at + 28));
                swc1(out + 4, lwc1(at + 32));
                swc1(out + 8, lwc1(at + 36));
                swc1(out + 12, lwc1(list + 12));
                swc1(out + 16, lwc1(list + 16));
                swc1(out + 20, lwc1(list + 20));
                swc1(out + 24, lwc1(list + 24));
            } else {
                copy7(lw(entry + 20));
            }
        }
        8 => copy7(frame * 28 + lw(entry + 20)),
        0 => copy7(lw(entry + 20)),
        _ => {}
    }
    let model = lw(chr + 88);
    if model != 0 && lw(model + 12) == 1 && !(lw(chr + 96) >= 566 && segment > 0) {
        scale = ps2float::mul(scale, lwc1(0x3ac838));
    }
    let (y, z) = (ps2float::mul(lwc1(out + 4), scale), ps2float::mul(lwc1(out + 8), scale));
    swc1(out, ps2float::mul(lwc1(out), scale));
    swc1(out + 4, y);
    swc1(out + 8, z);
}

/// Multiply a colour word by another, channel by channel (each product divided by 256) with the channels taken in the order the model's colours use (@ the loop of `partColour`).
fn modulate(c: i32, by: i32) -> i32 {
    ((((c & 255) * ((by as u32) >> 24) as i32) as u32) >> 8) as i32
        | ((((((c as u32) >> 24) as i32 * (by & 255)) as u32) >> 8) as i32) << 24
        | ((((((((c as u32) >> 16) as i32 & 255) * (((by as u32) >> 8) as i32 & 255)) as u32) >> 8) as i32) << 16
            | (((c as u32) >> 8) as i32 & 255) * (((by as u32) >> 16) as i32 & 255) & 0xff00)
}

/// Multiply the vertex colours of the parts of a bone and the bones below it by the colour `colour`: `which` 0 uses the bone's first list of chunks (+8), else the second
/// (+12); a chunk record (24 bytes, the vertex count at +16, the offset at +12) says where its colours are (@ 0x2604e0).
pub fn partColour(skeleton: i32, bone: i32, which: i32, colour: i32) {
    let mut bone = bone;
    loop {
        let bones = skeleton - lw(skeleton) * 80;
        let this = bones + bone * 80;
        let mut chunk = if which == 0 { lw(this + 8) } else { lw(this + 12) };
        let colours_at = if which == 0 { this + 20 } else { this + 44 };
        if chunk != 0 && lw(chunk + 20) >= 0 {
            let base = lw(colours_at + 12);
            let mut offset = lw(chunk + 12);
            loop {
                let colours = base + (offset << 2);
                let count = lw(chunk + 16);
                for k in 0..count.max(0) {
                    let at = colours + 4 * k;
                    sw(at, modulate(lw(at), colour));
                }
                chunk += 24;
                if lw(chunk + 20) < 0 {
                    break;
                }
                offset = lw(chunk + 12);
            }
        }
        let child = lb(this + 3);
        if child >= 0 && lb(this) != 3 {
            partColour(skeleton, child, which, colour);
        }
        bone = lb(this + 4);
        if bone < 0 {
            break;
        }
    }
}

/// The texture (`out_texture`), scale (`out_scale`) and colour (`out_colour`) of the decal of a bullet of kind `kind`: random choices by kind; with the cheat 16 selected
/// the decals are coloured by kind and bigger (@ 0x2a4758).
pub fn decalGetBulletTexScaleCol(kind: i32, out_texture: i32, out_scale: i32, out_colour: i32) {
    let s = kind - 2;
    let mut by_kind = false;
    if (s as u32) < 17 {
        match s {
            0..=2 => {}
            3..=5 => {
                let r = newrnd();
                let factor = lwc1(0x3ad5dc);
                swc1(out_scale, factor);
                sw(out_texture, ((r as i32 as u32) % 3) as i32 + 2);
                by_kind = true;
            }
            _ => {
                let r = newrnd();
                swc1(out_scale, 1.0);
                sw(out_texture, ((r as i32 as u32) % 3) as i32 + 2);
                by_kind = true;
            }
        }
    }
    if !by_kind {
        let r = newrnd();
        sw(out_texture, (r & 1) as i32);
        swc1(out_scale, 1.0);
    }
    if lw(g::allowedcheats) & 16 & (lw(g::unlockedcheats) & lw(g::selectedcheats)) == 0 {
        sw(out_colour, 0xffffff);
    } else {
        swc1(out_scale, 1.0);
        let mut default = true;
        if (s as u32) < 17 {
            let colour = match s {
                0 => None,
                1 => Some((0xff3f00, false)),
                2 => Some((0xffff, false)),
                3 => Some((255, false)),
                4 => Some((0xff00, false)),
                5 => Some((0xff0000, false)),
                6 | 7 => Some((0xff003f, true)),
                8..=11 => Some((0xff00ff, true)),
                12..=14 => Some((0x3f0000 | 0xff00, true)),
                _ => Some((0xff0000 | 0xff00, true)),
            };
            if let Some((value, big)) = colour {
                sw(out_colour, value);
                if big {
                    swc1(out_scale, 2.0);
                }
                default = false;
            }
        }
        if default {
            sw(out_colour, 0x3fff);
        }
        let r = newrnd();
        sw(out_texture, (r & 1) as i32 + 10);
    }
}
