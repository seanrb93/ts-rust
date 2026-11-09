//! The mode list of the arcade level page: the modes that mods add (`variant` in `modes.txt`) are listed after the game's own.
//!
//! The game's table of the page (`arcade_menu1`) has room for the six modes and nothing else, so the port works on a copy in its own memory with more places for them (`SLOTS`): the
//! tick of the page asks `av_arcade_menu` for the table and fills the places, and a mod's mode, once chosen, is its base mode (one of the six) with its variant set, as if it had been
//! chosen on the Rules row of the Options page that follows.

use super::avmenu::put_str;
use crate::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const BASE: i32 = 0x240_0000;
const MENU_AT: i32 = BASE + 0x17_3000;
const TEXT_AT: i32 = BASE + 0x17_3400;
/// the places for modes in the table (the game's has 6)
pub const SLOTS: i32 = 9;
/// the most modes the page lists: the lines above the buttons at the bottom
const MOST: i32 = 7;
/// the items of the game's table, and where its last three (a gadget, the hint and the cheats entry) go in the new one
const OLD_ITEMS: i32 = 11;

static BUILT: AtomicBool = AtomicBool::new(false);
/// The rows of a variant's options on the Options page: the mode, the variant, and the values of the rows (the game's variables, which the page
/// changes).
static ROWS_FOR: Mutex<Option<(i32, usize, Vec<i32>)>> = Mutex::new(None);
/// The values of the rows of options (the game's variables of 48 bytes) and their labels, and the most rows the page has.
const OPTION_GV_AT: i32 = BASE + 0x17_4000;
const OPTION_GV_BYTES: i32 = 48;
const OPTION_TEXT_AT: i32 = BASE + 0x17_4400;
const MOST_ROWS: i32 = 10;

/// A mode a mod adds to the list, as the tick of the page needs it.
pub struct ExtraMode {
    /// the name, as text in memory
    pub text: i32,
    /// the level has it, but it is not available yet (as the game shows a mode that is locked)
    pub locked: bool,
    /// which variant of the mods it is (in the order of `all_variants`)
    pub number: i32,
}

/// The table of the page, with `SLOTS` places for the modes (made from the game's the first time).
pub fn av_arcade_menu() -> i32 {
    if !BUILT.swap(true, Ordering::Relaxed) {
        let src = g::arcade_menu1;
        let item = |k: i32| MENU_AT + 16 + 32 * k;
        // (the header, then the items up to the six places for modes: the extra places are copies of the last of them)
        for i in 0..4 {
            sw(MENU_AT + 4 * i, lw(src + 4 * i));
        }
        sw(MENU_AT, 2 + SLOTS + 3);
        for k in 0..8 {
            for j in 0..8 {
                sw(item(k) + 4 * j, lw(src + 16 + 32 * k + 4 * j));
            }
        }
        for k in 8..(2 + SLOTS) {
            for j in 0..8 {
                sw(item(k) + 4 * j, lw(item(7) + 4 * j));
            }
        }
        // (the gadget, the hint and the cheats entry go after the places)
        for (old, new) in [(8, 2 + SLOTS), (9, 3 + SLOTS), (10, 4 + SLOTS)] {
            for j in 0..8 {
                sw(item(new) + 4 * j, lw(src + 16 + 32 * old + 4 * j));
            }
        }
        // (the cursor goes down the ring of the title and the places: the numbers of the items are their places plus one)
        let last = SLOTS + 2;
        sw(item(0) + 16, (3 << 16) | last);
        for k in 2..(2 + SLOTS) {
            let id = k + 1;
            sh(item(k) + 8, id);
            let prev = if k == 2 { 1 } else { id - 1 };
            let next = if k == 1 + SLOTS { 1 } else { id + 1 };
            sw(item(k) + 16, (next << 16) | prev);
        }
        debug_assert_eq!(OLD_ITEMS, 11);
    }
    MENU_AT
}

