//! Differential tester for the decompiler.
//!
//! For each decompiled function: run the ORIGINAL machine code (tiny interpreter, dev-only) and the DECOMPILED Rust on
//! identical randomised inputs and memory, then compare return values and every memory write.
//!
//! usage: ts-verify [--n N] [--elf PATH] [--list] [name ...]


use ts_verify::cpu::*;
use ts_verify::interp::*;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use ts_game::{call_any, FUNC_INFO};
use ts_rt::loader::{load_elf_data, read_symbols};
use ts_rt::*;


struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn float(&mut self) -> f32 {
        match self.below(8) {
            0 => 0.0,
            1 => 1.0,
            2 => -1.0,
            3 => (self.below(2000) as f32 - 1000.0) / 4.0,
            _ => (self.below(200_000) as f32 - 100_000.0) / 10_000.0,
        }
    }
}


fn run_native(addr: u32, ints: &[i64; 8], floats: &[f32; 8]) -> Outcome {
    trace::start();
    let res = silent(|| call_any(addr, *ints, *floats));
    let log = trace::stop();
    let writes = changed(&log);
    trace::rollback(&log);
    let log: Vec<trace::Store> = log.into_iter().filter(|s| !s.bulk && s.addr < STACK_LIMIT).collect();
    match res {
        Err(p) if p.contains("not yet implemented") => Outcome::Skip(format!("stub: {p}")),
        Err(p) if p.starts_with("trap:") => Outcome::Trap,
        Err(p) => Outcome::Panic(p),
        Ok((ret, fret)) => Outcome::Ok { ret, fret, writes, stores: log },
    }
}

fn gen_inputs(rng: &mut Rng, ni: u8, nf: u8) -> ([i64; 8], [f32; 8]) {
    let mut ints = [0i64; 8];
    let mut floats = [0f32; 8];
    for i in 0..ni as usize {
        ints[i] = match rng.below(6) {
            0 => 0,
            1 => rng.below(16) as i64,
            2 => rng.below(8) as i64 - 4,
            3 | 4 => (SCRATCH + 16 * rng.below(2048) as u32) as i64,
            // TS_VERIFY_SMALL=1: no huge counts and offsets, so that loops over the data finish
            _ if std::env::var_os("TS_VERIFY_SMALL").is_some() => rng.below(32) as i64,
            _ => (rng.next() as i32 as i64) >> 12,
        };
    }
    for i in 0..nf as usize {
        floats[i] = rng.float();
    }
    (ints, floats)
}

fn fill_scratch(rng: &mut Rng) {
    for a in (0..SCRATCH_LEN).step_by(4) {
        let v: u32 = match rng.below(4) {
            0 => rng.float().to_bits(),
            1 => rng.below(16) as u32,
            2 => rng.float().to_bits(),
            _ => (rng.next() as u32) & 0x00ff_ffff,
        };
        sw((SCRATCH + a) as i32, v as i32);
    }
}


fn same_f(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()) || (a - b).abs() <= 1e-5 * a.abs().max(b.abs())
}

