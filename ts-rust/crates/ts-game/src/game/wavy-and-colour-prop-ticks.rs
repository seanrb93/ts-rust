//! Props whose surface is moved or coloured every tick: the green thing, the streamers, the spawning planet, the laser; and the use of props by the player.

use crate::*;

/// The base address of a prop model's bones (the bone records of 80 bytes before the skeleton).
fn first_bone(prop: i32) -> i32 {
    let skeleton = lw(lw(prop + 32));
    skeleton - lw(skeleton) * 80
}

/// The green thing: the vertices of the first chunk list's override array are those of the model with the height moved by a wave that runs outward from the middle (@ 0x27af68).
pub fn greenthingPropTick(prop: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let bone = first_bone(prop);
    allocateOverrideChunk(prop, 0, 1, 1);
    let out = getOverrideChunk(prop, 0, 1, 1, sp);
    if out != 0 {
        let mut source = lw(bone + 48);
        let count = lw(sp);
        let mut i = 0;
        let mut at = out;
        if count > 0 {
            loop {
                let x = lwc1(source);
                let z = lwc1(source + 8);
                let radius = ps2float::add(ps2float::mul(x, x), ps2float::mul(z, z)).abs().psqrt();
                let k = lwc1(0x3acf6c);
                let time = ps2float::mul(lw(g::gameframe) as f32, lwc1(0x3acf68));
                swc1(at, x);
                i += 1;
                let height_scale = lwc1(0x3acf74);
                let y = lwc1(source + 4);
                let depth = ps2float::sub(lwc1(0x3acf70), radius);
                source += 12;
                let wave = lwc1(((ps2float::add(ps2float::mul(ps2float::add(ps2float::add(radius, radius), time), k), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable);
                swc1(at + 8, z);
                swc1(at + 4, ps2float::sub(y, ps2float::mul(depth, ps2float::mul(ps2float::add(wave, 0.5_f32), height_scale))));
                at += 12;
                if i >= count {
                    break;
                }
            }
        }
    }
}

/// The streamers: each vertex moves by three sines of the game's time (and the prop's number), more the farther out it is (@ 0x278d48).
pub fn streamersPropTick(prop: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let state = lw(prop + 352);
    let bone = first_bone(prop);
    allocateOverrideChunk(prop, 0, 1, 1);
    let out = getOverrideChunk(prop, 0, 1, 1, sp);
    if out != 0 {
        let count = lw(sp);
        let mut at = out;
        let mut source = lw(bone + 48);
        let mut i = 0;
        let time = ps2float::add(lw(g::gameframe) as f32, (lw(state + 4) << 5) as f32);
        let double_time = ps2float::add(time, time);
        if count > 0 {
            loop {
                let x = lwc1(source);
                let table = g::g_SinTable;
                let k1 = lwc1(0x3ace84);
                i += 1;
                let k2 = lwc1(0x3ace88);
                let base = ps2float::mul(x, 3.5_f32);
                let k4 = lwc1(0x3ace8c);
                let half = 0.5_f32;
                let k5 = lwc1(0x3ace94);
                let z_at = ((ps2float::add(ps2float::mul(ps2float::add(base, ps2float::mul(double_time, lwc1(0x3ace90))), k4), half) as i32 & 2047) << 2) + table;
                swc1(at, ps2float::add(x, ps2float::mul(ps2float::mul(lwc1(((ps2float::add(ps2float::mul(ps2float::add(base, ps2float::mul(double_time, k1)), k4), half) as i32 & 2047) << 2) + table), k1), x)));
                swc1(at + 4, ps2float::add(lwc1(source + 4), ps2float::mul(ps2float::mul(lwc1(((ps2float::add(ps2float::mul(ps2float::add(base, ps2float::mul(double_time, k2)), k4), half) as i32 & 2047) << 2) + table), k5), x)));
                let z = lwc1(source + 8);
                source += 12;
                swc1(at + 8, ps2float::add(z, ps2float::mul(ps2float::mul(ps2float::sub(lwc1(z_at), half), k2), x)));
                at += 12;
                if i >= count {
                    break;
                }
            }
        }
    }
}

/// Props that the player uses (flags 0x80000004 of +16 being 4): when the player's use button is newly pressed within 2.5 of the prop (6.25 squared) the prop gets flag 1 at +28
/// and its use function (+384, with the argument at +392) is called (@ 0x26fc20).
pub fn propTickPlayer() {
    let mut count = lw(g::num_props);
    let mut i = 0;
    if count > 0 {
        let mut offset = 0;
        loop {
            let prop = lw(g::levelprops) + offset;
            if (ld(prop + 16) & 0x80000004_i64) != 4_i64 {
                i += 1;
            } else {
                let pressed = joyNewBut(lw(lw(g::currentplayer)));
                let player = lw(g::currentplayer);
                if (pressed & lw(player + 36)) != 0 {
                    let dx = ps2float::sub(lwc1(player + 140), lwc1(prop + 48));
                    let dy = ps2float::sub(lwc1(player + 144), lwc1(prop + 52));
                    let dz = ps2float::sub(lwc1(player + 148), lwc1(prop + 56));
                    if ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) < 6.25_f32 {
                        let function = lw(prop + 384);
                        sw(prop + 28, lw(prop + 28) | 1);
                        if function != 0 {
                            call_indirect(function, [prop, lw(prop + 392), 0, 0, 0, 0, 0, 0], [0.0; 8]);
                        }
                    }
                }
                count = lw(g::num_props);
                i += 1;
            }
            offset += 592;
            if i >= count {
                break;
            }
        }
    }
}

/// The spawning planet: the colour of each vertex (the four bytes) is mixed from the prop's two colours (+52 to +96 of its state) by the vertex's height, and the model's matrix is
/// scaled in y by +44 (@ 0x27b9e0).
pub fn planetspawnPropTick(prop: i32) {
    let frame = Frame::new(256);
    let sp = frame.sp();
    let state = lw(prop + 352);
    let bone = first_bone(prop);
    allocateOverrideChunk(prop, 0, 1, 4);
    let out = getOverrideChunk(prop, 0, 1, 4, sp);
    if out != 0 {
        let mut at = out;
        let mut source = lw(bone + 48);
        let mut i = 0;
        let alpha_a = lwc1(state + 52);
        let alpha_b = lwc1(state + 56);
        if lw(sp) > 0 {
            let red_a = lwc1(state + 76);
            let red_b = lwc1(state + 88);
            let green_a = lwc1(state + 80);
            let green_b = lwc1(state + 92);
            let blue_a = lwc1(state + 84);
            let blue_b = lwc1(state + 96);
            loop {
                let height = lwc1(source + 4);
                i += 1;
                source += 12;
                let t = ps2float::mul(height, 0.25_f32);
                let top = 127.0_f32;
                let u = ps2float::sub(1.0_f32, t);
                let a = fptoui(ps2float::mul(ps2float::add(ps2float::mul(alpha_a, t), ps2float::mul(alpha_b, u)), top));
                let r = fptoui(ps2float::mul(ps2float::add(ps2float::mul(u, red_a), ps2float::mul(t, red_b)), top));
                let g_ = fptoui(ps2float::mul(ps2float::add(ps2float::mul(u, green_a), ps2float::mul(t, green_b)), top));
                let b = fptoui(ps2float::mul(ps2float::add(ps2float::mul(u, blue_a), ps2float::mul(t, blue_b)), top));
                sw(at, r | (a & 255) << 24 | (b << 16 | g_ << 8));
                at += 4;
                if i >= lw(sp) {
                    break;
                }
            }
        }
    }
    let model = lw(prop + 32);
    matrixScaleSet(lw(model + 4), 1.0_f32, lwc1(state + 44), 1.0_f32);
}

/// The laser: the alpha of each vertex colour of every bone (the hologram alpha times 127) is put in the top byte, and the model's matrix is scaled in y by a wave, with a sine of
/// the game's time kept at +72 of the prop (@ 0x27b440).
pub fn laserPropTick(prop: i32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let alpha = fptoui(ps2float::mul(lwc1(g::hologramAlpha), 127.0_f32));
    let mut model = lw(prop + 32);
    let skeleton = lw(model);
    let bones = lw(skeleton);
    let mut i = 0;
    if bones > 0 {
        loop {
            let bone = i * 80 + (skeleton - bones * 80);
            allocateOverrideChunk(prop, 0, 1, 4);
            let out = getOverrideChunk(prop, 0, 1, 4, sp);
            let mut source = lw(bone + 56);
            let mut k = 0;
            let next = i + 1;
            model = lw(prop + 32);
            if lw(sp) > 0 {
                let mut at = out;
                loop {
                    let colour = lw(source);
                    k += 1;
                    source += 4;
                    sw(at, colour & 0xffffff | (colour & 255) + (alpha << 24));
                    at += 4;
                    if k >= lw(sp) {
                        break;
                    }
                }
            }
            let mut left = lb(bone + 6);
            if left > 0 {
                loop {
                    left -= 1;
                    if left == 0 {
                        break;
                    }
                }
            }
            i = next;
            if i >= lw(lw(model)) {
                break;
            }
        }
    }
    let k = lwc1(0x3acf90);
    let table = g::g_SinTable;
    let time = ps2float::mul(lw(g::gameframe) as f32, lwc1(0x3acf8c));
    let rate = lwc1(0x3acf94);
    let half = 0.5_f32;
    let matrix = lw(model + 4);
    swc1(prop + 72, ps2float::mul(lwc1(((ps2float::add(ps2float::mul(ps2float::mul(time, k).pdiv(180.0_f32), rate), half) as i32 & 2047) << 2) + table), 70.0_f32));
    let squash = ps2float::add(ps2float::mul(lwc1(((ps2float::add(ps2float::mul(ps2float::mul(ps2float::add(ps2float::add(time, time), 135.0_f32), k).pdiv(180.0_f32), rate), half) as i32 & 2047) << 2) + table), 0.25_f32), 0.75_f32);
    matrixScaleSet(matrix, 1.0_f32, squash, 1.0_f32);
}
