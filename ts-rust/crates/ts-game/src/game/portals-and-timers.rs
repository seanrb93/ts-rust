//! Portals by room, timers, the prop view label, camping detection, tiles, windows, the pad editor and more.

use crate::*;

/// Close the portal between two rooms (@ 0x257820).
pub fn bgPortalDisableFromRooms(a: i32, b: i32) {
    bgPortalDisable(bgGetPortalFromRooms(a, b));
}

/// Open the portal between two rooms (@ 0x257840).
pub fn bgPortalEnableFromRooms(a: i32, b: i32) {
    bgPortalEnable(bgGetPortalFromRooms(a, b));
}

/// The size a map maker map takes in a file: 204 bytes, 36 for each object of kind 1 and 4 for each item (@ 0x247c38).
pub fn mmCalcFileSize() -> i32 {
    mmObjCount(1) * 36 + 204 + (mmItemCount(-1) << 2)
}

/// Start a timer (24 bytes; 75 at most): `kind` 0 counts down `minutes` * 60 ticks from now in whole ticks, kind 1 counts the time up to a float limit (@ 0x2cdb08).
pub fn timerNew(kind: i32, a: i32, b: i32, minutes: f32) {
    let n = lw(g::numtimers);
    if n < 75 {
        let rec = g::timers + n * 24;
        sw(rec, kind);
        sw(rec + 16, a);
        sw(rec + 8, lw(g::gameframe));
        sw(rec + 20, b);
        if kind == 0 {
            sw(rec + 4, (minutes * 60.0) as i32);
        } else if kind == 1 {
            sw(rec + 4, 0);
            swc1(rec + 12, minutes * 60.0);
        }
        sw(g::numtimers, n + 1);
    }
}

/// The label of the prop editor's view in the debug menu: a short text built in a buffer of the game's, by camera mode, or the off text (@ 0x20d628).
pub fn getpropview() -> i32 {
    let mode = camGetMode();
    let buffer = 0x1fb3220;
    if lw(lw(g::currentplayer) + 904) == 0 {
        return lw(g::offtext);
    }
    if mode >= 4 {
        if mode < 6 {
            sd(buffer, ld(0x39a108));
            sb(buffer + 8, lbu(0x39a110));
            return buffer;
        }
        if mode == 9 {
            let a = lw(0x3ae4b8);
            let b = lhu(0x3ae4b8 + 4);
            sb(buffer + 6, lbu(0x3ae4b8 + 6));
            sw(buffer, a);
            sh(buffer + 4, b);
            return buffer;
        }
    }
    sw(buffer, lw(0x3ae4c0));
    buffer
}

/// Detect players that stay in one place: each player has a position (+868) and a timer (+880); the timer is cleared each time, and when
/// the player is more than 6 units from the position, the position becomes the pad the player's character is on (@ 0x2cd7f8).
pub fn playerCampingDetection() {
    for i in 0..lw(g::lvnumplayers) {
        let player = i * 1820 + lw(g::players);
        let dx = lwc1(player + 140) - lwc1(player + 868);
        let dy = lwc1(player + 144) - lwc1(player + 872);
        let dz = lwc1(player + 148) - lwc1(player + 876);
        sw(player + 880, 0);
        if 36.0_f32 < dx * dx + dy * dy + dz * dz {
            let chr_info = lw(g::chrinfo);
            let pads = lw(g::levelpads);
            sw(player + 880, 0);
            let pad = lw(i * 0x1210 + chr_info + 680) * 28 + pads;
            swc1(player + 868, lwc1(pad + 12));
            swc1(player + 872, lwc1(pad + 16));
            swc1(player + 876, lwc1(pad + 20));
        }
    }
}

/// Add the tile's portal offset (+36) to the portal numbers in the portal lists of the tile's rooms (44 bytes each, the list at +48) (@ 0x254d08).
pub fn bgTileSetPortalNums(tile: i32) {
    let mut room = 0;
    while room < lw(tile + 16) {
        let list = lw(room * 44 + lw(tile) + 48);
        for i in 0..lw(list).max(0) {
            let at = (i << 2) + list;
            sw(at + 4, lw(at + 4) + lw(tile + 36));
        }
        room += 1;
    }
}

/// Whether a character is playing animation `n` in either of its two animation slots (@ 0x215820).
pub fn curAnim(chr: i32, n: i32) -> i32 {
    let data = lw(chr + 352);
    let mut n = n;
    let mut range = n - 443;
    if data != 0 && lw(data + 8) == 0 && (((n - 3) as u32) < 11 || ((n - 113) as u32) < 11) {
        n += lw(chr + 344) * 11;
        range = n - 443;
    }
    let state = lw(chr + 32);
    if (range as u32) < 32 && lw(lw(state + 88) + 12) == 1 {
        n += 38;
    }
    (lw(state + 96) == n || lw(state + 160) == n) as i32
}

