//! Resets that only restart, random spawns, the lists of bullets, the drawing of message pages and of the credits, and the swinging signs.

use crate::*;

/// Restart the enemies (@ 0x2ba040).
pub fn enemyReset() {
    enemyRestart();
}

/// Restart the GUI (@ 0x22e420).
pub fn guiReset() {
    guiRestart();
}

/// Restart the special effects (@ 0x2a9848).
pub fn specialfxReset() {
    specialfxRestart();
}

/// Restart the starfield (@ 0x2a8b38).
pub fn starfieldReset() {
    starfieldRestart();
}

/// A random pad for the character `chr` that characters of the computer spawn on (flag 1 of the pad) (@ 0x228390).
pub fn lvGetRandomAiSpawn(chr: i32) -> i32 {
    lvGetRandomPad(chr, 1)
}

/// A random pad for the character `chr` that players spawn on (flag 2 of the pad) (@ 0x228370).
pub fn lvGetRandomPlayerSpawn(chr: i32) -> i32 {
    lvGetRandomPad(chr, 2)
}

/// Make the matrix number `index` of the model's matrices (at 0x3afe7c) the animation's model view (multiplied with the one below) (@ 0x215700).
pub fn anim_LoadMatrix(index: i32) {
    anim_SetModelview(lw(0x3afe7c) + (index << 6), 2);
}

/// Undo the last change of the map maker: load the saved copy of the map (the temporary buffer) again, keeping the level, and set flag 0x1000 of the edit flags with 256 cleared (@ 0x248370).
pub fn mmUndoMap() {
    mmLoadMapFile(lw(g::mmtempbuffer), 1);
    sw(g::editflags, lw(g::editflags) & -257 | 0x1000);
}

/// Get memory for the particle groups (0x93a80 bytes of the level area) and restart them (@ 0x296a68).
pub fn particleReset() {
    let memory = memAlloc(0x93a80, 1);
    sw(g::pgroups, memory);
    particleRestart();
}

/// Draw the sound select page `page` with a blend and its heading (@ 0x2067d0).
pub fn soundselect_pageDraw(page: i32) {
    dlSetBlend(1);
    pagedrawbg(page, 0x399b08);
}

/// Take a bullet of the kind `kind` from the free list of its group (the heads at 0x3b0148) and put it at the head of the group's list; the record is returned (@ 0x289560).
pub fn bulletNew(kind: i32) -> i32 {
    let group = bulletGetGroupType(kind);
    let first = lw((group << 2) + 0x3b0148);
    disconnect(first, group);
    addhead(first, group);
    lw(0x3b0138) + first * 276
}

/// Put the bullet record `bullet` back on the free list of its group (@ 0x2895c8).
pub fn bulletFree(bullet: i32) {
    let kind = lw(bullet);
    let index = ((bullet - lw(0x3b0138)).wrapping_mul(-0x5cc0ed73)) >> 2;
    let group = bulletGetGroupType(kind);
    sw(bullet, 0);
    disconnect(index, group);
    addtail(index, group);
}

/// Restart the in-game menu: the GUI, the menu colours (a background of 64 and a foreground of 0x64646480) and fonts (the small one when more than one player is local), and clear
/// the 4 menus' records (368 bytes at 0x1fb3240) (@ 0x2124a8).
pub fn ingamemenuRestart() {
    guiRestart();
    sw(g::menuitem_bgcol, 64);
    sw(g::menuitem_fgcol, 0x64646480);
    let players = ilinkGetNumLocalPlayers();
    sw(g::menuitem_normalfont, if players >= 2 { 1 } else { 2 });
    ilinkGetNumLocalPlayers();
    sw(g::menuitem_bigfont, 2);
    memset(0x1fb3240, 0, 368);
    sw(0x3afe78, -1);
}

/// Work out the bounds of every part of the model skeleton `skeleton` (80 bytes each, the count in the word at its start) for the kind `kind` (@ 0x25e8c0).
pub fn obCalcBounds(skeleton: i32, kind: i32) {
    let count = lw(skeleton);
    let mut left = count - 1;
    let mut part = skeleton - count * 80;
    if count != 0 {
        loop {
            obCalcPartBounds(part, kind);
            left -= 1;
            part += 80;
            if left == -1 {
                break;
            }
        }
    }
}

/// Draw the message page of the map maker: the box and, once it is open, the text in the buffer at 0x1fc3a00 in the middle in the colour kept at 0x3b002c (@ 0x24c910).
pub fn message_pageDraw_24c910() {
    zoomboxGfx(0);
    let box_at = g::zoomboxes;
    if lw(box_at + 36) != 0 {
        return;
    }
    guiDrawFormattedText(0x1fc3a00, lw(box_at + 20) + 16, lw(box_at + 24) + 3, 1, lw(0x3b002c), 80);
}

