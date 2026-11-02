//! Debug aid (features `watch` / `calltrace`): report every store to one game address together with the Rust call
//! stack, to find which decompiled function writes a value.
use std::sync::atomic::{AtomicU32, Ordering};

static WATCH: AtomicU32 = AtomicU32::new(0);

pub fn set(addr: u32) {
    WATCH.store(addr, Ordering::Relaxed);
}

static CUR_PLAYER: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);
static WRITES: std::sync::Mutex<Option<std::collections::BTreeMap<i32, std::collections::BTreeSet<u32>>>> = std::sync::Mutex::new(None);

/// Which player's code is running (a test aid: the static data each player's code writes is collected; -1 stops collecting).
pub fn set_player(p: i32) {
    CUR_PLAYER.store(p, Ordering::Relaxed);
}

/// Print the words of the game's static data that each player's code wrote.
pub fn dump_players() {
    let w = WRITES.lock().unwrap();
    if let Some(m) = w.as_ref() {
        for (p, set) in m {
            let v: Vec<String> = set.iter().map(|a| format!("{a:x}")).collect();
            eprintln!("[writes] player {p} {}: {}", set.len(), v.join(" "));
        }
    }
}

#[inline]
pub fn check(a: i32, size: usize, v: u64) {
    let cur = CUR_PLAYER.load(Ordering::Relaxed);
    if cur >= 0 && (0x2f0000..0x3c0000).contains(&(a as u32)) {
        WRITES.lock().unwrap().get_or_insert_with(Default::default).entry(cur).or_default().insert(a as u32 & !3);
    }
    let w = WATCH.load(Ordering::Relaxed);
    if w != 0 && (a as u32) <= w && w < a as u32 + size as u32 {
        report(a as u32, size, v);
    }
}

#[cold]
fn report(a: u32, size: usize, v: u64) {
    eprintln!("[watch] store {size} bytes at {a:#x} = {v:#x}");
    let bt = std::backtrace::Backtrace::force_capture().to_string();
    for l in bt.lines().filter(|l| l.contains("ts_game::dec") || l.contains("ts_game::game") || l.contains("ts_game::manual") || l.contains("ts_sdk")) {
        eprintln!("   {}", l.trim());
    }
}
