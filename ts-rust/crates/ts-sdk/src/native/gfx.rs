//! Graphics SDK helpers (`sceGs*`, `sceDma*`) and the texture-upload hook.
//!
//! The game manages GS video memory by *block address*: it uploads an image to a block address and later draws with a
//! texture-register value that names the same address.  The port keeps that contract as an asset-binding table
//! (`ImageUpload` -> renderer texture); no GS pixel pipeline is modelled.
#![allow(non_snake_case)]

use std::sync::Mutex;
use ts_rt::*;

/// A pixel-image transfer into (virtual) GS memory.
#[derive(Debug, Clone, Copy)]
pub struct ImageUpload {
    /// destination block address (units of 64 words)
    pub dbp: u32,
    /// destination buffer width (units of 64 pixels)
    pub dbw: u32,
    /// pixel storage mode (0 = 32-bit, 2 = 16-bit, 19 = 8-bit indexed, 20 = 4-bit indexed, ...)
    pub psm: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// address of the pixel data in game memory
    pub data: i32,
}

pub type UploadFn = fn(&ImageUpload);
static UPLOAD: Mutex<Option<UploadFn>> = Mutex::new(None);

pub fn set_upload_hook(f: UploadFn) {
    *UPLOAD.lock().unwrap() = Some(f);
}

pub fn image_upload(u: &ImageUpload) {
    let f = *UPLOAD.lock().unwrap();
    if let Some(f) = f {
        f(u);
    }
}

/// DMA channel register block base addresses, indexed by channel number.
const DMA_CHANNELS: [i32; 10] = [
    0x1000_8000, 0x1000_9000, 0x1000_a000, 0x1000_b000, 0x1000_b400, 0x1000_c000, 0x1000_c400, 0x1000_c800,
    0x1000_d000, 0x1000_d400,
];

/// `sceDmaGetChan(ch)`: the channel's register block (the game pokes a few of its fields directly).
pub fn sceDmaGetChan(args: &[i64]) -> i32 {
    DMA_CHANNELS.get(args[0] as usize).copied().unwrap_or(0)
}

/// `sceGsSetDefLoadImage(lp, dbp, dbw, dpsm, x, y, w, h)`: fill the image-load descriptor (GIF tag + A+D registers)
/// exactly as libgs does; the game later patches its `giftag1` with the quad-word count.
pub fn sceGsSetDefLoadImage(args: &[i64]) -> i32 {
    let lp = args[0] as i32;
    let (dbp, dbw, dpsm) = (args[1] & 0x3fff, args[2] & 0x3f, args[3] & 0x3f);
    let (x, y, w, h) = (args[4] & 0x7ff, args[5] & 0x7ff, args[6] & 0xfff, args[7] & 0xfff);
    sd(lp, 4 | (1 << 15) | (0xe << 60)); // giftag0: NLOOP=4, EOP, REGS=A+D (NREG 1)
    sd(lp + 8, 0xe);
    sd(lp + 16, dbp << 32 | dbw << 48 | dpsm << 56); // BITBLTBUF
    sd(lp + 24, 0x50);
    sd(lp + 32, x << 32 | y << 48); // TRXPOS
    sd(lp + 40, 0x51);
    sd(lp + 48, w | h << 32); // TRXREG
    sd(lp + 56, 0x52);
    sd(lp + 64, 0); // TRXDIR host -> local
    sd(lp + 72, 0x53);
    sd(lp + 80, (1 << 15) | (2 << 58)); // giftag1: EOP, FLG=IMAGE (NLOOP patched by the caller)
    sd(lp + 88, 0);
    0
}

pub fn sceGsSyncPath(_args: &[i64]) -> i32 {
    0
}
pub fn sceGsResetPath(_args: &[i64]) -> i32 {
    0
}
pub fn sceGsResetGraph(_args: &[i64]) -> i32 {
    0
}
pub fn sceDmaReset(_args: &[i64]) -> i32 {
    0
}
pub fn sceDmaSend(_args: &[i64]) -> i32 {
    0
}
pub fn sceDmaSendN(_args: &[i64]) -> i32 {
    0
}
pub fn sceDmaSync(_args: &[i64]) -> i32 {
    0
}
