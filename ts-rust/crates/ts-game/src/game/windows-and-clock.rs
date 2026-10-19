//! Windows and viewports, the map maker's lists, the clock, guns' reloading and more.

use crate::*;

/// @ 0x244a38
pub fn mcseqWindowOpen() {
    mcseqWindowSet();
    sb(0x3affb8, 1);
}

/// Give a prop its table of override records: two for each part, taken from the level's list (@ 0x2743c8).
pub fn allocateOverridePointers(prop: i32) {
    let model = lw(prop + 32);
    let used = lw(g::numgfxpointers);
    let total = used + (lw(lw(model)) << 1);
    sw(model + 248, lw(g::levelgfxpointers) + used * 24);
    sw(g::numgfxpointers, total);
    nullOverridePointers(prop);
}

/// Leave the map maker's dialogs: the page is closed and opened again with flags 3 (@ 0x24fcc0).
pub fn exitDialogs() {
    let front = lw(g::front);
    let flags = lw(g::editflags) & -129;
    sw(front + 12, 0);
    sw(front + 20, 0);
    sw(g::editflags, flags);
    sw(front + 12, 3);
    mmPopCursorPos();
}

/// A field (at `at`) of the level list entry of the current level, or `default` (@ 0x225b10 and 0x225a70).
fn level_field(at: i32, default: i32) -> i32 {
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    let text_of = |entry: i32| lw((lh(entry) << 2) + texts);
    if text_of(LVINFO) == 0 {
        return default;
    }
    let level = lw(g::lvlevelnum);
    let mut i = 0;
    loop {
        let entry = LVINFO + i * 24;
        i += 1;
        if lw(entry + 4) == level {
            return lw(entry + at);
        }
        if text_of(i * 24 + LVINFO) == 0 {
            return default;
        }
    }
}

/// The texture number of the level's glow (@ 0x225b10).
pub fn lvGetGlowTexNum() -> i32 {
    level_field(16, -1)
}

/// The prop number of the level's sky (@ 0x225a70).
pub fn lvGetSkyPropNum() -> i32 {
    level_field(12, -1)
}

/// Place a player's window (record: player number at +0; top +8, bottom +16, left +4, right +12) for the number of players: the whole screen, halves, or quarters (@ 0x27efb0).
pub fn playerSetWindow(window: i32) {
    let players = ilinkGetNumLocalPlayers();
    if players == 2 {
        let lower = lw(window) & 1;
        sw(window + 16, if lower != 0 { 223 } else { 111 });
        sw(window + 8, if lower == 0 { 0 } else { 113 });
        sw(window + 12, 639);
        sw(window + 4, 0);
    } else if players >= 3 {
        if players < 5 {
            let number = lw(window);
            let lower = number & 2;
            let right = number & 1;
            sw(window + 16, if lower != 0 { 223 } else { 111 });
            sw(window + 4, if right == 0 { 0 } else { 322 });
            sw(window + 8, if lower == 0 { 0 } else { 113 });
            sw(window + 12, if right != 0 { 639 } else { 317 });
        }
    } else if players == 1 {
        sw(window + 16, 223);
        sw(window + 12, 639);
        sw(window + 4, 0);
        sw(window + 8, 0);
    }
}

/// Set the main window to the rectangle (x0, y0) - (x1, y1): its edges, size, centre, and the scale against the screen (@ 0x20c420).
pub fn windowSet(x0: i32, y0: i32, x1: i32, y1: i32) {
    let width = x1 - x0 + 1;
    let w = g::window;
    let screen_w = lw(w + 52) as f32;
    sw(g::window, x0);
    sw(w + 8, (x0 + x1 + 1) >> 1);
    sw(w + 16, width >> 1);
    sw(w + 4, x1);
    sw(w + 12, width);
    swc1(w + 80, width as f32 / screen_w);
    let split = (ilinkGetNumLocalPlayers() >= 2) as i32;
    let height = y1 - y0 + 1;
    let scale_y = height as f32 / lw(w + 72) as f32;
    sw(w + 20, y0);
    sw(w + 24, y1);
    sw(w + 88, split);
    sw(w + 28, (y0 + y1 + 1) >> 1);
    sw(w + 36, height >> 1);
    sw(w + 32, height);
    swc1(w + 84, scale_y);
    sw(w + 92, (ilinkGetNumLocalPlayers() >= 2) as i32);
}

/// Ticks left in the challenge: the end time of its first timed condition (flag word at +4 is 0), minus the game frame; 0 without one (@ 0x21f318).
pub fn GetChallengeTimeLeft() -> i32 {
    let setup = challengeGetSetup();
    let count = lw(setup + 20);
    let list = lw(setup + 16);
    for i in 0..count.max(0) {
        let cond = i * 20 + list;
        if lw(cond + 4) == 0 {
            return lw(cond + 16) - lw(g::gameframe);
        }
    }
    0
}