/// The modes of the mods that a level with the modes `allowed` (a mask of the game's set-up of the level) can be played in: those whose base mode it has, up to what the page has room for after the `have` modes listed.
pub fn arcade_extra_modes(allowed: i32, few_players: bool, have: i32) -> Vec<ExtraMode> {
    let mut out = Vec::new();
    let mut at = TEXT_AT;
    for (number, (kind, _, name)) in all_variants().into_iter().enumerate() {
        if have + out.len() as i32 >= MOST {
            break;
        }
        // (the mask of the base mode in the game's table of modes: pairs of the mode and its mask)
        let Some(mask) = (0..6).map(|i| (lbu(g::gmtab + 2 * i), lbu(g::gmtab + 2 * i + 1))).find(|m| m.0 == kind).map(|m| m.1) else {
            continue;
        };
        if (allowed & mask) == 0 || (kind == 6 && !few_players) {
            continue;
        }
        out.push(ExtraMode { text: put_str(&mut at, &name), locked: (allowed & (mask << 7)) != 0, number: number as i32 });
    }
    out
}

/// The base mode and the number of the variant (among those of that mode) of the mode number `number` of the list `all_variants`.
pub fn arcade_variant_of(number: i32) -> (i32, usize) {
    let all = all_variants();
    let Some((kind, index, _)) = all.get(number as usize).cloned() else {
        return (3, 0);
    };
    (kind, index)
}

/// The rows of the Options page of the mode `kind` (`source`: the game's list of `count` items): without the options that the variant being played
/// hides, the rest moved up, and after them the rows of the variant's own options (`option` in `modes.txt`), as many as the page has room for
/// (ten rows in all).  Gives the list and its count.
pub fn arcade_options_filter(kind: i32, source: i32, count: i32) -> (i32, i32) {
    let hidden = active_hidden(kind);
    let choice = super::arcadeopts::variant_choice(kind);
    // (what a player changed on the page the last time it was shown is kept before the rows are made again)
    option_rows_keep();
    let options = variant_options(kind, choice);
    if (hidden.is_empty() && options.is_empty()) || count == 0 {
        *ROWS_FOR.lock().unwrap() = None;
        return (source, count);
    }
    let rows = BASE + 0x17_3800;
    let ys: Vec<i32> = (0..count).map(|k| lh(source + 32 * k + 6)).collect();
    let mut kept = 0;
    // (the row the options' rows are copies of: the time limit's, a row of numbers, where the mode has one)
    let mut template = source;
    for k in 0..count {
        let item = source + 32 * k;
        if lw(item + 12) == g::gv_timelimit {
            template = item;
        }
        if hidden.contains(&lw(item + 12)) {
            continue;
        }
        for j in 0..8 {
            sw(rows + 32 * kept + 4 * j, lw(item + 4 * j));
        }
        sh(rows + 32 * kept + 6, ys[kept as usize]);
        kept += 1;
    }
    let step = if count >= 2 { ys[1] - ys[0] } else { 14 };
    let mut text = OPTION_TEXT_AT;
    let mut made = Vec::new();
    for (index, o) in options.iter().enumerate() {
        if kept >= MOST_ROWS {
            eprintln!("[mods] the option {:?} has no room on the Options page", o.name);
            break;
        }
        // (its value: a copy of the template's, with this option's range and value, shown as a plain number)
        let gv = OPTION_GV_AT + OPTION_GV_BYTES * index as i32;
        let template_gv = lw(template + 12);
        for j in 0..(OPTION_GV_BYTES / 4) {
            sw(gv + 4 * j, if template_gv != 0 { lw(template_gv + 4 * j) } else { 0 });
        }
        sw(gv, o.value.clamp(o.least, o.most));
        sw(gv + 4, o.least);
        sw(gv + 8, o.most);
        sw(gv + 12, 0);
        sw(gv + 16, 0);
        sw(gv + 20, 0);
        sw(gv + 24, 0);
        let row = rows + 32 * kept;
        for j in 0..8 {
            sw(row + 4 * j, lw(template + 4 * j));
        }
        sw(row, put_str(&mut text, &o.label));
        sw(row + 12, gv);
        let y = if (kept as usize) < ys.len() { ys[kept as usize] } else { ys[ys.len() - 1] + step * (kept + 1 - ys.len() as i32) };
        sh(row + 6, y);
        made.push(gv);
        kept += 1;
    }
    *ROWS_FOR.lock().unwrap() = (!made.is_empty()).then(|| (kind, choice, made));
    (rows, kept)
}

/// The values of the rows of the variant's options, as a player left them on the Options page, are kept with the variant (before the page is made
/// again, and when a match starts and its script reads them).
pub fn option_rows_keep() {
    let Some((kind, choice, gvs)) = ROWS_FOR.lock().unwrap().clone() else { return };
    for (index, gv) in gvs.iter().enumerate() {
        variant_option_set(kind, choice, index, lw(*gv));
    }
}
