//! Functions: the line test of a character against its hit shapes, a gun being fired by a computer character, the lights that touch an object instance and the tick of the
//! page of the character sets.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// The matrix of the model (at +4 of `model_data`) that a vertex of the part `part` (the record at `lo`) is moved with, by the kind `kind` of the vertex (0 the parent bone, 1 the bone
/// of the second parent, 2 the part's own bone; other kinds are not moved). A part with no bone kind (byte 0 of the record is 0) always uses its own bone.
fn vertex_matrix(model_data: i32, lo: i32, sp: i32, kind: i32) -> Option<i32> {
    let matrices = lw(model_data + 4);
    if lb(lo) != 0 {
        match kind {
            1 => Some(matrices + (lb(lw(sp + 452) + 1) << 6)),
            0 => Some(matrices + (lb(lw(sp + 448) + 1) << 6)),
            2 => Some(matrices + (lb(lo + 1) << 6)),
            _ => None,
        }
    } else {
        Some(matrices + (lb(lo + 1) << 6))
    }
}

/// Test the line from `a1` along `a2` against the shapes of the character `a0` (@ 0x286700). With the flag 1 of `a7` only the cylinder around the whole character is tested; else
/// the box around it is tested and then every part (spheres and capsules, and, with the flag 2, the polygons of the part). The nearest hit is kept: `a5` gets the place of the
/// hit and `a6` its normal. Returns 1 for a hit.
pub fn chrLineTestChr(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32) -> i32 {
    let frame = Frame::new(704);
    let sp = frame.sp();
    sw(sp + 400, a0);
    let model_data = lw(a0 + 32);
    sw(sp + 412, a7);
    let model = lw(model_data);
    sw(sp + 404, a5);
    sw(sp + 416, model);
    let count_first = lw(model);
    sw(sp + 408, a6);
    sw(sp + 424, 0);
    sw(sp + 420, lw(sp + 416) - count_first * 80);
    memset(sp, 0, 12);
    let mut best = lwc1(g::PrintDebug + 4);
    let flags = lw(sp + 412) & 1;
    sw(sp + 432, lw(lw(sp + 400) + 352));
    let word = lw(0x3a8878);
    sd(sp + 16, ld(0x3a8870));
    sw(sp + 24, word);
    let chr = lw(sp + 400);
    if flags != 0 {
        let y = ps2float::add(lwc1(chr + 52), lwc1(0x3ad1a0));
        let z = lwc1(chr + 56);
        swc1(sp + 224, lwc1(chr + 48));
        swc1(sp + 228, y);
        swc1(sp + 232, z);
        return HitTestLineCylinder(a1, a2, chr + 48, sp + 224, lw(sp + 404), lw(sp + 408), lwc1(0x3ad1a4));
    }
    let mind = lw(sp + 432);
    let x = lwc1(chr + 48);
    let y = lwc1(chr + 52);
    let min_x = ps2float::add(x, lwc1(mind + 3036));
    let z = lwc1(chr + 56);
    swc1(sp + 192, min_x);
    let min_y = ps2float::add(y, lwc1(mind + 3040));
    swc1(sp + 196, min_y);
    let min_z = ps2float::add(z, lwc1(mind + 3044));
    swc1(sp + 200, min_z);
    let max_x = ps2float::add(x, lwc1(mind + 3048));
    swc1(sp + 208, max_x);
    let max_y = ps2float::add(y, lwc1(mind + 3052));
    swc1(sp + 212, max_y);
    let max_z = ps2float::add(z, lwc1(mind + 3056));
    swc1(sp + 216, max_z);
    if a4 == 0 {
        let start_x = lwc1(a1);
        if start_x < min_x {
            if lwc1(a3) < min_x {
                return 0;
            }
        }
        if max_x < start_x {
            if max_x < lwc1(a3) {
                return 0;
            }
        }
        let start_y = lwc1(a1 + 4);
        if start_y < min_y {
            if lwc1(a3 + 4) < min_y {
                return 0;
            }
        }
        if max_y < start_y {
            if max_y < lwc1(a3 + 4) {
                return 0;
            }
        }
        let start_z = lwc1(a1 + 8);
        if start_z < min_z {
            if lwc1(a3 + 8) < min_z {
                return 0;
            }
        }
        if max_z < start_z {
            if max_z < lwc1(a3 + 8) {
                return 0;
            }
        }
    }
    if hittestLineBox(a1, a2, sp + 192, sp + 208) != 0 {
        let chr = lw(sp + 400);
        let turn = ps2float::mul(lwc1(chr + 76), lwc1(0x3ad1a8));
        sw(sp + 436, sp + 32);
        sw(sp + 428, 0);
        let chr = lw(sp + 400);
        let turn = (turn).pdiv(1.8e+02_f32);
        let dy = ps2float::sub(lwc1(a1 + 4), lwc1(chr + 52));
        let end_z = lwc1(a2 + 8);
        let dz = ps2float::sub(lwc1(a1 + 8), lwc1(chr + 56));
        let end_x = lwc1(a2);
        let end_y = lwc1(a2 + 4);
        swc1(sp + 160, ps2float::sub(lwc1(a1), lwc1(chr + 48)));
        swc1(sp + 164, dy);
        swc1(sp + 168, dz);
        swc1(sp + 176, end_x);
        swc1(sp + 180, end_y);
        swc1(sp + 184, end_z);
        matrixRotY(sp + 32, turn);
        sceVu0InversMatrix(sp + 96, lw(sp + 436));
        sw(sp + 440, sp + 160);
        matrixVecMulAligned(&[(sp + 96) as i64, (sp + 160) as i64]);
        sw(sp + 444, sp + 176);
        matrixVecMulAligned(&[(sp + 96) as i64, (sp + 176) as i64]);
        let parts = lw(lw(sp + 416));
        if parts > 0 {
            sw(sp + 460, sp + 256);
            sw(sp + 464, sp + 272);
            sw(sp + 468, sp + 288);
            loop {
                let lo = lw(sp + 420) + lw(sp + 428) * 80;
                let parent = lb(lo + 2);
                let table = lw(sp + 420);
                if parent < 0 {
                    sw(sp + 448, 0);
                } else {
                    sw(sp + 448, table + parent * 80);
                }
                let grand = lb(lo + 3);
                let table = lw(sp + 420);
                if grand < 0 {
                    sw(sp + 452, 0);
                } else {
                    sw(sp + 452, table + grand * 80);
                }
                let polys_at = lo + 20;
                let mind = lw(sp + 432);
                let polys = lw(lo + 8);
                sw(sp + 456, polys_at);
                let shape = mind + (lw(sp + 428) << 5) + 3060;
                'part: {
                    if polys == 0 {
                        sw(sp + 516, lw(sp + 428) + 1);
                        break 'part;
                    }
                    let f1_5 = lwc1(shape + 8);
                    let hit;
                    if lb(shape + 1) != 1 {
                        let matrices = lw(model_data + 4);
                        swc1(sp + 272, f1_5);
                        swc1(sp + 276, lwc1(shape + 12));
                        swc1(sp + 280, lwc1(shape + 16));
                        swc1(sp + 288, lwc1(shape + 20));
                        swc1(sp + 292, lwc1(shape + 24));
                        swc1(sp + 296, lwc1(shape + 28));
                        matrixVecMulAligned(&[(matrices + (lb(shape) << 6)) as i64, lw(sp + 464) as i64]);
                        matrixVecMulAligned(&[(lw(model_data + 4) + (lb(shape) << 6)) as i64, lw(sp + 468) as i64]);
                        hit = HitTestLineCylinder(lw(sp + 440), lw(sp + 444), lw(sp + 464), lw(sp + 468), sp + 224, sp + 240, lwc1(shape + 4));
                    } else {
                        let matrices = lw(model_data + 4);
                        swc1(sp + 256, f1_5);
                        swc1(sp + 260, lwc1(shape + 12));
                        swc1(sp + 264, lwc1(shape + 16));
                        matrixVecMulAligned(&[(matrices + (lb(shape) << 6)) as i64, lw(sp + 460) as i64]);
                        hit = HitTestLineSphere(lw(sp + 440), lw(sp + 444), lw(sp + 460), sp + 224, sp + 240, lwc1(shape + 4));
                    }
                    let flags = lw(sp + 412);
                    if hit == 0 {
                        sw(sp + 516, lw(sp + 428) + 1);
                        break 'part;
                    }
                    let hit_x = lwc1(sp + 224);
                    if (flags & 2) == 0 {
                        let hit_y = lwc1(sp + 228);
                        let dx = ps2float::sub(hit_x, lwc1(sp + 160));
                        let hit_z = lwc1(sp + 232);
                        let dy = ps2float::sub(hit_y, lwc1(sp + 164));
                        let start_z = lwc1(sp + 168);
                        swc1(sp + 304, dx);
                        let dz = ps2float::sub(hit_z, start_z);
                        swc1(sp + 308, dy);
                        let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
                        swc1(sp + 312, dz);
                        if !(distance < best) {
                            sw(sp + 516, lw(sp + 428) + 1);
                        } else {
                            let part_now = lw(sp + 428);
                            best = distance;
                            let nx = lwc1(sp + 240);
                            sw(sp + 424, part_now);
                            let ny = lwc1(sp + 244);
                            let nz = lwc1(sp + 248);
                            swc1(sp, hit_x);
                            swc1(sp + 4, hit_y);
                            swc1(sp + 8, hit_z);
                            swc1(sp + 16, nx);
                            swc1(sp + 20, ny);
                            swc1(sp + 24, nz);
                            sw(sp + 516, part_now + 1);
                        }
                    } else {
                        let part_now = lw(sp + 428);
                        if lw(polys + 20) < 0 {
                            sw(sp + 516, lw(sp + 428) + 1);
                            break 'part;
                        }
                        sw(sp + 516, part_now + 1);
                        let mut entry = polys;
                        let mut list = lw(sp + 456);
                        loop {
                            let table_base = lw(list);
                            sw(sp + 496, entry + 24);
                            let poly = table_base + (lw(entry + 4) << 4);
                            let list_now = lw(sp + 456);
                            let first_vertex = lw(entry + 12);
                            sw(sp + 472, poly);
                            let header = lw(poly);
                            let vertices = lw(list + 4);
                            let mut kinds = lw(list_now + 16) + (first_vertex << 2);
                            sw(sp + 476, 0);
                            sw(sp + 480, header & 0x7fff);
                            let mut at = vertices + first_vertex * 12;
                            sw(sp + 484, header & 0x8000);
                            let mut odd = ((header >> 16) ^ 1) & 1;
                            sw(sp + 488, sp + 304);
                            loop {
                                swc1(sp + 304, lwc1(at));
                                swc1(sp + 308, lwc1(at + 4));
                                swc1(sp + 312, lwc1(at + 8));
                                let mut slot = 2;
                                if let Some(matrix) = vertex_matrix(model_data, lo, sp, lbu(kinds)) {
                                    matrixVecMul(matrix, lw(sp + 488));
                                }
                                let second = at;
                                swc1(sp + 316, lwc1(second + 12));
                                swc1(sp + 320, lwc1(second + 16));
                                swc1(sp + 324, lwc1(second + 20));
                                let next_kind = kinds + 4;
                                if let Some(matrix) = vertex_matrix(model_data, lo, sp, lbu(next_kind)) {
                                    matrixVecMul(matrix, sp + 316);
                                }
                                kinds = next_kind + 4;
                                let vertex_count = lw(sp + 480);
                                let triangles = vertex_count - 2;
                                sw(sp + 492, triangles);
                                if triangles > 0 {
                                    sw(sp + 500, sp + 308);
                                    sw(sp + 504, sp + 312);
                                    sw(sp + 508, sp + 352);
                                    sw(sp + 512, sp + 368);
                                    let mut done = 0;
                                    loop {
                                        let target = sp + 304 + slot * 12;
                                        swc1(target, lwc1(at + 24));
                                        swc1(lw(sp + 500) + slot * 12, lwc1(at + 28));
                                        swc1(lw(sp + 504) + slot * 12, lwc1(at + 32));
                                        if let Some(matrix) = vertex_matrix(model_data, lo, sp, lbu(kinds)) {
                                            matrixVecMul(matrix, target);
                                        }
                                        kinds += 4;
                                        let base = sp + 304;
                                        let next_slot = slot + 1;
                                        let mut third = base;
                                        if slot != 2 {
                                            third = next_slot * 12 + base;
                                        }
                                        let (b, c);
                                        if odd == 0 {
                                            b = slot * 12 + base;
                                            c = if slot == 0 { sp + 328 } else { (slot - 1) * 12 + base };
                                        } else {
                                            b = if slot == 0 { sp + 328 } else { (slot - 1) * 12 + base };
                                            c = slot * 12 + (sp + 304);
                                        }
                                        let tri = hittestLineTri(lw(sp + 440), lw(sp + 444), third, b, c, lw(sp + 508), lw(sp + 512));
                                        let px = lwc1(sp + 352);
                                        if tri != 0 {
                                            let py = lwc1(sp + 356);
                                            let dx = ps2float::sub(px, lwc1(sp + 160));
                                            let pz = lwc1(sp + 360);
                                            let dy = ps2float::sub(py, lwc1(sp + 164));
                                            let start_z = lwc1(sp + 168);
                                            swc1(sp + 384, dx);
                                            let dz = ps2float::sub(pz, start_z);
                                            swc1(sp + 388, dy);
                                            let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
                                            swc1(sp + 392, dz);
                                            if distance < best {
                                                let nx = lwc1(sp + 368);
                                                best = distance;
                                                let ny = lwc1(sp + 372);
                                                let nz = lwc1(sp + 376);
                                                sw(sp + 424, lw(sp + 428));
                                                swc1(sp, px);
                                                swc1(sp + 4, py);
                                                swc1(sp + 8, pz);
                                                swc1(sp + 16, nx);
                                                swc1(sp + 20, ny);
                                                swc1(sp + 24, nz);
                                            }
                                        }
                                        let advanced = next_slot;
                                        done += 1;
                                        slot = if (advanced ^ 3) == 0 { 0 } else { advanced };
                                        at += 12;
                                        odd = 1 - odd;
                                        if done >= lw(sp + 492) {
                                            break;
                                        }
                                    }
                                }
                                at += 24;
                                if lw(sp + 484) != 0 {
                                    sw(sp + 476, 1);
                                } else {
                                    let next_poly = lw(sp + 472) + 16;
                                    sw(sp + 472, next_poly);
                                    let header = lw(next_poly);
                                    sw(sp + 480, header & 0x7fff);
                                    odd = ((header >> 16) ^ 1) & 1;
                                    sw(sp + 484, header & 0x8000);
                                }
                                if lw(sp + 476) != 0 {
                                    break;
                                }
                            }
                            entry = lw(sp + 496);
                            list = lw(sp + 456);
                            if lw(entry + 20) < 0 {
                                break;
                            }
                        }
                    }
                }
                let next = lw(sp + 516);
                sw(sp + 428, next);
                if next >= parts {
                    break;
                }
            }
        }
        let out = lw(sp + 404);
        if best < lwc1(g::PrintDebug + 4) {
            let x = lwc1(sp);
            if out != 0 {
                let y = lwc1(sp + 4);
                let z = lwc1(sp + 8);
                swc1(out, x);
                swc1(out + 4, y);
                swc1(out + 8, z);
                matrixVecMul(lw(sp + 436), out);
                let chr = lw(sp + 400);
                let out = lw(sp + 404);
                let y = lwc1(out + 4);
                let z = lwc1(out + 8);
                swc1(out, ps2float::add(lwc1(out), lwc1(chr + 48)));
                swc1(out + 4, ps2float::add(y, lwc1(chr + 52)));
                swc1(out + 8, ps2float::add(z, lwc1(chr + 56)));
            }
            let nx = lwc1(sp + 16);
            if lw(sp + 408) != 0 {
                let ny = lwc1(sp + 20);
                let nz = lwc1(sp + 24);
                let normal = lw(sp + 408);
                let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(nx, nx), ps2float::mul(ny, ny)), ps2float::mul(nz, nz)).abs().psqrt());
                swc1(normal + 8, ps2float::mul(nz, scale));
                swc1(normal, ps2float::mul(nx, scale));
                swc1(normal + 4, ps2float::mul(ny, scale));
                matrixVecMul(lw(sp + 436), normal);
            }
            sw(g::s_PossibleLastHitPart_3afa30, lw(sp + 424));
            return 1;
        }
    }
    0
}

