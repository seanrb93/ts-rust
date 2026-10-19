//! Menu stack, statistics extremes, spawn props, zoom boxes, portal search, the bot sets and the display list's texture upload.

use crate::*;

/// Leave the current menu: back to the one below it on the stack (or none; with the debug menu shown, back to the game's menu stack) (@ 0x20cda0).
pub fn menuEnd() {
    let pos = lw(g::menu_stackpos);
    let now = pos - 1;
    sw(g::menu_stackpos, now);
    if now != 0 {
        sw(g::curmenu, lw(((pos - 2) << 2) + g::menustack));
    } else if debugmenuGetJoyMode() != 1 {
        sw(g::curmenu, 0);
    } else {
        toggledebug();
        debugmenuEnd();
        sw(g::curmenu, 0);
    }
    joyClearAll(lw(lw(g::currentplayer)));
}

/// How many items of kind `kind` all the map maker's objects of kind 1 carry (@ 0x248cd0).
pub fn mmItemCount(kind: i32) -> i32 {
    let mut n = 0;
    let mut object = lw(g::mapobjects);
    while object != 0 {
        if lw(object) == 1 && lw(object + 52) != 0 {
            n += mmItemListCount(lw(object + 52), kind);
        }
        object = lw(object + 56);
    }
    n
}

/// The slot with the highest value of statistic `stat` among the first `n`, or -1 (@ 0x221298).
pub fn StatsGetMax(stat: i32, n: i32) -> i32 {
    let mut best = -1;
    let mut highest = -1.0_f32;
    for slot in 0..n.max(0) {
        if highest < StatsGet(stat, slot) {
            best = slot;
            highest = StatsGet(stat, best);
        }
    }
    best
}

/// The slot with the lowest value among the first `n`, or -1 (@ 0x221338).
pub fn StatsGetMin(stat: i32, n: i32) -> i32 {
    let mut best = -1;
    let mut lowest = lwc1(0x3ae5d0);
    for slot in 0..n.max(0) {
        if StatsGet(stat, slot) < lowest {
            best = slot;
            lowest = StatsGet(stat, best);
        }
    }
    best
}

/// Scale every part of a prop's model (matrices of 64 bytes at +4 of the model's list) by (`x`, `y`, `z`) (the loop of several spawn props).
fn scale_parts(prop: i32, x: f32, y: impl Fn() -> f32, z: f32) {
    let mut model = lw(prop + 32);
    if lw(lw(model)) > 0 {
        let mut i = 0;
        loop {
            matrixScaleSet(lw(model + 4) + (i << 6), x, y(), z);
            i += 1;
            model = lw(prop + 32);
            if i >= lw(lw(model)) {
                break;
            }
        }
    }
}

/// A splitter dome: parts scaled by (1, +44 of the data, 1); turns backwards at twice the frame time (@ 0x27be08).
pub fn splitterdomePropTick(prop: i32) {
    let data = lw(prop + 352);
    scale_parts(prop, 1.0, || lwc1(data + 44), 1.0);
    let f = lwc1(g::updateframesf);
    swc1(prop + 76, lwc1(prop + 76) - (f + f));
}

/// A rotating spawn: parts scaled evenly by the data's +44; turns at 8 degrees a tick on two axes (@ 0x27bc08).
pub fn rotatespawnPropTick(prop: i32) {
    let data = lw(prop + 352);
    let model = lw(prop + 32);
    if lw(lw(model)) > 0 {
        let mut model = model;
        let mut i = 0;
        loop {
            let s = lwc1(data + 44);
            matrixScaleSet(lw(model + 4) + (i << 6), s, s, s);
            i += 1;
            model = lw(prop + 32);
            if i >= lw(lw(model)) {
                break;
            }
        }
    }
    let turn = lwc1(g::updateframesf) * 8.0;
    let other = lwc1(prop + 72) + turn;
    swc1(prop + 76, lwc1(prop + 76) + turn);
    swc1(prop + 72, other);
}

/// Animate zoom box `n`: it shrinks towards its centre while the counter at +36 runs down from 10 (the size is (11 - counter) tenths) (@ 0x22efd0).
pub fn zoomboxTick(n: i32) {
    let b = n * 40 + g::zoomboxes;
    let (x0, y0) = (lw(b), lw(b + 4));
    let mut w = lw(b + 8) - x0;
    let counter = lw(b + 36);
    let mut h = lw(b + 12) - y0;
    let cx = x0 + w / 2;
    let cy = y0 + h / 2;
    if counter != 0 {
        let scale = 11 - counter;
        h = (h * scale).wrapping_div(10);
        w = (w * scale).wrapping_div(10);
    }
    sw(b + 20, cx - w / 2);
    sw(b + 24, cy - h / 2);
    sw(b + 28, w);
    sw(b + 32, h);
    if counter != 0 {
        sw(b + 36, counter - 1);
    }
}

