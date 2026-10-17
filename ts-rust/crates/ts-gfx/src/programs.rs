//! VU1 microprogram dispatch: the microcode image has been decompiled to native Rust (`vu1_gen.rs`).
//! `XGKICK` hands a GIF packet to the vertex stream interpreter.

use crate::gif::Gif;
use crate::scene::Frame;
use crate::vif::{Vu1, VU_MEM_QW};
use crate::vu::Regs;

/// Timing counters (`UNPACK_NS`, `VU_NS`, ...) cost a clock read per event: they only run when asked for (TS_BENCH, ...).
pub static TIMING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline(always)]
pub fn tstart() -> Option<std::time::Instant> {
    if TIMING.load(std::sync::atomic::Ordering::Relaxed) {
        Some(std::time::Instant::now())
    } else {
        None
    }
}

#[inline(always)]
pub fn tadd(counter: &std::sync::atomic::AtomicU64, t: Option<std::time::Instant>) {
    if let Some(t) = t {
        counter.fetch_add(t.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    }
}

/// `XGKICK`: send the GIF packet that starts at quadword `qw` of VU memory.
pub fn xgkick(vu: &Vu1, gif: &mut Gif, frame: &mut Frame, qw: usize) {
    let qw = qw & (VU_MEM_QW - 1);
    note_kick(qw);
    tlog!("    XGKICK {qw:#x}");
    gif.kick = gif.kick.wrapping_add(1);
    let t = tstart();
    #[cfg(feature = "vucheck")]
    capture(vu, qw);
    gif.process(frame, vu.bytes_from(qw));
    tadd(&GIF_NS, t);
}

fn dump_state(path: &str, vu: &Vu1, r: &Regs, entry: u32, kicks: &[usize]) {
    let mut b: Vec<u8> = Vec::new();
    for q in 0..VU_MEM_QW {
        for w in vu.mem[q] {
            b.extend_from_slice(&w.to_le_bytes());
        }
    }
    for v in r.vf {
        for l in v {
            b.extend_from_slice(&l.to_bits().to_le_bytes());
        }
    }
    for v in r.vi {
        b.extend_from_slice(&(v as u32).to_le_bytes());
    }
    for l in r.acc {
        b.extend_from_slice(&l.to_bits().to_le_bytes());
    }
    for x in [r.q.to_bits(), r.q_pend.to_bits(), r.p.to_bits(), r.i.to_bits(), r.mac as u32, r.clip, vu.top, vu.itop, entry] {
        b.extend_from_slice(&x.to_le_bytes());
    }
    b.extend_from_slice(&r.q_ready.to_le_bytes());
    b.extend_from_slice(&r.cycle.to_le_bytes());
    b.extend_from_slice(&(kicks.len() as u32).to_le_bytes());
    for k in kicks {
        b.extend_from_slice(&(*k as u32).to_le_bytes());
    }
    std::fs::write(path, b).ok();
}

fn dump_dir() -> Option<String> {
    static DUMP_DIR: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    DUMP_DIR.get_or_init(|| std::env::var("TS_VUDUMP").ok()).clone()
}

thread_local! {
    static KICKS: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
    static COUNTER: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

pub fn run(entry: u32, vu: &mut Vu1, regs: &mut Regs, gif: &mut Gif, frame: &mut Frame) {
    // Debug aid: TS_VUDUMP=<dir> writes the machine state before/after every program run so an independent reference
    // interpreter can be compared against the generated code.
    let dump = if crate::trace::enabled() { dump_dir() } else { None };
    let n = if dump.is_some() {
        COUNTER.with(|c| {
            let v = c.get();
            c.set(v + 1);
            v
        })
    } else {
        0
    };
    if let Some(dir) = &dump {
        KICKS.with(|k| k.borrow_mut().clear());
        dump_state(&format!("{dir}/before_{n:04}.bin"), vu, regs, entry, &[]);
    }
    gif.prog = entry;
    #[cfg(feature = "vucheck")]
    let reference = {
        // the original translation runs on a copy of the state first
        let (mut vu2, mut r2) = (vu.clone(), crate::vu_ref::Regs::from_new(regs));
        CAP.with(|c| c.borrow_mut().clear());
        crate::vu1_gen_ref::run(&mut vu2, &mut r2, gif, frame, entry as usize);
        (vu2, r2, CAP.with(|c| std::mem::take(&mut *c.borrow_mut())))
    };
    let t0 = tstart();
    crate::vu1_gen::run(vu, regs, gif, frame, entry as usize);
    #[cfg(feature = "vucheck")]
    check_against(&reference, vu, regs, entry);
    if let Some(t0) = t0 {
        let ns = t0.elapsed().as_nanos() as u64;
        VU_NS.fetch_add(ns, std::sync::atomic::Ordering::Relaxed);
        VU_RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if PROF_ON.load(std::sync::atomic::Ordering::Relaxed) {
            let i = (entry as usize).min(PROF.len() - 1);
            PROF[i].0.fetch_add(ns, std::sync::atomic::Ordering::Relaxed);
            PROF[i].1.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    if let Some(dir) = &dump {
        let kicks = KICKS.with(|k| k.borrow().clone());
        dump_state(&format!("{dir}/after_{n:04}.bin"), vu, regs, entry, &kicks);
    }
}

#[cfg(feature = "vucheck")]
thread_local! {
    pub static CAP: std::cell::RefCell<Vec<(usize, Vec<u32>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// `XGKICK` of the reference translation: only records the packet.
#[cfg(feature = "vucheck")]
pub fn xgkick_ref(vu: &Vu1, _gif: &mut Gif, _frame: &mut Frame, qw: usize) {
    capture(vu, qw);
}

#[cfg(feature = "vucheck")]
fn capture(vu: &Vu1, qw: usize) {
    let qw = qw & (VU_MEM_QW - 1);
    let words: Vec<u32> = (0..256).flat_map(|i| vu.mem[(qw + i) % VU_MEM_QW]).collect();
    CAP.with(|c| c.borrow_mut().push((qw, words)));
}

#[cfg(feature = "vucheck")]
static CHECKED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(feature = "vucheck")]
fn check_against(reference: &(Vu1, crate::vu_ref::Regs, Vec<(usize, Vec<u32>)>), vu: &Vu1, regs: &Regs, entry: u32) {
    let (vu2, r2, cap2) = reference;
    let cap1 = CAP.with(|c| std::mem::take(&mut *c.borrow_mut()));
    let n = CHECKED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut bad = Vec::new();
    if vu.mem != vu2.mem {
        let i = (0..VU_MEM_QW).find(|&i| vu.mem[i] != vu2.mem[i]).unwrap();
        bad.push(format!("memory differs at qw {i}: {:x?} vs ref {:x?}", vu.mem[i], vu2.mem[i]));
    }
    if cap1 != *cap2 {
        bad.push(format!("kicked packets differ ({} vs {})", cap1.len(), cap2.len()));
    }
    for i in 0..32 {
        let (a, b): ([u32; 4], [u32; 4]) = (regs.vf[i].map(f32::to_bits), r2.vf[i].map(f32::to_bits));
        if a != b {
            bad.push(format!("vf{i}: {a:x?} vs ref {b:x?}"));
        }
    }
    if regs.vi != r2.vi || regs.mac != r2.mac || regs.clip != r2.clip || regs.cycle != r2.cycle || regs.acc.map(f32::to_bits) != r2.acc.map(f32::to_bits) {
        bad.push(format!("vi/mac/clip/cycle/acc differ: {:?} {} {:#x} {} vs ref {:?} {} {:#x} {}", regs.vi, regs.mac, regs.clip, regs.cycle, r2.vi, r2.mac, r2.clip, r2.cycle));
    }
    if regs.q_now().to_bits() != r2.q.to_bits() && regs.cycle >= r2.q_ready {
        bad.push(format!("q {} vs ref {}", regs.q_now(), r2.q));
    }
    if n % 100_000 == 0 {
        eprintln!("[vucheck] {n} runs checked");
    }
    if !bad.is_empty() {
        panic!("[vucheck] run {n} (entry {entry:#x}) differs from the reference translation:
  {}", bad.join("
  "));
    }
}

pub fn note_kick(qw: usize) {
    if !crate::trace::enabled() {
        return;
    }
    KICKS.with(|k| k.borrow_mut().push(qw));
}

pub static UNPACK_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static STREAM_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static TEX_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static GIF_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROF_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub static PROF: [(std::sync::atomic::AtomicU64, std::sync::atomic::AtomicU64); 1024] = [const { (std::sync::atomic::AtomicU64::new(0), std::sync::atomic::AtomicU64::new(0)) }; 1024];

pub static VU_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static VU_RUNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
