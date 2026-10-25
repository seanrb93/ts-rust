//! In-game differential testing: for the functions named in `TS_SHADOW` (comma separated), run the ORIGINAL machine
//! code on the live game state at every outermost call, roll the memory back, let the decompiled Rust run, and
//! compare every byte it changed.  Needs the port to be built with `--features shadow`.
use crate::cpu::*;
use crate::interp::*;
use ts_rt::*;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static CTX: OnceLock<Ctx> = OnceLock::new();
static FILTER: OnceLock<HashSet<String>> = OnceLock::new();
static CALLS: Mutex<Option<HashMap<&'static str, u32>>> = Mutex::new(None);
static STATS: Mutex<Option<HashMap<&'static str, (u32, u32, u32)>>> = Mutex::new(None); // ok, mismatch, skipped

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
}

/// Register the hook (call once, after the executable has been loaded into the arena).
pub fn install(elf: &Path) {
    let Ok(list) = std::env::var("TS_SHADOW") else { return };
    let syms = loader::read_symbols(elf).expect("cannot read symbols");
    let gp = syms.iter().find(|s| s.name == "_gp").unwrap().addr as u64;
    let split = syms.iter().find(|s| s.name == "text_data_split").unwrap().addr;
    let externs: HashMap<u32, String> = syms.iter().filter(|s| s.is_func && s.addr >= split).map(|s| (s.addr, s.name.clone())).collect();
    let mut externs = externs;
    // library routines listed here run as the original machine code instead of the port's native replacement
    if let Ok(real) = std::env::var("TS_SHADOW_REAL") {
        for name in real.split(',') {
            externs.retain(|_, n| n != name.trim());
        }
    }
    let _ = CTX.set(Ctx { externs, gp });
    let _ = FILTER.set(list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
    shadow::set_hook(hook);
    eprintln!("[shadow] checking {:?}", FILTER.get().unwrap());
}

fn bump(name: &'static str, k: usize) {
    let mut g = STATS.lock().unwrap();
    let e = g.get_or_insert_with(HashMap::new).entry(name).or_insert((0, 0, 0));
    match k {
        0 => e.0 += 1,
        1 => e.1 += 1,
        _ => e.2 += 1,
    }
}

pub fn summary() -> String {
    let g = STATS.lock().unwrap();
    let mut v: Vec<_> = g.iter().flat_map(|m| m.iter()).map(|(n, s)| format!("{n}: ok {} MISMATCH {} skipped {}", s.0, s.1, s.2)).collect();
    v.sort();
    v.join("\n")
}

fn hook(addr: u32, name: &'static str, ints: [i64; 8], floats: [f32; 8]) -> Option<Box<dyn FnOnce()>> {
    let filter = FILTER.get()?;
    if ACTIVE.with(|a| a.get()) {
        return None;
    }
    if filter.contains("*") {
        // every function, but only some of its calls: the first `TS_SHADOW_FIRST` (3) and then one in `TS_SHADOW_EVERY` (1000), so that a whole run can be checked
        let first: u32 = std::env::var("TS_SHADOW_FIRST").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
        let every: u32 = std::env::var("TS_SHADOW_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(1000).max(1);
        let mut g = CALLS.lock().unwrap();
        let n = g.get_or_insert_with(HashMap::new).entry(name).or_insert(0);
        let this = *n;
        *n += 1;
        if this >= first && this % every != 0 {
            return None;
        }
    } else if !filter.contains(name) {
        return None;
    }
    ACTIVE.with(|a| a.set(true));
    let ctx = CTX.get()?;
    // run the original code on the stack area the native frames are about to use, scrubbed so that reads of
    // never-written locals see the same (zero) data on both sides
    let sp = frame::current_sp();
    for a in (sp - 0x4000..sp).step_by(4) {
        sw(a as i32, 0);
    }
    INTERP_SP.store(sp, std::sync::atomic::Ordering::Relaxed);
    let a = run_interp(ctx, addr, &ints, &floats);
    let (w1, s1) = match a {
        Outcome::Ok { writes, stores, .. } => (writes, stores),
        Outcome::Skip(why) => {
            static ONCE: Mutex<Option<HashSet<String>>> = Mutex::new(None);
            if ONCE.lock().unwrap().get_or_insert_with(HashSet::new).insert(format!("{name}{why}")) {
                eprintln!("[shadow] {name}: not checkable ({why})");
            }
            bump(name, 2);
            ACTIVE.with(|a| a.set(false));
            return None;
        }
        other => {
            eprintln!("[shadow] {name}: original code faulted ({other:?}) - skipped");
            bump(name, 2);
            ACTIVE.with(|a| a.set(false));
            return None;
        }
    };
    trace::start();
    Some(Box::new(move || {
        let log = trace::stop();
        let full = log.clone();
        let w2 = changed(&log);
        let log: Vec<trace::Store> = log.into_iter().filter(|s| !s.bulk && s.addr < STACK_TOP - 0x10_0000).collect();
        ACTIVE.with(|a| a.set(false));
        if w1 == w2 || stores_equal_mod_nan(&s1, &log) {
            bump(name, 0);
            return;
        }
        bump(name, 1);
        // `TS_SHADOW_DUMP=name,name` (or `*`): write the memory from before this call, with its arguments (what the port did is undone), and end the run: replay it with `TS_VERIFY_CAPTURE`
        if std::env::var("TS_SHADOW_DUMP").map_or(false, |v| v.split(',').any(|n| n == name || n == "*")) {
            trace::rollback(&full);
            let dir = std::env::var("TS_CAPTURE_DIR").or_else(|_| std::env::var("TEMP")).unwrap_or_else(|_| ".".into());
            ts_rt::capture::write(name, sp, &ints, &floats, &format!("{dir}/{name}-mismatch.cap"));
            std::process::exit(0);
        }
        let m1: HashMap<_, _> = w1.iter().cloned().collect();
        let m2: HashMap<_, _> = w2.iter().cloned().collect();
        let mut keys: Vec<_> = m1.keys().chain(m2.keys()).cloned().collect();
        keys.sort_unstable();
        keys.dedup();
        let mut d = String::new();
        for k in keys.into_iter().filter(|k| m1.get(k) != m2.get(k)).take(if std::env::var_os("TS_SHADOW_ALL").is_some() { 400 } else { 6 }) {
            d += &format!(" [{k:#x}: original {:?} port {:?}]", m1.get(&k), m2.get(&k));
        }
        let cover = |ss: &[trace::Store], a: u32| ss.iter().rev().find(|x| a >= x.addr && a < x.addr + x.size as u32).copied();
        let mut first = String::new();
        let mut dk: Vec<u32> = m1.keys().chain(m2.keys()).cloned().collect();
        dk.sort_unstable();
        dk.dedup();
        if let Some(&k) = dk.iter().find(|k| m1.get(k) != m2.get(k)) {
            let f = |s: Option<trace::Store>| match s {
                Some(s) => format!("[{:#x} +{} = {:02x?} pc {:#x}]", s.addr, s.size, &s.new[..s.size as usize], s.tag),
                None => "[no store]".to_string(),
            };
            first = format!(" | byte {k:#x}: original {} / port {}", f(cover(&s1, k)), f(cover(&log, k)));
        }
        d += &first;
        eprintln!("[shadow] MISMATCH {name} @ {addr:#x}: {} vs {} bytes changed{d}  args {ints:x?} {floats:?}", w1.len(), w2.len());
    }))
}
