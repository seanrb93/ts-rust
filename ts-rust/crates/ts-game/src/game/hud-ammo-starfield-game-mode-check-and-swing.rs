//! The ammo of the HUD, the starfield of the space levels, the check of the map maker's items for the game modes, and the swing of a prop.

use crate::*;
use super::random_as_float;

/// Draw the ammo of the gun the current player holds at the right edge `x` of the window: the ammo's picture (the texture numbers at 0x1feaf40 by the gun's ammo kind at +16),
/// the number in the gun (`vafmt` with the format at 0x3afba8; shadow, then outlined), and, for a gun with a clip (+20), the clip's number before it. `ammo` holds the gun
/// table: its clip counts after +4 (@ 0x2b1c20).
pub fn hudAmmoGfx(ammo: i32, _unused: i32, x: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let record = lw(lw(lw(g::currentplayer) + 384) + 352);
    let gun = lw(ammo) * 400 + GUNS_TABLE;
    let dither = dlSetDitherMatrix(2);
    sw(sp + 28, dither);
    let window = g::window;
    let mut y = lw(window + 24) - ps2float::mul(lwc1(window + 84), 2e+01_f32) as i32;
    let font;
    if lw(window + 92) != 0 {
        font = textSelect(1);
        y -= 4;
    } else {
        font = textSelect(0);
    }
    sw(sp + 24, font);
    let mut kind = lw(gun + 16);
    #[cfg(feature = "extensions")]
    let x = {
        // (the right hand's counter ends at the edge of the screen whatever its width; the original left it where its digits ended)
        let right = lw(window + 4);
        if x > (lw(window) + right) / 2 {
            let digits = vafmt(&[0x3afba8, lw(record + (kind << 2) + 620) as i64, x as i64, 0]);
            right - 4 - 16 - textWidth(digits)
        } else {
            x
        }
    };
    let texture = lw((kind << 2) + 0x1feaf40);
    let buffers = lw(g::thistexbuf);
    if texture >= 0 {
        let entries = lw(buffers + 4);
        let split = lw(g::window + 92);
        sw(sp + 16, x - 16);
        let entry = texture * 40 + entries;
        let top = if split == 0 { y } else { y - 2 };
        let height = lhu(entry + 2);
        let width = lhu(entry);
        dlSelectTextureKick(texture);
        let left = lw(sp + 16);
        sw(sp, 0x7f7f7f7f);
        dlTextureRectangle(left, top, left + width, top + (height >> 1), 0, 0, width << 4, height << 4);
        kind = lw(gun + 16);
    }
    let format = 0x3afba8;
    let shadow_y = y + 2;
    let text = vafmt(&[format, lw(record + (kind << 2) + 620) as i64, x as i64, texture as i64]);
    sw(sp + 16, x + 20);
    sw(sp + 20, shadow_y);
    textPrint(sp + 16, sp + 20, text, 64);
    sw(sp + 16, x + 16);
    sw(sp + 20, y);
    textOutline(x + 16, y, text, 0x7f7f7f80, 100);
    if lw(gun + 20) > 0 {
        let clip = vafmt(&[format, lw(ammo + (lw(gun + 16) << 2) + 4) as i64]);
        let width = textWidth(clip);
        sw(sp + 20, shadow_y);
        sw(sp + 16, x - (width + 16));
        textPrint(sp + 16, sp + 20, clip, 64);
        let left = x - textWidth(clip) - 20;
        sw(sp + 16, left);
        sw(sp + 20, y);
        textOutline(left, y, clip, 0x7f7f7f80, 100);
    }
    dlSetDitherMatrix(lw(sp + 28));
    textSelect(lw(sp + 24));
}

