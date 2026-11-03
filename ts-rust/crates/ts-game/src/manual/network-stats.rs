//! The Network section of the statistics page.
//!
//! A profile keeps the statistics of its games in four records of 396 bytes (all, story, arcade, challenge).  The records of the Network
//! games are kept beside them: a record for each profile, found by its name, in the file `network-statistics.bin` of the saves folder, and in
//! memory of the port's own (`RECORDS`) while the program runs.  A network match is played with temporary profiles; when the match is over
//! (`netlink_keep_stats`) what this machine's player did is added to the record of the profile that was signed in.  The page's Game type item
//! gets a fifth value, Network, and the page shows the record when it is chosen.

use crate::*;
use std::sync::Mutex;

const BASE: i32 = 0x240_0000;
/// The records, one for each of the first `SLOTS` profiles that have played a network match.
const RECORDS: i32 = BASE + 0x17_0000;
/// The list of the values of Game type (a value and a text each; the last has no text) and the text of the new value.
const LIST: i32 = BASE + 0x17_2000;
const LABEL: i32 = BASE + 0x17_2100;
const SLOTS: usize = 16;
const RECORD: i32 = 396;
/// A profile's name is at the start of its record in the sign-on array, 29 characters at most.
const NAME: usize = 29;

/// The names of the profiles that the records belong to (the number of a record is its place here).
static NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn file() -> std::path::PathBuf {
    ts_sdk::save_root().join("network-statistics.bin")
}

fn read_name(at: i32) -> String {
    let mut s = String::new();
    for i in 0..NAME as i32 {
        let c = lbu(at + i) as u8;
        if c == 0 {
            break;
        }
        s.push(c as char);
    }
    s
}

/// At the start of the program: the records that were saved are put in memory.
pub fn network_stats_init() {
    let mut names = NAMES.lock().unwrap();
    names.clear();
    memset_game(RECORDS, 0, SLOTS * RECORD as usize);
    let Ok(bytes) = std::fs::read(file()) else { return };
    for chunk in bytes.chunks_exact(32 + RECORD as usize).take(SLOTS) {
        let end = chunk[..32].iter().position(|&b| b == 0).unwrap_or(32);
        names.push(String::from_utf8_lossy(&chunk[..end]).into_owned());
        let at = RECORDS + (names.len() as i32 - 1) * RECORD;
        host_slice(at, RECORD as usize).copy_from_slice(&chunk[32..]);
    }
}

/// A test aid: `TS_NETSTATS_FAKE=<name>` adds a made-up match to the record of that profile (to see the page and the file without a network match).
pub fn network_stats_fake_aid() {
    let Ok(name) = std::env::var("TS_NETSTATS_FAKE") else { return };
    let src = LABEL + 0x100;
    memset_game(src, 0, RECORD as usize);
    sw(src, 1);
    sw(src + 4, 3600);
    sw(src + 8, 7);
    sw(src + 12, 3);
    sw(src + 16, 4);
    sw(src + 28, 40);
    sw(src + 32, 12);
    sw(src + 64 + 4, 7);
    let mut names = NAMES.lock().unwrap();
    let dst = record_of(&name, &mut names);
    merge(dst, src);
    save(&names);
}

fn save(names: &[String]) {
    let mut out = Vec::new();
    for (i, n) in names.iter().enumerate() {
        let mut name = [0u8; 32];
        for (k, b) in n.bytes().take(NAME).enumerate() {
            name[k] = b;
        }
        out.extend_from_slice(&name);
        out.extend_from_slice(host_slice_ro(RECORDS + i as i32 * RECORD, RECORD as usize));
    }
    let _ = std::fs::write(file(), out);
}

/// The record of the profile with this name (made when there is none; the last place is shared when all are taken), as an address.
fn record_of(name: &str, names: &mut Vec<String>) -> i32 {
    let i = match names.iter().position(|n| n == name) {
        Some(i) => i,
        None if names.len() < SLOTS => {
            names.push(name.to_string());
            names.len() - 1
        }
        None => SLOTS - 1,
    };
    RECORDS + i as i32 * RECORD
}

