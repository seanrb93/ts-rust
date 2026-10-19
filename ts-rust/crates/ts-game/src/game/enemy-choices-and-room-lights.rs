//! How an enemy chooses a health pickup, how a random pad is chosen, the depth of rooms from a portal, and the lights of rooms.

use crate::*;

/// A random number from 0 up to 1 (the 32 bits of `newrnd` as an unsigned number times 2^-32).
fn random_unit() -> f32 {
    let r = newrnd();
    (r as i32 as u32) as f32 * 2.3283064e-10_f32
}

/// Whether an enemy goes for a health pickup, and which: when its health fraction (+520, times its value at +12 plus 1) is below a limit, a random number is below its
/// chance (+2832) and the best pickup (by nearness, doubled when it matters for that health) is worth it; the enemy's target (+2788) is set to the pickup and
/// the number (the first word of the entry) is returned, else -1 (@ 0x2c5140).
pub fn enemyCheckHealth(enemy: i32) -> i32 {
    let data = lw(enemy + 352);
    let mut best = 0.0_f32;
    let need = lwc1(enemy + 520) * (lwc1(data + 12) + 1.0);
    let mut chosen = -1;
    if !(lwc1(0x3ad96c) < need) {
        let r = random_unit();
        let count = lw(g::numPickupHealth);
        if !(r < lwc1(data + 2832)) {
            for i in 0..count.max(0) {
                let entry = i << 3;
                let pickup = lw(entry + (g::healthspawn + 4));
                if lw(pickup + 16) & 1 == 0 {
                    continue;
                }
                let dx = lwc1(enemy + 48) - lwc1(pickup + 48);
                let dy = lwc1(enemy + 52) - lwc1(pickup + 52);
                let dz = lwc1(enemy + 56) - lwc1(pickup + 56);
                let kind = lw(lw(pickup + 352));
                let mut nearness = 1.0_f32 / (dx * dx + dy * dy + dz * dz).abs().sqrt();
                let health = lwc1(enemy + 520);
                let wants = if kind != 4 { lwc1(0x3ad974) <= health } else { health <= lwc1(0x3ad970) };
                if wants {
                    nearness += nearness;
                }
                if best < nearness {
                    best = nearness;
                    chosen = lw(entry + g::healthspawn);
                    sw(data + 2788, lw(entry + (g::healthspawn + 4)));
                }
            }
            if chosen == -1 {
                return -1;
            }
            if lwc1(0x3ad978) < (1.0 - need) * best {
                return chosen;
            }
        }
    }
    -1
}

/// The external reference of a random pad within a distance band of the point `pos`: `flags` has 0x40000000 and the least distance in the low byte, 0x20000000
/// and the greatest in the next byte; tries half as many times as there are pads; level 100 only takes pads with flag 16; 0 if none was found (@ 0x2283b0).
pub fn lvGetRandomPadExtref(flags: i32, pos: i32) -> i32 {
    let mut least2 = 0.0_f32;
    let mut greatest2 = 0.0_f32;
    let mut found = 0;
    let mut pad = 0;
    let tries = (lw(g::numpads) as f32 * 0.5_f32) as i32;
    if flags & 0x40000000 != 0 {
        let d = (flags & 255) as f32;
        least2 = d * d;
    }
    if flags & 0x20000000 != 0 {
        let d = ((flags & 0xff00) >> 8) as f32;
        greatest2 = d * d;
    }
    let mut tried = 0;
    if tries > 0 {
        loop {
            let r = random_unit();
            let count = lw(g::numpads) as f32;
            let mut ok = 1;
            let (py, pz) = (lwc1(pos + 4), lwc1(pos + 8));
            pad = lw(g::levelpads) + (count * r) as i32 * 28;
            let dx = lwc1(pad + 12) - lwc1(pos);
            let dy = lwc1(pad + 16) - py;
            let dz = lwc1(pad + 20) - pz;
            let d2 = dx * dx + dy * dy + dz * dz;
            if greatest2 < d2 {
                ok = 0;
            }
            let level = lw(g::lvlevelnum);
            if ok != 0 && d2 < least2 {
                ok = 0;
            }
            found = if pad == 0 { 0 } else { ok };
            if level == 100 {
                found = if lw(pad + 4) & 16 == 0 { 0 } else { found };
            }
            tried = (if found != 0 { tries } else { tried }) + 1;
            if tried >= tries {
                break;
            }
        }
    }
    if found != 0 { lw(pad) } else { 0 }
}

