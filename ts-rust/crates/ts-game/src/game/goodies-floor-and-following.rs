//! What the goodies page shows, whether a character is available, the floor and room under a point, and an enemy following its target.

use crate::*;

/// Which of the miscellaneous goodies are unlocked: bit 1 challenge mode, bit 2 the weapon set 8 (@ 0x2402d8).
pub fn goodiesGetMisc() -> i32 {
    let mut bits = if weaponsetAvail(8) != 0 { 2 } else { 0 };
    let with_challenge = bits | 1;
    if challengeModeAvail() != 0 {
        bits = with_challenge & 255;
    }
    bits
}

/// Which of the bot sets 0 to 7 are available, as the bits 9 to 16 (@ 0x2404b0).
pub fn goodiesGetBotSets() -> i32 {
    let mut bits = 0;
    let mut number = 9;
    loop {
        let available = botsetAvail(number - 9);
        let bit = 1 << (number & 31);
        if available != 0 {
            bits |= bit;
        }
        number += 1;
        if number >= 17 {
            break;
        }
    }
    bits
}

/// Which of the 64 characters (in the order of `characterorder`) can be played, as a bit each in the three words at `out` (@ 0x240418).
pub fn goodiesGetPlayableChrs(out: i32) {
    sw(out, 0);
    sw(out + 4, 0);
    sw(out + 8, 0);
    let mut i = 0;
    loop {
        let chr = lbu(i + g::characterorder);
        let playable = chrPlayable(chr);
        let word = ((i >> 5) << 2) + out;
        let bit = 1 << (i & 31 & 31);
        i += 1;
        if playable != 0 {
            sw(word, lw(word) | bit);
        }
        if !((i as u32) < 64) {
            break;
        }
    }
}

/// Whether character `chr` is available: when it is playable (with flag 2 of the player array), when it belongs to a bot set that is available (the ten characters of each of the
/// 8 presets of 44 bytes from `chrpresets` + 396), the characters 20 to 24 by their own items, and the rest by `chrPlayable` (@ 0x224c00).
pub fn chrAvail(chr: i32) -> i32 {
    let offset = chr - 20;
    if lw((chr << 4) + g::unlock_playable_chrs) != 0 && lb(g::pcarray + 2) != 0 {
        return 1;
    }
    if (offset as u32) >= 5 {
        let mut set = 0;
        let mut preset = g::chrpresets + 396;
        loop {
            if botsetAvail(set) != 0 {
                let mut k = 0;
                let mut at = preset;
                loop {
                    k += 1;
                    if lw(at) == chr {
                        return 1;
                    }
                    at += 4;
                    if k >= 10 {
                        break;
                    }
                }
            }
            set += 1;
            preset += 44;
            if set >= 8 {
                break;
            }
        }
        return chrPlayable(chr);
    }
    match offset {
        0 => unlockedEx(g::unlock_botsetup_duckman, 0),
        1 => unlockedEx(g::unlock_botsetup_impersonator, 0),
        2 => unlockedEx(g::unlock_botsetup_gingerbread, 0),
        3 => unlockedEx(g::unlock_botsetup_funbunny, 0),
        4 => unlockedEx(g::unlock_botsetup_robofish, 0),
        _ => unreachable!(),
    }
}

/// Make the box round the point `pos` (the sides `radius` away in x and z, from the point's height up by `height`) and the point itself (raised by `height`) in the scratch area
/// at `box_at` and `point_at`; find the rooms the box touches (at most 20) into `rooms`; returns their number.
fn floor_search_setup(pos: i32, height: f32, radius: f32, box_at: i32, point_at: i32, rooms: i32) -> i32 {
    let x = lwc1(pos);
    let z = lwc1(pos + 8);
    let y = lwc1(pos + 4);
    swc1(box_at + 4, y);
    swc1(box_at + 12, x + radius);
    swc1(box_at + 20, z + radius);
    swc1(box_at + 16, y + height);
    swc1(box_at, x - radius);
    swc1(box_at + 8, z - radius);
    let found = bgFloorBBIntersectionXZ(box_at, rooms, 20);
    let top = lwc1(pos + 4) + height;
    let x = lwc1(pos);
    swc1(point_at + 8, lwc1(pos + 8));
    swc1(point_at, x);
    swc1(point_at + 4, top);
    found
}

/// The height of the highest floor under the point `pos` (raised by `height`, within `radius` sideways) over the rooms the box round it touches: the floor item and prop are
/// written to `out_item` and `out_prop` (when not 0); -1 when there is none (@ 0x27c9f0).
pub fn moveFindFloor(pos: i32, out_item: i32, out_prop: i32, height: f32, radius: f32) -> f32 {
    let frame = Frame::new(336);
    let sp = frame.sp();
    let mut best_prop = 0;
    let mut best_item = 0;
    let mut best = lwc1(0x3af9cc);
    let count = floor_search_setup(pos, height, radius, sp + 16, sp, sp + 48);
    if count > 0 {
        let mut i = 0;
        sw(sp + 144, 0);
        loop {
            sw(sp + 148, 0);
            let room = lw(sp + 48 + (i << 2));
            let h = moveFindFloorRoom(room, sp, sp + 144, sp + 148);
            i += 1;
            if best < h {
                best_item = lw(sp + 144);
                best = h;
                best_prop = lw(sp + 148);
            }
            if i >= count {
                break;
            }
            sw(sp + 144, 0);
        }
    }
    if out_item != 0 {
        sw(out_item, best_item);
    }
    if out_prop != 0 {
        sw(out_prop, best_prop);
    }
    if best_item != 0 { best } else { -1.0_f32 }
}

