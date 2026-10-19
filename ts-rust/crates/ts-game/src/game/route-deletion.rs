//! Taking entries out of the route tables of the bots (the add side is in `route-tables-and-kills.rs`).

use crate::*;

/// Take entry `entry` out of the route list whose first entry is the number in the word `head_at` and put it on the free list (the first free
/// entry is the word `free_at`).  The code the two route finders share.
fn delete_route(table: i32, free_at: i32, head_at: i32, entry: i32) {
    let head = lw(head_at);
    let old_free = lhu(free_at);
    let entry_rec = (entry << 4) + table;
    if head == entry {
        sw(head_at, lh(entry_rec));
    } else if head >= 0 {
        let mut at = head;
        loop {
            let rec = (at << 4) + table;
            let next = lh(rec);
            if next == entry {
                sh(rec, lhu(entry_rec));
                break;
            }
            at = next;
            if at < 0 {
                break;
            }
        }
    }
    sw(free_at, entry);
    sh(entry_rec, old_free);
}

/// @ 0x266058
pub fn hr_deleteroute(head_at: i32, entry: i32) {
    delete_route(0x1fc6490, 0x3b00c0, head_at, entry);
}

/// @ 0x2666c0
pub fn r_deleteroute(head_at: i32, entry: i32) {
    delete_route(0x1fca310, 0x3b00e0, head_at, entry);
}
