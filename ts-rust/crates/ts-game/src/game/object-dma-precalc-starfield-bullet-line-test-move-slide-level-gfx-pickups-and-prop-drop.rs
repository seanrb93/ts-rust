//! Functions: precalculating the double buffered drawing data of an object, drawing the starfields, a bullet's straight line test, sliding a character along walls, drawing a
//! level, making a pickup, drawing the weapons page and dropping a prop.

use super::particle_spawners::random_as_float;
use crate::*;

/// Make the double buffered display list data of the parts of the object `a0` (@ 0x25fc90): eight tables of one pointer for each part are allocated (`+252` to `+280`) and for
/// each part with a display list the data of its two lists is built twice (by `partPrecalcDmaData`) into freshly allocated memory so that the two buffers can be flipped
/// between frames.
pub fn obPrecalcDbufGfxData(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let model = lw(a0);
    let count = lw(model);
    let size = ((count << 2) + 15) & -16;
    sw(sp, model - count * 80);
    let table = memAlloc(size, 1);
    let model = lw(a0);
    sw(a0 + 252, table);
    let table = memAlloc(((lw(model) << 2) + 15) & -16, 1);
    let model = lw(a0);
    sw(a0 + 256, table);
    let table = memAlloc(((lw(model) << 2) + 15) & -16, 1);
    let model = lw(a0);
    sw(a0 + 260, table);
    let table = memAlloc(((lw(model) << 2) + 15) & -16, 1);
    let model = lw(a0);
    sw(a0 + 264, table);
    let table = memAlloc(((lw(model) << 2) + 15) & -16, 1);
    let model = lw(a0);
    sw(a0 + 268, table);
    let table = memAlloc(((lw(model) << 2) + 15) & -16, 1);
    let model = lw(a0);
    sw(a0 + 272, table);
    let table = memAlloc(((lw(model) << 2) + 15) & -16, 1);
    let model = lw(a0);
    sw(a0 + 276, table);
    let table = memAlloc(((lw(model) << 2) + 15) & -16, 1);
    let mut model = lw(a0);
    sw(a0 + 284, 1);
    sw(a0 + 280, table);
    if lw(model) > 0 {
        let mut part = 0;
        let mut parts = lw(sp);
        loop {
            let entry = parts + part * 80;
            let first = lw(entry + 8);
            let next;
            let slot;
            if first == 0 {
                slot = part << 2;
                let table_a = lw(a0 + 268);
                next = part + 1;
                let table_b = lw(a0 + 260);
                sw(slot + lw(a0 + 252), 0);
                let table_c = lw(a0 + 276);
                sw(slot + table_a, 0);
                sw(slot + table_b, 0);
                sw(slot + table_c, 0);
            } else {
                let mut chain = first;
                let mut saved = 0;
                let mut slot_value = part << 2;
                'size: {
                    if lw(chain + 20) < 0 {
                        saved = lw(entry + 32);
                    } else {
                        if lw(chain + 44) >= 0 {
                            saved = lw(entry + 32);
                            loop {
                                chain += 24;
                                if lw(chain + 44) < 0 {
                                    break;
                                }
                            }
                            break 'size;
                        }
                        saved = lw(entry + 32);
                        slot_value = part << 2;
                    }
                }
                next = part + 1;
                slot = slot_value;
                let length = lw(chain + 16);
                let current = slot + lw(model + 28);
                let old = lw(current);
                let bytes = (((lw(chain + 12) + length) << 2) + 15) & -16;
                sw(current, 0);
                let memory = memAlloc(bytes, 1);
                sw(entry + 32, memory);
                partPrecalcDmaData(lw(a0), part, 0);
                let swap = slot + lw(lw(a0) + 28);
                let table = slot + lw(a0 + 268);
                sw(slot + lw(a0 + 252), lw(swap));
                sw(table, lw(entry + 32));
                sw(swap, old);
                sw(entry + 32, saved);
                let second = lw(swap);
                sw(swap, 0);
                let memory = memAlloc(bytes, 1);
                sw(entry + 32, memory);
                partPrecalcDmaData(lw(a0), part, 0);
                model = lw(a0);
                let swap = slot + lw(model + 28);
                let table = slot + lw(a0 + 276);
                sw(slot + lw(a0 + 260), lw(swap));
                sw(table, lw(entry + 32));
                sw(swap, second);
                sw(entry + 32, saved);
            }
            let first = lw(entry + 12);
            if first == 0 {
                let table_a = lw(a0 + 272);
                let table_b = lw(a0 + 264);
                sw(slot + lw(a0 + 256), 0);
                let table_c = lw(a0 + 280);
                sw(slot + table_a, 0);
                sw(slot + table_b, 0);
                sw(slot + table_c, 0);
            } else {
                let mut chain = first;
                let saved = lw(entry + 56);
                let length;
                'size2: {
                    if lw(chain + 20) >= 0 {
                        if lw(chain + 44) >= 0 {
                            loop {
                                chain += 24;
                                if lw(chain + 44) < 0 {
                                    break;
                                }
                            }
                            length = lw(chain + 16);
                            break 'size2;
                        }
                    }
                    length = lw(chain + 16);
                }
                let current = slot + lw(model + 32);
                let old = lw(current);
                let bytes = (((lw(chain + 12) + length) << 2) + 15) & -16;
                sw(current, 0);
                let memory = memAlloc(bytes, 1);
                sw(entry + 56, memory);
                partPrecalcDmaData(lw(a0), part, 1);
                let swap = slot + lw(lw(a0) + 32);
                let table = slot + lw(a0 + 272);
                sw(slot + lw(a0 + 256), lw(swap));
                sw(table, lw(entry + 56));
                sw(swap, old);
                sw(entry + 56, saved);
                let second = lw(swap);
                sw(swap, 0);
                let memory = memAlloc(bytes, 1);
                sw(entry + 56, memory);
                partPrecalcDmaData(lw(a0), part, 1);
                model = lw(a0);
                let swap = slot + lw(model + 32);
                let table = slot + lw(a0 + 280);
                sw(slot + lw(a0 + 264), lw(swap));
                sw(table, lw(entry + 56));
                sw(swap, second);
                sw(entry + 56, saved);
            }
            part = next;
            parts = lw(sp);
            if part >= lw(model) {
                break;
            }
        }
    }
}

