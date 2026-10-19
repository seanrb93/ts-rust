//! The names of the map maker's item groups.

use crate::*;

/// The text for item group `group` in the current language: groups 0, 5, 6 and 7 have texts of their own (at 4080, 4084, 4088 and 4092 bytes into the
/// table of texts), the others are found by the group in the item definitions (16 bytes each, 20 of them; text number at +0, group at +4); 0 if none (@ 0x248d98).
pub fn mmItemGroupName(group: i32) -> i32 {
    let texts = |offset: i32| lw(lw((lw(g::cur_language) << 2) + g::languages) + offset);
    match group {
        0 => return texts(4080),
        5 => return texts(4084),
        6 => return texts(4088),
        7 => return texts(4092),
        _ => {}
    }
    for i in 0..20 {
        let def = g::itemdefs + (i << 4);
        if lw(def + 4) == group {
            return lw((lh(def) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
        }
    }
    0
}