/// Put the main window back to its full-screen values (kept at +40..+76) (@ 0x20c378).
pub fn windowRestart() {
    let w = g::window;
    let screen_w = lw(w + 52);
    let screen_h = lw(w + 72);
    let (fw, fh) = (screen_w as f32, screen_h as f32);
    let (x1, cx, cy) = (lw(w + 44), lw(w + 48), lw(w + 56));
    let (y0, y1, mid_y, half_h) = (lw(w + 60), lw(w + 64), lw(w + 68), lw(w + 76));
    sw(g::window, lw(w + 40));
    sw(w + 4, x1);
    sw(w + 8, cx);
    sw(w + 16, cy);
    swc1(w + 80, (fw).pdiv(fw));
    sw(w + 20, y0);
    sw(w + 24, y1);
    sw(w + 28, mid_y);
    sw(w + 36, half_h);
    swc1(w + 84, (fh).pdiv(fh));
    sw(w + 92, 0);
    sw(w + 12, screen_w);
    sw(w + 88, 0);
    sw(w + 32, screen_h);
}

/// How many rooms' boxes (from room 1; boxes at +20 to +40) a box (min x, y, z, then max x, y, z) overlaps (@ 0x254048).
pub fn bgRoomBBIntersectionNumRooms(bb: i32) -> i32 {
    let mut n = 0;
    for i in 0..lw(g::bgNumRooms) {
        let room = lw(g::levelrooms) + (i * 44 + 44);
        if lwc1(bb + 12) < lwc1(room + 20) {
            continue;
        }
        if lwc1(room + 32) < lwc1(bb) {
            continue;
        }
        if lwc1(bb + 16) < lwc1(room + 24) {
            continue;
        }
        if lwc1(room + 36) < lwc1(bb + 4) {
            continue;
        }
        if lwc1(bb + 20) < lwc1(room + 28) {
            continue;
        }
        if !(lwc1(room + 40) < lwc1(bb + 8)) {
            n += 1;
        }
    }
    n
}

/// Delete a pad: every link that uses it is deleted too, and the pad is marked unused (@ 0x264798).
pub fn deletepad(pad: i32) {
    let number = (pad - lw(g::levelpads)).wrapping_mul(-0x49249249) >> 2;
    let mut i = 0;
    let mut count = lw(g::numlinks);
    while i < count {
        let link = lw(g::levellinks) + 20 * i;
        i += 1;
        if lw(link) & 1 != 0 && (lw(link + 4) == number || lw(link + 8) == number) {
            deletelink(link);
            count = lw(g::numlinks);
        }
    }
    let pads = lw(0x3b00a0);
    sw(pad + 4, lw(pad + 4) & -2);
    sw(0x3b00a0, pads - 1);
}

/// Looked through the rooms the players have drawn for room `room` and did nothing with the answer (@ 0x2cd970).
pub fn enemyRoomDrawn(_room: i32) {}

/// Find the box of a front end page that the pointer is in and make it the highlighted one (halfword at +4; 0xffff for none) (@ 0x22c9e8).
pub fn frontFocusTick(page: i32) {
    let front = lw(g::front);
    let x = lh(front + 34);
    let y = lh(front + 38);
    let mut box_ = lw(page + 8);
    for _ in 0..lw(page + 12) {
        if lhu(box_ + 10) & 1 == 0
            && ((x - lhu(box_)) as u32) < lhu(box_ + 4) as u32
            && ((y - lhu(box_ + 2)) as u32) < lhu(box_ + 6) as u32
        {
            sh(page + 4, lhu(box_ + 8));
            return;
        }
        box_ += 24;
    }
    sh(page + 4, 0xffff);
}

/// Pan the map maker's view back when its centre is further than 0x2800000 from the origin (@ 0x246530).
pub fn mmClampView() {
    const LIMIT: i32 = 0x2800000;
    let w = lw(g::m_w);
    let h = lw(g::m_h);
    let cx = lw(g::m_x) + w / 2;
    let cy = lw(g::m_y) + h / 2;
    if cx < -LIMIT {
        mmPanWindow(-LIMIT - cx, 0);
    } else if LIMIT < cx {
        mmPanWindow(LIMIT - cx, 0);
    }
    if cy < -LIMIT {
        mmPanWindow(0, -LIMIT - cy);
    } else if LIMIT < cy {
        mmPanWindow(0, LIMIT - cy);
    }
}

/// The text of the story mission briefing of the current level (level 2 to 27: nine of them have one) in the current language, or 0 (@ 0x2114d8).
pub fn StoryGetBriefing() -> i32 {
    let v = lw(g::lvlevelnum) - 2;
    let offset = if (v as u32) < 26 {
        match v {
            0 => 0x1194,
            3 => 0x1198,
            6 => 0x11a0,
            7 => 0x119c,
            12 => 0x1190,
            19 => 0x118c,
            21 => 0x1188,
            22 => 0x11a4,
            25 => 0x11a8,
            _ => return 0,
        }
    } else {
        return 0;
    };
    lw(lw((lw(g::cur_language) << 2) + g::languages) + offset)
}
