//! Drawing the pickup positions of a level in the editor of the map maker.

use crate::*;

/// Add a pointer entry to the display list: `count` quadwords at `data` (the tag at the end of the list is advanced by 16 bytes).
fn push_ref(count: i32, data: i32) {
    let tag = lw(g::gfxtag);
    sw(tag + 4, data);
    sw(g::gfxtag, tag + 16);
    sh(tag, count);
}

/// Mark the byte 3 of the tag at the end of the display list (the id of the entry about to be put).
fn mark_tag() {
    sb(lw(g::gfxtag) + 3, 48);
}

/// Draw the pickup positions of the level (their number at `num_pickuppos`; up to 16 in a batch, in packets that draw each as a textured quad from the points at 0x1fce190,
/// 0x1fcf450 and 0x1fd0710) with the depth test and the blend mode and dither matrix of the game's own choice; the settings before are put back (@ 0x272cc8).
pub fn pickupposGfx() {
    let depth = dlSetZB(2);
    let blend = dlSetBlend(2);
    dlSetDitherMatrix(2);
    let total = lw(g::num_pickuppos);
    let mut done = 0;
    if total > 0 {
        let mut left = total - done;
        loop {
            let n = if left >= 17 { 16 } else { left };
            let packet = memdbAlloc((n + 5) << 4);
            let texnum = lw(g::pickuppos_texnum);
            let texture = lw((if -1 >= texnum { 0 } else { texnum }) * 40 + lw(lw(g::thistexbuf) + 4) + 32);
            mark_tag();
            push_ref(8, texture);
            mark_tag();
            sw(packet, 0);
            push_ref(1, packet);
            let mut at = packet + 4;
            sw(at, 0);
            at += 4;
            sw(at, 0);
            at += 4;
            sw(at, n << 16 | 0x6c008000);
            mark_tag();
            at += 4;
            push_ref(n, at);
            let mut i = 0;
            if n > 0 {
                loop {
                    sw(at, if (i ^ (n - 1)) == 0 { 0x8004 } else { 4 });
                    at += 4;
                    sw(at, 0x302e4000);
                    i += 1;
                    at += 4;
                    sw(at, 1042);
                    at += 4;
                    sw(at, 0);
                    at += 4;
                    if i >= n {
                        break;
                    }
                }
            }
            let shifted = n << 18;
            let twice = n << 1;
            let from = done << 4;
            let after = done + n;
            mark_tag();
            sw(at, 0);
            push_ref(1, at);
            at += 4;
            sw(at, 0);
            at += 4;
            sw(at, 0);
            let triple = twice + n;
            at += 4;
            sw(at, shifted | 0x68008018);
            mark_tag();
            at += 4;
            push_ref(triple, done * 48 + 0x1fce190);
            let second = done * 48 + 0x1fcf450;
            mark_tag();
            sw(at, 0);
            done = after;
            push_ref(1, at);
            at += 4;
            sw(at, 0);
            at += 4;
            sw(at, 0);
            at += 4;
            sw(at, shifted | 0x68008098);
            mark_tag();
            at += 4;
            push_ref(triple, second);
            mark_tag();
            push_ref(1, at);
            sw(at, 0);
            at += 4;
            sw(at, 0);
            at += 4;
            sw(at, 0);
            at += 4;
            sw(at, shifted | 0x6e008058);
            mark_tag();
            at += 4;
            push_ref(n, from + 0x1fd0710);
            mark_tag();
            sw(at, 0x14000000);
            push_ref(1, at);
            at += 4;
            sw(at, 0);
            at += 4;
            sw(at + 4, 0);
            sw(at, 0);
            let total_now = lw(g::num_pickuppos);
            left = total_now - done;
            if done >= total_now {
                break;
            }
        }
    }
    dlSetZB(depth);
    dlSetBlend(blend);
}
