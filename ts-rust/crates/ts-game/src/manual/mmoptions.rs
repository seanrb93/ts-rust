//! The Options page of the MapMaker (`settings_pageTick`): the Grid Intensity slider works like the sliders of the other option pages (X
//! starts changing it, left and right then move it and show their arrows, X or triangle ends it), and a new row, No Limit
//! (Yes / No), takes away the limit of what a map can hold: the FREE bar is not drawn then and no placing is refused for the cost.

use crate::*;
use std::sync::atomic::{AtomicBool, Ordering};

const MENU_AT: i32 = 0x240_0000 + 0xd000;
const GV_AT: i32 = 0x240_0000 + 0xd100;
const LIST_AT: i32 = 0x240_0000 + 0xd180;
const TEXT_AT: i32 = 0x240_0000 + 0xd1c0;
/// the Tile Set row of the Map Settings page: a Yes / No row is made like it
const SELECTOR_GV: i32 = 0x3567d8;
const SLIDER_ID: i32 = 1;
const SELECT_ID: i32 = 2;
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

/// Called first by the page's tick: X and triangle start and end the change of a row, left and right do nothing otherwise.
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
    if (id == SLIDER_ID || id == SELECT_ID) && pressed & 64 != 0 {
        edit = if edit == id { 0 } else { id };
        soundStart(141, 0);
        joyClearButs(player, 64);
    } else if edit != 0 && pressed & 16 != 0 {
        edit = 0;
        soundStart(141, 0);
        joyClearButs(player, 16);
    }
    set_editing_row(edit);
    if (id == SLIDER_ID || id == SELECT_ID) && edit == 0 && flags & 0x30 != 0 {
        sw(front + 12, flags & -61);
    }
    // (the row's value, changed by the game's own tick in the last frame)
    if BUILT.load(Ordering::Relaxed) {
        let on = lw(GV_AT) != 0;
        if on != mm_unlimited() {
            set_mm_unlimited(on);
            mm_unlimited_changed(on);
        }
    }
}

/// The table of the page: the slider, the new row and Back.
pub fn mm_options_menu() -> i32 {
    let src = g::settings_menu;
    let item = |k: i32| MENU_AT + 16 + 32 * k;
    let edit = editing_row();
    for i in 0..4 {
        sw(MENU_AT + 4 * i, lw(src + 4 * i));
    }
    sw(MENU_AT, 3);
    sw(MENU_AT + 4, (0x5a << 16) | 0x8c);
    // the slider (the game's own item) goes down to the new row
    for i in 0..8 {
        sw(item(0) + 4 * i, lw(src + 16 + 4 * i));
        sw(item(2) + 4 * i, lw(src + 16 + 32 + 4 * i));
    }
    sw(item(0) + 16, if edit == SLIDER_ID { (SLIDER_ID << 16) | SLIDER_ID } else { SELECT_ID << 16 });
    // the new row: a copy of a selector of the game with the list No / Yes
    let mut at = TEXT_AT;
    let label = put_str(&mut at, "No Limit");
    let no = put_str(&mut at, "No");
    let yes = put_str(&mut at, "Yes");
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
    // the arrows are shown while the row is being changed
    sw(GV_AT + 32, if edit == SELECT_ID { -86 } else { 1500 });
    sw(GV_AT + 36, if edit == SELECT_ID { lw(SELECTOR_GV + 36) } else { 1500 });
    sw(item(1), label);
    sw(item(1) + 4, 20 << 16);
    sw(item(1) + 8, (0x0004 << 16) | SELECT_ID);
    sw(item(1) + 12, GV_AT);
    sw(item(1) + 16, if edit == SELECT_ID { (SELECT_ID << 16) | SELECT_ID } else { (BACK_ID << 16) | SLIDER_ID });
    for k in 5..8 {
        sw(item(1) + 4 * k, 0);
    }
    // Back (one line lower)
    sw(item(2) + 4, ((lw(item(2) + 4) >> 16) + 20) << 16);
    sw(item(2) + 16, SELECT_ID);
    BUILT.store(true, Ordering::Relaxed);
    MENU_AT
}
