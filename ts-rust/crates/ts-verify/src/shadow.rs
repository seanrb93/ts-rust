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
    /// The platform calls the port made during the check in progress (name and arguments), when `TS_SHADOW_EXTERN` is set.
    static PORT_EXTERNS: std::cell::RefCell<Vec<ExternCall>> = const { std::cell::RefCell::new(Vec::new()) };
}
static EXTERN_NAMES: OnceLock<HashSet<String>> = OnceLock::new();
static CHECKS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// `TS_SHADOW_EXTERN=1`: while a function is checked, the routines of the console that the interpreter cannot run (sound and pad servers, memory card, semaphores, disc) answer the same
/// made-up values on both sides and are not run; the calls the original and the port make must be the same.  The game misses those calls on the checked calls only.
fn extern_hook(name: &'static str, args: &[i64]) -> Option<i64> {
    if !ACTIVE.with(|a| a.get()) || extern_is_native(name) || !EXTERN_NAMES.get().map_or(false, |names| names.contains(name)) {
        return None;
    }
    let index = PORT_EXTERNS.with(|l| l.borrow().len());
    PORT_EXTERNS.with(|l| l.borrow_mut().push((name.to_string(), args.iter().take(8).map(|&v| v as i32).collect())));
    let variant = STUB_VARIANT.with(|v| v.get());
    let value = stub_value(name, index, variant);
    stub_effect(name, &args.iter().map(|&v| v as i32).collect::<Vec<_>>(), value, index, variant);
    Some(value)
}

/// The two logs equal; the port passes only the arguments the call has, so only those are compared, and two addresses on the stack count as equal (the port's locals are not where the original kept them).
fn externs_equal(original: &[ExternCall], port: &[ExternCall]) -> bool {
    // the console's debug print (the original prints a warning when the transfer of an image to the graphics chip takes long; the port's replacement of that transfer does not) is only text
    let (original, port): (Vec<&ExternCall>, Vec<&ExternCall>) = (original.iter().filter(|c| c.0 != "scePrintf").collect(), port.iter().filter(|c| c.0 != "scePrintf").collect());
    original.len() == port.len() && original.iter().zip(&port).all(|(o, p)| o.0 == p.0 && o.1.iter().zip(&p.1).all(|(x, y)| x == y || (*x as u32 >= STACK_LIMIT && *y as u32 >= STACK_LIMIT)))
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
    let _ = EXTERN_NAMES.set(externs.values().cloned().collect());
    let _ = CTX.set(Ctx { externs, gp });
    let _ = FILTER.set(list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
    shadow::set_hook(hook);
    if std::env::var_os("TS_SHADOW_EXTERN").is_some() {
        shadow::set_extern_hook(extern_hook);
    }
    eprintln!("[shadow] checking {:?}", FILTER.get().unwrap());
}

/// `TS_VERIFY_EXTERN=1` in the verifier on random memory: the same stubbing of the platform calls as the live check, run by run (`variant` is the number of the run).  Call before the original code runs ...
pub fn stub_begin(ctx: &Ctx, variant: usize) {
    let _ = EXTERN_NAMES.set(ctx.externs.values().cloned().collect());
    shadow::set_extern_hook(extern_hook);
    ACTIVE.with(|a| a.set(false));
    STUB_EXTERNS.with(|s| s.set(true));
    STUB_VARIANT.with(|v| v.set(variant));
    take_extern_log();
}

/// ... after it: returns what the original called, and lets the port's calls be answered and logged.
pub fn stub_interp_done() -> Vec<ExternCall> {
    STUB_EXTERNS.with(|s| s.set(false));
    PORT_EXTERNS.with(|l| l.borrow_mut().clear());
    ACTIVE.with(|a| a.set(true));
    take_extern_log()
}

/// ... and after the port ran: returns what the port called.
pub fn stub_end() -> Vec<ExternCall> {
    ACTIVE.with(|a| a.set(false));
    PORT_EXTERNS.with(|l| std::mem::take(&mut *l.borrow_mut()))
}

pub fn externs_match(original: &[ExternCall], port: &[ExternCall]) -> bool {
    externs_equal(original, port)
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
    // the copy of the depth buffer is replaced on purpose (`manual/gs.rs`), and so is the setting up of the console in `bossMakeAll`
    if matches!(name, "zbtestCopyZB" | "bossMakeAll") {
        return None;
    }
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
    let stub = std::env::var_os("TS_SHADOW_EXTERN").is_some();
    if stub {
        STUB_EXTERNS.with(|s| s.set(true));
        DENY_DISC.with(|s| s.set(true));
        STUB_VARIANT.with(|v| v.set(CHECKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        take_extern_log();
    }
    let a = run_interp(ctx, addr, &ints, &floats);
    STUB_EXTERNS.with(|s| s.set(false));
    let log1 = take_extern_log();
    PORT_EXTERNS.with(|l| l.borrow_mut().clear());
    let (w1, s1) = match a {
        Outcome::Ok { writes, stores, .. } => (writes, stores),
        Outcome::Skip(why) => {
            static ONCE: Mutex<Option<HashSet<String>>> = Mutex::new(None);
            if ONCE.lock().unwrap().get_or_insert_with(HashSet::new).insert(format!("{name}{why}")) {
                eprintln!("[shadow] {name}: not checkable ({why}){}", if log1.is_empty() { String::new() } else { format!("; last platform calls {:x?}", &log1[log1.len().saturating_sub(4)..]) });
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
        let port_calls = PORT_EXTERNS.with(|l| std::mem::take(&mut *l.borrow_mut()));
        let full = log.clone();
        // words the original takes from stack garbage (the last hit that `bulletInitialiseStraightLineTest` puts back, the wall normal of `moveStep`) and the first 64 KB are left out,
        // and so is the clock that `timeTickEnd` reads: they differ for reasons that are not in the code
        let ignored = |a: u32| a < 0x10000 || (0x3aea90..0x3aeab8).contains(&a) || (0x3b0118..0x3b0124).contains(&a) || (name == "timeTickEnd" && (0x3afc68..0x3afc70).contains(&a));
        let w1: Vec<(u32, u8)> = w1.into_iter().filter(|(a, _)| !ignored(*a)).collect();
        let w2 = changed(&log);
        let w2: Vec<(u32, u8)> = w2.into_iter().filter(|(a, _)| !ignored(*a)).collect();
        let log: Vec<trace::Store> = log.into_iter().filter(|s| !s.bulk && s.addr < STACK_TOP - 0x10_0000).collect();
        ACTIVE.with(|a| a.set(false));
        if !externs_equal(&log1, &port_calls) {
            bump(name, 1);
            eprintln!("[shadow] MISMATCH {name} @ {addr:#x}: platform calls differ  original {log1:x?}  port {port_calls:x?}  args {ints:x?} {floats:?}");
            return;
        }
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
