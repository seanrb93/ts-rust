//! The table of weapon combinations in the port's memory, with the dual versions of the weapons as combinations of their own.
//!
//! The game keeps its 33 combinations (`guncombos`: rank, left gun, right gun, name; 8 bytes each) and the order the weapons are cycled
//! in (`gunsinventory`) in its data, with other data right behind them, so they cannot grow there.  The generated code reads them through
//! `GUNCOMBOS` and `GUNSINV` (`tools/patches.py`, GLOBAL), which are places in the port's memory with room for 64 combinations.
//!
//! A pair of a weapon is a combination with a left gun ("Pistol (x2)"), and the game has them for eight of its weapons.  For each of the
//! other weapons one is added here, numbered from 33 up (`weapon_table_init`): the same gun in both hands, the rank a little below the
//! single weapon's, the name of the single weapon (`combo_name` adds " (x2)"), after the single weapon in the order of the cycling.  A
//! level can then have the single weapon and its pair at the same time, as the game's own pairs.
//!
//! What the game keeps per combination in other places has room for 33 only: what a character owns (`chr + 488`, 33 words, right before
//! the ammo) is kept for the added combinations here (`own_get`, `own_set`, `own_clear`; the generated code asks through them), and the
//! statistics count a pair with its single weapon (`stats_combo`).

use crate::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

/// the places of the tables in the port's memory
#[cfg(feature = "extended-tables")]
pub const GUNCOMBOS: i32 = 0x240_0000 + 0x40000;
#[cfg(feature = "extended-tables")]
pub const GUNSINV: i32 = 0x240_0000 + 0x40400;
/// (without `extended-tables` the game's own tables are used, with its 33 combinations)
#[cfg(not(feature = "extended-tables"))]
pub const GUNCOMBOS: i32 = g::guncombos;
#[cfg(not(feature = "extended-tables"))]
pub const GUNSINV: i32 = g::gunsinventory;
/// the table of the weapons (400 bytes each): the game's 30, then the weapons of mods
#[cfg(feature = "extended-tables")]
pub const GUNS_TABLE: i32 = 0x240_0000 + 0x41000;
#[cfg(not(feature = "extended-tables"))]
pub const GUNS_TABLE: i32 = g::guns;
/// the game's own weapons, and the most there can be with the added ones
pub const GAME_GUNS: i32 = 30;
pub const GUN_BYTES: i32 = 400;
pub const MAX_GUNS: i32 = 64;
/// room for this many combinations
pub const MAX_COMBOS: i32 = 64;
/// the game's own combinations
pub const GAME_COMBOS: i32 = 33;

/// A weapon that a mod adds: a copy of one of the weapons of the table with some of its fields changed (`weapons.txt` of a mod).
#[derive(Clone, Debug)]
pub struct NewWeapon {
    /// the number of the weapon: GAME_GUNS for the first added one, the next for the next
    pub number: i32,
    /// the weapon it is a copy of
    pub copy_of: i32,
    /// the name shown (empty: the name of the weapon it copies)
    pub name: String,
    /// changes of the entry: the place in it, the width in bytes (1, 2 or 4) and the value
    pub puts: Vec<(i32, i32, u32)>,
    /// the most that can be carried of its kind of ammo, when the mod says
    pub max_ammo: Option<i32>,
}

static NEW_WEAPONS: Mutex<Vec<NewWeapon>> = Mutex::new(Vec::new());

/// A mod adds a weapon (before `weapon_table_init`); a build without `extended-tables` cannot have it.
pub fn register_weapon(w: NewWeapon) {
    #[cfg(feature = "extended-tables")]
    NEW_WEAPONS.lock().unwrap().push(w);
    #[cfg(not(feature = "extended-tables"))]
    eprintln!("[mods] weapon {} ({}) needs a build with extended-tables", w.number, w.name);
}

static COUNT: AtomicI32 = AtomicI32::new(GAME_COMBOS);
/// the single weapons that got a pair: the pair of `SINGLES[k]` is the combination `GAME_COMBOS + k`
static SINGLES: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// The number of combinations (the game's and the added ones).
pub fn combo_count() -> i32 {
    COUNT.load(Ordering::Relaxed)
}

