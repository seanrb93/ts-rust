//! The Options page of the MapMaker (`settings_pageTick`): the Grid Intensity slider works like the sliders of the other option pages (X
//! starts changing it, left and right then move it and show their arrows, X or triangle ends it).  The Map Settings page
//! (`mapsettings_pageTick`) gets a new row, No Limit (Yes / No), which takes away the limit of what a map can hold: the FREE bar is not
//! drawn then and no placing is refused for the cost.

use crate::*;
use std::sync::atomic::{AtomicBool, Ordering};

const MENU_AT: i32 = 0x240_0000 + 0xd000;
/// the menu of the Map Settings page, with its new row
const SET_MENU_AT: i32 = 0x240_0000 + 0xd300;
const GV_AT: i32 = 0x240_0000 + 0xd400;
const LIST_AT: i32 = 0x240_0000 + 0xd440;
const TEXT_AT: i32 = 0x240_0000 + 0xd480;
/// the Tile Set row of the Map Settings page: a Yes / No row is made like it
const SELECTOR_GV: i32 = 0x3567d8;
const SLIDER_ID: i32 = 1;
/// the item number of the new row (the game's rows of the Map Settings page are 1 to 3)
const LIMIT_ID: i32 = 4;
const BACK_ID: i32 = 0x8000;

/// the table has been made (its row's value is the one to take)
static BUILT: AtomicBool = AtomicBool::new(false);
static UNLIMITED: AtomicBool = AtomicBool::new(false);

pub fn set_mm_unlimited(on: bool) {
    UNLIMITED.store(on, Ordering::Relaxed);
}

pub fn mm_unlimited() -> bool {
    UNLIMITED.load(Ordering::Relaxed)
}

/// The cost a map may have: what the game allows, or no limit.
pub fn mm_limit() -> i32 {
    if mm_unlimited() {
        i32::MAX
    } else {
        0x1389
    }
}

/// Called first by the Options page's tick: X and triangle start and end the change of the slider, left and right do nothing otherwise.
pub fn mm_options_tick(page: i32) {
    let front = lw(g::front);
    let player = lw(front);
    let id = lhu(page + 4) as i32;
    let pressed = joyNewBut(player);
    let flags = lw(front + 12);
    let mut edit = editing_row();
    if edit != 0 && edit != id {
        edit = 0;
    }
    if id == SLIDER_ID && pressed & 64 != 0 {
        edit = if edit == id { 0 } else { id };
        soundStart(141, 0);
        joyClearButs(player, 64);
    } else if edit != 0 && pressed & 16 != 0 {
        edit = 0;
        soundStart(141, 0);
        joyClearButs(player, 16);
    }
    set_editing_row(edit);
    if id == SLIDER_ID && edit == 0 && flags & 0x30 != 0 {
        sw(front + 12, flags & -61);
    }
}

/// The table of the Options page: the game's own (the slider and Back), the slider staying on its row while it is changed.
pub fn mm_options_menu() -> i32 {
    let src = g::settings_menu;
    let n = lw(src);
    for i in 0..(4 + 8 * n) {
        sw(MENU_AT + 4 * i, lw(src + 4 * i));
    }
    if editing_row() == SLIDER_ID {
        sw(MENU_AT + 16 + 16, (SLIDER_ID << 16) | SLIDER_ID);
    }
    MENU_AT
}

/// Called first by the Map Settings page's tick: the value of the new row, changed by the game's own tick in the last frame, is the limit's.
pub fn mm_settings_tick() {
    if BUILT.load(Ordering::Relaxed) {
        let on = lw(GV_AT) != 0;
        if on != mm_unlimited() {
            set_mm_unlimited(on);
            mm_unlimited_changed(on);
        }
    }
}

/// The table of the Map Settings page: the game's rows, the new row No Limit (Yes / No, made like the Tile Set row) before the last of them (the
/// button that edits the description), and Back.  The items that have the flag 32 are put below the one before them.
pub fn mm_settings_menu() -> i32 {
    let src = g::mapsettings_menu;
    let item = |k: i32| SET_MENU_AT + 16 + 32 * k;
    let from = |k: i32| src + 16 + 32 * k;
    let rows = lw(src) - 1;
    // (the place of the new row among the game's)
    let at = rows - 1;
    for i in 0..4 {
        sw(SET_MENU_AT + 4 * i, lw(src + 4 * i));
    }
    sw(SET_MENU_AT, rows + 2);
    for k in 0..=rows {
        // the game's items from the new place on, Back included, are one place lower
        let to = if k >= at { k + 1 } else { k };
        for i in 0..8 {
            sw(item(to) + 4 * i, lw(from(k) + 4 * i));
        }
    }
    // the new row, a copy of the row above it (a selector below the one before) with the list No / Yes
    let mut text = TEXT_AT;
    let label = put_str(&mut text, "No Limit");
    let no = put_str(&mut text, "No");
    let yes = put_str(&mut text, "Yes");
    for i in 0..12 {
        sw(GV_AT + 4 * i, lw(SELECTOR_GV + 4 * i));
    }
    sw(GV_AT, mm_unlimited() as i32);
    sw(GV_AT + 4, 0);
    sw(GV_AT + 8, 1);
    sw(GV_AT + 12, 0);
    sw(GV_AT + 16, LIST_AT);
    sw(LIST_AT, 0);
    sw(LIST_AT + 4, no);
    sw(LIST_AT + 8, 1);
    sw(LIST_AT + 12, yes);
    sw(LIST_AT + 16, 0);
    sw(LIST_AT + 20, 0);
    let above = item(at - 1);
    let new = item(at);
    let below = item(at + 1);
    for i in 0..8 {
        sw(new + 4 * i, lw(above + 4 * i));
    }
    let above_id = lhu(above + 8) as i32;
    let below_id = lhu(below + 8) as i32;
    sw(new, label);
    sw(new + 8, (lw(above + 8) & !0xffff) | LIMIT_ID);
    sw(new + 12, GV_AT);
    // the way the cursor goes: from the row above down to the new row, and from it down to the next
    sw(above + 16, (LIMIT_ID << 16) | (lw(above + 16) & 0xffff));
    sw(new + 16, (below_id << 16) | above_id);
    sw(below + 16, (lw(below + 16) & !0xffff) | LIMIT_ID);
    BUILT.store(true, Ordering::Relaxed);
    SET_MENU_AT
}
