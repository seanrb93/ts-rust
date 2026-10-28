//! Props for the model files of mods: a prop is an entry of the table of props (`proptable`, 28 bytes) that names an object file; a character, a weapon or a pickup uses the prop number to
//! make its model.  A mod's model is a new prop that copies the entry of a prop of the game (so it has the same kind, skeleton and flags) and names the mod's file instead.
//!
//! The numbers from 1073 to 1099 are free in the port's table (the rooms of a level of a mod are the numbers from 512 to 1072), so a mod can have 27 models of its own.

use crate::*;
use std::sync::atomic::{AtomicI32, Ordering};

const FIRST_NEW_PROP: i32 = 1073;
const LAST_NEW_PROP: i32 = 1099;
/// where the strings of the added props are kept
const STRINGS_AT: i32 = 0x240_0000 + 0x4a000;
static NEXT_PROP: AtomicI32 = AtomicI32::new(FIRST_NEW_PROP);
static STRINGS_USED: AtomicI32 = AtomicI32::new(0);

/// A new prop that is a copy of the prop `copy_of` and names the file `path` (in the packs, such as `ob/chrs/chr90.raw`); its number, or -1 when there is no more room (or the build has no
/// `extended-tables`).
pub fn new_prop(copy_of: i32, label: &str, path: &str) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        if ts_sdk::disc_has(&format!("host0:{path}")) == Some(false) {
            eprintln!("[mods] the model file {path} is not in the packs or the mod: left out (the copy's own model is used)");
            return -1;
        }
        let prop = NEXT_PROP.load(Ordering::Relaxed);
        if prop > LAST_NEW_PROP || !(0..713).contains(&copy_of) {
            return -1;
        }
        NEXT_PROP.store(prop + 1, Ordering::Relaxed);
        let from = super::proptable::PROPTABLE + 28 * copy_of;
        let to = super::proptable::PROPTABLE + 28 * prop;
        for w in 0..7 {
            sw(to + 4 * w, lw(from + 4 * w));
        }
        let mut at = STRINGS_AT + STRINGS_USED.load(Ordering::Relaxed);
        let name = super::avmenu::put_str(&mut at, label);
        let file = super::avmenu::put_str(&mut at, &format!("host0:{path}"));
        STRINGS_USED.store(at - STRINGS_AT, Ordering::Relaxed);
        sw(to + 8, name);
        sw(to + 12, file);
        if std::env::var_os("TS_COMBOLOG").is_some() {
            eprintln!("[prop] {prop} = a copy of {copy_of}, file host0:{path}");
        }
        prop
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = (copy_of, label, path, FIRST_NEW_PROP, LAST_NEW_PROP, STRINGS_AT, &NEXT_PROP, &STRINGS_USED, Ordering::Relaxed);
        -1
    }
}
