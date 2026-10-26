//! Override chunks of props, the team flags, objects under the map maker cursor, the boxes of portals, glass that heals, and the item count of a map file.

use crate::*;

/// The last record (24 bytes, valid while the word at +20 is not negative) of a list of chunk records.
pub(super) fn last_chunk(list: i32) -> i32 {
    let mut at = list;
    if lw(at + 20) < 0 || lw(at + 44) < 0 {
        return at;
    }
    loop {
        at += 24;
        if lw(at + 44) < 0 {
            return at;
        }
    }
}

/// For a bone of a prop (`which` 0 the first list of chunks at +8 of the bone, 1 the second at +12): the end of its vertices written to `out`, and the value of
/// the prop's override record (48 bytes a bone) selected by `field` (1, 2 or 4: +4, +8, +12 for the first list, +28, +32, +36 for the second); 0 when there is no
/// such value, and `out` 0 then (@ 0x274468).
pub fn getOverrideChunk(prop: i32, bone: i32, which: i32, field: i32, out: i32) -> i32 {
    let model = lw(prop + 32);
    let skeleton = lw(model);
    let this = skeleton - lw(skeleton) * 80 + bone * 80;
    if out == 0 {
        return 0;
    }
    if which == 0 {
        let list = lw(this + 8);
        if list != 0 {
            let chunk = last_chunk(list);
            sw(out, lw(chunk + 12) + lw(chunk + 16));
            let offset = match field {
                1 => Some(4),
                2 => Some(8),
                4 => Some(12),
                _ => None,
            };
            if let Some(offset) = offset {
                return lw(bone * 48 + lw(model + 248) + offset);
            }
        }
    }
    if which != 1 {
        sw(out, 0);
    } else {
        let list = lw(this + 12);
        if list == 0 {
            sw(out, 0);
        } else {
            let chunk = last_chunk(list);
            sw(out, lw(chunk + 12) + lw(chunk + 16));
            let offset = match field {
                1 => Some(28),
                2 => Some(32),
                4 => Some(36),
                _ => None,
            };
            if let Some(offset) = offset {
                return lw(bone * 48 + lw(model + 248) + offset);
            }
            sw(out, 0);
        }
    }
    0
}

/// Set the flags of the five teams for a hit of the character `attacker` on `victim` (`mode` 1) or clear them (`mode` 0): the bits are 1 or 4 (a hit on another team's character,
/// for the team of the attacker or the others) or 2 or 8 (on the attacker's own team) (@ 0x288e40).
pub fn updateTeamFlags(attacker: i32, victim: i32, mode: i32) {
    let team_a = lw(lw(attacker + 352) + 24);
    let team_b = lw(lw(victim + 352) + 16);
    if mode == 1 {
        for i in 0..5 {
            let at = g::teamflag + 4 * i;
            let bit = if team_a != team_b {
                if i != team_a { 4 } else { 1 }
            } else if i != team_a {
                8
            } else {
                2
            };
            sw(at, lw(at) | bit);
        }
    } else if mode == 0 {
        for i in 0..5 {
            let at = g::teamflag + 4 * i;
            let mask = if team_a == team_b {
                if i != team_a { -9 } else { -3 }
            } else if i != team_a {
                -5
            } else {
                -2
            };
            sw(at, lw(at) & mask);
        }
    }
}

/// The map maker object under the mouse: objects of kind 1 (or the kind of the first used object, else kinds 1 and 2) whose box (a kind 1 object covers its tile's width and
/// height in 2^21 units, and its byte of the tile must be set) holds the mouse position; 0 if none (@ 0x2454d8).
pub fn mmObjectUnderCursor() -> i32 {
    let scale_x = lw(g::scale_wtomx);
    let mouse_x = (lh(lw(g::front) + 34) - lw(g::w_x)) << 16;
    let moved_x = FMul(mouse_x, scale_x);
    let scale_y = lw(g::scale_wtomy);
    let mx = lw(g::m_x) + moved_x as i32;
    let mouse_y = (lh(lw(g::front) + 38) - lw(g::w_y)) << 16;
    let moved_y = FMul(mouse_y, scale_y);
    let used = lw(g::usedobj);
    let my = lw(g::m_y) + moved_y as i32;
    let mut kind = if used != 0 { lw(used) } else { 1 };
    loop {
        let level = lw(g::editlevel);
        let mut object = lw(g::mapobjects);
        while object != 0 {
            if lw(object) == kind {
                let x = lw(object + 12);
                let y = lw(object + 16);
                let (mut x1, mut y1) = (x, y);
                let mut tile = 0;
                let mut skip = false;
                if kind == 1 {
                    tile = lw(object + (lw(object + 24) << 2) + 36);
                    if ((level - lw(object + 20)) as u32) >= (lw(tile + 8) as u32) {
                        skip = true;
                    } else {
                        x1 = x.wrapping_add(lw(tile) << 21);
                        y1 = y.wrapping_add(lw(tile + 4) << 21);
                    }
                }
                if !skip && !(mx < x) && mx < x1 && !(my < y) && !(my >= y1) {
                    if kind != 1 || lbu(lw(tile + 12) + (((my - y) >> 21) * lw(tile) + ((mx - x) >> 21))) != 0 {
                        return object;
                    }
                }
            }
            object = lw(object + 56);
        }
        if used != 0 {
            return 0;
        }
        let more = (kind as u32) < 2;
        kind += 1;
        if !more {
            return 0;
        }
    }
}

