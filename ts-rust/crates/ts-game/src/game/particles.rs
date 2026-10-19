//! Particle groups.

use crate::*;

/// Free a particle group and the groups chained after it (+60): each is taken out of the list of its kind (a doubly linked list through +64 and
/// +68; the head of a kind is at 0x1fe99c0 + 4 * kind, its count at 0x1fe9a00 + 4 * kind) and put on the free list at 0x3b0150 (@ 0x296b08).
pub fn particleFree(group: i32) {
    let mut s = group;
    while s != 0 {
        let kind = lw(s);
        if kind == -1 {
            break;
        }
        if kind == 12 {
            enemyRemoveFromExplosionList();
        }
        let owner = lw(s + 20);
        if owner != 0 {
            sw(owner, 0);
        }
        let mut next = lw(s + 68);
        sw(s + 20, 0);
        if next != 0 {
            sw(next + 64, lw(s + 64));
            next = lw(s + 68);
        }
        let prev = lw(s + 64);
        if prev != 0 {
            sw(prev + 68, next);
        } else {
            sw((kind << 2) + 0x1fe99c0, next);
        }
        let free_head = lw(0x3b0150);
        let count_at = (kind << 2) + 0x1fe9a00;
        sw(s + 64, 0);
        let count = lw(count_at);
        sw(s + 68, free_head);
        sw(count_at, count - 1);
        if free_head != 0 {
            sw(free_head + 64, s);
        }
        sw(0x3b0150, s);
        sw(s, -1);
        s = lw(s + 60);
    }
}
