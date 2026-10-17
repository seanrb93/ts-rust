//! Native audio engine.
//!
//! The original game drove the PS2's sound chip (SPU2) through libsd and streamed music through a custom IOP
//! module.  This crate provides the same *service interface* natively: a 2 MB sample memory holding SPU-ADPCM data,
//! 48 voices with pitch / volume / ADSR envelopes, and a double-buffered ADPCM music streamer.  Everything is mixed
//! in floating point by `render`, which the host calls from its audio callback (or from a test harness).
//!
//! The game-facing entry points (`sd_*`, `stream_*`) are called by the SDK layer (`ts-sdk`) when the decompiled
//! sound code issues its `sceSdRemote` / stream RPC requests.

mod adpcm;
mod music;
mod spu;

use std::sync::Mutex;

pub use music::MusicCmd;

/// IOP RAM staging area (the game copies sample data EE -> IOP -> SPU2).
pub const IOP_RAM_SIZE: usize = 2 << 20;

struct Engine {
    iop: Vec<u8>,
    spu: spu::Spu,
    music: music::Music,
}

static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Engine) -> R) -> R {
    let mut g = ENGINE.lock().unwrap();
    let e = g.get_or_insert_with(|| Engine { iop: vec![0; IOP_RAM_SIZE], spu: spu::Spu::new(), music: music::Music::new() });
    f(e)
}

/// Copy bytes into IOP RAM at `addr` (SIF DMA from the EE).
pub fn iop_write(addr: u32, data: &[u8]) {
    with(|e| {
        let a = addr as usize;
        if a < e.iop.len() {
            let n = data.len().min(e.iop.len() - a);
            e.iop[a..a + n].copy_from_slice(&data[..n]);
        }
    });
}

/// Read bytes back from IOP RAM.
pub fn iop_read(addr: u32, out: &mut [u8]) {
    with(|e| {
        let a = addr as usize;
        if a < e.iop.len() {
            let n = out.len().min(e.iop.len() - a);
            out[..n].copy_from_slice(&e.iop[a..a + n]);
        }
    });
}

/// `sceSdSetParam(entry, value)`
pub fn sd_set_param(entry: u32, value: u32) {
    with(|e| e.spu.set_param(entry, value));
}

/// `sceSdSetSwitch(entry, value)` (key on / key off / noise / pitch-mod masks)
pub fn sd_set_switch(entry: u32, value: u32) {
    with(|e| e.spu.set_switch(entry, value));
}

/// `sceSdSetAddr(entry, value)` (voice start / loop addresses)
pub fn sd_set_addr(entry: u32, value: u32) {
    with(|e| e.spu.set_addr(entry, value));
}

/// `sceSdSetCoreAttr(entry, value)`
pub fn sd_set_core_attr(entry: u32, value: u32) {
    with(|e| e.spu.set_core_attr(entry, value));
}

/// `sceSdGetParam(entry)`
pub fn sd_get_param(entry: u32) -> u32 {
    with(|e| e.spu.get_param(entry))
}

/// `sceSdGetSwitch(entry)`
pub fn sd_get_switch(entry: u32) -> u32 {
    with(|e| e.spu.get_switch(entry))
}

/// `sceSdVoiceTrans`: copy `size` bytes from IOP RAM to sample memory.
pub fn sd_voice_trans(iop_addr: u32, spu_addr: u32, size: u32) {
    with(|e| {
        let (i, s, n) = (iop_addr as usize, spu_addr as usize, size as usize);
        if i < e.iop.len() {
            let n = n.min(e.iop.len() - i);
            let data = e.iop[i..i + n].to_vec();
            e.spu.write_ram(s, &data);
        }
    });
}

/// Execute a command batch located in IOP RAM: `n` records of {u16 func, u16 entry, u32 value}.
/// func 1 = set param, 2 = set switch, 3 = set address.
pub fn sd_proc_batch(iop_addr: u32, n: u32) {
    with(|e| {
        for k in 0..n as usize {
            let a = iop_addr as usize + 8 * k;
            if a + 8 > e.iop.len() {
                break;
            }
            let func = u16::from_le_bytes([e.iop[a], e.iop[a + 1]]);
            let entry = u16::from_le_bytes([e.iop[a + 2], e.iop[a + 3]]) as u32;
            let value = u32::from_le_bytes([e.iop[a + 4], e.iop[a + 5], e.iop[a + 6], e.iop[a + 7]]);
            match func {
                1 => e.spu.set_param(entry, value),
                2 => e.spu.set_switch(entry, value),
                3 => e.spu.set_addr(entry, value),
                _ => {}
            }
        }
    });
}

/// Music stream command (see [`MusicCmd`]); returns the command's reply value.
pub fn stream_command(cmd: MusicCmd) -> i32 {
    with(|e| e.music.command(cmd))
}

/// Mix `out.len() / 2` stereo frames at sample rate `rate` into `out` (interleaved L, R; overwritten).
pub fn render(out: &mut [f32], rate: u32) {
    with(|e| {
        for s in out.iter_mut() {
            *s = 0.0;
        }
        e.spu.mix(out, rate);
        e.music.mix(out, rate);
        for s in out.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }
    });
}

/// Names of the voices currently sounding (diagnostics).
pub fn active_voices() -> usize {
    with(|e| e.spu.active_voices())
}

/// True while a music stream is playing.
pub fn music_playing() -> bool {
    with(|e| e.music.playing())
}