fn find_root() -> PathBuf {
    let mut d = std::env::current_dir().unwrap();
    loop {
        if d.join("Game Files").is_dir() {
            return d;
        }
        if !d.pop() {
            panic!("could not find the 'Game Files' directory above the working directory");
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut n = 12usize;
    let mut elf: Option<PathBuf> = None;
    let mut names = Vec::new();
    let mut verbose = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--n" => n = it.next().unwrap().parse().unwrap(),
            "--elf" => elf = Some(it.next().unwrap().into()),
            "-v" => verbose = true,
            _ => names.push(a),
        }
    }
    let elf = elf.unwrap_or_else(|| find_root().join("Game Files").join("SLUS_200.90"));
    init_memory();
    load_elf_data(&elf).expect("cannot load executable");
    let syms = read_symbols(&elf).unwrap();
    let gp = syms.iter().find(|s| s.name == "_gp").unwrap().addr as u64;
    let split = syms.iter().find(|s| s.name == "text_data_split").unwrap().addr;
    let externs: HashMap<u32, String> =
        syms.iter().filter(|s| s.is_func && s.addr >= split).map(|s| (s.addr, s.name.clone())).collect();
    let ctx = Ctx { externs, gp };

    let (mut pass, mut fail, mut skip, mut vacuous) = (0, 0, 0, 0);
    let mut failures = Vec::new();
    for &(addr, name, ni, nf, _rk) in FUNC_INFO {
        if !names.is_empty() && !names.iter().any(|x| x == name) {
            continue;
        }
        // hand-written platform replacements (no original behaviour to compare against)
        if matches!(name, "SCEECdSearchFile") {
            skip += 1;
            continue;
        }
        let mut real = 0;
        let mut bad: Option<String> = None;
        let mut skipped: Option<String> = None;
        for t in 0..n {
            let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (addr as u64) << 20 ^ (t as u64 + 1) * 0x1234567);
            fill_scratch(&mut rng);
            let (ints, floats) = gen_inputs(&mut rng, ni, nf);
            let a = run_interp(&ctx, addr, &ints, &floats);
            if let Outcome::Skip(s) = &a {
                // TS_VERIFY_SMALL: a run whose loops do not finish is left out, the others still count
                if s == "timeout" && std::env::var_os("TS_VERIFY_SMALL").is_some() {
                    continue;
                }
                skipped = Some(s.clone());
                break;
            }
            let b = run_native(addr, &ints, &floats);
            if let Outcome::Skip(s) = &b {
                skipped = Some(s.clone());
                break;
            }
            match (&a, &b) {
                (Outcome::Ok { ret: r1, fret: f1, writes: w1, stores: s1 }, Outcome::Ok { ret: r2, fret: f2, writes: w2, stores: s2 }) => {
                    real += 1;
                    if _rk == 1 && (*r1 as i32) != (*r2 as i32) && (*r1 != *r2) {
                        bad = Some(format!("test {t}: int return differs: interp {r1:#x} native {r2:#x} (ints {ints:x?})"));
                    } else if !same_f(*f1, *f2) && _rk == 2 {
                        bad = Some(format!("test {t}: float return differs: interp {f1} native {f2} (ints {ints:x?} floats {floats:?})"));
                    } else if w1 != w2 && !stores_equal_mod_nan(s1, s2) {
                        let mut d = String::new();
                        let m1: HashMap<_, _> = w1.iter().cloned().collect();
                        let m2: HashMap<_, _> = w2.iter().cloned().collect();
                        let mut keys: Vec<_> = m1.keys().chain(m2.keys()).cloned().collect();
                        keys.sort_unstable();
                        keys.dedup();
                        let mut shown = 0;
                        for k in keys {
                            if m1.get(&k) != m2.get(&k) && shown < 4 {
                                d += &format!(" [{k:#x}: interp {:?} native {:?}]", m1.get(&k), m2.get(&k));
                                shown += 1;
                            }
                        }
                        let mut first = String::new();
                        let cover = |ss: &[trace::Store], a: u32| ss.iter().rev().find(|x| a >= x.addr && a < x.addr + x.size as u32).copied();
                        let mut dk: Vec<u32> = m1.keys().chain(m2.keys()).cloned().collect();
                        dk.sort_unstable();
                        dk.dedup();
                        if let Some(&k) = dk.iter().find(|k| m1.get(k) != m2.get(k)) {
                            let f = |s: Option<trace::Store>| match s {
                                Some(s) => format!("[{:#x} +{} = {:02x?} pc {:#x}]", s.addr, s.size, &s.new[..s.size as usize], s.tag),
                                None => "[no store]".to_string(),
                            };
                            first = format!(" | byte {k:#x}: last interp store {} / last native store {}", f(cover(s1, k)), f(cover(s2, k)));
                        }
                        bad = Some(format!("test {t}: memory writes differ ({} vs {} bytes){d}{first} (ints {ints:x?} floats {floats:?})", w1.len(), w2.len()));
                    }
                }
                (Outcome::Panic(_), Outcome::Panic(_)) | (Outcome::Trap, Outcome::Trap) => {}
                (Outcome::Panic(_), Outcome::Ok { .. }) => {} // dead loads removed by the decompiler never fault
                (Outcome::Trap, Outcome::Panic(_)) | (Outcome::Panic(_), Outcome::Trap) => {}
                (x, y) => {
                    bad = Some(format!("test {t}: outcome differs: interp {x:?} native {y:?} (ints {ints:x?} floats {floats:?})"));
                }
            }
            if bad.is_some() {
                break;
            }
        }
        if let Some(b) = bad {
            fail += 1;
            println!("FAIL {name} @ {addr:#x}: {b}");
            failures.push(name);
        } else if let Some(s) = skipped {
            skip += 1;
            if verbose {
                println!("skip {name}: {s}");
            }
        } else if real == 0 {
            vacuous += 1;
            if verbose {
                println!("vacuous {name} (every run faulted)");
            }
        } else {
            pass += 1;
            if verbose {
                println!("ok   {name} ({real} runs)");
            }
        }
    }
    println!("\n== pass {pass}, fail {fail}, skipped {skip}, vacuous {vacuous} ==");
}
