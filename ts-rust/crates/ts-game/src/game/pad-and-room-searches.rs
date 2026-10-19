//! Searches over the pad links and the room boxes of a level.

use crate::*;

/// The pads linked to pad `pad`: written to `out` (up to `max`, but a link is always looked at once); returns how many.  A link is used if
/// flag 1 is set, and is one way when flag 2 is set (from +4 to +8) (@ 0x264a18).
pub fn findpadslinked2pad(pad: i32, out: i32, max: i32) -> i32 {
    let mut count = 0;
    if lw(g::numlinks) <= 0 {
        return 0;
    }
    let links = lw(g::levellinks);
    let mut i = 0;
    loop {
        let link = i * 20 + links;
        if lw(link) & 1 != 0 {
            if lw(link + 4) == pad {
                sw((count << 2) + out, lw(link + 8));
                count += 1;
            }
            if lw(link + 8) == pad && lw(link) & 2 == 0 {
                sw((count << 2) + out, lw(link + 4));
                count += 1;
            }
        }
        let rooms_left = lw(g::numlinks);
        if !(count < max) {
            return count;
        }
        i += 1;
        if i >= rooms_left {
            return count;
        }
    }
}

/// The rooms (numbered from 1) whose floor box (48-byte records after the first, x and z limits at +0, +4, +8 and +12, +20) a box overlaps in x and
/// z, written to `out`; the search ends when `want` rooms are found; returns how many (@ 0x2571c8).
pub fn bgFloorBBIntersectionXZ(bb: i32, out: i32, want: i32) -> i32 {
    let mut count = 0;
    let mut done = false;
    let mut i = 0;
    let mut rooms = lw(g::bgNumRooms);
    if rooms > 0 {
        let boxes = lw(g::roombb);
        let max_x = lwc1(bb + 12);
        loop {
            let b = boxes + (i * 48 + 48);
            let mut next = i + 1;
            if !(max_x < lwc1(b))
                && !(lwc1(b + 12) < lwc1(bb))
                && !(lwc1(bb + 16) < lwc1(b + 4))
                && !(lwc1(bb + 20) < lwc1(b + 8))
                && !(lwc1(b + 20) < lwc1(bb + 8))
            {
                next = i + 1;
                sw((count << 2) + out, next);
                count += 1;
                if count == want {
                    done = true;
                }
                rooms = lw(g::bgNumRooms);
            }
            i = next;
            if i >= rooms || done {
                break;
            }
        }
    }
    count
}
