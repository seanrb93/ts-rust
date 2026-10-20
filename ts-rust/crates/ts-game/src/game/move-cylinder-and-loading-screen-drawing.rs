//! Drawing the cylinder that shows a character's move test, and the loading screen.

use crate::*;

/// Draw a cylinder at (`x`, `y`, `z`) of height `height` and radius `radius` as a display list (784 bytes of the debug memory): eight sides, each a quad from the angle
/// `i` * 45 degrees to the next, from `y` up to `y` + `height`, then the colours of the eight sides (the bits of the side's number choose the red, green and blue). The clip tag
/// is turned off for it (@ 0x27dff8).
pub fn moveCylGfx(x: f32, y: f32, z: f32, height: f32, radius: f32) {
    let list = memdbAlloc(784);
    dlSetClipTag(0);
    sb(lw(g::gfxtag) + 3, 48);
    sw(list, 0x6c088000);
    let tag = lw(g::gfxtag);
    sw(tag + 4, list);
    let mut at = list + 4;
    sh(tag, 49);
    let mut word = at + 4;
    sw(g::gfxtag, tag + 16);
    let mut next = word;
    let mut i = 0;
    loop {
        sw(word + 4, 1042);
        sw(word - 4, if (i ^ 7) == 0 { 0x8004 } else { 4 });
        i += 1;
        sw(word, 0x30264000);
        at += 16;
        sw(next + 8, 0);
        next += 16;
        word = next;
        if i >= 8 {
            break;
        }
    }
    let to_radians = lwc1(0x3acfc4);
    let half_turn = 1.8e+02_f32;
    sw(at, 0x6c208018);
    at += 4;
    let one = 1.0_f32;
    let mut first = at + 8;
    let mut i = 0;
    let mut second = first;
    loop {
        let from = i as f32;
        let to = i + 1;
        at += 64;
        i = to;
        let from_angle = from * 3.6e+02_f32 * 0.125_f32 * to_radians / half_turn;
        let sin0 = sinf(from_angle);
        let cos0 = cosf(from_angle);
        let to_angle = to as f32 * 3.6e+02_f32 * 0.125_f32 * to_radians / half_turn;
        let sin1 = sinf(to_angle);
        let cos1 = cosf(to_angle);
        swc1(second - 4, y);
        let x0 = x + sin0 * radius;
        let z0 = z + cos0 * radius;
        swc1(second - 8, x0);
        let x1 = x + sin1 * radius;
        swc1(first, z0);
        swc1(first + 4, one);
        let z1 = z + cos1 * radius;
        swc1(second + 8, x0);
        swc1(first + 12, y + height);
        swc1(second + 16, z0);
        swc1(second + 20, one);
        swc1(first + 24, x1);
        swc1(second + 28, y);
        swc1(first + 32, z1);
        swc1(second + 40, x1);
        swc1(second + 48, z1);
        swc1(second + 36, one);
        swc1(second + 44, y + height);
        swc1(second + 52, one);
        second += 64;
        first = second;
        if i >= 8 {
            break;
        }
    }
    let mut i = 0;
    sw(at, 0x6e20c058);
    at += 4;
    let mut colours = at + 4;
    loop {
        let green = if (i & 1) != 0 { -0x7f010000 } else { -0x80000000 };
        let blue = i & 4;
        let with_red = if (i & 2) == 0 { green } else { green | 0xff00 };
        i += 1;
        at += 16;
        let colour = if blue == 0 { with_red } else { with_red | 255 };
        sw(colours + 8, colour);
        sw(colours - 4, colour);
        sw(colours, colour);
        sw(colours + 4, colour);
        colours += 16;
        if i >= 8 {
            break;
        }
    }
    sw(at, 0x14000000);
    dlSetClipTag(1);
}

/// Draw the loading screen: the window of the current player is set, the three pictures of the loading panel (the textures at 0x1fbd078, 100 bytes apart) are drawn one under
/// the other, and the language's loading text (at +3416) is put at the bottom in the middle (@ 0x22a190).
pub fn loadingGfx() {
    let frame = Frame::new(2064);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
    windowGfx();
    let mut put = sp + 2016;
    let mut name = 0x1fbd078;
    let mut left = 2;
    loop {
        let texture = preloadGetTexName(name);
        left -= 1;
        name += 100;
        sw(put, lw(texture + 20));
        put += 4;
        if left < 0 {
            break;
        }
    }
    dlSetClipTag(1);
    let colour = 0x7f7f7f7f;
    dlSelectTextureKick(lw(sp + 2016));
    sw(sp, colour);
    dlTextureRectangle(64, 10, 576, 74, 0, 0, 0x2000, 2048);
    dlSelectTextureKick(lw(sp + 2020));
    sw(sp, colour);
    dlTextureRectangle(64, 74, 576, 138, 0, 0, 0x2000, 2048);
    dlSelectTextureKick(lw(sp + 2024));
    sw(sp, colour);
    dlTextureRectangle(64, 138, 576, 170, 0, 0, 0x2000, 1024);
    let language = lw((lw(g::cur_language) << 2) + g::languages);
    sprintf(&[(sp + 16) as i64, lw(language + 3416) as i64, language as i64]);
    textSelect(1);
    sb(g::fontflags, lbu(g::fontflags) | 1);
    textSize(sp + 2032, sp + 2036, sp + 16);
    let width = lw(sp + 2032);
    let window = g::window;
    let y = lw(window + 24) - lw(sp + 2036) - 10;
    sw(sp + 2040, lw(window + 8) - ((width + ((width as u32) >> 31) as i32) >> 1));
    sw(sp + 2044, y);
    dlSetBlend(1);
    textPrint(sp + 2040, sp + 2044, sp + 16, 0x7f7f7f7f);
    sb(g::fontflags, lbu(g::fontflags) & 254);
}
