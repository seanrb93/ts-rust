//! Levels that mods add.  The game has a table of its levels (`lvinfo`: the name's text number, the level number, the music, and what the sky and
//! fog are like) and a list of the levels the arcade page offers.  Both are copied into the port's memory with the mods' levels added, and a mod's
//! level is made from the level it is `like`: it has that level's record, its arcade set-up (the modes), and, where the mod has no files of its
//! own for it, its data (the mod loader gives it copies of the files of that level under its own number).
//!
//! A level has a number from 31 to 99; its name is a new text of the game's table of texts (the five language tables are copied too, with the
//! names added).

use crate::*;
use super::avmenu::read_str;
use std::sync::Mutex;

const BASE: i32 = 0x240_0000;
/// the level table (a record is 24 bytes) and the tables of texts
pub const LVINFO: i32 = BASE + 0x58000;
const TABLES_AT: i32 = BASE + 0x60000;
const TABLE_ROOM: i32 = 0x2000;
const NAMES_AT: i32 = BASE + 0x70000;
/// copies of the arcade set-ups of the mods' levels
const SETUPS_AT: i32 = BASE + 0x72000;
const SETUP_BYTES: i32 = 0x100;
/// the number of texts of the game
const TEXTS: i32 = 1246;
/// the text number of the first name of a level (the two before it are the names of the characters of `chrnames.rs`)
const FIRST_NAME: i32 = TEXTS + 2;
/// the most levels a mod can add (the tables of texts of `chrnames.rs` have room for so many names)
pub const MAX_LEVELS: usize = 28;

static CUSTOM: Mutex<Vec<ts_sdk::CustomLevel>> = Mutex::new(Vec::new());

/// Make the tables (before the game starts).
pub fn custom_levels_init() {
    // the game's records, up to the one that ends the table
    let texts = lw(g::languages);
    let mut n = 0;
    while lw((lh(g::lvinfo + 24 * n) << 2) + texts) != 0 {
        n += 1;
    }
    for i in 0..6 * (n + 1) {
        sw(LVINFO + 4 * i, lw(g::lvinfo + 4 * i));
    }
    let levels: Vec<ts_sdk::CustomLevel> = ts_sdk::custom_levels().into_iter().filter(|l| (31..=99).contains(&l.number)).take(MAX_LEVELS).collect();
    if levels.is_empty() {
        return;
    }
    // the tables of texts, with the names
    let mut at = NAMES_AT;
    let mut name_ptrs = Vec::new();
    for l in &levels {
        name_ptrs.push(put_str(&mut at, &l.name));
    }
    for lang in 0..5 {
        let old = lw(g::languages + 4 * lang);
        if !(0x100000..0x2000000).contains(&old) {
            continue;
        }
        let new = TABLES_AT + TABLE_ROOM * lang;
        // (the two texts after the game's are the names of two characters: `chrnames.rs`)
        for i in 0..TEXTS + 2 {
            sw(new + 4 * i, lw(old + 4 * i));
        }
        for (k, p) in name_ptrs.iter().enumerate() {
            sw(new + 4 * (FIRST_NAME + k as i32), *p);
        }
        sw(g::languages + 4 * lang, new);
    }
    // the records: the terminating record moves down, the mods' go before it
    let end: Vec<i32> = (0..6).map(|i| lw(LVINFO + 24 * n + 4 * i)).collect();
    let mut kept = Vec::new();
    for (k, l) in levels.iter().enumerate() {
        let Some(like) = (0..n).find(|&r| lw(LVINFO + 24 * r + 4) == l.like) else {
            eprintln!("[levels] level {} ({}): there is no level {} to be like", l.number, l.name, l.like);
            continue;
        };
        let at = LVINFO + 24 * (n + kept.len() as i32);
        for i in 0..6 {
            sw(at + 4 * i, lw(LVINFO + 24 * like + 4 * i));
        }
        sh(at, FIRST_NAME + k as i32);
        sw(at + 4, l.number);
        if l.tune > 0 {
            sw(at + 8, l.tune);
        }
        kept.push(l.clone());
    }
    let at = LVINFO + 24 * (n + kept.len() as i32);
    for (i, v) in end.iter().enumerate() {
        sw(at + 4 * i as i32, *v);
    }
    // the arcade set-ups: those of the levels they are like, with the modes of the mod
    for (k, l) in kept.iter().enumerate() {
        if let (Some(modes), Some(setup)) = (l.modes, original_setup(l.like)) {
            let copy = SETUPS_AT + SETUP_BYTES * k as i32;
            for i in 0..SETUP_BYTES / 4 {
                sw(copy + 4 * i, lw(setup + 4 * i));
            }
            sw(copy, (lw(copy) & !63) | modes as i32);
        }
    }
    eprintln!("[levels] {} level(s) added: {}", kept.len(), kept.iter().map(|l| format!("{} {}", l.number, l.name)).collect::<Vec<_>>().join(", "));
    if std::env::var_os("TS_LVLOG").is_some() {
        let texts = lw(g::languages);
        for r in 0..(n + kept.len() as i32 + 1) {
            eprintln!("[levels] record {r}: text {} level {} tune {} {:?}", lh(LVINFO + 24 * r), lw(LVINFO + 24 * r + 4), lw(LVINFO + 24 * r + 8), read_str(lw((lh(LVINFO + 24 * r) << 2) + texts)));
        }
    }
    if std::env::var_os("TS_LVLOG").is_some() {
        for l in &kept {
            eprintln!("[levels] lvGetNameFromNum({}) = {:?}", l.number, read_str(lvGetNameFromNum(l.number)));
        }
    }
    *CUSTOM.lock().unwrap() = kept;
}

/// The set-up of an original level (the game's function, with the mods' levels taken out).
fn original_setup(level: i32) -> Option<i32> {
    let p = lvGetARCADESetup(level);
    (p != 0).then_some(p)
}

/// The arcade set-up of a level of a mod: a copy with its own modes, or the one of the level it is like.
pub fn custom_setup(level: i32) -> Option<i32> {
    let (k, like, own_modes) = {
        let c = CUSTOM.lock().unwrap();
        let k = c.iter().position(|l| l.number == level)?;
        (k, c[k].like, c[k].modes.is_some())
    };
    if own_modes {
        Some(SETUPS_AT + SETUP_BYTES * k as i32)
    } else {
        // (the game's own function: the level is one of the game's)
        Some(lvGetARCADESetup(like))
    }
}

/// How many texts the mods' levels have added after the game's and the characters'.
pub fn custom_text_count() -> i32 {
    CUSTOM.lock().unwrap().len() as i32
}

/// The levels the arcade page lists after the game's: those of the mods.
pub fn custom_arcade_levels() -> Vec<i32> {
    CUSTOM.lock().unwrap().iter().map(|l| l.number).collect()
}
