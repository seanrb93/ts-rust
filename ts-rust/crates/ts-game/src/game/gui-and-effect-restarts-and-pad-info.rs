//! Restarting the GUI textures and the special effects, the sign-on list, the game options, the knockout set-up, the pad info text, and score texts.

use crate::*;

/// Find the texture numbers of the GUI: the 5 of the text pictures (names 38 bytes apart ending at 0x1fc0368, into `texnum_text_gfx`) and the 37 of the GUI images (names from
/// 0x1fc0390, numbers from 0x1fc0910; images 24 to 35 are left out on every level but 103) (@ 0x22e350).
pub fn guiRestart() {
    let mut put = g::texnum_text_gfx + 16;
    let mut name = 0x1fc0368;
    let mut left = 4;
    loop {
        let texture = preloadGetTexName(name);
        left -= 1;
        name -= 38;
        sw(put, lw(texture + 20));
        put -= 4;
        if left < 0 {
            break;
        }
    }
    let mut level = lw(g::lvlevelnum);
    let mut put = 0x1fc0910;
    let mut name = 0x1fc0390;
    let mut i = 0;
    loop {
        if level != 103 && ((i - 24) as u32) < 12 {
            i += 1;
        } else {
            let texture = preloadGetTexName(name);
            level = lw(g::lvlevelnum);
            sw(put, lw(texture + 20));
            i += 1;
        }
        put += 4;
        name += 38;
        if i >= 37 {
            break;
        }
    }
}

/// Delete the sign-on `index` (a record of 2936 bytes in `signonarray` and 8 in `signoninfo`; 143 of them): the players' sign-ons are cleared if they were it and moved down by one
/// if they were above it, the records above it are moved down, and the flag at +0x66808 of the array is cleared (@ 0x2207b0).
pub fn signonDelete(index: i32) {
    let mut i = 0;
    loop {
        let current = signonGet(i);
        if current != index {
            if index < current {
                signonSet(i, current - 1);
            }
        } else {
            signonSet(i, -1);
        }
        i += 1;
        if i >= 4 {
            break;
        }
    }
    if index < 143 {
        let count = 143 - index;
        let records = index * 2936 + lw(g::signonarray);
        memcpy(records, records + 2936, count * 2936);
        let at = index << 3;
        let infos = g::signoninfo;
        memcpy(at + infos, at + (infos + 8), count << 3);
    }
    sb(lw(g::signonarray) + 0x66808, 0);
}

/// Set the game options to their starts: the spawn gun 1, no handicaps and no bag shoot; and the 8 records of the options (32 bytes at 0x1fc2c98) cleared and given their starts
/// (1 at +0, +4 and +8, and for the records 1 to 6 the values that suit the mode) (@ 0x238e38).
pub fn gameoptionsMake() {
    sw(g::gv_spawngun, 1);
    sw(g::gv_handicaps, 0);
    sw(g::gv_bagshoot, 0);
    let mut i = 0;
    loop {
        let record = (i << 5) + 0x1fc2c98;
        memset(record, 0, 32);
        sw(record + 8, 1);
        sw(record, 1);
        sw(record + 4, 1);
        sw(record + 28, 0);
        sw(record + 24, 0);
        match i - 1 {
            0 => {
                sw(record, 0);
                sw(record + 12, 5);
            }
            1 | 4 => {
                sw(record + 12, 0);
                sw(record + 20, 10);
            }
            2 => {
                sw(record + 12, 0);
                sw(record + 16, 20);
                sw(record, 0);
            }
            3 | 5 => {
                sw(record + 12, 0);
                sw(record + 20, 0);
            }
            _ => {}
        }
        i += 1;
        if i >= 8 {
            break;
        }
    }
    mode_presets_x();
}