/// Normalise the vector of 3 floats at `v` using the double precision functions, as the original code does.
fn normalise_double(v: i32) {
    let x = lwc1(v);
    let y = lwc1(v + 4);
    let z = lwc1(v + 8);
    let length = sqrt(fptodp(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z))));
    let inverse = dptofp(dpdiv(0x3ff0000000000000_i64, length));
    let ny = ps2float::mul(lwc1(v + 4), inverse);
    let nz = ps2float::mul(lwc1(v + 8), inverse);
    swc1(v, ps2float::mul(lwc1(v), inverse));
    swc1(v + 4, ny);
    swc1(v + 8, nz);
}

/// A computer character `a0` fires the gun `a1` at its target (@ 0x28f2d0): where the shot is aimed is found by the kind of the target, and then, for a target that no player
/// sees, the damage is given with no bullet; otherwise bullets are made with `gunFire` (the number and the spread of the gun, the spread of the character added) and the sound is
/// started. Close characters (kind 2) use a single bullet with a spread of their own.
pub fn gunChrFire(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(432);
    let sp = frame.sp();
    let mut muzzle = sp + 240;
    memset(muzzle, 0, 12);
    let mind = lw(a0 + 352);
    sw(g::df, 0);
    let target = lw(mind + 2788);
    if target == 0 {
        return;
    }
    let frames = lw(g::updateframesi);
    if lw(a0 + 12) == 0 {
        return;
    }
    if frames <= 0 {
        return;
    }
    let kind;
    'b2: {
        'b3: {
            'b4: {
                let target_kind = lw(target + 8);
                if target_kind == 8 {
                    let x;
                    'b5: {
                        if lw(lw(target + 352)) >= lw(g::lvnumplayers) {
                            let target_flags = lw(target + 16);
                            let target_x = lwc1(target + 48);
                            if (target_flags & 128) == 0 {
                                if (target_flags & 0xf000000) != 0 {
                                    x = target_x;
                                    break 'b5;
                                }
                            }
                            let gun_number = lw(a1);
                            swc1(sp + 32, target_x);
                            let gun = gun_number * 400 + g::guns;
                            swc1(sp + 36, ps2float::add(lwc1(target + 52), lwc1(0x3ad2d8)));
                            let shot = if a2 == 0 { gun + 52 } else { gun + 188 };
                            let bullet_kind = lw(shot);
                            swc1(sp + 40, lwc1(target + 56));
                            let mut bullets = 6;
                            if bullet_kind != 4 {
                                bullets = if (bullet_kind ^ 9) == 0 { 3 } else { 1 };
                            }
                            if lw(g::curgameparms + 72) != 7 {
                                let skill = lbu(lw(mind) + (g::curgameparms + 49));
                                let scale = match skill {
                                    1 => 0.75_f32,
                                    2 => lwc1(0x3ad2dc),
                                    3 => 1.75_f32,
                                    0 => 0.25_f32,
                                    _ => 2.25_f32,
                                };
                                let mut n = 0;
                                if bullets != 0 {
                                    loop {
                                        n += 1;
                                        let damage = ps2float::mul(ps2float::mul(lwc1(shot + 20), scale), ps2float::sub(1.0_f32, lwc1(mind + 2836)));
                                        propDamage(lw(mind + 2788), a0, sp + 32, -1, 0, 0, 0, 0, damage);
                                        sw(g::offscreenshots, lw(g::offscreenshots) + 1);
                                        if n >= bullets {
                                            break;
                                        }
                                    }
                                }
                                let random = random_unit();
                                let amount = ps2float::mul((lw(g::curgameparms + 108) as f32).pdiv(4e+01_f32), 8.0_f32);
                                sw(a1 + 68, ps2float::mul(amount, random) as i32);
                                return;
                            }
                            sw(a1 + 68, 0);
                            return;
                        }
                        x = lwc1(target + 48);
                    }
                    let dx = ps2float::sub(lwc1(a0 + 48), x);
                    let gun_kind = lw(a1);
                    let dz = ps2float::sub(lwc1(a0 + 56), lwc1(target + 56));
                    let angle;
                    'b13: {
                        if gun_kind == 8 || gun_kind == 11 || gun_kind == 13 || gun_kind == 25 || ((gun_kind - 14) as u32) < (5 as u32) {
                            if lwc1(target + 52) < ps2float::sub(lwc1(a0 + 52), lwc1(0x3ad2e0)) {
                                if 25.0_f32 < ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dz, dz)) {
                                    let up = lwc1(0x3ad2e4);
                                    swc1(sp + 32, lwc1(target + 48));
                                    swc1(sp + 36, ps2float::add(lwc1(target + 52), up));
                                    let z = lwc1(target + 56);
                                    swc1(sp + 40, z);
                                    break 'b4;
                                }
                            }
                        }
                        angle = lwc1(target + 76);
                        let _ = ();
                        break 'b13;
                    }
                    matrixTransRotY(sp + 112, lwc1(target + 48), lwc1(target + 52), lwc1(target + 56), ps2float::mul(angle, lwc1(0x3ad2e8)).pdiv(1.8e+02_f32));
                    let data = lw(lw(mind + 2788) + 32);
                    let matrix = lw(data + 4) + (lw(lw(data + 92) + 20) << 6);
                    sceVu0MulMatrix(sp + 48, sp + 112, matrix);
                    let y = ps2float::sub(lwc1(sp + 100), lwc1(0x3ad2ec));
                    let z = lwc1(sp + 104);
                    swc1(sp + 32, lwc1(sp + 96));
                    swc1(sp + 40, z);
                    swc1(sp + 36, y);
                    break 'b3;
                }
                if target_kind < 9 {
                    if target_kind == 1 {
                        let x_off = lwc1(target + 508);
                        swc1(sp + 32, ps2float::add(lwc1(target + 48), x_off));
                        swc1(sp + 36, ps2float::add(lwc1(target + 52), lwc1(target + 512)));
                        swc1(sp + 40, ps2float::add(lwc1(target + 56), lwc1(target + 516)));
                        break 'b3;
                    }
                    kind = lw(mind + 8);
                    break 'b2;
                }
                if target_kind != 256 {
                    kind = lw(mind + 8);
                    break 'b2;
                }
                swc1(sp + 32, lwc1(target + 48));
                swc1(sp + 36, ps2float::add(lwc1(target + 52), 0.5_f32));
                let z = lwc1(target + 56);
                swc1(sp + 40, z);
                break 'b3;
            }
            // the place has been set by the case that left the block 'b4: the height of the target's head and the same z
            let _ = ();
        }
        kind = lw(mind + 8);
    }
    let gun_holder;
    let skeleton;
    if kind != 3 {
        if kind == 2 {
            if (lw(a0 + 16) & 128) != 0 {
                return;
            }
            matrixTransRotY(sp + 112, lwc1(a0 + 48), lwc1(a0 + 52), lwc1(a0 + 56), ps2float::mul(lwc1(a0 + 76), lwc1(0x3ad2f0)).pdiv(1.8e+02_f32));
            let data = lw(a0 + 32);
            let matrix = lw(data + 4) + (lw(lw(data + 92) + 28) << 6);
            sceVu0MulMatrix(sp + 48, sp + 112, matrix);
            let ox = lwc1(sp + 96);
            let oy = lwc1(sp + 100);
            let dx = ps2float::sub(lwc1(sp + 32), ox);
            let oz = lwc1(sp + 104);
            let dy = ps2float::sub(lwc1(sp + 36), oy);
            let tz = lwc1(sp + 40);
            swc1(a1 + 72, ox);
            let dz = ps2float::sub(tz, oz);
            swc1(a1 + 80, oz);
            swc1(a1 + 76, oy);
            swc1(sp + 16, dx);
            swc1(sp + 20, dy);
            swc1(sp + 24, dz);
            normalise_double(sp + 16);
            let r1 = random_unit();
            swc1(mind + 2976, ps2float::sub(ps2float::add(lwc1(sp + 16), ps2float::mul(r1, lwc1(0x3ad2f4))), lwc1(0x3ad2f8)));
            let r2 = random_unit();
            swc1(mind + 2980, ps2float::sub(ps2float::add(lwc1(sp + 20), ps2float::mul(r2, lwc1(0x3ad2fc))), lwc1(0x3ad300)));
            let r3 = random_unit();
            swc1(mind + 2984, ps2float::sub(ps2float::add(lwc1(sp + 24), ps2float::mul(r3, lwc1(0x3ad304))), lwc1(0x3ad308)));
            gunFire(a0, a1, 0, 0, mind + 2976, 2, lw(mind), 1);
            gunFireSfx(a1, 0, a0 + 48);
            return;
        }
        gun_holder = lw(a1 + 64);
        let model = lw(lw(gun_holder + 32));
        skeleton = model - lw(model) * 80;
    } else {
        gun_holder = a0;
        let model = lw(lw(a0 + 32));
        let df = lw(g::df) + 1;
        let first = lw(model);
        sw(g::df, df);
        skeleton = model - first * 80;
        if df >= lb(skeleton + 7) {
            sw(g::df, 0);
        }
    }
    sw(a1 + 68, 0);
    chrMuzzleFlash(a1);
    let index = lw(g::df);
    if index < lb(skeleton + 7) {
        muzzle = lw(skeleton + 72) + index * 12;
    }
    'b20: {
        let mut player = lw(g::currentplayer);
        if (lw(a0 + 16) & 0xf000000) != 0 {
            if lw(lw(gun_holder + 32) + 4) != 0 {
                matrixTransRotY(sp + 112, lwc1(gun_holder + 48), lwc1(gun_holder + 52), lwc1(gun_holder + 56), ps2float::mul(lwc1(gun_holder + 76), lwc1(0x3ad30c)).pdiv(1.8e+02_f32));
                sceVu0MulMatrix(sp + 176, sp + 112, lw(lw(gun_holder + 32) + 4));
                matrixTrans(sp + 112, lwc1(muzzle), lwc1(muzzle + 4), lwc1(muzzle + 8));
                sceVu0MulMatrix(sp + 176, sp + 176, sp + 112);
                let y = lwc1(sp + 228);
                let z = lwc1(sp + 232);
                let dx = lwc1(sp + 208);
                let dy = lwc1(sp + 212);
                let dz = lwc1(sp + 216);
                swc1(a1 + 72, lwc1(sp + 224));
                swc1(a1 + 76, y);
                swc1(a1 + 80, z);
                swc1(sp, dx);
                swc1(sp + 4, dy);
                swc1(sp + 8, dz);
                break 'b20;
            }
            player = lw(g::currentplayer);
        }
        let room = moveFindRoom(mind + 2808, lwc1(0x3ad310), lwc1(player + 132));
        sw(a0 + 12, room);
        if room == 0 {
            return;
        }
        let target = lw(mind + 2788);
        swc1(a1 + 72, lwc1(mind + 2808));
        swc1(a1 + 76, lwc1(mind + 2812));
        swc1(a1 + 80, lwc1(mind + 2816));
        let y = lwc1(a0 + 52);
        let dx = ps2float::sub(lwc1(target + 48), lwc1(a0 + 48));
        let z = lwc1(a0 + 56);
        swc1(sp, dx);
        let dy = ps2float::sub(lwc1(target + 52), y);
        swc1(sp + 4, dy);
        let dz = ps2float::sub(lwc1(target + 56), z);
        swc1(sp + 8, dz);
        normalise_double(sp);
    }
    let dx = ps2float::sub(lwc1(sp + 32), lwc1(a1 + 72));
    let tz = lwc1(sp + 40);
    let dy = ps2float::sub(lwc1(sp + 36), lwc1(a1 + 76));
    let oz = lwc1(a1 + 80);
    swc1(sp + 16, dx);
    let dz = ps2float::sub(tz, oz);
    swc1(sp + 20, dy);
    swc1(sp + 24, dz);
    normalise_double(sp + 16);
    let target = lw(mind + 2788);
    let x = lwc1(a0 + 48);
    let distance = ps2float::sub(x, lwc1(target + 48));
    let squared = ps2float::mul(distance, distance);
    let forward = lwc1(sp);
    'aim: {
        'b22: {
            if curAnim(a0, 449) == 0 {
                if curAnim(a0, 477) == 0 {
                    if curAnim(a0, 478) == 0 {
                        let nx = lwc1(sp + 16);
                        if lwc1(0x3ad318) < ps2float::add(ps2float::add(ps2float::mul(forward, nx), ps2float::mul(lwc1(sp + 4), lwc1(sp + 20))), ps2float::mul(lwc1(sp + 8), lwc1(sp + 24))) {
                            swc1(mind + 2976, nx);
                        } else {
                            if !(ps2float::add(ps2float::add(squared, squared), squared).abs().psqrt() < 2.5_f32) {
                                return;
                            }
                            swc1(mind + 2976, nx);
                        }
                        swc1(mind + 2980, lwc1(sp + 20));
                        swc1(mind + 2984, lwc1(sp + 24));
                        break 'aim;
                    }
                }
            }
            let limit = lwc1(0x3ad314);
            swc1(mind + 2976, forward);
            let up = lwc1(sp + 20);
            swc1(mind + 2980, up);
            swc1(mind + 2984, lwc1(sp + 8));
            if up < limit {
                if curAnim(a0, 477) != 0 {
                    swc1(mind + 2980, limit);
                } else if curAnim(a0, 478) == 0 {
                    break 'b22;
                } else {
                    swc1(mind + 2980, limit);
                }
            }
        }
    }
    let first = lwc1(mind + 2976);
    let gun_number = lw(a1);
    swc1(sp, first);
    swc1(sp + 4, lwc1(mind + 2980));
    let gun = gun_number * 400 + g::guns;
    let shot = gun + 52;
    let bullets = if (lw(shot) ^ 4) == 0 { 3 } else { 1 };
    swc1(sp + 8, lwc1(mind + 2984));
    if bullets != 0 {
        let mut n = 0;
        loop {
            let random = random_unit();
            let spread = ps2float::add(lwc1(shot + 32), lwc1(mind + 2964));
            let a = ps2float::mul(ps2float::sub(ps2float::add(random, random), 1.0_f32), spread);
            let random = random_unit();
            let b = ps2float::mul(ps2float::sub(ps2float::add(random, random), 1.0_f32), ps2float::add(lwc1(shot + 32), lwc1(mind + 2964)));
            let (x, y, z, inverse);
            if n != 0 {
                x = lwc1(sp);
                y = lwc1(sp + 4);
                z = lwc1(sp + 8);
                inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
            } else {
                inverse = 1.0_f32;
                x = lwc1(sp);
                y = lwc1(sp + 4);
                z = lwc1(sp + 8);
            }
            let zs = ps2float::mul(z, inverse);
            let speed = lwc1(shot + 28);
            n += 1;
            let new_x = ps2float::mul(ps2float::add(ps2float::mul(x, inverse), ps2float::mul(a, zs)), speed);
            swc1(sp + 4, ps2float::mul(ps2float::add(ps2float::mul(y, inverse), b), speed));
            swc1(sp, new_x);
            swc1(sp + 8, ps2float::mul(ps2float::sub(zs, ps2float::mul(a, ps2float::mul(new_x, inverse))), speed));
            gunFire(a0, a1, 0, 0, sp, 2, lw(mind), 1);
            sw(g::propershots, lw(g::propershots) + 1);
            if n >= bullets {
                break;
            }
        }
    }
    gunFireSfx(a1, 0, a0 + 48);
}

