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
/// the row "Rules" after Level Music, for the modes that a mod gives variants (`modes.txt`)
const NEW2: i32 = 17;
const NEW2_ID: i32 = 16;
const GV2_AT: i32 = BASE + 0x5_0580;
const LIST2_AT: i32 = BASE + 0x5_05c0;
/// the variant chosen (0: Standard) and the mode it was chosen for
static VARIANT: AtomicI32 = AtomicI32::new(0);
static VARIANT_MODE: AtomicI32 = AtomicI32::new(-1);
static VARIANT_LAST: AtomicI32 = AtomicI32::new(-1);
/// item numbers (the cursor's value) of Bot Set, GO! and the new row
const BOT_ID: i32 = 13;
const GO_ID: i32 = 14;
const NEW_ID: i32 = 15;
/// the lines between Bot Set and the new row, and between the new row and GO!
const ROW_GAP: i32 = 13;
const GO_GAP: i32 = 17;
/// how much lower the Back and Edit legends are than the game has them (in line with the Bots)
const LEGEND_DOWN: i32 = 7;
/// room for the left arrow beside a long value (in the units of the page)
const ARROW_ROOM: i32 = 34;
/// how much higher the page's items are than the game has them
const UP: i32 = 6;

/// the number of tracks (`MUSIC\1001.MSC` .. `1027.MSC`)
const TRACKS: i32 = 27;

/// an arcade match is being set up or played (the options page was shown since the main menu was): only those take the player's choice of music
pub(crate) static ARCADE: AtomicBool = AtomicBool::new(false);
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

/// The name of a track.  The MapMaker's list of music has the game's own names for most of the tracks (its values 1 to 23 are tracks,
/// `disptunenums` says which); the others are named here (the tile sets' tracks are the first five, and 3 and 5 are not Ron's and Tech's:
/// the levels of these names play the tile sets' music).
fn track_name(track: i32) -> String {
    match track {
        19 => return "Cyberden Launch".to_string(),
        21 => return "Credits".to_string(),
        25 => return "MapMaker".to_string(),
        27 => return "Frontend".to_string(),
        _ => {}
    }
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    for v in 1..24 {
        if lbu(g::disptunenums + v) == track {
            let name = read_str(lw(texts + 4 * lw(g::disp_tunes + 8 * v + 4)));
            // (shorter names: the row has only so much room)
            let name = match name.as_str() {
                "The Bank" => "Bank".to_string(),
                "Building Site" => "Site".to_string(),
                _ => name,
            };
            return if (1..=5).contains(&track) { format!("{name} Tile Set") } else { name };
        }
    }
    format!("Track {track}")
}

/// The variant chosen on the Options page for the mode `mode` (0: Standard, or the page was not shown for this mode).
pub fn variant_choice(mode: i32) -> usize {
    // (a test aid: TS_VARIANT=<number> plays that variant of whatever mode runs, without the page)
    if let Some(v) = std::env::var("TS_VARIANT").ok().and_then(|v| v.parse::<usize>().ok()) {
        return v;
    }
    if VARIANT_MODE.load(Ordering::Relaxed) != mode || !ARCADE.load(Ordering::Relaxed) {
        return 0;
    }
    VARIANT.load(Ordering::Relaxed).max(0) as usize
}

