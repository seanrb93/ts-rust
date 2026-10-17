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

/// Switch the calling thread's floating point unit to what the vector unit does with tiny numbers: denormal operands and
/// results are zero (flush-to-zero / denormals-are-zero).  Call it on the thread that runs the translated microcode.
pub fn enter_vu_mode() {
    #[cfg(target_arch = "x86_64")]
    #[allow(deprecated)]
    unsafe {
        use std::arch::x86_64::{_mm_getcsr, _mm_setcsr};
        _mm_setcsr(_mm_getcsr() | 0x8040);
    }
}

/// Clamp a result to what the VU can represent: overflow to +-FLT_MAX, NaN to 0 (denormals are flushed by the hardware mode
/// set in `enter_vu_mode`).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn fz(x: f32) -> f32 {
    use std::arch::x86_64::*;
    // SAFETY: SSE is part of x86_64.
    unsafe {
        let v = _mm_set_ss(x);
        let c = _mm_max_ss(_mm_min_ss(v, _mm_set_ss(f32::MAX)), _mm_set_ss(-f32::MAX));
        _mm_cvtss_f32(_mm_and_ps(c, _mm_cmpord_ss(v, v)))
    }
}

/// `fz` of four lanes at once.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn fz4(r: [f32; 4]) -> [f32; 4] {
    use std::arch::x86_64::*;
    // SAFETY: SSE is part of x86_64; the array has four floats.
    unsafe {
        let v = _mm_loadu_ps(r.as_ptr());
        let c = _mm_max_ps(_mm_min_ps(v, _mm_set1_ps(f32::MAX)), _mm_set1_ps(-f32::MAX));
        let c = _mm_and_ps(c, _mm_cmpord_ps(v, v));
        let mut out = [0.0f32; 4];
        _mm_storeu_ps(out.as_mut_ptr(), c);
        out
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn fz4(r: [f32; 4]) -> [f32; 4] {
    [fz(r[0]), fz(r[1]), fz(r[2]), fz(r[3])]
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn fz(x: f32) -> f32 {
    let b = x.to_bits();
    if (b & 0x7f80_0000).wrapping_sub(0x0080_0000) < 0x7f00_0000 {
        return x;
    }
    let sign = b & 0x8000_0000;
    if b & 0x7f80_0000 == 0 {
        f32::from_bits(sign)
    } else if b & 0x007f_ffff != 0 {
        0.0
    } else {
        f32::from_bits(sign | 0x7f7f_ffff)
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
    }

    /// The Q register as the program sees it now: a division that has finished replaces the old value.
    #[inline(always)]
    pub fn q_now(&self) -> f32 {
        if self.cycle >= self.q_ready {
            self.q_pend
        } else {
            self.q
        }
    }

    #[inline(always)]
    fn settle_q(&mut self) {
        if self.cycle >= self.q_ready {
            self.q = self.q_pend;
        }
    }

    /// Write the lanes of `r` selected by `mask` (bit n = lane n) to vf[fd]; update MAC flags if `flags`.
    #[inline(always)]
    pub fn commit(&mut self, fd: usize, mask: u8, r: [f32; 4], flags: bool) {
        let v = fz4(r);
        if flags {
            let mut mac = 0u16;
            for l in 0..4 {
                if mask & (1 << l) == 0 {
                    continue;
                }
                let raw = r[l];
                let sh = 3 - l; // x -> bit 3 .. w -> bit 0
                if v[l] == 0.0 {
                    mac |= 1 << sh;
                }
                if raw.is_sign_negative() && raw != 0.0 {
                    mac |= 0x10 << sh;
                }
                if raw.is_infinite() {
                    mac |= 0x100 << sh;
                }
            }
            self.mac = mac;
        }
        if fd != 0 {
            if mask == 15 {
                self.vf[fd] = v;
            } else {
                for l in 0..4 {
                    if mask & (1 << l) != 0 {
                        self.vf[fd][l] = v[l];
                    }
                }
            }
        }
    }

    /// `commit_acc` for code whose MAC flags are never read.
    #[inline(always)]
    pub fn commit_acc_nf(&mut self, mask: u8, r: [f32; 4]) {
        let v = fz4(r);
        if mask == 15 {
            self.acc = v;
        } else {
            for l in 0..4 {
                if mask & (1 << l) != 0 {
                    self.acc[l] = v[l];
                }
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
        self.settle_q();
        let r = if b == 0.0 { f32::MAX.copysign(a) * if b.is_sign_negative() { -1.0 } else { 1.0 } } else { fz(a / b) };
        self.q_pend = if a == 0.0 && b == 0.0 { 0.0 } else { r };
        self.q_ready = self.cycle + 7;
    }

    #[inline(always)]
    pub fn sqrt(&mut self, b: f32) {
        self.settle_q();
        self.q_pend = fz(b.abs().sqrt());
        self.q_ready = self.cycle + 7;
    }

    #[inline(always)]
    pub fn rsqrt(&mut self, a: f32, b: f32) {
        self.settle_q();
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
