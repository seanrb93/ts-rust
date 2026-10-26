//! The blast spawn effect, the making of the menu items of a page, and the profile timers.

use crate::*;

/// The tick of the blast spawn effect for the effects group `group`: the prop in the first slot (+56) swings by a sine of the group's time (still 1.0 in the second value while the
/// time is below 0.75) and is deleted after 1.5, and the group is freed after 1.5 (@ 0x2accd8).
pub fn specialfxBlastSpawnTick(group: i32) {
    let groups = g::fxgroups;
    let prop = lw(groups + group * 92 + 56);
    if prop != 0 {
        let limit = 1.5_f32;
        let time_at = group * 92 + (groups + 48);
        let table = g::g_SinTable;
        let fraction = (lwc1(time_at)).pdiv(limit);
        let scale = ld(0x3a9768);
        let scale_two = ld(0x3a9770);
        let half = 0x3fe0000000000000_i64;
        let state = lw(prop + 352);
        let first = dptoli(dpadd(dpmul(dpmul(fptodp(fraction), scale), scale_two), half));
        swc1(state + 44, lwc1(((first & 2047) << 2) + table));
        let time = lwc1(time_at);
        if !(time < 0.75_f32) {
            let fraction = (time).pdiv(limit);
            let second = dptoli(dpadd(dpmul(dpmul(fptodp(fraction), scale), scale_two), half));
            swc1(state + 48, lwc1(((second & 2047) << 2) + table));
        } else {
            swc1(state + 48, 1.0_f32);
        }
        let groups = g::fxgroups;
        if 1.5_f32 < lwc1(groups + group * 92 + 48) {
            let slot = group * 92 + (groups + 56);
            specialPropDelete(lw(slot));
            sw(slot, 0);
        }
    }
    if !(1.5_f32 < lwc1(group * 92 + (g::fxgroups + 48))) {
        return;
    }
    specialfxFree(group);
}

/// Make the items of the page `page` into `menus` (records of 24 bytes): an item with flag 2 is a list (`menulistmake` of the list at +12), the others are made by `menuitemmake` at the
/// page's position plus the item's, the y of an item with flag 32 following the line of the font; the number made is returned (@ 0x24c270).
pub fn menumake(menus: i32, page: i32) -> i32 {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut at = menus;
    let start = at;
    let mut i = 0;
    let mut item = page + 16;
    if lw(page) > 0 {
        loop {
            let flags = lhu(item + 10);
            if (flags & 2) == 0 {
                sh(sp, lhu(page + 4) + lhu(item + 4));
                if (flags & 32) == 0 {
                    sh(sp + 2, lhu(page + 6) + lhu(item + 6));
                } else {
                    let offset = lh(item + 6);
                    let y = lh(sp + 2);
                    let line = if (lbu(g::fontflags) & 1) == 0 {
                        lw(lw(g::curfont) + 12)
                    } else {
                        lw(lw(g::curfont) + 12) >> 1
                    };
                    sh(sp + 2, (y - 2) + (offset + line));
                }
                let made = at;
                at += 24;
                menuitemmake(made, sp, item);
            } else {
                let count = menulistmake(at, lw(item + 12));
                at = count * 24 + at;
            }
            i += 1;
            item += 32;
            if i >= lw(page) {
                break;
            }
        }
    }
    ((at - start).wrapping_mul(-0x55555555)) >> 3
}

/// Make the items of the page `page` into `menus` like `menumake` but with the standard menu's items (`std_menuitemmake`, `std_menulistmake`): the normal font is selected first, a
/// big item (flag 256) is made in the big font, the position is shifted by the window's split flags, and an item with flag 8 moves the y down by a line afterwards; the font that was
/// selected is put back and the number made is returned (@ 0x22daf8).
pub fn std_menumake(menus: i32, page: i32) -> i32 {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut at = menus;
    let start = at;
    let mut i = 0;
    let mut item = page + 16;
    let old_font = textSelect(lw(g::menuitem_normalfont));
    if lw(page) > 0 {
        loop {
            let mut flags = lhu(item + 10);
            let mut count = 0;
            let mut counted = false;
            if (flags & 2) == 0 {
                if (flags & 256) != 0 {
                    textSelect(lw(g::menuitem_bigfont));
                    flags = lhu(item + 10);
                }
                let window = g::window;
                sh(sp, (lh(page + 4) + lh(item + 4)) >> (lw(window + 88) & 31));
                let y = if (flags & 32) != 0 {
                    let offset = (lh(item + 6) - 2) >> (lw(window + 92) & 31);
                    let y = lh(sp + 2);
                    let line = if (lbu(g::fontflags) & 1) == 0 {
                        lw(lw(g::curfont) + 12)
                    } else {
                        lw(lw(g::curfont) + 12) >> 1
                    };
                    y + (offset + line)
                } else {
                    (lh(page + 6) + lh(item + 6)) >> (lw(window + 92) & 31)
                };
                sh(sp + 2, y);
                let made = at;
                at += 24;
                std_menuitemmake(made, sp, item);
                let flags = lhu(item + 10);
                let y = lhu(sp + 2);
                if (flags & 8) != 0 {
                    let y = y as i32;
                    let y = (y << 16) >> 16;
                    let moved = if (lbu(g::fontflags) & 1) == 0 {
                        y + lw(lw(g::curfont) + 12)
                    } else {
                        y + (lw(lw(g::curfont) + 12) >> 1)
                    };
                    sh(sp + 2, moved);
                }
                if (flags & 256) == 0 {
                    count = lw(page);
                    counted = true;
                } else {
                    textSelect(lw(g::menuitem_normalfont));
                }
            } else {
                let made = std_menulistmake(at, lw(item + 12));
                at = made * 24 + at;
            }
            if !counted {
                count = lw(page);
            }
            i += 1;
            item += 32;
            if i >= count {
                break;
            }
        }
    }
    textSelect(old_font);
    ((at - start).wrapping_mul(-0x55555555)) >> 3
}

/// Start the timer `timer` (records of 28 bytes at `profdata`): the timing starts with the first one, a timer that was never given a name is started with the name at 0x3a9a90, its
/// running flag (+4) is set and the time now is taken off its total (+8) (@ 0x2b6320).
pub fn profileTimeStart(timer: i32) {
    let mut running = lw(g::profilenumtiming);
    if running == 0 {
        profileStartTiming();
        running = lw(g::profilenumtiming);
    }
    let data = g::profdata;
    sw(g::profilenumtiming, running + 1);
    let mut record = data + timer * 28;
    if lw(timer * 28 + data) == 0 {
        profileStart(timer, 0x3a9a90, -0xffff81);
        record = data + timer * 28;
    }
    sw(record + 4, 1);
    let now = profileTimeGet();
    let total = timer * 28 + (data + 8);
    swc1(total, lwc1(total) - now);
}

/// Stop the timer `timer`: the timing stops with the last one, the running flag is cleared and the time now is added to the total, which is brought up by 128 when it is below 0
/// (@ 0x2b63c8).
pub fn profileTimeEnd(timer: i32) {
    let data = g::profdata;
    let running = lw(g::profilenumtiming) - 1;
    sw(g::profilenumtiming, running);
    if running == 0 {
        profileStopTiming();
    }
    sw(timer * 28 + (data + 4), 0);
    let now = profileTimeGet();
    let total = timer * 28 + (data + 8);
    let sum = lwc1(total) + now;
    swc1(total, sum);
    if sum < 0.0_f32 {
        swc1(total, sum + 128.0_f32);
    }
}