/// Draw the starfields of the levels (@ 0x2a9068): on the level 17 the stars of the rooms 11, 47 and 16 move by their speeds and fade, falling back to a new random place
/// when they run out; the room 47 also has 50 spacefires. The points and colours are collected on the stack and drawn as particle sprites.
pub fn starfieldGfx() {
    let frame = Frame::new(4256);
    let sp = frame.sp();
    if lw(g::lvlevelnum) != 17 {
        return;
    }
    sw(sp + 4032, 0);
    let mut fires = 0;
    if isroomvisible(11) != 0 {
        let stars = g::stars;
        let reach = 1.5e+02_f32;
        sw(sp + 4032, 1);
        let one = 1.0_f32;
        let source = 0x3a9658;
        let word = lw(source + 8);
        sd(sp + 4016, ld(source));
        sw(sp + 4024, word);
        let mut colour = sp + 2400;
        let mut speed = stars + 20;
        let mut base = stars;
        let mut offset = 0;
        let mut i = 0;
        loop {
            let star = stars + offset;
            let phase_at = offset + (stars + 16);
            let phase = lwc1(phase_at);
            let step = lwc1(g::updateframesf);
            let rgb = lw(star + 24);
            let y = ps2float::add(ps2float::mul(ps2float::mul(ps2float::mul(lwc1(star + 4), lwc1(star + 12)), reach), phase), 0.0_f32);
            swc1(sp + i * 12, ps2float::add(ps2float::mul(ps2float::mul(ps2float::mul(lwc1(base), lwc1(star + 8)), reach), phase), 0.0_f32));
            swc1(sp + 4 + i * 12, y);
            swc1(sp + 8 + i * 12, ps2float::add(ps2float::mul(88.0_f32, phase), -1e+02_f32));
            let moved = ps2float::mul(step, lwc1(speed));
            sw(colour, rgb & 0xffffff | (ps2float::mul(phase, 127.0_f32) as i32) << 24);
            let next = ps2float::add(phase, moved);
            swc1(phase_at, next);
            if one <= next {
                swc1(phase_at, ps2float::sub(next, one));
                setStar(i);
            }
            i += 1;
            colour += 4;
            speed += 28;
            base += 28;
            offset += 28;
            if i >= 200 {
                break;
            }
        }
    }
    if isroomvisible(47) != 0 {
        let stars = g::stars;
        let mut step = lwc1(g::updateframesf);
        sw(sp + 4032, 1);
        let one = 1.0_f32;
        let reach = 1.5e+02_f32;
        let source = 0x3a9668;
        let word = lw(source + 8);
        sd(sp + 4016, ld(source));
        sw(sp + 4024, word);
        let mut colour = sp + 2400;
        let scale = 127.0_f32;
        let mut i = 0;
        loop {
            let phase_at = i * 28 + (stars + 16);
            let star = stars + i * 28;
            let phase = lwc1(phase_at);
            let inverse = ps2float::sub(one, phase);
            let rgb = lw(stars + i * 28 + 24) & 0xffffff;
            let y = ps2float::add(ps2float::mul(ps2float::mul(ps2float::mul(lwc1(star + 4), lwc1(star + 12)), reach), inverse), 0.0_f32);
            swc1(sp + i * 12, ps2float::add(ps2float::mul(ps2float::mul(ps2float::mul(lwc1(i * 28 + stars), lwc1(star + 8)), reach), inverse), 0.0_f32));
            swc1(sp + 4 + i * 12, y);
            swc1(sp + 8 + i * 12, ps2float::add(ps2float::mul(1e+02_f32, phase), 4e+01_f32));
            let moved = ps2float::mul(step, lwc1(i * 28 + (g::stars + 20)));
            sw(colour, rgb | (ps2float::mul(inverse, scale) as i32) << 24);
            let next = ps2float::add(phase, moved);
            swc1(phase_at, next);
            if one <= next {
                swc1(phase_at, ps2float::sub(next, one));
                setStar(i);
                step = lwc1(g::updateframesf);
            }
            i += 1;
            colour += 4;
            if i >= 200 {
                break;
            }
        }
        let mut position = sp + 3200;
        let limit = lwc1(0x3ad648);
        let mut fire = g::spacefires + 12;
        let mut colours = sp + 3808;
        let mut y_at = sp + 3208;
        let mut x_at = sp + 3204;
        fires = 1;
        let mut left = 49;
        loop {
            let z = lwc1(fire - 4);
            let phase = lwc1(fire);
            let inverse = ps2float::sub(1.0_f32, phase);
            let y = ps2float::mul(lwc1(fire - 8), inverse);
            swc1(position, ps2float::mul(lwc1(fire - 12), inverse));
            swc1(x_at, y);
            let alpha = ps2float::mul(inverse, scale) as i32;
            swc1(y_at, ps2float::add(z, ps2float::mul(ps2float::sub(1.3e+02_f32, z), phase)));
            let value;
            if !(phase < limit) {
                value = lw(fire + 8) & 0xffffff | alpha << 24;
            } else {
                value = alpha << 24 | g::tomb_spawninfos_1up + 495;
            }
            sw(colours, value);
            let one = 1.0_f32;
            let next = ps2float::add(lwc1(fire), ps2float::mul(step, lwc1(fire + 4)));
            swc1(fire, next);
            if one <= next {
                swc1(fire, ps2float::sub(next, one));
            }
            fire += 24;
            colours += 4;
            y_at += 12;
            x_at += 12;
            left -= 1;
            position += 12;
            if left < 0 {
                break;
            }
        }
    }
    if isroomvisible(16) != 0 {
        let stars = g::stars;
        let source = 0x3a9678;
        let word = lw(source + 8);
        sd(sp + 4016, ld(source));
        sw(sp + 4024, word);
        sw(sp + 4032, 1);
        let mut colour = sp + 2400;
        let mut i = 0;
        loop {
            let offset = i * 28;
            let phase_at = i * 28 + (stars + 16);
            let phase = lwc1(phase_at);
            let centred = ps2float::sub(phase, 0.5_f32);
            let mut distance = ps2float::add(centred, centred);
            if distance < 0.0_f32 {
                distance = -distance;
            }
            let x = lwc1(offset + stars);
            let one = 1.0_f32;
            let top = lwc1(0x3ad64c);
            let y = ps2float::add(ps2float::mul(lwc1(stars + offset + 4), 1.5e+02_f32), 0.0_f32);
            swc1(sp + i * 12, ps2float::add(ps2float::mul(x, 2e+02_f32), 35.0_f32));
            swc1(sp + 4 + i * 12, y);
            swc1(sp + 8 + i * 12, ps2float::add(ps2float::mul(3e+02_f32, phase), -1.5e+02_f32));
            let next = ps2float::add(phase, ps2float::mul(lwc1(g::updateframesf), lwc1(offset + (stars + 20))));
            sw(colour, lw(stars + offset + 24) & 0xffffff | (ps2float::mul(ps2float::mul(ps2float::sub(top, x), ps2float::sub(one, distance)), 127.0_f32) as i32) << 24);
            swc1(phase_at, next);
            if one <= next {
                swc1(phase_at, ps2float::sub(next, one));
                setStar(i);
            }
            i += 1;
            colour += 4;
            if i >= 200 {
                break;
            }
        }
    }
    if lw(sp + 4032) != 0 {
        let old_zb = dlSetZB(2);
        dlSelectTextureKick(lw(g::starfieldtexnum));
        dlSetBlend(2);
        specialParticlesGfx(sp, sp + 2400, 200, 128, 128);
        if fires != 0 {
            dlSelectTextureKick(lw(g::spacefiretexnum));
            specialParticlesGfx(sp + 3200, sp + 3808, 50, 128, 64);
        }
        dlSetZB(old_zb);
    }
}

