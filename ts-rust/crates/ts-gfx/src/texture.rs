//! Decoding of images in the virtual texture memory to RGBA8.

use crate::scene::{psm, TexKey};
use crate::vram::{Image, Vram};

/// CSM1 palette entry order for 256-entry palettes (bits 3 and 4 of the index are exchanged).
fn csm1_perm(i: usize) -> usize {
    (i & 0xe7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1)
}

fn clut_entry(clut: &Image, idx: usize, cpsm: u32) -> [u8; 4] {
    match cpsm {
        psm::CT16 | psm::CT16S => {
            let o = idx * 2;
            if o + 2 > clut.data.len() {
                return [0, 0, 0, 0];
            }
            let v = u16::from_le_bytes([clut.data[o], clut.data[o + 1]]) as u32;
            [((v & 31) << 3) as u8, (((v >> 5) & 31) << 3) as u8, (((v >> 10) & 31) << 3) as u8, if v & 0x8000 != 0 { 128 } else { 0 }]
        }
        _ => {
            let o = idx * 4;
            if o + 4 > clut.data.len() {
                return [0, 0, 0, 0];
            }
            let a = clut.data[o + 3] as u32;
            [clut.data[o], clut.data[o + 1], clut.data[o + 2], a as u8]
        }
    }
}

/// Size and content signature of the texture `key` without decoding it (cheap; changes when its pixels or palette
/// are re-uploaded).
pub fn signature(key: &TexKey, vram: &Vram) -> Option<(u32, u32, u64)> {
    let w = 1u32 << key.tw.min(11);
    let h = 1u32 << key.th.min(11);
    let img = vram.find(key.tbp0)?;
    let mut sig = img.generation;
    if matches!(key.psm, psm::T8 | psm::T4) {
        if let Some(c) = vram.find(key.cbp) {
            sig = sig.wrapping_mul(1_000_003).wrapping_add(c.generation);
        }
    }
    if let Some(hd) = crate::hd::for_texture(key.tbp0) {
        sig = sig.wrapping_mul(1_000_003).wrapping_add(hd.id << 32);
    }
    Some((w, h, sig))
}

