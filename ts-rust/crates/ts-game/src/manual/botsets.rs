//! Five custom bot sets instead of three (the same way as the weapon sets, see `weaponsets.rs`).
//!
//! `gv_chrset` is 0..7 for the preset bot sets and 8..10 for the three custom ones, whose ten characters each are bytes at `SETS_AT`.
//! The port adds the values 11 and 12.  The game's code only knows the three, so the sets "Custom 3", 4 and 5 share the storage of the
//! third: whenever the game asks which bot set is chosen (`chrset_effective`, which replaces its reads of `gv_chrset`) the chosen one
//! is swapped into that place and the value the game is told is 10.  The sets are kept in `bot_sets.txt` in the save folder.

use crate::*;
use std::sync::Mutex;

/// the custom sets of the game: ten bytes each (a byte of -1 is no bot)
const SETS_AT: i32 = 0x1fc2b28;
const GAME_LIST: i32 = 0x1fc1960;
const LIST_AT: i32 = 0x240_0000 + 0xa800;
const TEXT_AT: i32 = 0x240_0000 + 0xaa00;

struct State {
    /// which of the three sets that share the third's place is in it (0..2)
    loaded: usize,
    slots: [[u8; 10]; 3],
    saved: [[u8; 10]; 3],
    started: bool,
}

static STATE: Mutex<State> = Mutex::new(State { loaded: 0, slots: [[255; 10]; 3], saved: [[255; 10]; 3], started: false });

fn file() -> std::path::PathBuf {
    ts_sdk::save_root().join("bot_sets.txt")
}

fn read_place() -> [u8; 10] {
    let mut a = [0u8; 10];
    for (k, v) in a.iter_mut().enumerate() {
        *v = lbu(SETS_AT + 20 + k as i32) as u8;
    }
    a
}

fn write_place(a: &[u8; 10]) {
    for (k, v) in a.iter().enumerate() {
        sb(SETS_AT + 20 + k as i32, *v as i32);
    }
}

/// The names of the sets: the game's own (the list is built when the front end starts) and "Custom 4" and "Custom 5".
fn fix_list() {
    let gv = g::gv_chrset;
    for k in 0..11 {
        sw(LIST_AT + 8 * k, lw(GAME_LIST + 8 * k));
        sw(LIST_AT + 8 * k + 4, lw(GAME_LIST + 8 * k + 4));
    }
    let mut at = TEXT_AT;
    for k in [11, 12] {
        let s = put_str(&mut at, &format!("Custom {}", k - 7));
        sw(LIST_AT + 8 * k, k as i32);
        sw(LIST_AT + 8 * k + 4, s);
    }
    sw(LIST_AT + 8 * 13, 0);
    sw(LIST_AT + 8 * 13 + 4, 0);
    if lw(gv + 16) != LIST_AT {
        sw(gv + 16, LIST_AT);
        sw(gv + 8, 12);
    }
}

fn sync(v: i32) {
    let mut st = STATE.lock().unwrap();
    if !st.started {
        st.started = true;
        st.slots[0] = read_place();
        if let Ok(text) = std::fs::read_to_string(file()) {
            for (i, line) in text.lines().take(3).enumerate() {
                let n: Vec<i32> = line.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                if n.len() == 10 {
                    for k in 0..10 {
                        st.slots[i][k] = n[k] as u8;
                    }
                }
            }
        }
        st.saved = st.slots;
        let first = st.slots[0];
        write_place(&first);
    }
    let loaded = st.loaded;
    st.slots[loaded] = read_place();
    if (10..=12).contains(&v) {
        let t = (v - 10) as usize;
        if t != st.loaded {
            let set = st.slots[t];
            write_place(&set);
            st.loaded = t;
        }
    }
    if st.slots != st.saved {
        st.saved = st.slots;
        let text: String = st.slots.iter().map(|s| format!("{}\n", s.iter().map(|&b| (b as i8).to_string()).collect::<Vec<_>>().join(" "))).collect();
        let _ = std::fs::write(file(), text);
    }
}

/// The bot set the game's code is told is chosen: the sets 11 and 12 are 10 (their characters are in the third's place by then).
pub fn chrset_effective() -> i32 {
    fix_list();
    let v = lw(g::gv_chrset);
    sync(v);
    v.min(10)
}

/// `chrsetDec` / `chrsetInc`: the previous / next bot set that can be chosen (the presets that are unlocked, the custom sets).
pub fn chrset_step(dir: i32) {
    soundStart(141, 0);
    fix_list();
    let gv = g::gv_chrset;
    let mut v = lw(gv);
    for _ in 0..13 {
        v = (v + 13 + dir) % 13;
        if v >= 8 || botsetAvail(v) != 0 {
            break;
        }
    }
    sw(gv, v);
    sync(v);
}
