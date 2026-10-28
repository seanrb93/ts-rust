//! Pages drawn as zoom boxes, the cheats that are unlocked, and choosing a character on the sign-on page.

use crate::*;

/// Draw the colour page of the map maker: a zoom box over the whole page (@ 0x251888).
pub fn mapmakercolour_pageDraw() {
    zoomboxDraw(54, 50, 313, 171, 0x2880, 0x7f7f7f7f);
}

/// Draw zoom box `index` (records of 40 bytes at `zoomboxes`: the fill at +16, the corner at +20 and +24, the size at +28 and +32) (@ 0x22f0a0).
pub fn zoomboxGfx(index: i32) {
    let record = index * 40 + g::zoomboxes;
    let x0 = lw(record + 20);
    let y0 = lw(record + 24);
    let fill = lw(record + 16);
    let x1 = x0 + lw(record + 28) - 1;
    let y1 = y0 + lw(record + 32) - 1;
    zoomboxDraw(x0, y0, x1, y1, fill, 0x7f7f7f7f);
}

/// Work out which of the 15 cheats are unlocked (a bit each in `unlockedcheats`): with flag 3 of the player array all, else those whose list of items is unlocked; in the level
/// 102 only the cheats 0x2007 are allowed (@ 0x225508).
pub fn cheatsUpdate() {
    let mut defs = g::cheatdefs + 8;
    let mut cheat = 0;
    sw(g::unlockedcheats, 0);
    loop {
        let mut unlocked = lw(g::unlockedcheats);
        let mut set = true;
        if lb(g::pcarray + 3) == 0 {
            let available = unlockedset(lw(defs));
            unlocked = lw(g::unlockedcheats);
            if available == 0 {
                set = false;
            }
        }
        if set {
            sw(g::unlockedcheats, unlocked | 1 << (cheat & 31));
        }
        cheat += 1;
        defs += 12;
        if cheat >= 15 {
            break;
        }
    }
    if lw(g::lvlevelnum) == 102 {
        sw(g::allowedcheats, 0x2007);
    }
}

/// Draw the zoom box that opens on the sign-on page: from the box at 0x1fc33a0 to the final box (80, 56 to 559, 167) in ten steps by `0x3aff98` (the step counts down; -1 becomes
/// -2 after the last) (@ 0x23f288).
pub fn soZoomGfx() {
    let bottom_now = lw(0x1fc33ac);
    let left_now = lw(0x1fc33a0);
    let top_now = lw(0x1fc33a4);
    let right_now = lw(0x1fc33a8);
    let step = 9 - lw(0x3aff98);
    let bottom = (167 - bottom_now).wrapping_div(10) * step + bottom_now;
    let left = (80 - left_now).wrapping_div(10) * step + left_now;
    let top = (56 - top_now).wrapping_div(10) * step + top_now;
    let right = (559 - right_now).wrapping_div(10) * step + right_now;
    zoomboxDraw(left, top, right, bottom, 0x2880, 0x7f7f7f80);
    if lw(0x3aff98) == -1 {
        sw(0x3aff98, -2);
    }
}

/// Move the character of the sign-on that the front end is on one on in the character order (`right` or not) to the next playable one (kind 0), or toggle between the two
/// characters of the story part kept at 0x1fc1218 (@ both fxSelectChar functions).
fn select_character(kind: i32, right: bool) {
    let sign_on = signonGet(lw(lw(g::front)));
    let record = lw(g::signonarray) + sign_on * 2936 + 32;
    if kind == 0 {
        // (the order is the game's 64 characters, then the ones mods added, all playable)
        let extra = crate::manual::chartable::added_characters();
        let total = 64 + extra;
        let order = |i: i32| if i < 64 { lbu(i + g::characterorder) } else { crate::manual::chartable::GAME_CHARACTERS + i - 64 };
        let current = lw(record + 84);
        let mut at = 0;
        if order(0) != current {
            at += 1;
            loop {
                if at >= total {
                    break;
                }
                if order(at) == current {
                    break;
                }
                at += 1;
            }
        }
        let mut candidate = if right { at + 1 } else { at - 1 };
        let chosen;
        loop {
            let wrapped = if right {
                if candidate >= total { 0 } else { candidate }
            } else if -1 >= candidate {
                total - 1
            } else {
                candidate
            };
            if wrapped >= 64 || chrPlayable(order(wrapped)) != 0 {
                chosen = wrapped;
                break;
            }
            candidate = if right { wrapped + 1 } else { wrapped - 1 };
        }
        let chr = order(chosen);
        if lw(record + 84) != chr {
            sw(record + 84, chr);
        }
    } else {
        let story = (lw(0x1fc1218) << 3) + g::storydata;
        let mut chr = lhu(story + 4);
        if lw(record + 84) == chr {
            chr = lhu(story + 6);
        }
        sw(record + 84, chr);
    }
}

/// Go to the previous character on the sign-on page (@ 0x238268).
pub fn fxSelectCharLeft(kind: i32) {
    select_character(kind, false);
}

/// Go to the next character on the sign-on page (@ 0x238388).
pub fn fxSelectCharRight(kind: i32) {
    select_character(kind, true);
}
