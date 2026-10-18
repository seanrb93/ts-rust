//! The levels that mods add (`levels.txt` of a mod): the mod loader tells the game here.

use std::sync::Mutex;

/// A level of a mod: it has the number, the name in the level list, the music, and the data and set-up of the level `like` (a mod may put
/// files of its own where the level's are: `bg/level<number>/level<number>.raw`, `pad/data/level<number>.raw`, `propsetup/data/level<number>.raw`).
#[derive(Clone, Debug)]
pub struct CustomLevel {
    pub number: i32,
    pub name: String,
    pub like: i32,
    /// the music (a track, 1 to 27), 0 for the one of the level it is like
    pub tune: i32,
    /// the arcade modes it has (bits: 1 deathmatch, 2 bagtag, 4 capture the bag, 8 escort, 16 knockout, 32 last stand), none given: its model's
    pub modes: Option<u32>,
}

static LEVELS: Mutex<Vec<CustomLevel>> = Mutex::new(Vec::new());

pub fn set_custom_levels(v: Vec<CustomLevel>) {
    *LEVELS.lock().unwrap() = v;
}

pub fn custom_levels() -> Vec<CustomLevel> {
    LEVELS.lock().unwrap().clone()
}