/// The matrix part of `obInstLightUpdate`: the direction vectors in sp+48, +64 and +80 are normalised, and the matrices of the lights of the instance are made from them.
fn light_matrices(a0: i32, sp: i32, prop: i32, player_record: i32) {
    let x = lwc1(sp + 48);
    let y = lwc1(sp + 52);
    let z = lwc1(sp + 56);
    let x2 = lwc1(sp + 64);
    let y2 = lwc1(sp + 68);
    let z2 = lwc1(sp + 72);
    let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
    swc1(sp + 56, ps2float::mul(z, scale));
    swc1(sp + 48, ps2float::mul(x, scale));
    swc1(sp + 52, ps2float::mul(y, scale));
    let x3 = lwc1(sp + 80);
    let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x2, x2), ps2float::mul(y2, y2)), ps2float::mul(z2, z2)).abs().psqrt());
    let y3 = lwc1(sp + 84);
    let z3 = lwc1(sp + 88);
    swc1(sp + 72, ps2float::mul(z2, scale));
    swc1(sp + 64, ps2float::mul(x2, scale));
    swc1(sp + 68, ps2float::mul(y2, scale));
    let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x3, x3), ps2float::mul(y3, y3)), ps2float::mul(z3, z3)).abs().psqrt());
    let to_radians = lwc1(0x3acb44);
    swc1(sp + 88, ps2float::mul(z3, scale));
    swc1(sp + 80, ps2float::mul(x3, scale));
    swc1(sp + 84, ps2float::mul(y3, scale));
    matrixTransRotY(sp + 160, lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56), ps2float::mul(lwc1(prop + 76), to_radians).pdiv(1.8e+02_f32));
    let model_matrices = lw(lw(a0) + 4);
    let matrices = if 0 >= model_matrices { 1 } else { model_matrices };
    let mut left = matrices;
    if matrices > 0 {
        let mut offset = 0;
        loop {
            let f1 = lwc1(sp + 52);
            let f2 = lwc1(sp + 56);
            let f3 = lwc1(sp + 64);
            let f4 = lwc1(sp + 68);
            let f5 = lwc1(sp + 72);
            let f6 = lwc1(sp + 80);
            let f7 = lwc1(sp + 84);
            let f8 = lwc1(sp + 88);
            let bones = lw(a0 + 4);
            swc1(sp + 432, lwc1(sp + 48));
            swc1(sp + 436, f1);
            swc1(sp + 440, f2);
            swc1(sp + 448, f3);
            swc1(sp + 452, f4);
            swc1(sp + 456, f5);
            swc1(sp + 480, f6);
            swc1(sp + 484, f7);
            swc1(sp + 488, f8);
            if bones != 0 {
                let combined;
                if lw(sp + 508) == 0 {
                    sceVu0MulMatrix(sp + 224, sp + 160, bones + offset);
                    combined = sp + 224;
                } else {
                    let view = lw(player_record + 1772);
                    sceVu0MulMatrix(sp + 352, view, sp + 160);
                    sceVu0MulMatrix(sp + 224, sp + 352, lw(a0 + 4) + offset);
                    combined = sp + 224;
                }
                sceVu0TransposeMatrix(sp + 288, combined);
                matrixVecRotAligned(&[(sp + 288) as i64, (sp + 432) as i64]);
                matrixVecRotAligned(&[(sp + 288) as i64, (sp + 448) as i64]);
                matrixVecRotAligned(&[(sp + 288) as i64, (sp + 480) as i64]);
            }
            left -= 1;
            let out = lw(a0 + 24) + offset;
            offset += 64;
            swc1(out, lwc1(sp + 432));
            swc1(out + 4, lwc1(sp + 448));
            swc1(out + 8, lwc1(sp + 480));
            sw(out + 12, 0);
            swc1(out + 16, lwc1(sp + 436));
            swc1(out + 20, lwc1(sp + 452));
            swc1(out + 24, lwc1(sp + 484));
            sw(out + 28, 0);
            swc1(out + 32, lwc1(sp + 440));
            swc1(out + 36, lwc1(sp + 456));
            swc1(out + 40, lwc1(sp + 488));
            sw(out + 44, 0);
            sw(out + 48, 0);
            sw(out + 52, 0);
            swc1(out + 60, 1.0_f32);
            sw(out + 56, 0);
            if left == 0 {
                break;
            }
        }
    }
}

