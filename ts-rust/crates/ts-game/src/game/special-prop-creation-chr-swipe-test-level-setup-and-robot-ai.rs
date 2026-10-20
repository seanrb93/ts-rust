//! Functions: making a prop with special data, the swipe test of a character against the hit spheres of another, the setup of the things of a level and the mind of a robot.

use super::particle_spawners::random_as_float;
use crate::*;

/// Make a prop of the kind `a0` with special data (@ 0x277bf8): the data (148 bytes) comes from the 25 specials of the level, or, for a temporary prop (the flag 2048), from the
/// 50 temporary ones. Returns the prop, or 0 when there was no room for its data. The data is cleared and then set by the kind of the prop (`lw(prop + 4) - 246`, 0 to 73).
pub fn specialPropNew(a0: i32, a1: i32, f12: f32, f13: f32, f14: f32, f15: f32) -> i32 {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let prop = propNew(a0, f12, f13, f14, f15);
    let mut id = 0;
    let mut data = 0;
    'made: {
        if (lw(prop + 16) & 2048) == 0 {
            let count = lw(g::numspecials);
            if count < 25 {
                id = -1;
                sw(prop + 352, lw(g::levelspecials) + count * 148);
                sw(g::numspecials, count + 1);
                data = lw(prop + 352);
                break 'made;
            }
        } else {
            let mut n = 0;
            let found;
            'search: {
                if lw(g::tmpSpecialDataUsed) != 0 {
                    n += 1;
                    let mut slot;
                    loop {
                        if n >= 50 {
                            found = false;
                            break 'search;
                        }
                        slot = (n << 2) + g::tmpSpecialDataUsed;
                        if lw(slot) == 0 {
                            break;
                        }
                        n += 1;
                    }
                    id = n;
                    sw(slot, 1);
                    sw(prop + 352, id * 148 + g::tmpSpecialData);
                } else {
                    sw(g::tmpSpecialDataUsed, 1);
                    id = 0;
                    sw(prop + 352, g::tmpSpecialData);
                }
                found = true;
            }
            if found {
                data = lw(prop + 352);
                break 'made;
            }
        }
        propDelete(prop);
        return 0;
    }
    let one = 1.0_f32;
    sw(data, id);
    sw(data + 4, a1);
    sw(data + 72, 0);
    swc1(data + 56, one);
    let kind = lw(prop + 4);
    sw(data + 8, 0);
    let kind = kind - 246;
    sw(data + 12, 0);
    sw(data + 16, 0);
    sw(data + 20, 0);
    sw(data + 24, 0);
    sw(data + 28, 0);
    sw(data + 36, 0);
    sw(data + 40, 0);
    swc1(data + 44, one);
    swc1(data + 48, one);
    swc1(data + 52, one);
    sw(data + 60, 0);
    sw(data + 64, 0);
    sw(data + 68, 0);
    sw(data + 76, 0);
    sw(data + 80, 0);
    sw(data + 84, 0);
    sw(data + 88, 0);
    sw(data + 92, 0);
    sw(data + 96, 0);
    if (kind as u32) < (74 as u32) {
        match kind {
            0 => {
                soundStartLoopRange(12, prop + 48, 25.0_f32);
                sw(prop + 16, lw(prop + 16) | 0x40000000);
            }
            1 | 25 => {
                obInstSetAnim(lw(prop + 32), 1, 0.0_f32);
                calculateClockTime();
                let flags = lw(prop + 16);
                sw(0x3b0110, 0);
                sw(prop + 16, flags | 0x40000000);
            }
            2 | 3 | 4 | 8 | 10 | 24 => {
                obInstSetAnim(lw(prop + 32), 1, 0.0_f32);
                let flags = lw(prop + 16) | 0x40000010;
                let routine = match kind {
                    2 => 0x2796f8,
                    3 => 0x2797a8,
                    4 => 0x279858,
                    8 => 0x279908,
                    10 => 0x2799e0,
                    _ => 0x279ae8,
                };
                sw(prop + 400, routine);
                sw(prop + 16, flags);
            }
            5 | 6 | 27 => {
                let flags = lw(prop + 16);
                let (extra, mode) = if kind == 27 { (0x40000200, 2) } else { (0x40000000, 1) };
                sw(prop + 140, mode);
                sw(prop + 16, flags | extra);
            }
            7 => {
                let flags = lw(prop + 16) | 0x40000200;
                sw(prop + 140, 2);
                sw(prop + 16, flags);
                soundStartLoopRange(6, prop + 48, 4e+01_f32);
                return prop;
            }
            9 => {
                allocateOverridePointers(prop);
                let flags = lw(prop + 16) | 0x40000200;
                sw(prop + 140, 2);
                swc1(prop + 88, 25.0_f32);
                sw(prop + 16, flags);
            }
            11 => {
                let flags = lw(prop + 16);
                sw(prop + 16, flags | 0x40000220);
            }
            12 => {
                let mut texture = g::electricityTextureOffset;
                let mut on = g::electricityOn;
                let mut count = 11;
                loop {
                    sw(on, 0);
                    count -= 1;
                    sw(texture, 0);
                    on += 4;
                    texture += 4;
                    if count < 0 {
                        break;
                    }
                }
                allocateOverridePointers(prop);
                sw(prop + 16, lw(prop + 16) | 1536);
                soundStartLoopRange(17, prop + 48, 25.0_f32);
                let flags = lw(prop + 16);
                sw(prop + 16, flags | 0x40000000);
            }
            13 | 14 | 15 | 16 | 17 | 18 | 23 | 33 | 34 | 35 | 36 | 41 | 42 | 45 | 46 | 47 | 63 | 64 | 65 | 66 | 67 | 69 | 70 | 71 => {}
            19 | 20 | 21 => {
                if kind == 19 {
                    allocateOverridePointers(prop);
                    let flags = lw(prop + 16) | 0x40000200;
                    sw(prop + 12, 16);
                    sw(prop + 140, 2);
                    sw(prop + 16, flags);
                } else if kind == 20 {
                    let flags = lw(prop + 16) | 0x40000000;
                    sw(prop + 12, 16);
                    sw(prop + 16, flags);
                    sw(prop + 140, 2);
                } else {
                    let flags = lw(prop + 16) | 0x40000000;
                    sw(prop + 12, 47);
                    sw(prop + 16, flags);
                    sw(prop + 140, 2);
                }
                let random = random_as_float();
                sw(prop + 16, lw(prop + 16) | 0x40000000);
                swc1(data + 32, random * 2.3283064e-10_f32);
            }
            22 => {
                allocateOverridePointers(prop);
                let flags = lw(prop + 16);
                sw(prop + 16, flags | 0x40000600);
            }
            26 | 38 | 39 | 40 | 68 | 72 => {
                let flags = lw(prop + 16) | 0x40000400;
                sw(prop + 12, 211);
                let inst = lw(prop + 32);
                sw(prop + 16, flags);
                obInstSetOverrideStates(inst, -1, 0, -1, 0);
                return prop;
            }
            28 | 29 => {
                if kind == 28 {
                    let flags = lw(prop + 16);
                    sw(prop + 140, 2);
                    sw(g::hologramAlpha, 0);
                    sw(prop + 16, flags | 1024);
                } else {
                    let inst = lw(prop + 32);
                    sw(prop + 16, lw(prop + 16) | 1024);
                    sw(prop + 140, 2);
                    obInstSetAnim(inst, 1, 0.0_f32);
                }
                allocateOverridePointers(prop);
                let flags = lw(prop + 16);
                sw(prop + 16, flags | 0x40000000);
            }
            30 | 31 => {
                if kind == 30 {
                    let from = 0x3a83f0;
                    let word = lw(from + 8);
                    sd(sp, ld(from));
                    sw(sp + 8, word);
                    soundStartLoopRange(21, prop + 48, 1e+02_f32);
                }
                sw(prop + 16, lw(prop + 16) | 1536);
                allocateOverridePointers(prop);
                let flags = lw(prop + 16) | 0x40000000;
                sw(prop + 12, 28);
                sw(prop + 16, flags);
            }
            32 => {
                let x = lwc1(0x3ace60);
                sw(prop + 16, lw(prop + 16) | 32);
                let y = lwc1(0x3ace64);
                let z = lwc1(0x3ace68);
                let mode = lw(g::curgameparms + 72) ^ 7;
                swc1(data + 44, 5.5_f32);
                swc1(data + 68, z);
                sw(data + 72, ((0 as u32) < (mode as u32)) as i32);
                swc1(data + 60, x);
                let flags = lw(prop + 16);
                swc1(data + 64, y);
                sw(prop + 16, flags | 0x40000000);
            }
            37 => {
                sw(prop + 16, lw(prop + 16) & -34 | 0x40000400);
                allocateOverridePointers(prop);
            }
            43 | 44 => {
                let inst = lw(prop + 32);
                let flags = lw(prop + 16) | 0x40000000;
                sw(prop + 140, 2);
                sh(inst + 292, 2);
                sw(prop + 16, flags);
            }
            48 => {
                obInstSetAnim(lw(prop + 32), 1, 0.0_f32);
                let inst = lw(prop + 32);
                swc1(data + 44, 0.0_f32);
                sw(inst + 248, data + 100);
                nullOverridePointers(prop);
                obInstSetOverrideStates(lw(prop + 32), 2, 0, -1, 0);
                sw(prop + 16, lw(prop + 16) | 1024);
            }
            49 => {
                sw(prop + 16, lw(prop + 16) | 1024);
            }
            50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 => {
                let inst = lw(prop + 32);
                sw(prop + 16, lw(prop + 16) | 1024);
                obInstSetAnim(inst, 1, 0.0_f32);
                swc1(data + 44, 0.0_f32);
                obInstSetOverrideStates(lw(prop + 32), 2, 0, -1, 0);
                sw(prop + 140, 2);
            }
            62 => {
                let inst = lw(prop + 32);
                sw(prop + 16, lw(prop + 16) | 1024);
                obInstSetAnim(inst, 1, 0.0_f32);
                swc1(data + 44, 0.0_f32);
                obInstSetOverrideStates(lw(prop + 32), 2, 0, -1, 0);
                return prop;
            }
            _ => {
                // 73
                sw(prop + 16, lw(prop + 16) | 512);
                allocateOverridePointers(prop);
                let flags = lw(prop + 16) | 0x40000400;
                sw(prop + 12, 9);
                sw(prop + 16, flags);
            }
        }
    }
    prop
}

