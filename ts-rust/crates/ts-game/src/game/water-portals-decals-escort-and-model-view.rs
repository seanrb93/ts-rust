//! The texture coordinates of water, the clip codes of portals, the draw list of decals, the escort pad, the spawn tick of pick-ups, the credits camera and the model view of the
//! animation matrices.

use crate::*;

/// Make `count` vertices at `out` (16 bytes each: x, y, then 1.0 and 1.0) from the ones at `source` (16 bytes each): x gets the water's moving offset in s (a sine of the game's time
/// added to `waterOffsets`), y the offset in t, and then both offsets move with the frame time (and keep within -1 and 0) (@ 0x278998).
pub fn vtxlistWaterst(source: i32, out: i32, count: i32) {
    let mut from = source;
    let mut to = out;
    let mut left = count;
    let mut offset_s = lwc1(g::waterOffsets);
    let wave = offset_s + lwc1(((((lw(g::gameframe) as f32 * lwc1(0x3ace70) * lwc1(0x3ace74) + 0.5_f32) as i32) & 2047) << 2) + g::g_SinTable) * lwc1(0x3ace78);
    if left > 0 {
        let one = 1.0_f32;
        loop {
            left -= 1;
            swc1(to, lwc1(from) + wave);
            let v = lwc1(from + 4);
            let offset_t = lwc1(g::waterOffsett);
            from += 16;
            swc1(to + 8, one);
            swc1(to + 12, one);
            swc1(to + 4, v + offset_t);
            to += 16;
            if left == 0 {
                break;
            }
        }
        offset_s = lwc1(g::waterOffsets);
    }
    let frame_time = lwc1(g::updateframesf);
    let limit = -1.0_f32;
    let next_s = offset_s - frame_time * lwc1(0x3ace7c);
    let next_t = lwc1(g::waterOffsett) - frame_time * lwc1(0x3ace80);
    swc1(g::waterOffsets, next_s);
    swc1(g::waterOffsett, next_t);
    if next_s <= limit {
        swc1(g::waterOffsets, next_s + 1.0_f32);
    }
    if next_t <= limit {
        swc1(g::waterOffsett, next_t + 1.0_f32);
    }
}

/// Work out the clip code (+16) of the portal point `point` (x, y at +0, +4, depth at +12): bit 16 when the depth is below 0, 4 and 8 when x is outside the left and right edges
/// and 1 and 2 when -y is outside the top and bottom edges (the edges are the game's slopes at 0x3b0088 to 0x3b0094) (@ 0x257e38).
pub fn bgPortalCalcOutCode(point: i32) {
    sw(point + 16, 0);
    if dpcmp(fptodp(lwc1(point + 12)), 0_i64) < 0 {
        sw(point + 16, lw(point + 16) | 16);
    }
    let depth = lwc1(point + 12);
    let x = lwc1(point);
    if x < depth * lwc1(0x3b0088) {
        sw(point + 16, lw(point + 16) | 4);
    }
    if (depth * lwc1(0x3b008c)) < x {
        sw(point + 16, lw(point + 16) | 8);
    }
    let y = -lwc1(point + 4);
    if y < depth * lwc1(0x3b0090) {
        sw(point + 16, lw(point + 16) | 1);
    }
    if (depth * lwc1(0x3b0094)) < y {
        sw(point + 16, lw(point + 16) | 2);
    }
}

/// Put the decal `decal` first in the list of the draw list `slot` (heads at 0x1fe9d48; the list is double-linked by +192 and +196, the slot is kept at +208), and set the fade of
/// every decal of the group of eight that `slot` belongs to: 1 for those at least 11 places on from the group's start, else a rising fraction (@ 0x2a30e0).
pub fn decalAddToDrawList(decal: i32, slot: i32) {
    let group = (if -1 < slot { slot } else { slot + 7 }) >> 3;
    let head_at = (slot << 2) + 0x1fe9d48;
    let head = lw(head_at);
    let first = group * lw(g::s_DecalsToGroup);
    if head != 0 {
        sw(head + 196, decal);
        sw(decal + 192, head);
    }
    sw(decal + 208, slot);
    let decals = lw(g::s_Decals);
    let size = lw(g::s_DecalsToGroup);
    sw(head_at, decal);
    let mut i = 0;
    let mut at = decals + first * 240;
    if size > 0 {
        loop {
            if size == 0 {
                trap("break");
            }
            let place = (size + i - lw((group << 2) + 0x1fe9cd8)).wrapping_rem(size);
            let fade = if place >= 11 { 1.0_f32 } else { place as f32 * lwc1(0x3ad5bc) };
            swc1(at + 224, fade);
            i += 1;
            at += 240;
            if i >= size {
                break;
            }
        }
    }
}

