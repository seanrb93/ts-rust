//! Messages, route tables, the depth test lists, guns' ammo and mines, and more short functions.

use crate::*;

/// @ 0x210fc0
pub fn challenge_pageTick_210fc0() {
    tickRetryQuit();
}

/// Set how many particles a group makes: `rate` for the number of players (@ 0x296e50).
pub fn particleUpdateRate(group: i32, rate: f32) {
    if group != 0 {
        sw(group + 44, particleGetNum(rate as i32));
    }
}

/// Stop acting sound `n` and free its slot (@ 0x238490).
pub fn actingsoundStop(n: i32) {
    let slot = (n << 2) + 0x1fc12f8;
    let sound = lw(slot);
    if sound >= 0 {
        soundDelayStop(sound);
        sw(slot, -1);
    }
}

/// Show an on-screen message (`message`, with `ticker` its timer) unless one is shown; the buttons are cleared so the press that opened it does not close it (@ 0x20d418).
pub fn menuStartMessage(message: i32, ticker: i32) {
    if lw(g::in_message) != 0 {
        return;
    }
    let player = lw(g::currentplayer);
    sw(g::in_message, message);
    let number = lw(player);
    sw(g::in_message_ticker, ticker);
    joyClearButs(number, 0xffff);
}

/// The default wait of a message: any of the face buttons ends it (@ 0x20d3d0).
pub fn menuDefMsgWaitFunc() {
    if joyNewBut(lw(lw(g::currentplayer))) & 2288 == 0 {
        return;
    }
    joyClearButs(lw(lw(g::currentplayer)), 0xffff);
    menuEndMessage();
}

/// The scores page of modes 4 and 6 lets the first player retry or quit (@ 0x212078).
pub fn scores_pageTick() {
    let mode = lw(g::curgameparms + 72);
    if (mode == 4 || mode == 6) && lw(lw(g::currentplayer)) == 0 {
        tickRetryQuit();
    }
}

/// Make an item of kind `kind` and group `group` at the head of the list of object `owner` (list head at +52); 0 if there is none free (@ 0x248b30).
pub fn mmNewItem(owner: i32, kind: i32, group: i32) -> i32 {
    let item = mmAllocItem();
    if item == 0 {
        return 0;
    }
    let head = lw(owner + 52);
    sw(owner + 52, item);
    sw(item + 4, head);
    sh(item, kind);
    sh(item + 2, group);
    item
}

/// Leaving the debug menu: the controller goes back to mode 2 from the move view (camera mode 3), else to mode 0 (@ 0x20dbf0).
pub fn debugmenuEnd() {
    debugmenuSetJoyMode(if camGetMode() == 3 { 2 } else { 0 });
}

/// Whether a player's gun is punching: its model (at +612 of the player) has flag 0x8000 and the player has the fist (gun 0) (@ 0x2951d8).
pub fn gunIsPunching(gun: i32) -> i32 {
    if gun != 0 {
        let number = lw(gun);
        if number < lw(g::lvnumplayers) {
            let player = lw(g::players) + number * 1820;
            if lw(gun + 260) == 0 {
                let model = lw(player + 612);
                if model != 0 && lw(player + 756) != 0 {
                    return (lhu(lw(model + 32) + 292) & 0x8000 != 0) as i32;
                }
            }
        }
    }
    0
}

/// Set up a route table: 1000 entries of 16 bytes from `base`, each starting with its own number (the last one -1) (@ 0x265eb0).
fn init_routes(base: i32) {
    for i in 1..=999 {
        sh(base + 16 * i, i);
    }
}

/// @ 0x265eb0
pub fn hr_initroutes() {
    init_routes(0x1fc6480);
    sw(0x3b00b8, -1);
    sh(0x1fca300, -1);
    sw(0x3b00b0, -1);
    sw(0x3b00c0, 0);
}

/// @ 0x266570
pub fn r_initroutes() {
    init_routes(0x1fca300);
    sw(0x3b00d8, -1);
    sh(0x1fce180, -1);
    sw(0x3b00d0, -1);
    sw(0x3b00e0, 0);
}

/// Count a HUD message of player `n` on: its timer runs to 180 ticks, then clears (@ 0x2b09e8).
pub fn hudMessageTick(n: i32) {
    let hud = n * 352 + g::hudparms;
    let t = lw(hud + 348);
    if t != 0 {
        let frames = lw(g::updateframesi);
        if t < 180 {
            let now = t + frames;
            sw(hud + 348, now);
            if now < 180 {
                return;
            }
        }
        sw(hud + 348, 0);
    }
}

/// Write the text "mem_mark" over memory from `start` up to `end`, to be checked later (@ 0x201a60).
pub fn memMark(start: i32, end: i32) {
    let mut at = start;
    if (at as u32) < (end as u32) {
        loop {
            for (i, c) in b"mem_mark".iter().enumerate() {
                sb(at + i as i32, *c as i32);
            }
            at += 8;
            if (at as u32) >= (end as u32) {
                break;
            }
        }
    }
}

/// Looked for "mem_mark" and did nothing about it (@ 0x201ac0).
pub fn memCheck(_at: i32) {}

/// Walked the props and printed nothing (@ 0x266e98).
pub fn propDump(_props: i32, _count: i32) {}

