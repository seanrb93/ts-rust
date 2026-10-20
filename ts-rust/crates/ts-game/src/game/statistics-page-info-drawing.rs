//! The information drawn on the statistics page.

use crate::*;

/// The text of the language entry at `offset` of the current language.
fn language_text(offset: i32) -> i32 {
    lw(lw((lw(g::cur_language) << 2) + g::languages) + offset)
}

/// Print the label at `offset` of the current language one line down (10) at the left of the page; the position is kept at sp+256 (x) and sp+260 (y), the left at sp+264.
fn print_label(sp: i32, offset: i32, colour: i32) {
    let language = lw(g::cur_language);
    let y = lw(sp + 260);
    sw(sp + 256, lw(sp + 264));
    let table = lw((language << 2) + g::languages);
    sw(sp + 260, y + 10);
    textPrint(sp + 256, sp + 260, lw(table + offset), colour);
}

/// Print the text `text` ending at the right of the page (sp+268).
fn print_value(sp: i32, text: i32, colour: i32) {
    let width = textWidth(text);
    sw(sp + 256, lw(sp + 268) - width);
    textPrint(sp + 256, sp + 260, text, colour);
}

/// Format the number `value` and print it at the right.
fn print_number(sp: i32, value: i32, colour: i32) {
    let text = vafmt(&[0x3ae858_i64, value as i64]);
    print_value(sp, text, colour);
}