/// The room that holds the point `pos` (raised by `height`, within `radius` sideways): the room whose floor is the highest under it, found from the rooms the box touches and then
/// followed through the portals from the highest floor up to the point; the floor height is kept at `lastfloorheight` (@ 0x27c870).
pub fn moveFindRoom(pos: i32, height: f32, radius: f32) -> i32 {
    let frame = Frame::new(288);
    let sp = frame.sp();
    let mut best_room = 0;
    let mut best = lwc1(g::lastfloorheight + 4);
    let x = lwc1(pos);
    let z = lwc1(pos + 8);
    let y = lwc1(pos + 4);
    swc1(sp, x);
    swc1(sp + 20, y);
    let top = y + height;
    swc1(sp + 28, x + radius);
    swc1(sp + 36, z + radius);
    swc1(sp + 16, x - radius);
    swc1(sp + 32, top);
    swc1(sp + 24, z - radius);
    swc1(sp + 4, top);
    swc1(sp + 8, z);
    let count = bgFloorBBIntersectionXZ(sp + 16, sp + 48, 20);
    let mut previous = lwc1(g::lastfloorheight + 4);
    if count > 0 {
        let mut at = sp + 48;
        let mut left = count;
        loop {
            let room = lw(at);
            let h = moveFindFloorRoom(room, sp, 0, 0);
            left -= 1;
            if best < h {
                best_room = lw(at);
                best = h;
            }
            at += 4;
            if left == 0 {
                break;
            }
        }
        previous = lwc1(g::lastfloorheight + 4);
    }
    let mut current = best_room;
    if !(best == previous) {
        let px = lwc1(pos);
        let mut skip = 0;
        let pz = lwc1(pos + 8);
        let below = lwc1(pos + 4) - best;
        let mut result = 0;
        swc1(sp + 160, px);
        swc1(sp + 148, below);
        swc1(sp + 144, px - px);
        swc1(sp + 152, pz - pz);
        swc1(sp + 164, best);
        swc1(sp + 168, pz);
        if best_room != 0 {
            loop {
                result = bgPortalCrossed(sp + 160, sp + 144, current, skip, 0);
                skip = current;
                current = result;
                if result == skip {
                    break;
                }
            }
        }
        best_room = result;
    }
    if best_room > 0 {
        swc1(g::lastfloorheight, best);
    }
    best_room
}

/// An enemy follows its target: the pad of its target is kept at +684 of its state (the route is made again when it changed on the tick of the enemy's own turn); `findNextPad`
/// gives the next pad on the way, and the enemy goes to move mode 8 when there is none, to state 128 when there is no route, or else moves to that pad (and, when it
/// has been near the pad for the time given, goes to state 256) (@ 0x2c1480).
pub fn followTarget(chr: i32) {
    let state = lw(chr + 352);
    let target_pad = enemyGetTargetPad(chr);
    let turn = lw(g::updatedistindex);
    if lw(state + 684) != target_pad && turn == lw(state) {
        sw(state + 1688, 0);
        sw(state + 2692, 0);
    }
    sw(state + 684, target_pad);
    let next = findNextPad(state + 680);
    if next == -2 {
        let old = lw(state + 2716);
        sw(state + 2716, 128);
        sw(state + 2720, old);
        sw(state + 2892, 0);
        return;
    }
    if next == -1 {
        enemySetMoveMode(chr, 8);
        return;
    }
    enemySetMoveMode(chr, 1);
    let pads = lw(g::levelpads);
    swc1(state + 2872, lwc1(state + 2856));
    let near = lwc1(0x3ad8b8);
    let pad = next * 28 + pads;
    let x = lwc1(pad + 12);
    swc1(state + 2796, x);
    let y = lwc1(pad + 16);
    swc1(state + 2800, y);
    let z = lwc1(pad + 20);
    swc1(state + 2804, z);
    let dx = lwc1(chr + 48) - x;
    let dy = lwc1(chr + 52) - y;
    let dz = lwc1(chr + 56) - z;
    let waited;
    if !((dx * dx + dy * dy + dz * dz) < near) {
        waited = lwc1(state + 2892);
    } else {
        let count = lw(state + 1688);
        sw(state + 680, next);
        sw(state + 1688, count - 1);
        waited = lwc1(state + 2892);
    }
    if !(lwc1(0x3ad8bc) < waited) {
        return;
    }
    if lw(state + 2704) & 1 == 0 {
        return;
    }
    let old = lw(state + 2716);
    sw(state + 2724, 2048);
    sw(state + 2716, 256);
    sw(state + 2720, old);
    sw(state + 2892, 0);
}