/// Line test of a bullet that moves straight (@ 0x289d30): the line from the bullet's old position (or, for a bullet just fired, from the gun) to the new position `a1` is
/// tested against the level; the hit on a prop (`+184`) is accepted when the bullet is heading into it, and the hit position and normal go to `a2` and `a3`. Returns the
/// result of the line test or the prop that was hit.
pub fn bulletStraightLineTest(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) -> i32 {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let mut owner = lw(a0 + 220);
    let owner_flags = lw(owner + 16);
    let s2 = a0 + 100;
    let mut room;
    let mut line_result = 0;
    let mut tail_owner = 0;
    let mut hit_prop = 0;
    'result: {
        'tail: {
            if lw(a0 + 216) == 0 {
                room = lw(a0 + 100);
            } else {
                let source = lw(a0 + 104);
                let mut hidden = 0;
                let start_room;
                sw(sp + 64, 0);
                sw(a0 + 216, 0);
                'start: {
                    if source != 1 {
                        if source != 2 {
                            if source == 3 {
                                remoteGetPivotePos(owner, sp + 16);
                                owner = lw(a0 + 220);
                                let remote = lw(owner + 352);
                                hidden = lw(remote + 96);
                                start_room = lw(remote + 120);
                                let flags = lw(hidden + 16);
                                sw(sp + 64, flags);
                                sw(hidden + 16, flags & -33);
                                break 'start;
                            }
                            let x = lwc1(owner + 48);
                            start_room = lw(owner + 12);
                            swc1(sp + 16, x);
                            swc1(sp + 20, lwc1(owner + 52));
                            let z = lwc1(owner + 56);
                            swc1(sp + 24, z);
                        } else {
                            let player = lw(owner + 352);
                            start_room = lw(owner + 12);
                            swc1(sp + 16, lwc1(player + 2808));
                            swc1(sp + 20, lwc1(player + 2812));
                            let z = lwc1(player + 2816);
                            swc1(sp + 24, z);
                        }
                    } else {
                        let chr = lw(owner + 356);
                        start_room = lw(chr + 796);
                        swc1(sp + 16, lwc1(chr + 152));
                        swc1(sp + 20, lwc1(chr + 156));
                        swc1(sp + 24, lwc1(chr + 160));
                    }
                }
                let dy = ps2float::sub(lwc1(a0 + 28), lwc1(sp + 20));
                let flags = lw(owner + 16);
                let dz = ps2float::sub(lwc1(a0 + 32), lwc1(sp + 24));
                swc1(sp + 32, ps2float::sub(lwc1(a0 + 24), lwc1(sp + 16)));
                swc1(sp + 36, dy);
                sw(owner + 16, flags & -33);
                swc1(sp + 40, dz);
                sw(sp, 0);
                sw(sp + 8, a4);
                let hit = bgLineTest(start_room, sp + 16, sp + 32, 0, a2, a3, s2, 48);
                line_result = hit;
                bulletInitialiseStraightLineTest(a0);
                let saved = lw(sp + 64);
                if hidden != 0 {
                    sw(hidden + 16, saved);
                }
                if line_result != 0 {
                    tail_owner = lw(a0 + 220);
                    break 'tail;
                }
                if a4 == 0 {
                    room = lw(a0 + 100);
                } else {
                    let glass = lw(a4);
                    if glass == 0 {
                        room = lw(a0 + 100);
                    } else {
                        bulletProcessGlasshits(a0, glass);
                        room = lw(a0 + 100);
                    }
                }
            }
            'second: {
                sw(sp + 8, a4);
                sw(sp, 0);
                let hit = bgLineTest(room, a0 + 24, a1, 0, a2, a3, s2, 114);
                let prop = lw(a0 + 184);
                line_result = hit;
                if prop != 0 {
                    let dx = lwc1(a1);
                    let dy = lwc1(a1 + 4);
                    let dz = lwc1(a1 + 8);
                    let hx = lwc1(a0 + 160);
                    let hy = lwc1(a0 + 164);
                    let ax = ps2float::sub(hx, ps2float::add(lwc1(a0 + 24), dx));
                    let ay = ps2float::sub(hy, ps2float::add(lwc1(a0 + 28), dy));
                    let hz = lwc1(a0 + 168);
                    let bz = ps2float::add(lwc1(a0 + 32), dz);
                    swc1(sp + 48, ax);
                    swc1(sp + 52, ay);
                    let az = ps2float::sub(hz, bz);
                    swc1(sp + 56, az);
                    if ps2float::add(ps2float::add(ps2float::mul(dx, ax), ps2float::mul(dy, ay)), ps2float::mul(dz, az)) <= 0.0_f32 {
                        let hit_inst;
                        if line_result == 0 {
                            hit_inst = lw(a0 + 188);
                        } else {
                            let bx = ps2float::sub(lwc1(a2), hx);
                            let by = ps2float::sub(lwc1(a2 + 4), hy);
                            let bz = ps2float::sub(lwc1(a2 + 8), hz);
                            swc1(sp + 48, bx);
                            swc1(sp + 52, by);
                            swc1(sp + 56, bz);
                            if ps2float::add(ps2float::add(ps2float::mul(dx, bx), ps2float::mul(dy, by)), ps2float::mul(dz, bz)) <= 0.0_f32 {
                                break 'second;
                            }
                            hit_inst = lw(a0 + 188);
                        }
                        obSetLastHitInst(hit_inst, lw(a0 + 192), lw(a0 + 196), lw(a0 + 200));
                        let owner = lw(a0 + 220);
                        swc1(a2, lwc1(a0 + 160));
                        sw(owner + 16, owner_flags);
                        swc1(a2 + 4, lwc1(a0 + 164));
                        swc1(a2 + 8, lwc1(a0 + 168));
                        swc1(a3, lwc1(a0 + 172));
                        swc1(a3 + 4, lwc1(a0 + 176));
                        swc1(a3 + 8, lwc1(a0 + 180));
                        hit_prop = prop;
                        break 'result;
                    }
                }
                if room == lw(a0 + 100) {
                    tail_owner = lw(a0 + 220);
                    break 'tail;
                }
                bulletInitialiseStraightLineTest(a0);
                let prop = lw(a0 + 184);
                if prop == 0 {
                    tail_owner = lw(a0 + 220);
                    break 'tail;
                }
                let dx = lwc1(a1);
                let dy = lwc1(a1 + 4);
                let dz = lwc1(a1 + 8);
                let hx = lwc1(a0 + 160);
                let hy = lwc1(a0 + 164);
                let ax = ps2float::sub(hx, ps2float::add(lwc1(a0 + 24), dx));
                let ay = ps2float::sub(hy, ps2float::add(lwc1(a0 + 28), dy));
                let hz = lwc1(a0 + 168);
                let bz = ps2float::add(lwc1(a0 + 32), dz);
                swc1(sp + 48, ax);
                swc1(sp + 52, ay);
                let az = ps2float::sub(hz, bz);
                swc1(sp + 56, az);
                if ps2float::add(ps2float::add(ps2float::mul(dx, ax), ps2float::mul(dy, ay)), ps2float::mul(dz, az)) <= 0.0_f32 {
                    let hit_inst;
                    if line_result == 0 {
                        hit_inst = lw(a0 + 188);
                    } else {
                        let bx = ps2float::sub(lwc1(a2), hx);
                        let by = ps2float::sub(lwc1(a2 + 4), hy);
                        let bz = ps2float::sub(lwc1(a2 + 8), hz);
                        swc1(sp + 48, bx);
                        swc1(sp + 52, by);
                        swc1(sp + 56, bz);
                        if ps2float::add(ps2float::add(ps2float::mul(dx, bx), ps2float::mul(dy, by)), ps2float::mul(dz, bz)) <= 0.0_f32 {
                            break 'second;
                        }
                        hit_inst = lw(a0 + 188);
                    }
                    obSetLastHitInst(hit_inst, lw(a0 + 192), lw(a0 + 196), lw(a0 + 200));
                    let owner = lw(a0 + 220);
                    swc1(a2, lwc1(a0 + 160));
                    let marker = lw(a0 + 96);
                    let height = lwc1(a0 + 164);
                    sw(owner + 16, owner_flags);
                    swc1(a2 + 4, height);
                    swc1(a2 + 8, lwc1(a0 + 168));
                    swc1(a3, lwc1(a0 + 172));
                    swc1(a3 + 4, lwc1(a0 + 176));
                    swc1(a3 + 8, lwc1(a0 + 180));
                    if marker != 0 {
                        sw(marker + 12, lw(a0 + 100));
                    }
                    return prop;
                }
            }
            tail_owner = lw(a0 + 220);
        }
        let marker = lw(a0 + 96);
        sw(tail_owner + 16, owner_flags);
        if marker != 0 {
            sw(marker + 12, lw(a0 + 100));
        }
        return line_result;
    }
    hit_prop
}