/// Set up the knockout game: from the records of the level's set-up (48 bytes each) that pass the create flags and have kind 1, up to 20 of them, the first one's value at +4 is kept
/// (at 0x3afef4) and the value at +44 of each in the list at 0x1fbb9d8 (the count at 0x3afeec) (@ 0x21d9b8).
pub fn knockoutSetup() {
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    let mut i = 0;
    sw(0x3afeec, 0);
    let total = lw(setup + 20);
    sw(0x3afef4, 0);
    if total > 0 {
        let mut offset = 0;
        let mut list = lw(setup + 16);
        loop {
            let record = list + offset;
            let passed = lvCheckCreateFlags(lw(record));
            if passed != 0 && lw(record + 8) == 1 {
                let mut count = lw(0x3afeec);
                if lw(0x3afef4) == 0 {
                    sw(0x3afef4, lw(record + 4));
                    count = lw(0x3afeec);
                }
                sw((count << 2) + 0x1fbb9d8, lw(record + 44));
                sw(0x3afeec, count + 1);
            }
            let total = lw(setup + 20);
            i += 1;
            offset += 48;
            if i >= total {
                break;
            }
            if lw(0x3afeec) >= 20 {
                break;
            }
            list = lw(setup + 16);
        }
    }
    sw(0x3afef0, 0);
}

/// Restart the special effects: clear the 56 groups of effects (92 bytes each, 0x1420 in all) and the 14 spawn groups (32 bytes each), mark them free (-1 at +0) and find the
/// texture numbers of the two effects' pictures (kept at 0x3b0188 and 0x3b018c) (@ 0x2a9780).
pub fn specialfxRestart() {
    let groups = g::fxgroups;
    memset(groups, 0, 0x1420);
    let mut at = groups + 0x13c4;
    let mut left = 55;
    loop {
        sw(at, -1);
        left -= 1;
        at -= 92;
        if left < 0 {
            break;
        }
    }
    let spawn = g::spawngroups;
    memset(spawn, 0, 448);
    let mut at = spawn + 416;
    let mut left = 13;
    loop {
        sw(at, -1);
        left -= 1;
        at -= 32;
        if left < 0 {
            break;
        }
    }
    let first = preloadGetTexName(0x3a9690);
    sw(0x3b0188, lw(first + 20));
    let second = preloadGetTexName(0x3a96b0);
    sw(0x3b018c, lw(second + 20));
}

/// Draw the test map page `page`: its background, a marker (image 18 when the entry is on, else 19) before each of the four entries (boxes 7 to 10, the flags at 0x3b0040), and
/// for the page's current box (2 to 5) when its entry is off the language's text at +3832 beside the box (@ 0x24cea0).
pub fn testmap_pageDraw(page: i32) {
    pagebackgroundGfx_24c670(page);
    let mut i = 0;
    loop {
        let found = frontGetBox(page, (i + 7) & 0xffff);
        let on = lbu(i + 0x3b0040);
        let y = (lhu(found + 2) + 2) as f32;
        let x = (lhu(found) - 20) as f32;
        let colour = if on != 0 { 0x800080 } else { -0x7fffff80 };
        let image = if on != 0 { 18 } else { 19 };
        i += 1;
        guiDrawImage(image, colour, x, y);
        if (i as u32) >= 4 {
            break;
        }
    }
    let current = lhu(page + 4);
    let entry = current - 2;
    if (entry as u32) < 4 && lbu(entry + 0x3b0040) == 0 {
        let found = frontGetBox(page, current);
        let y = lhu(found + 2);
        let fg = lw(g::menuitem_fgcol);
        let x = lhu(found) + 184;
        let bg = lw(g::menuitem_bgcol);
        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3832);
        textOutline(x, y, text, fg, bg);
    }
}

/// Draw the text about the pad `pad` in the top right of the window: its number, place (4 floats) and flags (@ 0x264d00).
pub fn padInfoGfx(pad: i32) {
    let frame = Frame::new(1136);
    let sp = frame.sp();
    let y = lw(g::window + 20) + 60;
    sw(sp + 1120, lw(g::window) + 360);
    sw(sp + 1124, y);
    padFlagsToString(lw(pad + 4), 8, sp + 1008);
    let a = fptodp(lwc1(pad + 12));
    let b = fptodp(lwc1(pad + 16));
    let c = fptodp(lwc1(pad + 20));
    let d = fptodp(lwc1(pad + 24));
    sprintf(&[sp as i64, 0x3a2798, lw(pad) as i64, a, b, c, d, (sp + 1008) as i64]);
    textPrint(sp + 1120, sp + 1124, sp, 0x7f7f7f7f);
}

