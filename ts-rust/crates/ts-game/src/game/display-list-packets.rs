//! The display list's register packets, skeleton transforms, the map maker's pools, challenges and more.

use crate::*;

/// @ 0x20c410
pub fn windowReset() {
    windowRestart();
}

/// A stick axis of a player's controller as -1 to 1 (0 left x, 1 left y, 2 right x, 3 right y, y axes up positive) (@ 0x203ee0).
pub fn joyGetAxisLOCAL(player: i32, axis: i32) -> f32 {
    let c = lw((player << 2) + g::playercontrollers) * 384 + g::controllers;
    match axis {
        0 => joyFixVal(lw(c + 300), lwc1(c + 316)),
        1 => -joyFixVal(lw(c + 308), lwc1(c + 316)),
        2 => joyFixVal(lw(c + 304), lwc1(c + 316)),
        3 => -joyFixVal(lw(c + 312), lwc1(c + 316)),
        _ => 0.0,
    }
}

/// The one-player setup of story level `level`, or 0 (@ 0x227af0).
pub fn lvGetSTORYSetup(level: i32) -> i32 {
    let v = level - 2;
    if (v as u32) < 104 {
        match v {
            0 => g::setup_village_1up,
            3 => g::setup_industrial_1up,
            6 => g::setup_mansion_1up,
            7 => g::setup_planet_1up,
            12 => g::setup_scifi_1up,
            19 => g::setup_chinese_1up,
            21 => g::setup_tomb_1up,
            22 => g::setup_docks_1up,
            25 => g::setup_launch_1up,
            _ => 0,
        }
    } else {
        0
    }
}

/// Select dither matrix `mode` (0 to 2) in the display list (-1 keeps the current one); returns the mode that was set before (@ 0x2b7df0).
pub fn dlSetDitherMatrix(mode: i32) -> i32 {
    let previous = lw(g::dlDitherMode);
    if mode != previous && mode != -1 {
        if (0..=2).contains(&mode) {
            sb(lw(g::gfxtag) + 3, 48);
            let tag = lw(g::gfxtag);
            sw(tag + 4, g::dmvifpackets + 48 * mode);
            sh(tag, 3);
            sw(g::gfxtag, tag + 16);
        }
        sw(g::dlDitherMode, mode);
    }
    previous
}

/// Make the map maker's free lists: 1024 objects of 60 bytes linked through +56, and 258 items of 8 bytes linked through +4 (@ 0x2485b8).
pub fn mmClearObjects() {
    let objects = lw(g::objects);
    let items = lw(g::itemspool);
    for i in 0..1023 {
        sw(objects + 60 * i + 56, objects + 60 * (i + 1));
    }
    sw(g::numfreeobjs, 1024);
    sw(objects + 0xeffc, 0);
    sw(0x3b0014, objects);
    sw(g::usedobj, 0);
    for i in 0..257 {
        sw(items + 8 * i + 4, items + 8 * (i + 1));
    }
    sw(0x3b0018, items);
    sw(0x3b001c, 258);
    sw(items + 2060, 0);
}

/// Choose the icon of a pad in the map maker from the pad's item (@ 0x250438).
pub fn mm_setpadicon(pad: i32) {
    let item = objFindItem(lw(g::zoomtileobj), lw(lw(pad)), -1);
    let mut icon = 1;
    if item != 0 {
        let group = lhu(item + 2);
        if (group as u32) < 20 {
            icon = match group {
                0..=4 => 4,
                5..=8 => group + 3,
                9..=13 => 2,
                14 | 15 => 3,
                16 | 17 => 5,
                18 => 6,
                _ => 7,
            };
        }
    }
    sw(lw(lw(pad + 4) + 32), lw((icon << 2) + g::iconobs));
}

/// Whether challenge `n` can be played: the first of each group of three needs all but two of the earlier groups' last challenges done, the others the one before them (@ 0x225038).
pub fn challengeAvail(n: i32) -> i32 {
    let rem = n.wrapping_rem(3);
    if rem == 0 {
        let groups = n.wrapping_div(3);
        if groups >= 3 {
            let mut done = 0;
            for i in 0..groups {
                if lw(statsGet() + ((i * 3 + 2) << 3) + 1404) != 0 {
                    done += 1;
                }
            }
            return !(done < groups - 2) as i32;
        }
        return 1;
    }
    let v = rem + 65;
    if v >= 65 && v < 68 {
        (lw(statsGet() + ((n - 1) << 3) + 1404) != 0) as i32
    } else {
        0
    }
}

/// Write a 48-byte packet of the display list that sets one GS register through the GIF (`value` is the data word at +20, `value2` at +24,
/// `register` the register number at +28) (the code the `dl...` functions share).
fn register_packet(value: i32, value2: i32, register: i32) {
    let packet = memdbAlloc(48);
    sb(lw(g::gfxtag) + 3, 48);
    sw(packet, 0x6c028000);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    sw(packet + 4, 0x8001);
    sw(packet + 8, 0x10000000);
    sw(packet + 12, 14);
    sw(packet + 16, 0);
    sw(packet + 20, value);
    sw(packet + 24, value2);
    sw(packet + 28, register);
    sw(packet + 32, 0);
    sw(packet + 36, 0x140007fc);
    sw(packet + 40, 0x11000000);
    sw(packet + 44, 0);
    sh(tag, 3);
    sw(g::gfxtag, tag + 16);
}

/// End the display list's drawing (register 96) (@ 0x2b7a10).
pub fn dlFinish() {
    register_packet(0, 0, 96);
}

/// Put a label into the display list (register 98) that the depth tests use to find their place (@ 0x2b7d28).
pub fn dlLabel(label: i32) {
    register_packet(label, 0, 98);
}

/// Set the clip rectangle (x0, y0, x1, y1), kept inside the 640 x 224 field (register 64) (@ 0x2b8050).
pub fn dlSetClip(x0: i32, y0: i32, x1: i32, y1: i32) {
    let x0 = if -1 >= x0 { 0 } else { x0 };
    let x1 = if x1 >= 640 { 639 } else { x1 };
    let y0 = if -1 >= y0 { 0 } else { y0 };
    let y1 = if y1 >= 224 { 223 } else { y1 };
    register_packet(x0 | x1 << 16, y0 | y1 << 16, 64);
}

/// Counted the texture buffer's entries and printed nothing (@ 0x2b6ba0).
pub fn emPrintStats() {}

/// The rectangle that holds all boxes (24 bytes: x, y, width, height as halfwords) of a front end page: min x, min y, max x, max y into `out` (@ 0x22c3b0).
pub fn frontGetPageSize(page: i32, out: i32) {
    sw(out + 4, 2000);
    sw(out + 12, -1);
    sw(out, 2000);
    sw(out + 8, -1);
    let mut i = 0;
    let boxes = if lw(page + 12) > 0 { lw(page + 8) } else { 0 };
    // the count is read again after each box (the output may overlap the page)
    while lw(page + 12) > 0 && i < lw(page + 12) {
        let b = boxes + 24 * i;
        i += 1;
        let x = lhu(b);
        let y = lhu(b + 2);
        let x2 = x + lhu(b + 4);
        let y2 = y + lhu(b + 6);
        let (min_x, max_x, min_y, max_y) = (lw(out), lw(out + 8), lw(out + 4), lw(out + 12));
        sw(out, if min_x < x { min_x } else { x });
        sw(out + 8, if x2 < max_x { max_x } else { x2 });
        sw(out + 4, if min_y < y { min_y } else { y });
        sw(out + 12, if y2 < max_y { max_y } else { y2 });
    }
}
