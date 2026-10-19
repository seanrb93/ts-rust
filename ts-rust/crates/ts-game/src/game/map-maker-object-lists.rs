//! The map maker's lists of objects and items.

use crate::*;

/// Free a map maker object: take it out of the list of objects (a list through +56 from `mapobjects`), free its items if it is of kind 1 and put it on
/// the free list at 0x3b0014 (@ 0x2486a0).
pub fn mmFreeSingleObject(object: i32) {
    if lw(g::highlightobj) == object {
        sw(g::highlightobj, 0);
    }
    let kind;
    let head = lw(g::mapobjects);
    if head == object {
        let count = lw(g::nummapobjects) - 1;
        kind = lw(object);
        sw(g::mapobjects, lw(object + 56));
        sw(g::nummapobjects, count);
    } else {
        kind = lw(object);
        if head != 0 {
            let mut previous = head;
            let mut next = lw(previous + 56);
            let mut found = next == object;
            while !found {
                previous = next;
                if previous == 0 {
                    break;
                }
                next = lw(previous + 56);
                found = next == object;
            }
            if found {
                let count = lw(g::nummapobjects);
                sw(previous + 56, lw(object + 56));
                sw(g::nummapobjects, count - 1);
            }
        }
    }
    let mut free_head = lw(0x3b0014);
    if kind == 1 && lw(object + 52) != 0 {
        while lw(object + 52) != 0 {
            mmFreeItem(object, lw(object + 52));
        }
        free_head = lw(0x3b0014);
    }
    let free = lw(g::numfreeobjs);
    sw(0x3b0014, object);
    sw(object + 56, free_head);
    sw(g::numfreeobjs, free + 1);
}

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
