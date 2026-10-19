//! Gradient boxes and images of the user interface, the screen as a texture, and the animation test.

use crate::*;

/// Draw a four-vertex triangle strip of (colour, x, y) vertices from a frame of memory.
fn gradient_strip(vertices: [(i32, i32, i32); 4]) {
    let frame = Frame::new(64);
    let sp = frame.sp();
    for (i, (colour, x, y)) in vertices.iter().enumerate() {
        let at = sp + 12 * i as i32;
        sw(at, *colour);
        sw(at + 4, *x);
        sw(at + 8, *y);
    }
    dlDrawTriangleStrip(sp, 4, 0);
}

/// A box from (`x0`, `y0`) to (`x1`, `y1`) whose colour 0xrrggbbaa goes from alpha `alpha_a` at the top left to `alpha_b` at the other two corners, the last corner clear (@ 0x22e8b0).
pub fn guiTriGradient(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32, alpha_a: i32, alpha_b: i32) {
    let base = colour & -256;
    let a = base | alpha_a & 255;
    let b = base | alpha_b & 255;
    gradient_strip([(a, x0, y0), (b, x1, y0), (b, x0, y1), (0, x1, y1)]);
}

/// A box whose alpha goes from `alpha_a` on the left to `alpha_b` on the right (@ 0x22e7b0).
pub fn guiHGradient(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32, alpha_a: i32, alpha_b: i32) {
    let base = colour & -256;
    let a = base | alpha_a & 255;
    let b = base | alpha_b & 255;
    gradient_strip([(a, x0, y0), (b, x1, y0), (a, x0, y1), (b, x1, y1)]);
}

/// A box whose alpha goes from `alpha_a` at the top to `alpha_b` at the bottom (@ 0x22e830).
pub fn guiVGradient(x0: i32, y0: i32, x1: i32, y1: i32, colour: i32, alpha_a: i32, alpha_b: i32) {
    let base = colour & -256;
    let a = base | alpha_a & 255;
    let b = base | alpha_b & 255;
    gradient_strip([(a, x0, y0), (a, x1, y0), (b, x0, y1), (b, x1, y1)]);
}

/// Draw image `image` (a texture number from the table at 0x1fc0910) at (`x`, `y`) in a colour; its height is halved (the picture is drawn on a field of half the lines) (@ 0x22e548).
pub fn guiDrawImage(image: i32, colour: i32, x: f32, y: f32) {
    let texture = lw((image << 2) + 0x1fc0910);
    let record = texture * 40 + lw(lw(g::thistexbuf) + 4);
    let height = lhu(record + 2);
    let width = lhu(record);
    dlSelectTextureKick(texture);
    dlTextureRectangleFloat(0, 0, (width - 1) << 4, (height - 1) << 4, colour, x, y, x + width as f32, y + height as f32 * 0.5);
}

/// The frame buffer being drawn is the texture of the next drawing (@ 0x295f88).
pub fn ipSetCurScreenAsTexture() {
    ipSetScreenAsTexture(if lw(g::lastfrbuf) & 1 != 0 { 0 } else { 40 });
}

/// The frame buffer drawn before is the texture of the next drawing (@ 0x295fa0).
pub fn ipSetLastScreenAsTexture() {
    ipSetScreenAsTexture(if lw(g::lastfrbuf) & 1 == 0 { 0 } else { 40 });
}

/// The depth buffer is the texture of the next drawing (@ 0x295fb8).
pub fn ipSetZBAsTexture() {
    ipSetScreenAsTexture(80);
}

/// Colour both lists of chunks of all the bones of a model (@ 0x2606a0).
pub fn obColour(model: i32, colour: i32) {
    partColour(model, 0, 0, colour);
    partColour(model, 0, 1, colour);
}

/// Read the last frame of the animations 3 to 714 (except 565, 586, 607 and 498 to 512) for a character, as a test (@ 0x219c08).
pub fn animTest(chr: i32) {
    let frame = Frame::new(144);
    let out = frame.sp();
    let mut at = g::pAnimations + 12;
    let mut i = 3;
    let mut anim = lw(at);
    loop {
        at += 4;
        if i != 565 {
            if i == 586 {
                i += 1;
            } else {
                if i != 607 && ((i - 498) as u32) >= 15 {
                    extractAnimData(chr, anim, 0, lw(anim + 4) - 1, out, 1.0);
                }
                i += 1;
            }
        } else {
            i += 1;
        }
        if i >= 715 {
            break;
        }
        anim = lw(at);
    }
}

/// The science fiction story: the control panel (state 0) closes and locks door 1, links and unlinks pads and opens doors 2 and 4; then (state 1) when a player is in the
/// game door 3 closes and locks (@ 0x2cc050).
pub fn enemyAIScifiHandler1() {
    let state = lw(g::story_state1);
    if state == 1 {
        let players = lw(g::lvnumplayers);
        let chr_info = lw(g::chrinfo);
        let mut any = false;
        for i in 0..players {
            if lw(i * 0x1210 + chr_info + 484) != 0 {
                any = true;
            }
        }
        if any {
            doorClose(3);
            doorLock(3);
            linkDisable(padGetLinkNum(1075, 1076));
            sw(g::story_state1, 2);
        }
    } else if state == 0 && propControlPanelActivated() != 0 {
        doorClose(1);
        doorLock(1);
        linkDisable(padGetLinkNum(1215, 1216));
        doorOpen(2);
        doorLock(2);
        linkEnable(padGetLinkNum(1141, 1381));
        doorOpen(4);
        doorLock(4);
        linkEnable(padGetLinkNum(1237, 1152));
        sw(g::story_state1, 1);
    }
}