/// The triangle test of `chrSwipeTestChr`: the point `point` (3 floats) is tested against the triangle of the swipe at sp+96, +112 and +128. The vectors of the triangle are
/// stored for the caller, and the cross products are kept from `scratch` on. Returns true when the point is inside the triangle.
fn swipe_triangle(sp: i32, point: i32, scratch: i32) -> bool {
    let f7 = lwc1(sp + 100);
    let f8 = lwc1(sp + 104);
    let f10 = lwc1(sp + 132) - f7;
    let f9 = lwc1(sp + 136) - f8;
    let f4 = lwc1(sp + 116) - f7;
    let f6 = lwc1(sp + 96);
    let f3 = lwc1(sp + 120) - f8;
    let px = lwc1(point);
    let f12 = lwc1(sp + 128) - f6;
    let f5 = lwc1(sp + 112);
    let py = lwc1(point + 4);
    let f13 = px - f6;
    let pz = lwc1(point + 8);
    swc1(sp + 152, f3);
    let f5b = f5 - f6;
    swc1(sp + 148, f4);
    let f14 = py - f7;
    swc1(sp + 160, f13);
    let f11 = pz - f8;
    swc1(sp + 176, f12);
    let f15 = f4 * f9 - f3 * f10;
    swc1(sp + 144, f5b);
    swc1(sp + 164, f14);
    swc1(sp + 168, f11);
    swc1(sp + 180, f10);
    swc1(sp + 184, f9);
    swc1(scratch, f15);
    let fa = f3 * f12 - f5b * f9;
    let fb = f4 * f11 - f3 * f14;
    let fc = f3 * f13 - f5b * f11;
    swc1(scratch + 4, fa);
    swc1(scratch + 16, fb);
    swc1(scratch + 20, fc);
    let fd = f5b * f14 - f4 * f13;
    let fe = f5b * f10 - f4 * f12;
    swc1(scratch + 24, fd);
    swc1(scratch + 8, fe);
    if (f15 * fb + fa * fc + fe * fd) < 0.0_f32 {
        return false;
    }
    let g1 = f11 * f12 - f13 * f9;
    let g2 = f14 * f9 - f11 * f10;
    swc1(scratch + 20, g1);
    swc1(scratch + 16, g2);
    let g3 = f13 * f10 - f14 * f12;
    swc1(scratch + 24, g3);
    !((f15 * g2 + fa * g1 + fe * g3) < 0.0_f32)
}

