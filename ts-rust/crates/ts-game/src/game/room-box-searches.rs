//! Which rooms a box overlaps: the rooms' boxes, their floors' boxes and their walls' boxes.

use crate::*;

/// Go through the rooms (numbered from 1; records of `stride` bytes after the first one in the table whose address is the word `table_at`), writing
/// the numbers of those for which `hit` is true to `out`; stops when `want` have been found.  Returns how many were found.  `setup` reads what
/// `hit` needs of the box once, and only when there are rooms (the original does not touch the box otherwise).
fn collect_rooms<T>(table_at: i32, stride: i32, out: i32, want: i32, setup: impl FnOnce() -> T, hit: impl Fn(i32, &T) -> bool) -> i32 {
    let mut count = 0;
    let mut done = false;
    let mut rooms = lw(g::bgNumRooms);
    if rooms > 0 {
        let table = lw(table_at);
        let context = setup();
        let mut i = 0;
        loop {
            let rec = table + (i * stride + stride);
            let mut next = i + 1;
            if hit(rec, &context) {
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

/// The rooms whose floor box (48-byte records at `roombb`) the box `bb` (min x, y, z, then max x, y, z) overlaps; the tests are the game's own,
/// including a second test of the box's top against the floor's top (@ 0x2570d0).
pub fn bgFloorBBIntersection(bb: i32, out: i32, want: i32) {
    collect_rooms(g::roombb, 48, out, want, || lwc1(bb + 12), |b, &max_x| {
        if max_x < lwc1(b) {
            return false;
        }
        if lwc1(b + 12) < lwc1(bb) {
            return false;
        }
        let max_y = lwc1(bb + 16);
        if max_y < lwc1(b + 4) {
            return false;
        }
        if lwc1(b + 16) < max_y {
            return false;
        }
        if lwc1(bb + 20) < lwc1(b + 8) {
            return false;
        }
        !(lwc1(b + 20) < lwc1(bb + 8))
    });
}

/// The rooms (records of 44 bytes, their boxes at +20 to +40) that the box `bb` overlaps (@ 0x256fd0).
pub fn bgRoomBBIntersection(bb: i32, out: i32, want: i32) -> i32 {
    collect_rooms(g::levelrooms, 44, out, want, || lwc1(bb + 12), |r, &max_x| {
        !(max_x < lwc1(r + 20))
            && !(lwc1(r + 32) < lwc1(bb))
            && !(lwc1(bb + 16) < lwc1(r + 24))
            && !(lwc1(r + 36) < lwc1(bb + 4))
            && !(lwc1(bb + 20) < lwc1(r + 28))
            && !(lwc1(r + 40) < lwc1(bb + 8))
    })
}

/// The rooms whose wall box (48-byte records at `roombb`, the box at +24 to +44) the box `bb` overlaps (@ 0x2572b0).
pub fn bgWallBBIntersection(bb: i32, out: i32, want: i32) -> i32 {
    collect_rooms(g::roombb, 48, out, want, || lwc1(bb + 12), |b, &max_x| {
        !(max_x < lwc1(b + 24))
            && !(lwc1(b + 36) < lwc1(bb))
            && !(lwc1(bb + 16) < lwc1(b + 28))
            && !(lwc1(b + 40) < lwc1(bb + 4))
            && !(lwc1(bb + 20) < lwc1(b + 32))
            && !(lwc1(b + 44) < lwc1(bb + 8))
    })
}