/// Draw the message page: the box and, once it is open, the text in the buffer at 0x1fb2528 in the colour kept at 0x3afe30 (@ 0x206b08).
pub fn message_pageDraw_206b08() {
    let frame = Frame::new(16);
    let sp = frame.sp();
    zoomboxGfx(0);
    let box_at = g::zoomboxes;
    if lw(box_at + 36) == 0 {
        let colour = lw(0x3afe30);
        let x = lw(box_at + 20) + 8;
        sw(sp + 4, lw(box_at + 24) + 3);
        sw(sp, x);
        textPrint(sp, sp | 4, 0x1fb2528, colour);
    }
}

/// Draw the message page of the front end: the box of the current definition and, once it is open, its text (the buffer at 1040 bytes a definition from 0x1fc19c0, the colour at
/// +1024 of it) in the middle (@ 0x2312e0).
pub fn message_pageDraw_2312e0() {
    let definition = lw(lw(g::front));
    zoomboxGfx(definition);
    let box_at = lw(lw(g::front)) * 40 + g::zoomboxes;
    if lw(box_at + 36) != 0 {
        return;
    }
    let width = lw(box_at + 28);
    let text = definition * 1040 + 0x1fc19c0;
    let colour = lw(definition * 1040 + 0x1fc1dc4);
    let outline = lw(g::menuitem_bgcol);
    let x = lw(box_at + 20) + ((width + ((width as u32) >> 31) as i32) >> 1);
    let y = lw(box_at + 24) + 3;
    guiDrawFormattedText(text, x, y, 1, colour, outline);
}

/// Delete the props of all the cartridges of the cartridge group (10 of 96 bytes each) (@ 0x295288).
pub fn cartridgesEnd() {
    let mut group = 0;
    loop {
        let next = group + 1;
        let mut left = 9;
        let mut at = group * 976 + g::cartridge_groups;
        loop {
            let prop = lw(at + 64);
            left -= 1;
            if prop != 0 {
                propDelete(prop);
                sw(at + 64, 0);
            }
            at += 96;
            if left < 0 {
                break;
            }
        }
        group = next;
        if group > 0 {
            break;
        }
    }
}

/// The tick of the sign of the boat: its matrix is made unit and it swings about x by the ticks (swing speed 3, 90 degrees as the limit) and the second matrix is turned by the
/// angle (@ 0x279cf8).
pub fn boatsignPropTick(prop: i32) {
    let mut i = 0;
    sceVu0UnitMatrix(lw(lw(prop + 32) + 4));
    if lw(g::updateframesi) > 0 {
        loop {
            i += 1;
            swingProp(prop, 0, 3.0_f32, lwc1(0x3acec0), 9e+01_f32);
            if i >= lw(g::updateframesi) {
                break;
            }
        }
    }
    let state = lw(prop + 352);
    matrixRotX(lw(lw(prop + 32) + 4) + 64, lwc1(state + 8) * lwc1(0x3acec4) / 1.8e+02_f32);
}

/// The tick of the gong: like the boat's sign, with speed 3.5 and a limit of 20 degrees (@ 0x279da0).
pub fn gongPropTick(prop: i32) {
    let mut i = 0;
    sceVu0UnitMatrix(lw(lw(prop + 32) + 4));
    if lw(g::updateframesi) > 0 {
        loop {
            i += 1;
            swingProp(prop, 0, 3.5_f32, lwc1(0x3acec8), 2e+01_f32);
            if i >= lw(g::updateframesi) {
                break;
            }
        }
    }
    let state = lw(prop + 352);
    matrixRotX(lw(lw(prop + 32) + 4) + 64, lwc1(state + 8) * lwc1(0x3acecc) / 1.8e+02_f32);
}

/// Draw the credits: the clip is the window, then the background, the text and the motion blur (`blur_123`); in the first part of the time (before the constant at 0x3ad754) a
/// black box fades the screen (@ 0x2ae6c8).
pub fn creditsGfx() {
    let window = g::window;
    dlSetClip(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24));
    creditsBgGfx();
    creditsTextGfx();
    ipMotionBlur(lw(g::blur_123));
    if !(lwc1(g::creditstime) < lwc1(0x3ad754)) {
        return;
    }
    dlSetBlend(1);
    dlFillRectangle(0, 0, 640, 224, 127);
}
