//! Screen effects that draw the screen as a texture (motion blur, depth view, defocus), and icons drawn as a textured rectangle.

use crate::*;

/// Blur the picture with the one before: the last frame's screen is drawn over the window with the strength `strength` as the alpha (@ 0x2960b8).
pub fn ipMotionBlur(strength: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    ipSetLastScreenAsTexture();
    dlSetBlend(1);
    let window = g::window;
    let x0 = lw(g::window);
    let y1 = lw(window + 24);
    let y0 = lw(window + 20);
    let x1 = lw(window + 4);
    sw(sp, strength & 255 | 0x7f7f7f00);
    dlTextureRectangle(x0, y0, x1, y1, x0 << 4, y0 << 4, x1 << 4, y1 << 4);
}

/// Show the depth buffer: it is drawn as a texture in squares of 32 by 32 pixels (20 by 7 of them, alternate rows and columns taking the other half of the texture) (@ 0x296120).
pub fn ipViewZB() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    ipSetZBAsTexture();
    dlSetBlend(1);
    let mut row = 0;
    let mut other_row = row ^ 1;
    loop {
        let mut column = 0;
        let top = row << 5;
        let v1 = other_row << 5 | 31;
        let next_row = row + 1;
        let mut other_column = column ^ 1;
        loop {
            let next_column = column + 1;
            let u0 = other_column << 9;
            let x0 = column << 5;
            sw(sp, 0x7f7f7f4f);
            let u1 = u0 | 496;
            dlTextureRectangle(x0, top, next_column << 5, next_row << 5, u0, other_row << 9, u1, v1 << 4);
            column = next_column;
            other_column = column ^ 1;
            if column >= 20 {
                break;
            }
        }
        row = next_row;
        other_row = row ^ 1;
        if row >= 7 {
            break;
        }
    }
}

/// Defocus the picture: the screen is drawn over itself three times, moved a pixel right, down and right again, faintly (@ 0x295fc0).
pub fn ipDefocus() {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let window = g::window;
    ipSetCurScreenAsTexture();
    let colour = 0x7f7f7f3f;
    dlSetBlend(1);
    let mut x1 = lw(window + 4);
    let mut left = 0;
    loop {
        left -= 1;
        let x0 = lw(g::window);
        let y0 = lw(window + 20);
        let y1 = lw(window + 24);
        sw(sp, colour);
        dlTextureRectangle(x0 + 1, y0, x1 + 1, y1, x0 << 4, y0 << 4, x1 << 4, y1 << 4);
        let y0 = lw(window + 20);
        let y1 = lw(window + 24);
        let x1b = lw(window + 4);
        let x0 = lw(g::window);
        sw(sp, colour);
        dlTextureRectangle(x0, y0 + 1, x1b, y1 + 1, x0 << 4, y0 << 4, x1b << 4, y1 << 4);
        let x1c = lw(window + 4);
        let x0 = lw(g::window);
        let y0 = lw(window + 20);
        let y1 = lw(window + 24);
        sw(sp, colour);
        dlTextureRectangle(x0 + 1, y0, x1c + 1, y1, x0 << 4, y0 << 4, x1c << 4, y1 << 4);
        if left < 0 {
            break;
        }
        x1 = lw(window + 4);
    }
}

/// Draw a button image: texture `image` of the text graphics (a rectangle of 24 by 11 at (`x`, `y`), the whole texture), bright when `enabled` is not 0, dim otherwise (@ 0x22e428).
pub fn guiDrawButton(x: i32, y: i32, image: i32, enabled: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let texture = lw((image << 2) + g::texnum_text_gfx);
    let record = texture * 40 + lw(lw(g::thistexbuf) + 4);
    let height = lhu(record + 2);
    let width = lhu(record);
    dlSelectTextureKick(texture);
    sw(sp, if enabled != 0 { 0x7f7f7f7f } else { 0x5f5f5f3f });
    dlTextureRectangle(x, y, x + 24, y + 11, 0, 0, width << 4, height << 4);
}

/// Draw the icon of the bag the first player's character carries (kept at +484 of its character info): the texture at 0x3b019c, centred on the right of the screen at height `y`,
/// in the team's colour (story) or white, flashing in alpha (@ 0x2b1ac0).
pub fn hudBagIconGfx(y: i32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let carried = lw(lw(lw(lw(g::currentplayer) + 384) + 352) + 484);
    let texture = lw(0x3b019c);
    if carried != 0 && texture >= 0 {
        let info = lw(carried + 352);
        let centre = lw(g::window) + (32 >> (lw(g::window + 88) & 31));
        let dither = dlSetDitherMatrix(2);
        let colour = if (lw(g::curgameparms + 80) & 1) == 0 { 0x7f7f7f80 } else { lw((lw(info + 16) << 3) + (g::storydata + 68)) };
        let texture = lw(0x3b019c);
        let record = texture * 40 + lw(lw(g::thistexbuf) + 4);
        let colour = colour & -256 | lbu(g::flashalpha);
        let width = lhu(record);
        let height = lhu(record + 2);
        dlSelectTextureKick(texture);
        let left = centre + 27 - ((width as u32) >> 1) as i32;
        sw(sp, colour);
        dlTextureRectangle(left, y, left + width, y + (height >> 1), 0, 0, width << 4, height << 4);
        dlSetDitherMatrix(dither);
    }
}
