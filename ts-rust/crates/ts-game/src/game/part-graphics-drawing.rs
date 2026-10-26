//! Function: drawing the parts of a model (building the display list of every part and glow of a character, a prop or a gun).

use crate::*;

/// The tag kind of a packet that loads from memory.
const LOAD: i32 = 48;
/// The tag kind of a packet that calls a ready made display list.
const CALL: i32 = 80;

/// Put a packet of the kind `kind` in the display list: `count` quad words from `address`.
fn gfx_push(kind: i32, count: i32, address: i32) {
    let tag = lw(g::gfxtag);
    sb(tag + 3, kind);
    sw(tag + 4, address);
    sh(tag, count);
    sw(g::gfxtag, tag + 16);
}

/// Load a matrix: the microcode address, the 4 quad words of the matrix, and the entry point that takes it in.
fn gfx_matrix(load: i32, matrix: i32, entry: i32) {
    gfx_push(LOAD, 1, load);
    gfx_push(LOAD, 4, matrix);
    gfx_push(LOAD, 1, entry);
}

/// The size in quad words of a list of `n` three-float points, rounded up to a multiple of 4 floats.
fn point_quads(n: i32) -> i32 {
    let rounded = (n + 3) & -4;
    ((((rounded << 1) + rounded) as u32) >> 2) as i32
}

/// Put the packets of the draw entries (24 bytes each, ended by a negative +20) of a part in the display list: for each the vertices, the normals, the colours and the
/// texture coordinates, the texture, and the microcode that draws it. `stride` is the size of the vertex data it fills (80 bytes, or 96 for the parts with the extra
/// table), `extra_kind` the kind of entry that has one more table of normals.
fn draw_entries(first: i32, tables: i32, fp: i32, s7: i32, s6: i32, base: i32, stride: i32, wide: bool, materials: i32) {
    let mut entry = first;
    let mut n = 0;
    loop {
        let size = lw(entry + 16);
        let items = lw(entry + 12);
        let lo = base + n * stride;
        gfx_push(LOAD, 1, lo);
        gfx_push(LOAD, lw(entry + 8), (lw(entry + 4) << 4) + lw(tables));
        gfx_push(LOAD, 1, lo + 16);
        gfx_push(LOAD, point_quads(size), items * 12 + fp);
        if !wide {
            gfx_push(LOAD, 1, lo + 32);
            gfx_push(LOAD, size, (items << 4) + s7);
            gfx_push(LOAD, 1, lo + 48);
            gfx_push(LOAD, (size + 3) >> 2, (items << 2) + s6);
            if lw(entry + 20) == 3 {
                gfx_push(LOAD, 1, lo + 64);
                gfx_push(LOAD, point_quads(lw(entry + 16)), lw(entry + 12) * 12 + lw(tables + 20));
            }
        } else {
            gfx_push(LOAD, 1, lo + 32);
            gfx_push(LOAD, (size + 3) >> 2, (items << 2) + lw(tables + 16));
            gfx_push(LOAD, 1, lo + 48);
            gfx_push(LOAD, size, (items << 4) + s7);
            gfx_push(LOAD, 1, lo + 64);
            gfx_push(LOAD, (size + 3) >> 2, (items << 2) + s6);
            if lw(entry + 20) == 4 {
                gfx_push(LOAD, 1, lo + 80);
                gfx_push(LOAD, point_quads(lw(entry + 16)), lw(entry + 12) * 12 + lw(tables + 20));
            }
        }
        let texture = lw(entry);
        if texture >= 0 {
            let material = lw((texture << 4) + materials);
            let index = if -1 >= material { 0 } else { material };
            let address = lw(index * 40 + lw(lw(g::thistexbuf) + 4) + 32);
            gfx_push(LOAD, 8, address);
        }
        let kind = lw(entry + 20);
        n += 1;
        entry += 24;
        gfx_push(LOAD, 1, (kind << 5) + g::startucode);
        if lw(entry + 20) < 0 {
            break;
        }
    }
}

