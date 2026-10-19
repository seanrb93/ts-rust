//! Finding an item of a menu page.

use crate::*;

/// The first item of a menu page (items of 32 bytes from +16, their count at +0) for which `test` is true, looking into the sub-lists of the items with flag 2 (a
/// pointer at +12 to a list with its count, a halfword, and items of 32 bytes from +16); 0 if there is none.
fn find_item(page: i32, test: impl Fn(i32) -> bool) -> i32 {
    let count = lw(page);
    let mut item = page + 16;
    for _ in 0..count.max(0) {
        if lhu(item + 10) & 2 == 0 {
            if test(item) {
                return item;
            }
        } else {
            let sub = lw(item + 12);
            let mut entry = sub + 16;
            for _ in 0..lhu(sub) {
                if test(entry) {
                    return entry;
                }
                entry += 32;
            }
        }
        item += 32;
    }
    0
}

/// The item of a page whose id (the halfword at +8) is `id` (@ 0x22dcd8).
pub fn menuitem(page: i32, id: i32) -> i32 {
    find_item(page, |item| lhu(item + 8) == id & 0xffff)
}

/// The selected item of a page: the first with flag 128 (@ 0x22dd68).
pub fn menuitemGetSelect(page: i32) -> i32 {
    find_item(page, |item| lhu(item + 10) & 128 != 0)
}

/// Make the item with id `id` the selected one: flag 128 is taken from the items that have it and set on that item (@ 0x22ddf8).
pub fn menuitemSelect(page: i32, id: i32) {
    loop {
        let selected = menuitemGetSelect(page);
        if selected == 0 {
            break;
        }
        sh(selected + 10, lhu(selected + 10) & 0xff7f);
    }
    let item = menuitem(page, id & 0xffff);
    if item != 0 {
        sh(item + 10, lhu(item + 10) | 128);
    }
}
