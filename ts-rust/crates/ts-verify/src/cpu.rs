//! A deliberately small R5900 interpreter used ONLY to check the decompiler's output.
//! It implements the integer / FPU / load-store subset that appears in the game's code, with the arithmetic of the console's FPU (`ts_rt::ps2float`: no
//! infinity and no NaN, denormals are zero) and rounding to the nearest like the decompiled Rust, so any difference between the two executions is a decompiler bug.

use ts_rt::*;

pub const SENTINEL: u32 = 0xFFFF_FFF0;
pub const STEP_LIMIT: u64 = 60_000_000;

/// The number of instructions after which a run of the original is given up (`TS_VERIFY_STEPS` changes it: the loads of big files need more).
fn step_limit() -> u64 {
    static LIMIT: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *LIMIT.get_or_init(|| std::env::var("TS_VERIFY_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(STEP_LIMIT))
}

#[derive(Debug)]
pub enum Stop {
    Unsupported(String),
    Timeout,
    Trap,
}

pub struct Cpu {
    pub r: [u64; 32],
    pub rh: [u64; 32],
    pub hi: u64,
    pub lo: u64,
    pub hi1: u64,
    pub lo1: u64,
    pub f: [u32; 32],
    pub fcc: bool,
    pub acc: f32,
    pub vf: [[u32; 4]; 32],
    pub vacc: [f32; 4],
    pub vq: f32,
    pub vi_: f32,
    pub pc: u32,
    pub steps: u64,
}

#[inline]
fn sx32(v: u32) -> u64 {
    v as i32 as i64 as u64
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { r: [0; 32], rh: [0; 32], hi: 0, lo: 0, hi1: 0, lo1: 0, f: [0; 32], fcc: false, acc: 0.0, vf: { let mut v = [[0u32; 4]; 32]; v[0][3] = 0x3f80_0000; v }, vacc: [0.0; 4], vq: 0.0, vi_: 0.0, pc: 0, steps: 0 }
    }

    #[inline]
    fn fr(&self, i: usize) -> f32 {
        f32::from_bits(self.f[i])
    }
    #[inline]
    fn setf(&mut self, i: usize, v: f32) {
        self.f[i] = v.to_bits();
    }

    /// Run until the function returns to SENTINEL.  `hook(pc, cpu)` may handle a call natively (returning true after
    /// it has set `cpu.pc` to the return address).
    pub fn run(&mut self, hook: &mut dyn FnMut(u32, &mut Cpu) -> bool) -> Result<(), Stop> {
        while self.pc != SENTINEL {
            self.steps += 1;
            if self.steps > step_limit() {
                // `TS_VERIFY_TIMEOUT_PC=1`: say where a run that does not finish was when it was given up (the loop it is in)
                if std::env::var_os("TS_VERIFY_TIMEOUT_PC").is_some() {
                    eprintln!("[verify] gave up at pc {:#x}; a0..a3 {:#x} {:#x} {:#x} {:#x} v0 {:#x} v1 {:#x} ra {:#x} s0..s3 {:#x} {:#x} {:#x} {:#x}", self.pc, self.r[4], self.r[5], self.r[6], self.r[7], self.r[2], self.r[3], self.r[31], self.r[16], self.r[17], self.r[18], self.r[19]);
                }
                return Err(Stop::Timeout);
            }
            if hook(self.pc, self) {
                continue;
            }
            self.step()?;
        }
        Ok(())
    }

    fn delay(&mut self, addr: u32) -> Result<(), Stop> {
        let w = lw(addr as i32) as u32;
        trace::set_tag(addr);
        self.exec(w, addr)
    }

    fn step(&mut self) -> Result<(), Stop> {
        let pc = self.pc;
        let w = lw(pc as i32) as u32;
        let op = w >> 26;
        let rs = ((w >> 21) & 31) as usize;
        let rt = ((w >> 16) & 31) as usize;
        let rd = ((w >> 11) & 31) as usize;
        let imm = (w & 0xffff) as i16 as i64;
        let btarget = pc.wrapping_add(4).wrapping_add((imm << 2) as u32);
        let (a, b) = (self.r[rs], self.r[rt]);
        macro_rules! br {
            ($taken:expr, $likely:expr) => {{
                let t: bool = $taken;
                if t {
                    self.delay(pc + 4)?;
                    self.pc = btarget;
                } else if $likely {
                    self.pc = pc + 8;
                } else {
                    self.delay(pc + 4)?;
                    self.pc = pc + 8;
                }
                return Ok(());
            }};
        }
        match op {
            0 => match w & 63 {
                8 => {
                    // jr
                    let t = a as u32;
                    self.delay(pc + 4)?;
                    self.pc = t;
                    return Ok(());
                }
                9 => {
                    let t = a as u32;
                    self.delay(pc + 4)?;
                    if rd != 0 {
                        self.r[rd] = (pc + 8) as u64;
                    }
                    self.pc = t;
                    return Ok(());
                }
                _ => {}
            },
            1 => match rt {
                0 => br!((a as i64) < 0, false),
                1 => br!((a as i64) >= 0, false),
                2 => br!((a as i64) < 0, true),
                3 => br!((a as i64) >= 0, true),
                _ => return Err(Stop::Unsupported(format!("regimm {rt}"))),
            },
            2 => {
                let t = (pc & 0xf000_0000) | ((w & 0x03ff_ffff) << 2);
                self.delay(pc + 4)?;
                self.pc = t;
                return Ok(());
            }
            3 => {
                let t = (pc & 0xf000_0000) | ((w & 0x03ff_ffff) << 2);
                self.delay(pc + 4)?;
                self.r[31] = (pc + 8) as u64;
                self.pc = t;
                return Ok(());
            }
            4 => br!(a == b, false),
            5 => br!(a != b, false),
            6 => br!((a as i64) <= 0, false),
            7 => br!((a as i64) > 0, false),
            20 => br!(a == b, true),
            21 => br!(a != b, true),
            22 => br!((a as i64) <= 0, true),
            23 => br!((a as i64) > 0, true),
            17 if rs == 8 => {
                let c = self.fcc;
                match rt {
                    0 => br!(!c, false),
                    1 => br!(c, false),
                    2 => br!(!c, true),
                    3 => br!(c, true),
                    _ => {}
                }
            }
            _ => {}
        }
        trace::set_tag(pc);
        self.exec(w, pc)?;
        self.pc = pc + 4;
        Ok(())
    }

    fn exec(&mut self, w: u32, pc: u32) -> Result<(), Stop> {
        let op = w >> 26;
        let rs = ((w >> 21) & 31) as usize;
        let rt = ((w >> 16) & 31) as usize;
        let rd = ((w >> 11) & 31) as usize;
        let sa = (w >> 6) & 31;
        let imm = (w & 0xffff) as i16 as i64;
        let uimm = (w & 0xffff) as u64;
        let (a, b) = (self.r[rs], self.r[rt]);
        let ea = (a as u32).wrapping_add(imm as u32) as i32;
        let unsup = |s: &str| Err(Stop::Unsupported(format!("{s} @ {pc:#x} ({w:08x})")));
        match op {
            0 => {
                let f = w & 63;
                let (a32, b32) = (a as u32, b as u32);
                let v: Option<u64> = match f {
                    0 => Some(sx32(b32 << sa)),
                    2 => Some(sx32(b32 >> sa)),
                    3 => Some(sx32(((b32 as i32) >> sa) as u32)),
                    4 => Some(sx32(b32 << (a32 & 31))),
                    6 => Some(sx32(b32 >> (a32 & 31))),
                    7 => Some(sx32(((b32 as i32) >> (a32 & 31)) as u32)),
                    10 => {
                        if b == 0 {
                            Some(a)
                        } else {
                            Some(self.r[rd])
                        }
                    }
                    11 => {
                        if b != 0 {
                            Some(a)
                        } else {
                            Some(self.r[rd])
                        }
                    }
                    13 => return Err(Stop::Trap),
                    12 => return unsup("syscall"),
                    15 => None, // sync
                    16 => Some(self.hi),
                    17 => {
                        self.hi = a;
                        None
                    }
                    18 => Some(self.lo),
                    19 => {
                        self.lo = a;
                        None
                    }
                    20 => Some(b << (a & 63)),
                    22 => Some(b >> (a & 63)),
                    23 => Some(((b as i64) >> (a & 63)) as u64),
                    24 | 25 => {
                        let p: u64 = if f == 24 {
                            ((a32 as i32 as i64) * (b32 as i32 as i64)) as u64
                        } else {
                            (a32 as u64) * (b32 as u64)
                        };
                        self.lo = sx32(p as u32);
                        self.hi = sx32((p >> 32) as u32);
                        if rd != 0 {
                            Some(self.lo)
                        } else {
                            None
                        }
                    }
                    26 => {
                        if b32 != 0 {
                            let (x, y) = (a32 as i32, b32 as i32);
                            self.lo = sx32(x.wrapping_div(y) as u32);
                            self.hi = sx32(x.wrapping_rem(y) as u32);
                        }
                        None
                    }
                    27 => {
                        if b32 != 0 {
                            self.lo = sx32(a32 / b32);
                            self.hi = sx32(a32 % b32);
                        }
                        None
                    }
                    32 | 33 => Some(sx32(a32.wrapping_add(b32))),
                    34 | 35 => Some(sx32(a32.wrapping_sub(b32))),
                    36 => Some(a & b),
                    37 => Some(a | b),
                    38 => Some(a ^ b),
                    39 => Some(!(a | b)),
                    42 => Some(((a as i64) < (b as i64)) as u64),
                    43 => Some((a < b) as u64),
                    44 | 45 => Some(a.wrapping_add(b)),
                    46 | 47 => Some(a.wrapping_sub(b)),
                    56 => Some(b << sa),
                    58 => Some(b >> sa),
                    59 => Some(((b as i64) >> sa) as u64),
                    60 => Some(b << (sa + 32)),
                    62 => Some(b >> (sa + 32)),
                    63 => Some(((b as i64) >> (sa + 32)) as u64),
                    _ => return unsup("special"),
                };
                if let Some(v) = v {
                    if rd != 0 {
                        self.r[rd] = v;
                    }
                }
            }
            8 | 9 => {
                if rt != 0 {
                    self.r[rt] = sx32((a as u32).wrapping_add(imm as u32));
                }
            }
            24 | 25 => {
                if rt != 0 {
                    self.r[rt] = a.wrapping_add(imm as u64);
                }
            }
            10 => self.set(rt, ((a as i64) < imm) as u64),
            11 => self.set(rt, (a < imm as u64) as u64),
            12 => self.set(rt, a & uimm),
            13 => self.set(rt, a | uimm),
            14 => self.set(rt, a ^ uimm),
            15 => self.set(rt, sx32((uimm as u32) << 16)),
            // loads
            32 => self.set(rt, lb(ea) as i64 as u64),
            33 => self.set(rt, lh(ea) as i64 as u64),
            35 => self.set(rt, lw(ea) as i64 as u64),
            36 => self.set(rt, lbu(ea) as u64),
            37 => self.set(rt, lhu(ea) as u64),
            39 => self.set(rt, lwu(ea) as u64),
            55 => self.set(rt, ld(ea) as u64),
            30 => {
                let v = lq(ea & !15);
                self.set(rt, v as u64);
                if rt != 0 {
                    self.rh[rt] = (v >> 64) as u64;
                }
            }
            54 => {
                let v = lq(ea & !15);
                if rt != 0 {
                    for k in 0..4 {
                        self.vf[rt][k] = (v >> (32 * k)) as u32;
                    }
                }
            }
            62 => {
                let v: u128 = (0..4).fold(0u128, |a, k| a | ((self.vf[rt][k] as u128) << (32 * k)));
                sq(ea & !15, v);
            }
            49 => {
                let v = lw(ea) as u32;
                self.f[rt] = v;
            }
            // stores
            40 => sb(ea, b as i32),
            41 => sh(ea, b as i32),
            43 => sw(ea, b as i32),
            63 => sd(ea, b as i64),
            31 => sq(ea & !15, (b as u128) | ((self.rh[rt] as u128) << 64)),
            57 => sw(ea, self.f[rt] as i32),
            // unaligned
            34 => {
                // lwl
                let n = (ea & 3) as u32;
                let word = lw(ea & !3) as u32;
                let keep = if n == 3 { 0 } else { (b as u32) & ((1u32 << (8 * (3 - n))) - 1) };
                let v = (word << (8 * (3 - n))) | keep;
                self.set(rt, sx32(v));
            }
            38 => {
                // lwr
                let n = (ea & 3) as u32;
                let word = lw(ea & !3) as u32;
                let keep = if n == 0 { 0 } else { (b as u32) & (u32::MAX << (32 - 8 * n)) };
                let v = (word >> (8 * n)) | keep;
                self.set(rt, sx32(v));
            }
            26 => {
                let n = (ea & 7) as u64;
                let d = ld(ea & !7) as u64;
                let keep = if n == 7 { 0 } else { b & ((1u64 << (8 * (7 - n))) - 1) };
                self.set(rt, (d << (8 * (7 - n))) | keep);
            }
            27 => {
                let n = (ea & 7) as u64;
                let d = ld(ea & !7) as u64;
                let keep = if n == 0 { 0 } else { b & (u64::MAX << (64 - 8 * n)) };
                self.set(rt, (d >> (8 * n)) | keep);
            }
            42 => {
                let n = (ea & 3) as i32;
                for k in 0..=n {
                    sb(ea - k, ((b as u32) >> (8 * (3 - k))) as i32);
                }
            }
            46 => {
                let n = (ea & 3) as i32;
                for k in 0..(4 - n) {
                    sb(ea + k, ((b as u32) >> (8 * k)) as i32);
                }
            }
            44 => {
                let n = (ea & 7) as i32;
                for k in 0..=n {
                    sb(ea - k, (b >> (8 * (7 - k))) as i32);
                }
            }
            45 => {
                let n = (ea & 7) as i32;
                for k in 0..(8 - n) {
                    sb(ea + k, (b >> (8 * k)) as i32);
                }
            }
            47 | 51 => {} // cache / pref
            28 => {
                // MMI: only madd/maddu/mult1 family appear in game code
                let f = w & 63;
                let (a32, b32) = (a as u32, b as u32);
                match f {
                    0 | 1 => {
                        let acc = ((self.hi as u32 as u64) << 32) | (self.lo as u32 as u64);
                        let p = if f == 0 {
                            ((a32 as i32 as i64) * (b32 as i32 as i64)) as u64
                        } else {
                            (a32 as u64) * (b32 as u64)
                        };
                        let n = acc.wrapping_add(p);
                        self.lo = sx32(n as u32);
                        self.hi = sx32((n >> 32) as u32);
                        if rd != 0 {
                            self.r[rd] = self.lo;
                        }
                    }
                    0x10 => self.set(rd, self.hi1),
                    0x11 => self.hi1 = a,
                    0x12 => self.set(rd, self.lo1),
                    0x13 => self.lo1 = a,
                    0x18 | 0x19 => {
                        let p: u64 = if f == 0x18 {
                            ((a32 as i32 as i64) * (b32 as i32 as i64)) as u64
                        } else {
                            (a32 as u64) * (b32 as u64)
                        };
                        self.lo1 = sx32(p as u32);
                        self.hi1 = sx32((p >> 32) as u32);
                        if rd != 0 {
                            self.r[rd] = self.lo1;
                        }
                    }
                    0x1a => {
                        if b32 != 0 {
                            let (x, y) = (a32 as i32, b32 as i32);
                            self.lo1 = sx32(x.wrapping_div(y) as u32);
                            self.hi1 = sx32(x.wrapping_rem(y) as u32);
                        }
                    }
                    0x1b => {
                        if b32 != 0 {
                            self.lo1 = sx32(a32 / b32);
                            self.hi1 = sx32(a32 % b32);
                        }
                    }
                    _ => return unsup("mmi"),
                }
            }
            17 => self.cop1(w, rs, rt, rd, sa)?,
            18 => self.cop2(w, rs, rt, rd)?,
            _ => return unsup("opcode"),
        }
        self.r[0] = 0;
        Ok(())
    }

    #[inline]
    fn set(&mut self, r: usize, v: u64) {
        if r != 0 {
            self.r[r] = v;
        }
    }

    fn cop1(&mut self, w: u32, rs: usize, rt: usize, rd: usize, sa: u32) -> Result<(), Stop> {
        let (fs, ft, fd) = (rd, rt, sa as usize);
        match rs {
            0 => self.set(rt, sx32(self.f[fs])),
            2 => self.set(rt, 0),
            4 => self.f[fs] = self.r[rt] as u32,
            6 => {}
            16 => {
                let (x, y) = (self.fr(fs), self.fr(ft));
                match w & 63 {
                    0 => self.setf(fd, ps2float::add(x, y)),
                    1 => self.setf(fd, ps2float::sub(x, y)),
                    2 => self.setf(fd, ps2float::mul(x, y)),
                    3 => self.setf(fd, ps2float::div(x, y)),
                    4 => self.setf(fd, ps2float::sqrt(y)),
                    5 => self.setf(fd, x.abs()),
                    6 => self.setf(fd, x),
                    7 => self.setf(fd, -x),
                    22 => self.setf(fd, ps2float::rsqrt(x, y)),
                    24 => self.acc = ps2float::add(x, y),
                    25 => self.acc = ps2float::sub(x, y),
                    26 => self.acc = ps2float::mul(x, y),
                    28 => self.setf(fd, ps2float::add(self.acc, ps2float::mul(x, y))),
                    29 => self.setf(fd, ps2float::sub(self.acc, ps2float::mul(x, y))),
                    30 => self.acc = ps2float::add(self.acc, ps2float::mul(x, y)),
                    31 => self.acc = ps2float::sub(self.acc, ps2float::mul(x, y)),
                    36 => self.f[fd] = (x as i32) as u32,
                    40 => self.setf(fd, x.max(y)),
                    41 => self.setf(fd, x.min(y)),
                    48 => self.fcc = false,
                    50 => self.fcc = x == y,
                    52 => self.fcc = x < y,
                    54 => self.fcc = x <= y,
                    // the signalling forms of the compares and their negations (the console's FPU has no NaN, so they are the plain ones)
                    56 => self.fcc = false,
                    58 => self.fcc = x == y,
                    59 => self.fcc = x == y,
                    60 => self.fcc = x < y,
                    61 => self.fcc = x < y,
                    62 => self.fcc = x <= y,
                    63 => self.fcc = x <= y,
                    _ => return Err(Stop::Unsupported(format!("cop1.s {}", w & 63))),
                }
            }
            20 => {
                if w & 63 == 32 {
                    self.setf(fd, self.f[fs] as i32 as f32);
                } else {
                    return Err(Stop::Unsupported("cop1.w".into()));
                }
            }
            _ => return Err(Stop::Unsupported(format!("cop1 rs={rs}"))),
        }
        Ok(())
    }

    fn cop2(&mut self, w: u32, rs: usize, rt: usize, rd: usize) -> Result<(), Stop> {
        match rs {
            1 => {
                // qmfc2
                if rt != 0 {
                    self.r[rt] = (self.vf[rd][0] as u64) | ((self.vf[rd][1] as u64) << 32);
                    self.rh[rt] = (self.vf[rd][2] as u64) | ((self.vf[rd][3] as u64) << 32);
                }
                return Ok(());
            }
            5 => {
                if rd != 0 {
                    let (a, b) = (self.r[rt], self.rh[rt]);
                    self.vf[rd] = [a as u32, (a >> 32) as u32, b as u32, (b >> 32) as u32];
                }
                return Ok(());
            }
            r if r >= 16 => {}
            _ => return Err(Stop::Unsupported(format!("cop2 rs={rs}"))),
        }
        let fn_ = w & 63;
        let (ft, fs, fd) = (((w >> 16) & 31) as usize, ((w >> 11) & 31) as usize, ((w >> 6) & 31) as usize);
        let lanes: [bool; 4] = [w >> 24 & 1 != 0, w >> 23 & 1 != 0, w >> 22 & 1 != 0, w >> 21 & 1 != 0];
        let fv = |v: u32| f32::from_bits(v);
        let a: [f32; 4] = [fv(self.vf[fs][0]), fv(self.vf[fs][1]), fv(self.vf[fs][2]), fv(self.vf[fs][3])];
        let t: [f32; 4] = [fv(self.vf[ft][0]), fv(self.vf[ft][1]), fv(self.vf[ft][2]), fv(self.vf[ft][3])];
        let acc = self.vacc;
        #[derive(Clone, Copy, PartialEq)]
        enum K { Add, Sub, Madd, Msub, Mul, Max, Min }
        // (op, operand bank: 0 vector, 1..4 broadcast x..w, 5 Q, 6 I), result goes to ACC
        let (op, bank, to_acc): (K, usize, bool);
        if fn_ < 0x3c {
            let b4 = (fn_ & 3) as usize + 1;
            (op, bank) = match fn_ {
                0x00..=0x03 => (K::Add, b4),
                0x04..=0x07 => (K::Sub, b4),
                0x08..=0x0b => (K::Madd, b4),
                0x0c..=0x0f => (K::Msub, b4),
                0x10..=0x13 => (K::Max, b4),
                0x14..=0x17 => (K::Min, b4),
                0x18..=0x1b => (K::Mul, b4),
                0x1c => (K::Mul, 5),
                0x1d => (K::Max, 6),
                0x1e => (K::Mul, 6),
                0x1f => (K::Min, 6),
                0x20 => (K::Add, 5),
                0x21 => (K::Madd, 5),
                0x22 => (K::Add, 6),
                0x23 => (K::Madd, 6),
                0x24 => (K::Sub, 5),
                0x25 => (K::Msub, 5),
                0x26 => (K::Sub, 6),
                0x27 => (K::Msub, 6),
                0x28 => (K::Add, 0),
                0x29 => (K::Madd, 0),
                0x2a => (K::Mul, 0),
                0x2b => (K::Max, 0),
                0x2c => (K::Sub, 0),
                0x2d => (K::Msub, 0),
                0x2f => (K::Min, 0),
                0x2e => {
                    // opmsub
                    let r = [acc[0] - a[1] * t[2], acc[1] - a[2] * t[0], acc[2] - a[0] * t[1]];
                    for k in 0..3 {
                        if lanes[k] && fd != 0 {
                            self.vf[fd][k] = r[k].to_bits();
                        }
                    }
                    if lanes[3] && fd != 0 {
                        self.vf[fd][3] = 0;
                    }
                    return Ok(());
                }
                _ => return Err(Stop::Unsupported(format!("cop2 upper {fn_:#x}"))),
            };
            to_acc = false;
        } else {
            let idx = ((w >> 6) & 31) * 4 + (w & 3);
            let b4 = (idx & 3) as usize + 1;
            (op, bank) = match idx {
                0..=3 => (K::Add, b4),
                4..=7 => (K::Sub, b4),
                8..=11 => (K::Madd, b4),
                12..=15 => (K::Msub, b4),
                24..=27 => (K::Mul, b4),
                28 => (K::Mul, 5),
                30 => (K::Mul, 6),
                32 => (K::Add, 5),
                33 => (K::Madd, 5),
                34 => (K::Add, 6),
                35 => (K::Madd, 6),
                36 => (K::Sub, 5),
                37 => (K::Msub, 5),
                38 => (K::Sub, 6),
                39 => (K::Msub, 6),
                40 => (K::Add, 0),
                41 => (K::Madd, 0),
                42 => (K::Mul, 0),
                44 => (K::Sub, 0),
                45 => (K::Msub, 0),
                _ => return Err(Stop::Unsupported(format!("cop2 special {idx} @ {w:08x}"))),
            };
            to_acc = true;
        }
        let b: [f32; 4] = match bank {
            0 => t,
            1..=4 => [t[bank - 1]; 4],
            5 => [self.vq; 4],
            _ => [self.vi_; 4],
        };
        let mut r = [0f32; 4];
        for k in 0..4 {
            r[k] = match op {
                K::Add => a[k] + b[k],
                K::Sub => a[k] - b[k],
                K::Mul => a[k] * b[k],
                K::Max => a[k].max(b[k]),
                K::Min => a[k].min(b[k]),
                K::Madd => acc[k] + a[k] * b[k],
                K::Msub => acc[k] - a[k] * b[k],
            };
        }
        for k in 0..4 {
            if lanes[k] {
                if to_acc {
                    self.vacc[k] = r[k];
                } else if fd != 0 {
                    self.vf[fd][k] = r[k].to_bits();
                }
            }
        }
        Ok(())
    }
}