/// Start the stars of the space level (17): the texture numbers of the stars and the fires, 200 stars at random places in the square from 0x3ad634 across 0x3ad630 (the
/// brightness the product of the distances from the corner) and set by `setStar`, and the fires (24 bytes each, to the end of the array) at random starts (@ 0x2a8778).
pub fn starfieldRestart() {
    if lw(g::lvlevelnum) != 17 {
        return;
    }
    let scale = 2.3283064e-10_f32;
    let size = lwc1(0x3ad630);
    let texture = preloadGetTexName(0x3a9608);
    let corner = lwc1(0x3ad634);
    sw(g::starfieldtexnum, lw(texture + 20));
    let fire_texture = preloadGetTexName(0x3a9630);
    sw(g::spacefiretexnum, lw(fire_texture + 20));
    let mut i = 0;
    let mut star = g::stars;
    loop {
        swc1(star, ps2float::add(ps2float::mul(ps2float::mul(random_as_float(), scale), size), corner));
        swc1(star + 4, ps2float::add(ps2float::mul(ps2float::mul(random_as_float(), scale), size), corner));
        let brightness = lwc1(0x3ad638);
        swc1(star + 20, ps2float::mul(ps2float::mul(ps2float::sub(1.0_f32, lwc1(star)), ps2float::sub(1.0_f32, lwc1(star + 4))), brightness));
        setStar(i);
        swc1(star + 16, ps2float::mul(random_as_float(), scale));
        i += 1;
        star += 28;
        if i >= 200 {
            break;
        }
    }
    let mut offset = 0;
    let mut fire = g::spacefires;
    loop {
        swc1(fire, ps2float::add(lwc1(g::spacefirestart), ps2float::mul(ps2float::mul(random_as_float(), scale), 4.0_f32)));
        let unit = ps2float::mul(random_as_float(), scale);
        let third = lwc1(g::spacefirestart + 8);
        swc1(fire + 4, ps2float::add(lwc1(g::spacefirestart + 4), ps2float::add(unit, unit)));
        swc1(offset + (g::spacefires + 8), third);
        swc1(fire + 16, ps2float::mul(ps2float::add(ps2float::mul(random_as_float(), scale), lwc1(0x3ad63c)), lwc1(0x3ad640)));
        let red = newrnd();
        let green = newrnd();
        let blue = newrnd();
        sw(
            offset + (g::spacefires + 20),
            ((((red as i32 as u32) % 28) as i32 + 100) << 16) + ((((green as i32 as u32) % 28) as i32 + 50) << 8) + 50 + ((blue as i32 as u32) % 28) as i32,
        );
        swc1(fire + 12, ps2float::mul(random_as_float(), scale));
        fire += 24;
        offset += 24;
        if fire >= g::vif_load_mmtx_36c1a0 {
            break;
        }
    }
}

/// Check the map maker's items for the game modes in `modes` (flags 1 to start points, 2 for two teams with start points, 16 for the flags) and make the message for the first
/// one missing: the items of kind 14 are counted (s4: exactly one is wanted), those of kind 15 (flags), and the four teams' items (1 to 4 against 5 to 8); the result has 4, 1
/// more for one item of kind 14, 2 more for two teams complete and 16 more for a flag; the message (the language's texts at +3664, +3672, +3668) is started when a wanted mode is
/// missing (@ 0x247a00).
pub fn mmCheckGameModes(modes: i32) -> i32 {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let mut message = 0;
    let mut flags_found = 0;
    let mut starts_found = 0;
    sw(sp + 32, 4);
    let mut object = lw(g::mapobjects);
    while object != 0 {
        let kind = lw(object);
        let next = lw(object + 56);
        if kind == 1 {
            let mut item = lw(object + 52);
            if item != 0 {
                loop {
                    let item_kind = lhu(item + 2);
                    if item_kind == 14 {
                        starts_found += 1;
                    } else if item_kind == 15 {
                        flags_found += 1;
                    }
                    item = lw(item + 4);
                    if item == 0 {
                        break;
                    }
                }
            }
        }
        object = next;
    }
    let mut i = 0;
    loop {
        let counted = i + 1;
        sw(sp + (i << 2), mmItemCount(counted));
        sw(sp + 16 + (i << 2), mmItemCount(i + 5));
        i = counted;
        if i >= 4 {
            break;
        }
    }
    if starts_found != 1 {
        let language = lw(g::cur_language);
        if (modes & 1) != 0 {
            message = lw(lw((language << 2) + g::languages) + 3664);
        }
    } else {
        sw(sp + 32, 5);
    }
    let mut teams = 0;
    let mut i = 0;
    loop {
        if lw(sp + (i << 2)) != 0 && lw(sp + 16 + (i << 2)) != 0 {
            teams += 1;
        }
        i += 1;
        if i >= 4 {
            break;
        }
    }
    let few = teams < 2;
    if few {
        let language = lw(g::cur_language);
        if (modes & 2) != 0 {
            message = lw(lw((language << 2) + g::languages) + 3672);
        }
    } else {
        sw(sp + 32, lw(sp + 32) | 2);
    }
    let mut wants_flags = false;
    if !few {
        let result = lw(sp + 32);
        if flags_found > 0 {
            sw(sp + 32, result | 16);
        } else {
            wants_flags = true;
        }
    } else {
        wants_flags = true;
    }
    if wants_flags {
        let wanted = modes & 16;
        let language = lw(g::cur_language);
        if wanted != 0 {
            message = lw(lw((language << 2) + g::languages) + 3668);
        }
    }
    if message != 0 {
        mmStartMessage(message, 0x6464647f, 0);
    }
    lw(sp + 32)
}

