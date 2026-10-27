//! Text offsets, message button codes, bone matrices, vector rotation and more.

use super::stats::stats_slot_in_play;
use crate::*;

/// Counted the glass left and did nothing with the count (@ 0x25a398).
pub fn bgAllGlassBroken() {
    bgAmountGlassLeft();
}

/// Add texture `tex` to the first preload pass unless it is there: a record (24 bytes, kind 3, the texture at +12) kept sorted (@ 0x21c480).
pub fn preloadAddTexNum(tex: i32, extra: i32) {
    if preloadGetTexNum(tex) != 0 {
        return;
    }
    let n = lw(g::numpreload);
    sw(g::numpreload, n + 1);
    let rec = n * 24 + preload_table();
    sw(rec + 12, tex);
    sw(rec + 16, extra);
    sw(rec, 3);
    sw(rec + 8, 0);
    sw(rec + 4, 0);
    preloadInsert(rec);
}

/// Remove all decals on prop `prop` and clear the 15 effect slots (44 bytes each) that belong to it (@ 0x2a4b10).
pub fn decalRemoveDecalsFromProp(prop: i32) {
    let mut i = 0;
    while i < lw(g::s_MaxNumDecals) {
        let decal = i * 240 + lw(g::s_Decals);
        if lw(decal + 216) == prop {
            sw(decal + 204, -1);
            decalRemoveFromDrawList(decal);
        }
        i += 1;
    }
    for k in 0..15 {
        let slot = 0x1fe9a44 + 44 * k;
        let owner = lw(slot);
        if owner != 0 && lw(owner + 244) == prop {
            sw(slot, 0);
        }
    }
}

/// The sub-list (of a list of entries of 32 bytes from +16 with flag 2 and a pointer at +12) that holds the id `id` in one of its own entries, or 0 (@ 0x22dec8).
pub fn isinlist(list: i32, id: i32) -> i32 {
    for i in 0..lw(list) {
        let entry = list + 16 + 32 * i;
        if lhu(entry + 10) & 2 != 0 {
            let sub = lw(entry + 12);
            for k in 0..lhu(sub) {
                if lhu(sub + 16 + 32 * k + 8) == id & 0xffff {
                    return sub;
                }
            }
        }
    }
    0
}

/// A split-screen message (260 bytes each, text from +4) can contain `^1` to `^4` for a button; if that button was newly pressed, the digit's value (1 to 4) is returned (@ 0x230bb0).
pub fn splitMessageTick(n: i32) -> i32 {
    let msg = n * 260 + 0x1fc1488;
    if lw(msg) == 0 {
        return 0;
    }
    let text = msg + 4;
    let mut i = 0;
    if lbu(text) != 0 {
        loop {
            let next = i + 1;
            if lbu(text + i) == 94 {
                let c = lbu(text + next);
                let button = c - 49;
                i = next;
                if (button as u32) < 4 && joyNewBut(n) & lw((button << 2) + g::joybutton_147) != 0 {
                    return c - 48;
                }
            } else {
                i = next;
            }
            if lbu(text + i) == 0 {
                break;
            }
        }
    }
    0
}

/// For each bone (80 bytes; the matrix number at +1, kind at +0) of a skeleton note its matrix slot in the object's table, and the matrices of the first 12 kind-0 bones in six fields of the object (@ 0x262bf0).
pub fn getBoneMatrixNum(skeleton: i32, object: i32) {
    let n = lw(skeleton);
    if n <= 0 {
        return;
    }
    let table = lw(object + 32);
    let mut slot = 0;
    for i in 0..n {
        let bone = i * 80 + (skeleton - n * 80);
        if (slot as u32) < 12 {
            match slot {
                0 => sw(object + 8, lb(bone + 1)),
                1 => sw(object + 12, lb(bone + 1)),
                2 => sw(object + 16, lb(bone + 1)),
                6 => sw(object + 24, lb(bone + 1)),
                10 => sw(object + 28, lb(bone + 1)),
                11 => sw(object + 20, lb(bone + 1)),
                _ => {}
            }
        }
        sw((i << 2) + table, slot);
        if lb(bone) == 0 {
            slot += 1;
        }
    }
}