/// The table, with the new row made, for the page's menu (called by the tick after it has put the game's rows where they go).
pub fn av_opt_ready() -> i32 {
    let m = av_opt_menu();
    ARCADE.store(true, Ordering::Relaxed);
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
    // the value names: Level's Own, Random, the tracks
    let mut at = TEXT_AT;
    let label = put_str(&mut at, "Level Music");
    let mut names = vec!["Default".to_string(), "Random".to_string()];
    for t in 1..=TRACKS {
        names.push(track_name(t));
    }
    let mut list = LIST_AT;
    let mut width_of_value = 0;
    // (the value is drawn in the big font of the rows; its width says where the left arrow goes)
    let font = textSelect(lw(g::menuitem_bigfont));
    for (v, name) in names.iter().enumerate() {
        let s = put_str(&mut at, name);
        sw(list, v as i32);
        sw(list + 4, s);
        list += 8;
        if v as i32 == choice {
            width_of_value = textWidth(s);
        }
    }
    textSelect(font);
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
    // the left arrow is where the other rows have it, or left of a long value
    sw(GV_AT + 32, -(256.max(width_of_value + ARROW_ROOM)));
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
    // the row "Rules": the variants of the mode that was chosen
    let mode = lw(0x1fc11f8 + 48);
    let variants = variants_of(mode);
    if !variants.is_empty() {
        if VARIANT_MODE.swap(mode, Ordering::Relaxed) != mode {
            VARIANT.store(0, Ordering::Relaxed);
            VARIANT_LAST.store(-1, Ordering::Relaxed);
        }
        let now = lw(GV2_AT);
        let last = VARIANT_LAST.load(Ordering::Relaxed);
        let mut choice = VARIANT.load(Ordering::Relaxed);
        if last >= 0 && now != last && (0..=variants.len() as i32).contains(&now) {
            choice = now;
            VARIANT.store(choice, Ordering::Relaxed);
            apply_variant(mode, choice as usize);
        }
        VARIANT_LAST.store(choice, Ordering::Relaxed);
        let label = put_str(&mut at, "Rules");
        let mut list = LIST2_AT;
        let mut width_of_value = 0;
        let font = textSelect(lw(g::menuitem_bigfont));
        for (v, name) in std::iter::once("Standard".to_string()).chain(variants.iter().cloned()).enumerate() {
            let s = put_str(&mut at, &name);
            sw(list, v as i32);
            sw(list + 4, s);
            list += 8;
            if v as i32 == choice {
                width_of_value = textWidth(s);
            }
        }
        textSelect(font);
        sw(list, 0);
        sw(list + 4, 0);
        for i in 0..12 {
            sw(GV2_AT + 4 * i, lw(g::gv_weaponset + 4 * i));
        }
        sw(GV2_AT, choice);
        sw(GV2_AT + 4, 0);
        sw(GV2_AT + 8, variants.len() as i32);
        sw(GV2_AT + 12, 0);
        sw(GV2_AT + 16, LIST2_AT);
        sw(GV2_AT + 20, 0);
        sw(GV2_AT + 32, -(256.max(width_of_value + ARROW_ROOM)));
        for i in 0..8 {
            sw(item(NEW2) + 4 * i, lw(item(BOT) + 4 * i));
        }
        sw(item(NEW2), label);
        sh(item(NEW2) + 6, bot_y + 2 * ROW_GAP);
        sw(item(NEW2) + 8, (lw(item(BOT) + 8) & !0xffff) | NEW2_ID);
        sw(item(NEW2) + 12, GV2_AT);
        sw(item(NEW2) + 16, (GO_ID << 16) | NEW_ID);
        sw(item(NEW) + 16, (NEW2_ID << 16) | BOT_ID);
        sw(item(GO) + 16, (lw(item(GO) + 16) & !0xffff) | NEW2_ID);
        sh(item(GO) + 6, bot_y + 2 * ROW_GAP + GO_GAP);
        sw(m, NEW2 + 1);
    }
    // the legends (Back, Edit) go down to the line of the Bots
    for k in [14, 15] {
        let at = item(k) + 4;
        sw(at, (lw(at) & 0xffff) | ((0xae + LEGEND_DOWN) << 16));
    }
    // everything a little higher: more room between GO! and the legend below it
    sw(m + 4, (lw(m + 4) & 0xffff) | ((0x20 - UP) << 16));
    m
}

/// The tune of the level that starts (`own` is the level's): the one the player chose on the arcade options page.
pub fn av_level_music(own: i32) -> i32 {
    let choice = provider().map_or(0, |p| p.level_music());
    // (levels 101 .. 105 are the pages of the front end, which keep their own music)
    if choice <= 0 || !ARCADE.load(Ordering::Relaxed) || ((lw(g::lvlevelnum) - 101) as u32) < 5 {
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
