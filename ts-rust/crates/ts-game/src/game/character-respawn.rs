//! Putting a computer character back into play.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    random_as_float() * 2.3283064e-10_f32
}

/// Reset a computer character `chr` for a new life: the prop it is (at +3020) gets its flags and health back, the state of its mind (the many words from +2692 to +3028 of its
/// record) is cleared, its pad is the one nearest to it (or 1 with no pads), the animation state of its prop is cleared, and by its kind (+8: 0 to 3) it gets reaction and
/// aim times, some at random; finally its head prop is chosen (@ 0x287fc0).
pub fn chrRespawn(chr: i32) {
    let prop = lw(chr + 3020);
    let flags = lw(prop + 16);
    swc1(prop + 520, 1.0_f32);
    let pads = lw(g::levelpads);
    sw(chr + 12, 0);
    sw(prop + 16, (flags | 0x40000031) & -0x30000001);
    sw(chr + 256, 0);
    sw(chr + 2708, 1);
    sw(chr + 28, 0);
    if pads == 0 {
        sw(chr + 680, 1);
    } else {
        let pad = closestpad(prop + 48, lwc1(chr + 12));
        sw(chr + 680, pad);
    }
    let half = 0.5_f32;
    let last = lwc1(g::chrnew_lastchrnum + 4);
    let one = 1.0_f32;
    swc1(chr + 2836, half);
    swc1(chr + 2960, one);
    let mut left = 19;
    swc1(chr + 2832, half);
    let mut at = chr + 0x120c;
    swc1(chr + 2896, one);
    swc1(chr + 2900, 2.5_f32);
    sw(chr + 2780, 0);
    sw(chr + 2784, 0);
    sw(chr + 2876, 0);
    sw(chr + 2892, 0);
    sw(chr + 2948, 0);
    sw(chr + 2908, 0);
    sw(chr + 2912, 0);
    sw(chr + 2952, 0);
    swc1(chr + 2956, last);
    sw(chr + 2968, 0);
    sw(chr + 2716, 32);
    let angle = lwc1(prop + 80);
    sw(chr + 2704, 288);
    swc1(chr + 2972, angle);
    sw(chr + 1688, 0);
    sw(chr + 2692, 0);
    sw(chr + 2720, 0);
    sw(chr + 2724, 0);
    sw(chr + 2728, 0);
    sw(chr + 2768, 0);
    sw(chr + 2772, 0);
    sw(chr + 2776, -1);
    sw(chr + 2916, 0);
    sw(chr + 2788, 0);
    sw(chr + 2792, 0);
    sw(chr + 3000, 0);
    sw(chr + 3004, 0);
    sw(chr + 3028, 0);
    swc1(chr + 2796, lwc1(prop + 48));
    let animation = lw(prop + 32);
    swc1(chr + 2800, lwc1(prop + 52));
    let kind = lw(chr + 8);
    let z = lwc1(prop + 56);
    sw(chr + 0x1194, 1);
    swc1(chr + 2804, z);
    sw(chr + 0x11bc, -1);
    let x = lwc1(prop + 48);
    sw(chr + 2944, 0);
    swc1(chr + 2808, x);
    sw(chr + 0x119c, 0);
    let y = lwc1(prop + 52);
    sw(chr + 0x11a0, 0);
    swc1(chr + 2812, y);
    sw(chr + 0x11b4, 0);
    let z = lwc1(prop + 56);
    sw(chr + 0x11b8, 0);
    sw(chr + 0x11b0, 0);
    swc1(chr + 2884, 36.0_f32);
    swc1(chr + 2816, z);
    swc1(chr + 2888, 6.0_f32);
    swc1(chr + 0x1198, last);
    sw(chr + 2920, 0);
    sw(chr + 2924, 0);
    sw(chr + 2928, 0);
    sw(chr + 2932, 0);
    sw(chr + 2936, 0);
    sw(chr + 2940, 0);
    sw(chr + 2864, 0);
    sw(chr + 2904, 0);
    loop {
        sw(at, 0);
        left -= 1;
        at -= 4;
        if left < 0 {
            break;
        }
    }
    swc1(chr + 2964, lwc1(0x3ad1b8));
    let mode = lw(g::curgameparms + 72);
    for offset in [188, 104, 108, 112, 116, 120, 124, 168, 172, 176, 180, 184] {
        sw(animation + offset, 0);
    }
    if mode == 6 {
        sw(chr + 252, 0);
    }
    if kind == 1 {
        let r = random_unit() + 2.0_f32;
        swc1(chr + 2872, r);
        swc1(chr + 2856, r);
        let r2 = random_as_float();
        swc1(chr + 2868, 0.25_f32);
        swc1(chr + 2880, 3.5_f32);
        sw(chr + 2700, 0);
        swc1(chr + 2860, r2 * 2.3283064e-10_f32 * 0.75_f32 + 1.0_f32);
    } else if kind >= 2 {
        if kind == 2 {
            let eight = 8.0_f32;
            let quarter = 0.25_f32;
            sw(chr + 376, 26);
            swc1(chr + 2872, eight);
            swc1(chr + 2860, 3.0_f32);
            swc1(chr + 2880, 6.5_f32);
            swc1(chr + 2700, quarter);
            swc1(chr + 2856, eight);
            swc1(chr + 2868, quarter);
        } else if kind == 3 {
            let eight = 8.0_f32;
            swc1(chr + 2872, eight);
            swc1(chr + 2860, 4.0_f32);
            swc1(chr + 2868, 0.25_f32);
            swc1(chr + 2880, 10.0_f32);
            sw(chr + 376, 0);
            sw(chr + 268, 0);
            swc1(chr + 2856, eight);
            sw(chr + 2700, 0);
        }
    } else if kind == 0 {
        let r = random_unit();
        let wait = 8.0_f32 - (r + r);
        swc1(chr + 2872, wait);
        swc1(chr + 2856, wait);
        let r2 = random_as_float();
        swc1(chr + 2868, 0.25_f32);
        swc1(chr + 2880, 8.0_f32);
        swc1(chr + 2700, 0.5_f32);
        swc1(chr + 2860, 4.0_f32 - r2 * 2.3283064e-10_f32);
    }
    let head = chrGetChrHeadProp(lh(chr + 4), lh(chr + 6));
    sw(chr + 3024, head);
}