/// The single weapon an added combination is the pair of.
pub fn single_of(id: i32) -> Option<i32> {
    if id < GAME_COMBOS {
        return None;
    }
    SINGLES.lock().unwrap().get((id - GAME_COMBOS) as usize).copied()
}

/// The added pair of a single weapon (a combination), if it has one.
pub fn dual_of(single: i32) -> Option<i32> {
    SINGLES.lock().unwrap().iter().position(|&s| s == single).map(|k| GAME_COMBOS + k as i32)
}

/// The number of the added pairs.
pub fn dual_count() -> i32 {
    SINGLES.lock().unwrap().len() as i32
}

/// Make the tables in the port's memory: the game's own, then the pairs (once, before anything runs).
#[cfg(feature = "extended-tables")]
pub fn weapon_table_init() {
    for i in 0..GAME_GUNS * 100 {
        sw(GUNS_TABLE + 4 * i, lw(g::guns + 4 * i));
    }
    for i in 0..GAME_COMBOS * 2 {
        sw(GUNCOMBOS + 4 * i, lw(g::guncombos + 4 * i));
    }
    let entry = |k: i32| (lh(GUNCOMBOS + 8 * k), lh(GUNCOMBOS + 8 * k + 2), lh(GUNCOMBOS + 8 * k + 4), lhu(GUNCOMBOS + 8 * k + 6));
    // (the weapons that have a pair already, and the fists, which have no gun)
    let paired: Vec<i32> = (1..GAME_COMBOS).map(entry).filter(|c| c.1 != -1 && c.1 == c.2).map(|c| c.2).collect();
    let singles: Vec<i32> = (1..GAME_COMBOS).filter(|&k| entry(k).1 == -1 && entry(k).2 != -1 && !paired.contains(&entry(k).2)).collect();
    // the weapons of mods: each gets a single combination and a pair, after the pairs of the game's weapons
    let mut added = NEW_WEAPONS.lock().unwrap().clone();
    added.sort_by_key(|w| w.number);
    let mut accepted: Vec<NewWeapon> = Vec::new();
    for w in added {
        let next = GAME_GUNS + accepted.len() as i32;
        if w.number != next {
            eprintln!("[mods] weapon {} is not the next added weapon ({} is); left out", w.number, next);
        } else if !(0..next).contains(&w.copy_of) {
            eprintln!("[mods] weapon {}: `copy {}` is not a weapon that exists at that point", w.number, w.copy_of);
        } else if next >= MAX_GUNS || GAME_COMBOS + singles.len() as i32 + 2 * (accepted.len() as i32 + 1) > MAX_COMBOS {
            eprintln!("[mods] weapon {}: the tables are full", w.number);
        } else {
            accepted.push(w);
        }
    }
    let n_new = accepted.len() as i32;
    let count = GAME_COMBOS + singles.len() as i32 + 2 * n_new;
    for w in &accepted {
        let at = GUNS_TABLE + GUN_BYTES * w.number;
        for i in 0..GUN_BYTES / 4 {
            sw(at + 4 * i, lw(GUNS_TABLE + GUN_BYTES * w.copy_of + 4 * i));
        }
        for &(off, width, bits) in &w.puts {
            match width {
                1 => sb(at + off, bits as i32),
                2 => sh(at + off, bits as i32),
                _ => sw(at + off, bits as i32),
            }
        }
        if let Some(m) = w.max_ammo {
            let kind = lw(at + 16);
            if (0..15).contains(&kind) {
                sw(g::ammoinfo + 8 * kind, m);
            }
        }
    }
    for (k, &s) in singles.iter().enumerate() {
        let at = GUNCOMBOS + 8 * (GAME_COMBOS + k as i32);
        let (rank, _, gun, name) = entry(s);
        sh(at, (rank - 5).max(1));
        sh(at + 2, gun);
        sh(at + 4, gun);
        sh(at + 6, name as i32);
    }
    let first_new_single = GAME_COMBOS + singles.len() as i32 + n_new;
    let mut new_singles = Vec::new();
    for (j, w) in accepted.iter().enumerate() {
        // (the combination of the weapon it copies gives the rank and the name)
        let model = (1..GAME_COMBOS).map(entry).find(|c| c.1 == -1 && c.2 == w.copy_of).or_else(|| (1..GAME_COMBOS).map(entry).find(|c| c.2 == w.copy_of));
        let (rank, name) = model.map_or((50, 0), |c| (c.0 as i32, c.3 as i32));
        let text = if w.name.is_empty() { name } else { super::customlevels::register_text(&w.name) };
        let text = if text < 0 { name } else { text };
        let dual = GAME_COMBOS + singles.len() as i32 + j as i32;
        let single = first_new_single + j as i32;
        let d_at = GUNCOMBOS + 8 * dual;
        sh(d_at, (rank - 5).max(1));
        sh(d_at + 2, w.number);
        sh(d_at + 4, w.number);
        sh(d_at + 6, text);
        let s_at = GUNCOMBOS + 8 * single;
        sh(s_at, rank);
        sh(s_at + 2, -1);
        sh(s_at + 4, w.number);
        sh(s_at + 6, text);
        new_singles.push(single);
    }
    // the order of the cycling: each pair follows its single weapon
    let mut order = Vec::new();
    for i in 0..GAME_COMBOS {
        let v = lw(g::gunsinventory + 4 * i);
        order.push(v);
        if let Some(k) = singles.iter().position(|&s| s == v) {
            order.push(GAME_COMBOS + k as i32);
        }
    }
    for j in 0..n_new {
        order.push(first_new_single + j);
        order.push(GAME_COMBOS + singles.len() as i32 + j);
    }
    for (i, v) in order.iter().enumerate() {
        sw(GUNSINV + 4 * i as i32, *v);
    }
    COUNT.store(count, Ordering::Relaxed);
    let mut single_of_dual = singles;
    single_of_dual.extend(new_singles);
    *SINGLES.lock().unwrap() = single_of_dual;
    if std::env::var_os("TS_COMBOLOG").is_some() {
        eprintln!("[combo] table: {} combinations, order {:?}", count, order);
    }
}

