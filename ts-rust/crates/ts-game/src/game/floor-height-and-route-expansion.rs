//! The height of the floor under a point, and expanding the routes of the pathfinder from a pad.

use crate::*;

/// The height of the floor under the point `point` (x, y, z): among the items of the list `list` (a null-ended list of pointers) whose kind (the halfword at +4) is 0 or has a bit of
/// `mask` and, if the item has flag 16, whose height (+12) is under the point's, those whose outline of `n` points (12 bytes each from +8, tested in x and z) holds the point
/// give the height (the item's own at +12 with flag 16, else by `calcfloorheight` on its three first points); the highest below the point's own height is returned, and
/// the item that gave it is written to `out` (not when 0) (@ 0x27c578).
pub fn moveFindFloorHeight(list: i32, mask: i32, point: i32, out: i32) -> f32 {
    let mut best = 0;
    let mut index = 0;
    let mut item = lw(list);
    let mut height = lwc1(g::curprop + 4);
    if item != 0 {
        loop {
            let kind = lhu(item + 4);
            let next = index + 1;
            let mut skip = kind != 0 && (kind & mask) == 0;
            let mut flags = 0;
            let mut n = 0;
            if !skip {
                flags = lhu(item + 6);
                if flags & 16 != 0 && !(lwc1(item + 12) < lwc1(point + 4)) {
                    skip = true;
                }
                n = lw(item);
            }
            if !skip {
                let mut inside = 0;
                let z = lwc1(point + 8);
                let mut last = item + (n * 12 - 4);
                let mut last_above = if z <= lwc1(last + 8) { 1 } else { 0 };
                let mut k = 0;
                if n != 0 {
                    loop {
                        let vertex = item + (k * 12 + 8);
                        let vz = lwc1(vertex + 8);
                        let above = if z <= vz { 1 } else { 0 };
                        if last_above == above {
                            k += 1;
                        } else {
                            let vx = lwc1(vertex);
                            let crossing = (((vx - lwc1(point)) * (lwc1(last + 8) - vz)) <= (lwc1(last) - vx) * (vz - z)) as i32;
                            k += 1;
                            if crossing == above {
                                inside = 1 - inside;
                            }
                        }
                        last = vertex;
                        last_above = above;
                        if (k as u32) >= (n as u32) {
                            break;
                        }
                    }
                }
                if inside != 0 {
                    let floor;
                    if flags & 16 == 0 {
                        let x = lwc1(point);
                        let pz = lwc1(point + 8);
                        floor = calcfloorheight(item + 8, item + 20, item + 32, x, pz);
                    } else {
                        floor = lwc1(item + 12);
                    }
                    if height < floor && floor < lwc1(point + 4) {
                        height = floor;
                        best = item;
                    }
                }
            }
            index = next;
            item = lw((index << 2) + list);
            if item == 0 {
                break;
            }
        }
    }
    if out != 0 {
        sw(out, best);
    }
    height
}