/// Read one of the two strongest lights (`light`, the record of the light of the scene) into the instance `a0`: the colour and the direction are blended with the earlier ones
/// kept in the instance (from `store`) so that lights fade rather than jump. `slot` is the place of the colour in the stack (sp+48 or sp+64) and `colour` the one of the
/// scaled colour (sp+96 or sp+112); `scale_at` is the address of the blending factor.
fn light_blend(a0: i32, sp: i32, light: i32, store: i32, id_at: i32, slot: i32, colour: i32, blend: i32, position: i32) {
    let f12 = lwc1(light + 16);
    let attenuation = if !(f12 <= 9.0_f32) { (ps2float::sub(2e+01_f32, f12.abs().psqrt())).pdiv(17.0_f32) } else { 1.0_f32 };
    let light_data = lw(light);
    let color_r = lbu(light_data + 143);
    let scale = ps2float::mul(ps2float::mul(attenuation, 0.00390625_f32), ps2float::mul(lbu(light_data + 148) as f32, 0.00390625_f32));
    swc1(sp + 432, (color_r & 255) as f32);
    let color_g = lhu(lw(light) + 142) & 255;
    let g_float = color_g as f32;
    swc1(sp + 436, g_float);
    let color_b = (((lw(lw(light) + 140)) as u32) >> 8) as i32 & 255;
    let b_float = color_b as f32;
    swc1(sp + 440, b_float);
    let previous = lw(a0 + id_at);
    if previous == 0 {
        let data = lw(light);
        let r = lwc1(sp + 432);
        let g = lwc1(sp + 436);
        swc1(a0 + store, lwc1(data + 68));
        swc1(a0 + store + 4, lwc1(data + 72));
        swc1(a0 + store + 8, lwc1(data + 76));
        swc1(sp + slot, lwc1(light + 4));
        swc1(sp + slot + 4, lwc1(light + 8));
        let z = lwc1(light + 12);
        swc1(a0 + position, r);
        swc1(sp + slot + 8, z);
        swc1(a0 + position + 4, g);
        swc1(a0 + position + 8, b_float);
    } else {
        let data = lw(light);
        let sx = lwc1(a0 + store);
        let sy = lwc1(a0 + store + 4);
        let dx = ps2float::sub(lwc1(data + 68), sx);
        let sz = lwc1(a0 + store + 8);
        let pr = lwc1(a0 + position);
        let rate = lwc1(blend);
        swc1(sp + 448, dx);
        let pg = lwc1(a0 + position + 4);
        let dr = ps2float::sub(lwc1(sp + 432), pr);
        let dy = ps2float::sub(lwc1(data + 72), sy);
        let pb = lwc1(a0 + position + 8);
        let nx = ps2float::add(sx, ps2float::mul(dx, rate));
        let dg = ps2float::sub(lwc1(sp + 436), pg);
        let db = ps2float::sub(b_float, pb);
        let eye_x = lwc1(sp + 416);
        swc1(sp + 452, dy);
        let eye_y = lwc1(sp + 420);
        let eye_z = lwc1(sp + 424);
        let dz = ps2float::sub(lwc1(data + 76), sz);
        swc1(a0 + store, nx);
        let ny = ps2float::add(sy, ps2float::mul(dy, rate));
        swc1(sp + 448, dr);
        swc1(sp + 452, dg);
        swc1(sp + 456, db);
        swc1(a0 + store + 4, ny);
        let nz = ps2float::add(sz, ps2float::mul(dz, rate));
        swc1(sp + slot, ps2float::sub(nx, eye_x));
        swc1(sp + slot + 4, ps2float::sub(ny, eye_y));
        swc1(a0 + store + 8, nz);
        swc1(a0 + position, ps2float::add(pr, ps2float::mul(dr, rate)));
        swc1(a0 + position + 4, ps2float::add(pg, ps2float::mul(dg, rate)));
        swc1(sp + slot + 8, ps2float::sub(nz, eye_z));
        swc1(a0 + position + 8, ps2float::add(pb, ps2float::mul(db, rate)));
    }
    let r = ps2float::mul(lwc1(a0 + position), scale);
    let g = ps2float::mul(lwc1(a0 + position + 4), scale);
    let b = ps2float::mul(lwc1(a0 + position + 8), scale);
    swc1(sp + colour, r);
    swc1(sp + colour + 4, g);
    swc1(sp + colour + 8, b);
    sw(a0 + id_at, lw(light));
}

