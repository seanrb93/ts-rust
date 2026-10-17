//! Optional call tracing (feature `calltrace`): every decompiled function pushes its name on a per-thread stack.
//! A watchdog can ask the game thread to print that stack, which shows where a hang is.
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    static STACK: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

static DUMP: AtomicBool = AtomicBool::new(false);

pub struct Guard {
    timed: bool,
}

thread_local! {
    // (entry time, time spent in callees) of the functions being timed while profiling
    static TSTACK: RefCell<Vec<(std::time::Instant, u64)>> = const { RefCell::new(Vec::new()) };
    // function -> (calls, time including callees, time in the function itself), nanoseconds
    static TIMES: RefCell<std::collections::HashMap<&'static str, (u64, u64, u64)>> = RefCell::new(std::collections::HashMap::new());
}

impl Drop for Guard {
    #[inline]
    fn drop(&mut self) {
        STACK.with(|s| {
            let name = s.borrow_mut().pop();
            if self.timed {
                if let (Some(name), Some((t0, child))) = (name, TSTACK.with(|t| t.borrow_mut().pop())) {
                    let total = t0.elapsed().as_nanos() as u64;
                    TSTACK.with(|t| {
                        if let Some(parent) = t.borrow_mut().last_mut() {
                            parent.1 += total;
                        }
                    });
                    TIMES.with(|c| {
                        let mut c = c.borrow_mut();
                        let e = c.entry(name).or_insert((0, 0, 0));
                        e.0 += 1;
                        e.1 += total;
                        e.2 += total.saturating_sub(child);
                    });
                }
            }
        });
    }
}

static PROFILE: AtomicBool = AtomicBool::new(false);
thread_local! {
    static COUNTS: RefCell<std::collections::HashMap<&'static str, u64>> = RefCell::new(std::collections::HashMap::new());
}

/// Start / stop counting function entries (diagnostics).
pub fn profile(on: bool) {
    PROFILE.store(on, Ordering::Relaxed);
}

/// Print the most frequently entered functions since the last dump and reset the counters.
pub fn profile_dump(top: usize) {
    COUNTS.with(|c| {
        let mut v: Vec<_> = c.borrow().iter().map(|(k, v)| (*k, *v)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        eprintln!("[profile] {} distinct functions", v.len());
        for (n, c) in v.iter().take(top) {
            eprintln!("    {c:8} {n}");
        }
        c.borrow_mut().clear();
    });
    TIMES.with(|c| {
        let mut v: Vec<_> = c.borrow().iter().map(|(k, v)| (*k, *v)).collect();
        v.sort_by(|a, b| b.1 .2.cmp(&a.1 .2));
        eprintln!("[profile] time: function, calls, self ms, total ms");
        for (n, (calls, total, own)) in v.iter().take(top) {
            eprintln!("    {n:36} {calls:8} {:9.3} {:9.3}", *own as f64 / 1e6, *total as f64 / 1e6);
        }
        c.borrow_mut().clear();
    });
}

#[inline]
pub fn enter(name: &'static str) -> Guard {
    STACK.with(|s| s.borrow_mut().push(name));
    let timed = PROFILE.load(Ordering::Relaxed);
    if timed {
        COUNTS.with(|c| *c.borrow_mut().entry(name).or_insert(0) += 1);
        TSTACK.with(|t| t.borrow_mut().push((std::time::Instant::now(), 0)));
    }
    if DUMP.load(Ordering::Relaxed) {
        dump_now();
    }
    if name == "newrnd2" && RNG_LOG.load(Ordering::Relaxed) {
        STACK.with(|s| {
            let s = s.borrow();
            let tail: Vec<&str> = s.iter().rev().skip(1).take(5).cloned().collect();
            eprintln!("[rnd] f={} seed={:016x} {}", FRAME.load(Ordering::Relaxed), crate::ld(0x3afc70) as u64, tail.join(" <- "));
        });
    }
    Guard { timed }
}

static RNG_LOG: AtomicBool = AtomicBool::new(false);
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Diagnostics: log the call chain of every random number drawn (with the frame number set by `set_frame`).
pub fn rng_log(on: bool) {
    RNG_LOG.store(on, Ordering::Relaxed);
}

pub fn set_frame(n: u32) {
    FRAME.store(n, Ordering::Relaxed);
}

/// Ask the game thread to print its call stack at the next function entry / stub call.
pub fn request_dump() {
    DUMP.store(true, Ordering::Relaxed);
}

pub fn dump_now() {
    DUMP.store(false, Ordering::Relaxed);
    STACK.with(|s| {
        let s = s.borrow();
        eprintln!("[calltrace] call stack ({} deep), innermost last:", s.len());
        for n in s.iter() {
            eprintln!("    {n}");
        }
    });
}

/// Check for a pending dump request (used by stubs that may be called in a spin loop).
#[inline]
pub fn poll() {
    if DUMP.load(Ordering::Relaxed) {
        dump_now();
    }
}

fn filter() -> &'static Vec<String> {
    use std::sync::OnceLock;
    static F: OnceLock<Vec<String>> = OnceLock::new();
    F.get_or_init(|| {
        std::env::var("TS_TRACE").map(|v| v.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default()
    })
}

/// Print a function's arguments on entry when its name is listed in the `TS_TRACE` environment variable
/// (comma separated; `*` traces everything).
#[inline]
pub fn trace_args(name: &str, args: &[i64]) {
    let f = filter();
    if f.is_empty() {
        return;
    }
    if f.iter().any(|n| n == "*" || n == name) {
        let a: Vec<String> = args.iter().map(|v| format!("{v:#x}")).collect();
        eprintln!("[trace] {name}({})", a.join(", "));
    }
}


// ---- float store profiler: how much each function changes the floats it stores to ---------------------------------------
//
// A function that moves things by a fixed amount on every call changes the same floats twice as much in a second when it is
// called twice as often; one that scales by the frame time does not.  Comparing the totals of runs at different frame
// rates points at logic that is tied to the frame rate (`TS_FPROF=<first tick>,<last tick>`).

static FPROF: AtomicBool = AtomicBool::new(false);
thread_local! {
    // function -> (stores, stores that changed the value, sum of |change|)
    static FSTATS: RefCell<std::collections::HashMap<&'static str, (u64, u64, f64)>> = RefCell::new(std::collections::HashMap::new());
}

pub fn fprof(on: bool) {
    FPROF.store(on, Ordering::Relaxed);
}

#[inline]
pub fn fstore(a: i32, new: f32) {
    if !FPROF.load(Ordering::Relaxed) {
        return;
    }
    let old = crate::lwc1(a);
    STACK.with(|s| {
        if let Some(&name) = s.borrow().last() {
            FSTATS.with(|f| {
                let mut f = f.borrow_mut();
                let e = f.entry(name).or_insert((0, 0, 0.0));
                e.0 += 1;
                if old.is_finite() && new.is_finite() && old != new && old.abs() < 1e6 && new.abs() < 1e6 {
                    e.1 += 1;
                    e.2 += (new - old).abs() as f64;
                }
            });
        }
    });
}

pub fn fprof_dump() {
    FSTATS.with(|f| {
        let mut v: Vec<_> = f.borrow().iter().map(|(k, v)| (*k, *v)).collect();
        v.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
        eprintln!("[fprof] {} functions", v.len());
        for (n, (stores, changed, sum)) in v {
            eprintln!("    {n} {stores} {changed} {sum:.4}");
        }
        f.borrow_mut().clear();
    });
}
