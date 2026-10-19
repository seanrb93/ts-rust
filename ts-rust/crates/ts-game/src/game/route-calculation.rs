//! Working out a route between two pads, and between two halls.

use crate::*;

/// Copy a found route to `out` (at most `max` links, the last one found first): the link that reached the goal, then the links of the entries (16 bytes each from `table`, the
/// way back by the halfword at +2 until it is negative, the link in the halfword at +6); `entry` is the last entry taken (negative when the goal was reached at once); returns
/// the number written.
fn write_route(out: i32, mut max: i32, goal_link: i32, entry: i32, table: i32) -> i32 {
    let mut written = 0;
    if max > 0 {
        max -= 1;
        written = 1;
        sw(out, goal_link);
    }
    let mut entry = entry;
    if entry >= 0 && max > 0 {
        let mut at = entry << 4;
        loop {
            let record = at + table;
            let to = (written << 2) + out;
            written += 1;
            sw(to, lh(record + 6));
            entry = lh(record + 2);
            max -= 1;
            if entry < 0 {
                break;
            }
            at = entry << 4;
            if max <= 0 {
                break;
            }
        }
    }
    written
}

/// Work out the route from the pad `from` to the pad `to` by the links of the level: the pathfinder takes the cheapest open entry (the list at 0x3b00d0, the closed list
/// at 0x3b00d8, entries of 16 bytes at 0x1fca310) and expands it until a link leads to the goal (`r_addnewroutes`); the links of the route are written to `out`, at most
/// `max` of them, and the cost kept in `lastroutecost`; returns the number of links (@ 0x266a40).
pub fn routeCalc(from: i32, to: i32, out: i32, max: i32) -> i32 {
    let mut entry = -1;
    r_initroutes();
    sw(0x3b00e4, to);
    let mut result = r_addnewroutes(-1, from, 0.0_f32);
    let mut record = 0;
    if result < 0 {
        entry = lw(0x3b00d0);
        loop {
            let closed = lhu(0x3b00d8);
            record = (entry << 4) + 0x1fca310;
            let next = lh(record);
            sw(0x3b00d8, entry);
            sw(0x3b00d0, next);
            let cost = lwc1(record + 8);
            let pad = lh(record + 4);
            sh(record, closed);
            result = r_addnewroutes(entry, pad, cost);
            if result >= 0 {
                break;
            }
            entry = lw(0x3b00d0);
        }
    }
    let written = write_route(out, max, result, entry, 0x1fca310);
    let cost;
    if record == 0 {
        cost = lwc1(lw(out) * 20 + lw(g::levellinks) + 16);
    } else {
        cost = lwc1(record + 8);
    }
    swc1(g::lastroutecost, cost);
    written
}

/// The same for the halls (open list 0x3b00b0, closed 0x3b00b8, entries at 0x1fc6490, the goal hall at 0x3b00c8); 0 if no route is found; the cost is kept in
/// `lasthallroutecost` (@ 0x2663e8).
pub fn hallrouteCalc(from: i32, to: i32, out: i32, max: i32) -> i32 {
    let mut entry = -1;
    hr_initroutes();
    let pads = lw(g::levelpads);
    sw(0x3b00c4, to);
    let mut record = 0;
    sw(0x3b00c8, lh(to * 28 + pads + 8));
    let mut result = hr_addnewroutes(-1, from, 0.0_f32);
    let head = lw(0x3b00b0);
    if head < 0 {
        return 0;
    }
    if result < 0 {
        entry = head;
        loop {
            let closed = lhu(0x3b00b8);
            record = (entry << 4) + 0x1fc6490;
            let next = lh(record);
            sw(0x3b00b8, entry);
            sw(0x3b00b0, next);
            let cost = lwc1(record + 8);
            let hall = lh(record + 4);
            sh(record, closed);
            result = hr_addnewroutes(entry, hall, cost);
            let open = lw(0x3b00b0);
            if open < 0 {
                return 0;
            }
            if result >= 0 {
                break;
            }
            entry = open;
        }
    }
    let written = write_route(out, max, result, entry, 0x1fc6490);
    let cost;
    if record == 0 {
        cost = lwc1(lw(out) * 20 + lw(g::levellinks) + 16);
    } else {
        cost = lwc1(record + 8);
    }
    swc1(g::lasthallroutecost, cost);
    written
}