/// The number of the portal that joins room `a` and room `b`, or -1 (a room's record has a list of its portals at +4) (@ 0x2577b8).
pub fn bgGetPortalFromRooms(a: i32, b: i32) -> i32 {
    let list = lw(a * 44 + lw(g::levelrooms) + 4);
    for i in 0..lw(list).max(0) {
        let portal_number = lw(list + 4 + 4 * i);
        let portal = lw((portal_number << 2) + lw(g::levelportals));
        let mut other = lw(portal);
        if other == a {
            other = lw(portal + 4);
        }
        if other == b {
            return portal_number;
        }
    }
    -1
}

/// Whether any local player sees the room (the same test as `isroomvisible` on each player's table) (@ 0x253fd0).
pub fn isRoomVisibleByAnyPlayer(room: i32) -> i32 {
    for i in 0..lw(g::lvnumplayers) {
        let entry = room * 20 + lw((i << 2) + g::roomdata);
        let last = lw(entry + 12);
        if last >= 0 {
            let first = lw(entry + 4);
            if first < 640 && first < last {
                return 1;
            }
        }
    }
    0
}

/// Distance of a point from a line, worked out into locals and not returned (@ 0x263628).
pub fn LinePointDistanceSquared(_origin: i32, _direction: i32, _point: i32) {}

/// Load bot set `n` into the 10 character slots (16 bytes each from 0x1fc2a88): sets 0 to 7 come from the character presets (44 bytes), later ones from bytes at 0x1fc2b28 (@ 0x234478).
pub fn botsetInit(n: i32) {
    if n >= 8 {
        for i in 0..10 {
            sw(0x1fc2a88 + 16 * i, lb((n - 8) * 10 + 0x1fc2b28 + i));
        }
        return;
    }
    for i in 0..10 {
        sw(0x1fc2a88 + 16 * i, lw(n * 44 + (g::chrpresets + 396) + 4 * i));
    }
}

/// How high above the floor a pickup floats, for its prop number and kind: a constant of the game for each case (@ 0x273038).
pub fn pickupGetExtraHeight(prop: i32, kind: i32) -> f32 {
    let d = prop - 342;
    let mode7 = lw(g::curgameparms + 72) == 7;
    if (d as u32) < 11 && mode7 {
        return lwc1(0x3acd70);
    }
    if kind != 8 {
        if !mode7 && kind == 2 && (d as u32) >= 11 {
            if prop != 360 && prop != 381 && prop != 364 {
                return lwc1(0x3acd7c);
            }
            return lwc1(0x3acd78);
        }
        return 0.0;
    }
    lwc1(0x3acd74)
}

/// Add an image upload to the display list: the GIF packet (112 bytes from the debug allocator) that sets the transfer registers for
/// a `w` x `h` picture at `base`, followed by the picture data at `data` (@ 0x2b7b40).
pub fn dlLoadTexture(a: i32, b: i32, c: i32, w: i32, h: i32, data: i32) {
    let packet = memdbAlloc(112);
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    let quads = ((w * h) << 1) >> 4;
    let next = tag + 16;
    sw(g::gfxtag, next);
    sh(tag, 7);
    sd(packet + 32, (a as i64) << 32 | (b as i64) << 48 | (c as i64) << 56);
    sd(packet + 64, w as i64 | (h as i64) << 32);
    sw(tag + 4, packet);
    sw(packet + 4, 0x6008000);
    sw(packet + 8, 0x13000000);
    sw(packet + 12, quads + 6 | 0x50000000);
    sd(packet + 16, 0x1000000000000004_i64);
    sd(packet + 24, 14_i64);
    sd(packet + 40, 80_i64);
    sd(packet + 56, 81_i64);
    sd(packet + 72, 82_i64);
    sd(packet + 88, 83_i64);
    sd(packet + 96, (quads | 0x8000) as i64 | 0x800000000000000_i64);
    sd(packet + 104, 0_i64);
    sw(packet, 0);
    sd(packet + 48, 0_i64);
    sd(packet + 80, 0_i64);
    sb(next + 3, 48);
    let tag2 = lw(g::gfxtag);
    sw(tag2 + 4, data);
    sh(tag2, quads);
    sw(g::gfxtag, tag2 + 16);
}
