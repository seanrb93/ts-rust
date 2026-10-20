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

pub fn call_extern(name: &str, c: &mut Cpu) -> Option<()> {
    use ts_sdk::*;
    let (a0, a1, a2) = (c.r[4] as i32, c.r[5] as i32, c.r[6] as i32);
    let (d0, d1) = (c.r[4] as i64, c.r[5] as i64);
    let (f12, f13) = (f32::from_bits(c.f[12]), f32::from_bits(c.f[13]));
    let ret_i = |c: &mut Cpu, v: i64| c.r[2] = v as u64;
    let ret_i32 = |c: &mut Cpu, v: i32| c.r[2] = v as i64 as u64;
    let ret_f = |c: &mut Cpu, v: f32| c.f[0] = v.to_bits();
    match name {
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
        _ => return None,
    }
    Some(())
}

pub fn run_interp(ctx: &Ctx, addr: u32, ints: &[i64; 8], floats: &[f32; 8]) -> Outcome {
    let mut c = Cpu::new();
    for i in 0..8 {
        c.r[4 + i] = ints[i] as u64;
        c.f[12 + i] = floats[i].to_bits();
    }
    c.r[28] = ctx.gp;
    c.r[29] = match INTERP_SP.load(std::sync::atomic::Ordering::Relaxed) { 0 => STACK_TOP - 16, v => v } as u64;
    c.r[31] = SENTINEL as u64;
    c.pc = addr;
    trace::start();
    let mut unsupported: Option<String> = None;
    let res = silent(|| {
        let mut hook = |pc: u32, cpu: &mut Cpu| -> bool {
            if let Some(name) = ctx.externs.get(&pc) {
                if call_extern(name, cpu).is_none() {
                    unsupported = Some(format!("extern {name}"));
                    cpu.pc = SENTINEL;
                    return true;
                }
                cpu.pc = cpu.r[31] as u32;
                return true;
            }
            false
        };
        c.run(&mut hook)
    });
    let log = trace::stop();
    let writes = changed(&log);
    trace::rollback(&log);
    let log: Vec<trace::Store> = log.into_iter().filter(|s| !s.bulk && s.addr < STACK_LIMIT).collect();
    if let Some(u) = unsupported {
        return Outcome::Skip(u);
    }
    match res {
        Err(p) => Outcome::Panic(p),
        Ok(Err(Stop::Unsupported(s))) => Outcome::Skip(s),
        Ok(Err(Stop::Timeout)) => Outcome::Skip("timeout".into()),
        Ok(Err(Stop::Trap)) => Outcome::Trap,
        Ok(Ok(())) => Outcome::Ok { ret: c.r[2] as i64, fret: f32::from_bits(c.f[0]), writes, stores: log },
    }
}

pub fn is_nan_bytes(b: &[u8; 16]) -> bool {
    let w = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    (w >> 23) & 0xff == 0xff && w & 0x7f_ffff != 0
}

/// Store sequences equal, treating any two NaN words as equal (x86 NaN sign / payload is not meaningful).
pub fn stores_equal_mod_nan(s1: &[trace::Store], s2: &[trace::Store]) -> bool {
    s1.len() == s2.len()
        && s1.iter().zip(s2).all(|(x, y)| {
            x.addr == y.addr && x.size == y.size && (x.new == y.new || (x.size == 4 && is_nan_bytes(&x.new) && is_nan_bytes(&y.new)))
        })
}