/// Work out the depth of the rooms seen through the portals of `room` for the first player: `depth` is how many portals away it is; the rooms already reached
/// (the array at 0x3b0084) are not entered again; a room is marked with the least depth, and the greatest depth is kept (@ 0x2579f8).
pub fn bgPortalCalcRoomDepth(room: i32, depth: i32) {
    let list = lw(room * 44 + lw(g::levelrooms) + 4);
    let count = lw(list);
    let reached = lw(0x3b0084);
    for i in 0..depth.max(0) {
        if lw((i << 2) + reached) == room {
            return;
        }
    }
    let player = lw(g::currentplayer);
    sw((depth << 2) + reached, room);
    let entry = room * 20 + lw((lw(player) << 2) + g::roomdata);
    let current = lw(entry);
    if current != 0 && current < depth {
        sw(entry, depth);
        let max_at = (lw(player) << 2) + g::maxlevel;
        if lw(max_at) < depth {
            sw(max_at, depth);
        }
        for k in 0..count.max(0) {
            let number = lw(list + 4 + 4 * k);
            let portal = lw((number << 2) + lw(g::levelportals));
            if lw(lw(g::portaldata) + number * 48 + 40) != 0 {
                continue;
            }
            if bgPortalBackFaceTest(room, lw(g::currentplayer) + 884, portal) != 0 {
                continue;
            }
            let mut other = lw(portal);
            if other == room {
                other = lw(portal + 4);
            }
            bgPortalCalcRoomDepth(other, depth + 1);
        }
    }
}

/// Set the lights of all rooms (records of 28 bytes after the first) from the level's initial lights (24 bytes each) or to white, and give each room's model
/// its override records (two a part, 24 bytes, from the table `overrides`), cleared (@ 0x2a7840).
pub fn roomlightRestart() {
    let mut next_override = 0;
    let mut i = 0;
    if lw(g::bgNumRooms) > 0 {
        let lights = lw(g::roomlights);
        let props = lw(g::roomprops);
        let initial = lw(g::roomlightinitial);
        let overrides = lw(g::overrides);
        loop {
            let prop = lw((i << 2) + props + 4);
            let light = lights + (i * 28 + 28);
            if initial == 0 {
                sw(light + 20, -1);
                swc1(light + 8, 1.0);
                sw(light + 12, 0xf0f0f);
                sw(light, 0);
                sw(light + 4, 0);
                sw(light + 16, -1);
            } else {
                let source = i * 24 + initial;
                sw(light + 12, 0xf0f0f);
                sw(light, lw(source));
                swc1(light + 4, lwc1(source + 4));
                sw(light + 16, lw(source + 12));
                swc1(light + 8, lwc1(source + 8));
                sw(light + 20, lw(source + 16));
            }
            let model = lw(prop + 32);
            sw(model + 248, next_override * 48 + overrides);
            let parts = lw(lw(model));
            next_override += parts;
            i += 1;
            if (parts << 1) > 0 {
                let mut k = 0;
                loop {
                    let record = k * 24 + lw(model + 248);
                    k += 1;
                    let more = k < (lw(lw(model)) << 1);
                    for at in [20, 0, 4, 8, 12, 16] {
                        sw(record + at, 0);
                    }
                    if !more {
                        break;
                    }
                }
            }
            if i >= lw(g::bgNumRooms) {
                break;
            }
        }
    }
}