/// Put the packets that load the lighting matrix and colours of the model (when it has lighting) in the display list.
fn lighting_packets(chr: i32, matrix: i32) {
    gfx_matrix(g::vif_load_lmtx1, matrix, g::vif_entry_loadldir1);
    gfx_push(LOAD, 1, g::vif_load_lcol);
    gfx_push(LOAD, 4, lw(chr + 28));
    gfx_push(LOAD, 1, g::vif_entry_loadlcol);
}

/// Draw the parts of a model (@ 0x260ae0): part `part` of the character, prop or gun `a0` for the pass `pass` (0 solid, others the see through parts), then its child parts (the
/// ones at +3 and +4 of its record). Fills the display list with the matrices, the draw entries of the part, and the glows (flares and lights) it has.
pub fn partGfx(a0: i32, part: i32, pass: i32) {
    let frame = Frame::new(944);
    let sp = frame.sp();
    draw_part(a0, part, pass, sp);
    let chr = a0;
    let header = lw(chr);
    let rec = header - lw(header) * 80 + part * 80;
    let first_child = lb(rec + 3);
    if first_child >= 0 {
        if lb(rec) != 3 {
            partGfx(chr, first_child, pass);
        }
    }
    let second_child = lb(rec + 4);
    if second_child >= 0 {
        partGfx(chr, second_child, pass);
    }
}