/// Expand the routes of the pathfinder from the pad `pad` (reached at cost `cost`): for each link of the pad's hall (the links after the hall's rooms, `n` of them at +8) that is
/// in use (flag 1) and goes from the pad (or, for a link without flag 2, to it), work out its cost and the estimated cost to the goal at 0x3b00e4; a link that leads to the goal
/// is returned at once; otherwise a better route replaces a worse one of the open or the closed list (at 0x3b00d0 and 0x3b00d8), and a new one is added. -1 if none got to the goal (@ 0x266810).
pub fn r_addnewroutes(routes: i32, pad: i32, cost: f32) -> i32 {
    let mut count = 0;
    let hall = lw((lh(lw(g::levelpads) + pad * 28 + 8) << 2) + lw(g::halls));
    let n = lw(hall + 8);
    let links = hall + (((lw(hall) + lw(hall + 4)) << 2) + 12);
    if n > 0 {
        let mut at = links;
        let mut link_number = lw(at);
        loop {
            let link = lw(g::levellinks) + link_number * 20;
            let flags = lw(link);
            let mut handled = false;
            if flags & 1 != 0 {
                let from = lw(link + 4);
                let other;
                if from != pad {
                    if lw(link + 8) != pad || flags & 2 != 0 {
                        handled = true;
                        other = 0;
                    } else {
                        other = from;
                    }
                } else {
                    other = lw(link + 8);
                }
                if !handled {
                    if other == lw(0x3b00e4) {
                        return link_number;
                    }
                    let link_cost = r_calclinkcost(link);
                    let total = cost + link_cost;
                    let goal = lw(0x3b00e4);
                    let estimate = r_calcestcost(other, goal);
                    let with_estimate = total + estimate;
                    let mut entry = r_findroute(0x3b00d0, link_number);
                    let mut replace_in = 0;
                    if entry < 0 {
                        entry = r_findroute(0x3b00d8, link_number);
                        if entry < 0 {
                            r_addroute(routes, other, link_number, total, with_estimate);
                        } else if with_estimate < lwc1(entry * 16 + 0x1fca31c) && lh(entry * 16 + 0x1fca314) == other {
                            replace_in = 0x3b00d8;
                        }
                    } else if with_estimate < lwc1(entry * 16 + 0x1fca31c) {
                        replace_in = 0x3b00d0;
                    }
                    if replace_in != 0 {
                        r_deleteroute(replace_in, entry);
                        r_addroute(routes, other, link_number, total, with_estimate);
                    }
                }
            }
            count += 1;
            at += 4;
            if count >= n {
                break;
            }
            link_number = lw(at);
        }
    }
    -1
}

/// The same for the pathfinder of the halls (open list at 0x3b00b0, closed at 0x3b00b8, goal 0x3b00c4): the links count are the hall's second number, a link from a pad of another
/// hall needs no flag 2; at the end the route to the hall in 0x3b00c8 is returned (@ 0x2661a8).
pub fn hr_addnewroutes(routes: i32, pad: i32, cost: f32) -> i32 {
    let hall_number = lh(lw(g::levelpads) + pad * 28 + 8);
    let hall = lw((hall_number << 2) + lw(g::halls));
    let n = lw(hall + 4);
    let links = hall + ((lw(hall) << 2) + 12);
    if n > 0 {
        let mut at = links;
        let mut left = n;
        let mut link_number = lw(at);
        loop {
            let link = lw(g::levellinks) + link_number * 20;
            let flags = lw(link);
            if flags & 1 != 0 {
                let from = lw(link + 4);
                let mut skip = false;
                let to;
                let estimate_from;
                if lh(from * 28 + lw(g::levelpads) + 8) != hall_number {
                    if flags & 2 != 0 {
                        skip = true;
                        to = 0;
                        estimate_from = 0;
                    } else {
                        estimate_from = lw(link + 8);
                        to = from;
                    }
                } else {
                    estimate_from = from;
                    to = lw(link + 8);
                }
                if !skip {
                    let estimate = hr_calcestcost(pad, estimate_from);
                    let link_cost = hr_calclinkcost(link);
                    let goal = lw(0x3b00c4);
                    let total = cost + estimate + link_cost;
                    let remaining = hr_calcestcost(to, goal);
                    let with_estimate = total + remaining;
                    let mut entry = hr_findroute(0x3b00b0, link_number);
                    let mut replace_in = 0;
                    if entry >= 0 {
                        if with_estimate < lwc1(entry * 16 + 0x1fc649c) {
                            replace_in = 0x3b00b0;
                        }
                    } else {
                        entry = hr_findroute(0x3b00b8, link_number);
                        if entry < 0 {
                            hr_addroute(routes, to, link_number, total, with_estimate);
                        } else if with_estimate < lwc1(entry * 16 + 0x1fc649c) && lh(entry * 16 + 0x1fc6494) == to {
                            replace_in = 0x3b00b8;
                        }
                    }
                    if replace_in != 0 {
                        hr_deleteroute(replace_in, entry);
                        hr_addroute(routes, to, link_number, total, with_estimate);
                    }
                }
            }
            at += 4;
            left -= 1;
            if left == 0 {
                break;
            }
            link_number = lw(at);
        }
    }
    hr_findroutetohall(0x3b00b0, lw(0x3b00c8))
}
