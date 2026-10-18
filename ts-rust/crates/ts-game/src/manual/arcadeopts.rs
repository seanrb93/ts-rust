//! The Options page of an arcade match (`arcadeoptions_pageTick`, the page with the game's settings, Weapon Set, Bot Set and GO!): the
//! port adds a row after Bot Set, Level Music.  It chooses the music the levels play: the level's own (the default), a random track
//! each time a level starts, or one of the 27 tracks.
//!
//! The page's table has no room for another item where the game keeps it, so the port works on a copy in its own memory (made the first
//! time the page asks for it; `av_opt_menu` stands for the game's table in the page's tick).  The new row is a copy of Bot Set's item with a
//! value list of its own, put between Bot Set and GO!.  Which tune a level plays is asked for by `musicRestart`, which asks `av_level_music`.

use super::avmenu::{provider, put_str, read_str};
use crate::*;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

const BASE: i32 = 0x240_0000;
const MENU_AT: i32 = BASE + 0x5_0000;
const GV_AT: i32 = BASE + 0x5_0400;
const LIST_AT: i32 = BASE + 0x5_0480;
const TEXT_AT: i32 = BASE + 0x5_0800;
/// the items of the table: the title, the game's rows (up to ten), Weapon Set (11), Bot Set, GO!, Back, the hint, and the new row
const BOT: i32 = 12;
const GO: i32 = 13;
const NEW: i32 = 16;
/// item numbers (the cursor's value) of Bot Set, GO! and the new row
const BOT_ID: i32 = 13;
const GO_ID: i32 = 14;
const NEW_ID: i32 = 15;
/// the lines between Bot Set and the new row, and between the new row and GO!
const ROW_GAP: i32 = 13;
const GO_GAP: i32 = 14;

/// the number of tracks (`MUSIC\1001.MSC` .. `1027.MSC`)
const TRACKS: i32 = 27;

static BUILT: AtomicBool = AtomicBool::new(false);
/// the value of the row the last time it was looked at (-1: not yet): a different one is what the player chose
static LAST: AtomicI32 = AtomicI32::new(-1);
static LAST_RANDOM: AtomicI32 = AtomicI32::new(0);

/// The page's table, in the port's memory (a copy of the game's, with room for the new row).
pub fn av_opt_menu() -> i32 {
    if !BUILT.swap(true, Ordering::Relaxed) {
        let src = g::arcadeoptions_menu;
        // (header, then the sixteen items of the game)
        for i in 0..(4 + 8 * 16) {
            sw(MENU_AT + 4 * i, lw(src + 4 * i));
        }
        for i in 0..8 {
            sw(MENU_AT + 16 + 32 * NEW + 4 * i, 0);
        }
    }
    MENU_AT
}

/// The name of the first level that plays a track, if there is one.
fn track_level(track: i32) -> Option<String> {
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    let mut rec = g::lvinfo;
    // (the table ends with the entry whose name is no text)
    while lw((lh(rec) << 2) + texts) != 0 {
        if lw(rec + 8) == track && lw(rec + 4) < 100 {
            return Some(read_str(lw((lh(rec) << 2) + texts)));
        }
        rec += 24;
    }
    None
}

/// The table, with the new row made, for the page's menu (called by the tick after it has put the game's rows where they go).
pub fn av_opt_ready() -> i32 {
    let m = av_opt_menu();
    let item = |k: i32| m + 16 + 32 * k;
    let Some(p) = provider() else {
        return m;
    };
    // the value: what the player changed it to, else what the port has
    let now = lw(GV_AT);
    let last = LAST.load(Ordering::Relaxed);
    let choice = if last >= 0 && now != last && (0..=TRACKS + 1).contains(&now) {
        p.set_level_music(now);
        now
    } else {
        p.level_music().clamp(0, TRACKS + 1)
    };
    LAST.store(choice, Ordering::Relaxed);
    // the value names: Level's Own, Random, the tracks with the level that plays each
    let mut at = TEXT_AT;
    let label = put_str(&mut at, "Level Music");
    let mut names = vec!["Level's Own".to_string(), "Random".to_string()];
    for t in 1..=TRACKS {
        names.push(match track_level(t) {
            Some(level) => format!("{t} {level}"),
            None => format!("Track {t}"),
        });
    }
    let mut list = LIST_AT;
    for (v, name) in names.iter().enumerate() {
        let s = put_str(&mut at, name);
        sw(list, v as i32);
        sw(list + 4, s);
        list += 8;
    }
    sw(list, 0);
    sw(list + 4, 0);
    // the value is kept like the ones of the game: a copy of Weapon Set's with this row's list
    for i in 0..12 {
        sw(GV_AT + 4 * i, lw(g::gv_weaponset + 4 * i));
    }
    sw(GV_AT, choice);
    sw(GV_AT + 4, 0);
    sw(GV_AT + 8, TRACKS + 1);
    sw(GV_AT + 12, 0);
    sw(GV_AT + 16, LIST_AT);
    sw(GV_AT + 20, 0);
    // the item: a copy of Bot Set's, a line lower, and GO! one line lower than that
    for i in 0..8 {
        sw(item(NEW) + 4 * i, lw(item(BOT) + 4 * i));
    }
    let bot_y = lh(item(BOT) + 6);
    sw(item(NEW), label);
    sh(item(NEW) + 6, bot_y + ROW_GAP);
    sw(item(NEW) + 8, (lw(item(BOT) + 8) & !0xffff) | NEW_ID);
    sw(item(NEW) + 12, GV_AT);
    sw(item(NEW) + 16, (GO_ID << 16) | BOT_ID);
    // the cursor goes Bot Set, the new row, GO!
    sw(item(BOT) + 16, (NEW_ID << 16) | (lw(item(BOT) + 16) & 0xffff));
    sw(item(GO) + 16, (lw(item(GO) + 16) & !0xffff) | NEW_ID);
    sh(item(GO) + 6, bot_y + ROW_GAP + GO_GAP);
    sw(m, NEW + 1);
    m
}

/// The tune of the level that starts (`own` is the level's): the one the player chose on the arcade options page.
pub fn av_level_music(own: i32) -> i32 {
    let choice = provider().map_or(0, |p| p.level_music());
    // (levels 101 .. 105 are the pages of the front end, which keep their own music)
    if choice <= 0 || ((lw(g::lvlevelnum) - 101) as u32) < 5 {
        return own;
    }
    if choice == 1 {
        static SEED: AtomicU32 = AtomicU32::new(0);
        let mut x = SEED.load(Ordering::Relaxed);
        if x == 0 {
            x = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0x9e37_79b9, |d| d.subsec_nanos() ^ d.as_secs() as u32) | 1;
        }
        let last = LAST_RANDOM.load(Ordering::Relaxed);
        let mut track;
        loop {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            track = (x >> 8) as i32 % TRACKS + 1;
            // (not the track of the last time)
            if track != last {
                break;
            }
        }
        SEED.store(x, Ordering::Relaxed);
        LAST_RANDOM.store(track, Ordering::Relaxed);
        return track;
    }
    (choice - 1).clamp(1, TRACKS)
}