/// Draw the glows (flares, lights) of a part: for each, project the corner points and the middle ones of its box, add the points to the test of what is visible, and draw
/// the glow or flare at a size from how well the viewer faces it.
fn draw_glows(chr: i32, sp: i32, part: i32, owner: i32, rec: i32, first_glow: i32) {
    let mut count = 0;
    let mut glow = first_glow;
    loop {
        if lw(glow) != 1 {
            let mut started = true;
            let started_ok = zbtestStartAddGlow();
            if started_ok != 0 {
                let owner_inst = lw(lw(owner + 32) + 4);
                if owner_inst != 0 {
                    sceVu0MulMatrix(sp + 192, sp + 128, owner_inst + (part << 6));
                }
                let kind = lw(glow);
                let matrix = if owner_inst == 0 { sp + 128 } else { sp + 192 };
                let points_x = glow + 68;
                let points_y = glow + 72;
                let points_z = glow + 76;
                for k in 0..5 {
                    swc1(sp + 400 + k * 16, lwc1(points_x + k * 12));
                    swc1(sp + 404 + k * 16, lwc1(points_y + k * 12));
                    swc1(sp + 408 + k * 16, lwc1(points_z + k * 12));
                }
                for k in 0..4 {
                    let base_x = lwc1(glow + 68);
                    swc1(sp + 480 + k * 16, ps2float::add(base_x, ps2float::mul(ps2float::sub(lwc1(points_x + 12 + k * 12), base_x), 0.5_f32)));
                    let base_y = lwc1(glow + 72);
                    swc1(sp + 484 + k * 16, ps2float::add(base_y, ps2float::mul(ps2float::sub(lwc1(points_y + 12 + k * 12), base_y), 0.5_f32)));
                    let base_z = lwc1(glow + 76);
                    swc1(sp + 488 + k * 16, ps2float::add(base_z, ps2float::mul(ps2float::sub(lwc1(points_z + 12 + k * 12), base_z), 0.5_f32)));
                }
                if kind == 2 {
                    if lw(g::updateframesi) > 0 {
                        let roll = newrnd();
                        if (roll & 31_i64) == 0_i64 {
                            sb(glow + 148, 160);
                        }
                        let level = lbu(glow + 148) + 8;
                        sb(glow + 149, 8);
                        let clamped = if -1 >= level { 0 } else { level };
                        sb(glow + 148, if clamped >= 256 { 255 } else { clamped });
                    }
                } else {
                    sb(glow + 148, 255);
                    let found = roomlightGetRoomColourIntensity(lw(owner + 12), sp + 688, sp + 692);
                    if found != 0 {
                        sw(glow + 140, lw(sp + 688));
                        sb(glow + 148, lbu(sp + 692));
                    }
                }
                let mut w_at = sp + 268;
                let mut out_at = sp + 256;
                let mut in_at = sp + 400;
                let mut n = 0;
                'project: loop {
                    'next: {
                        matrixVec3Mul4Aligned(&[matrix as i64, in_at as i64, out_at as i64]);
                        let x = lwc1(out_at);
                        let w = lwc1(w_at);
                        if !(x < w) {
                            n += 1;
                            break 'next;
                        }
                        let neg_w = -w;
                        let at = sp + (n << 4);
                        if !(neg_w < x) {
                            n += 1;
                            break 'next;
                        }
                        let y = lwc1(at + 260);
                        if !(y < w) {
                            n += 1;
                            break 'next;
                        }
                        if !(neg_w < y) {
                            n += 1;
                            break 'next;
                        }
                        let z = lwc1(at + 264);
                        if !(z < w) {
                            n += 1;
                            break 'next;
                        }
                        if !(neg_w < z) {
                            n += 1;
                            break 'next;
                        }
                        let inverse = (1.0_f32).pdiv(w);
                        let kind_now = lw(glow);
                        let screen_y = ps2float::sub(lw(g::window + 28) as f32, ps2float::mul(lw(g::window + 36) as f32, ps2float::mul(y, inverse)));
                        let depth = ps2float::sub(32768.0_f32, ps2float::mul(ps2float::mul(z, inverse), lwc1(0x3acb80)));
                        swc1(sp + 544 + n * 12, ps2float::add(lw(g::window + 8) as f32, ps2float::mul(lw(g::window + 16) as f32, ps2float::mul(x, inverse))));
                        swc1(sp + 548 + n * 12, screen_y);
                        swc1(sp + 552 + n * 12, depth);
                        if kind_now == 0 {
                            let size = ps2float::mul(ps2float::mul(depth, lwc1(0x3acb84)), ((6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260))));
                            let small = if ilinkGetNumLocalPlayers() < 2 { (size as i32) < 8 } else { (ps2float::mul(size, 0.5_f32) as i32) < 8 };
                            if small {
                                started = false;
                                break 'project;
                            }
                        }
                        let mut weight = 100;
                        if n != 0 {
                            weight = if 4 >= n { 10 } else { 25 };
                        }
                        zbtestAddPoint(
                            lw(lw(g::currentplayer)),
                            glow,
                            chr,
                            lwc1(sp + 544 + n * 12) as i32,
                            lwc1(sp + 548 + n * 12) as i32,
                            lwc1(sp + 552 + n * 12) as i32,
                            weight,
                        );
                        'b74: {
                            if n == 0 {
                                let kind_of = lw(glow);
                                if kind_of == 0 {
                                    break 'b74;
                                }
                                if kind_of == 3 {
                                    break 'b74;
                                }
                            }
                            if n != 8 {
                                n += 1;
                                break 'next;
                            }
                            if lw(glow) != 2 {
                                n += 1;
                                break 'next;
                            }
                        }
                        let viewer = lw(g::currentplayer);
                        let dx = ps2float::sub(lwc1(viewer + 884), lwc1(glow + 68));
                        swc1(sp + 656, dx);
                        let dy = ps2float::sub(lwc1(viewer + 888), lwc1(glow + 72));
                        swc1(sp + 660, dy);
                        let dz = ps2float::sub(lwc1(viewer + 892), lwc1(glow + 76));
                        swc1(sp + 664, dz);
                        let facing = ps2float::add(ps2float::add(ps2float::mul(lwc1(glow + 128), dx), ps2float::mul(lwc1(glow + 132), dy)), ps2float::mul(lwc1(glow + 136), dz));
                        let cosine = (facing).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
                        let strength;
                        if lh(glow + 6) == 0 {
                            strength = ps2float::add(ps2float::mul(ps2float::add(cosine, 1.0_f32), 0.25_f32), 0.5_f32);
                        } else if !(-0.5_f32 < cosine) {
                            strength = 0.0_f32;
                        } else {
                            strength = (ps2float::add(cosine, 0.5_f32)).pdiv(1.5_f32);
                        }
                        let bias = zbtestGetVisibleBias(lw(viewer), glow, chr);
                        let amount = fptoui(ps2float::mul(ps2float::mul(strength, ps2float::mul(bias as f32, lwc1(0x3acb88))), 127.0_f32));
                        let previous_at = glow + 144 + lw(lw(g::currentplayer));
                        let blended = lb(previous_at) + amount;
                        let half = ((blended as u32) >> 1) as i32;
                        sb(previous_at, amount);
                        let level = half as f32;
                        let alpha = fptoui(ps2float::mul(level, ps2float::mul(lbu(glow + 148) as f32, 0.00390625_f32)));
                        if alpha != 0 {
                            let viewer_now = lw(g::currentplayer);
                            if lw(glow) == 3 {
                                let inst = lw(owner + 32);
                                swc1(sp + 672, lwc1(glow + 68));
                                let owner_matrices = lw(inst + 4);
                                swc1(sp + 676, lwc1(glow + 72));
                                swc1(sp + 680, lwc1(glow + 76));
                                if owner_matrices != 0 {
                                    matrixVecMul(owner_matrices + (part << 6), sp + 672);
                                }
                                let owner_x = lwc1(owner + 48);
                                let y = lwc1(sp + 676);
                                let z = lwc1(sp + 680);
                                swc1(sp + 672, ps2float::add(lwc1(sp + 672), owner_x));
                                swc1(sp + 676, ps2float::add(y, lwc1(owner + 52)));
                                swc1(sp + 680, ps2float::add(z, lwc1(owner + 56)));
                                zbtestDrawFlareFx(lw(lw(g::currentplayer)), lh(glow + 4), sp + 672, alpha);
                                n += 1;
                                break 'next;
                            }
                            let size = ps2float::mul(ps2float::mul(lwc1(sp + 552), lwc1(0x3acb8c)), ((6e+01_f32).pdiv(lwc1(viewer_now + 260))));
                            let amount_size = if ilinkGetNumLocalPlayers() < 2 { (size as i32) * alpha } else { (ps2float::mul(size, 0.5_f32) as i32) * alpha };
                            zbtestDrawGlow(
                                lw(lw(g::currentplayer)),
                                lh(glow + 4),
                                ((amount_size as u32) / (127 as u32)) as i32,
                                lw(glow + 140) | alpha,
                                lwc1(sp + 544),
                                lwc1(sp + 548),
                            );
                        }
                        n += 1;
                    }
                    w_at += 16;
                    out_at += 16;
                    in_at += 16;
                    if n >= 9 {
                        break 'project;
                    }
                }
                if started {
                    zbtestEndAddGlow();
                }
            }
        } else {
            sb(glow + 148, 255);
        }
        count += 1;
        glow += 152;
        if count >= lb(rec + 6) {
            break;
        }
    }
}

