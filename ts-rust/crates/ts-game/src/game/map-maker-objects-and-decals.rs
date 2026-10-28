//! Map maker objects, decals, the pad links, text boxes, preload order, ammo counts and more.

use crate::*;

/// Empty the map maker's map (@ 0x2464f8).
pub fn mmClearMap() {
    mmClearObjects();
    sb(g::mapname, 0);
    sb(g::mapdesc, 0);
    sw(g::highlightobj, 0);
    sw(g::nummapobjects, 0);
    sw(g::mapobjects, 0);
}

/// Take a decal out of the lists it is on (a doubly linked list through +192 and +196 for each depth slot, the head of a slot at 0x1fe9d48 + 4 * slot; +208 is the slot) (@ 0x2a31c8).
pub fn decalRemoveFromDrawList(decal: i32) {
    let before = lw(decal + 192);
    let after = lw(decal + 196);
    if before != 0 && after == 0 {
        let slot = lw(decal + 208);
        sw(before + 196, 0);
        sw(decal + 192, 0);
        sw((slot << 2) + 0x1fe9d48, before);
    } else if before == 0 && after == 0 {
        let slot = lw(decal + 208);
        if slot > 0 {
            let head = (slot << 2) + 0x1fe9d48;
            if lw(head) == decal {
                sw(head, 0);
            }
        }
    } else if before != 0 {
        sw(before + 196, after);
        sw(lw(decal + 196) + 192, lw(decal + 192));
        sw(decal + 196, 0);
        sw(decal + 192, 0);
    } else {
        sw(after + 192, 0);
        sw(decal + 196, 0);
    }
    sw(decal + 208, 0);
}

/// Find the data part of a map maker file in memory: after the header (two words) the parts are tagged and sized; the one tagged 0x1324627 is returned, or 0 (@ 0x253710).
pub fn mmfileGetInfo(file: i32) -> i32 {
    if lw(file) != 0x1324626 || lw(file + 4) != 114 {
        return 0;
    }
    let end = file + 0x1ffc;
    let mut part = file + 8;
    if (part as u32) < (end as u32) {
        let mut tag = lw(part);
        let mut data = part + 8;
        if tag != 0x1324629 {
            while tag != 0x1324627 {
                part = data + lw(part + 4);
                if (part as u32) >= (end as u32) {
                    return 0;
                }
                tag = lw(part);
                data = part + 8;
                if tag == 0x1324629 {
                    return 0;
                }
            }
            return data;
        }
    }
    0
}

/// Save the main window's 96 bytes of values (@ 0x20c518).
pub fn windowSave() {
    for i in 0..12 {
        sd(0x1fb31c0 + 8 * i, ld(g::window + 8 * i));
    }
}

/// Whether any room whose flag in the array `flags` is set is visible to player `player` (@ 0x2541d8).
pub fn bgIsAnyRoomsVisibleByPlayer(flags: i32, player: i32) -> i32 {
    for i in 0..lw(g::bgNumRooms) {
        if lw(flags + 4 * i) != 0 {
            let entry = 20 * i + lw((player << 2) + g::roomdata) + 20;
            let last = lw(entry + 12);
            if last >= 0 {
                let first = lw(entry + 4);
                if first < 640 && first < last {
                    return 1;
                }
            }
        }
    }
    0
}

/// Find the map maker object of kind 1 that covers the point (`x`, `y`): its tile has a width and height (in 2^21 units) and a mask of bytes at +12 that must be set there; 0 if none (@ 0x245238).
pub fn mmCheckObject(x: i32, y: i32) -> i32 {
    let level = lw(g::editlevel);
    let mut object = lw(g::mapobjects);
    while object != 0 {
        if lw(object) == 1 {
            let tile = lw(object + (lw(object + 24) << 2) + 36);
            if ((level - lw(object + 20)) as u32) < (lw(tile + 8) as u32) {
                let width = lw(tile);
                let dx = x - lw(object + 12);
                let height21 = lw(tile + 4) << 21;
                if (dx as u32) < ((width << 21) as u32) {
                    let dy = y - lw(object + 16);
                    if (dy as u32) < (height21 as u32) && lbu(lw(tile + 12) + ((dy >> 21) * width + (dx >> 21))) != 0 {
                        return object;
                    }
                }
            }
        }
        object = lw(object + 56);
    }
    0
}

/// Begin editing text in text box `n`: the text at `text` (its length is measured), a limit `a3`, `a4`, the box's position (`x`, `y`; 0 centres it in the window) (@ 0x22fd98).
pub fn stringeditStart(n: i32, text: i32, a2: i32, a3: i32, a4: i32, x: i32, y: i32) {
    stringeditSet(n);
    let b = lw(g::textbox);
    sw(b + 52, a2);
    sw(b + 48, text);
    sw(b, x);
    sw(b + 4, y);
    sw(b + 32, 0);
    sw(b + 36, 0);
    sw(b + 24, 0);
    if lbu(text) != 0 {
        loop {
            let len = lw(b + 24) + 1;
            sw(b + 24, len);
            if lbu(text + len) == 0 {
                break;
            }
        }
    }
    sw(b + 16, a3);
    sw(b + 20, a4);
    if x == 0 {
        sw(b, lw(g::window + 8) - lw(g::window) - 140);
    }
    if y == 0 {
        let w = g::window;
        sw(b + 4, lw(w + 28) - lw(w + 20) - 46);
    }
    sw(b + 8, 280);
    sw(b + 12, 92);
    if !(a3 < 3) {
        sw(b + 12, 79);
    }
    sw(b + 56, 0);
    sw(b + 28, g::chartab1);
    sw(b + 40, 0);
    sw(b + 44, 0);
}

