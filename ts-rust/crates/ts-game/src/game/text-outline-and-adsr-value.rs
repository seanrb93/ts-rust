//! Drawing a text with an outline, and the value of an envelope in the sound tool.

use crate::*;

/// Draw the text `text` at (`x`, `y`) in colour `colour` with an outline of colour `outline`: the text is drawn first in the outline colour at 2 left, 1 up, 2 right and 1 down of
/// the place (the font flags 4 and 8 set, so that the drawing leaves the dither and does not reset it), then in its own colour on the place; the dither matrix is set to 2 for all of it and
/// put back (@ 0x2023b0).
pub fn textOutline(x: i32, y: i32, text: i32, colour: i32, outline: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    sw(sp, x);
    sw(sp + 4, y);
    let old = dlSetDitherMatrix(2);
    let flags = lbu(g::fontflags);
    sw(sp, x - 2);
    sw(sp + 4, y);
    sb(g::fontflags, flags | 12);
    textPrint(sp, sp + 4, text, outline);
    sw(sp, x);
    sw(sp + 4, y - 1);
    textPrint(sp, sp + 4, text, outline);
    sw(sp + 4, y);
    sw(sp, x + 2);
    textPrint(sp, sp + 4, text, outline);
    sw(sp, x);
    sw(sp + 4, y + 1);
    textPrint(sp, sp + 4, text, outline);
    let flags = lbu(g::fontflags);
    sw(sp, x);
    sw(sp + 4, y);
    sb(g::fontflags, flags & 251);
    textPrint(sp, sp + 4, text, colour);
    dlSetDitherMatrix(old);
    sb(g::fontflags, lbu(g::fontflags) & 247);
}

/// Write the value of an envelope step of the sound tool's entry `entry` as a text (the kind is the halfword at +8 less 5: 0 a linear rise, 1 an exponential fall, 3 a level
/// of 15 steps, else a linear fall; the value is taken from the table of its kind by the halfword of `snd`'s array) and draw it, right-aligned at 224 from the entry's x, in grey
/// (@ 0x206e58).
pub fn adsrvalue_draw(entry: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    if entry == 0 {
        return;
    }
    let kind = lhu(entry + 8) - 5;
    let value = lh((kind << 1) + lw(g::snd + 4));
    'text: {
        let shown;
        let format;
        if kind != 3 {
            let table = if kind != 0 {
                if kind != 1 { g::lindectable } else { g::expdectable }
            } else {
                g::lininctable
            };
            let rate = lwc1((value << 2) + table);
            if rate < 0.0_f32 {
                sprintf(&[sp as i64, 0x399b48]);
                break 'text;
            }
            if !(rate < lwc1(0x3ac800)) {
                if !(lwc1(0x3ac804) <= rate) {
                    sprintf(&[sp as i64, 0x399b78, fptodp(rate)]);
                    break 'text;
                }
                sprintf(&[sp as i64, 0x399b68]);
                break 'text;
            }
            format = 0x399b58;
            shown = rate * 1e+03_f32;
        } else {
            shown = (value as f32).pdiv(15.0_f32);
            format = 0x3ae3a8;
        }
        sprintf(&[sp as i64, format as i64, fptodp(shown)]);
    }
    let width = textWidth(sp);
    let x = lhu(entry) - (width - 224);
    sw(sp + 20, lhu(entry + 2));
    sw(sp + 16, x);
    textPrint(sp + 16, sp + 20, sp, 0x64646480);
}