/// Slide the character `a0` from the position `a1` towards the target `a2` along the walls (@ 0x27db10). The move is tried with `moveTest`; when it was blocked by a wall the
/// remaining move is projected along the wall and tried again (and once more along the direction of the second wall). For a player a move that ends close enough to
/// where it started is cancelled and the floor the character stood on is kept.
pub fn moveTestDoSlide(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, f12: f32, f13: f32) -> i32 {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let target_x = lwc1(a2);
    let target_z = lwc1(a2 + 8);
    let chr = lw(g::chrinfo) + a0 * 0x1210;
    let dx = ps2float::sub(target_x, lwc1(a1));
    let target_y = lwc1(a2 + 4);
    let dz = ps2float::sub(target_z, lwc1(a1 + 8));
    let floor_y = lwc1(chr + 0x1198);
    sw(sp + 128, lw(chr + 0x119c));
    swc1(sp + 48, dx);
    swc1(sp + 52, dz);
    swc1(sp + 64, target_x);
    swc1(sp + 72, target_z);
    swc1(sp + 68, target_y);
    sw(sp + 132, 0);
    let mut result = moveTest(a0, a1, a2, sp, a3, a4, f12, f13);
    sw(sp + 140, 0);
    let old_floor = lw(chr + 0x11b8);
    let is_player = (a0 < lw(g::lvnumplayers)) as i32;
    sw(sp + 136, lw(chr + 0x11bc));
    if is_player != 0 {
        let mut x;
        let mut z;
        let mut blocked_bit;
        'slide: {
            x = lwc1(sp);
            blocked_bit = 0;
            z = lwc1(sp + 8);
            if (result & 2) != 0 {
                let tx = lwc1(a2);
                let sx = x;
                let tz;
                let sz;
                if !(tx == x) {
                    tz = lwc1(a2 + 8);
                    sz = lwc1(sp + 8);
                    blocked_bit = result & 4;
                } else {
                    tz = lwc1(a2 + 8);
                    sz = lwc1(sp + 8);
                    blocked_bit = result & 4;
                    if tz == sz {
                        break 'slide;
                    }
                }
                let wall_x = lwc1(0x3b0118);
                if blocked_bit != 0 {
                    let wall_z = lwc1(0x3b011c);
                    sw(sp + 132, 1);
                    swc1(sp + 32, wall_x);
                    swc1(sp + 36, wall_z);
                }
                let ox = ps2float::sub(sx, tx);
                let oz = sz - tz;
                let ny = lwc1(sp + 52);
                let along = ps2float::mul(lwc1(sp + 48), ox);
                swc1(sp + 80, ox);
                swc1(sp + 84, oz);
                if !(0.0_f32 < ps2float::add(along, ps2float::mul(ny, oz))) {
                    break 'slide;
                }
                result = moveTest(a0, a2, sp, sp + 16, a3, a4, f12, f13);
                x = lwc1(sp);
                if (result & 2) != 0 {
                    let slide_x = lwc1(sp + 16);
                    let sx = x;
                    let sz2;
                    let slide_z;
                    let delta_x;
                    if !(x == slide_x) {
                        sz2 = lwc1(sp + 8);
                        slide_z = lwc1(sp + 24);
                        delta_x = ps2float::sub(slide_x, sx);
                    } else {
                        sz2 = lwc1(sp + 8);
                        slide_z = lwc1(sp + 24);
                        blocked_bit = result & 4;
                        if sz2 == slide_z {
                            z = sz2;
                            break 'slide;
                        }
                        delta_x = ps2float::sub(slide_x, sx);
                    }
                    let delta_z = slide_z - sz2;
                    let nz = lwc1(sp + 52);
                    let along = ps2float::mul(lwc1(sp + 48), delta_x);
                    swc1(sp + 96, delta_x);
                    swc1(sp + 100, delta_z);
                    if 0.0_f32 < ps2float::add(along, ps2float::mul(nz, delta_z)) {
                        let length = (delta_x * delta_x + delta_z * delta_z).abs().psqrt();
                        let inverse = (1.0_f32).pdiv(length);
                        let ux = lwc1(sp + 96);
                        let rx = ps2float::sub(lwc1(sp + 64), x);
                        let uz = lwc1(sp + 100);
                        let rz = ps2float::sub(lwc1(sp + 72), sz2);
                        swc1(sp + 112, rx);
                        let nx = ps2float::mul(ux, inverse);
                        swc1(sp + 116, rz);
                        let nzv = ps2float::mul(uz, inverse);
                        swc1(sp + 96, nx);
                        let dot = ps2float::add(ps2float::mul(rx, nx), ps2float::mul(rz, nzv));
                        swc1(sp + 100, nzv);
                        if 0.0_f32 < dot {
                            swc1(sp + 24, ps2float::add(sz2, ps2float::mul(dot, nzv)));
                            sw(sp + 140, 1);
                            swc1(sp + 16, ps2float::add(x, ps2float::mul(dot, nx)));
                            result = moveTest(a0, sp, sp + 16, 0, a3, a4, f12, f13);
                            let new_x = lwc1(sp + 16);
                            let new_z = lwc1(sp + 24);
                            blocked_bit = result & 4;
                            let new_y = lwc1(sp + 20);
                            x = new_x;
                            z = new_z;
                            swc1(sp, new_x);
                            swc1(sp + 4, new_y);
                            swc1(sp + 8, new_z);
                            break 'slide;
                        }
                    }
                    blocked_bit = result & 4;
                    z = lwc1(sp + 8);
                    break 'slide;
                }
            }
            x = lwc1(sp);
            blocked_bit = result & 4;
            z = lwc1(sp + 8);
        }
        let y = lwc1(sp + 4);
        swc1(a2, x);
        swc1(a2 + 8, z);
        swc1(a2 + 4, y);
        let start_x = lwc1(a1);
        let moved_x = ps2float::sub(start_x, x);
        let moved_z = ps2float::sub(lwc1(a1 + 8), z);
        let limit;
        let squared;
        'limit: {
            if blocked_bit != 0 {
                let wall_x = lwc1(0x3b0118);
                if lw(sp + 132) != 0 {
                    let tight = lwc1(0x3acfb8);
                    if ps2float::add(ps2float::mul(lwc1(sp + 32), wall_x), ps2float::mul(lwc1(sp + 36), lwc1(0x3b011c))) < lwc1(0x3acfb4) {
                        limit = tight;
                        squared = ps2float::mul(moved_x, moved_x);
                        break 'limit;
                    }
                }
            }
            let mut loose = lwc1(0x3acfbc);
            squared = ps2float::mul(moved_x, moved_x);
            if lw(sp + 140) != 0 {
                loose = lwc1(0x3acfc0);
            }
            limit = loose;
        }
        if ps2float::add(squared, ps2float::mul(moved_z, moved_z)) < limit {
            swc1(a2, start_x);
            swc1(a2 + 4, lwc1(a1 + 4));
            swc1(a2 + 8, lwc1(a1 + 8));
            let floor = lw(sp + 128);
            if floor != 0 {
                swc1(chr + 0x1198, floor_y);
                swc1(a2 + 4, floor_y);
                sw(chr + 0x119c, floor);
                if a3 != 0 {
                    sw(a3, floor);
                }
                if !(lwc1(a2 + 4) < target_y) {
                    result |= 1;
                } else {
                    swc1(a2 + 4, target_y);
                }
            }
        }
    } else {
        'again: {
            let x = lwc1(sp);
            if (result & 2) != 0 {
                let z = lwc1(sp + 8);
                if lwc1(a2) == x {
                    if lwc1(a2 + 8) == z {
                        break 'again;
                    }
                }
                result = moveTest(a0, a2, sp, 0, a3, a4, f12, f13);
            }
        }
        let x = lwc1(sp);
        let z = lwc1(sp + 8);
        let y = lwc1(sp + 4);
        swc1(a2 + 8, z);
        swc1(a2, x);
        swc1(a2 + 4, y);
    }
    if old_floor != 0 {
        if lw(chr + 0x11b8) == 0 {
            let value = lw(sp + 136);
            sw(chr + 0x11b8, old_floor);
            sw(chr + 0x11bc, value);
        }
    }
    result
}