/// Update the lights of the instance `a0` of an object (@ 0x25c160): the two strongest lights near the instance and the closest bullet light are found, blended with the
/// earlier ones, and the matrices of the instance's lights are made. In level 102 (the menu level) the light is made from the angles set for the front end.
pub fn obInstLightUpdate(a0: i32) {
    let frame = Frame::new(704);
    let sp = frame.sp();
    let mut player_record = 0;
    let prop = lw(a0 + 244);
    sw(sp + 508, 0);
    if lw(g::lvlevelnum) != 102 {
        let kind = lw(prop + 8);
        sw(sp + 496, 0);
        sw(sp + 500, 0);
        let number = lw(prop + 12);
        let z;
        if kind != 2048 {
            if ((number - 201) as u32) >= (4 as u32) {
                swc1(sp + 416, lwc1(prop + 48));
                swc1(sp + 420, lwc1(prop + 52));
                z = lwc1(prop + 56);
            } else {
                let players = lw(g::players);
                sw(sp + 508, 1);
                player_record = players + (number * 1820 - 0x594fc);
                swc1(sp + 416, lwc1(player_record + 152));
                swc1(sp + 420, lwc1(player_record + 156));
                z = lwc1(player_record + 160);
            }
        } else {
            let parent = lw(a0 + 4);
            swc1(sp + 416, ps2float::add(lwc1(prop + 48), lwc1(parent + 48)));
            swc1(sp + 420, ps2float::add(lwc1(prop + 52), lwc1(parent + 52)));
            z = ps2float::add(lwc1(prop + 56), lwc1(parent + 56));
        }
        swc1(sp + 424, z);
        if kind == 8 || kind == 0x1000 || kind == 2048 {
            swc1(sp + 420, ps2float::add(lwc1(sp + 420), 1.0_f32));
        }
        bglightGet(a0, sp + 416, sp + 496, sp + 500);
        let bullet = bgBulletGetClosest(a0, sp + 416, sp + 504);
        let mut first = lw(sp + 496);
        let second = lw(sp + 500);
        let id = lw(first);
        if id != 0 && id == lw(a0 + 36) {
            sw(sp + 500, first);
            sw(sp + 496, second);
            first = second;
        }
        let second = lw(sp + 500);
        let id = lw(second);
        if id != 0 && id == lw(a0 + 32) {
            sw(sp + 500, first);
            sw(sp + 496, second);
            first = second;
        }
        if lw(first) == 0 {
            sw(sp + 52, 0);
            swc1(sp + 48, 1.0_f32);
            sw(sp + 56, 0);
            sw(sp + 96, 0);
            sw(sp + 100, 0);
            sw(sp + 104, 0);
        } else {
            light_blend(a0, sp, first, 40, 32, 48, 96, 0x3acb3c, 64);
        }
        let second = lw(sp + 500);
        if lw(second) == 0 {
            sw(sp + 68, 0);
            swc1(sp + 64, 1.0_f32);
            sw(sp + 72, 0);
            sw(sp + 112, 0);
            sw(sp + 116, 0);
            sw(sp + 120, 0);
        } else {
            light_blend(a0, sp, second, 52, 36, 64, 112, 0x3acb40, 76);
        }
        let eye_x = lwc1(sp + 416);
        if bullet == 0 {
            sw(sp + 84, 0);
            swc1(sp + 80, 1.0_f32);
            sw(sp + 88, 0);
            sw(sp + 128, 0);
            sw(sp + 132, 0);
            sw(sp + 136, 0);
        } else {
            let eye_y = lwc1(sp + 420);
            let eye_z = lwc1(sp + 424);
            let strength = lwc1(sp + 504);
            swc1(sp + 80, ps2float::sub(lwc1(bullet + 24), eye_x));
            swc1(sp + 84, ps2float::sub(lwc1(bullet + 28), eye_y));
            swc1(sp + 88, ps2float::sub(lwc1(bullet + 32), eye_z));
            swc1(sp + 128, ps2float::mul(lwc1(bullet + 12), strength));
            swc1(sp + 132, ps2float::mul(lwc1(bullet + 16), strength));
            swc1(sp + 136, ps2float::mul(lwc1(bullet + 20), strength));
        }
        obinstCalcAmbientLight(a0, sp + 144);
    } else {
        let light_set = lw(prop + 576);
        if light_set == -1 {
            return;
        }
        let to_radians = lwc1(0x3acb38);
        let theta = light_set * 12 + g::theta;
        let phi = light_set * 12 + g::phi;
        let a = cosf(ps2float::mul(lwc1(theta), to_radians).pdiv(1.8e+02_f32));
        swc1(sp, ps2float::mul(a, 1e+01_f32));
        let b = sinf(ps2float::mul(lwc1(phi), to_radians).pdiv(1.8e+02_f32));
        swc1(sp + 4, ps2float::mul(b, 1e+01_f32));
        let c = sinf(ps2float::mul(lwc1(theta), to_radians).pdiv(1.8e+02_f32));
        let d = cosf(ps2float::mul(lwc1(phi), to_radians).pdiv(1.8e+02_f32));
        let theta1 = light_set * 12 + (g::theta + 4);
        let phi1 = light_set * 12 + (g::phi + 4);
        swc1(sp + 8, ps2float::mul(ps2float::mul(c, 1e+01_f32), d));
        let e = cosf(ps2float::mul(lwc1(theta1), to_radians).pdiv(1.8e+02_f32));
        swc1(sp + 16, ps2float::mul(e, 1e+01_f32));
        let f = sinf(ps2float::mul(lwc1(phi1), to_radians).pdiv(1.8e+02_f32));
        swc1(sp + 20, ps2float::mul(f, 1e+01_f32));
        let h = sinf(ps2float::mul(lwc1(theta1), to_radians).pdiv(1.8e+02_f32));
        let i = cosf(ps2float::mul(lwc1(phi1), to_radians).pdiv(1.8e+02_f32));
        let theta2 = light_set * 12 + (g::theta + 8);
        let phi2 = light_set * 12 + (g::phi + 8);
        swc1(sp + 24, ps2float::mul(ps2float::mul(h, 1e+01_f32), i));
        let j = cosf(ps2float::mul(lwc1(theta2), to_radians).pdiv(1.8e+02_f32));
        swc1(sp + 32, ps2float::mul(j, 1e+01_f32));
        let k = sinf(ps2float::mul(lwc1(phi2), to_radians).pdiv(1.8e+02_f32));
        swc1(sp + 36, ps2float::mul(k, 1e+01_f32));
        let l = sinf(ps2float::mul(lwc1(theta2), to_radians).pdiv(1.8e+02_f32));
        let m = cosf(ps2float::mul(lwc1(phi2), to_radians).pdiv(1.8e+02_f32));
        let x1 = lwc1(sp);
        let y1 = lwc1(sp + 4);
        let z1 = lwc1(sp + 8);
        swc1(sp + 40, ps2float::mul(ps2float::mul(l, 1e+01_f32), m));
        let x2 = lwc1(sp + 16);
        let y2 = lwc1(sp + 20);
        let z2 = lwc1(sp + 24);
        let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x1, x1), ps2float::mul(y1, y1)), ps2float::mul(z1, z1)).abs().psqrt());
        swc1(sp + 56, ps2float::mul(-z1, scale));
        swc1(sp + 48, ps2float::mul(-x1, scale));
        swc1(sp + 52, ps2float::mul(-y1, scale));
        let x3 = lwc1(sp + 32);
        let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x2, x2), ps2float::mul(y2, y2)), ps2float::mul(z2, z2)).abs().psqrt());
        let y3 = lwc1(sp + 36);
        let z3 = lwc1(sp + 40);
        swc1(sp + 64, ps2float::mul(-x2, scale));
        swc1(sp + 68, ps2float::mul(-y2, scale));
        swc1(sp + 72, ps2float::mul(-z2, scale));
        let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x3, x3), ps2float::mul(y3, y3)), ps2float::mul(z3, z3)).abs().psqrt());
        let glow = sinf(ps2float::mul(ps2float::mul(lwc1(phi), 0.5_f32), to_radians).pdiv(1.8e+02_f32));
        swc1(sp + 88, ps2float::mul(-z3, scale));
        swc1(sp + 80, ps2float::mul(-x3, scale));
        swc1(sp + 84, ps2float::mul(-y3, scale));
        let x4 = lwc1(sp + 80);
        let y4 = lwc1(sp + 84);
        let z4 = lwc1(sp + 88);
        swc1(sp + 104, glow);
        swc1(sp + 96, glow);
        swc1(sp + 100, glow);
        let ambient = lwc1(g::frontambient);
        sw(sp + 112, 0);
        swc1(sp + 152, ambient);
        sw(sp + 116, 0);
        sw(sp + 120, 0);
        sw(sp + 128, 0);
        sw(sp + 132, 0);
        sw(sp + 136, 0);
        swc1(sp + 144, ambient);
        swc1(sp + 148, ambient);
        let scale = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x4, x4), ps2float::mul(y4, y4)), ps2float::mul(z4, z4)).abs().psqrt());
        swc1(sp + 88, ps2float::mul(z4, scale));
        swc1(sp + 80, ps2float::mul(x4, scale));
        swc1(sp + 84, ps2float::mul(y4, scale));
    }
    light_matrices(a0, sp, prop, player_record);
    let out = lw(a0 + 28);
    swc1(out, lwc1(sp + 96));
    swc1(out + 4, lwc1(sp + 100));
    swc1(out + 8, lwc1(sp + 104));
    sw(out + 12, 0);
    swc1(out + 16, lwc1(sp + 112));
    swc1(out + 20, lwc1(sp + 116));
    swc1(out + 24, lwc1(sp + 120));
    sw(out + 28, 0);
    swc1(out + 32, lwc1(sp + 128));
    swc1(out + 36, lwc1(sp + 132));
    swc1(out + 40, lwc1(sp + 136));
    sw(out + 44, 0);
    swc1(out + 48, lwc1(sp + 144));
    swc1(out + 52, lwc1(sp + 148));
    let last = lwc1(sp + 152);
    swc1(out + 60, 1.0_f32);
    swc1(out + 56, last);
}

