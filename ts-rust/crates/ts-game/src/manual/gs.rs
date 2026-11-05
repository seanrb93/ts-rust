//! Replacements for the few game functions that drove graphics hardware registers directly.

use crate::*;

/// Upload an image (described by a libgs load-image descriptor) to texture memory.
/// The original kicked DMA channel 2 and polled it; here the pixel data is handed to the renderer's upload hook.
pub fn sjeGsExecLoadImage(desc: i32, data: i32) {
    let bitbltbuf = ld(desc + 16) as u64;
    let trxpos = ld(desc + 32) as u64;
    let trxreg = ld(desc + 48) as u64;
    ts_sdk::image_upload(&ts_sdk::ImageUpload {
        dbp: ts_rt::gswide::bitbltbuf_dbp(bitbltbuf),
        dbw: ((bitbltbuf >> 48) & 0x3f) as u32,
        psm: ((bitbltbuf >> 56) & 0x3f) as u32,
        x: ((trxpos >> 32) & 0x7ff) as u32,
        y: ((trxpos >> 48) & 0x7ff) as u32,
        w: (trxreg & 0xfff) as u32,
        h: ((trxreg >> 32) & 0xfff) as u32,
        data,
    });
}

/// Copy the depth buffer back to main memory (the game's light-glow occlusion probes).  Only the 640x224 Z16 copy
/// into `zbuffer` is ever requested; it comes from the renderer's CPU depth reconstruction (one frame old, as the
/// original's asynchronous copy effectively was).  Only `zbtestDoTest`, straight after this, reads the copy, and only at the points
/// of the test slot, so only those are worked out.  (The shadow check compares all the memory with the original's: it gets the whole picture.)
pub fn sjeStoreImage(_a0: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, _a5: i32, _a6: i32, _a7: i32) {
    if cfg!(feature = "shadow") {
        ts_sdk::depth_request(|scene| {
            let dst = lw(g::zbuffer);
            for (i, z) in scene.rasterize().iter().enumerate() {
                sh(dst + 2 * i as i32, *z as i32);
            }
        });
        return;
    }
    let slot = lw(0x3b0174) << 2;
    let (first, points) = (lw(slot + g::zbval), lw(slot + 0x1fea0a0).max(0) * 9);
    let mut pixels = Vec::with_capacity(points as usize);
    for i in 0..points {
        let point = first + i * 24;
        let at = lw(point + 12);
        if lw(point) >= 0 && (0..(ts_rt::depth::W * ts_rt::depth::H * 2) as i32).contains(&at) {
            pixels.push(at as u32 / 2);
        }
    }
    pixels.sort_unstable();
    pixels.dedup();
    let mut values = Vec::new();
    if ts_sdk::depth_request(|scene| scene.probe(&pixels, &mut values)).is_some() {
        let dst = lw(g::zbuffer);
        for (p, z) in pixels.iter().zip(&values) {
            sh(dst + *p as i32 * 2, *z as i32);
        }
    }
}

/// EE timer setup; the port derives time from the host clock instead.
pub fn timeMake() {}

/// Per-frame reset of the image-processing effect flags (the original zeroed them every tick; gameplay code then
/// raises them).  Debug aid: `TS_IP=fisheye|blur|invert|defocus|viewzb` forces an effect for player 0.
pub fn ipTick() {
    for arr in [g::ipfisheye, g::ipviewzb, g::ipblur, g::ipinvert, g::ipdefocus] {
        for i in 0..4 {
            sw(arr + 4 * i, 0);
        }
    }
    if let Ok(v) = std::env::var("TS_IP") {
        let arr = match v.as_str() {
            "fisheye" => g::ipfisheye,
            "blur" => g::ipblur,
            "invert" => g::ipinvert,
            "defocus" => g::ipdefocus,
            "viewzb" => g::ipviewzb,
            _ => return,
        };
        sw(arr, 1);
    }
}