/// Draw one level (@ 0x2266f8). The story front end levels (101 to 105) draw only their own page: the character select and mission pages, the map maker, the loading
/// screen and the credits. In a real level the window is set, the props the camera may see through (the player's gun, attached props) are hidden while the rooms are
/// drawn, the view matrix is put in the display list and the sky, weather, bullets, particles, effects and the debug views are drawn followed by the hud, menu and front end.
pub fn lvGfx() {
    let frame = Frame::new(48);
    let sp = frame.sp();
    propUpdateRooms();
    propGfxRoomPreproc();
    frontSet(lw(lw(g::currentplayer)));
    let mut level = lw(g::lvlevelnum);
    if lw(lw(g::currentplayer)) == 0 {
        guiFlashAlpha();
        level = lw(g::lvlevelnum);
    }
    let mode = level - 101;
    if (mode as u32) >= (5 as u32) {
        let player = lw(g::currentplayer);
        windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
        windowGfx();
        dlSetClipTag(1);
        let player = lw(g::currentplayer);
        let first = lw(lw(lw(player + 384) + 352) + 332);
        sw(sp + 16, first);
        let second = lw(lw(lw(player + 384) + 352) + 440);
        sw(sp + 20, second);
        let mut first_flags = 0;
        if first != 0 {
            first_flags = lw(first + 16);
        }
        sw(sp + 4, first_flags);
        let mut second_flags = 0;
        if second != 0 {
            second_flags = lw(second + 16);
        }
        sw(sp + 8, second_flags);
        let camera = camGetMode();
        let player = lw(g::currentplayer);
        if camera != 2 {
            bulletMakeAttachedMinesInvisible(lw(player + 384));
            let prop = lw(sp + 16);
            if prop != 0 {
                sw(prop + 16, lw(prop + 16) & -2);
            }
            let prop = lw(sp + 20);
            let player = lw(g::currentplayer);
            if prop != 0 {
                sw(prop + 16, lw(prop + 16) & -2);
            }
            let player = if prop != 0 { lw(g::currentplayer) } else { player };
            let data = lw(lw(player + 384) + 352);
            let attached = lw(data + 484);
            let held;
            if attached == 0 {
                held = lw(data + 440);
            } else {
                sw(attached + 16, lw(attached + 16) & -2);
                held = lw(lw(lw(player + 384) + 352) + 440);
            }
            if held != 0 {
                sw(held + 16, lw(held + 16) & -2);
            }
            let body = lw(player + 384);
            let prop = lw(lw(body + 352) + 332);
            if prop != 0 {
                sw(prop + 16, lw(prop + 16) & -2);
            }
        }
        bgGfx();
        let prop = lw(sp + 16);
        let flags = lw(sp + 4);
        if prop != 0 {
            sw(prop + 16, flags);
        }
        let prop = lw(sp + 20);
        let flags = lw(sp + 8);
        if prop != 0 {
            sw(prop + 16, flags);
        }
        let player = lw(g::currentplayer);
        let attached = lw(lw(lw(player + 384) + 352) + 484);
        if attached != 0 {
            sw(attached + 16, lw(attached + 16) | 1);
        }
        let state = lw(player + 296);
        if state != 4 {
            if camGetMode() != 2 {
                gunGfx();
            }
        }
        let packet = memdbAlloc(80);
        sb(lw(g::gfxtag) + 3, 48);
        sw(packet, 0x1000404);
        let tag = lw(g::gfxtag);
        let player = lw(g::currentplayer);
        sw(tag + 4, packet);
        sw(packet + 4, 0x6c048004);
        let matrix = lw(player + 1764);
        sh(tag, 5);
        let word = lw(matrix);
        sw(g::gfxtag, tag + 16);
        sw(packet + 8, word);
        let mut k = 1;
        while k < 16 {
            sw(packet + 8 + k * 4, lw(matrix + k * 4));
            k += 1;
        }
        sw(packet + 76, 0x11000000);
        sw(packet + 72, 0x140007a8);
        pickupposGfx();
        starfieldGfx();
        weatherGfx();
        bulletGfx();
        particleGfx();
        specialfxGfx();
        zbtestGfx();
        if lw(g::viewfloors) != 0 {
            bgFloorGfx();
        }
        if lw(g::viewwalls) != 0 {
            bgWallGfx();
        }
        if lw(g::viewvolumes) != 0 {
            bgVolumeGfx();
        }
        if lw(g::viewportals) != 0 {
            bgPortalGfx();
        }
        if lw(g::viewportalrects) != 0 {
            bgPortalRectGfx();
        }
        padGfx();
        propEditGfx();
        ipGfx();
        playerHitPosGfx();
        hudGfx();
        menuGfx();
        frontGfx();
    } else {
        match mode {
            0 | 1 => {
                let player = lw(g::currentplayer);
                windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
                windowGfx();
                dlSetClipTag(1);
                frontGfx();
            }
            2 => {
                let player = lw(g::currentplayer);
                windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
                windowGfx();
                dlSetClipTag(1);
                mapmakerGfx();
                frontGfx();
            }
            3 => {
                loadingGfx();
            }
            _ => {
                let player = lw(g::currentplayer);
                windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
                windowGfx();
                dlSetClipTag(1);
                creditsGfx();
            }
        }
    }
}

