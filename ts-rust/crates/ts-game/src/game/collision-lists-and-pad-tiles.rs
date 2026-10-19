//! The map maker's free item list, the lists of props that collide, and the link tiles of the pad editor.

use crate::*;

/// Put item `item` back on the free list of the map maker; if `owner` is not 0 it is first taken out of the owner's list (head at +52, links at +4) (@ 0x248a18).
pub fn mmFreeItem(owner: i32, item: i32) {
    let head = lw(0x3b0018);
    let count = lw(0x3b001c);
    if owner != 0 {
        let mut previous = 0;
        let mut current = lw(owner + 52);
        if current != 0 && current != item {
            previous = current;
            loop {
                current = lw(previous + 4);
                if current == 0 || current == item {
                    break;
                }
                previous = current;
            }
        }
        if current == item {
            let next = lw(item + 4);
            if previous == 0 {
                sw(owner + 52, next);
            } else {
                sw(previous + 4, next);
            }
        }
    }
    sw(item + 4, head);
    sw(0x3b0018, item);
    sw(0x3b001c, count + 1);
}

/// Take a prop out of the list of the props that push (`0x1fd3410`, the count at 0x3b00ec) or of those that do not (`0x1fd2f60`, the count at
/// 0x3b00e8): its entry is cleared and the count of cleared entries goes up (at 0x3b00f4 and 0x3b00f0) (@ 0x2693c8).
pub fn propRemovePropFromCollisionList(prop: i32) {
    let can_push = propDoCollisionsCanPush(lw(prop + 4));
    let pushers = lw(0x3b00ec);
    let (list, count, cleared_at) = if can_push == 0 {
        if propDoCollisionsCanNotPush(lw(prop + 4)) == 0 {
            return;
        }
        (0x1fd2f60, lw(0x3b00e8), 0x3b00f0)
    } else {
        (0x1fd3410, pushers, 0x3b00f4)
    };
    for i in 0..count.max(0) {
        let slot = (i << 2) + list;
        if lw(slot) == prop {
            sw(slot, 0);
            sw(cleared_at, lw(cleared_at) + 1);
            return;
        }
    }
}

/// Make links between the pads whose flags say they plug into each other (the link tiles): for each pad with a plug flag (480) not yet linked (512), the
/// twin pad found by `findTwinPlugPad` in another hall gets a link of its own (@ 0x264fd0).
pub fn padLinkTileSetPads() {
    let mut i = 0;
    while i < lw(g::numpads) {
        let pad = i * 28 + lw(g::levelpads);
        let flags = lw(pad + 4);
        if flags & 480 != 0 && flags & 512 == 0 {
            let hall = lh(pad + 8);
            let plug = ((flags & 384) as u32 >> 2) as i32 | (flags & 96) << 2;
            let twin = findTwinPlugPad(pad + 12, plug, hall);
            if twin != -1 {
                let pads = lw(g::levelpads);
                let this = i * 28 + pads;
                let other = twin * 28 + pads;
                let n = lw(g::numlinks);
                if lh(other + 8) != lh(this + 8) {
                    let links = lw(g::levellinks);
                    let between = lw(0x3b00a8);
                    sw(g::numlinks, n + 1);
                    let link = links + n * 20;
                    sw(link, 1);
                    sw(link + 8, twin);
                    sw(link + 4, i);
                    sw(link + 16, 0);
                    let old = lw(other + 4);
                    sw(link + 12, 0);
                    sw(other + 4, old | 512);
                    sw(this + 4, lw(this + 4) | 512);
                    sw(0x3b00a8, between + 1);
                }
            }
        }
        i += 1;
    }
}