/// Find the pad of the escort in the level's set-up and keep its number at 0x3afef8 (-1 for none): the first of the set-up's records (48 bytes each) that passes the create flags
/// and has kind 16 gives its pad (the reference at +24) (@ 0x21dd78).
pub fn gmEscortSetup() {
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    sw(0x3afef8, -1);
    if setup == 0 {
        return;
    }
    let mut i = 0;
    let mut offset = 0;
    let mut record;
    loop {
        if i >= lw(setup + 20) {
            return;
        }
        record = lw(setup + 16) + offset;
        let passed = lvCheckCreateFlags(lw(record));
        offset += 48;
        i += 1;
        if passed != 0 && lw(record + 8) == 16 {
            break;
        }
    }
    let pad = padNumFromExtref(lw(record + 24));
    sw(0x3afef8, pad);
}

/// The tick of the pick-up's spawn effect for the effects group `group`: its model matrix (the particle group at +72) is turned about y and made larger every tick, and the group is
/// freed when its time (+48) is past 3.5 (@ 0x2ad400).
pub fn specialfxPickupSpawnTick(group: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let groups = g::fxgroups;
    let slot = group * 92 + (groups + 72);
    let step = lwc1(g::updateframesf) * lwc1(0x3ad724);
    if lw(slot) != 0 {
        matrixRotY(sp, step * 6e+02_f32 * lwc1(0x3ad728) / 1.8e+02_f32);
        let one = 1.0_f32;
        let scale = step * lwc1(0x3ad72c) + one;
        matrixScaleSet(sp + 64, scale, one, scale);
        let matrix = lw(slot) + 928;
        sceVu0MulMatrix(matrix, matrix, sp + 64);
        let matrix = lw(slot) + 928;
        sceVu0MulMatrix(matrix, matrix, sp);
    }
    if 3.5_f32 < lwc1(groups + group * 92 + 48) {
        specialfxFree(group);
    }
}

/// Work out the x and y of the credits' camera at the distance `distance` along the tunnel (50 to a point, 54 points round) into `*x` and `*y`: the point's place moved the
/// cosine ease of the way to the next point's (@ 0x2aeea8).
pub fn getCreditTunnelPos(x: i32, y: i32, distance: f32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let points = distance / 5e+01_f32;
    let two_pi = ld(0x3a9898);
    let whole = points as i32;
    let fraction = points - whole as f32;
    let angle = dptofp(dpadd(dpmul(fptodp(fraction), two_pi), two_pi));
    let ease = cosf(angle) * 0.5_f32 + 0.5_f32;
    let y_base = g::tunnel + 4;
    let point = whole.wrapping_rem(54);
    let point_x = lwc1(point * 48 + g::tunnel);
    let point_y = point * 48 + y_base;
    let next = (point + 1).wrapping_rem(54);
    let dx = lwc1(next * 48 + g::tunnel) - point_x;
    let dy = lwc1(next * 48 + y_base) - lwc1(point_y);
    swc1(sp, dx);
    swc1(sp + 4, dy);
    swc1(x, point_x + ease * dx);
    swc1(y, lwc1(point_y) + ease * dy);
}

/// Set the model view of the animation matrices from the matrix `matrix` by `flags`: without flag 2 it is copied to the matrix at the stack position (flag 1: in the second stack at
/// 0x1fb37f0 and the position goes on by one, else in the first at 0x1fb37b0); with flag 2 it is multiplied with the one below in the second stack (flag 1: and the position goes on,
/// else the offset flag is set to 1) (@ 0x215720).
pub fn anim_SetModelview(matrix: i32, flags: i32) {
    let position = lw(g::animMTX_stackpos);
    if (flags & 2) == 0 {
        if (flags & 1) == 0 {
            sceVu0CopyMatrix((position << 6) + 0x1fb37b0, matrix);
            return;
        }
        sceVu0CopyMatrix((position << 6) + 0x1fb37f0, matrix);
    } else {
        let at = position << 6;
        if (flags & 1) == 0 {
            sceVu0MulMatrix(at + 0x1fb37f0, at + (0x1fb37f0 - 64), matrix);
            sw(g::animMTX_offset, 1);
            return;
        }
        sceVu0MulMatrix(at + 0x1fb37f0, at + (0x1fb37f0 - 64), matrix);
    }
    let position = lw(g::animMTX_stackpos);
    sw(g::animMTX_offset, 0);
    sw(g::animMTX_stackpos, position + 1);
}