/// Swing the prop `prop` about the axis `axis` (0 to 2): its angle (the three at +8 of its record, degrees) goes on by its swing speed (+20, kept to within 90) times `rate`,
/// slowed or helped by where the swing is (a quarter wave about 90, 180 and 270), kept within 0 and 360, and, when `limit` is not 0, between `limit` and 360 less `limit`;
/// the swing speed becomes what the angle moved (@ 0x27a228).
pub fn swingProp(prop: i32, axis: i32, speed: f32, rate: f32, limit: f32) {
    let ninety = 9e+01_f32;
    let at = axis << 2;
    let record = lw(prop + 352);
    let speeds = record + 20;
    let slot = speeds + at;
    let mut moving = lwc1(slot);
    if ninety < moving {
        swc1(slot, ninety);
        moving = ninety;
    }
    let low = -9e+01_f32;
    let angles = record + 8;
    if moving < low {
        swc1(slot, low);
        moving = low;
    }
    let angle_at = angles + at;
    let mut before = lwc1(angle_at);
    let angle = ps2float::add(before, ps2float::mul(moving, rate));
    swc1(angle_at, angle);
    let pull;
    'pull: {
        if !(angle < ninety) {
            let wave;
            'wave: {
                if ninety <= angle && angle < 1.8e+02_f32 {
                    wave = dpsub(dpmul(fptodp(ps2float::sub(angle, ninety)), ld(0x3a85e8)), ld(0x3a85f0));
                    break 'wave;
                }
                let half = 1.8e+02_f32;
                let again = lwc1(angles + at);
                if half <= again && again < 2.7e+02_f32 {
                    wave = dpadd(dpmul(fptodp(ps2float::sub(again, half)), ld(0x3a85f8)), 0x4022000000000000_i64);
                    break 'wave;
                }
                pull = ps2float::sub(3.6e+02_f32, lwc1(angles + at));
                break 'pull;
            }
            pull = dptofp(wave);
        } else {
            pull = ps2float::sub(ps2float::sub(ninety, angle), ninety);
        }
    }
    let angle_at = angles + at;
    let full = 3.6e+02_f32;
    let turned = ps2float::add(lwc1(angle_at), ps2float::mul((pull).pdiv(9e+01_f32), speed));
    swc1(angle_at, turned);
    if full <= turned {
        before = ps2float::sub(before, full);
        swc1(angle_at, ps2float::sub(turned, full));
    }
    let now = lwc1(angle_at);
    if now < 0.0_f32 {
        before = ps2float::add(before, full);
        swc1(angle_at, ps2float::add(now, full));
    }
    if !(limit == 0.0_f32) {
        let now = lwc1(angle_at);
        if limit < now && dpcmp(fptodp(now), ld(0x3a8600)) < 0 {
            swc1(angle_at, limit);
        }
    }
    if !(limit == 0.0_f32) {
        let now = lwc1(angles + at);
        let upper = ps2float::sub(3.6e+02_f32, limit);
        if now < upper && dpcmp(fptodp(now), ld(0x3a8608)) >= 0 {
            swc1(angles + at, upper);
        }
    }
    swc1(speeds + at, ps2float::sub(lwc1(angles + at), before));
}
