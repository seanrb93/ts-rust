//! Presets of the game modes that mods give (`modes.txt`): the options a mode starts with and its name in the menus.
//!
//! The game keeps the options of each mode in a record of 32 bytes (`0x1fc2c98 + 32 * mode`, the mode being 1 BagTag, 2 Capture the Bag, 3 Deathmatch, 4 Escort,
//! 5 KnockOut, 6 Last Stand): team play, friendly fire, radar, time limit, kill limit, score limit, scoring method and one shot kills, one word each.  `gameoptionsMake` fills
//! them when a game starts and the menus keep what the player changes in them (`storeGameModeOptions`, `getGameModeOptions`), so a preset is written into the record
//! right after `gameoptionsMake`, and a player can still change the options on the Game Options page.
//!
//! The name of a mode is the text number in `gamemodedata` (12 bytes a mode); a mod's name is a new text of the port's tables (`customlevels::register_text`), so it needs
//! a build with `extended-tables`.

use crate::*;
use std::sync::Mutex;

const RECORDS: i32 = 0x1fc2c98;

/// What a mod says about a mode: the changes of its record (the place in it, the value) and its name.
#[derive(Clone, Debug)]
pub struct ModePreset {
    pub kind: i32,
    pub name: String,
    pub fields: Vec<(i32, i32)>,
}

static PRESETS: Mutex<Vec<ModePreset>> = Mutex::new(Vec::new());

/// A mod gives a preset (before the game starts); a later one wins for the fields it sets.
pub fn register_mode_preset(p: ModePreset) {
    if !(1..=6).contains(&p.kind) {
        return;
    }
    if !p.name.is_empty() {
        #[cfg(feature = "extended-tables")]
        {
            let text = super::customlevels::register_text(&p.name);
            if text >= 0 {
                sh(g::gamemodedata + 12 * p.kind, text);
            }
        }
        #[cfg(not(feature = "extended-tables"))]
        eprintln!("[mods] the name of mode {} needs a build with extended-tables", p.kind);
    }
    PRESETS.lock().unwrap().push(p);
}

/// Write the presets into the records of the options (`gameoptionsMake` has just filled them).
pub fn mode_presets_apply() {
    for p in PRESETS.lock().unwrap().iter() {
        let record = RECORDS + (p.kind << 5);
        for &(at, value) in &p.fields {
            sw(record + at, value);
        }
        if std::env::var_os("TS_COMBOLOG").is_some() {
            eprintln!("[mode] {} record: {:?}", p.kind, (0..8).map(|i| lw(record + 4 * i)).collect::<Vec<_>>());
        }
    }
}