/// The tick of the page of the character sets (@ 0x2345f8) for the page `a0`: the list of the characters that can be chosen is made, and the keys move the cursor in the list
/// of the 10 characters of the set (state 3) or in the whole list (state 2), pick characters, swap them and change what goes in the 4 columns (state 3 columns 0 to 3).
pub fn chrsets_pageTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let page = g::chrsets_page;
    let mut index = 0;
    let set = lw(g::gv_chrset);
    let builtin = (set < 8) as i32;
    pageSet(lw(lw((lw(g::cur_language) << 2) + g::languages) + 504), page);
    frontfxTick(0);
    let rows = if lw(g::gv_teamplay) != 0 { 1 } else { 2 };
    sw(0x3aff50, 0);
    sw(0x3aff6c, rows);
    loop {
        let id = lbu(index + g::characterorder);
        if chrAvail(id) == 0 {
            index += 1;
        } else {
            let count = lw(0x3aff50);
            let mut found = 0;
            if count > 0 {
                if lw(0x1fc2b48) != id {
                    found += 1;
                    loop {
                        if found >= count {
                            break;
                        }
                        if lw((found << 2) + 0x1fc2b48) == id {
                            break;
                        }
                        found += 1;
                    }
                }
            }
            index += 1;
            if found == count {
                sw(0x3aff50, found + 1);
                sw((found << 2) + 0x1fc2b48, id);
            }
        }
        if (index as u32) >= (64 as u32) {
            break;
        }
    }
    let mut sound = 0;
    'b7: {
        'b8: {
            'b9: {
                'b10: {
                    botsetInit(set);
                    let buttons = lw(lw(g::front) + 12);
                    if (buttons & 8) == 0 {
                        let up = buttons & 4;
                        if up == 0 {
                            if (buttons & 16) == 0 {
                                if (buttons & 32) == 0 {
                                    if (joyNewBut(0) & 64) == 0 {
                                        let pressed = joyNewBut(lw(lw(g::front)));
                                        let mode = lw(0x3aff70);
                                        if (pressed & 16) != 0 {
                                            let row = lw(0x3aff58);
                                            if (mode & 1) == 0 {
                                                if lhu(a0 + 4) == 1 {
                                                    break 'b8;
                                                }
                                                sh(a0 + 4, 1);
                                            } else {
                                                let saved = lw(0x3aff54);
                                                if row != 0 {
                                                    sh(a0 + 4, 3);
                                                    sw(0x3aff5c, saved);
                                                } else {
                                                    let saved = lw(0x3aff54);
                                                    sh(a0 + 4, 2);
                                                    sw(0x3aff68, saved);
                                                }
                                                sw(0x3aff70, mode & -2);
                                                soundStart(142, 0);
                                            }
                                            joyClearAll(0);
                                            break 'b8;
                                        }
                                        let pressed = joyNewBut(0);
                                        let mode = lw(0x3aff70);
                                        if (pressed & 128) == 0 {
                                            let pressed = joyNewBut(0);
                                            let mode = lw(0x3aff70);
                                            if (pressed & 32) != 0 {
                                                if (mode & 1) != 0 {
                                                    break 'b8;
                                                }
                                                let state = lhu(a0 + 4);
                                                let column = lw(0x3aff60);
                                                if state != 3 {
                                                    break 'b8;
                                                }
                                                let cursor = lw(0x3aff5c);
                                                if column == 0 {
                                                    if builtin != 0 {
                                                        break 'b7;
                                                    }
                                                }
                                                if lw((cursor << 4) + 0x1fc2a88) >= 0 {
                                                    soundStart(144, 0);
                                                    let mut n = 0;
                                                    let from = lw(0x3aff5c);
                                                    let offset = from << 4;
                                                    let mut at = 0x1fc2a88;
                                                    loop {
                                                        'b15: {
                                                            let column = lw(0x3aff60);
                                                            if n != from {
                                                                if column == 1 {
                                                                    sw(at + 4, lw(offset + 0x1fc2a8c));
                                                                } else if column >= 2 {
                                                                    if column == 2 {
                                                                        let value = lw(offset + 0x1fc2a90);
                                                                        sw(at + 8, value);
                                                                    } else {
                                                                        if column != 3 {
                                                                            n += 1;
                                                                            break 'b15;
                                                                        }
                                                                        let value = lw(offset + 0x1fc2a94);
                                                                        sw(at + 12, value);
                                                                    }
                                                                } else {
                                                                    if column != 0 {
                                                                        n += 1;
                                                                        break 'b15;
                                                                    }
                                                                    let value = lw(offset + 0x1fc2a88);
                                                                    sw(at, value);
                                                                }
                                                            }
                                                            n += 1;
                                                        }
                                                        at += 16;
                                                        if n >= 10 {
                                                            break 'b8;
                                                        }
                                                    }
                                                }
                                            }
                                            break 'b8;
                                        } else {
                                            if (mode & 1) != 0 {
                                                break 'b8;
                                            }
                                            let state = lhu(a0 + 4);
                                            if state != 3 {
                                                break 'b8;
                                            }
                                            if builtin != 0 {
                                                break 'b7;
                                            }
                                            soundStart(142, 0);
                                            let cursor = lw(0x3aff5c);
                                            if cursor < 9 {
                                                let offset = cursor << 4;
                                                memcpy(offset + 0x1fc2a88, offset + 0x1fc2a98, (9 - cursor) << 4);
                                            }
                                            sw(0x1fc2b18, -1);
                                            break 'b8;
                                        }
                                    } else {
                                        // the select button
                                        let state = lhu(a0 + 4);
                                        let shown = state & 0xffff;
                                        if shown != 1 {
                                            if shown != 3 {
                                                if shown != 2 {
                                                    break 'b8;
                                                }
                                                let mode = lw(0x3aff70);
                                                let scroll = lw(0x3aff64);
                                                let a7;
                                                let a2_7;
                                                let a3_4;
                                                if (mode & 1) == 0 {
                                                    if (scroll + lw(0x3aff68)) >= lw(0x3aff50) {
                                                        break 'b8;
                                                    }
                                                    if builtin != 0 {
                                                        break 'b7;
                                                    }
                                                    soundStart(143, 0);
                                                    let row = lw(0x3aff68);
                                                    let first = lw(0x1fc2a88);
                                                    sw(0x3aff70, lw(0x3aff70) | 1);
                                                    sh(a0 + 4, 3);
                                                    sw(0x3aff54, row);
                                                    sw(0x3aff58, 0);
                                                    sw(0x3aff5c, 0);
                                                    let mut slot = 0;
                                                    if first >= 0 {
                                                        slot = builtin + 1;
                                                        loop {
                                                            sw(0x3aff5c, slot);
                                                            if slot >= 10 {
                                                                break;
                                                            }
                                                            if lw((slot << 4) + 0x1fc2a88) < 0 {
                                                                break;
                                                            }
                                                            slot += 1;
                                                        }
                                                    }
                                                    if slot == 10 {
                                                        sw(0x3aff5c, row);
                                                        break 'b8;
                                                    }
                                                    break 'b8;
                                                }
                                                a2_7 = lw(0x3aff68);
                                                a3_4 = scroll;
                                                let saved = lw(0x3aff54);
                                                if (a2_7 + a3_4) >= lw(0x3aff50) {
                                                    sound = 142;
                                                    break 'b9;
                                                }
                                                sh(a0 + 4, 3);
                                                sw(0x3aff5c, saved);
                                                sw(0x3aff60, 0);
                                                a7 = lw(0x3aff58);
                                                chrsets_place(a0, mode, a7, a2_7, a3_4);
                                                sound = 143;
                                                break 'b9;
                                            } else {
                                                let column = lw(0x3aff60);
                                                let mode = lw(0x3aff70);
                                                if column != 0 {
                                                    let cursor = lw(0x3aff5c);
                                                    if column != 1 {
                                                        if column != 2 {
                                                            if column != 3 {
                                                                break 'b8;
                                                            }
                                                            let at = (lw(0x3aff5c) << 4) + 0x1fc2a94;
                                                            sw(at, lw(at) ^ 1);
                                                            soundStart(140, 0);
                                                            break 'b8;
                                                        }
                                                        let at = (cursor << 4) + 0x1fc2a90;
                                                        let value = (lw(at) + 1).wrapping_rem(5);
                                                        sw(at, value);
                                                        soundStart(140, 0);
                                                        break 'b8;
                                                    }
                                                    let at = (lw(0x3aff5c) << 4) + 0x1fc2a8c;
                                                    let current = lw(at);
                                                    let next = current + 1;
                                                    let value = next - (((if -1 < next { next } else { current + 4 }) >> 2) << 2);
                                                    sw(at, value);
                                                    soundStart(140, 0);
                                                    break 'b8;
                                                }
                                                if (mode & 1) == 0 {
                                                    if builtin != 0 {
                                                        break 'b7;
                                                    }
                                                    let cursor = lw(0x3aff5c);
                                                    sw(0x3aff70, mode | 1);
                                                    sw(0x3aff54, cursor);
                                                    sound = 141;
                                                    sw(0x3aff58, 1);
                                                    break 'b9;
                                                }
                                                let a7 = lw(0x3aff58);
                                                if a7 == 1 {
                                                    let held = lw(0x3aff54);
                                                    let cursor = lw(0x3aff5c);
                                                    if held != cursor {
                                                        let a = (held << 4) + 0x1fc2a88;
                                                        let b = (cursor << 4) + 0x1fc2a88;
                                                        let b_hi = ld(b + 8);
                                                        sd(sp, ld(b));
                                                        sd(sp + 8, b_hi);
                                                        let a_hi = ld(a + 8);
                                                        sd(b, ld(a));
                                                        sd(b + 8, a_hi);
                                                        sw(0x3aff70, mode & -2);
                                                        let saved_hi = ld(sp + 8);
                                                        sd(a, ld(sp));
                                                        sd(a + 8, saved_hi);
                                                        soundStart(141, 0);
                                                        break 'b8;
                                                    }
                                                }
                                                if a7 != 0 {
                                                    sw(0x3aff70, mode & -2);
                                                    break 'b8;
                                                }
                                                let a2_7 = lw(0x3aff68);
                                                let a3_4 = lw(0x3aff64);
                                                chrsets_place(a0, mode, a7, a2_7, a3_4);
                                                sound = 143;
                                                break 'b9;
                                            }
                                        }
                                        // the state 1
                                        if builtin == 0 {
                                            let found;
                                            'b30: {
                                                'b31: {
                                                    soundStart(140, 0);
                                                    let mut ok;
                                                    if lw(0x1fc2a88) >= 0 {
                                                        ok = true;
                                                    } else {
                                                        ok = false;
                                                        let mut n = 1;
                                                        loop {
                                                            if !(n < 10) {
                                                                break 'b31;
                                                            }
                                                            let value = lw((n << 4) + 0x1fc2a88);
                                                            n += 1;
                                                            if value >= 0 {
                                                                ok = true;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    found = ok;
                                                    if found {
                                                        sh(a0 + 4, 3);
                                                        break 'b30;
                                                    }
                                                }
                                                sw(0x3aff68, 0);
                                                sh(a0 + 4, 2);
                                                sw(0x3aff64, 0);
                                            }
                                            sw(0x3aff5c, 0);
                                            sw(0x3aff60, 0);
                                            break 'b8;
                                        }
                                        soundStart(140, 0);
                                        let rows = lw(0x3aff6c);
                                        sh(a0 + 4, 3);
                                        sw(0x3aff60, rows);
                                        sw(0x3aff5c, 0);
                                        break 'b8;
                                    }
                                } else {
                                    let state = lhu(a0 + 4);
                                    let shown = lhu(a0 + 4) & 0xffff;
                                    if shown == 1 {
                                        chrsetInc();
                                        break 'b8;
                                    }
                                    if shown == 3 {
                                        let mode = lw(0x3aff70);
                                        if lw(0x3aff60) < 3 {
                                            if (mode & 1) == 0 {
                                                soundStart(141, 0);
                                                let rows = lw(0x3aff6c);
                                                let column = lw(0x3aff60) + 1;
                                                sw(0x3aff60, column);
                                                if column >= rows {
                                                    break 'b8;
                                                }
                                                sw(0x3aff60, rows);
                                                break 'b8;
                                            }
                                        }
                                    }
                                    sound = 142;
                                    if state != 2 {
                                        break 'b9;
                                    }
                                    soundStart(141, 0);
                                    let row = lw(0x3aff68);
                                    sh(a0 + 4, 3);
                                    sw(0x3aff5c, row);
                                    break 'b8;
                                }
                            } else {
                                let shown = lhu(a0 + 4);
                                if shown == 1 {
                                    chrsetDec();
                                    break 'b8;
                                }
                                if shown != 3 {
                                    break 'b8;
                                }
                                let column = lw(0x3aff60);
                                let previous = column - 1;
                                if column > 0 {
                                    let rows = lw(0x3aff6c);
                                    sw(0x3aff60, previous);
                                    if previous < rows {
                                        sw(0x3aff60, 0);
                                        if up < rows {
                                            if builtin != 0 {
                                                soundStart(142, 0);
                                                sw(0x3aff60, lw(0x3aff6c));
                                                break 'b8;
                                            }
                                        }
                                    }
                                    sound = 141;
                                    break 'b9;
                                }
                                sound = 142;
                                if builtin != 0 {
                                    break 'b9;
                                }
                                soundStart(141, 0);
                                let row = lw(0x3aff5c);
                                sh(a0 + 4, 2);
                                sw(0x3aff68, row);
                                break 'b8;
                            }
                        }
                        // up
                        let shown = lhu(a0 + 4);
                        if shown == 3 {
                            let cursor = lw(0x3aff5c) - 1;
                            sw(0x3aff5c, cursor);
                            if cursor >= 0 {
                                break 'b10;
                            }
                            sw(0x3aff5c, 9);
                            break 'b10;
                        }
                        if shown != 2 {
                            break 'b8;
                        }
                        let row = lw(0x3aff68) - 1;
                        sw(0x3aff68, row);
                        if row >= 0 {
                            break 'b10;
                        }
                        let scroll = lw(0x3aff64);
                        let new_scroll;
                        if scroll <= 0 {
                            let total = lw(0x3aff50);
                            sw(0x3aff68, 9);
                            new_scroll = if total >= 10 { total - 10 } else { 0 };
                        } else {
                            new_scroll = scroll - 1;
                            sw(0x3aff68, 0);
                        }
                        sw(0x3aff64, new_scroll);
                    } else {
                        // down
                        let shown = lhu(a0 + 4);
                        if shown == 3 {
                            let cursor = lw(0x3aff5c) + 1;
                            sw(0x3aff5c, cursor);
                            if cursor < 10 {
                                break 'b10;
                            }
                            sw(0x3aff5c, 0);
                            break 'b10;
                        }
                        if shown != 2 {
                            break 'b8;
                        }
                        let row = lw(0x3aff68) + 1;
                        sw(0x3aff68, row);
                        if row < 10 {
                            break 'b10;
                        }
                        let scroll = lw(0x3aff64);
                        if (lw(0x3aff50) - scroll) < 11 {
                            sw(0x3aff68, 0);
                            sw(0x3aff64, 0);
                            break 'b10;
                        }
                        let new_scroll = scroll + 1;
                        sw(0x3aff68, 9);
                        sw(0x3aff64, new_scroll);
                    }
                }
                sound = 140;
            }
            soundStart(sound, 0);
        }
        if builtin == 0 {
            let mut at = 0x1fc2a88;
            let mut left = 9;
            let mut out = (set - 8) * 10 + 0x1fc2b28;
            loop {
                let value = lbu(at);
                left -= 1;
                at += 16;
                sb(out, value);
                out += 1;
                if left < 0 {
                    break;
                }
            }
        }
    }
    let shown = lhu(a0 + 4);
    let strings = lw((lw(g::cur_language) << 2) + g::languages);
    let mut text = 0;
    sw(g::i0500 + 128, lw(strings + 672));
    let state = shown & 0xffff;
    if state != 1 {
        if state != 2 {
            if state == 3 {
                let column = lw(0x3aff60);
                if (lw(0x3aff70) & 1) == 0 {
                    if column != 0 {
                        if column > 0 {
                            if builtin == 0 {
                                text = lw(strings + 692);
                            } else {
                                text = lw(strings + 688);
                            }
                        }
                    } else if builtin == 0 {
                        text = lw(strings + 684);
                    }
                } else {
                    text = lw(strings + 680);
                    sw(g::i0500 + 128, 0);
                }
            }
        } else {
            text = lw(strings + 676);
        }
    } else {
        text = lw(strings + 1680);
    }
    sw(g::i0500 + 96, text);
    if shown == 1 {
        let front = lw(g::front);
        sw(front + 12, lw(front + 12) & -49);
    }
    menutick_230238(a0, g::chrsets_menu);
    let made = std_menumake(lw(a0 + 8), g::chrsets_menu);
    sw(a0 + 12, made);
}

