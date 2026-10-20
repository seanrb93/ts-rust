//! The tick of the front end's effects (the logo's slide in and the flare that goes with it).

use crate::*;

/// The sine of the table for the angle `x` (as a double), taken modulo 2048 after the table index is rounded.
fn sine_of(x: i64) -> f32 {
    let index = dptoli(x);
    lwc1(((index & 2047) << 2) + g::g_SinTable)
}

/// Tick the front end's effects for the page kind `mode`: the menu note effect, the background and (for the goodies page, kind 4) the goodies, the effects' time (it wraps at 3),
/// and for the main page (kind 1) the time since the page came up (from 100 when it was not the page before) moves the logo in: the first part scales it up while it flares
/// (the flare's scales and intensities follow sines of the time), then it settles; for the other pages the logo is small at (500, 19) and the flare is off (@ 0x22a610).
pub fn frontfxTick(mode: i32) {
    let previous = lw(g::frontstate);
    if mode != previous {
        if mode == 1 && previous != -1 {
            let scale_y = lwc1(0x3ac8e0);
            swc1(g::fronttime, 1e+02_f32);
            swc1(g::globalscalex, 1.5_f32);
            swc1(g::globalscaley, scale_y);
        }
        if mode == 4 {
            frontfxGoodiesReset();
        }
    }
    sw(g::frontstate, mode);
    frontfxBgTick();
    let mut time = lwc1(g::frontfxtime);
    if mode == 4 {
        frontfxGoodiesTick();
        time = lwc1(g::frontfxtime);
    }
    let rate = lwc1(0x3ac8e4);
    let wrap = 3.0_f32;
    let time = time + rate;
    swc1(g::frontfxtime, time);
    if wrap <= time {
        swc1(g::frontfxtime, time - wrap);
    }
    if mode != 1 {
        let scale_y = lwc1(0x3ac910);
        let flare = g::flareintensity;
        swc1(g::globalscalex, 0.5_f32);
        swc1(g::globalscaley, scale_y);
        swc1(g::fronttime, 1e+02_f32);
        sw(g::flareintensity, 0);
        sw(g::logoposx, 500);
        sw(g::logoposy, 19);
        sw(flare + 20, 0);
        sw(flare + 4, 0);
        sw(flare + 12, 0);
        sw(flare + 16, 0);
        return;
    }
    let first_end = lwc1(0x3ac8e8);
    let page_time = lwc1(g::fronttime) + rate;
    sw(g::logoposx, 320);
    sw(g::logoposy, 89);
    swc1(g::fronttime, page_time);
    let half = 0x3fe0000000000000_i64;
    if !(page_time < first_end) {
        let second_start = lwc1(0x3ac8f4);
        let scale_x = lwc1(g::globalscalex);
        if !(page_time < second_start) {
            if page_time < lwc1(0x3ac904) {
                let past = page_time - second_start;
                let radians = ld(0x3a1890);
                let factor = ld(0x3a1898);
                let fade = 1.0_f32 - past * 0.5_f32;
                let angle = fptodp(past);
                let value = sine_of(dpadd(dpmul(dpadd(dpmul(dpmul(angle, radians), half), ld(0x3a18a0)), factor), half));
                swc1(g::flarescalarx, value * lwc1(0x3ac908));
                let angle = fptodp(past);
                let second = sine_of(dpadd(dpmul(dpadd(dpmul(dpmul(angle, radians), half), radians), factor), half));
                let tall = lwc1(0x3ac90c);
                let flare = g::flareintensity;
                swc1(flare + 20, fade);
                swc1(g::flareintensity, fade);
                swc1(flare + 4, fade);
                swc1(flare + 12, fade);
                swc1(g::flarescalary, second * tall);
                swc1(flare + 16, fade);
            }
        } else {
            let past = page_time - first_end;
            let one_less = 1.0_f32 - past;
            let radians = ld(0x3a1878);
            let factor = ld(0x3a1880);
            swc1(g::globalscaley, one_less + lwc1(0x3ac8f8));
            swc1(g::globalscalex, scale_x + one_less * wrap * rate);
            let angle = fptodp(past);
            let value = sine_of(dpadd(dpmul(dpmul(dpmul(angle, radians), half), factor), half));
            swc1(g::flarescalarx, value * lwc1(0x3ac8fc));
            let angle = fptodp(past);
            let value = sine_of(dpadd(dpmul(dpadd(dpmul(dpmul(angle, radians), half), radians), factor), half));
            let tall = lwc1(0x3ac900);
            swc1(g::flarescalary, value * tall + tall);
            let angle = fptodp(past);
            let value = sine_of(dpadd(dpmul(dpadd(dpmul(dpmul(angle, radians), half), ld(0x3a1888)), factor), half));
            swc1(g::flareintensity + 8, value);
        }
    } else {
        let progress = page_time / first_end;
        let radians = ld(0x3a1860);
        let shift = ld(0x3a1868);
        let factor = ld(0x3a1870);
        swc1(g::globalscaley, lwc1(0x3ac8ec));
        sw(g::globalscalex, 0);
        let one = 1.0_f32;
        let angle = fptodp(progress);
        let value = sine_of(dpadd(dpmul(dpadd(dpmul(dpmul(angle, radians), half), shift), factor), half));
        let flare = g::flareintensity;
        swc1(g::flarescalary, lwc1(0x3ac8f0));
        swc1(g::flarescalarx, value);
        swc1(g::flareintensity, one);
        swc1(flare + 4, one);
        swc1(flare + 12, one);
        swc1(flare + 16, one);
        swc1(flare + 20, one);
        let angle = fptodp(progress);
        let value = sine_of(dpadd(dpmul(dpadd(dpmul(dpmul(angle, radians), half), shift), factor), half));
        swc1(flare + 8, value + one);
    }
}
