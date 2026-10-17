//! SPU2-style voice mixer: 2 cores x 24 voices playing SPU-ADPCM from a 2 MB sample memory.

use crate::adpcm::{decode_block, State};

pub const RAM_SIZE: usize = 2 << 20;
const VOICES: usize = 24;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone)]
struct Voice {
    phase: Phase,
    ssa: u32,
    lsa: u32,
    lsa_set: bool,
    addr: u32,
    pitch: u32,
    vol_l: i32,
    vol_r: i32,
    adsr1: u32,
    adsr2: u32,
    env: i32,
    env_counter: i32,
    block: [i16; 28],
    pos: usize,
    frac: u32,
    hist: [f32; 4],
    state: State,
    ended: bool,
}

impl Voice {
    fn new() -> Voice {
        Voice {
            phase: Phase::Off,
            ssa: 0,
            lsa: 0,
            lsa_set: false,
            addr: 0,
            pitch: 0,
            vol_l: 0,
            vol_r: 0,
            adsr1: 0,
            adsr2: 0,
            env: 0,
            env_counter: 0,
            block: [0; 28],
            pos: 28,
            frac: 0,
            hist: [0.0; 4],
            state: State::default(),
            ended: false,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Core {
    mvol_l: i32,
    mvol_r: i32,
    noise: u32,
    pmon: u32,
}

pub struct Spu {
    ram: Vec<u8>,
    cores: [Core; 2],
    voices: Vec<Voice>, // core * VOICES + voice
}

/// 15-bit signed volume register -> -1..1 (sweep modes are treated as their static level).
fn vol(x: u32) -> i32 {
    if x & 0x8000 != 0 {
        return 0x3fff;
    }
    (((x & 0x7fff) << 1) as u16 as i16 >> 1) as i32
}

impl Spu {
    pub fn new() -> Spu {
        Spu { ram: vec![0; RAM_SIZE], cores: [Core { mvol_l: 0x3fff, mvol_r: 0x3fff, ..Default::default() }; 2], voices: vec![Voice::new(); 2 * VOICES] }
    }

    pub fn write_ram(&mut self, addr: usize, data: &[u8]) {
        if addr < self.ram.len() {
            let n = data.len().min(self.ram.len() - addr);
            self.ram[addr..addr + n].copy_from_slice(&data[..n]);
        }
    }

    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|v| v.phase != Phase::Off).count()
    }

    fn voice_mut(&mut self, entry: u32) -> Option<&mut Voice> {
        let core = (entry & 1) as usize;
        let v = ((entry & 0xff) >> 1) as usize;
        if v < VOICES {
            self.voices.get_mut(core * VOICES + v)
        } else {
            None
        }
    }

    pub fn set_param(&mut self, entry: u32, value: u32) {
        let core = (entry & 1) as usize;
        let ty = (entry >> 8) & 0x3f;
        if entry & 0x80 != 0 {
            match ty {
                0x09 => self.cores[core].mvol_l = vol(value),
                0x0a => self.cores[core].mvol_r = vol(value),
                _ => {}
            }
            return;
        }
        if let Some(v) = self.voice_mut(entry) {
            match ty {
                0 => v.vol_l = vol(value),
                1 => v.vol_r = vol(value),
                2 => v.pitch = value & 0x3fff,
                3 => v.adsr1 = value & 0xffff,
                4 => v.adsr2 = value & 0xffff,
                _ => {}
            }
        }
    }

    pub fn get_param(&self, entry: u32) -> u32 {
        let core = (entry & 1) as usize;
        if entry & 0x80 != 0 {
            return 0;
        }
        let v = ((entry & 0xff) >> 1) as usize;
        match (entry >> 8) & 0x3f {
            5 if v < VOICES => self.voices[core * VOICES + v].env as u32,
            _ => 0,
        }
    }

    pub fn set_switch(&mut self, entry: u32, value: u32) {
        let core = (entry & 1) as usize;
        match (entry >> 8) & 0xff {
            0x13 => self.cores[core].pmon = value,
            0x14 => self.cores[core].noise = value,
            0x15 => {
                for i in 0..VOICES {
                    if value >> i & 1 != 0 {
                        self.key_on(core * VOICES + i);
                    }
                }
            }
            0x16 => {
                for i in 0..VOICES {
                    if value >> i & 1 != 0 {
                        let v = &mut self.voices[core * VOICES + i];
                        if v.phase != Phase::Off {
                            v.phase = Phase::Release;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_switch(&self, entry: u32) -> u32 {
        let core = (entry & 1) as usize;
        if (entry >> 8) & 0xff == 0x17 {
            let mut m = 0;
            for i in 0..VOICES {
                if self.voices[core * VOICES + i].ended {
                    m |= 1 << i;
                }
            }
            return m;
        }
        0
    }

    pub fn set_addr(&mut self, entry: u32, value: u32) {
        // SD_VA_* entries are 0x2040 / 0x2240 / ...: the 0x40 bit is part of the constant, voice = (entry & 0x3f) >> 1
        let ty = (entry >> 8) & 0xff;
        let core = (entry & 1) as usize;
        let v = ((entry & 0x3f) >> 1) as usize;
        if v >= VOICES {
            return;
        }
        let voice = &mut self.voices[core * VOICES + v];
        match ty {
            0x20 => voice.ssa = value & (RAM_SIZE as u32 - 1),
            0x22 => {
                voice.lsa = value & (RAM_SIZE as u32 - 1);
                voice.lsa_set = true;
            }
            _ => {}
        }
    }

    pub fn set_core_attr(&mut self, _entry: u32, _value: u32) {}

    fn key_on(&mut self, idx: usize) {
        let v = &mut self.voices[idx];
        if std::env::var_os("TS_SPULOG").is_some() {
            eprintln!("[spu] key on voice {idx}: ssa={:#x} pitch={:#x} vol=({},{}) adsr={:#06x},{:#06x}", v.ssa, v.pitch, v.vol_l, v.vol_r, v.adsr1, v.adsr2);
        }
        v.phase = Phase::Attack;
        v.addr = v.ssa;
        v.env = 0;
        v.env_counter = 0;
        v.pos = 28;
        v.frac = 0;
        v.hist = [0.0; 4];
        v.state = State::default();
        v.lsa_set = false;
        v.lsa = v.ssa;
        v.ended = false;
    }

    /// Decode the next block of voice `idx`, handling loop / end flags.
    fn next_block(ram: &[u8], v: &mut Voice) {
        let a = v.addr as usize & (RAM_SIZE - 1) & !15;
        let blk = &ram[a..a + 16];
        let flags = blk[1];
        if flags & 4 != 0 {
            v.lsa = a as u32;
            v.lsa_set = true;
        }
        decode_block(blk, &mut v.state, &mut v.block);
        v.pos = 0;
        if flags & 1 != 0 {
            if flags & 2 != 0 {
                v.addr = v.lsa;
            } else {
                // plays out this block, then stops
                v.ended = true;
                v.addr = a as u32 + 16;
            }
        } else {
            v.addr = (a + 16) as u32;
        }
    }

    fn step_env(v: &mut Voice) {
        let (shift, step, dec, exp, target, next) = match v.phase {
            Phase::Attack => ((v.adsr1 >> 10) & 31, 7 - ((v.adsr1 >> 8) & 3) as i32, false, v.adsr1 & 0x8000 != 0, 0x7fff, Phase::Decay),
            Phase::Decay => ((v.adsr1 >> 4) & 15, -8, true, true, ((v.adsr1 & 15) as i32 + 1) * 0x800, Phase::Sustain),
            Phase::Sustain => {
                let dec = v.adsr2 & 0x4000 != 0;
                let st = ((v.adsr2 >> 6) & 3) as i32;
                ((v.adsr2 >> 8) & 31, if dec { -8 + st } else { 7 - st }, dec, v.adsr2 & 0x8000 != 0, if dec { 0 } else { 0x7fff }, Phase::Sustain)
            }
            Phase::Release => ((v.adsr2) & 31, -8, true, v.adsr2 & 0x20 != 0, 0, Phase::Off),
            Phase::Off => return,
        };
        let mut inc = 0x8000i32 >> shift.saturating_sub(11);
        if inc == 0 {
            inc = 1;
        }
        let mut delta = step << 11u32.saturating_sub(shift);
        if exp && !dec && v.env > 0x6000 {
            inc = (inc / 4).max(1);
        }
        if exp && dec {
            delta = delta * v.env / 0x8000;
        }
        v.env_counter += inc;
        if v.env_counter & 0x8000 == 0 {
            return;
        }
        v.env_counter = 0;
        v.env = (v.env + delta).clamp(0, 0x7fff);
        let reached = if dec { v.env <= target } else { v.env >= target };
        if reached && v.phase != Phase::Sustain {
            v.phase = next;
            if next == Phase::Off {
                v.env = 0;
            }
        }
    }

    /// Add this SPU's voices into `out` (interleaved stereo at `rate` Hz).
    pub fn mix(&mut self, out: &mut [f32], rate: u32) {
        let ratio = 48000.0 / rate as f32;
        let ram = &self.ram;
        for ci in 0..2 {
            let core = self.cores[ci];
            let (ml, mr) = (core.mvol_l as f32 / 16384.0, core.mvol_r as f32 / 16384.0);
            for vi in 0..VOICES {
                let v = &mut self.voices[ci * VOICES + vi];
                if v.phase == Phase::Off {
                    continue;
                }
                let vl = v.vol_l as f32 / 16384.0 * ml;
                let vr = v.vol_r as f32 / 16384.0 * mr;
                let step = v.pitch as f32 / 4096.0 * ratio;
                let stepfx = (step * 4096.0) as u32;
                for frame in out.chunks_exact_mut(2) {
                    if v.phase == Phase::Off {
                        break;
                    }
                    // advance
                    v.frac += stepfx;
                    while v.frac >= 4096 {
                        v.frac -= 4096;
                        if v.pos >= 28 {
                            if v.ended {
                                v.phase = Phase::Off;
                                break;
                            }
                            Self::next_block(ram, v);
                        }
                        v.hist = [v.hist[1], v.hist[2], v.hist[3], v.block[v.pos] as f32];
                        v.pos += 1;
                    }
                    if v.phase == Phase::Off {
                        break;
                    }
                    // Catmull-Rom between hist[1] and hist[2]
                    let t = v.frac as f32 / 4096.0;
                    let [p0, p1, p2, p3] = v.hist;
                    let s = p1 + 0.5 * t * (p2 - p0 + t * (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3 + t * (3.0 * (p1 - p2) + p3 - p0)));
                    let e = v.env as f32 / 32768.0;
                    let s = s / 32768.0 * e;
                    frame[0] += s * vl;
                    frame[1] += s * vr;
                    Self::step_env(v);
                }
            }
        }
    }
}
