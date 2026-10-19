//! The help text and the cursor of the map maker's map view.

use crate::*;

/// The string of the current language at `offset` of its record.
fn language_text(language: i32, offset: i32) -> i32 {
    lw(lw((language << 2) + g::languages) + offset)
}

/// Choose the three lines of button help text (`button_text`) and the cursor image (`mmcursor`) for the map view of the map maker, by what the mouse is over (`object`),
/// the edit mode and the edit flags: 8 while the flag 32 is set (no text), 2 + the panning direction while flag 2, 7 while flag 1, otherwise by the mode: 1 and 2 give a text
/// each (2 may set flag 0x100000 and cursor 15), the other modes 0, 1 or 2 by whether an object is in the hand (@ 0x2465f8).
pub fn mmMapViewHelp() {
    let mut cursor = 16;
    let object = mmObjectUnderCursor();
    let old_flags = lw(g::editflags);
    let flags = lw(g::editflags) & -0x100001;
    let locked = lw(g::editflags) & 32;
    sw(g::editflags, flags);
    if locked != 0 {
        cursor = 8;
        sw(g::button_text, 0);
        sw(g::button_text + 12, 0);
        sw(g::button_text + 8, 0);
    } else {
        let panning = lbu(g::panning);
        if old_flags & 2 != 0 {
            cursor = panning + 2;
        } else if old_flags & 1 != 0 {
            cursor = 7;
        } else {
            let mode = lw(g::editmode);
            if object == 0 {
                let language = lw(g::cur_language);
                if lw(g::lassocount) != 0 {
                    sw(g::button_text + 8, language_text(language, 3592));
                    sw(g::mmcursor, cursor);
                    return;
                }
            }
            if mode == 1 {
                let language = lw(g::cur_language);
                if object == 0 {
                    sw(g::button_text + 4, language_text(language, 3600));
                } else {
                    sw(g::button_text + 4, language_text(language, 3596));
                }
            } else if mode == 2 {
                let language = lw(g::cur_language);
                if object == 0 {
                    sw(g::button_text + 4, language_text(lw(g::cur_language), 3620));
                } else {
                    let record = lw((language << 2) + g::languages);
                    let kind = lw(object);
                    sw(g::button_text + 8, lw(record + 3604));
                    if kind == 1 && (lbu(object + 28) as u32) >= 2 {
                        sw(g::button_text, lw(record + 3608));
                    }
                    let record = lw((language << 2) + g::languages);
                    sw(g::editflags, flags | 0x100000);
                    cursor = 15;
                    let third = lw(record + 3612);
                    sw(g::button_text + 4, lw(record + 3616));
                    sw(g::button_text + 12, third);
                }
            } else {
                let used = lw(g::usedobj);
                let language = lw(g::cur_language);
                if used == 0 {
                    if object != 0 {
                        let offset = if lw(g::lassocount) != 0 && flags & 32 == 0 && lw(object + 4) & 1 != 0 { 3636 } else { 3640 };
                        sw(g::button_text + 8, language_text(language, offset));
                        let record = lw((language << 2) + g::languages);
                        cursor = 2;
                        let third = lw(record + 3652);
                        let second = lw(record + 3648);
                        sw(g::button_text, lw(record + 3644));
                        sw(g::button_text + 12, third);
                        sw(g::button_text + 4, second);
                    } else {
                        sw(g::button_text + 4, language_text(language, 3656));
                    }
                } else {
                    let record = lw((language << 2) + g::languages);
                    let first = lw(record + 3628);
                    sw(g::button_text + 8, lw(record + 3624));
                    sw(g::button_text, first);
                    if flags & 0x400004 == 4 {
                        cursor = 0;
                        sw(g::button_text + 4, lw(record + 3632));
                    } else {
                        cursor = 1;
                    }
                }
            }
        }
    }
    sw(g::mmcursor, cursor);
}
