//! The credits roll, and the lights of the character select screen.

use crate::*;

/// Move the credits on: the page time (`creditspagetime`) grows by a frame in seconds; when it is past the end of the page (the page's two times plus 8) the next
/// page starts (back to 0 after the last, marked by a kind of -1).  The size of the page, the waiting and text times and the start position are set (records of
/// 28 bytes at `creditsText`: size at +0, kind at +12 (0 text, 4 end of a page), times at +20 and +24) (@ 0x2aff40).
pub fn creditsTextTick() {
    let start = lw(g::creditsTextIndex);
    let text = g::creditsText;
    let mut page_end = start;
    let frame = lwc1(g::updateframesf);
    let mut page_time = lwc1(g::creditspagetime);
    if lw(text + start * 28 + 12) != 4 {
        loop {
            page_end += 1;
            if lw(page_end * 28 + text + 12) == 4 {
                break;
            }
        }
    }
    let record = text + page_end * 28;
    page_time += (frame).pdiv(60.0_f32);
    let finished = (lwc1(record + 20) + lwc1(record + 24) + 4.0 + 4.0) < page_time;
    swc1(g::creditspagetime, page_time);
    let mut index = start;
    if finished {
        index = page_end + 1;
        sw(g::creditspagetime, 0);
        sw(g::creditsTextIndex, index);
        if lw(index * 28 + text + 12) == -1 {
            sw(g::creditsTextIndex, 0);
            index = 0;
        }
    }
    let mut end = index;
    sw(g::creditsPageSize, 0);
    let mut height = 0;
    if lw(text + end * 28 + 12) != 4 {
        let mut at = end * 28 + text;
        let mut page_size = 0;
        let mut kind = lw(at + 12);
        loop {
            if kind == 0 {
                height += lw(at);
                page_size = height;
            }
            at += 28;
            kind = lw(at + 12);
            end += 1;
            if kind == 4 {
                break;
            }
        }
        sw(g::creditsPageSize, page_size);
    }
    let window_height = lw(g::window + 32);
    let record = end * 28 + g::creditsText;
    let text_time = lwc1(record + 24);
    swc1(g::creditswaittime, lwc1(record + 20));
    swc1(g::creditstexttime, text_time);
    swc1(g::creditsStartPos, ((window_height / 2) - (height / 2)) as f32);
}

/// Reset the lights of the four characters of the select screen: three floats each of the light (1.5), theta (270) and phi (180), the other words of the
/// four-entry tables, and no ambient light (@ 0x2380f8).
pub fn fxSelectCharReset() {
    for i in 0..4 {
        let at = 12 * i;
        let word = 4 * i;
        swc1(g::lighty + at, 1.5);
        swc1(g::lighty + at + 4, 1.5);
        swc1(g::theta + at, 270.0);
        swc1(g::theta + at + 4, 270.0);
        swc1(g::phi + at, 180.0);
        swc1(g::phi + at + 4, 180.0);
        swc1(g::lighty + at + 8, 1.5);
        sw(0x1fc1248 + word, 0);
        sw(0x1fc1268 + word, 0);
        sw(0x1fc1258 + word, 0);
        sw(0x1fc1288 + word, -1);
        sw(0x1fc1298 + word, -1);
        sw(0x1fc12c8 + word, 0);
        sw(0x1fc12a8 + word, 0);
        swc1(0x1fc1278 + word, 3.0);
        sw(0x1fc12b8 + word, 0);
        swc1(g::theta + at + 8, 270.0);
        swc1(g::phi + at + 8, 180.0);
        sw(0x1fc12f8 + word, -1);
    }
    sw(g::frontambient, 0);
}
