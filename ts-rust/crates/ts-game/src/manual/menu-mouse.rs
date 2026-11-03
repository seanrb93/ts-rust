//! The mouse in the menus: the page of the front end that is on top has a box for each of its items (position, size, number, flags); the pointer
//! of the window, brought to the picture's own coordinates by the port, makes the box it is over the page's current one (as the game's own pointer
//! does in the MapMaker), and the port turns a click into the X button.
//!
//! The port's window thread hands the pointer over (`menu_pointer_set`); the game thread takes it at the start of the front end's tick.

use crate::*;
use std::sync::atomic::{AtomicI64, AtomicU8, Ordering};

const NONE: i64 = i64::MIN;
static POINTER: AtomicI64 = AtomicI64::new(NONE);
static OVER: AtomicU8 = AtomicU8::new(0);

/// The pointer is at (`x`, `y`) of the 640 by 224 picture (the front end's own coordinates).
pub fn menu_pointer_set(x: i32, y: i32) {
    POINTER.store(((x as i64) << 32) | (y as u32 as i64), Ordering::Relaxed);
}

/// What the pointer was over the last time it moved: 0 nothing, 1 an item of the page (a click chooses it), 2 the Back prompt (a click is the triangle), 3 the Select
/// prompt (a click is the X button on the current item).
pub fn menu_pointer_over() -> u8 {
    OVER.load(Ordering::Relaxed)
}

/// At the start of the tick of the front end: make the item under the pointer the current one of the top page.
pub fn menu_pointer_apply() {
    let packed = POINTER.swap(NONE, Ordering::Relaxed);
    if packed == NONE {
        return;
    }
    let (x, y) = ((packed >> 32) as i32, packed as i32);
    let front = lw(g::front);
    let count = lw(front + 20);
    if count <= 0 {
        OVER.store(0, Ordering::Relaxed);
        return;
    }
    let page = lw(front + 4) + ((count - 1) << 4);
    let (x, y) = (x - lw(g::window), y - lw(g::window + 20));
    let mut box_ = lw(page + 8);
    let mut hit = 0u8;
    if box_ != 0 {
        for _ in 0..lw(page + 12) {
            if lhu(box_ + 10) & 1 == 0 && ((x - lhu(box_)) as u32) < lhu(box_ + 4) as u32 && ((y - lhu(box_ + 2)) as u32) < lhu(box_ + 6) as u32 {
                // (the prompts at the bottom are boxes too: Back has the number 0x8000, Select is the last box of the row)
                let prompt = lhu(box_ + 2) >= 200;
                if lhu(box_ + 8) == 0x8000 {
                    hit = 2;
                } else if prompt && lhu(box_ + 8) == 0 {
                    hit = 3;
                } else {
                    sh(page + 4, lhu(box_ + 8));
                    hit = 1;
                }
                break;
            }
            box_ += 24;
        }
    }
    if std::env::var_os("TS_BOXDBG").is_some() {
        eprintln!("[box] pointer {x},{y} window {},{} page boxes {} hit {hit}", lw(g::window), lw(g::window + 20), lw(page + 12));
        let mut b = lw(page + 8);
        for _ in 0..lw(page + 12) {
            eprintln!("[box]   x {} y {} w {} h {} id {} flags {:#x}", lhu(b), lhu(b + 2), lhu(b + 4), lhu(b + 6), lhu(b + 8), lhu(b + 10));
            b += 24;
        }
    }
    OVER.store(hit, Ordering::Relaxed);
}
