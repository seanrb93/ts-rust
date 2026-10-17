//! Five custom weapon sets instead of one.
//!
//! The game has one custom set (`gunzpack`, five weapons, "Custom") and nine presets; `gv_weaponset` is 0 for the custom set and 1..9 for
//! a preset.  The port adds the values 10..13 for four more custom sets.  The game's code only knows the one custom set, so the set
//! being used is always the one in `gunzpack`: whenever the game asks which set is chosen (`weaponset_effective`, which replaces its reads of
//! `gv_weaponset`) the sets are swapped in and out of that place, and the value it is told is 0 for any custom set.  The sets are kept in
//! `weapon_sets.txt` in the save folder (the game itself forgets its custom set when it ends).

use crate::*;
use std::sync::Mutex;

const SETS: usize = 5;
/// the order of the sets when the player changes the weapon set: the custom ones first, then the presets
const ORDER: [i32; 14] = [0, 10, 11, 12, 13, 1, 2, 3, 4, 5, 6, 7, 8, 9];
/// the game's list of the names of the weapon sets (built when the front end starts) and the port's copy of it, with the new sets
const GAME_LIST: i32 = 0x1fc1908;
const LIST_AT: i32 = 0x240_0000 + 0xa000;
const TEXT_AT: i32 = 0x240_0000 + 0xa200;

struct State {
    /// the custom set that is in `gunzpack` now
    loaded: usize,
    slots: [[i32; 5]; SETS],
    /// what the file has
    saved: [[i32; 5]; SETS],
    started: bool,
}

static STATE: Mutex<State> = Mutex::new(State { loaded: 0, slots: [[-1; 5]; SETS], saved: [[-1; 5]; SETS], started: false });

fn file() -> std::path::PathBuf {
    ts_sdk::save_root().join("weapon_sets.txt")
}

fn read_gunzpack() -> [i32; 5] {
    let mut a = [0; 5];
    for (k, v) in a.iter_mut().enumerate() {
        *v = lw(g::gunzpack + 4 * k as i32);
    }
    a
}

fn write_gunzpack(a: &[i32; 5]) {
    for (k, v) in a.iter().enumerate() {
        sw(g::gunzpack + 4 * k as i32, *v);
    }
}

/// The index of the custom set that a value of `gv_weaponset` stands for.
fn custom_index(v: i32) -> Option<usize> {
    match v {
        0 => Some(0),
        10..=13 => Some((v - 9) as usize),
        _ => None,
    }
}

/// The game's list of names, with "Custom 1" to "Custom 5" for the custom sets (the list is read by value).
fn fix_list() {
    let gv = g::gv_weaponset;
    let mut at = TEXT_AT;
    // (the game's own entries 1..9 are copied every time: the front end builds them again when the language changes)
    for k in 1..10 {
        sw(LIST_AT + 8 * k, lw(GAME_LIST + 8 * k));
        sw(LIST_AT + 8 * k + 4, lw(GAME_LIST + 8 * k + 4));
    }
    for (k, v) in [(0, 0), (10, 10), (11, 11), (12, 12), (13, 13)] {
        let name = format!("Custom {}", custom_index(v).map_or(0, |i| i + 1));
        let s = put_str(&mut at, &name);
        sw(LIST_AT + 8 * k, v);
        sw(LIST_AT + 8 * k + 4, s);
    }
    sw(LIST_AT + 8 * 14, 0);
    sw(LIST_AT + 8 * 14 + 4, 0);
    if lw(gv + 16) != LIST_AT {
        sw(gv + 16, LIST_AT);
        sw(gv + 8, 13);
    }
}

/// Swap the custom sets so that the one `v` stands for is in `gunzpack`; write the file when a set changed.
fn sync(v: i32) {
    let mut st = STATE.lock().unwrap();
    if !st.started {
        st.started = true;
        if let Ok(text) = std::fs::read_to_string(file()) {
            for (i, line) in text.lines().take(SETS).enumerate() {
                let nums: Vec<i32> = line.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                if nums.len() == 5 {
                    st.slots[i].copy_from_slice(&nums);
                }
            }
            st.saved = st.slots;
        }
        let first = st.slots[0];
        write_gunzpack(&first);
    }
    // what is in gunzpack belongs to the set that is loaded
    let loaded = st.loaded;
    st.slots[loaded] = read_gunzpack();
    if let Some(t) = custom_index(v) {
        if t != st.loaded {
            let set = st.slots[t];
            write_gunzpack(&set);
            st.loaded = t;
        }
    }
    if st.slots != st.saved {
        st.saved = st.slots;
        let text: String = st.slots.iter().map(|s| format!("{} {} {} {} {}\n", s[0], s[1], s[2], s[3], s[4])).collect();
        let _ = std::fs::write(file(), text);
    }
}

/// What the game's code is told is the weapon set: any custom set is 0 (its weapons are in `gunzpack` by then), a preset is itself.
pub fn weaponset_effective() -> i32 {
    fix_list();
    let v = lw(g::gv_weaponset);
    sync(v);
    if custom_index(v).is_some() {
        0
    } else {
        v
    }
}

/// Is the weapon set that is chosen a custom one (the port's own code: `gv_weaponset` is read as it is)?
pub fn weaponset_is_custom() -> bool {
    custom_index(lw(g::gv_weaponset)).is_some()
}

/// `weaponSetInc` / `weaponSetDec`: the next / previous weapon set that can be chosen.
pub fn weapon_set_step(dir: i32) {
    soundStart(141, 0);
    fix_list();
    let gv = g::gv_weaponset;
    let n = ORDER.len() as i32;
    let mut i = ORDER.iter().position(|&v| v == lw(gv)).unwrap_or(0) as i32;
    for _ in 0..n {
        i = (i + n + dir) % n;
        let v = ORDER[i as usize];
        if v == 0 || v >= 10 || weaponsetAvail(v - 1) != 0 {
            sw(gv, v);
            break;
        }
    }
    sync(lw(gv));
}
