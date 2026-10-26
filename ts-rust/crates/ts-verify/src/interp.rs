//! Shared interpreter plumbing: run original machine code and capture its effects.
use crate::cpu::*;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use ts_rt::*;

pub const SCRATCH: u32 = 0x0180_0000;
pub const SCRATCH_LEN: u32 = 0x4_0000;
pub static INTERP_SP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub const STACK_LIMIT: u32 = STACK_TOP - 0x10_0000;

#[derive(Debug)]
pub enum Outcome {
    Ok { ret: i64, fret: f32, writes: Vec<(u32, u8)>, stores: Vec<trace::Store> },
    Panic(String),
    Trap,
    Skip(String),
}

pub fn silent<R>(f: impl FnOnce() -> R) -> Result<R, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|e| {
        e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()
    })
}

/// Reduce a trace log to the set of bytes whose value actually changed.
pub fn changed(log: &[trace::Store]) -> Vec<(u32, u8)> {
    let mut first: HashMap<u32, u8> = HashMap::new();
    for (a, old) in trace::undo_list(log) {
        first.entry(a).or_insert(old);
    }
    let mut v: Vec<(u32, u8)> = first
        .into_iter()
        .filter(|&(a, _)| a < STACK_LIMIT)
        .filter_map(|(a, old)| {
            let new = unsafe { *host_ptr(a as i32) };
            if new != old { Some((a, new)) } else { None }
        })
        .collect();
    v.sort_unstable();
    v
}

pub struct Ctx {
    pub externs: HashMap<u32, String>,
    pub gp: u64,
}

/// One call of a platform routine that the original code made while the replies were stubbed (`TS_SHADOW_EXTERN`): the name and the first eight arguments as 32-bit numbers.
pub type ExternCall = (String, Vec<i32>);

