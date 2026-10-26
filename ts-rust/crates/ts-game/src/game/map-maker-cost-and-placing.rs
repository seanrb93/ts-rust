//! The cost of the objects of a map in the map maker, and placing a new object.

use crate::*;

/// Work out the cost of the map (`mapcost`) and of the objects in the air (`flyingobjcost`: the cost of each object at +8, and for those of kind 1 the items they hold at +52) and
/// set flag 0x400000 of `editflags` when the two together are 5001 or more, else clear it (@ 0x246980).
pub fn mmUpdateMapCost() {
    let cost = mmMapCost();
    let first = lw(g::usedobj);
    let mut flying = 0;
    sw(g::mapcost, cost);
    sw(g::flyingobjcost, 0);
    if first != 0 {
        let mut object = first;
        let mut own = lw(object + 8);
        loop {
            let kind = lw(object);
            flying += own;
            sw(g::flyingobjcost, flying);
            if kind == 1 {
                let items = lw(object + 52);
                if items != 0 {
                    let count = mmItemListCount(items, -1);
                    flying = lw(g::flyingobjcost) + count;
                    sw(g::flyingobjcost, flying);
                }
            }
            object = lw(object + 56);
            if object == 0 {
                break;
            }
            own = lw(object + 8);
        }
    }
    let flags = lw(g::editflags);
    if (lw(g::mapcost) + flying) < mm_cost_limit() {
        sw(g::editflags, flags & -0x400001);
    } else {
        sw(g::editflags, lw(g::editflags) | 0x400000);
    }
}

/// Put the list of new objects `list` into the map's: it is allowed when the cost of the map and of the list (the cost of each object, and the items of those of kind 1) is
/// under 5001 and the file size with the list (36 for each kind 1 object and 4 for each item) is under 0x1ffd; then the list is put in front of the map's objects and counted
/// (1 for placed, 0 for not) (@ 0x2487d0).
pub fn mmPlaceObject(list: i32) -> i32 {
    let mut objects = 0;
    let mut items = 0;
    let mut cost = 0;
    let mut object = list;
    if list != 0 {
        let mut kind = lw(object);
        loop {
            if kind != 1 {
                object = lw(object + 56);
            } else {
                let held = lw(object + 52);
                objects += 1;
                let count = mmItemListCount(held, -1);
                cost += lw(object + 8);
                if count != 0 {
                    cost += count;
                    items += count;
                }
                object = lw(object + 56);
            }
            if object == 0 {
                break;
            }
            kind = lw(object);
        }
    }
    let map_cost = mmMapCost();
    if (map_cost + cost) < mm_cost_limit() {
        let size = mmCalcFileSize();
        if ((size + objects * 36 + (items << 2)) as u32) < 0x1ffd_u32 {
            let next = lw(list + 56);
            let mut last = list;
            let first_object;
            let count;
            if next == 0 {
                first_object = lw(g::mapobjects);
                count = lw(g::nummapobjects) + 1;
            } else {
                let mut running = lw(g::nummapobjects);
                let mut walk = next;
                first_object = lw(g::mapobjects);
                loop {
                    last = walk;
                    running += 1;
                    let following = lw(last + 56);
                    sw(g::nummapobjects, running);
                    walk = following;
                    if following == 0 {
                        break;
                    }
                }
                count = running + 1;
            }
            sw(last + 56, first_object);
            sw(g::mapobjects, list);
            sw(g::nummapobjects, count);
            return 1;
        }
    }
    0
}