/// The game's own tables are used as they are: no pairs are added.
#[cfg(not(feature = "extended-tables"))]
pub fn weapon_table_init() {}

// ---- what a character owns -----------------------------------------------------------------------------------------------------

/// the added combinations a character owns (bit k: the combination `GAME_COMBOS + k`), by the character's address
static OWNED: Mutex<Option<HashMap<i32, u64>>> = Mutex::new(None);

/// Does the character `chr` own the combination (non zero)?  (`chr + 488 + 4 * combo` for the game's own.)
pub fn own_get(chr: i32, combo: i32) -> i32 {
    if combo < GAME_COMBOS {
        return lw(chr + 488 + 4 * combo);
    }
    let bit = (combo - GAME_COMBOS).clamp(0, 63);
    OWNED.lock().unwrap().as_ref().and_then(|m| m.get(&chr)).map_or(0, |bits| (bits >> bit & 1) as i32)
}

pub fn own_set(chr: i32, combo: i32) {
    if combo < GAME_COMBOS {
        sw(chr + 488 + 4 * combo, 1);
        return;
    }
    let bit = (combo - GAME_COMBOS).clamp(0, 63);
    if std::env::var_os("TS_COMBOLOG").is_some() {
        eprintln!("[combo] f{} chr {:#x} picks up {} (the pair of {:?})", ts_sdk::frame_count(), chr, combo, single_of(combo));
    }
    *OWNED.lock().unwrap().get_or_insert_with(HashMap::new).entry(chr).or_insert(0) |= 1 << bit;
}

/// A game starts: nobody owns an added pair.
pub fn own_clear_all() {
    *OWNED.lock().unwrap() = None;
}

/// The character has no guns (`chrClearGuns`).
pub fn own_clear(chr: i32) {
    if let Some(m) = OWNED.lock().unwrap().as_mut() {
        m.remove(&chr);
    }
}

/// The number that the statistics count a combination under: a pair counts with its single weapon.
pub fn stats_combo(combo: i32) -> i32 {
    single_of(combo).unwrap_or(combo)
}