/// Make a pickup prop of the kind `a0` (@ 0x2730e0). The pickup's data comes from the level's list of pickups, or, when `a6` is set (a pickup that was dropped), from the
/// 20 slots of temporary pickups (the oldest dropped ones are removed so that only three stay marked). The position is given by `f13` to `f15`, `a1` is the pickup's kind and the
/// remaining parameters set its fields.
pub fn propNewPickup(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, f12: f32, f13: f32, f14: f32, f15: f32, f16: f32, f17: f32, f18: f32, f19: f32) -> i32 {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let mut spin = f18;
    let extra = pickupGetExtraHeight(a0, a1);
    let prop = propNew(a0, f13, ps2float::add(ps2float::add(f14, f19), extra), f15, 0.0_f32);
    let data;
    let mut slot = 0;
    'made: {
        if a6 == 0 {
            let count = lw(g::numpickups);
            if count >= 120 {
                propDelete(prop);
                return 0;
            }
            slot = -1;
            let entry = lw(g::levelpickups) + count * 120;
            sw(prop + 352, entry);
            data = entry;
            sw(g::numpickups, count + 1);
        } else {
            let mut free = 0;
            let mut used = g::tmpPickupDataUsed;
            let mut left = 19;
            loop {
                let value = lw(used);
                used += 4;
                left -= 1;
                free = if value == 0 { free + 1 } else { free };
                if left < 0 {
                    break;
                }
            }
            if free == 0 {
                let mut oldest_frame = lw(g::gameframe);
                let mut frames = g::tmpPickupDataFramenum;
                let mut oldest = 0;
                let mut i = 0;
                loop {
                    let value = lw(frames);
                    if value >= oldest_frame {
                        i += 1;
                    } else {
                        oldest_frame = value;
                        oldest = i;
                        i += 1;
                    }
                    frames += 4;
                    if i >= 20 {
                        break;
                    }
                }
                pickupPropDelete(lw((oldest << 2) + g::tmpPickupDataProp));
                free = 1;
            }
            let mut i = 0;
            let mut used = g::tmpPickupDataUsed;
            let mut marker = used;
            'slot: {
                let mut offset;
                loop {
                    offset = i << 2;
                    let at = marker;
                    if lw(used) == 0 {
                        slot = i;
                        marker = at;
                        break;
                    }
                    i += 1;
                    marker = at + 4;
                    used += 4;
                    if i >= 20 {
                        break 'slot;
                    }
                }
                let frame_number = lw(g::gameframe);
                sw(marker, 1);
                sw(offset + g::tmpPickupDataFramenum, frame_number);
                sw(prop + 352, slot * 120 + g::tmpPickupData);
                free -= 1;
                sw(offset + g::tmpPickupDataProp, prop);
            }
            let total = free + 1;
            if free < 3 {
                let mut i = 0;
                if total > 0 {
                    data = lw(prop + 352);
                    loop {
                        let mut oldest_frame = lw(g::gameframe);
                        let next = i + 1;
                        let mut frames = g::tmpPickupDataFramenum;
                        let mut used = g::tmpPickupDataUsed;
                        let mut oldest = 0;
                        let mut j = 0;
                        loop {
                            if lw(used) == 0 {
                                j += 1;
                            } else {
                                let value = lw(frames);
                                if value >= oldest_frame {
                                    j += 1;
                                } else {
                                    oldest_frame = value;
                                    oldest = j;
                                    j += 1;
                                }
                            }
                            frames += 4;
                            used += 4;
                            if j >= 20 {
                                break;
                            }
                        }
                        let old_prop = lw((oldest << 2) + g::tmpPickupDataProp);
                        if lw(old_prop + 488) == 0 {
                            sw(old_prop + 492, 0x1518);
                            sw(old_prop + 488, 1);
                        }
                        i = next;
                        if i >= total {
                            break 'made;
                        }
                    }
                }
            }
            data = lw(prop + 352);
        }
    }
    sw(data + 92, slot);
    sw(prop + 8, 64);
    sw(prop + 140, 2);
    swc1(data + 100, 1.0_f32);
    let random = random_as_float();
    sw(data + 96, -1);
    let mode = lw(g::curgameparms + 72);
    swc1(data + 104, ps2float::mul(random, 2.3283064e-10_f32));
    let offset = a0 - 342;
    'speed: {
        'on_pad: {
            if mode != 7 {
                if a1 != 16 {
                    let kind = a0 - 360;
                    swc1(sp + 4, f14);
                    swc1(sp, f13);
                    spin = 0.0_f32;
                    swc1(sp + 8, f15);
                    if (kind as u32) < (30 as u32) {
                        if lw(data + 92) == -1 {
                            let index = pickupposAdd(sp);
                            sw(data + 96, index);
                            if (kind as u32) < (24 as u32) {
                                match kind {
                                    0 => swc1(data + 100, 2.0_f32),
                                    4 | 21 => swc1(data + 100, lwc1(0x3acd88)),
                                    5 => swc1(data + 100, lwc1(0x3acd80)),
                                    9 => swc1(data + 100, 2.0_f32),
                                    18 => swc1(data + 100, lwc1(0x3acd84)),
                                    23 => swc1(data + 100, 2.0_f32),
                                    _ => {}
                                }
                            }
                            if lwc1(data + 100) == 1.0_f32 {
                                swc1(prop + 72, f16);
                                break 'speed;
                            }
                            obInstSetAnim(lw(prop + 32), 1, 0.0_f32);
                            swc1(prop + 72, f16);
                            break 'speed;
                        }
                    }
                    if a1 != 8 {
                        if a1 != 4 {
                            swc1(prop + 72, f16);
                            break 'speed;
                        }
                    }
                    let index = pickupposAdd(sp);
                    sw(data + 96, index);
                    break 'on_pad;
                }
            }
            if (offset as u32) >= (11 as u32) {
                swc1(prop + 72, f16);
                break 'speed;
            }
            spin = 0.0_f32;
        }
        swc1(prop + 72, f16);
    }
    swc1(prop + 76, f17);
    swc1(prop + 88, spin);
    let flags = lw(prop + 16);
    swc1(data + 20, f19);
    swc1(data + 8, f12);
    swc1(data + 24, f13);
    sw(prop + 16, flags | 2);
    let height = lwc1(prop + 52);
    swc1(data + 32, f15);
    sw(data + 64, -1);
    sw(data + 12, a2);
    sw(data + 16, a3);
    swc1(data + 28, height);
    sw(data + 44, a4);
    sw(data + 48, a5);
    sw(data + 68, 1);
    sw(data, a1);
    sw(data + 4, 0);
    sw(data + 36, -1);
    sw(data + 40, -1);
    if a1 == 16 {
        sw(prop + 16, lw(prop + 16) & -2);
    }
    prop
}