/// How many files of a table of 16 sign-on files are used, counting from the start until the first unused one (@ 0x220708).
pub fn signonFileCount(table: i32) -> i32 {
    let mut at = table;
    let mut n = 0;
    if lbu(at) != 0 {
        n += 1;
        loop {
            at += 2936;
            if n >= 16 || lbu(at) == 0 {
                break;
            }
            n += 1;
        }
    }
    n
}

/// Add a point to the depth test list (up to 72 tests of 9 points of 24 bytes) (@ 0x2a6e60).
pub fn zbtestAddPoint(a: i32, b: i32, c: i32, x: i32, y: i32, size: i32, d: i32) {
    let list = lw(0x3b0170) << 2;
    let tests = lw(list + 0x1fea0a0);
    if tests < 72 {
        let point = lw(g::CurrentAddIndex);
        if point < 9 {
            let rec = (tests * 9 + point) * 24 + lw(list + g::zbval);
            sw(rec + 20, d);
            sw(rec, a);
            sw(rec + 12, (y * 640 + x) << 1);
            sw(rec + 4, b);
            sw(rec + 8, c);
            sh(rec + 16, size);
            sw(g::CurrentAddIndex, point + 1);
        }
    }
}

/// The description (20 bytes) of last stand level `n`, or 0 (@ 0x21df58).
pub fn getLastStandInfo(n: i32) -> i32 {
    for i in 0..5 {
        let at = g::lsdesc + 20 * i;
        if lw(at) == n {
            return at;
        }
    }
    0
}

/// Copy the light palette (64 bytes) into the map maker dialogs' colours (@ 0x253498).
pub fn mmDialogsMake() {
    for i in 0..8 {
        sd(0x1fc4040 + 8 * i, ld(g::lightpalette + 8 * i));
    }
}

/// Release timer `n` by moving the last timer (24 bytes) into its place (@ 0x2cdd58).
pub fn timerRelease(n: i32) {
    let count = lw(g::numtimers);
    let last = count - 1;
    if n != last {
        let from = count * 24 + (g::timers - 24);
        let to = n * 24 + g::timers;
        let (a, b, c) = (ld(from + 8), ld(from + 16), ld(from));
        sd(to, c);
        sd(to + 8, a);
        sd(to + 16, b);
    }
    sw(g::numtimers, last);
}

/// Give a head (112 bytes, 15 of them) back: its used flag is cleared (@ 0x2686c0).
pub fn freehead(head: i32) {
    for i in 0..15 {
        if lw(g::levelheads) + 112 * i == head {
            sw(g::headused + 4 * i, 0);
            return;
        }
    }
}

/// The last particle of list `n` (particles are linked through +68) (@ 0x296bf0).
pub fn getlastinusedlist(n: i32) -> i32 {
    let mut p = lw((n << 2) + 0x1fe99c0);
    if p != 0 {
        let mut next = lw(p + 68);
        while next != 0 {
            p = next;
            next = lw(p + 68);
        }
    }
    p
}

/// Whether a character has any mine that could be detonated: one of its 20 mine words (from +0x11c0) is set (@ 0x295088).
pub fn gunCanDetonateMines(chr: i32) -> i32 {
    (0..20).any(|i| lw(chr + 0x11c0 + 4 * i) != 0) as i32
}

/// Fill the ammo of both guns of combination `combo` to the most the game allows (@ 0x28d3a8).
pub fn gunSetAmmoToMax(player: i32, combo: i32) {
    let entry = GUNCOMBOS + (combo << 3);
    for hand in [2, 4] {
        let gun = lhu(entry + hand);
        if (gun as u32) < 30 {
            let ammo_type = lw((gun as i16 as i32) * 400 + (g::guns + 16));
            sw(player + (ammo_type << 2) + 620, lw((ammo_type << 3) + g::ammoinfo));
        }
    }
}

/// How many items of a list have kind `kind` (all of them if it is negative) (@ 0x248c98).
pub fn mmItemListCount(list: i32, kind: i32) -> i32 {
    let mut n = 0;
    let mut item = list;
    while item != 0 {
        if kind >= 0 {
            if lhu(item + 2) == kind {
                n += 1;
            }
        } else {
            n += 1;
        }
        item = lw(item + 4);
    }
    n
}

/// The instance hit last and, through the pointers that are not 0, its part, chunk and polygon (@ 0x263730).
pub fn obGetLastHitInst(part: i32, chunk: i32, poly: i32) -> i32 {
    let inst = lw(g::s_pLastHitInst);
    if inst != 0 {
        let p = lw(g::s_LastHitPart_3aeaa0);
        if part != 0 {
            sw(part, p);
        }
        let c = lw(g::s_LastHitChunk);
        if chunk != 0 {
            sw(chunk, c);
        }
        let q = lw(g::s_LastHitPoly);
        if poly != 0 {
            sw(poly, q);
        }
    }
    inst
}

/// Clear bit 0x10000 of every pad's flags (@ 0x264200).
pub fn padRestart() {
    for i in 0..lw(g::numpads) {
        let pad = i * 28 + lw(g::levelpads);
        sw(pad + 4, lw(pad + 4) & !0x10000);
    }
}
