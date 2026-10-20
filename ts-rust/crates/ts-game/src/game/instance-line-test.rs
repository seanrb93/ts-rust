//! The line test against the model of an object instance.

use crate::*;

/// Test the line (from `from`, along or to `dir`; a segment when `segment` is not 0) against the polygons of the instance `inst`: the line is put in the space of each bone with a
/// collision record (+76 of the bone, restricted by the flags of +68 when set), tested against the box of each chunk of the bone's points (with a quick test of the segment
/// against the box when it is a segment, `HitTestLineBox` otherwise) and then against the chunk's polygons (`hittestLinePoly`); the closest hit is kept (the place, the normal, and
/// which part, chunk and polygon it was in `s_PossibleLastHit*`). 1 is returned for a hit, and `hit` and `normal` (when not 0) get the place and the unit normal in the
/// world's space (@ 0x263768).
pub fn obInstLineTest(inst: i32, from: i32, dir: i32, segment: i32, hit: i32, normal: i32) -> i32 {
    let frame = Frame::new(736);
    let sp = frame.sp();
    sw(sp + 448, inst);
    let skeleton = lw(inst);
    sw(sp + 452, from);
    sw(sp + 472, skeleton);
    let prop = lw(inst + 244);
    sw(sp + 456, dir);
    sw(sp + 480, prop);
    memset(sp, 0, 12);
    sw(sp + 460, segment);
    sw(sp + 464, hit);
    sw(sp + 468, normal);
    sw(sp + 476, lw(sp + 472) - lw(skeleton) * 80);
    let mut best = lwc1(g::tests + 4);
    let second = lw(0x3a2728);
    sd(sp + 16, ld(0x3a2720));
    sw(sp + 24, second);
    let model = lw(sp + 480);
    if lw(sp + 460) == 0 {
        let vector = lw(sp + 456);
        let x = lwc1(vector);
        let y = lwc1(vector + 4);
        let z = lwc1(vector + 8);
        best = x * x + y * y + z * z;
    }
    let limit = best;
    if lw(model + 8) != 2 {
        let turned = lw(sp + 480);
        let matrix;
        if lw(model + 140) != 2 {
            let angle = lwc1(turned + 76) * lwc1(0x3acb98) / 1.8e+02_f32;
            matrixRotY(sp + 64, angle);
            matrix = sp + 64;
        } else {
            let radians = lwc1(0x3acb94);
            let a = lwc1(model + 72) * radians / 1.8e+02_f32;
            let b = lwc1(model + 76) * radians / 1.8e+02_f32;
            let c = lwc1(model + 88) * radians / 1.8e+02_f32;
            matrixRotZXY(sp + 64, a, b, c);
            matrix = sp + 64;
        }
        sceVu0InversMatrix(sp + 128, matrix);
    }
    let mut bone_number = 0;
    let bone_count = lw(lw(sp + 472));
    sw(g::cycles, 0);
    sw(g::tests, 0);
    let mut best = best;
    if bone_count > 0 {
        sw(sp + 492, sp + 128);
        sw(sp + 496, sp + 32);
        let scale = lwc1(0x3acb9c);
        sw(sp + 500, sp + 48);
        sw(sp + 504, sp + 192);
        sw(sp + 508, sp + 256);
        sw(sp + 516, sp + 384);
        let mut bones = lw(sp + 476);
        let mut count = bone_count;
        loop {
            'bone: {
                let bone = bones + bone_number * 80;
                sw(sp + 484, bone);
                let mut boxes = lw(bone + 76);
                let next_bone;
                if boxes == 0 {
                    next_bone = bone_number + 1;
                } else {
                    let mut chunks;
                    if lw(boxes + 24) == 0 {
                        chunks = lw(lw(sp + 484) + 8);
                        sw(sp + 488, lw(sp + 484) + 20);
                    } else {
                        chunks = lw(bone + 12);
                        sw(sp + 488, bone + 44);
                    }
                    let mask = lhu(lw(sp + 484) + 68);
                    let owner = lw(sp + 448);
                    if mask != 0 && (lhu(owner + 292) & mask) == 0 {
                        next_bone = bone_number + 1;
                    } else {
                        let model_now = lw(sp + 480);
                        if chunks == 0 {
                            next_bone = bone_number + 1;
                        } else {
                            let model_type = lw(model_now + 8);
                            let first;
                            if model_type == 2 {
                                let from = lw(sp + 452);
                                let dir = lw(sp + 456);
                                swc1(sp + 32, lwc1(from));
                                swc1(sp + 36, lwc1(from + 4));
                                swc1(sp + 40, lwc1(from + 8));
                                swc1(sp + 48, lwc1(dir));
                                swc1(sp + 52, lwc1(dir + 4));
                                swc1(sp + 56, lwc1(dir + 8));
                                first = lw(chunks + 20);
                            } else {
                                let dir = lw(sp + 456);
                                let from = lw(sp + 452);
                                let dx = lwc1(dir);
                                let fy = lwc1(from + 4);
                                let fz = lwc1(from + 8);
                                let dy = lwc1(dir + 4);
                                swc1(sp + 32, lwc1(lw(sp + 452)) - lwc1(model_now + 48));
                                let dz = lwc1(dir + 8);
                                swc1(sp + 36, fy - lwc1(model_now + 52));
                                let offset_z = lwc1(model_now + 56);
                                swc1(sp + 48, dx);
                                swc1(sp + 52, dy);
                                swc1(sp + 56, dz);
                                swc1(sp + 40, fz - offset_z);
                                matrixVecMulAligned(&[lw(sp + 492) as i64, lw(sp + 496) as i64]);
                                matrixVecMulAligned(&[lw(sp + 492) as i64, lw(sp + 500) as i64]);
                                sceVu0CopyMatrix(lw(sp + 504), lw(lw(sp + 448) + 4) + (lb(lw(sp + 484) + 1) << 6));
                                let y = lwc1(sp + 36) - lwc1(sp + 244);
                                let z = lwc1(sp + 40) - lwc1(sp + 248);
                                swc1(sp + 32, lwc1(sp + 32) - lwc1(sp + 240));
                                sw(sp + 240, 0);
                                swc1(sp + 36, y);
                                sw(sp + 244, 0);
                                swc1(sp + 40, z);
                                sw(sp + 248, 0);
                                sceVu0InversMatrix(lw(sp + 508), lw(sp + 504));
                                matrixVecMulAligned(&[lw(sp + 508) as i64, lw(sp + 496) as i64]);
                                matrixVecMulAligned(&[lw(sp + 508) as i64, lw(sp + 500) as i64]);
                                first = lw(chunks + 20);
                            }
                            let mut chunk_number = 0;
                            if first >= 0 {
                                sw(sp + 524, bone_number + 1);
                                loop {
                                    let following;
                                    let next_chunk;
                                    let kind_after;
                                    'chunk: {
                                        'rejected: {
                                            let flag = lw(sp + 460);
                                            sw(sp + 512, lw(lw(sp + 488) + 4) + lw(chunks + 12) * 12);
                                            if flag == 0 {
                                                let mut axis = 0;
                                                while axis < 3 {
                                                    let start = lwc1(sp + 32 + axis * 4);
                                                    let low = lwc1(boxes + axis * 4);
                                                    let delta = lwc1(sp + 48 + axis * 4);
                                                    if start < low && (start + delta) < low {
                                                        chunks += 24;
                                                        break 'rejected;
                                                    }
                                                    let high = lwc1(boxes + 12 + axis * 4);
                                                    let delta = lwc1(sp + 48 + axis * 4);
                                                    if high < start && high < start + delta {
                                                        chunks += 24;
                                                        break 'rejected;
                                                    }
                                                    axis += 1;
                                                }
                                            }
                                            let touched = HitTestLineBox(sp + 32, sp + 48, boxes, boxes + 12, 0, 0);
                                            if touched == 0 {
                                                chunks += 24;
                                            } else {
                                                let mut polygon = 0;
                                                if lw(boxes + 28) > 0 {
                                                    sw(sp + 532, sp + 352);
                                                    sw(sp + 520, sp + 320);
                                                    chunks += 24;
                                                    sw(sp + 528, sp + 336);
                                                    following = boxes + 36;
                                                    next_chunk = chunk_number + 1;
                                                    loop {
                                                        let points = lw(sp + 512);
                                                        let normal_at = lw(sp + 532);
                                                        let out_hit = lw(sp + 520);
                                                        let out_normal = lw(sp + 528);
                                                        let record = polygon * 12 + lw(boxes + 32);
                                                        let vertices = lw(record + 8);
                                                        swc1(sp + 352, lh(record + 2) as f32 / scale);
                                                        swc1(sp + 356, lh(record + 4) as f32 / scale);
                                                        swc1(sp + 360, lh(record + 6) as f32 / scale);
                                                        let count_here = lbu(record);
                                                        let hit_poly = hittestLinePoly(
                                                            sp + 32,
                                                            sp + 48,
                                                            points,
                                                            vertices,
                                                            count_here,
                                                            normal_at,
                                                            out_hit,
                                                            out_normal,
                                                        );
                                                        sw(g::tests, lw(g::tests) + 1);
                                                        if hit_poly != 0 {
                                                            let hx = lwc1(sp + 320);
                                                            let hy = lwc1(sp + 324);
                                                            let dx = hx - lwc1(sp + 32);
                                                            let hz = lwc1(sp + 328);
                                                            let dy = hy - lwc1(sp + 36);
                                                            let from_z = lwc1(sp + 40);
                                                            swc1(sp + 368, dx);
                                                            let dz = hz - from_z;
                                                            swc1(sp + 372, dy);
                                                            let distance = dx * dx + dy * dy + dz * dz;
                                                            swc1(sp + 376, dz);
                                                            if distance < best {
                                                                best = distance;
                                                                let nx = lwc1(sp + 336);
                                                                let ny = lwc1(sp + 340);
                                                                let nz = lwc1(sp + 344);
                                                                let owner = lw(sp + 448);
                                                                let kind = lw(lw(sp + 480) + 8);
                                                                swc1(sp, hx);
                                                                swc1(sp + 4, hy);
                                                                swc1(sp + 8, hz);
                                                                swc1(sp + 16, nx);
                                                                swc1(sp + 20, ny);
                                                                swc1(sp + 24, nz);
                                                                sw(g::s_pPossibleLastHitInst, owner);
                                                                sw(g::s_PossibleLastHitPart_3aeaa4, bone_number);
                                                                sw(g::s_PossibleLastHitChunk, chunk_number);
                                                                sw(g::s_PossibleLastHitPoly, polygon);
                                                                if kind != 2 {
                                                                    let matrix = lw(sp + 516);
                                                                    sceVu0CopyMatrix(matrix, lw(owner + 4) + (lb(lw(sp + 484) + 1) << 6));
                                                                    matrixVecMulAligned(&[lw(sp + 516) as i64, sp as i64]);
                                                                    let matrix = lw(sp + 516);
                                                                    sw(sp + 432, 0);
                                                                    sw(sp + 436, 0);
                                                                    sw(sp + 440, 0);
                                                                    matrixVecMulAligned(&[matrix as i64, (sp + 16) as i64]);
                                                                }
                                                            }
                                                        }
                                                        polygon += 1;
                                                        if polygon >= lw(boxes + 28) {
                                                            break;
                                                        }
                                                    }
                                                    kind_after = lw(chunks + 20);
                                                    break 'chunk;
                                                }
                                                chunks += 24;
                                            }
                                        }
                                        following = boxes + 36;
                                        next_chunk = chunk_number + 1;
                                        kind_after = lw(chunks + 20);
                                    }
                                    boxes = following;
                                    chunk_number = next_chunk;
                                    if kind_after < 0 {
                                        break;
                                    }
                                }
                                count = lw(lw(sp + 472));
                                break 'bone;
                            }
                            next_bone = bone_number + 1;
                            count = lw(lw(sp + 472));
                        }
                    }
                }
                sw(sp + 524, next_bone);
            }
            bone_number = lw(sp + 524);
            bones = lw(sp + 476);
            if bone_number >= count {
                break;
            }
        }
    }
    let total = lw(g::totaltests) + lw(g::tests);
    sw(g::totalcycles, lw(g::totalcycles) + lw(g::cycles));
    sw(g::totaltests, total);
    if !(best < limit) {
        return 0;
    }
    let hit_out = lw(sp + 464);
    if hit_out != 0 {
        let y = lwc1(sp + 4);
        let z = lwc1(sp + 8);
        let kind = lw(lw(sp + 480) + 8);
        swc1(hit_out, lwc1(sp));
        swc1(hit_out + 4, y);
        swc1(hit_out + 8, z);
        if kind != 2 {
            let hit_now = lw(sp + 464);
            matrixVecMul(sp + 64, hit_now);
            let model = lw(sp + 480);
            let at = lw(sp + 464);
            let y = lwc1(at + 4);
            let z = lwc1(at + 8);
            swc1(at, lwc1(at) + lwc1(model + 48));
            swc1(at + 4, y + lwc1(model + 52));
            swc1(at + 8, z + lwc1(model + 56));
        }
    }
    let nx = lwc1(sp + 16);
    if lw(sp + 468) != 0 {
        let ny = lwc1(sp + 20);
        let nz = lwc1(sp + 24);
        let unit = 1.0_f32 / (nx * nx + ny * ny + nz * nz).abs().sqrt();
        let out = lw(sp + 468);
        let kind = lw(lw(sp + 480) + 8);
        swc1(out + 8, nz * unit);
        swc1(out, nx * unit);
        swc1(out + 4, ny * unit);
        if kind != 2 {
            matrixVecMul(sp + 64, lw(sp + 468));
        }
    }
    1
}