/// Draw the weapons page (@ 0x232ed8): the box with the five picked weapons (each with its number, the colour bar and the weapon combo's name), the scrolling list of
/// all the weapons with the cursor on the picked row and the arrows when the list scrolls.
pub fn weapons_pageDraw(a0: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let old_font = textSelect(1);
    sw(sp + 56, old_font);
    pagebackgroundGfx_230688(a0);
    let weaponset = weaponset_read();
    let colours = 0x3a1b80;
    let second = ld(colours + 8);
    let third = lw(colours + 16);
    sd(sp, ld(colours));
    sd(sp + 8, second);
    sw(sp + 16, third);
    let picked = if weaponset == 0 { g::gunzpack } else { weaponset * 20 + (g::gunpresets + 160) };
    zoomboxDraw(312, 52, 575, 119, 0x2040, 0x6e6e6e80);
    let language = lw(g::cur_language);
    let title = if weaponset_read() == 0 { lw(lw((language << 2) + g::languages) + 1652) } else { lw(lw((language << 2) + g::languages) + 1648) };
    let mut entry = picked;
    let width = textWidth(title);
    let mut i = 0;
    let gap = 264 - width;
    let x = ((gap + ((gap as u32) >> 31) as i32) >> 1) + 312;
    let mut bottom = 75;
    textOutline(x, 52, title, 0x6e6e6e80, 0x2040);
    let mut top = 65;
    let mut colour = sp;
    let mut text_y = 64;
    loop {
        let combo = lw(entry);
        sb(sp + 32, i + 49);
        sb(sp + 33, 0);
        guiHGradient(312, top, 575, bottom, lw(colour), 128, 0);
        let width = textWidth(sp + 32);
        textOutline(302 - width, text_y, sp + 32, 0x6e6e6e80, 0x2040);
        let language = lw(g::cur_language);
        if combo >= 0 {
            textOutline(314, text_y, lw((lh((combo << 3) + (g::guncombos + 6)) << 2) + lw((language << 2) + g::languages)), 0x6e6e6e80, 80);
        }
        dlDrawLine(312, text_y, 575, text_y, 0x6e6e6e80);
        i += 1;
        colour += 4;
        entry += 4;
        bottom += 11;
        top += 11;
        text_y += 11;
        if i >= 5 {
            break;
        }
    }
    let item;
    let line_colour;
    'item: {
        let page_item = lhu(a0 + 4);
        item = page_item & 0xffff;
        let mut current = page_item;
        if item == 1 {
            let row = lw(g::gunzlistrow) - lw(g::gunzlistrowtop);
            dlFillRectangle(64, 52, 263, 173, 0x2040);
            let flash = lbu(g::flashalpha) | 0x8000;
            dlFillRectangle(64, row * 11 + 64, 263, row * 11 + 74, flash);
            current = lhu(a0 + 4);
            if current == item {
                line_colour = 0x6e6e6e80;
                let _ = current;
                break 'item;
            }
        }
        line_colour = if weaponset_read() != 0 { 0x20202040 } else { 0x40404080 };
    }
    let current = lhu(a0 + 4);
    let box_colour = if (current ^ 1) == 0 { 0x2040 } else { 128 };
    zoomboxDraw(64, 52, 263, 173, box_colour, line_colour);
    let mut i = 0;
    let languages = g::languages;
    let mut y = 64;
    let heading = lw(lw((lw(g::cur_language) << 2) + languages) + 1644);
    let width = textWidth(heading);
    let gap = 200 - width;
    let x = ((gap + ((gap as u32) >> 31) as i32) >> 1) + 64;
    textOutline(x, 52, heading, line_colour, box_colour);
    loop {
        dlDrawLine(64, y, 263, y, line_colour);
        let row = lw(g::gunzlistrowtop) + i;
        if row < lw(0x3aff4c) {
            sw(sp + 48, 66);
            let language = lw(g::cur_language);
            let combo = lw((row << 2) + 0x1fc2a00);
            sw(sp + 52, y);
            let name = lw((lh((combo << 3) + (g::guncombos + 6)) << 2) + lw((language << 2) + languages));
            textPrint(sp + 48, sp + 52, name, line_colour);
        }
        i += 1;
        y += 11;
        if i >= 10 {
            break;
        }
    }
    if lhu(a0 + 4) == 1 {
        let alpha = lbu(g::flashalpha);
        let row = lw(g::gunzlistrow) - lw(g::gunzlistrowtop);
        let colour = ((((alpha as u32) >> 1) as i32) << 24) | 128 | (alpha << 8 | alpha << 16);
        zoomboxDraw(65, row * 11 + 65, 264, row * 11 + 74, 0, colour);
    }
    let alpha = lbu(g::flashalpha);
    if weaponset_read() == 0 {
        let mut top = lw(g::gunzlistrowtop);
        let colour = ((((alpha as u32) >> 1) as i32) << 24) | 128 | (alpha << 8 | alpha << 16);
        if (lw(0x3aff4c) - top) >= 11 {
            guiDrawImage(5, colour, 4e+01_f32, 164.0_f32);
            top = lw(g::gunzlistrowtop);
        }
        if top > 0 {
            guiDrawImage(6, colour, 4e+01_f32, 66.0_f32);
        }
    }
    textSelect(lw(sp + 56));
}