thread_local! {
    /// In the live check the routines that read the disc are not stubbed (the game would miss the file it loads): a function that calls one is left unchecked.
    pub static DENY_DISC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub static STUB_VARIANT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Whether the routines that the interpreter cannot run answer with `stub_value` and are logged (set by the live shadow check).
    pub static STUB_EXTERNS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub static EXTERN_LOG: std::cell::RefCell<Vec<ExternCall>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Take what was logged since the last call.
pub fn take_extern_log() -> Vec<ExternCall> {
    EXTERN_LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
}

/// The reply both sides get for the `index`-th stubbed call of a check (`variant` changes from check to check, so that different branches of the callers are reached).
pub fn stub_value(name: &str, index: usize, variant: usize) -> i64 {
    let list: &[i64] = match name {
        "scePadGetState" => &[6, 1, 0, 2],
        "scePadGetSlotMax" => &[1, 4],
        "scePadGetReqState" => &[0, 1],
        "scePadRead" => &[1, 0],
        "sceMcSync" => &[1, 1, 0, -1],
        "malloc" => return 0x0170_0000,
        "sceSifAllocIopHeap" => return 0x1_0000 + index as i64 * 0x1000,
        "scePadPortOpen" | "scePadInit" | "sceMtapInit" | "sceMtapPortOpen" | "sceSifLoadModule" | "sceMcInit" => &[1],
        "sceMtapGetConnection" => &[0, 1, 1],
        "CreateSema" => return 1,
        _ => &[0, 0, 1, -1, 2],
    };
    list[(variant + index) % list.len()]
}

/// Platform routines that the interpreter runs itself or answers with 0 (they are neither stubbed nor logged).
const NATIVE_EXTERNS: &[&str] = &[
    "sscanf",
    "sceCdRead", "sceCdSync", "sceCdInit", "sceCdMmode", "sceCdDiskReady", "sceCdGetError", "sceCdReadClock", "sceOpen", "sceLseek", "sceClose", "sceWrite",
    "FlushCache",
    "InvalidDCache",
    "__divdi3",
    "acosf",
    "asinf",
    "atan",
    "atan2",
    "atan2f",
    "atoi",
    "bcopy",
    "cosf",
    "dpadd",
    "dpcmp",
    "dpdiv",
    "dpmul",
    "dpsub",
    "dptofp",
    "dptoli",
    "dptoul",
    "exit",
    "floor",
    "floorf",
    "fptodp",
    "fptoui",
    "memcpy",
    "memset",
    "pow",
    "powf",
    "sceSdRemote",
    "sceSdTransToIOP",
    "sceVu0CopyMatrix",
    "sceVu0InversMatrix",
    "sceVu0MulMatrix",
    "sceVu0TransposeMatrix",
    "sceVu0UnitMatrix",
    "sinf",
    "sprintf",
    "sqrt",
    "sqrtf",
    "strcasecmp",
    "strcat",
    "strchr",
    "strcmp",
    "strcpy",
    "strlen",
    "strncmp",
    "strncpy",
    "tanf",
    "vsprintf",
];

/// What a stubbed routine writes through its pointer arguments (the same on both sides): the memory card's answer, so that the code that reads it does not read stale stack.
pub fn stub_effect(name: &str, args: &[i32], ret: i64, index: usize, variant: usize) {
    if name == "sceMcSync" && ret == 1 && args.len() > 2 {
        if args[1] != 0 {
            sw(args[1], 0);
        }
        if args[2] != 0 {
            sw(args[2], [0, -2, 3, 1, -1, 7][(variant + index) % 6]);
        }
    }
}

pub fn is_disc_routine(name: &str) -> bool {
    ["sceCd", "sceOpen", "sceClose", "sceRead", "sceWrite", "sceLseek", "sceFs", "sceDev", "sceIo"].iter().any(|p| name.starts_with(p))
}

pub fn extern_is_native(name: &str) -> bool {
    NATIVE_EXTERNS.contains(&name)
}

pub fn call_extern(name: &str, c: &mut Cpu) -> Option<()> {
    use ts_sdk::*;
    let (a0, a1, a2) = (c.r[4] as i32, c.r[5] as i32, c.r[6] as i32);
    let (d0, d1) = (c.r[4] as i64, c.r[5] as i64);
    let (f12, f13) = (f32::from_bits(c.f[12]), f32::from_bits(c.f[13]));
    let ret_i = |c: &mut Cpu, v: i64| c.r[2] = v as u64;
    let ret_i32 = |c: &mut Cpu, v: i32| c.r[2] = v as i64 as u64;
    let ret_f = |c: &mut Cpu, v: f32| c.f[0] = v.to_bits();
    match name {
        // calls to the console that change nothing in the game's memory (the port's sound transfer writes only to the sound chip's memory)
        "FlushCache" | "InvalidDCache" | "sceSdTransToIOP" | "sceSdRemote" => ret_i32(c, 0),
        // the disc and the files of the virtual disc: the port's own routines (they read files and change nothing but the memory, which the trace undoes, and the list of open files, which the game closes again)
        "sceCdRead" | "sceCdSync" | "sceCdInit" | "sceCdMmode" | "sceCdDiskReady" | "sceCdGetError" | "sceCdReadClock" | "sceOpen" | "sceLseek" | "sceClose" | "sceWrite" => {
            let args: Vec<i64> = (4..12).map(|i| c.r[i] as i32 as i64).collect();
            let r = match name {
                "sceCdRead" => sceCdRead(&args),
                "sceCdSync" => sceCdSync(&args),
                "sceCdInit" => sceCdInit(&args),
                "sceCdMmode" => sceCdMmode(&args),
                "sceCdDiskReady" => sceCdDiskReady(&args),
                "sceCdGetError" => sceCdGetError(&args),
                "sceCdReadClock" => sceCdReadClock(&args),
                "sceOpen" => sceOpen(&args),
                "sceLseek" => sceLseek(&args),
                "sceClose" => sceClose(&args),
                _ => sceWrite(&args),
            };
            ret_i32(c, r)
        }
        "memset" => ret_i32(c, memset(a0, a1, a2)),
        "memcpy" => ret_i32(c, memcpy(a0, a1, a2)),
        "bcopy" => ret_i32(c, bcopy(a0, a1, a2)),
        "strlen" => ret_i32(c, strlen(a0)),
        "strcpy" => ret_i32(c, strcpy(a0, a1)),
        "strncpy" => ret_i32(c, strncpy(a0, a1, a2)),
        "strcmp" => ret_i32(c, strcmp(a0, a1)),
        "strncmp" => ret_i32(c, strncmp(a0, a1, a2)),
        "strcat" => ret_i32(c, strcat(a0, a1)),
        "strchr" => ret_i32(c, strchr(a0, a1)),
        "strcasecmp" => ret_i32(c, strcasecmp(a0, a1)),
        "atoi" => ret_i32(c, atoi(a0)),
        "sceVu0MulMatrix" => ret_i32(c, sceVu0MulMatrix(a0, a1, a2)),
        "sceVu0CopyMatrix" => ret_i32(c, sceVu0CopyMatrix(a0, a1)),
        "sceVu0UnitMatrix" => ret_i32(c, sceVu0UnitMatrix(a0)),
        "sceVu0TransposeMatrix" => ret_i32(c, sceVu0TransposeMatrix(a0, a1)),
        "sceVu0InversMatrix" => ret_i32(c, sceVu0InversMatrix(a0, a1)),
        "vsprintf" => ret_i32(c, vsprintf(a0, a1, a2)),
        "sscanf" => {
            let v: Vec<i64> = (4..12).map(|i| c.r[i] as i64).collect();
            ret_i32(c, sscanf(&v))
        }
        "sprintf" => {
            let v: Vec<i64> = (4..12).map(|i| c.r[i] as i64).collect();
            ret_i32(c, sprintf(&v))
        }
        "sqrtf" => ret_f(c, sqrtf(f12)),
        "sinf" => ret_f(c, sinf(f12)),
        "cosf" => ret_f(c, cosf(f12)),
        "tanf" => ret_f(c, tanf(f12)),
        "acosf" => ret_f(c, acosf(f12)),
        "asinf" => ret_f(c, asinf(f12)),
        "floorf" => ret_f(c, floorf(f12)),
        "atan2f" => ret_f(c, atan2f(f12, f13)),
        "powf" => ret_f(c, powf(f12, f13)),
        "fptodp" => ret_i(c, fptodp(f12)),
        "dptofp" => ret_f(c, dptofp(d0)),
        "dpadd" => ret_i(c, dpadd(d0, d1)),
        "dpsub" => ret_i(c, dpsub(d0, d1)),
        "dpmul" => ret_i(c, dpmul(d0, d1)),
        "dpdiv" => ret_i(c, dpdiv(d0, d1)),
        "dpcmp" => ret_i32(c, dpcmp(d0, d1)),
        "dptoli" => ret_i32(c, dptoli(d0)),
        "dptoul" => ret_i32(c, dptoul(d0)),
        "fptoui" => ret_i32(c, fptoui(f12)),
        "sqrt" => ret_i(c, sqrt(d0)),
        "atan" => ret_i(c, atan(d0)),
        "floor" => ret_i(c, floor(d0)),
        "atan2" => ret_i(c, atan2(d0, d1)),
        "pow" => ret_i(c, pow(d0, d1)),
        "__divdi3" => ret_i(c, __divdi3(d0, d1)),
        _ => return None,
    }
    Some(())
}

/// The addresses of the instructions the interpreter ran (kept only when `TS_VERIFY_COVERAGE` is set, to show which parts of a function the tests reach).
pub static COVERAGE: std::sync::Mutex<Option<std::collections::HashSet<u32>>> = std::sync::Mutex::new(None);

/// What one run of the original code ended with (without the memory writes, which `run_interp` collects).
pub struct Ran {
    pub res: Result<Result<(), Stop>, String>,
    pub unsupported: Option<String>,
    pub exited: bool,
    pub ret: i64,
    pub fret: f32,
}

/// Run the original code at `addr` in the interpreter; the caller keeps the trace of the stores (and rolls them back).
pub fn exec_interp(ctx: &Ctx, addr: u32, ints: &[i64; 8], floats: &[f32; 8]) -> Ran {
    let mut c = Cpu::new();
    for i in 0..8 {
        c.r[4 + i] = ints[i] as u64;
        c.f[12 + i] = floats[i].to_bits();
    }
    c.r[28] = ctx.gp;
    c.r[29] = match INTERP_SP.load(std::sync::atomic::Ordering::Relaxed) { 0 => STACK_TOP - 16, v => v } as u64;
    c.r[31] = SENTINEL as u64;
    c.pc = addr;
    let mut unsupported: Option<String> = None;
    let mut exited = false;
    let res = silent(|| {
        let cover = std::env::var_os("TS_VERIFY_COVERAGE").is_some();
        // `TS_VERIFY_WATCH=hexaddress`: print the caller and the arguments each time the original enters that address
        let watch_pc = std::env::var("TS_VERIFY_WATCH").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok());
        let mut seen: Vec<u32> = Vec::new();
        let mut hook = |pc: u32, cpu: &mut Cpu| -> bool {
            if cover {
                seen.push(pc);
                if seen.len() >= 4096 {
                    COVERAGE.lock().unwrap().get_or_insert_with(Default::default).extend(seen.drain(..));
                }
            }
            if let Some(watch) = watch_pc {
                if pc == watch {
                    println!("  entered {pc:#x} from {:#x}: a0 {:#x} a1 {:#x} a2 {:#x} a3 {:#x} t0 {:#x} f12 {:#x}", cpu.r[31], cpu.r[4], cpu.r[5], cpu.r[6], cpu.r[7], cpu.r[8], cpu.f[12]);
                }
            }
            if let Some(name) = ctx.externs.get(&pc) {
                if name == "exit" {
                    exited = true;
                    cpu.pc = SENTINEL;
                    return true;
                }
                if call_extern(name, cpu).is_none() {
                    if STUB_EXTERNS.with(|s| s.get()) && !(DENY_DISC.with(|d| d.get()) && is_disc_routine(name)) {
                        let index = EXTERN_LOG.with(|l| l.borrow().len());
                        EXTERN_LOG.with(|l| l.borrow_mut().push((name.clone(), (4..12).map(|i| cpu.r[i] as i32).collect())));
                        let variant = STUB_VARIANT.with(|v| v.get());
                        let value = stub_value(name, index, variant);
                        cpu.r[2] = value as u64;
                        stub_effect(name, &(4..12).map(|i| cpu.r[i] as i32).collect::<Vec<_>>(), value, index, variant);
                    } else {
                        unsupported = Some(format!("extern {name}"));
                        cpu.pc = SENTINEL;
                        return true;
                    }
                }
                cpu.pc = cpu.r[31] as u32;
                return true;
            }
            false
        };
        let r = c.run(&mut hook);
        if cover {
            COVERAGE.lock().unwrap().get_or_insert_with(Default::default).extend(seen.drain(..));
        }
        r
    });
    Ran { res, unsupported, exited, ret: c.r[2] as i64, fret: f32::from_bits(c.f[0]) }
}

