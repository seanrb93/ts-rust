//! Dual versions of the weapons that have none: "Uzi (x2)" next to "Uzi", for the weapons that the game only has in one hand.
//!
//! The pairs are combinations of their own (`weapontable.rs`: numbers 33 and up), so a set of weapons can have a weapon and its pair, the
//! weapon page lists them after the single weapons (`weapons_list_extend`), `combo_name` gives their names.  With the Dual Wield
//! Everything option off the pairs are not listed, and an entry of a set that is a pair is the single weapon (`dual_combo`).

use crate::*;
use std::sync::Mutex;

/// the first number of the added pairs
const FIRST: i32 = GAME_COMBOS;
/// the game's list of the weapons of the weapon page and its count
const LIST: i32 = 0x1fc2a00;
const COUNT: i32 = 0x3aff4c;
const NAME_AT: i32 = 0x240_0000 + 0xb000;

/// The name of a combination or of an added pair ("Uzi (x2)"), as text in memory.
pub fn combo_name(id: i32) -> i32 {
    let lang = lw(g::cur_language);
    let table = lw(g::languages + 4 * lang);
    match single_of(id) {
        None => lw(table + 4 * lh(GUNCOMBOS + 8 * id + 6)),
        Some(single) => {
            let base = lw(table + 4 * lh(GUNCOMBOS + 8 * single + 6));
            let mut name = String::new();
            let mut at = base;
            while lbu(at) != 0 && name.len() < 40 {
                name.push(lbu(at) as u8 as char);
                at += 1;
            }
            let mut at = NAME_AT + 0x40 * (id - FIRST);
            put_str(&mut at, &format!("{name} (x2)"))
        }
    }
}

static WEAPONS_PAGE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Is the weapon page being shown (its tick ran in this frame or the one before)?
pub fn weapons_page_active() -> bool {
    let at = WEAPONS_PAGE.load(std::sync::atomic::Ordering::Relaxed);
    at != 0 && (ts_sdk::frame_count() as u64) <= at + 2
}

/// The name that the message of a pickup of the weapon `combo` shows: `name` (the game's) or, for an added pair, its own.
pub fn pickup_name(combo: i32, name: i32) -> i32 {
    if single_of(combo).is_some() {
        combo_name(combo)
    } else {
        name
    }
}

/// The dual versions, the left fist: what a script of a mod says (a build without extensions has none of it, as the original game).
fn pair_listed(single: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        script_pair_listed(single)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = single;
        false
    }
}

fn pair_allowed(pair: i32, single: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        script_pair_allowed(pair, single)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (pair, single);
        false
    }
}

fn fists_alternate() -> bool {
    #[cfg(feature = "extensions")]
    {
        script_alternate_fists()
    }
    #[cfg(not(feature = "extensions"))]
    {
        false
    }
}

/// `weapons_pageTick`, after it made the list of the weapons that can be chosen: the added pairs follow the singles they are made from.
pub fn weapons_list_extend() {
    WEAPONS_PAGE.store(ts_sdk::frame_count() as u64, std::sync::atomic::Ordering::Relaxed);
    let mut n = lw(COUNT).clamp(0, 40);
    // (the weapons of mods come after the game's, singles first, then their pairs below)
    for id in added_singles() {
        if n < 64 && !(0..n).any(|i| lw(LIST + 4 * i) == id) {
            sw(LIST + 4 * n, id);
            n += 1;
        }
    }
    sw(COUNT, n);
    let have: Vec<i32> = (0..n).map(|i| lw(LIST + 4 * i)).collect();
    let extra: Vec<i32> = (0..dual_count()).filter(|&k| single_of(FIRST + k).map_or(false, |s| have.contains(&s) && pair_listed(s))).map(|k| FIRST + k).collect();
    if extra.is_empty() {
        return;
    }
    let mut count = n;
    for id in extra {
        if !have.contains(&id) && count < 64 {
            sw(LIST + 4 * count, id);
            count += 1;
        }
    }
    sw(COUNT, count);
}