/// Draw the part itself (see `partGfx`): the matrices and draw entries of the part, and its glows.
#[inline(never)]
fn draw_part(a0: i32, part: i32, pass: i32, sp: i32) {
    let chr = a0;
    let header = lw(chr);
    let owner = lw(chr + 244);
    let parts = header - lw(header) * 80;
    let rec = parts + part * 80;
    let mut materials = lw(header + 16);
    if materials == 0 {
        materials = lw(header + 12);
    }
    let overlay_table = lw(chr + 248);
    let overlay = if overlay_table == 0 { 0 } else { part * 48 + overlay_table + pass * 24 };
    let mut has_overlay = 0;
    if overlay != 0 {
        if lw(overlay) != 0 || lw(overlay + 4) != 0 || lw(overlay + 8) != 0 || lw(overlay + 12) != 0 || lw(overlay + 16) != 0 || lw(overlay + 20) != 0 {
            has_overlay = 1;
        }
    }
    let player = lw(g::currentplayer);
    let pass_offset = pass << 2;
    let list_a = lw(header + pass_offset + 20);
    let list_b = lw(header + pass_offset + 28);
    let part_offset = part << 2;
    let list_base = lw(part_offset + list_a);
    let call_list = lw(part_offset + list_b);
    let mut matrices = 0;
    match lw(player) {
        1 => matrices = lw(chr + 12),
        2 => matrices = lw(chr + 16),
        3 => matrices = lw(chr + 20),
        0 => matrices = lw(chr + 8),
        _ => {}
    }
    let mask = lhu(rec + 68);
    let mut draw = true;
    if mask != 0 {
        if (lhu(chr + 292) & mask) == 0 {
            draw = false;
        }
    }
    'drawn: {
        if draw {
            let entries;
            let tables;
            'b11: {
                if pass != 0 {
                    entries = lw(rec + 12);
                    tables = rec + 44;
                    break 'b11;
                }
                let tris = lhu(rec + 70);
                sw(g::ucodetris, lw(g::ucodetris) + tris);
                if lw(player) == 0 {
                    if lw(owner + 8) == 2 {
                        sw(g::bgtris, lw(g::bgtris) + tris);
                        if lw(player + 796) == lw(owner + 4) - 512 {
                            sw(g::bgcurtris, lw(g::bgcurtris) + tris);
                        }
                    }
                }
                entries = lw(rec + 8);
                tables = rec + 20;
            }
            'geometry: {
                if entries == 0 {
                    break 'geometry;
                }
                let fp = if overlay == 0 {
                    lw(tables + 4)
                } else {
                    let v = lw(overlay + 4);
                    if v == 0 { lw(tables + 4) } else { v }
                };
                let s7 = if overlay == 0 {
                    lw(tables + 8)
                } else {
                    let v = lw(overlay + 8);
                    if v == 0 { lw(tables + 8) } else { v }
                };
                let s6 = if overlay == 0 {
                    lw(tables + 12)
                } else {
                    let v = lw(overlay + 12);
                    if v != 0 { v } else { lw(tables + 12) }
                };
                let model_kind = lw(header + 4);
                let alt_list = |chr: i32| lw((part_offset) + lw(chr + pass_offset + (lw(chr + 288) << 3) + 252));
                'b20: {
                    if model_kind != 0 {
                        if lw(owner + 188) == 0 {
                            if lw(owner + 140) != 4 {
                                let kind = lbu(rec);
                                if lb(rec) == 0 {
                                    gfx_matrix(g::vif_load_mmtx_358fc0, (lb(rec + 1) << 6) + matrices, g::vif_entry_loadm_358ff0);
                                    if lw(header + 8) != 0 {
                                        if lw(chr + 24) != 0 {
                                            if lw(chr + 28) != 0 {
                                                lighting_packets(chr, (lb(rec + 1) << 6) + lw(chr + 24));
                                            }
                                        }
                                    }
                                    if has_overlay == 0 {
                                        if call_list != 0 {
                                            gfx_push(CALL, 0, call_list);
                                            break 'b20;
                                        }
                                    }
                                    if lw(chr + 284) == 0 {
                                        if lw(entries + 20) < 0 {
                                            break 'b20;
                                        }
                                        draw_entries(entries, tables, fp, s7, s6, list_base, 80, false, materials);
                                        break 'b20;
                                    }
                                    gfx_push(CALL, 0, alt_list(chr));
                                    break 'b20;
                                }
                                if ((kind - 1) & 255) >= 3 {
                                    break 'b20;
                                }
                                let first = lb(lb(rec + 2) * 80 + parts + 1);
                                gfx_matrix(g::vif_load_mmtx_358fc0, (first << 6) + matrices, g::vif_entry_loadm_358ff0);
                                let second = lb(lb(rec + 3) * 80 + parts + 1);
                                gfx_matrix(g::vif_load_mmtx2, (second << 6) + matrices, g::vif_entry_loadm2);
                                if lb(rec + 1) >= 0 {
                                    gfx_matrix(g::vif_load_mmtx3, (lb(rec + 1) << 6) + matrices, g::vif_entry_loadm3);
                                }
                                if lw(header + 8) != 0 {
                                    let first = lb(lb(rec + 2) * 80 + parts + 1);
                                    gfx_matrix(g::vif_load_lmtx1, (first << 6) + lw(chr + 24), g::vif_entry_loadldir1);
                                    let second = lb(lb(rec + 3) * 80 + parts + 1);
                                    gfx_matrix(g::vif_load_lmtx2, (second << 6) + lw(chr + 24), g::vif_entry_loadldir2);
                                    if lb(rec + 1) >= 0 {
                                        gfx_matrix(g::vif_load_lmtx3, (lb(rec + 1) << 6) + lw(chr + 24), g::vif_entry_loadldir3);
                                    }
                                    gfx_push(LOAD, 1, g::vif_load_lcol);
                                    gfx_push(LOAD, 4, lw(chr + 28));
                                    gfx_push(LOAD, 1, g::vif_entry_loadlcol);
                                }
                                if has_overlay == 0 {
                                    if call_list != 0 {
                                        gfx_push(CALL, 0, call_list);
                                        break 'b20;
                                    }
                                }
                                if lw(chr + 284) != 0 {
                                    gfx_push(CALL, 0, alt_list(chr));
                                    break 'b20;
                                }
                                if lw(entries + 20) < 0 {
                                    break 'geometry;
                                }
                                draw_entries(entries, tables, fp, s7, s6, list_base, 96, true, materials);
                                break 'b20;
                            }
                        }
                    }
                    // the part is drawn with a matrix made from the owner (model kind 0, hidden, or mode 4)
                    let matrix = memdbAlloc(64);
                    let mode = lw(owner + 140);
                    let stage = lw(owner + 136);
                    let view = lw(lw(g::currentplayer) + 1764);
                    let owner_inst = lw(lw(owner + 32) + 4);
                    'b38: {
                        match mode {
                            0 => {
                                if stage != 0 {
                                    break 'b38;
                                }
                                sceVu0CopyMatrix(matrix, view);
                                break 'b38;
                            }
                            1 | 2 => {
                                if mode == 1 {
                                    matrixTransRotY(sp, lwc1(owner + 48), lwc1(owner + 52), lwc1(owner + 56), ps2float::mul(lwc1(owner + 76), lwc1(0x3acb60)).pdiv(1.8e+02_f32));
                                } else {
                                    let pi = lwc1(0x3acb64);
                                    matrixTransRotZXY(
                                        sp,
                                        lwc1(owner + 48),
                                        lwc1(owner + 52),
                                        lwc1(owner + 56),
                                        ps2float::mul(lwc1(owner + 72), pi).pdiv(1.8e+02_f32),
                                        ps2float::mul(lwc1(owner + 76), pi).pdiv(1.8e+02_f32),
                                        ps2float::mul(lwc1(owner + 88), pi).pdiv(1.8e+02_f32),
                                    );
                                }
                                let stage = lw(owner + 136);
                                if stage == 1 {
                                    sceVu0MulMatrix(matrix, view, sp);
                                } else if stage == 2 {
                                    sceVu0CopyMatrix(matrix, sp);
                                }
                                break 'b38;
                            }
                            3 => {
                                matrixTransRotY(sp, lwc1(owner + 48), lwc1(owner + 52), lwc1(owner + 56), ps2float::mul(lwc1(owner + 76), lwc1(0x3acb68)).pdiv(1.8e+02_f32));
                                let stage = lw(owner + 136);
                                if stage == 1 {
                                    sceVu0MulMatrix(sp + 64, view, sp);
                                    sceVu0MulMatrix(matrix, sp + 64, owner_inst);
                                    if part != 0 {
                                        sceVu0MulMatrix(matrix, matrix, owner_inst + (part << 6));
                                    }
                                    break 'b38;
                                }
                                if stage == 2 {
                                    sceVu0MulMatrix(matrix, sp, owner_inst);
                                }
                                break 'b38;
                            }
                            4 => {
                                matrixTransRotY(sp, lwc1(owner + 48), lwc1(owner + 52), lwc1(owner + 56), ps2float::mul(lwc1(owner + 76), lwc1(0x3acb6c)).pdiv(1.8e+02_f32));
                                matrixroll(lw(lw(owner + 32) + 4), owner + 100, lwc1(owner + 92));
                                sceVu0MulMatrix(sp, sp, lw(lw(owner + 32) + 4));
                                let stage = lw(owner + 136);
                                if stage == 1 {
                                    sceVu0MulMatrix(matrix, view, sp);
                                } else if stage == 2 {
                                    sceVu0CopyMatrix(matrix, sp);
                                }
                                break 'b38;
                            }
                            _ => {}
                        }
                    }
                    gfx_matrix(g::vif_load_mmtx_358fc0, matrix, g::vif_entry_loadm_358ff0);
                    'lit: {
                        if lw(header + 8) != 0 {
                            if lw(chr + 24) == 0 {
                                break 'lit;
                            }
                            if lw(chr + 28) != 0 {
                                lighting_packets(chr, lw(chr + 24));
                            }
                        }
                    }
                    if has_overlay == 0 {
                        if call_list != 0 {
                            gfx_push(CALL, 0, call_list);
                            break 'b20;
                        }
                    }
                    if lw(chr + 284) == 0 {
                        if lw(entries + 20) < 0 {
                            break 'geometry;
                        }
                        draw_entries(entries, tables, fp, s7, s6, list_base, 80, false, materials);
                        break 'b20;
                    }
                    gfx_push(CALL, 0, alt_list(chr));
                }
            }
            if pass != 0 {
                break 'drawn;
            }
            let glows = lbu(rec + 6);
            if lb(rec + 6) <= 0 {
                break 'drawn;
            }
            let mode = lw(owner + 140);
            let stage = lw(owner + 136);
            let glow = lw(rec + 16);
            let view_a = lw(lw(g::currentplayer) + 1768);
            let view_b = lw(lw(g::currentplayer) + 1760);
            let owner_inst = lw(lw(owner + 32) + 4);
            'matrix: {
                match mode {
                    0 => {
                        if stage != 0 {
                            break 'matrix;
                        }
                        sceVu0CopyMatrix(sp + 128, view_a);
                    }
                    1 | 2 => {
                        if mode == 1 {
                            matrixTransRotY(sp, lwc1(owner + 48), lwc1(owner + 52), lwc1(owner + 56), ps2float::mul(lwc1(owner + 76), lwc1(0x3acb70)).pdiv(1.8e+02_f32));
                        } else {
                            let pi = lwc1(0x3acb74);
                            matrixTransRotZXY(
                                sp,
                                lwc1(owner + 48),
                                lwc1(owner + 52),
                                lwc1(owner + 56),
                                ps2float::mul(lwc1(owner + 72), pi).pdiv(1.8e+02_f32),
                                ps2float::mul(lwc1(owner + 76), pi).pdiv(1.8e+02_f32),
                                ps2float::mul(lwc1(owner + 88), pi).pdiv(1.8e+02_f32),
                            );
                        }
                        let stage = lw(owner + 136);
                        if stage == 1 {
                            sceVu0MulMatrix(sp + 128, view_a, sp);
                        } else if stage == 2 {
                            sceVu0MulMatrix(sp + 128, view_b, sp);
                        }
                    }
                    3 => {
                        matrixTransRotY(sp, lwc1(owner + 48), lwc1(owner + 52), lwc1(owner + 56), ps2float::mul(lwc1(owner + 76), lwc1(0x3acb78)).pdiv(1.8e+02_f32));
                        let stage = lw(owner + 136);
                        if stage == 1 {
                            sceVu0MulMatrix(sp + 64, lw(lw(g::currentplayer) + 1764), sp);
                            sceVu0MulMatrix(sp + 128, sp + 64, owner_inst);
                        } else if stage == 2 {
                            sceVu0MulMatrix(sp + 128, sp, owner_inst);
                        }
                    }
                    4 => {
                        matrixTransRotY(sp, lwc1(owner + 48), lwc1(owner + 52), lwc1(owner + 56), ps2float::mul(lwc1(owner + 76), lwc1(0x3acb7c)).pdiv(1.8e+02_f32));
                        matrixroll(lw(lw(owner + 32) + 4), owner + 100, lwc1(owner + 92));
                        sceVu0MulMatrix(sp, sp, lw(lw(owner + 32) + 4));
                        let stage = lw(owner + 136);
                        if stage == 1 {
                            sceVu0MulMatrix(sp + 128, view_a, sp);
                        } else if stage == 2 {
                            sceVu0MulMatrix(sp + 128, view_b, sp);
                        }
                    }
                    _ => {}
                }
            }
            if (glows << 24) as i32 > 0 {
                draw_glows(chr, sp, part, owner, rec, glow);
            }
        }
    }
}