/// Test a swipe of the character `a0` from `a1` to `a2` (with the triangle made with the point `a3`) against the hit spheres and capsules of its model, and keep the nearest hit
/// (@ 0x2872a8). `a4` gets the hit place (in the world) and `a5` the direction of the swipe at the hit, when there is a hit. The number of the part that was hit is kept in
/// `s_PossibleLastHitPart_3afa30`.
pub fn chrSwipeTestChr(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) {
    let frame = Frame::new(624);
    let sp = frame.sp();
    let model_data = lw(a0 + 32);
    let mind = lw(a0 + 352);
    let model = lw(model_data);
    sw(sp + 384, a4);
    sw(sp + 392, model);
    sw(sp + 388, a5);
    memset(sp, 0, 12);
    let end_x = lwc1(a2);
    let start_x = lwc1(a1);
    let end_y = lwc1(a2 + 4);
    let start_y = lwc1(a1 + 4);
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let end_z = lwc1(a2 + 8);
    let start_z = lwc1(a1 + 8);
    let angle = lwc1(a0 + 76) * lwc1(0x3ad1ac) / 1.8e+02_f32;
    let chr_y = lwc1(a0 + 52);
    let chr_z = lwc1(a0 + 56);
    let dz = end_z - start_z;
    let chr_x = lwc1(a0 + 48);
    let other_z = lwc1(a3 + 8);
    let other_x = lwc1(a3);
    let other_y = lwc1(a3 + 4);
    let word = lw(0x3a8878);
    sd(sp + 16, ld(0x3a8870));
    sw(sp + 24, word);
    swc1(sp + 116, end_y - chr_y);
    swc1(sp + 120, end_z - chr_z);
    swc1(sp + 96, start_x - chr_x);
    swc1(sp + 136, other_z - chr_z);
    swc1(sp + 128, other_x - chr_x);
    swc1(sp + 132, other_y - chr_y);
    let turn = -angle;
    let mut best = lwc1(0x3afa54);
    let length = dx * dx + dy * dy + dz * dz;
    swc1(sp + 100, start_y - chr_y);
    swc1(sp + 104, start_z - chr_z);
    swc1(sp + 112, end_x - chr_x);
    matrixTransRotY(sp + 32, 0.0_f32, 0.0_f32, 0.0_f32, turn);
    matrixVecMulAligned(&[(sp + 32) as i64, (sp + 96) as i64]);
    sw(sp + 396, sp + 112);
    matrixVecMulAligned(&[(sp + 32) as i64, (sp + 112) as i64]);
    sw(sp + 400, sp + 128);
    matrixVecMulAligned(&[(sp + 32) as i64, (sp + 128) as i64]);
    let f0 = lwc1(sp + 96);
    let f8 = lwc1(sp + 104);
    let f4 = lwc1(sp + 100);
    let f1 = lwc1(sp + 120) - f8;
    let f7 = lwc1(sp + 128) - f0;
    let f2 = lwc1(sp + 112) - f0;
    let f3 = lwc1(sp + 136);
    let f5 = lwc1(sp + 116) - f4;
    let f6 = lwc1(sp + 132) - f4;
    swc1(sp + 152, f1);
    let f3b = f3 - f8;
    swc1(sp + 144, f2);
    swc1(sp + 148, f5);
    swc1(sp + 160, f7);
    swc1(sp + 168, f3b);
    swc1(sp + 164, f6);
    let normal_y = f1 * f7 - f2 * f3b;
    let normal_x = f5 * f3b - f1 * f6;
    swc1(sp + 196, normal_y);
    swc1(sp + 192, normal_x);
    let normal_z = f2 * f6 - f5 * f7;
    let size = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).abs().sqrt();
    swc1(sp + 200, normal_z);
    let scale = 1.0_f32 / size;
    let count = lw(lw(sp + 392));
    let nx = lwc1(sp + 192) * scale;
    let ny = lwc1(sp + 196) * scale;
    swc1(sp + 200, lwc1(sp + 200) * scale);
    swc1(sp + 192, nx);
    swc1(sp + 196, ny);
    if count > 0 {
        let mut part = 0;
        let mut at = mind + 3060;
        loop {
            'next: {
                let f1_4 = lwc1(at + 8);
                let hit_point;
                let nearest_x;
                let nearest_y;
                let nearest_z;
                if lb(at + 1) != 1 {
                    let matrices = lw(model_data + 4);
                    swc1(sp + 272, f1_4);
                    swc1(sp + 276, lwc1(at + 12));
                    swc1(sp + 280, lwc1(at + 16));
                    swc1(sp + 224, lwc1(at + 20));
                    swc1(sp + 228, lwc1(at + 24));
                    swc1(sp + 232, lwc1(at + 28));
                    matrixVecMulAligned(&[(matrices + (lb(at) << 6)) as i64, (sp + 272) as i64]);
                    matrixVecMulAligned(&[(lw(model_data + 4) + (lb(at) << 6)) as i64, (sp + 224) as i64]);
                    let distance = HitTestDistancePointToLineSegmentSquared(sp + 96, sp + 272, sp + 224, sp + 304);
                    if (length + lwc1(at + 4)) < distance {
                        break 'next;
                    }
                    let plane = HitTestDistanceLineSegmentToPlane(sp + 272, sp + 224, sp + 96, sp + 192, 0, sp + 320);
                    let plane_squared = plane * plane;
                    if lwc1(at + 4) < plane_squared {
                        break 'next;
                    }
                    'edge: {
                        let first = HitTestDistanceLineSegmentToLineSegmentSquared(sp + 272, sp + 224, lw(sp + 400), sp + 96, 0, sp + 336);
                        let mut foot_x = lwc1(sp + 336);
                        if !(first <= lwc1(at + 4)) {
                            let second = HitTestDistanceLineSegmentToLineSegmentSquared(sp + 272, sp + 224, lw(sp + 396), sp + 96, 0, sp + 336);
                            foot_x = lwc1(sp + 336);
                            if !(second <= lwc1(at + 4)) {
                                break 'edge;
                            }
                        }
                        let foot_y = lwc1(sp + 340);
                        let foot_z = lwc1(sp + 344);
                        swc1(sp + 320, foot_x);
                        swc1(sp + 324, foot_y);
                        swc1(sp + 328, foot_z);
                    }
                    if !swipe_triangle(sp, sp + 320, sp + 352) {
                        break 'next;
                    }
                    nearest_x = lwc1(sp + 304);
                    if !(plane_squared < best) {
                        break 'next;
                    }
                    best = plane_squared;
                    nearest_y = lwc1(sp + 308);
                    nearest_z = lwc1(sp + 312);
                    hit_point = sp + 320;
                } else {
                    let matrices = lw(model_data + 4);
                    swc1(sp + 208, f1_4);
                    swc1(sp + 212, lwc1(at + 12));
                    swc1(sp + 216, lwc1(at + 16));
                    matrixVecMulAligned(&[(matrices + (lb(at) << 6)) as i64, (sp + 208) as i64]);
                    let x = lwc1(sp + 208) - lwc1(sp + 96);
                    let high = lwc1(sp + 216);
                    let y = lwc1(sp + 212) - lwc1(sp + 100);
                    let low = lwc1(sp + 104);
                    swc1(sp + 224, x);
                    let z = high - low;
                    swc1(sp + 228, y);
                    swc1(sp + 232, z);
                    let radius = lwc1(at + 4);
                    if (length + radius) < x * x + y * y + z * z {
                        break 'next;
                    }
                    let mut depth = x * lwc1(sp + 192) + y * lwc1(sp + 196) + z * lwc1(sp + 200);
                    if !(0.0_f32 <= depth) {
                        depth = -depth;
                    }
                    let depth_squared = depth * depth;
                    if radius < depth_squared {
                        break 'next;
                    }
                    'edge: {
                        let first = HitTestDistancePointToLineSegmentSquared(sp + 208, lw(sp + 400), sp + 96, sp + 256);
                        let mut foot_x = lwc1(sp + 256);
                        if !((first * first) <= lwc1(at + 4)) {
                            let second = HitTestDistancePointToLineSegmentSquared(sp + 208, lw(sp + 396), sp + 96, sp + 256);
                            foot_x = lwc1(sp + 256);
                            if !((second * second) <= lwc1(at + 4)) {
                                let foot_y = lwc1(sp + 212) - depth * lwc1(sp + 196);
                                let foot_z = lwc1(sp + 216) - depth * lwc1(sp + 200);
                                swc1(sp + 240, lwc1(sp + 208) - depth * lwc1(sp + 192));
                                swc1(sp + 244, foot_y);
                                swc1(sp + 248, foot_z);
                                break 'edge;
                            }
                        }
                        let foot_y = lwc1(sp + 260);
                        let foot_z = lwc1(sp + 264);
                        swc1(sp + 240, foot_x);
                        swc1(sp + 244, foot_y);
                        swc1(sp + 248, foot_z);
                    }
                    if !swipe_triangle(sp, sp + 240, sp + 272) {
                        break 'next;
                    }
                    nearest_x = lwc1(sp + 208);
                    if !(depth_squared < best) {
                        break 'next;
                    }
                    best = depth_squared;
                    nearest_y = lwc1(sp + 212);
                    nearest_z = lwc1(sp + 216);
                    hit_point = sp + 240;
                }
                let hit_x = lwc1(hit_point);
                let hit_y = lwc1(hit_point + 4);
                let hit_z = lwc1(hit_point + 8);
                swc1(sp, hit_x);
                swc1(sp + 4, hit_y);
                swc1(sp + 16, hit_x - nearest_x);
                swc1(sp + 20, hit_y - nearest_y);
                swc1(sp + 24, hit_z - nearest_z);
                swc1(sp + 8, hit_z);
                sw(g::s_PossibleLastHitPart_3afa30, part);
            }
            part += 1;
            at += 32;
            if part >= lw(lw(sp + 392)) {
                break;
            }
        }
    }
    let place = lw(sp + 384);
    if best < lwc1(0x3afa54) {
        let hit_y = lwc1(sp + 4);
        if place != 0 {
            let hit_z = lwc1(sp + 8);
            let hit_x = lwc1(sp);
            swc1(place + 4, hit_y);
            swc1(place + 8, hit_z);
            swc1(place, hit_x);
            let x = lwc1(a0 + 48);
            let y = lwc1(a0 + 52);
            let turn = lwc1(a0 + 76) * lwc1(0x3ad1b0) / 1.8e+02_f32;
            let z = lwc1(a0 + 56);
            matrixTransRotY(sp + 32, x, y, z, turn);
            matrixVecMul(sp + 32, lw(sp + 384));
        }
        let dir_x = lwc1(sp + 16);
        if lw(sp + 388) != 0 {
            let dir_y = lwc1(sp + 20);
            let dir_z = lwc1(sp + 24);
            let direction = lw(sp + 388);
            let scale = 1.0_f32 / (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).abs().sqrt();
            let factor = lwc1(0x3ad1b4);
            swc1(direction + 8, dir_z * scale);
            swc1(direction, dir_x * scale);
            swc1(direction + 4, dir_y * scale);
            let turn = lwc1(a0 + 76) * factor / 1.8e+02_f32;
            matrixRotY(sp + 32, turn);
            matrixVecMul(sp + 32, lw(sp + 388));
        }
    }
}