/// Drop the prop `a0` held by the character `a1` (@ 0x26eee8). With no weapon `a2` the prop is just put down where it is; with a weapon it is replaced by a new pickup of the
/// weapon (for the kinds that can be dropped and when the character has ammo for it). The dropped prop is then given a destination pad out of the pads linked to the
/// character's pad, the nearest of those in front of the character that are low enough, and the velocity to move there.
pub fn propDrop(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let mut prop = a0;
    let holder_prop = lw(lw(prop + 188) + 244);
    let flags = lw(holder_prop + 16);
    if (flags & 128) == 0 {
        if (flags & 0xf000000) != 0 {
            let y = lwc1(prop + 568);
            let z = lwc1(prop + 572);
            swc1(prop + 48, ps2float::add(lwc1(holder_prop + 48), lwc1(prop + 564)));
            swc1(prop + 52, ps2float::add(lwc1(holder_prop + 52), y));
            swc1(prop + 56, ps2float::add(lwc1(holder_prop + 56), z));
        }
    }
    if a2 == 0 {
        sw(prop + 72, 0);
        sw(prop + 192, -1);
        sw(prop + 76, 0);
        sw(prop + 88, 0);
        sw(prop + 188, 0);
        sw(prop + 196, 0);
    } else {
        let mut made = 0;
        'weapon: {
            let combo = lw(a1 + 260);
            if ((combo - 1) as u32) < (32 as u32) {
                lvIsValidGunCombo(combo);
                let combo = lw(a1 + 260);
                if ((combo - 1) as u32) < (32 as u32) {
                    if lvIsValidGunCombo(combo) != 0 {
                        let kind = lw(a2);
                        if kind != 18 {
                            if kind >= 19 {
                                if kind == 25 {
                                    break 'weapon;
                                }
                            } else if kind < 17 {
                                if kind >= 14 {
                                    break 'weapon;
                                }
                            }
                            let slot = lw(kind * 400 + (g::guns + 16));
                            let ammo = lw(a1 + (slot << 2) + 620);
                            if ammo > 0 {
                                let height = lwc1(0x3accbc);
                                made = propNewPickup(lw(prop + 4), 2, lw(a1 + 260), 0, slot, ammo, 1, 0.0_f32, lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56), 0.0_f32, 0.0_f32, 0.0_f32, height);
                            }
                        }
                    }
                }
            }
        }
        propDelete(prop);
        prop = made;
    }
    if prop != 0 {
        let model = lw(prop + 32);
        sw(prop + 16, lw(prop + 16) & -129 & -0x40000001);
        sw(prop + 140, 2);
        let data = lw(prop + 352);
        let mut found = 0;
        sh(model + 292, 0);
        let mut i = 0;
        obInstSetAnim(model, 0, 0.0_f32);
        let mut best = lwc1(0x3accc0);
        sw(data + 68, 2);
        let owner_number = lw(a1);
        sw(data + 64, -1);
        sw(data + 72, owner_number);
        let linked = findpadslinked2pad(lw(a1 + 680), sp + 16, 19);
        let total = linked + 1;
        sw(sp + 16 + (linked << 2), lw(a1 + 680));
        let mut pads;
        let first;
        if total <= 0 {
            pads = lw(g::levelpads);
            first = 0;
        } else {
            pads = lw(g::levelpads);
            let mut offset = i << 2;
            loop {
                let at = sp + 16 + offset;
                let pad = lw(at) * 28 + pads;
                if !(ps2float::sub(lwc1(pad + 16), lwc1(prop + 52)) <= lwc1(0x3accc4)) {
                    i += 1;
                } else {
                    let in_front = checkInFront(lw(a1 + 3020), pad + 12, 0.0_f32);
                    let list = found << 2;
                    if in_front != 0 {
                        found += 1;
                        sw(sp + list + 96, lw(at));
                    }
                    pads = lw(g::levelpads);
                    i += 1;
                }
                offset = i << 2;
                if i >= total {
                    break;
                }
            }
            first = 0;
        }
        let height;
        let mut chosen;
        if found <= 0 {
            height = lwc1(data + 20);
            chosen = lw(data + 64);
        } else {
            let y = lwc1(prop + 52);
            let x = lwc1(prop + 48);
            let z = lwc1(prop + 56);
            height = lwc1(data + 20);
            let mut j = first;
            let mut offset = j << 2;
            loop {
                'candidate: {
                    let number = lw(sp + 96 + offset);
                    let pad = number * 28 + pads;
                    let dx = ps2float::sub(lwc1(pad + 12), x);
                    let dy = ps2float::sub(lwc1(pad + 16), y);
                    let dz = ps2float::sub(lwc1(pad + 20), z);
                    let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
                    if !(distance < best) {
                        j += 1;
                    } else {
                        if distance < 14.0_f32 {
                            if (lw(pad + 4) & 0x10000) != 0 {
                                j += 1;
                                break 'candidate;
                            }
                            sw(data + 64, number);
                            best = distance;
                        }
                        j += 1;
                    }
                }
                offset = j << 2;
                if j >= found {
                    break;
                }
            }
            chosen = lw(data + 64);
        }
        if chosen == -1 {
            let mut j = 0;
            let mut nearest = lwc1(0x3accc8);
            sw(data + 64, lw(a1 + 680));
            if total > 0 {
                let y = lwc1(prop + 52);
                let x = lwc1(prop + 48);
                let z = lwc1(prop + 56);
                let mut offset = j << 2;
                loop {
                    let number = lw(sp + 16 + offset);
                    let pad = number * 28 + pads;
                    let dx = ps2float::sub(lwc1(pad + 12), x);
                    let dy = ps2float::sub(lwc1(pad + 16), y);
                    let dz = ps2float::sub(lwc1(pad + 20), z);
                    let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
                    j += 1;
                    if distance < nearest {
                        sw(data + 64, number);
                        nearest = distance;
                    }
                    offset = j << 2;
                    if j >= total {
                        break;
                    }
                }
            }
            chosen = lw(data + 64);
        }
        let destination;
        if a2 != 0 {
            destination = lw(data + 64);
        } else {
            let flags = lw(prop + 16);
            let with_bits = flags | 3;
            sw(prop + 16, with_bits);
            let mut value = flags | 67;
            if lw(data + 64) == lw(data + 40) {
                value = with_bits & -65;
            }
            sw(prop + 16, value);
            destination = lw(data + 64);
            sw(data + 36, destination);
        }
        let _ = chosen;
        let at = destination * 28 + pads;
        sw(at + 4, lw(at + 4) | 0x10000);
        let pad = lw(data + 64) * 28 + pads;
        let x = lwc1(pad + 12);
        swc1(data + 52, x);
        swc1(data + 56, ps2float::add(lwc1(pad + 16), height));
        swc1(data + 60, lwc1(pad + 20));
        swc1(sp, ps2float::sub(x, lwc1(prop + 48)));
        swc1(sp + 4, ps2float::sub(lwc1(data + 56), lwc1(prop + 52)));
        swc1(sp + 8, ps2float::sub(lwc1(data + 60), lwc1(prop + 56)));
        let frames = if a2 == 0 { 10 } else { 20 };
        sw(data + 76, frames);
        let count = lw(data + 76) as f32;
        let scale = 5e+01_f32;
        let step_x = (lwc1(sp)).pdiv(count);
        swc1(prop + 60, step_x);
        swc1(prop + 68, (lwc1(sp + 8)).pdiv(count));
        swc1(prop + 64, (lwc1(sp + 4)).pdiv(count));
        swc1(data + 80, ps2float::mul(step_x, scale));
        swc1(data + 84, ps2float::mul(lwc1(prop + 64), scale));
        swc1(data + 88, ps2float::mul(lwc1(prop + 68), scale));
    }
}