/// `frontmpgGetGunComboNum`: the combination the set has at place `place`, `combo` as the game found it.  An added pair is the single
/// weapon when the pairs are switched off.
pub fn dual_combo(_place: i32, combo: i32) -> i32 {
    if std::env::var_os("TS_COMBOLOG").is_some() {
        eprintln!("[combo] set place {_place} combo {combo} set {} gunzpack {} state {}", weaponset_effective(), lw(g::gunzpack + 4 * _place), lw(g::gamestate));
    }
    match single_of(combo) {
        Some(single) if !pair_allowed(combo, single) => single,
        Some(_) => combo,
        // (the single weapon of a mod is a combination past the pairs: it stays)
        None if combo >= FIRST && !added_singles().contains(&combo) => 0,
        None => combo,
    }
}

/// A game starts (`gameReset`).
pub fn dual_reset() {
    hands_reset();
    own_clear_all();
    mm_clear(PUNCHES_AT);
}

// ---- the left fist ------------------------------------------------------------------------------------------------------------
//
// Without a weapon the player attacks with one arm: a model of the arm (a prop of its own, the "puppet") plays a swipe.  The game has no
// animation for the left arm, so every second swipe is drawn mirrored left to right: the display list gets a LABEL write around the
// puppet's picture (`propGfxRoom`), which the graphics code takes as "mirror what follows".  The hit test is the game's own.

/// (puppet prop, the next swipe is the left fist's): a table in the game's memory (see `memmap.rs`)
const PUNCHES_AT: i32 = 0x240_0000 + 0xc800;

/// The puppet `prop` starts a swipe (`gunUpdate`): the swipes take turns.
pub fn punch_swing(prop: i32) {
    if !fists_alternate() {
        return;
    }
    let left = mm_get(PUNCHES_AT, prop).map_or(false, |v| v != 0);
    mm_set(PUNCHES_AT, prop, if mm_get(PUNCHES_AT, prop).is_some() { !left as u64 } else { 0 });
}

/// Is the picture of this prop mirrored (`propGfxRoom`)?
pub fn punch_mirrored(prop: i32) -> bool {
    fists_alternate() && mm_get(PUNCHES_AT, prop).map_or(false, |v| v != 0)
}

const SCRATCH: i32 = 0x240_0000 + 0xc000;

/// The hit test of a swipe (`gunUpdate`) uses two points of the puppet's arm: when the swipe is the left fist's they are put on the
/// other side of the plane through the player that goes along the way he looks (`prop` is the puppet, `chr` the player's).
pub fn punch_mirror_points(prop: i32, chr: i32, p1: i32, p2: i32) {
    if !punch_mirrored(prop) {
        return;
    }
    // the player's right-hand direction: the game's own rotation by his heading of the sideways unit vector
    matrixRotY(SCRATCH, lwc1(chr + 80) * lwc1(0x3ad3bc) / 1.8e+02_f32);
    for (k, v) in [1.0_f32, 0.0, 0.0, 0.0].iter().enumerate() {
        swc1(SCRATCH + 64 + 4 * k as i32, *v);
    }
    matrixVecMulAligned(&[SCRATCH as i64, (SCRATCH + 64) as i64]);
    let mut r = [lwc1(SCRATCH + 64), lwc1(SCRATCH + 68), lwc1(SCRATCH + 72)];
    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    if len < 1.0e-4 {
        return;
    }
    for c in r.iter_mut() {
        *c /= len;
    }
    let origin = [lwc1(chr + 48), lwc1(chr + 52), lwc1(chr + 56)];
    for p in [p1, p2] {
        let q = [lwc1(p), lwc1(p + 4), lwc1(p + 8)];
        let d = (q[0] - origin[0]) * r[0] + (q[1] - origin[1]) * r[1] + (q[2] - origin[2]) * r[2];
        for k in 0..3 {
            swc1(p + 4 * k as i32, q[k] - 2.0 * d * r[k]);
        }
    }
}
