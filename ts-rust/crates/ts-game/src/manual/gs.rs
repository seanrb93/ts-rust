//! Replacements for the few game functions that drove graphics hardware registers directly.

use crate::*;

/// Upload an image (described by a libgs load-image descriptor) to texture memory.
/// The original kicked DMA channel 2 and polled it; here the pixel data is handed to the renderer's upload hook.
pub fn sjeGsExecLoadImage(desc: i32, data: i32) {
    let bitbltbuf = ld(desc + 16) as u64;
    let trxpos = ld(desc + 32) as u64;
    let trxreg = ld(desc + 48) as u64;
    ts_sdk::image_upload(&ts_sdk::ImageUpload {
        dbp: ((bitbltbuf >> 32) & 0x3fff) as u32,
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
/// original's asynchronous copy effectively was).
pub fn sjeStoreImage(_a0: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, _a5: i32, _a6: i32, _a7: i32) {
    if let Some(d) = ts_sdk::depth_request() {
        let dst = lw(g::zbuffer);
        for (i, z) in d.iter().enumerate() {
            sh(dst + 2 * i as i32, *z as i32);
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