/// Work out the box (min x, y, z, max x, y, z; 48-byte records after the first) of each portal from its points (12 bytes each from +24, their number a halfword at +20) (@ 0x2560a0).
pub fn bgPortalInitBB() {
    let portals = lw(g::bgNumPortals);
    let mut i = 0;
    if portals > 0 {
        let list = lw(g::levelportals);
        let records = lw(g::portaldata);
        loop {
            let portal = lw((i << 2) + list + 4);
            let high = lwc1(0x3aea44);
            let low = lwc1(0x3aea48);
            i += 1;
            let rec = records + (i * 48);
            for k in 0..3 {
                swc1(rec + 4 * k, high);
                swc1(rec + 12 + 4 * k, low);
            }
            let count = (lhu(portal + 20) << 16) >> 16;
            for k in 0..count.max(0) {
                let point = portal + 24 + 12 * k;
                for axis in 0..3 {
                    let v = lwc1(point + 4 * axis);
                    if v < lwc1(rec + 4 * axis) {
                        swc1(rec + 4 * axis, v);
                    }
                    if lwc1(rec + 12 + 4 * axis) < lwc1(point + 4 * axis) {
                        swc1(rec + 12 + 4 * axis, lwc1(point + 4 * axis));
                    }
                }
            }
            if i >= portals {
                break;
            }
        }
    }
}

/// Count the items of kind `kind` (a halfword in the lists after each item record of 36 bytes) in a map maker file in memory: the chunk tagged 0x1324627 gives the number of
/// item records, the one tagged 0x1324628 holds them (@ 0x2537b0).
pub fn mmfileGetItemCount(file: i32, kind: i32) -> i32 {
    let mut records = 0;
    let mut count = 0;
    if lw(file) != 0x1324626 || lw(file + 4) != 114 {
        return 0;
    }
    let end = file + 0x1ffc;
    let mut part = file + 8;
    if (part as u32) < (end as u32) {
        let mut tag = lw(part);
        let mut data = part + 8;
        if tag != 0x1324629 {
            loop {
                let size = lw(part + 4);
                if tag == 0x1324627 {
                    records = lw(data + 4);
                } else if tag == 0x1324628 && records > 0 {
                    let mut at = data;
                    for _ in 0..records {
                        let n = lhu(at + 10);
                        at += 36;
                        for _ in 0..n {
                            if lh(at) == kind {
                                count += 1;
                            }
                            at += 4;
                        }
                    }
                }
                part = data + size;
                if (part as u32) >= (end as u32) {
                    break;
                }
                tag = lw(part);
                data = part + 8;
                if tag == 0x1324629 {
                    break;
                }
            }
        }
    }
    count
}

/// Glass heals and decals on it fade: each pane of glass (a float at +0 that is negative while broken, counting up to 0) of every room gets the frame time (in seconds)
/// added, except in mode 7 or with flag 64 while it is not nearly healed; its decals (four numbers at +176) fade, and are removed when the time is 0.5 past (@ 0x25a3b8).
pub fn bgTick() {
    let dt = (lwc1(g::updateframesf)).pdiv(60.0_f32);
    let mut room = 0;
    while room < lw(g::bgNumRooms) {
        let list = lw(room * 44 + lw(g::levelrooms) + 60);
        room += 1;
        if list != 0 && lw(list) != 0 {
            let mut k = 0;
            loop {
                let glass = lw((k << 2) + list);
                let value = lwc1(glass);
                k += 1;
                if value < 0.0 {
                    let progress = ps2float::add(value, 60.0);
                    let mut update = true;
                    if !(progress < 0.5) {
                        let params = g::curgameparms;
                        if lw(params + 72) == 7 || lw(params + 80) & 64 != 0 {
                            update = false;
                        }
                    }
                    if update {
                        let new = ps2float::add(value, dt);
                        swc1(glass, new);
                        if 0.0 < new {
                            swc1(glass, 0.0);
                        }
                    }
                    let slot = ((k - 1) << 2) + list;
                    for d in 0..4 {
                        let at = d << 2;
                        let decal = lw(lw(slot) + at + 176);
                        if decal == -1 {
                            continue;
                        }
                        if !(0.5 <= progress) {
                            decalSetAlphaScale(decal, ps2float::sub(1.0, ps2float::add(progress, progress)));
                        } else {
                            decalRemoveDecal(decal);
                            sw(lw(slot) + at + 176, -1);
                        }
                    }
                }
                if lw((k << 2) + list) == 0 {
                    break;
                }
            }
        }
    }
}
