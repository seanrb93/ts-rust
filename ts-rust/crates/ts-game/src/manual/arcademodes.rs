//! The mode list of the arcade level page: the modes that mods add (`variant` in `modes.txt`) are listed after the game's own.
//!
//! The game's table of the page (`arcade_menu1`) has room for the six modes and nothing else, so the port works on a copy in its own memory with more places for them (`SLOTS`): the
//! tick of the page asks `av_arcade_menu` for the table and fills the places, and a mod's mode, once chosen, is its base mode (one of the six) with its variant set, as if it had been
//! chosen on the Rules row of the Options page that follows.

use super::avmenu::put_str;
use crate::*;
use std::sync::atomic::{AtomicBool, Ordering};

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