/// Reload a gun from the ammo carried: move what is needed (and there is) from the player's store (at +620) to the gun's clip (at +4 of `gun`); returns the amount (@ 0x28db18).
pub fn gunReloadCore(player: i32, gun: i32) -> i32 {
    let entry = lw(gun) * 400 + g::guns;
    let ammo = lw(entry + 16) << 2;
    let store = player + 620 + ammo;
    let carried = lw(store);
    if carried != 0 {
        let capacity = lw(entry + 20);
        if capacity > 0 {
            let clip = gun + 4 + ammo;
            let loaded = lw(clip);
            let room = capacity - loaded;
            let take = if carried < room { carried } else { room };
            sw(clip, loaded + take);
            sw(store, lw(store) - take);
            return take;
        }
    }
    0
}

/// Keep the map maker's mouse cursor inside the rectangle (top +0, bottom +4, left +8, right +12) (@ 0x246920).
pub fn mmClampMouse(rect: i32) {
    let front = lw(g::front);
    let x = lw(front + 32);
    let (left, right) = (lw(rect + 8), lw(rect + 12));
    if x < left {
        sw(front + 32, left);
    } else if right < x {
        sw(front + 32, right);
    }
    let y = lw(front + 36);
    let top = lw(rect);
    if y < top {
        sw(front + 36, top);
    } else {
        let bottom = lw(rect + 4);
        if bottom < y {
            sw(front + 36, bottom);
        }
    }
}

/// Save the map maker's cursor in the stack (10 slots of 12 bytes) and go one up (@ 0x24fd58).
pub fn mmPushCursorPos() {
    let n = lw(0x3b0060);
    if n < 10 {
        let front = lw(g::front);
        let base = 0x1fc4080;
        let cursor = lw(g::mmcursor);
        sw(n * 12 + base + 4, lw(front + 32));
        sw(n * 12 + base, cursor);
        let y = lw(front + 36);
        sw(0x3b0060, n + 1);
        sw(n * 12 + base + 8, y);
        if n + 1 < 10 {
            let next = lw((n + 1) * 12 + base);
            sw(front + 32, lw((n + 1) * 12 + base + 4));
            sw(g::mmcursor, next);
            sw(front + 36, lw((n + 1) * 12 + base + 8));
        }
    }
}

/// Work out the aim from the direction `aim` (unit vector) to a target at `to`: the distance goes to +8 of `out`, the cosine of the angle to +4 (@ 0x280780).
pub fn playerAutoAimCalc(aim: i32, to: i32, out: i32) {
    let (x, y, z) = (lwc1(to), lwc1(to + 4), lwc1(to + 8));
    let dist = (x * x + y * y + z * z).abs().sqrt();
    swc1(out + 8, dist);
    let inv = 1.0_f32 / dist;
    let (ay, az) = (lwc1(aim + 4), lwc1(aim + 8));
    let dx = lwc1(to) * inv;
    let dy = lwc1(to + 4) * inv;
    let dz = lwc1(to + 8) * inv;
    let ax = lwc1(aim) * dx;
    swc1(out + 4, ax + ay * dy + az * dz);
}

/// How many barrels (prop type 202) are not blown up (value at +532 below 1) (@ 0x275648).
pub fn propNumBallelsNotBlownUp() -> i32 {
    let mut n = 0;
    for i in 0..lw(g::num_props) {
        let prop = i * 592 + lw(g::levelprops);
        if lw(prop + 4) == 202 && lwc1(prop + 532) < 1.0 {
            n += 1;
        }
    }
    n
}

/// Set the three vectors (12 bytes each from +12) of effect group `fx` (records of 92 bytes, 56 of them) (@ 0x2ae570).
pub fn specialfxUpdatePos(fx: i32, a: i32, b: i32, c: i32) {
    if fx != 0 {
        let mut group = g::fxgroups;
        while fx != group {
            group += 92;
            if group >= g::fxgroups + 0x1420 {
                return;
            }
        }
        for (i, v) in [a, b, c].into_iter().enumerate() {
            let at = group + 12 + 12 * i as i32;
            swc1(at, lwc1(v));
            swc1(at + 4, lwc1(v + 4));
            swc1(at + 8, lwc1(v + 8));
        }
    }
}

