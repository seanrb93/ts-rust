//! Runtime for the natively ported VU1 microcode (`vu1_gen.rs`): register file, FMAC helpers with the vector unit's
//! float behaviour (no NaN/Inf, denormals flushed), MAC/clip flags and the Q-register divide latency.
//!
//! This is *not* an instruction interpreter: the generated code is straight Rust (one function, basic blocks from
//! the original control flow); this module only provides the arithmetic building blocks it calls.

use crate::vif::{Vu1, VU_MEM_QW};


#[derive(Clone)]
pub struct Regs {
    pub vf: [[f32; 4]; 32],
    pub vi: [u16; 16],
    pub acc: [f32; 4],
    pub q: f32,
    pub q_pend: f32,
    pub q_ready: u64,
    pub p: f32,
    pub i: f32,
    pub mac: u16,
    pub clip: u32,
    pub cycle: u64,
}

impl Default for Regs {
    fn default() -> Self {
        let mut vf = [[0.0f32; 4]; 32];
        vf[0] = [0.0, 0.0, 0.0, 1.0];
        Regs { vf, vi: [0; 16], acc: [0.0; 4], q: 0.0, q_pend: 0.0, q_ready: 0, p: 0.0, i: 0.0, mac: 0, clip: 0, cycle: 0 }
    }
}

/// Clamp a result to what the VU can represent: overflow to +-FLT_MAX, NaN to 0, denormals to signed zero.
#[inline(always)]
pub fn fz(x: f32) -> f32 {
    if x.is_nan() {
        0.0
    } else if x.is_infinite() {
        f32::MAX.copysign(x)
    } else if x != 0.0 && x.abs() < f32::MIN_POSITIVE {
        0.0f32.copysign(x)
    } else {
        x
    }
}

#[inline(always)]
pub fn ftoi(x: f32, scale: f32) -> u32 {
    let v = (fz(x) * scale) as i32; // saturating, truncating
    v as u32
}

impl Regs {
    #[inline(always)]
    pub fn tick(&mut self) {
        self.cycle += 1;
        if self.cycle >= self.q_ready {
            self.q = self.q_pend;
        }
    }

    /// Write the lanes of `r` selected by `mask` (bit n = lane n) to vf[fd]; update MAC flags if `flags`.
    #[inline(always)]
    pub fn commit(&mut self, fd: usize, mask: u8, r: [f32; 4], flags: bool) {
        let mut mac = 0u16;
        for l in 0..4 {
            if mask & (1 << l) == 0 {
                continue;
            }
            let raw = r[l];
            let v = fz(raw);
            if flags {
                let sh = 3 - l; // x -> bit 3 .. w -> bit 0
                if v == 0.0 {
                    mac |= 1 << sh;
                }
                if raw.is_sign_negative() && raw != 0.0 {
                    mac |= 0x10 << sh;
                }
                if raw.is_infinite() {
                    mac |= 0x100 << sh;
                }
            }
            if fd != 0 {
                self.vf[fd][l] = v;
            }
        }
        if flags {
            self.mac = mac;
        }
    }

    /// `commit_acc` for code whose MAC flags are never read.
    #[inline(always)]
    pub fn commit_acc_nf(&mut self, mask: u8, r: [f32; 4]) {
        for l in 0..4 {
            if mask & (1 << l) != 0 {
                self.acc[l] = fz(r[l]);
            }
        }
    }

    #[inline(always)]
    pub fn commit_acc(&mut self, mask: u8, r: [f32; 4]) {
        let mut mac = 0u16;
        for l in 0..4 {
            if mask & (1 << l) == 0 {
                continue;
            }
            let raw = r[l];
            let v = fz(raw);
            let sh = 3 - l;
            if v == 0.0 {
                mac |= 1 << sh;
            }
            if raw.is_sign_negative() && raw != 0.0 {
                mac |= 0x10 << sh;
            }
            if raw.is_infinite() {
                mac |= 0x100 << sh;
            }
            self.acc[l] = v;
        }
        self.mac = mac;
    }

    /// Write raw lane bits (no float clamping / flags): conversions, moves, loads.
    #[inline(always)]
    pub fn put_bits(&mut self, fd: usize, mask: u8, r: [u32; 4]) {
        if fd == 0 {
            return;
        }
        for l in 0..4 {
            if mask & (1 << l) != 0 {
                self.vf[fd][l] = f32::from_bits(r[l]);
            }
        }
    }

    #[inline(always)]
    pub fn put(&mut self, fd: usize, mask: u8, r: [f32; 4]) {
        if fd == 0 {
            return;
        }
        for l in 0..4 {
            if mask & (1 << l) != 0 {
                self.vf[fd][l] = r[l];
            }
        }
    }

    #[inline(always)]
    pub fn vi_set(&mut self, r: usize, v: u16) {
        if r != 0 {
            self.vi[r] = v;
        }
    }

    pub fn clip_op(&mut self, fs: [f32; 4], ft_w: f32) {
        let w = fz(ft_w).abs();
        let mut b = 0u32;
        if fs[0] > w { b |= 1 }
        if fs[0] < -w { b |= 2 }
        if fs[1] > w { b |= 4 }
        if fs[1] < -w { b |= 8 }
        if fs[2] > w { b |= 16 }
        if fs[2] < -w { b |= 32 }
        self.clip = ((self.clip << 6) | b) & 0xff_ffff;
        if crate::trace::vu_enabled() {
            crate::trace::line(format!("      CLIP fs={:?} w={} bits={:#x} clip={:#x}", fs, ft_w, b, self.clip));
        }
    }

    #[inline(always)]
    pub fn div(&mut self, a: f32, b: f32) {
        let r = if b == 0.0 { f32::MAX.copysign(a) * if b.is_sign_negative() { -1.0 } else { 1.0 } } else { fz(a / b) };
        self.q_pend = if a == 0.0 && b == 0.0 { 0.0 } else { r };
        self.q_ready = self.cycle + 7;
    }

    #[inline(always)]
    pub fn sqrt(&mut self, b: f32) {
        self.q_pend = fz(b.abs().sqrt());
        self.q_ready = self.cycle + 7;
    }

    #[inline(always)]
    pub fn rsqrt(&mut self, a: f32, b: f32) {
        let s = b.abs().sqrt();
        self.q_pend = if s == 0.0 { f32::MAX.copysign(a) } else { fz(a / s) };
        self.q_ready = self.cycle + 13;
    }

    #[inline(always)]
    pub fn waitq(&mut self) {
        if self.cycle < self.q_ready {
            self.cycle = self.q_ready;
        }
        self.q = self.q_pend;
    }
}

#[inline(always)]
pub fn qw(vi: u16, imm: i32) -> usize {
    ((vi as i32 + imm) as usize) & (VU_MEM_QW - 1)
}

#[inline(always)]
pub fn mem_lane(vu: &Vu1, q: usize, l: usize) -> u32 {
    vu.mem[q][l]
}

impl Regs {
    /// The state of the production register file (used by the cross-check of `programs::run`).
    pub fn from_new(r: &crate::vu::Regs) -> Regs {
        Regs { vf: r.vf, vi: r.vi, acc: r.acc, q: r.q_now(), q_pend: r.q_pend, q_ready: r.q_ready, p: r.p, i: r.i, mac: r.mac, clip: r.clip, cycle: r.cycle }
    }
}
