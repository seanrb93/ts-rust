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

/// A way to play a mode that a mod offers on the Options page of the arcade match (the row "Rules"): a name and the options it sets, the others staying as the mode has them.
#[derive(Clone, Debug)]
pub struct ModeVariant {
    pub kind: i32,
    pub name: String,
    pub fields: Vec<(i32, i32)>,
}

static VARIANTS: Mutex<Vec<ModeVariant>> = Mutex::new(Vec::new());
/// the records as the game and the presets left them when the game started: what "Standard" stands for
static BASE: Mutex<[[i32; 8]; 7]> = Mutex::new([[0; 8]; 7]);

pub fn register_mode_variant(v: ModeVariant) {
    if (1..=6).contains(&v.kind) {
        VARIANTS.lock().unwrap().push(v);
    }
}

/// The names of the variants of a mode.
pub fn variants_of(kind: i32) -> Vec<String> {
    VARIANTS.lock().unwrap().iter().filter(|v| v.kind == kind).map(|v| v.name.clone()).collect()
}

/// Set the options of the menus for the variant `index` of the mode `kind` (0: the mode as it starts, 1 and on: the variants in the order of the mods).
pub fn apply_variant(kind: i32, index: usize) {
    if !(1..=6).contains(&kind) {
        return;
    }
    let mut words = BASE.lock().unwrap()[kind as usize];
    if index > 0 {
        if let Some(v) = VARIANTS.lock().unwrap().iter().filter(|v| v.kind == kind).nth(index - 1) {
            for &(at, value) in &v.fields {
                words[(at / 4) as usize] = value;
            }
        }
    }
    // (the order of `getGameModeOptions`)
    let targets = [g::gv_teamplay, g::gv_friendfire, g::gv_dispradar, g::gv_timelimit, g::gv_killlimit, g::gv_scorelimit, g::gv_scoremethod, g::gv_oneshotkill];
    for (t, w) in targets.iter().zip(words) {
        gvSetValue(*t, w);
    }
}

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
    let mut presets = PRESETS.lock().unwrap().clone();
    presets.retain(|p| !p.fields.is_empty());
    for p in presets.iter() {
        let record = RECORDS + (p.kind << 5);
        for &(at, value) in &p.fields {
            sw(record + at, value);
        }
        if std::env::var_os("TS_COMBOLOG").is_some() {
            eprintln!("[mode] {} record: {:?}", p.kind, (0..8).map(|i| lw(record + 4 * i)).collect::<Vec<_>>());
        }
    }
    mode_base_save();
}

/// Remember the records as the game starts with them (after the presets), for "Standard" (`gameoptionsMake` calls this through `mode_presets_x`).
pub fn mode_base_save() {
    let mut base = BASE.lock().unwrap();
    for kind in 1..=6usize {
        for i in 0..8 {
            base[kind][i] = lw(RECORDS + ((kind as i32) << 5) + 4 * i as i32);
        }
    }
}