/// Transform a vector by the three axes and the translation of a matrix into 4 outputs (the fourth from the last column) (@ 0x2b54f8).
pub fn matrixVec3Mul4(m: i32, v: i32, out: i32) {
    for i in 0..4 {
        let (c0, c1, c2, c3) = (lwc1(m + 4 * i), lwc1(m + 16 + 4 * i), lwc1(m + 32 + 4 * i), lwc1(m + 48 + 4 * i));
        swc1(out + 4 * i, lwc1(v) * c0 + lwc1(v + 4) * c1 + lwc1(v + 8) * c2 + c3);
    }
}

/// Copy the items of the list of `src` onto the front of the list of `dst` (list heads at +52) (@ 0x248ab8).
pub fn mmDuplicateItems(src: i32, dst: i32) {
    let mut from = lw(src + 52);
    let mut head = lw(dst + 52);
    while from != 0 {
        let item = mmAllocItem();
        if item == 0 {
            break;
        }
        let kind = lhu(from);
        sw(item + 4, head);
        sh(item, kind);
        head = item;
        let group = lhu(from + 2);
        from = lw(from + 4);
        sh(item + 2, group);
    }
    sw(dst + 52, head);
}

/// The preload record (24 bytes) of kind 3 for texture `tex`, or 0 (@ 0x21c6c8).
pub fn preloadGetTexNum(tex: i32) -> i32 {
    for i in 0..lw(g::numpreload) {
        let rec = i * 24 + 0x1fb4c60;
        if lw(rec) == 3 && lw(rec + 12) == tex {
            return rec;
        }
    }
    0
}

/// Seconds to a memory card clock (BCD bytes: second +1, minute +2, hour +3, day +5, month +6, year +7; months of 31 days, years of 12 months) (@ 0x2b3a88).
pub fn u322clock(clock: i32, seconds: i32) {
    let s = seconds as u32;
    let minutes = s / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let months = days / 31;
    let years = months / 12;
    sb(clock + 1, int2bcd((s - minutes * 60) as i32));
    sb(clock + 2, int2bcd((minutes - hours * 60) as i32));
    sb(clock + 3, int2bcd((hours - days * 24) as i32));
    sb(clock + 5, int2bcd((days as i32) - ((months as i32) << 5) + months as i32));
    sb(clock + 6, int2bcd((months - years * 12) as i32));
    sb(clock + 7, int2bcd(years as i32));
}

/// Move the focus of the current menu to the previous item that can take focus (flag 1 at +20 of its 32-byte item), wrapping to the last (@ 0x20ca20).
pub fn focusprev() {
    let menu = lw(g::curmenu);
    let (mut last, mut previous, mut seen) = (0, 0, 0);
    let mut i = lbu(menu + 4) - 1;
    if i >= 0 {
        let mut item = (i << 5) + 16 + menu;
        loop {
            if lbu(item + 20) & 1 != 0 {
                if last == 0 {
                    last = item;
                }
                if previous == 0 {
                    previous = if seen == 0 { 0 } else { item };
                }
                if item == lw(menu + 8) {
                    seen = 1;
                }
            }
            i -= 1;
            item -= 32;
            if i < 0 {
                break;
            }
        }
    }
    sw(menu + 8, if previous != 0 { previous } else { last });
}

/// Move the front end's mouse with the left stick: each axis speeds up from 4 to 8 while the stick is pushed beyond a threshold, and the position (+32, +36 of the
/// front end) is moved by the stick times the speed (@ 0x22c868).
pub fn frontMouseTick() {
    if lw(lw(g::front) + 12) & 2 == 0 {
        return;
    }
    let stick_x = joyGetAxis(0, 0);
    let stick_y = joyGetAxis(0, 1);
    let mut speed_x = lwc1(0x3aff10);
    let pushed_x = if !(0.0 <= stick_x) { lwc1(0x3ac970) <= -stick_x } else { lwc1(0x3ac96c) <= stick_x };
    if !pushed_x {
        speed_x = 4.0;
        swc1(0x3aff10, speed_x);
    } else if speed_x < 8.0 {
        speed_x += lwc1(0x3ac974);
        swc1(0x3aff10, speed_x);
    }
    let mut speed_y = lwc1(0x3aff14);
    let pushed_y = if !(0.0 <= stick_y) { lwc1(0x3ac97c) <= -stick_y } else { lwc1(0x3ac978) <= stick_y };
    if !pushed_y {
        speed_y = 4.0;
        swc1(0x3aff14, speed_y);
    } else if speed_y < 8.0 {
        speed_y += lwc1(0x3ac980);
        swc1(0x3aff14, speed_y);
    }
    let move_x = stick_x * speed_x;
    let front = lw(g::front);
    let y = lw(front + 36) - (stick_y * speed_y * 32768.0_f32) as i32;
    sw(front + 32, lw(front + 32) + (move_x * 65536.0_f32) as i32);
    sw(front + 36, y);
}
