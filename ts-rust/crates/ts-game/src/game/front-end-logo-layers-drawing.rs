//! The drawing of the layers of the front end's logo.

use crate::*;

/// Draw the logo of the front end: each of the 20 layers (records of 60 bytes from `logolayers`; the kind at +0, the times at +4 and +8, the values at the start (+12 to +40) and at
/// the end (+20 to +56)) whose time (`frontfxtime`, less 3 for a layer lasting past 3 when the time is early) has come is drawn by `frontfxRenderLayer` with the values mixed by
/// how far through it is, the last (brightness) value toned down by the constants at 0x3ac914 and 0x3ac918 (for kind 2) unless `mode` is 1. With mode 1 two swaying layers (by the
/// game's time) and the dimmed and black layers follow, otherwise only the dimmed and black layers (@ 0x22ae40).
pub fn frontfxLogoGfx(mode: i32) {
    let layers = g::logolayers;
    let mut layer = layers;
    let mut offset = 0;
    let three = 3.0_f32;
    loop {
        let mut end_time = lwc1(layer + 8);
        let mut start_time = lwc1(layers + offset + 4);
        let time;
        let span;
        if !(three <= end_time) {
            time = lwc1(g::frontfxtime);
            span = end_time - start_time;
        } else {
            let shifted = end_time - three;
            time = lwc1(g::frontfxtime);
            if !(time < shifted) {
                span = end_time - start_time;
            } else {
                end_time = shifted;
                start_time -= three;
                span = end_time - start_time;
            }
        }
        let progress = (time - start_time) / span;
        if 0.0_f32 <= progress {
            let base = g::logolayers;
            if progress < 1.0_f32 {
                let at = base + offset;
                let a = lwc1(at + 12);
                let b = lwc1(at + 16);
                let c = lwc1(at + 28);
                let d = lwc1(at + 32);
                let e = lwc1(at + 36);
                let f = lwc1(at + 40);
                let x = a + (lwc1(at + 20) - a) * progress;
                let y = b + (lwc1(at + 24) - b) * progress;
                let red = c + (lwc1(at + 44) - c) * progress;
                let green = d + (lwc1(at + 48) - d) * progress;
                let blue = e + (lwc1(at + 52) - e) * progress;
                let mut alpha = f + (lwc1(at + 56) - f) * progress;
                if mode != 1 {
                    let tone = lwc1(0x3ac914);
                    if lw(offset + base) != 2 {
                        alpha *= tone;
                    } else {
                        alpha *= lwc1(0x3ac918);
                    }
                }
                frontfxRenderLayer(lw(layer), x * lwc1(g::globalscalex), y * lwc1(g::globalscaley), red, green, blue, alpha);
            }
        }
        layer += 60;
        offset += 60;
        if layer >= layers + 1200 {
            break;
        }
    }
    if mode == 1 {
        let rate_a = lwc1(0x3ac928);
        let rate_b = lwc1(0x3ac92c);
        let table = g::g_SinTable;
        let radians = ld(0x3a18a8);
        let scale = ld(0x3a18b0);
        let half = 0x3fe0000000000000_i64;
        let quarter = 0.5_f32;
        let sway = lw(g::gameframe) as f32 * rate_a * rate_b;
        let offset_c = lwc1(0x3ac930);
        let screen = ld(0x3a18b8);
        let three_quarters = 0.75_f32;
        let doubled = sway + sway;
        let index = dptoli(dpadd(dpmul(dpmul(fptodp(doubled), radians), scale), half));
        let sway = lw(g::gameframe) as f32 * rate_a * rate_b;
        let bright_a = lwc1(((index & 2047) << 2) + table) * quarter + offset_c;
        let doubled = sway + sway;
        let index = dptoli(dpadd(dpmul(dpadd(dpmul(fptodp(doubled), radians), radians), scale), half));
        let bright_b = lwc1(((index & 2047) << 2) + table) * rate_b + offset_c;
        let x = dptofp(dpmul(fptodp(lwc1(g::globalscalex)), screen));
        let y = dptofp(dpmul(fptodp(lwc1(g::globalscaley)), screen));
        frontfxRenderLayer(0, x, y, 0.25_f32, three_quarters, quarter, bright_a);
        let x = dptofp(dpmul(fptodp(lwc1(g::globalscalex)), screen));
        let y = dptofp(dpmul(fptodp(lwc1(g::globalscaley)), screen));
        frontfxRenderLayer(0, x, y, 0.0_f32, quarter, three_quarters, bright_b);
        dlSetBlend(2);
        frontfxRenderLayer(0, lwc1(g::globalscalex), lwc1(g::globalscaley), three_quarters, three_quarters, three_quarters, lwc1(0x3ac934));
        dlSetBlend(1);
        frontfxRenderLayer(-1, lwc1(g::globalscalex), lwc1(g::globalscaley), 0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32);
        return;
    }
    let bright = lwc1(0x3ac924);
    dlSetBlend(2);
    frontfxRenderLayer(0, lwc1(g::globalscalex), lwc1(g::globalscaley), 0.5_f32, 0.5_f32, 0.5_f32, bright);
    dlSetBlend(1);
    frontfxRenderLayer(-1, lwc1(g::globalscalex), lwc1(g::globalscaley), 0.0_f32, 0.0_f32, 0.0_f32, bright);
}
