//! The mouse in the menus: what the pointer of the window is over becomes the choice of the page, and a click becomes a button of the pad.
//!
//! Two kinds of things can be clicked.  The items of a page of the front end have a box each (position, size, number, flags): the box under the pointer
//! becomes the page's current one, as the game's own pointer does in the MapMaker.  The other things are noted when the game draws them (`zone_note`):
//! the arrows of a row (the pictures 20 and 21), a button's picture with the words after it (a prompt such as "X Select", the answers of a question box)
//! and the items of the menu that opens over a level.  The port's window thread hands the pointer over (`menu_pointer_set`); the game thread takes it at
//! the start of the tick of the front end or of the menu.

use crate::*;
use std::sync::atomic::{AtomicI64, AtomicU8, Ordering};
use std::sync::Mutex;

const NONE: i64 = i64::MIN;
static POINTER: AtomicI64 = AtomicI64::new(NONE);
static OVER: AtomicU8 = AtomicU8::new(0);

/// A place of the picture that can be clicked: its kind (`menu_pointer_over`'s numbers 4 to 10, and 11 for the text of a box of a page, whose number is `item`), its corners and, for the item of a menu, the item.
#[derive(Clone, Copy)]
struct Zone {
    kind: u8,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    item: i32,
}

/// The zones drawn since the last page was drawn, and those of the page drawn before them (what the pointer is compared with).
static DRAWN: Mutex<Vec<Zone>> = Mutex::new(Vec::new());
static SHOWN: Mutex<Vec<Zone>> = Mutex::new(Vec::new());

/// The pointer is at (`x`, `y`) of the 640 by 224 picture (the front end's own coordinates).
pub fn menu_pointer_set(x: i32, y: i32) {
    POINTER.store(((x as i64) << 32) | (y as u32 as i64), Ordering::Relaxed);
}

/// What the pointer was over the last time it moved: 0 nothing, 1 an item of a page (a click chooses it), 2 the Back box, 3 a box of the prompts' row with no
/// button (X), 4 and 5 the left and the right arrow of a row, 6 to 9 the words of a button's prompt (cross, triangle, circle, square), 10 an item of the menu
/// over a level (X).
pub fn menu_pointer_over() -> u8 {
    OVER.load(Ordering::Relaxed)
}

/// A page is about to be drawn: what was drawn before is what the pointer is compared with, and the drawing starts the list again.
pub fn zones_begin() {
    let mut drawn = DRAWN.lock().unwrap();
    let mut shown = SHOWN.lock().unwrap();
    shown.clear();
    shown.extend(drawn.drain(..));
}

/// The game draws something that can be clicked (only while a page or a menu is up).
pub fn zone_note(kind: u8, x0: i32, y0: i32, x1: i32, y1: i32, item: i32) {
    if lw(lw(g::front) + 20) == 0 && lw(g::curmenu) == 0 && lw(g::gamestate) == 3 {
        return;
    }
    let mut drawn = DRAWN.lock().unwrap();
    if drawn.len() < 128 {
        drawn.push(Zone { kind, x0, y0, x1, y1, item });
    }
}

/// The box of a page is about to be drawn: the window that the page has now is where its items are, so the box is noted at its place on the screen (the page's
/// window is not the same when the pointer is taken, at the start of the tick).
pub fn zone_note_box(box_: i32) {
    let (w, h, id, flags) = (lhu(box_ + 4) as i32, lhu(box_ + 6) as i32, lhu(box_ + 8) as i32, lhu(box_ + 10));
    if w == 0 || h == 0 || flags & 1 != 0 {
        return;
    }
    let (x, y) = (lw(g::window) + lhu(box_) as i32, lw(g::window + 20) + lhu(box_ + 2) as i32);
    // (the prompts at the bottom are boxes too: Back has the number 0x8000, Select is the last box of the row)
    let kind = if id == 0x8000 {
        2
    } else if id == 0 && y >= 200 {
        3
    } else {
        11
    };
    zone_note(kind, x, y, x + w, y + h, id);
}

/// At the start of the tick of the front end or of the menu: take the pointer's new place.
pub fn menu_pointer_apply() {
    let packed = POINTER.swap(NONE, Ordering::Relaxed);
    if packed == NONE {
        return;
    }
    let (px, py) = ((packed >> 32) as i32, packed as i32);
    let front = lw(g::front);
    let count = lw(front + 20);
    let page = if count > 0 { lw(front + 4) + ((count - 1) << 4) } else { 0 };
    let (x, y) = (px - lw(g::window), py - lw(g::window + 20));
    // the things the game drew, the last drawn on top
    let zone = SHOWN.lock().unwrap().iter().rev().find(|z| px >= z.x0 && px < z.x1 && py >= z.y0 && py < z.y1).copied();
    if let Some(z) = zone {
        match z.kind {
            4 | 5 if page != 0 => {
                // (an arrow belongs to the row of text it is level with)
                let cy = (z.y0 + z.y1) / 2;
                let row = SHOWN.lock().unwrap().iter().rev().find(|r| r.kind == 11 && cy >= r.y0 - 2 && cy < r.y1 + 2).copied();
                if let Some(r) = row {
                    sh(page + 4, r.item);
                }
            }
            11 if page != 0 => {
                sh(page + 4, z.item);
            }
            10 => {
                let menu = lw(g::curmenu);
                if menu != 0 && z.item != 0 {
                    sw(menu + 8, z.item);
                }
            }
            _ => {}
        }
        if std::env::var_os("TS_BOXDBG").is_some() {
            eprintln!("[box] pointer {px},{py} zone {} x {}..{} y {}..{}", z.kind, z.x0, z.x1, z.y0, z.y1);
        }
        OVER.store(if z.kind == 11 { 1 } else { z.kind }, Ordering::Relaxed);
        return;
    }
    let mut hit = 0u8;
    let mut box_ = if page != 0 { lw(page + 8) } else { 0 };
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
        eprintln!("[box] pointer {px},{py} page boxes {} hit {hit}", if page != 0 { lw(page + 12) } else { 0 });
        for z in SHOWN.lock().unwrap().iter() {
            eprintln!("[box]   zone {} x {}..{} y {}..{} item {:#x}", z.kind, z.x0, z.x1, z.y0, z.y1, z.item);
        }
        if page != 0 {
            let mut b = lw(page + 8);
            for _ in 0..lw(page + 12) {
                eprintln!("[box]   x {} y {} w {} h {} id {} flags {:#x}", lhu(b), lhu(b + 2), lhu(b + 4), lhu(b + 6), lhu(b + 8), lhu(b + 10));
                b += 24;
            }
        }
    }
    OVER.store(hit, Ordering::Relaxed);
}