/// The glyph number of a character of the current font's table: from 32 up, with the characters of 128 and above mapped through `ascii8to7` (96 glyphs).
fn glyph_of(c: i32) -> i32 {
    if c & 128 == 0 { c - 32 } else { lbu((c & 127) + g::ascii8to7) - 32 }
}

/// The offset of the character at `text` from the previous: the glyph's own (+8 of its entry, 12 bytes) plus the kerning against the next character if the font has a table (@ 0x2022a8).
pub fn GetXOffset(text: i32) -> i32 {
    let font = lw(g::curfont);
    let glyph = glyph_of(lbu(text));
    let table = lw(font + 4);
    let mut offset = 0;
    if (glyph as u32) < 96 {
        offset = if table == 0 { 0 } else { lw(glyph * 12 + table + 8) };
        if table != 0 && lw(font + 8) != 0 {
            let next = lbu(text + 1);
            let class_a = lw(glyph * 12 + table + 4);
            if next != 0 && next != 10 {
                let glyph_b = glyph_of(next);
                if (glyph_b as u32) < 96 {
                    let index = class_a * 11 + lw(glyph_b * 12 + table);
                    if (index as u32) < 121 {
                        offset += lw((index << 2) + lw(font + 8));
                    }
                }
            }
        }
    }
    offset
}

/// Squared distance of the point `p` from the line through `a` with direction `dir` (a unit vector); the nearest point on the line goes to `nearest` if it is not 0 (@ 0x20acf0).
pub fn HitTestDistancePointToLineSquared(p: i32, a: i32, dir: i32, nearest: i32) -> f32 {
    let ax = lwc1(a);
    let vy = ps2float::sub(lwc1(p + 4), lwc1(a + 4));
    let dy = lwc1(dir + 4);
    let vx = ps2float::sub(lwc1(p), ax);
    let dx = lwc1(dir);
    let vz = ps2float::sub(lwc1(p + 8), lwc1(a + 8));
    let dz = lwc1(dir + 8);
    let along = ps2float::add(ps2float::add(ps2float::mul(vx, dx), ps2float::mul(vy, dy)), ps2float::mul(vz, dz));
    let (px, py, pz) = (ps2float::mul(along, dx), ps2float::mul(along, dy), ps2float::mul(along, dz));
    if nearest != 0 {
        swc1(nearest, ps2float::add(ax, px));
        swc1(nearest + 4, ps2float::add(lwc1(a + 4), py));
        swc1(nearest + 8, ps2float::add(lwc1(a + 8), pz));
    }
    let (rx, ry, rz) = (ps2float::sub(vx, px), ps2float::sub(vy, py), ps2float::sub(vz, pz));
    ps2float::add(ps2float::add(ps2float::mul(rx, rx), ps2float::mul(ry, ry)), ps2float::mul(rz, rz))
}

/// Rotate the vector `v` in place by the upper 3 x 3 of the matrix `m` (@ 0x2b55a0).
pub fn matrixVecRot(m: i32, v: i32) {
    let (x, y, z) = (lwc1(v), lwc1(v + 4), lwc1(v + 8));
    let mut out = [0.0_f32; 3];
    for (i, o) in out.iter_mut().enumerate() {
        let i = i as i32;
        *o = ps2float::add(ps2float::add(ps2float::mul(x, lwc1(m + 4 * i)), ps2float::mul(y, lwc1(m + 16 + 4 * i))), ps2float::mul(z, lwc1(m + 32 + 4 * i)));
    }
    for (i, o) in out.iter().enumerate() {
        swc1(v + 4 * i as i32, *o);
    }
}

/// The kind (1 to 7) of a picture format from the 16-bit code in its header, 0 if unknown (@ 0x2b9ad8).
pub fn GetFormat(header: i32) -> i32 {
    match lhu(header) {
        474 | 0xda01 => 1,
        0x3651 => 4,
        0x3850 => 3,
        0x3851 => 5,
        0x384d => 7,
        0x364d => 6,
        0x3650 => 2,
        _ => 0,
    }
}

/// Add to a statistic of `slot` about another character `other` (@ 0x224338).
pub fn StatsAddChar(stat: i32, slot: i32, other: i32, value: f32) {
    if stats_slot_in_play(slot) && stats_slot_in_play(other) {
        let at = (stat << 2) + other * 24 + slot * 1368 + (g::matchstats + 936);
        swc1(at, ps2float::add(lwc1(at), value));
    }
}