/// Put the character picked in the list at the cursor of the page of the character sets into its slot (the common end of two ways of picking): the slot before the cursor is
/// searched for an earlier filled one to copy, or the slot is made a copy of the earlier values with the column set to 2.
fn chrsets_place(a0: i32, mode: i32, a7: i32, a2_7: i32, a3_4: i32) {
    let cursor = lw(0x3aff5c);
    'copied: {
        'search: {
            let mut slot = lw(0x3aff5c) - 1;
            if slot >= 0 {
                let mut offset = slot << 4;
                'b27: {
                    if lw(offset + 0x1fc2a88) < 0 {
                        slot -= 1;
                        loop {
                            offset = slot << 4;
                            if slot < 0 {
                                break 'search;
                            }
                            if lw(offset + 0x1fc2a88) >= 0 {
                                break 'b27;
                            }
                            slot -= 1;
                        }
                    }
                }
                if slot >= 0 {
                    let from = offset + 0x1fc2a88;
                    let to = (cursor << 4) + 0x1fc2a88;
                    let hi = ld(from + 8);
                    sd(to, ld(from));
                    sd(to + 8, hi);
                    break 'copied;
                }
            }
        }
        sw((cursor << 4) + 0x1fc2a90, 2);
    }
    let offset = cursor << 4;
    sw(offset + 0x1fc2a88, lw(((a3_4 + a2_7) << 2) + 0x1fc2b48));
    sw(0x3aff70, mode & -2);
    sw(offset + 0x1fc2a94, 1);
    let state = if a7 == 0 { 2 } else { 3 };
    sh(a0 + 4, state);
}