/// Decode the texture named by `key`.  Returns (width, height, RGBA8, content signature).
pub fn decode(key: &TexKey, vram: &Vram) -> Option<(u32, u32, Vec<u8>, u64)> {
    let w = 1u32 << key.tw.min(11);
    let h = 1u32 << key.th.min(11);
    let img = vram.find(key.tbp0)?;
    let mut sig = img.generation;
    // a picture of a mod in place of the texture (its own size: the caller addresses it by the size of the texture)
    if let Some(entry) = crate::hd::for_texture(key.tbp0) {
        if let Some(hd) = entry.image() {
            if let Some(c) = matches!(key.psm, psm::T8 | psm::T4).then(|| vram.find(key.cbp)).flatten() {
                sig = sig.wrapping_mul(1_000_003).wrapping_add(c.generation);
            }
            sig = sig.wrapping_mul(1_000_003).wrapping_add(entry.id << 32);
            if std::env::var_os("TS_HDLOG").is_some() {
                eprintln!("[hd] decode {:#x}: {}x{} for a texture of {w}x{h}", key.tbp0, hd.w, hd.h);
            }
            return Some((hd.w, hd.h, hd.rgba.clone(), sig));
        }
    }
    let stride = img.w.max(1) as usize;
    let mut out = vec![0u8; (w * h * 4) as usize];
    let indexed = matches!(key.psm, psm::T8 | psm::T4);
    let clut = if indexed { vram.find(key.cbp) } else { None };
    if let Some(c) = clut {
        sig = sig.wrapping_mul(1_000_003).wrapping_add(c.generation);
    }
    let bits = match key.psm {
        psm::CT32 => 32,
        psm::CT24 => 24,
        psm::CT16 | psm::CT16S => 16,
        psm::T8 => 8,
        psm::T4 => 4,
        _ => 32,
    };
    for y in 0..h as usize {
        let sy = y.min(img.h.max(1) as usize - 1);
        for x in 0..w as usize {
            let sx = x.min(stride - 1);
            let pix = sy * stride + sx;
            let dst = (y * w as usize + x) * 4;
            let px: [u8; 4] = match key.psm {
                psm::CT32 => {
                    let o = pix * 4;
                    if o + 4 > img.data.len() {
                        [0; 4]
                    } else {
                        let a = img.data[o + 3] as u32;
                        [img.data[o], img.data[o + 1], img.data[o + 2], a as u8]
                    }
                }
                psm::CT24 => {
                    let o = pix * 3;
                    if o + 3 > img.data.len() { [0; 4] } else { [img.data[o], img.data[o + 1], img.data[o + 2], 128] }
                }
                psm::CT16 | psm::CT16S => {
                    let o = pix * 2;
                    if o + 2 > img.data.len() {
                        [0; 4]
                    } else {
                        let v = u16::from_le_bytes([img.data[o], img.data[o + 1]]) as u32;
                        [((v & 31) << 3) as u8, (((v >> 5) & 31) << 3) as u8, (((v >> 10) & 31) << 3) as u8, 128]
                    }
                }
                psm::T8 => {
                    let idx = *img.data.get(pix).unwrap_or(&0) as usize;
                    match clut {
                        Some(c) => clut_entry(c, csm1_perm(idx), key.cpsm),
                        None => [idx as u8, idx as u8, idx as u8, 128],
                    }
                }
                psm::T4 => {
                    let b = *img.data.get(pix / 2).unwrap_or(&0);
                    let idx = if pix % 2 == 0 { b & 15 } else { b >> 4 } as usize;
                    match clut {
                        Some(c) => clut_entry(c, key.csa as usize * 16 + idx, key.cpsm),
                        None => [idx as u8 * 17, idx as u8 * 17, idx as u8 * 17, 128],
                    }
                }
                _ => [255, 0, 255, 128],
            };
            out[dst..dst + 4].copy_from_slice(&px);
        }
    }
    let _ = bits;
    // Fully transparent texels carry arbitrary colours; bilinear filtering at a higher resolution blends them into the edges
    // of sprites and glyphs as coloured fringes.  Give them the colour of an opaque neighbour instead.
    if out.chunks_exact(4).any(|p| p[3] == 0) {
        let (wu, hu) = (w as usize, h as usize);
        let src = out.clone();
        for y in 0..hu {
            for x in 0..wu {
                let i = (y * wu + x) * 4;
                if src[i + 3] != 0 {
                    continue;
                }
                let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= wu as i32 || ny >= hu as i32 {
                            continue;
                        }
                        let j = (ny as usize * wu + nx as usize) * 4;
                        if src[j + 3] != 0 {
                            r += src[j] as u32;
                            g += src[j + 1] as u32;
                            b += src[j + 2] as u32;
                            n += 1;
                        }
                    }
                }
                if n > 0 {
                    out[i] = (r / n) as u8;
                    out[i + 1] = (g / n) as u8;
                    out[i + 2] = (b / n) as u8;
                }
            }
        }
    }
    static ALPHALOG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ALPHALOG.get_or_init(|| std::env::var_os("TS_ALPHALOG").is_some()) {
        let hh = h;
        let mut hist = [0u32; 4]; // <128, ==128, 129..254, 255
        for px in out.chunks_exact(4) {
            hist[match px[3] { 0..=127 => 0, 128 => 1, 255 => 3, _ => 2 }] += 1;
        }
        eprintln!("[tex] tbp0={:#x} psm={} {}x{} alpha: <128:{} ==128:{} 129-254:{} 255:{}", key.tbp0, key.psm, w, hh, hist[0], hist[1], hist[2], hist[3]);
    }
    Some((w, h, out, sig))
}