/// Draw the lines of information of one statistics page: `a0` and `a1` are the left and right of the lines at the line y 93 down, `a2` the colour, `a3` the statistics
/// record, `a4` the kind of record (0 the one for a level, 1 for a player, 2 a team, 3 all), `a5` the flag for the comparison page and `a6` the player (@ 0x23cc38). Without the
/// flag the page shows the time played (days, hours, minutes and seconds as far as they are not zero), the counts of kills and the rest, the hit ratio and the best weapon (the
/// gun with the most kills). With it, the page shows the counts of the first record, the rate of hits, the playing rate, and the number of the 48 tiles that are 1.
pub fn statistics_pageDrawInfo(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) {
    let frame = Frame::new(480);
    let sp = frame.sp();
    sw(sp + 264, a0);
    sw(sp + 268, a1);
    sw(sp + 272, a4);
    sw(sp + 276, a6);
    sw(sp + 260, 93);
    if a5 != 0 {
        let language = lw(g::cur_language);
        sw(sp + 256, lw(sp + 264));
        textPrint(sp + 256, sp + 260, lw(lw((language << 2) + g::languages) + 1156), a2);
        print_number(sp, lw(a3 + 28), a2);
        print_label(sp, 1160, a2);
        let count = lw(a3 + 28);
        let text;
        if count <= 0 {
            text = language_text(1164);
        } else {
            let ratio = lw(a3 + 32) as f32 / count as f32;
            let percent = (ratio * 1e+02_f32) as i32;
            if percent != 0 {
                text = vafmt(&[0x3ae860_i64, percent as i64, ratio.to_bits() as i64]);
            } else {
                text = vafmt(&[0x3ae868_i64, fptodp(ratio)]);
            }
        }
        print_value(sp, text, a2);
        print_label(sp, 1168, a2);
        let count = lw(a3 + 32);
        let text;
        if count <= 0 {
            text = language_text(1172);
        } else {
            let percent = (lwc1(a3 + 36) / count as f32 * 1e+02_f32) as i32;
            text = vafmt(&[0x3ae860_i64, percent as i64]);
        }
        print_value(sp, text, a2);
        print_label(sp, 1200, a2);
        print_number(sp, lw(a3 + 52), a2);
        print_label(sp, 1204, a2);
        print_number(sp, lw(a3 + 56), a2);
        print_label(sp, 1208, a2);
        print_number(sp, lw(a3 + 60), a2);
        print_label(sp, 1212, a2);
        let rate = lwc1(a3 + 344) / lwc1(0x3ac9e8);
        let text;
        if !(0.0_f32 < rate) {
            text = language_text(1432);
        } else {
            let value = fptodp(rate);
            let unit = language_text(1216);
            text = vafmt(&[0x3ae870_i64, value, unit as i64]);
        }
        print_value(sp, text, a2);
        print_label(sp, 1220, a2);
        let hours = lw(a3 + 4) as f32 * lwc1(0x3ac9ec);
        let text;
        if !(0.0_f32 < hours) {
            text = language_text(1440);
        } else {
            let per_hour = rate / (hours / 3.6e+03_f32);
            let value = fptodp(per_hour);
            let unit = language_text(1224);
            text = vafmt(&[0x3ae878_i64, value, unit as i64]);
        }
        let text = 'tiles: {
            print_value(sp, text, a2);
            print_label(sp, 1148, a2);
            let mut ones = 0;
            if lw(sp + 272) != 0 {
                let language = lw(g::cur_language);
                if lw(sp + 272) != 2 {
                    break 'tiles lw(lw((language << 2) + g::languages) + 1444);
                }
            }
            let tiles = a3 + 348;
            let mut i = 0;
            loop {
                let tile = lb(tiles + i);
                i += 1;
                ones = if (tile ^ 1) == 0 { ones + 1 } else { ones };
                if i >= 48 {
                    break;
                }
            }
            vafmt(&[0x3ae858_i64, ones as i64, i as i64, tiles as i64])
        };
        print_value(sp, text, a2);
    } else {
        let language = lw(g::cur_language);
        sw(sp + 256, a0);
        textPrint(sp + 256, sp + 260, lw(lw((language << 2) + g::languages) + 1072), a2);
        let seconds_f = lw(a3 + 4) as f32;
        let scale = lwc1(0x3ac9e4);
        sb(sp, 0);
        let total = (seconds_f * scale) as i32;
        sw(sp + 280, total.wrapping_rem(60));
        sw(sp + 284, total.wrapping_div(60).wrapping_rem(60));
        let days = total.wrapping_div(60).wrapping_div(60).wrapping_div(24);
        let language = lw(g::cur_language);
        let text;
        if total > 0 {
            if total.wrapping_div(60).wrapping_div(60).wrapping_div(24) > 0 {
                let unit = lw(lw((language << 2) + g::languages) + 1080);
                let part = vafmt(&[0x3ae850_i64, days as i64, unit as i64]);
                strcat(sp, part);
            }
            if total.wrapping_div(60).wrapping_div(60) > 0 {
                let hours = total.wrapping_div(60).wrapping_div(60).wrapping_rem(24);
                let unit = language_text(1084);
                let part = vafmt(&[0x3ae850_i64, hours as i64, unit as i64]);
                strcat(sp, part);
            }
            if total.wrapping_div(60) > 0 {
                let minutes = lw(sp + 284);
                let unit = language_text(1088);
                let part = vafmt(&[0x3ae850_i64, minutes as i64, unit as i64]);
                strcat(sp, part);
            }
            let seconds = lw(sp + 280);
            let unit = language_text(1092);
            let part = vafmt(&[0x3ae850_i64, seconds as i64, unit as i64]);
            strcat(sp, part);
            text = sp;
        } else {
            text = lw(lw((language << 2) + g::languages) + 1076);
        }
        print_value(sp, text, a2);
        print_label(sp, 1096, a2);
        print_number(sp, lw(a3), a2);
        print_label(sp, 1100, a2);
        print_number(sp, lw(a3 + 8), a2);
        print_label(sp, 1104, a2);
        print_number(sp, lw(a3 + 12), a2);
        print_label(sp, 1108, a2);
        print_number(sp, lw(a3 + 16), a2);
        print_label(sp, 1112, a2);
        print_number(sp, lw(a3 + 20), a2);
        print_label(sp, 1116, a2);
        print_number(sp, lw(a3 + 24), a2);
        print_label(sp, 1120, a2);
        let language = lw(g::cur_language);
        let text;
        if lw(sp + 272) != 1 {
            text = lw(lw((language << 2) + g::languages) + 1128);
        } else {
            let shots = lw(a3);
            if shots <= 0 {
                text = language_text(1124);
            } else {
                let percent = (lw(a3 + 340) as f32 / shots as f32 * 1e+02_f32) as i32;
                text = vafmt(&[0x3ae860_i64, percent as i64]);
            }
        }
        print_value(sp, text, a2);
        print_label(sp, 1132, a2);
        let language = lw(g::cur_language);
        let text;
        if lw(sp + 272) != 1 {
            text = lw(lw((language << 2) + g::languages) + 1136);
        } else {
            text = vafmt(&[0x3ae858_i64, lw(a3 + 336) as i64]);
        }
        print_value(sp, text, a2);
        print_label(sp, 1424, a2);
        let kind = lw(sp + 272);
        let mut best = -1;
        let text;
        if kind == 3 {
            let language = lw(g::cur_language);
            sw(sp + 288, a3 + 200);
            text = lw(lw((language << 2) + g::languages) + 1436);
        } else {
            sw(sp + 288, a3 + 200);
            let mut level = 0;
            let mut most = 0;
            let mut at = 0;
            loop {
                let mut count = 0;
                if (kind as u32) < 2 {
                    let player = statsGetPlayer(lw(sp + 276));
                    count = lw(player + at + 320);
                }
                let mut more;
                'count: {
                    let record = lw(sp + 272);
                    if record != 0 {
                        more = (most < count) as i32;
                        if record != 2 {
                            break 'count;
                        }
                    }
                    let player = statsGetPlayer(lw(sp + 276));
                    count += lw(player + at + 352);
                    more = (most < count) as i32;
                }
                if more == 0 {
                    level += 1;
                } else {
                    most = count;
                    best = level;
                    level += 1;
                }
                at += 36;
                if level >= 30 {
                    break;
                }
            }
            let language = lw(g::cur_language);
            if best >= 0 {
                text = lvGetNameFromNum(best);
            } else {
                text = lw(lw((language << 2) + g::languages) + 1428);
            }
        }
        print_value(sp, text, a2);
        print_label(sp, 1140, a2);
        let language = lw(g::cur_language);
        let mut gun = 0;
        let mut at = lw(sp + 288);
        let mut i = 0;
        let mut top = 0.0_f32;
        loop {
            let value = lwc1(at);
            if !(top < value) {
                i += 1;
            } else {
                top = value;
                gun = i;
                i += 1;
            }
            at += 4;
            if i >= 34 {
                break;
            }
        }
        let text;
        if gun > 0 {
            let entry;
            if gun != 33 {
                let combo = gun << 3;
                entry = (lh(combo + (GUNCOMBOS + 6)) << 2) + lw((language << 2) + g::languages);
            } else {
                entry = (lh(g::fixedgunname) << 2) + lw((language << 2) + g::languages);
            }
            text = lw(entry);
        } else {
            text = lw(lw((language << 2) + g::languages) + 1144);
        }
        print_value(sp, text, a2);
    }
}