pub fn run_interp(ctx: &Ctx, addr: u32, ints: &[i64; 8], floats: &[f32; 8]) -> Outcome {
    trace::start();
    let ran = exec_interp(ctx, addr, ints, floats);
    let log = trace::stop();
    let writes = changed(&log);
    trace::rollback(&log);
    let log: Vec<trace::Store> = log.into_iter().filter(|s| !s.bulk && s.addr < STACK_LIMIT).collect();
    if let Some(u) = ran.unsupported {
        return Outcome::Skip(u);
    }
    if ran.exited {
        return Outcome::Trap;
    }
    match ran.res {
        Err(p) => Outcome::Panic(p),
        Ok(Err(Stop::Unsupported(s))) => Outcome::Skip(s),
        Ok(Err(Stop::Timeout)) => Outcome::Skip("timeout".into()),
        Ok(Err(Stop::Trap)) => Outcome::Trap,
        Ok(Ok(())) => Outcome::Ok { ret: ran.ret, fret: ran.fret, writes, stores: log },
    }
}

pub fn is_nan_bytes(b: &[u8; 16]) -> bool {
    let w = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    (w >> 23) & 0xff == 0xff && w & 0x7f_ffff != 0
}

fn lane_is_nan(b: &[u8]) -> bool {
    let w = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    (w >> 23) & 0xff == 0xff && w & 0x7f_ffff != 0
}

/// Two stored values equal, treating a NaN in the same 4-byte lane of both as equal (x86 NaN sign / payload is not meaningful); the 8- and 16-byte stores are compared lane by lane.
fn same_mod_nan(x: &[u8], y: &[u8], size: usize) -> bool {
    if x[..size] == y[..size] {
        return true;
    }
    if size % 4 != 0 {
        return false;
    }
    (0..size / 4).all(|k| {
        let (a, b) = (&x[k * 4..k * 4 + 4], &y[k * 4..k * 4 + 4]);
        a == b || (lane_is_nan(a) && lane_is_nan(b))
    })
}

/// Store sequences equal, treating any two NaN words as equal (x86 NaN sign / payload is not meaningful).
pub fn stores_equal_mod_nan(s1: &[trace::Store], s2: &[trace::Store]) -> bool {
    s1.len() == s2.len() && s1.iter().zip(s2).all(|(x, y)| x.addr == y.addr && x.size == y.size && same_mod_nan(&x.new, &y.new, x.size as usize))
}