/// Set up the things of the level (@ 0x2291e8): the props, the pickups (with their lists for the enemies), the doors, the cars, the remote props and the special effects of the
/// setup of the level; then every player gets a start position and the enemies are set up. Levels 6 and 101 to 105 have nothing to set up.
pub fn lvLevelSetup() {
    let level = lw(g::lvlevelnum);
    let setup = lvGetLevelSetup(level);
    let level = lw(g::lvlevelnum);
    if level == 6 || level == 101 || level == 102 || level == 103 || level == 104 || level == 105 {
        return;
    }
    if level == 100 {
        lvCreateTileSetSetup(setup);
    }
    if setup != 0 {
        let combos = lvGetGunComboNums();
        lvLevelSetPadNumbers();
        if lw(setup + 12) > 0 {
            let mut n = 0;
            loop {
                let prop = lw(setup + 8) + n * 28;
                if lvCheckCreateFlags(lw(prop)) != 0 {
                    setupProp(prop);
                }
                n += 1;
                if n >= lw(setup + 12) {
                    break;
                }
            }
        }
        if lw(setup + 92) > 0 {
            let mut n = 0;
            let mut offset = 0;
            loop {
                let prop = lw(setup + 88) + offset;
                if lvCheckCreateFlags(lw(prop)) != 0 {
                    setupProp(prop);
                }
                n += 1;
                offset += 28;
                if n >= lw(setup + 92) {
                    break;
                }
            }
        }
        if lw(setup + 20) > 0 {
            let mut n = 0;
            loop {
                'next: {
                    let mut made = 0;
                    let pickup = lw(setup + 16) + n * 48;
                    if lvCheckCreateFlags(lw(pickup)) == 0 {
                        break 'next;
                    }
                    let prop_number = lvGetPropNum(lw(pickup + 4));
                    let mut ammo = 0;
                    if prop_number < 0 {
                        let kind = lw(pickup + 4);
                        if kind < -20 {
                            if kind < -25 {
                                break 'next;
                            }
                            let f16 = lwc1(pickup + 40);
                            let pad = lw(pickup + 8);
                            let f12 = lwc1(pickup + 12);
                            let extref = lw(pickup + 24);
                            let f13 = lwc1(pickup + 28);
                            let f14 = lwc1(pickup + 32);
                            let f15 = lwc1(pickup + 36);
                            let gun = lh((lw(combos - ((kind + 20) << 2) - 4) << 3) + (g::guncombos + 4)) * 400 + g::guns;
                            let a3 = lw(gun + 16);
                            sw(pickup + 20, a3);
                            let a0 = lw(gun + 12);
                            let a2 = lw(gun + 24);
                            propNewPickupOnPadExtref(a0, pad, a2, a3, extref, 0, 0, 0, f12, f13, f14, f15, f16);
                        }
                    } else {
                        let id = lw(pickup + 4);
                        let mut second = 0;
                        let combo;
                        if id < 0 {
                            combo = if id >= -5 { lw(combos - (id << 2) - 4) } else { 0 };
                        } else {
                            combo = id;
                        }
                        let amount = lw(pickup + 8);
                        let parms = g::curgameparms;
                        if amount == 2 {
                            let entry = (combo << 3) + g::guncombos;
                            let guns = g::guns;
                            let table = guns + 16;
                            let reserve = lw(guns + lh(entry + 4) * 400 + 24);
                            ammo = lw(lh(entry + 4) * 400 + table);
                            second = if (lw(lh(entry + 2) * 400 + table) ^ ammo) == 0 { reserve << 1 } else { reserve };
                        }
                        let kind;
                        'create: {
                            if lw(parms + 72) == 5 {
                                if amount == 1 {
                                    kind = amount - 1;
                                    break 'create;
                                }
                            }
                            let f12 = lwc1(pickup + 12);
                            let a3 = lw(pickup + 20);
                            let extref = lw(pickup + 24);
                            let f13 = lwc1(pickup + 28);
                            let f14 = lwc1(pickup + 32);
                            let f15 = lwc1(pickup + 36);
                            let f16 = lwc1(pickup + 40);
                            made = propNewPickupOnPadExtref(prop_number, amount, combo, a3, extref, ammo, second, 0, f12, f13, f14, f15, f16);
                            kind = lw(pickup + 8) - 1;
                        }
                        if (kind as u32) < (16 as u32) {
                            match kind {
                                0 => {
                                    if lw(g::curgameparms + 72) != 5 {
                                        enemyAddToBagList(made, lw(pickup + 44));
                                    }
                                }
                                1 => enemyAddToWeaponList(made, lw(pickup + 44)),
                                3 | 7 => enemyAddToHealthList(made, lw(pickup + 44)),
                                15 => enemyAddToDropList(made, lw(pickup + 44)),
                                _ => {}
                            }
                        }
                        let id = lw(pickup + 4);
                        if ((-6 - id) as u32) < (4 as u32) {
                            if lw(g::curgameparms + 72) != 2 {
                                break 'next;
                            }
                            let f12 = lwc1(pickup + 12);
                            let a3 = lw(pickup + 20);
                            let extref = lw(pickup + 24);
                            let f13 = lwc1(pickup + 28);
                            let f14 = lwc1(pickup + 32);
                            let f15 = lwc1(pickup + 36);
                            let prop = propNewPickupOnPadExtref(314 - id, 1, combo, a3, extref, 0, 0, 0, f12, f13, f14, f15, 0.5_f32);
                            let pad = padNumFromExtref(lw(pickup + 24));
                            enemyAddToBagList(prop, pad);
                            break 'next;
                        }
                    }
                }
                n += 1;
                if n >= lw(setup + 20) {
                    break;
                }
            }
        }
        if lw(setup + 28) > 0 {
            let mut n = 0;
            loop {
                let door = lw(setup + 24) + (n << 4);
                if lvCheckCreateFlags(lw(door)) != 0 {
                    doorNewInVolume(lw(door + 4), lw(door + 8), lw(door + 12));
                }
                n += 1;
                if n >= lw(setup + 28) {
                    break;
                }
            }
        }
        'cars: {
            if lw(g::lvlevelnum) == 27 {
                if lw(g::curgameparms + 72) != 7 {
                    break 'cars;
                }
            }
            if lw(setup + 76) > 0 {
                let mut n = 0;
                loop {
                    let car = lw(setup + 72) + (n << 5);
                    if lvCheckCreateFlags(lw(car)) != 0 {
                        let prop = carPropNew(lw(car + 4), lw(car + 8), lw(car + 12), lw(car + 16), lw(car + 20), lw(car + 24), lw(car + 28));
                        if lw(g::curgameparms + 72) == 4 {
                            if lw(g::lvlevelnum) == 11 {
                                enemyAddToEscortList(prop);
                            }
                        }
                    }
                    n += 1;
                    if n >= lw(setup + 76) {
                        break;
                    }
                }
            }
        }
        if lw(setup + 84) > 0 {
            let mut n = 0;
            loop {
                let remote = lw(setup + 80) + (n << 6);
                if lvCheckCreateFlags(lw(remote)) != 0 {
                    remotePropNew(
                        lw(remote + 4),
                        remote + 16,
                        remote + 32,
                        lw(remote + 8),
                        lw(remote + 12),
                        0,
                        lwc1(remote + 28),
                        lwc1(remote + 44),
                        lwc1(remote + 48),
                        lwc1(remote + 52),
                        lwc1(remote + 56),
                        lwc1(remote + 60),
                    );
                }
                n += 1;
                if n >= lw(setup + 84) {
                    break;
                }
            }
        }
        if lw(setup + 100) > 0 {
            let mut n = 0;
            let mut offset = 0;
            loop {
                let effect = lw(setup + 96) + offset;
                n += 1;
                offset += 20;
                specialfxNew(lw(effect), effect + 4, 0, 0, lw(effect + 16), -1, 0);
                if n >= lw(setup + 100) {
                    break;
                }
            }
        }
        weatherSet();
    }
    if lw(g::lvnumplayers) > 0 {
        let mut n = 0;
        let mut offset = 0;
        loop {
            n += 1;
            playerSetStartPos(lw(g::players) + offset);
            offset += 1820;
            if n >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    enemySetup();
    let parms = g::curgameparms;
    if (lw(parms + 80) & 32) != 0 {
        if lw(parms + 68) == 15 {
            if lw(g::challenge6a_barrel_pads) >= 0 {
                let mut n = 0;
                let mut offset = 0;
                let pads = g::challenge6a_barrel_pads;
                loop {
                    n += 1;
                    let pad = padNumFromExtref(lw(offset + pads));
                    let record = lw(g::levelpads) + pad * 28;
                    let z = lwc1(record + 20);
                    let y = lwc1(record + 16) + lwc1(0x3ac8d8);
                    let x = lwc1(record + 12);
                    propNew(202, x, y, z, 0.0_f32);
                    offset = n << 2;
                    if lw(offset + pads) < 0 {
                        break;
                    }
                }
            }
        }
    }
}

/// The mind of a robot character `a0` (@ 0x2c70e8): it keeps a target, follows its pads (or goes for the first player once it has seen them), turns towards where it goes
/// at the turning speed of its record, picks up speed and moves, with sideways movement that depends on the time, and the rooms it is in are found again.
pub fn robotAI(a0: i32) {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let mind = lw(a0 + 352);
    let first_player = lw(lw(g::players) + 384);
    let number = ((mind.wrapping_sub(lw(g::chrinfo))).wrapping_mul(-0x795b3d1f)) >> 4;
    let target = suggestTarget(a0, first_player, 0);
    swc1(mind + 3008, lwc1(a0 + 48));
    let alerted = lw(mind + 2772);
    let target_mind = lw(target + 352);
    let eye = lwc1(a0 + 52) + 1.0_f32;
    sw(mind + 2788, target);
    swc1(mind + 3012, eye);
    swc1(mind + 3016, lwc1(a0 + 56));
    if alerted != 0 {
        let state = lw(mind + 2728);
        if (state as u32) < (7 as u32) {
            match state {
                0 => {
                    enemySetMoveMode(a0, 1);
                    let speed = lwc1(mind + 2856);
                    sw(mind + 2728, 1);
                    swc1(mind + 2872, speed);
                }
                1 => {
                    enemySetMoveMode(a0, 1);
                    let player = lw(g::players);
                    let x = lwc1(player + 140);
                    swc1(mind + 2796, x);
                    let y = lwc1(player + 144);
                    swc1(mind + 2800, y);
                    let z = lwc1(player + 148);
                    swc1(mind + 2804, z);
                    let dx = x - lwc1(a0 + 48);
                    let dy = y - lwc1(a0 + 52);
                    let dz = z - lwc1(a0 + 56);
                    if (dx * dx + dy * dy + dz * dz) < 1e+02_f32 {
                        sw(mind + 2728, 6);
                    }
                }
                2 => {
                    enemySetMoveMode(a0, 1);
                }
                3 => {
                    enemySetMoveMode(a0, 1);
                    let found = findpadslinked2pad(lw(mind + 680), sp + 96, 20);
                    if found > 0 {
                        let pads = lw(g::levelpads);
                        let mut n = 0;
                        loop {
                            let pad = lw(sp + 96 + (n << 2)) * 28 + pads;
                            let x = lwc1(pad + 12);
                            swc1(mind + 2796, x);
                            let y = lwc1(pad + 16);
                            swc1(mind + 2800, y);
                            let z = lwc1(pad + 20);
                            swc1(mind + 2804, z);
                            let dx = x - lwc1(a0 + 48);
                            let dy = y - lwc1(a0 + 52);
                            let dz = z - lwc1(a0 + 56);
                            if 1e+02_f32 < dx * dx + dy * dy + dz * dz {
                                sw(mind + 2728, 4);
                            }
                            n += 1;
                            if n >= found {
                                break;
                            }
                        }
                    }
                }
                4 => {
                    enemySetMoveMode(a0, 1);
                    let x = lwc1(mind + 2796);
                    let dy = lwc1(mind + 2800) - lwc1(a0 + 52);
                    let z = lwc1(mind + 2804);
                    let dx = x - lwc1(a0 + 48);
                    let dz = z - lwc1(a0 + 56);
                    if (dx * dx + dy * dy + dz * dz) < lwc1(0x3ad998) {
                        sw(mind + 2728, 1);
                    }
                }
                5 => {}
                _ => {
                    enemySetMoveMode(a0, 8);
                    let player = lw(g::players);
                    let x = lwc1(player + 140);
                    swc1(mind + 2796, x);
                    let y = lwc1(player + 144);
                    swc1(mind + 2800, y);
                    let z = lwc1(player + 148);
                    swc1(mind + 2804, z);
                    let dx = x - lwc1(a0 + 48);
                    let dy = y - lwc1(a0 + 52);
                    let dz = z - lwc1(a0 + 56);
                    if 64.0_f32 < dx * dx + dy * dy + dz * dz {
                        sw(mind + 2728, 1);
                    }
                    if 0.5_f32 < lwc1(mind + 2896) {
                        let flags = lw(mind + 2704);
                        sw(mind + 2896, 0);
                        sw(mind + 2704, flags | 2);
                    }
                }
            }
        }
    } else {
        sw(mind + 2728, 0);
        let pad = lw(target_mind + 680);
        if lw(mind + 684) == pad {
            sw(mind + 684, pad);
        } else {
            sw(mind + 1688, 0);
            sw(mind + 2692, 0);
            sw(mind + 684, lw(target_mind + 680));
        }
        let next = findNextPad(mind + 680);
        if ((next + 2) as u32) < (2 as u32) {
            enemySetMoveMode(a0, 8);
            return;
        }
        enemySetMoveMode(a0, 1);
        let pads = lw(g::levelpads);
        swc1(mind + 2872, lwc1(mind + 2860));
        let near = lwc1(0x3ad994);
        let record = next * 28 + pads;
        let x = lwc1(record + 12);
        swc1(mind + 2796, x);
        let y = lwc1(record + 16);
        swc1(mind + 2800, y);
        let z = lwc1(record + 20);
        swc1(mind + 2804, z);
        let dx = lwc1(a0 + 48) - x;
        let dy = lwc1(a0 + 52) - y;
        let dz = lwc1(a0 + 56) - z;
        if (dx * dx + dy * dy + dz * dz) < near {
            let left = lw(mind + 1688);
            sw(mind + 680, next);
            sw(mind + 1688, left - 1);
        }
    }
    let x = lwc1(a0 + 48);
    let z = lwc1(a0 + 56);
    let goal_x = lwc1(mind + 2796);
    let goal_z = lwc1(mind + 2804);
    let at = a0 + 48;
    let previous = a0 + 112;
    let toward_x = goal_x - x;
    let toward_z = goal_z - z;
    let bearing = atan2(fptodp(toward_x), fptodp(toward_z));
    let bearing = dpmul(bearing, ld(0x3a9e90));
    let bearing = dpdiv(bearing, ld(0x3a9e98));
    let bearing = dptofp(bearing);
    let difference = anglediff(lwc1(a0 + 76), bearing);
    let mut size = fptodp(difference);
    if dpcmp(size, 0_i64) < 0 {
        size = dpsub(0_i64, size);
    }
    let turn_speed = lwc1(mind + 2880);
    let step = turn_speed * lwc1(g::updateframesf);
    let turned;
    if dpcmp(size, fptodp(step)) <= 0 {
        turned = lwc1(a0 + 76) + difference;
    } else {
        let heading = lwc1(a0 + 76);
        if !(0.0_f32 < difference) {
            turned = heading - step;
        } else {
            turned = heading + step;
        }
    }
    swc1(a0 + 76, turned);
    let mut heading = lwc1(a0 + 76);
    let full = 3.6e+02_f32;
    let mut last = heading;
    let mode = lw(mind + 2708);
    if full < heading {
        loop {
            heading = last - full;
            swc1(a0 + 76, heading);
            last = heading;
            if !(full < heading) {
                break;
            }
        }
    }
    if heading < 0.0_f32 {
        loop {
            let wrapped = heading + 3.6e+02_f32;
            heading = wrapped;
            if !(wrapped < 0.0_f32) {
                break;
            }
        }
        swc1(a0 + 76, heading);
    }
    let acceleration = lwc1(mind + 2868);
    'speed: {
        if mode != 1 {
            let speed = lwc1(mind + 2876);
            let reached;
            if !(0.0_f32 < acceleration) {
                let faster = speed + acceleration;
                reached = 0.0_f32 < faster;
                swc1(mind + 2876, faster);
            } else {
                let slower = speed - acceleration;
                reached = slower < 0.0_f32;
                swc1(mind + 2876, slower);
            }
            if reached {
                swc1(mind + 2876, 0.0_f32);
            }
        } else {
            let top = lwc1(mind + 2872);
            let faster = lwc1(mind + 2876) + acceleration;
            swc1(mind + 2876, faster);
            if top < faster {
                swc1(mind + 2876, top);
            }
            let bottom = -top;
            if !(lwc1(mind + 2876) < bottom) {
                sw(sp + 16, 0);
                break 'speed;
            }
            swc1(mind + 2876, bottom);
        }
        sw(sp + 16, 0);
    }
    sw(sp + 20, 0);
    let angle = lwc1(a0 + 76) * lwc1(0x3ad99c) / 1.8e+02_f32;
    swc1(sp + 24, lwc1(mind + 2876) / 6e+01_f32);
    matrixRotY(sp + 32, angle);
    matrixVecRotAligned(&[(sp + 32) as i64, (sp + 16) as i64]);
    swc1(sp + 16, lwc1(a0 + 48) + lwc1(sp + 16));
    let time = lwc1(mind + 2892);
    let double = time + time;
    let wave = cosf(double);
    let fast = lwc1(mind + 2892) * 5.0_f32;
    let bob = wave * lwc1(0x3ad9a0) + lwc1(0x3ad9a4);
    let wave = sinf(fast);
    let faster = lwc1(mind + 2892) * 7.0_f32;
    let bob = bob + wave * lwc1(0x3ad9a8);
    let wave = cosf(faster);
    let player = lw(g::currentplayer);
    let radius = lwc1(0x3ad9b0);
    let bob = bob + wave * lwc1(0x3ad9ac);
    swc1(sp + 24, lwc1(a0 + 56) + lwc1(sp + 24));
    swc1(sp + 20, bob);
    let height = lwc1(player + 132);
    moveTestDoSlide(number, at, sp + 16, 0, 1, radius, height);
    let new_z = lwc1(sp + 24);
    let new_x = lwc1(sp + 16);
    let move_z = new_z - lwc1(a0 + 120);
    let move_x = new_x - lwc1(a0 + 112);
    swc1(a0 + 52, lwc1(sp + 20));
    swc1(sp + 4, 1.0_f32);
    swc1(sp + 8, move_z);
    swc1(a0 + 48, new_x);
    swc1(a0 + 56, new_z);
    swc1(sp, move_x);
    propCalcRooms(a0);
    let room_now = lw(a0 + 12);
    let mut room = bgPortalCrossed(previous, sp, room_now, 0, 0);
    let mut result = 0;
    if room != 0 {
        let mut before = 0;
        loop {
            result = bgPortalCrossed(previous, sp, room, before, 0);
            before = room;
            room = result;
            if result == before {
                break;
            }
        }
    }
    sw(a0 + 12, result);
    updateNPCPadPos(a0);
}
