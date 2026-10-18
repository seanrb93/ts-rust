//! Names of the two TimeSplitter characters.
//!
//! The character table (`chrs`) has two playable characters that share the text "TimeSplitter": character 71 is the one the story
//! puts into the levels of the past (the level presets of Village, Castle, Tomb ... start their bot list with it) and character 69
//! the one of the levels of the future (Planet-X, Spaceship, Spaceways ...).  The text table is read-only data of the game, so
//! a copy of it with two more texts is made, and the two characters point at the new texts.

use crate::*;

const LANGUAGES: i32 = 5;
/// number of texts of a language in the console game
const TEXTS: i32 = 1246;
const COPY_AT: i32 = 0x240_0000 + 0x10000;
const STR_AT: i32 = 0x240_0000 + 0x18000;

/// (character number, new text)
const NAMES: [(i32, &str); 2] = [(71, "Past TimeSplitter"), (69, "Future TimeSplitter")];

pub fn chr_names_once() {
    static DONE: std::sync::Once = std::sync::Once::new();
    DONE.call_once(|| {
        let mut strs = STR_AT;
        let mut ptrs = Vec::new();
        for (_, name) in NAMES {
            ptrs.push(strs);
            for b in name.bytes() {
                sb(strs, b as i32);
                strs += 1;
            }
            sb(strs, 0);
            strs += 1;
        }
        for lang in 0..LANGUAGES {
            let src = lw(g::languages + 4 * lang);
            if src == 0 {
                continue;
            }
            let dst = COPY_AT + lang * 0x1400;
            // (and the names of the levels of mods, which come after the two of the characters)
            for i in 0..TEXTS + 2 + custom_text_count() {
                sw(dst + 4 * i, lw(src + 4 * i));
            }
            for (k, p) in ptrs.iter().enumerate() {
                sw(dst + 4 * (TEXTS + k as i32), *p);
            }
            sw(g::languages + 4 * lang, dst);
        }
        for (k, (chr, _)) in NAMES.iter().enumerate() {
            // (the first text is the name the menus show, the second the one the level shows: "Killed by ...", the scores)
            sh(g::chrs + 44 * chr, TEXTS + k as i32);
            sh(g::chrs + 44 * chr + 2, TEXTS + k as i32);
        }
    });
    if std::env::var_os("TS_CHRLOG").is_some() {
        let text = |id: i32| {
            let mut at = lw(lw(g::languages + 4 * lw(g::cur_language)) + 4 * id);
            let mut t = String::new();
            while lbu(at) != 0 && t.len() < 40 {
                t.push(lbu(at) as u8 as char);
                at += 1;
            }
            t
        };
        for chr in [69, 71] {
            eprintln!("[chr] {chr}: menu name {:?}, level name {:?}", text(lhu(g::chrs + 44 * chr)), text(lhu(g::chrs + 44 * chr + 2)));
        }
    }
}

/// The save of the game's data after a match is being answered by the port (the prompt is answered Yes, the message after it is
/// acknowledged): see `statsupdate_pageTick` and `mcseq_pageTick` in `tools/patches.py`.
static SAVE_FLOW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn save_flow(on: bool) {
    SAVE_FLOW.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn save_flow_active() -> bool {
    SAVE_FLOW.load(std::sync::atomic::Ordering::Relaxed)
}