/// The number of the link between the pads with external references `a` and `b` (either way round), or 0 (@ 0x264978).
pub fn padGetLinkNum(a: i32, b: i32) -> i32 {
    let pad_a = padNumFromExtref(a);
    let pad_b = padNumFromExtref(b);
    let links = lw(g::levellinks);
    for i in 0..lw(g::numlinks) {
        let link = i * 20 + links;
        let from = lw(link + 4);
        if from == pad_a && lw(link + 8) == pad_b {
            return i;
        }
        if from == pad_b && lw(link + 8) == pad_a {
            return i;
        }
    }
    0
}

/// For every room set the word of `out` to 1 if the box (min x, max x, min y, max y, min z, max z) overlaps its box (@ 0x254118).
pub fn bgCalcRoomArray(out: i32, min_x: f32, max_x: f32, min_y: f32, max_y: f32, min_z: f32, max_z: f32) {
    for i in 0..lw(g::bgNumRooms) {
        let room = lw(g::levelrooms) + (i * 44 + 44);
        let mut overlap = 0;
        if !(max_x < lwc1(room + 20))
            && !(lwc1(room + 32) < min_x)
            && !(max_y < lwc1(room + 24))
            && !(lwc1(room + 36) < min_y)
            && !(max_z < lwc1(room + 28))
            && !(lwc1(room + 40) < min_z)
        {
            overlap = 1;
        }
        sw(out + 4 * i, overlap);
    }
}

/// Count the mines of each kind attached to prop `prop`: nodes 20 to 49 of the bullet pool are mines; types 16, 14 and 15 are counted in
/// `out16`, `out14` and `out15` (@ 0x28a688).
pub fn bulletGetMinesAttachedToProp(prop: i32, out16: i32, out14: i32, out15: i32) {
    sw(out16, 0);
    sw(out14, 0);
    let nodes = lw(0x3b0138);
    sw(out15, 0);
    for i in 20..50 {
        let node = nodes + 276 * i;
        let kind = lw(node);
        if kind != 0 && lw(node + 224) == prop {
            match kind {
                15 => sw(out15, lw(out15) + 1),
                16 => sw(out16, lw(out16) + 1),
                14 => sw(out14, lw(out14) + 1),
                _ => {}
            }
        }
    }
}

/// Insert a preload record into the list sorted by its first word (smallest first; before records with an equal key at the head, after them elsewhere); the link is at +12 (@ 0x21c840).
pub fn preloadPass2Insert(item: i32) {
    let head = lw(0x3afee4);
    if head == 0 {
        sw(0x3afee4, item);
        return;
    }
    let key = lw(item);
    if lw(head) < key {
        let mut at = head;
        let mut next = lw(at + 12);
        while next != 0 && lw(next) < key {
            at = next;
            next = lw(at + 12);
        }
        sw(item + 12, next);
        sw(at + 12, item);
    } else {
        sw(item + 12, head);
        sw(0x3afee4, item);
    }
}

/// The pad a character is heading for, by what it targets (type 8: the target's pad, 64: a player's pad, 256: a spot, 1: the base in mode 6), or -1 (@ 0x2c1328).
pub fn enemyGetTargetPad(chr: i32) -> i32 {
    let data = lw(chr + 352);
    let mode = lw(g::curgameparms + 72);
    if mode == 1 && lw(data + 484) != 0 {
        return lw(data + 684);
    }
    let target = lw(data + 2788);
    match lw(target + 8) {
        8 => lw(lw(target + 352) + 680),
        64 => {
            if lw(data + 484) != 0 {
                lw(data + 684)
            } else {
                lw(lw(target + 352) + 36)
            }
        }
        256 => lw(lw(target + 352) + 100),
        1 if mode == 6 => lw((lw(g::currentbase) << 3) + g::bases),
        _ => -1,
    }
}

/// How much ammo of type `ammo` a character has in its two guns' clips and stores: the right gun's (at +380) when it has that type, plus the left gun's (at +272) (@ 0x294b48).
pub fn gunGetGunAmmo(chr: i32, ammo: i32) -> i32 {
    let combo = lw(chr + 264) << 3;
    let (mut right_type, mut right_ammo) = (0, 0);
    let (mut left_type, mut left_ammo) = (0, 0);
    let right_gun = lh(combo + (GUNCOMBOS + 4));
    if right_gun != -1 && (lw(g::currentplayer) + 600) != 0 && (chr + 376) != 0 {
        right_type = lw(right_gun * 400 + (GUNS_TABLE + 16));
        right_ammo = lw(chr + (right_type << 2) + 380);
    }
    let left_gun = lh(combo + (GUNCOMBOS + 2));
    if left_gun != -1 && (lw(g::currentplayer) + 408) != 0 && (chr + 268) != 0 {
        left_type = lw(left_gun * 400 + (GUNS_TABLE + 16));
        left_ammo = lw(chr + (left_type << 2) + 272);
    }
    let right = if right_type != ammo { 0 } else { right_ammo };
    if left_type != ammo { right } else { right + left_ammo }
}