/// The record that the page shows for the player `player` (the one of the profile that is signed in as that player; a record of zeros when it has played no network match).
pub fn network_stats_for_page(player: i32) -> i32 {
    let empty = RECORDS + (SLOTS as i32 - 1) * RECORD;
    let stats = statsGetPlayer(player);
    if stats == 0x1fbc168 {
        return empty;
    }
    let name = read_name(stats - 132);
    let names = NAMES.lock().unwrap();
    match names.iter().position(|n| *n == name) {
        Some(i) => RECORDS + i as i32 * RECORD,
        None => empty,
    }
}

/// Add the life record `src` (what one game made) to the life record `dst`: the counts and the times are added, the best of a kind is kept, the medals are joined.
fn merge(dst: i32, src: i32) {
    let int_add = |off: i32| sw(dst + off, lw(dst + off) + lw(src + off));
    let float_add = |off: i32| swc1(dst + off, lwc1(dst + off) + lwc1(src + off));
    for off in [0, 4, 8, 12, 28, 32, 52, 56, 60, 196, 336] {
        int_add(off);
    }
    for off in [16, 20, 24] {
        if lw(dst + off) < lw(src + off) {
            sw(dst + off, lw(src + off));
        }
    }
    for off in [36, 40, 44, 48, 332, 344] {
        float_add(off);
    }
    for gun in 0..33 {
        int_add(64 + 4 * gun);
        float_add(200 + 4 * gun);
    }
    for medal in 0..48 {
        sb(dst + 348 + medal, lbu(dst + 348 + medal) | lbu(src + 348 + medal));
    }
}

/// The Network record of the player's profile is added to `dst`, the record that the page makes of the other three for Overall.
pub fn network_stats_add_to_overall(dst: i32, player: i32) {
    merge(dst, network_stats_for_page(player));
}

/// A network match is over: what the temporary profile `mine` made (its record of arcade games, which holds only this match) is added to the Network record of the profile
/// `real` that was signed in.
pub fn network_stats_add_match(real: i32, mine: i32) {
    let array = lw(g::signonarray);
    let name = read_name(array + real * 2936);
    let src = array + mine * 2936 + 132 + 2012;
    if std::env::var_os("TS_NETLOG").is_some() {
        eprintln!("[net] network statistics of {name:?}: this match games {} time {} kills {} losses {}", lw(src), lw(src + 4), lw(src + 8), lw(src + 12));
    }
    if lw(src) == 0 && lw(src + 4) == 0 {
        return;
    }
    let mut names = NAMES.lock().unwrap();
    let dst = record_of(&name, &mut names);
    merge(dst, src);
    save(&names);
}

/// A test aid: `TS_OPENSTATS=<ticks>` opens the statistics page when the front end has ticked that many times.
pub fn network_stats_open_aid() {
    use std::sync::atomic::{AtomicU32, Ordering};
    static TICKS: AtomicU32 = AtomicU32::new(0);
    let Some(at) = std::env::var("TS_OPENSTATS").ok().and_then(|v| v.parse::<u32>().ok()) else { return };
    if TICKS.fetch_add(1, Ordering::Relaxed) + 1 == at {
        frontPageSet(0, g::statistics_page);
    }
}

/// At the tick of the statistics page: Game type has a fifth value, Network.
pub fn network_stats_page_tick() {
    let list = lw(g::gv_statstype + 16);
    if list == LIST {
        return;
    }
    // (the game's own list: four values and the end, in a place that has no room for another)
    memcpy_game(LIST, list, 32);
    for (i, b) in b"Network\0".iter().enumerate() {
        sb(LABEL + i as i32, *b as i32);
    }
    sw(LIST + 32, 4);
    sw(LIST + 36, LABEL);
    sw(LIST + 40, 0);
    sw(LIST + 44, 0);
    sw(g::gv_statstype + 16, LIST);
    sw(g::gv_statstype + 8, 4);
}
