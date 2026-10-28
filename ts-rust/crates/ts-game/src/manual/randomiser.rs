//! Randomisers of the front end's arcade setup: the Square button on the character select page picks a random character, and on the
//! weapon set page (with the cursor on the set, not editing it) it fills the custom weapon set with random weapons.

use crate::*;
use std::sync::atomic::{AtomicU64, Ordering};

const TEXT_AT: i32 = 0x240_0000 + 0x9000;
const SQUARE: i32 = 128;

static SEED: AtomicU64 = AtomicU64::new(0);

/// A random number below `n` (xorshift, seeded from the clock the first time).
fn below(n: u32) -> u32 {
    let mut x = SEED.load(Ordering::Relaxed);
    if x == 0 {
        x = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0x9e37_79b9_7f4a_7c15, |d| d.as_nanos() as u64) | 1;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    SEED.store(x, Ordering::Relaxed);
    ((x >> 32) % n.max(1) as u64) as u32
}

/// Character select page (`chrplayer_pageTick`, first thing): the Square button of a player who is choosing a character gives
/// that player a random one among those that can be chosen; the legend names the button.
pub fn char_randomise(page: i32) {
    // the legend of the page: "Ok" (and "Back" for the first player) and the new button
    let mut at = TEXT_AT;
    let first = put_str(&mut at, "^2 Ok   ^4 Random   ^3 Back");
    let others = put_str(&mut at, "^2 Ok   ^4 Random");
    sw(g::chrselect_main_menu + 16 + 32, first);
    sw(g::chrselect_main_menu_p234 + 16 + 32, others);
    let front = lw(g::front);
    let player = lw(front);
    // (the page's own state: choosing the character, the cursor on it, not confirmed, no change of character under way)
    if lbu(player + 0x3aff28) != 2 || lhu(page + 4) != 1 || lw((player << 2) + 0x1fc12a8) != 0 || lw((player << 2) + 0x1fc1268) != 0 {
        return;
    }
    if joyNewBut(player) & SQUARE == 0 {
        return;
    }
    let id = signonGet(player);
    if id < 0 {
        return;
    }
    let prof = lw(g::signonarray) + id * 2936 + 32;
    let current = lw(prof + 84);
    let mut choices: Vec<i32> = (0..64).map(|i| lbu(g::characterorder + i)).filter(|&c| c != current && chrPlayable(c) != 0).collect();
    // (the characters that mods added come after, all playable)
    choices.extend((0..added_characters().min(20)).map(|k| GAME_CHARACTERS + k).filter(|&c| c != current));
    if choices.is_empty() {
        return;
    }
    let chr = choices[below(choices.len() as u32) as usize];
    sw(prof + 84, chr);
    soundStart(140, 0);
    fxSelectCharInit(player, chr);
    joyClearButs(player, SQUARE);
}

/// Weapon set page (`weapons_pageTick`, first thing): the Square button with the cursor on the set (not while the weapons of the
/// custom set are being edited, where it removes one) fills the custom set with five different random weapons of those available.
pub fn weapons_randomise(page: i32) {
    if lhu(page + 4) != 2 || !weaponset_is_custom() {
        return;
    }
    weaponset_effective(); // (the chosen custom set is in gunzpack)
    let player = lw(lw(g::front));
    if joyNewBut(player) & SQUARE == 0 {
        return;
    }
    // (the page makes the list of the weapons that can be chosen every tick: count at 0x3aff4c, the weapons at 0x1fc2a00)
    let n = lw(0x3aff4c).clamp(0, 40);
    if n == 0 {
        return;
    }
    let mut order: Vec<i32> = (0..n).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, below(i as u32 + 1) as usize);
    }
    for k in 0..5 {
        sw(g::gunzpack + 4 * k, if k < n { lw(0x1fc2a00 + 4 * order[k as usize]) } else { -1 });
    }
    soundStart(143, 0);
    joyClearButs(player, SQUARE);
}

/// The legend of the weapon set page while the cursor is on the set (the game's own text is "Edit Weapons"; none when a preset is shown).
pub fn weapons_legend() -> i32 {
    if !weaponset_is_custom() {
        return 0;
    }
    let mut at = TEXT_AT + 0x100;
    put_str(&mut at, "^2 Edit Weapons   ^4 Random Set")
}

/// Bot set page (`chrsets_pageTick`, first thing): the Square button with the cursor on the set (state 1) of a custom bot set fills the
/// set with random bots (and, when the game mode has teams, puts them in random teams) of those that can be chosen (ten of them, each once, as many as there are); the rows of the page's table are written (the
/// page keeps them in the set itself).
pub fn bots_randomise(page: i32) {
    if lhu(page + 4) != 1 || lw(g::gv_chrset) < 8 {
        return;
    }
    let set = chrset_effective();
    if set < 8 {
        return;
    }
    let player = lw(lw(g::front));
    if joyNewBut(player) & SQUARE == 0 {
        return;
    }
    // (the page makes the list of the bots that can be chosen every tick: count at 0x3aff50, the characters at 0x1fc2b48)
    let n = lw(0x3aff50).clamp(0, 84);
    if n == 0 {
        return;
    }
    let mut order: Vec<i32> = (0..n).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, below(i as u32 + 1) as usize);
    }
    let count = 10.min(n);
    // with teams the bots are spread over the teams the game mode has (the bots are in random order already)
    let mut teams: Vec<i32> = if lw(g::gv_teamplay) != 0 { (0..4).filter(|&t| lbu(0x1fc1234 + t) != 0).collect() } else { Vec::new() };
    let shift = below(4) as usize;
    for i in (1..teams.len()).rev() {
        teams.swap(i, below(i as u32 + 1) as usize);
    }
    for row in 0..10 {
        let chr = if row < count { lw(0x1fc2b48 + 4 * order[row as usize]) } else { -1 };
        sw(0x1fc2a88 + 16 * row, chr);
        // (the set itself: the page loads its rows from it at the start of its tick)
        sb(0x1fc2b28 + 10 * (set - 8) + row, chr);
        // (the other columns of the row: the team, the skill and the tick that says the bot takes part)
        if chr >= 0 {
            let team = if teams.is_empty() { 0 } else { teams[(row as usize + shift) % teams.len()] };
            sw(0x1fc2a88 + 16 * row + 4, team);
            sw(0x1fc2a88 + 16 * row + 8, 2);
            sw(0x1fc2a88 + 16 * row + 12, 1);
        }
    }
    soundStart(143, 0);
    joyClearButs(player, SQUARE);
}

/// The legend of the bot set page while the cursor is on the set: the game's own text (Edit Bots) and, for a custom set, the new button.
pub fn bots_legend(text: i32, preset: i32) -> i32 {
    if preset != 0 {
        return text;
    }
    let mut name = String::new();
    let mut at = text;
    while lbu(at) != 0 && name.len() < 40 {
        name.push(lbu(at) as u8 as char);
        at += 1;
    }
    let mut out = TEXT_AT + 0x300;
    put_str(&mut out, &format!("{name}   ^4 Random Set"))
}
