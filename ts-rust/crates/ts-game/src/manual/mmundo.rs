//! Undo and redo in the MapMaker (extensions).  Every tick of the map maker the map is written as the game writes a map file (`mmSaveMapFile`,
//! leaving out the object in the hand); when it differs from the last, the last goes on the undo list and the redo list is emptied.  Undo puts the
//! map before the last change back (`mmLoadMapFile`) and the one it replaced on the redo list; redo does the reverse.  The buttons: L3 (undo) and
//! R3 (redo) of the first controller, which the map maker does not use, or the keys U and Y (`request_mm_undo` / `request_mm_redo`, from the
//! program's window).  A map put back leaves nothing in the hand.

use crate::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// The changes kept to undo.
const MOST: usize = 100;
/// The memory the map is written into (a map of the game is under 8 KB; one with no limit can be larger).
const BUFFER_BYTES: i32 = 0x40000;
/// L3 and R3 of the pad.
const L3: i32 = 0x0200;
const R3: i32 = 0x0400;
/// The sound of a step in the menus.
const STEP_SOUND: i32 = 140;

struct History {
    buffer: i32,
    last: Option<Vec<u8>>,
    undo: Vec<Vec<u8>>,
    redo: Vec<Vec<u8>>,
}

static HISTORY: Mutex<History> = Mutex::new(History { buffer: 0, last: None, undo: Vec::new(), redo: Vec::new() });
/// The frames the keys of undo and redo were pressed on (0: not since they were last used): a key pressed while the map maker does not tick (a dialog,
/// a name being typed) is forgotten, not used when it ticks again.
static UNDO_ASKED: AtomicU64 = AtomicU64::new(0);
static REDO_ASKED: AtomicU64 = AtomicU64::new(0);
/// How many frames a key press waits for the map maker's tick.
const KEY_FRAMES: u64 = 3;

/// The key of undo was pressed (the window's keys).
pub fn request_mm_undo() {
    UNDO_ASKED.store(ts_sdk::frame_count() as u64, Ordering::Relaxed);
}

/// The key of redo was pressed.
pub fn request_mm_redo() {
    REDO_ASKED.store(ts_sdk::frame_count() as u64, Ordering::Relaxed);
}

/// The map as a map file (empty for a map with no objects), without what is in the hand: the object in the hand (`usedobj`) and the ones after it
/// are at the end of the list of the map's objects (through +56), which is cut before it while the map is written, so moving the hand is no change.
fn map_bytes(buffer: i32) -> Vec<u8> {
    let used = lw(g::usedobj);
    if used == 0 {
        return written(buffer);
    }
    let head = lw(g::mapobjects);
    if head == used {
        return Vec::new();
    }
    let mut before = head;
    while before != 0 && lw(before + 56) != used {
        before = lw(before + 56);
    }
    if before == 0 {
        return written(buffer);
    }
    sw(before + 56, 0);
    let map = written(buffer);
    sw(before + 56, used);
    map
}

fn written(buffer: i32) -> Vec<u8> {
    let size = mmSaveMapFile(buffer, 1);
    if size <= 0 || size > BUFFER_BYTES {
        return Vec::new();
    }
    (0..size).map(|i| lbu(buffer + i) as u8).collect()
}

/// Put a map back.
fn put_back(buffer: i32, map: &[u8]) {
    if map.is_empty() {
        mmClearMap();
        return;
    }
    for (i, b) in map.iter().enumerate() {
        sb(buffer + i as i32, *b as i32);
    }
    mmLoadMapFile(buffer, 1);
}

/// Forget the history: the map maker starts with an empty map (`mapmakerReset`; a map it takes back from its backup, after a map was tried, keeps it).
pub fn mm_undo_reset() {
    let mut h = HISTORY.lock().unwrap();
    h.last = None;
    h.undo.clear();
    h.redo.clear();
}

/// The map maker's tick (`mapmakerTick`, while no message or dialog is up): note a change of the map, and undo or redo when asked.
pub fn mm_undo_tick() {
    let pressed = joyNewBut(0);
    let now = ts_sdk::frame_count() as u64;
    let fresh = |at: u64| at != 0 && now.saturating_sub(at) <= KEY_FRAMES;
    let undo = fresh(UNDO_ASKED.swap(0, Ordering::Relaxed)) || pressed & L3 != 0;
    let redo = fresh(REDO_ASKED.swap(0, Ordering::Relaxed)) || pressed & R3 != 0;
    let mut h = HISTORY.lock().unwrap();
    if h.buffer == 0 {
        h.buffer = malloc(BUFFER_BYTES);
        if h.buffer == 0 {
            return;
        }
    }
    let buffer = h.buffer;
    let now = map_bytes(buffer);
    match &h.last {
        None => h.last = Some(now),
        Some(last) if *last != now => {
            let last = h.last.replace(now).unwrap();
            h.undo.push(last);
            if h.undo.len() > MOST {
                h.undo.remove(0);
            }
            h.redo.clear();
            if std::env::var_os("TS_MMUNDOLOG").is_some() {
                eprintln!("[mmundo] f{} change: {} to undo, map {} bytes", lw(g::gameframe), h.undo.len(), h.last.as_ref().map_or(0, |m| m.len()));
            }
        }
        _ => {}
    }
    if undo {
        if let Some(before) = h.undo.pop() {
            let current = h.last.replace(before.clone()).unwrap_or_default();
            h.redo.push(current);
            put_back(buffer, &before);
            // (the map as the game writes it again, so that the next tick does not take a difference of writing for a change)
            h.last = Some(map_bytes(buffer));
            soundStart(STEP_SOUND, 0);
            if std::env::var_os("TS_MMUNDOLOG").is_some() {
                eprintln!("[mmundo] f{} undo: {} to undo, {} to redo, map {} bytes", lw(g::gameframe), h.undo.len(), h.redo.len(), h.last.as_ref().map_or(0, |m| m.len()));
            }
        }
    } else if redo {
        if let Some(after) = h.redo.pop() {
            let current = h.last.replace(after.clone()).unwrap_or_default();
            h.undo.push(current);
            put_back(buffer, &after);
            h.last = Some(map_bytes(buffer));
            soundStart(STEP_SOUND, 0);
            if std::env::var_os("TS_MMUNDOLOG").is_some() {
                eprintln!("[mmundo] f{} redo: {} to undo, {} to redo, map {} bytes", lw(g::gameframe), h.undo.len(), h.redo.len(), h.last.as_ref().map_or(0, |m| m.len()));
            }
        }
    }
}