/// The text of the score `score` of the challenge `challenge`, by its kind (the halfword at +2 of its score condition): 1 a time, 2 a number with the language's text at +2636,
/// 3 a percentage (the score is in 16.16, with the text at +2640), 4 a number scaled by 1/65536, else a plain number (@ 0x21f168).
pub fn challengeFormatScore(challenge: i32, score: i32) -> i32 {
    let condition = challengeGetScoreCondition(challenge);
    let kind = lhu(condition + 2);
    if kind == 2 {
        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 2636);
        return vafmt(&[(g::gameranksizechr + 4) as i64, score as i64, text as i64]);
    }
    if kind >= 3 {
        if kind == 3 {
            let percent = ps2float::mul(ps2float::mul(score as f32, 1.5258789e-05_f32), 1e+02_f32);
            let texts = lw((lw(g::cur_language) << 2) + g::languages);
            let wide = fptodp(percent);
            return vafmt(&[0x3a0928, lw(texts + 2640) as i64, wide]);
        }
        if kind == 4 {
            return vafmt(&[0x3ae5b0, ps2float::mul(score as f32, 1.5258789e-05_f32) as i32 as i64]);
        }
    } else if kind == 1 {
        return gameFormatTime(score);
    }
    vafmt(&[0x3ae5b0, score as i64])
}

/// The tick of the effect where a prop turns as it spawns, for the effects group `group`: the prop's state (+44) gets a sine of the group's time (0 before it starts); the prop is
/// deleted after 1.5 and the group freed after the time at 0x3ad704 (@ 0x2acb78).
pub fn specialfxRotateSpawnTick(group: i32) {
    let record = group * 92 + g::fxgroups;
    let prop = lw(record + 56);
    if prop != 0 {
        let time = lwc1(record + 48);
        let state = lw(prop + 352);
        if !(time < 0.0_f32) {
            let wide = fptodp((time).pdiv(1.5_f32));
            let turned = dpadd(dpmul(dpmul(wide, ld(0x3a9758)), ld(0x3a9760)), 0x3fe0000000000000_i64);
            let index = dptoli(turned);
            swc1(state + 44, lwc1(((index & 2047) << 2) + g::g_SinTable));
        } else {
            swc1(state + 44, 0.0_f32);
        }
        let groups = g::fxgroups;
        if 1.5_f32 < lwc1(groups + group * 92 + 48) {
            let slot = group * 92 + (groups + 56);
            specialPropDelete(lw(slot));
            sw(slot, 0);
        }
    }
    if !(lwc1(0x3ad704) < lwc1(group * 92 + (g::fxgroups + 48))) {
        return;
    }
    specialfxFree(group);
}

/// Get and clear the memory of the pads (0x10680 bytes), the links (0x157c0) and the extra pad data (0x10680); on level 100 there are none (all the counts are cleared), else
/// the pad file is added to the files to preload (@ 0x264130).
pub fn padPreload() {
    let pads = memAlloc(0x10680, 1);
    sw(g::levelpads, pads);
    memset(pads, 0, 0x10680);
    let links = memAlloc(0x157c0, 1);
    sw(g::levellinks, links);
    memset(links, 0, 0x157c0);
    let extra = memAlloc(0x10680, 1);
    sw(g::levelpadsextra, extra);
    memset(extra, 0, 0x10680);
    if lw(g::lvlevelnum) == 100 {
        sw(g::numpads, 0);
        sw(g::numlinks, 0);
        sw(0x3b00a0, 0);
        sw(0x3b00a4, 0);
        sw(0x3b00a8, 0);
        sw(g::numhalls, 0);
        return;
    }
    preloadAddFile(padFileInName(), 0);
}
