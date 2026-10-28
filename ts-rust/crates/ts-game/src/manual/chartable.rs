//! The table of the characters (`chrs`: 44 bytes each, 80 of them with other data right behind) in the port's memory, with room for 128, so that mods can add characters.
//!
//! A character a mod adds is a copy of one of the table with some fields changed, a model file of its own (a new prop of the port's table of props that names the file), and a name (a new
//! text).  The entries of the game's own characters are copied at the start (after the mods' edits of `chrs` in `patches.txt` and `characters.txt` have been made in the game's own memory);
//! the added ones follow.  Without the `extended-tables` feature the game's own table is used and no character is added.

use crate::*;
use std::sync::Mutex;

#[cfg(feature = "extended-tables")]
pub const CHRS_TABLE: i32 = 0x240_0000 + 0x48000;
#[cfg(not(feature = "extended-tables"))]
pub const CHRS_TABLE: i32 = g::chrs;
/// the game's characters, and the most there can be with the added ones
pub const GAME_CHARACTERS: i32 = 80;
pub const MAX_CHARACTERS: i32 = 128;
pub const CHR_BYTES: i32 = 44;

/// A character that a mod adds.
#[derive(Clone, Debug)]
pub struct NewCharacter {
    /// the number: GAME_CHARACTERS for the first added one, the next for the next
    pub number: i32,
    pub copy_of: i32,
    /// the name shown
    pub name: String,
    /// the model file in the packs (`ob/chrs/chr90.raw`); empty: the model of the character it copies
    pub model_file: String,
    /// the character whose voice (the list of sounds at +4) it has; -1: the voice of the character it copies
    pub voice_of: i32,
    /// changes of the entry: the place in it, the width in bytes and the value
    pub puts: Vec<(i32, i32, u32)>,
}

static NEW_CHARACTERS: Mutex<Vec<NewCharacter>> = Mutex::new(Vec::new());

/// A mod adds a character (before `character_table_init`); a build without `extended-tables` cannot have it.
pub fn register_character(c: NewCharacter) {
    #[cfg(feature = "extended-tables")]
    NEW_CHARACTERS.lock().unwrap().push(c);
    #[cfg(not(feature = "extended-tables"))]
    eprintln!("[mods] character {} ({}) needs a build with extended-tables", c.number, c.name);
}

/// How many characters mods have added (0 without the `extended-tables` feature): the select page lists them after the game's 64.
pub fn added_characters() -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        ADDED.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        0
    }
}

#[cfg(feature = "extended-tables")]
static ADDED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Whether `chr` is a character that a mod added (never in a build without `extended-tables`).
pub fn is_added(chr: i32) -> bool {
    #[cfg(feature = "extended-tables")]
    {
        chr >= GAME_CHARACTERS
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = chr;
        false
    }
}

/// The game's character that the tables of the front end (the animation list, the display's state) are read for: an added character shows and moves as the character it copies, as those
/// tables have the game's 80 entries only.
pub fn front_end_chr(chr: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        let mut chr = chr;
        let copies = COPIES.lock().unwrap();
        while chr >= GAME_CHARACTERS {
            match copies.get((chr - GAME_CHARACTERS) as usize) {
                Some(&of) => chr = of,
                None => return 0,
            }
        }
        chr
    }
    #[cfg(not(feature = "extended-tables"))]
    chr
}

#[cfg(feature = "extended-tables")]
static COPIES: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// Make the table in the port's memory: the game's characters, then the added ones.
#[cfg(feature = "extended-tables")]
pub fn character_table_init() {
    for i in 0..GAME_CHARACTERS * CHR_BYTES / 4 {
        sw(CHRS_TABLE + 4 * i, lw(g::chrs + 4 * i));
    }
    let mut added = NEW_CHARACTERS.lock().unwrap().clone();
    added.sort_by_key(|c| c.number);
    let mut accepted = 0;
    for c in added {
        let next = GAME_CHARACTERS + accepted;
        if c.number != next {
            eprintln!("[mods] character {} is not the next added character ({} is); left out", c.number, next);
            continue;
        }
        if !(0..next).contains(&c.copy_of) || next >= MAX_CHARACTERS {
            eprintln!("[mods] character {}: `copy {}` is not a character that exists at that point, or the table is full", c.number, c.copy_of);
            continue;
        }
        let entry = CHRS_TABLE + CHR_BYTES * next;
        for i in 0..CHR_BYTES / 4 {
            sw(entry + 4 * i, lw(CHRS_TABLE + CHR_BYTES * c.copy_of + 4 * i));
        }
        for &(off, width, bits) in &c.puts {
            match width {
                1 => sb(entry + off, bits as i32),
                2 => sh(entry + off, bits as i32),
                _ => sw(entry + off, bits as i32),
            }
        }
        if (0..next).contains(&c.voice_of) {
            sw(entry + 4, lw(CHRS_TABLE + CHR_BYTES * c.voice_of + 4));
        }
        // the name
        let text = super::customlevels::register_text(&c.name);
        if text >= 0 {
            sh(entry, text);
            sh(entry + 2, text);
        }
        // the model: a prop of its own that copies the prop of the model it was a copy of and names the file (a game character's prop has its own number; one that a mod's edit pointed at a new prop
        // is the character's own number)
        if !c.model_file.is_empty() {
            let source = lw(entry + 8);
            let source = if (0..713).contains(&source) { source } else { c.copy_of };
            let prop = super::modelprops::new_prop(source, &format!("CHR{}", next), &c.model_file);
            if prop >= 0 {
                sw(entry + 8, prop);
            } else {
                eprintln!("[mods] character {}: no room for another model prop", c.number);
            }
            // (the same file for the front end, which shows the character on the select page: a prop of its own copying the front end prop)
            let front_source = lw(entry + 12);
            let front_source = if (0..713).contains(&front_source) { front_source } else { 80 + c.copy_of };
            let front = super::modelprops::new_prop(front_source, &format!("FE{}", next), &c.model_file);
            if front >= 0 {
                sw(entry + 12, front);
            }
        }
        eprintln!("[mods] character {} \"{}\" added (a copy of {}{})", next, c.name, c.copy_of, if c.model_file.is_empty() { String::new() } else { format!(", model {}", c.model_file) });
        accepted += 1;
        ADDED.store(accepted, std::sync::atomic::Ordering::Relaxed);
        COPIES.lock().unwrap().push(c.copy_of);
    }
}

#[cfg(not(feature = "extended-tables"))]
pub fn character_table_init() {}
