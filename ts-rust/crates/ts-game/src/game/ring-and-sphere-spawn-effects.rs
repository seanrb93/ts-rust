//! The special effects of the rings and the spheres that spawn characters.

use crate::*;

/// The tick of the ring spawn effect for the effects group `group`: the prop in the first slot (+56) swings by a sine of the group's time (deleted after 1.5), the two particle
/// groups (at +72 and +76) are turned about y in opposite directions and the group is freed after 3 (@ 0x2acef0).
pub fn specialfxRingSpawnTick(group: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let groups = g::fxgroups;
    let slot = group * 92 + (groups + 56);
    let prop = lw(slot);
    let step = ps2float::mul(lwc1(g::updateframesf), lwc1(0x3ad708));
    if prop != 0 {
        let limit = 1.5_f32;
        let time_at = group * 92 + (groups + 48);
        let fraction = (lwc1(time_at)).pdiv(limit);
        let state = lw(prop + 352);
        let turned = dpadd(dpmul(dpmul(fptodp(fraction), ld(0x3a9778)), ld(0x3a9780)), 0x3fe0000000000000_i64);
        let index = dptoli(turned);
        swc1(state + 44, lwc1(((index & 2047) << 2) + g::g_SinTable));
        if limit < lwc1(time_at) {
            specialPropDelete(lw(slot));
            sw(slot, 0);
        }
    }
    let groups = g::fxgroups;
    let first = group * 92 + (groups + 72);
    if lw(first) != 0 {
        matrixRotY(sp, ps2float::mul(ps2float::mul(ps2float::mul(step, 3e+02_f32), lwc1(0x3ad70c)).pdiv(1.8e+02_f32), lw(groups + group * 92 + 4) as f32));
        matrixScaleSet(sp + 64, 1.0_f32, 1.0_f32, 1.0_f32);
        let matrix = lw(first) + 928;
        sceVu0MulMatrix(matrix, matrix, sp + 64);
        let matrix = lw(first) + 928;
        sceVu0MulMatrix(matrix, matrix, sp);
    }
    let second = group * 92 + (groups + 76);
    if lw(second) != 0 {
        matrixRotY(sp, ps2float::mul(ps2float::mul(ps2float::mul(step, -3e+02_f32), lwc1(0x3ad710)).pdiv(1.8e+02_f32), lw(groups + group * 92 + 4) as f32));
        matrixScaleSet(sp + 64, 1.0_f32, 1.0_f32, 1.0_f32);
        let matrix = lw(second) + 928;
        sceVu0MulMatrix(matrix, matrix, sp + 64);
        let matrix = lw(second) + 928;
        sceVu0MulMatrix(matrix, matrix, sp);
    }
    if 3.0_f32 < lwc1(groups + group * 92 + 48) {
        specialfxFree(group);
    }
}

/// The tick of the sphere spawn effect for the effects group `group`: each of the four props (from +56) swings by a sine of the group's time (still while the time is below 0, and
/// deleted past the constant at 0x3ad718), the particle group at +72 is turned about y and made bigger, and the group is freed after 4 (@ 0x2ad170).
pub fn specialfxSphereSpawnTick(group: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let groups = g::fxgroups;
    let step = ps2float::mul(lwc1(g::updateframesf), lwc1(0x3ad714));
    let times = groups + 48;
    let slots = groups + 56;
    let mut offset = 0;
    let mut left = 3;
    let mut slot = group * 92 + slots;
    let limit = lwc1(0x3ad718);
    loop {
        let prop = lw(slot);
        if prop != 0 {
            let time = lwc1(group * 92 + times);
            let state = lw(prop + 352);
            if !(time < 0.0_f32) {
                let turned = dpadd(dpmul(dpdiv(dpmul(fptodp(time), ld(0x3a9788)), ld(0x3a9790)), ld(0x3a9798)), 0x3fe0000000000000_i64);
                let index = dptoli(turned);
                swc1(state + 44, lwc1(((index & 2047) << 2) + g::g_SinTable));
            } else {
                sw(state + 44, 0);
            }
            let this = offset + group * 92;
            if limit < lwc1(group * 92 + times) {
                let at = this + slots;
                specialPropDelete(lw(at));
                sw(at, 0);
            }
        }
        slot += 4;
        left -= 1;
        offset += 4;
        if left < 0 {
            break;
        }
    }
    let groups = g::fxgroups;
    let particles = group * 92 + (groups + 72);
    if lw(particles) != 0 {
        matrixRotY(sp, ps2float::mul(ps2float::mul(ps2float::mul(step, 3e+02_f32), lwc1(0x3ad71c)).pdiv(1.8e+02_f32), lw(groups + group * 92 + 4) as f32));
        let grow = ps2float::add(ps2float::mul(step, lwc1(0x3ad720)), 1.0_f32);
        let tall = dptofp(dpadd(dpmul(fptodp(step), ld(0x3a97a0)), 0x3ff0000000000000_i64));
        matrixScaleSet(sp + 64, grow, tall, grow);
        let matrix = lw(particles) + 928;
        sceVu0MulMatrix(matrix, matrix, sp + 64);
        let matrix = lw(particles) + 928;
        sceVu0MulMatrix(matrix, matrix, sp);
    }
    if 4.0_f32 < lwc1(groups + group * 92 + 48) {
        specialfxFree(group);
    }
}
