//! Function: the tick of the page of the map files of the map maker (loading, saving, deleting and naming maps).

use crate::*;

/// The state of the page, kept in a byte.
const STATE: i32 = 0x3b0038;
/// The map file in the buffer for the entry being edited (a copy of the entry of the directory).
const ENTRY: i32 = 0x1fc4018;
/// The colour and the style the messages of the page are shown with.
const MESSAGE_STYLE: i32 = 0x6464647f;

/// Copy a 40 byte entry of the directory of the map files.
fn copy_entry(dst: i32, src: i32) {
    let a = ld(src + 8);
    let b = ld(src + 16);
    sd(dst, ld(src));
    sd(dst + 8, a);
    sd(dst + 16, b);
    let c = ld(src + 32);
    sd(dst + 24, ld(src + 24));
    sd(dst + 32, c);
}

/// Copy a 32 byte item of the menu.
fn copy_item(dst: i32, src: i32) {
    let a = ld(src + 8);
    let b = ld(src + 16);
    sd(dst, ld(src));
    sd(dst + 8, a);
    sd(dst + 16, b);
    sd(dst + 24, ld(src + 24));
}

/// Make the menu of the page again.
fn remake_menu(page: i32) {
    let menu = menumake(lw(page + 8), g::mapfile_menu_356ba8);
    sw(page + 12, menu);
}

/// Go back to the first state, with the directory read again.
fn restart_directory() {
    mmfileDirReset();
    sb(STATE, 0);
}

/// Show the message of a failed memory card action and wait for it (state 15).
fn report_card_error() {
    mcseqWindowClose();
    let message = mcseqGetMsg(6);
    mmStartMessage(message, MESSAGE_STYLE, 0x24d268);
    sb(STATE, 15);
}

/// Load the map in the buffer, close the menu and say what happened.
fn load_map(page: i32) {
    mmExitMenu(page);
    let loaded = mmLoadMapFile(lw(g::mmtempbuffer), 0);
    if loaded == 0 {
        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3940);
        mmStartMessage(text, MESSAGE_STYLE, 0);
        return;
    }
    mmInitView();
    mmCenterMap();
    let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3936);
    mmStartMessage(text, MESSAGE_STYLE, 0);
    sw(0x3b004c, lw(0x3b0048) - lw(0x3b0054));
}

/// Start saving the map of the entry to the memory card (state 13), then make the menu again.
fn save_map(page: i32) {
    let language = lw(g::cur_language);
    let block = lhu(ENTRY + 36);
    let strings = lw((language << 2) + g::languages);
    let size = lhu(ENTRY + 38);
    let offset = mmfileOffset(block);
    mcseqSave(2, lw(strings + 1892), 0, 0x3a2100, lw(g::mmtempbuffer), size, offset);
    sb(STATE, 13);
    remake_menu(page);
}

/// Wait for the message that was started to go away, then go on with the state it asks for (the state 8 when a map was found by name, 10 when the name is new).
fn wait_for_message(page: i32) {
    let message = messageGetState_24c908();
    if message == 1 {
        sb(STATE, 10);
        remake_menu(page);
        return;
    }
    if message < 2 {
        remake_menu(page);
        return;
    }
    if message != 2 {
        remake_menu(page);
        return;
    }
    if (lw(g::editflags) & 0x200000) != 0 {
        sb(STATE, 8);
        remake_menu(page);
        return;
    }
    sb(STATE, message);
    remake_menu(page);
}

/// Tick of the page of the map files (@ 0x24d6b8): a state machine that reads the directory of the maps (from the memory card or the built in maps), shows them in a menu
/// of 10 at a time, and loads, saves, deletes, makes and names maps; the state is a byte (0x3b0038) and the menu is made again at the end of most of the states.
pub fn mapfile_pageTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let state = lbu(STATE);
    sw(sp, a0);
    if (state as u32) >= (18 as u32) {
        remake_menu(a0);
        return;
    }
    let mut go_on_to_input = false;
    match state {
        0 => {
            if lw(lw(g::mapdir1) + 4) != 0 {
                sb(STATE, 2);
                return;
            }
            mcseqUnvalidateCards();
            let language = lw(g::cur_language);
            let title = if (lw(g::editflags) & 2048) == 0 {
                lw(lw((language << 2) + g::languages) + 1948)
            } else {
                lw(lw((language << 2) + g::languages) + 1944)
            };
            let mut flags;
            'b17: {
                let size = mmfileDirSize();
                let dir = lw(g::mapdir1);
                mcseqLoad(1, title, 0, 0x3a2100, dir, size, 0);
                flags = lw(g::editflags);
                let language = lw(g::cur_language);
                if (flags & 2048) != 0 {
                    let option = lw(lw((language << 2) + g::languages) + 1952);
                    mcseqAddOption(option);
                    flags = lw(g::editflags);
                    if (flags & 2048) != 0 {
                        break 'b17;
                    }
                }
                sw(g::mcseq_flags, lw(g::mcseq_flags) | 32);
            }
            sb(STATE, 1);
            sw(0x3b004c, -1);
            sw(g::editflags, flags & -0x10001);
            sw(g::firstmap_3ae9bc, 0);
            return;
        }
        1 => {
            if mcseqGetState() != 1_i64 {
                return;
            }
            mcseqWindowClose();
            let result = mcseqGetResult();
            let choice = result - 1;
            if (choice as u32) < (9 as u32) {
                match choice {
                    0 => {
                        let dir = lw(g::mapdir1) + 4;
                        let size = mmfileDirSize();
                        let bad = crcCheck(dir, size - 4, lw(lw(g::mapdir1)));
                        if bad != 0 {
                            sb(STATE, 2);
                            return;
                        }
                        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3868);
                        mmStartMessage(text, MESSAGE_STYLE, 0);
                        mmfileDirReset();
                        sw(lw(g::mapdir1) + 4, 1);
                        sb(STATE, 2);
                        return;
                    }
                    1 | 2 | 4 | 6 => {}
                    3 => {
                        mmExitMenu(lw(sp));
                        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3872);
                        mmStartMessage(text, MESSAGE_STYLE, 0);
                        return;
                    }
                    5 => {
                        let strings = g::languages;
                        let language_strings = lw((lw(g::cur_language) << 2) + strings);
                        let name = mcseqSlotGetLongName();
                        let first = lw(language_strings + 3876);
                        let language_strings = lw((lw(g::cur_language) << 2) + strings);
                        let third = lw(language_strings + 3884);
                        let second = lw(language_strings + 3880);
                        let text = vafmt(&[0x3a2130, first as i64, name as i64, second as i64, third as i64]);
                        mmStartMessage(text, MESSAGE_STYLE, 0x24d268);
                        sb(STATE, 16);
                        return;
                    }
                    7 => {
                        mmExitMenu(lw(sp));
                        return;
                    }
                    _ => {
                        mmfileDirBuiltinMaps();
                        let flags = lw(g::editflags) | 0x10000;
                        sb(STATE, 2);
                        sw(g::editflags, flags);
                        return;
                    }
                }
            }
            mmfileDirReset();
            let dir = lw(g::mapdir1);
            sw(dir + 4, 1);
            sb(STATE, 2);
            return;
        }
        2 => {
            let flags = lw(g::editflags);
            let mut on_card = flags & 2048;
            if (flags & 0x10000) == 0 {
                mcseqPollLoop();
                on_card = lw(g::editflags) & 2048;
            }
            let mut has_parent = 0;
            let dir = lw(g::mapdir1);
            if on_card == 0 {
                has_parent = (lw(dir + 4) != -666) as i32;
            }
            let count = lw(dir + 8);
            sw(0x3b0054, has_parent);
            if count <= 0 {
                let page = lw(sp);
                if has_parent == 0 {
                    sh(lw(sp) + 4, 666);
                } else {
                    sh(page + 4, 1);
                }
            } else {
                let parent = lhu(0x3b0054);
                let selected = if lw(0x3b004c) < 0 { 1 } else { parent + lhu(0x3b004c) - lhu(g::firstmap_3ae9bc) + 1 };
                sh(lw(sp) + 4, selected);
            }
            go_on_to_input = true;
        }
        3 => {
            go_on_to_input = true;
        }
        4 => {}
        5 => {
            let message = messageGetState_24c908();
            if message == 1 {
                mmfileEntryDelete(lw(0x3b0048));
                updateDir(1);
                sb(0x3b0044, 0);
                sw(0x3b0048, 0);
                return;
            }
            if message < 2 {
                remake_menu(lw(sp));
                return;
            }
            if message == 2 {
                sb(STATE, message);
                remake_menu(lw(sp));
                return;
            }
            remake_menu(lw(sp));
            return;
        }
        6 => {
            let strings = g::languages;
            let language_strings = lw((lw(g::cur_language) << 2) + strings);
            let entry_name = lw(g::mapdir1) + (lw(0x3b0048) * 40 + 12);
            let slot = mcseqSlotGetLongName();
            let third = lw(language_strings + 3908);
            let first = lw(language_strings + 3904);
            let fourth = lw(lw((lw(g::cur_language) << 2) + strings) + 3912);
            let text = vafmt(&[0x3a2140, first as i64, entry_name as i64, third as i64, slot as i64, fourth as i64]);
            mmStartMessage(text, MESSAGE_STYLE, 0x24d2a0);
            let entry = lw(0x3b0048) * 40 + lw(g::mapdir1);
            copy_entry(ENTRY, entry + 12);
            sb(STATE, 7);
            sb(0x3b0044, 0);
            wait_for_message(lw(sp));
            return;
        }
        7 => {
            wait_for_message(lw(sp));
            return;
        }
        8 => {
            sb(0x3b0044, 1);
            stringeditStart(0, ENTRY, 31, 1, 264, 0, 0);
            frontPageSet(lw(lw(g::front) + 20), g::mapnameedit_page);
            let front = lw(g::front);
            sw(front + 12, lw(front + 12) & -4);
            sb(STATE, 9);
            return;
        }
        9 => {
            stringeditSet(0);
            let textbox = lw(g::textbox);
            if lw(textbox + 32) == 0 {
                if lw(textbox + 36) == 0 {
                    remake_menu(lw(sp));
                    return;
                }
                sb(STATE, 2);
                remake_menu(lw(sp));
                return;
            }
            let mut found = 0;
            if lb(ENTRY) == 0 {
                sb(STATE, 2);
                remake_menu(lw(sp));
                return;
            }
            loop {
                let dir = lw(g::mapdir1);
                if found >= lw(dir + 8) {
                    sb(STATE, 10);
                    remake_menu(lw(sp));
                    return;
                }
                let same = strcmp(dir + (found * 40 + 12), ENTRY);
                if same == 0 {
                    break;
                }
                found = found + 1;
            }
            let flags = lw(g::editflags);
            sw(0x3b0048, found);
            sb(STATE, 6);
            sw(g::editflags, flags | 0x200000);
            if found < lw(lw(g::mapdir1) + 8) {
                remake_menu(lw(sp));
                return;
            }
            sb(STATE, 10);
            remake_menu(lw(sp));
            return;
        }
        10 => {
            let time = rtcGetTime();
            sw(ENTRY + 32, time);
            strcpy(g::mapname, ENTRY);
            let size = mmSaveMapFile(lw(g::mmtempbuffer), 0);
            let length = size & 0xffff;
            sh(ENTRY + 38, size);
            if length == 0 {
                mmExitMenu(lw(sp));
                return;
            }
            let crc = crcCalc(lw(g::mmtempbuffer), length);
            let length_now = lhu(ENTRY + 38);
            let dir = lw(g::mapdir1);
            sw(lw(g::mmtempbuffer) + lhu(ENTRY + 38), crc);
            sh(ENTRY + 38, length_now + 4);
            let language = lw(g::cur_language);
            if lw(dir + 4) != 1 {
                save_map(lw(sp));
                return;
            }
            let strings = lw((language << 2) + g::languages);
            let max = mmfileMaxSize();
            mcseqSave(3, lw(strings + 1892), 0, 0x3a2100, lw(g::mapdir1), max, 0);
            sb(STATE, 12);
            remake_menu(lw(sp));
            return;
        }
        11 => {
            save_map(lw(sp));
            return;
        }
        12 => {
            if mcseqGetState() != 3_i64 {
                return;
            }
            let result = mcseqGetResult();
            if result == 7 {
                mcseqWindowClose();
                let message = mcseqGetMsg(12);
                mmStartMessage(message, MESSAGE_STYLE, 0);
                sb(STATE, 2);
                return;
            }
            if result >= 8 {
                if result == 8 {
                    mcseqWindowClose();
                    sb(STATE, 2);
                    return;
                }
            } else {
                let dir = lw(g::mapdir1);
                if result == 1 {
                    sw(dir + 4, lw(dir + 4) + 1);
                    sb(STATE, 11);
                    return;
                }
            }
            report_card_error();
            return;
        }
        13 => {
            let card_state = mcseqGetState();
            if card_state != 2_i64 {
                return;
            }
            let result = mcseqGetResult();
            if result == 7 {
                mcseqWindowClose();
                let message = mcseqGetMsg(12);
                mmStartMessage(message, MESSAGE_STYLE, 0);
                sb(STATE, card_state as i32);
                return;
            }
            if result >= 8 {
                if result == 8 {
                    mcseqWindowClose();
                    sb(STATE, card_state as i32);
                    return;
                }
            } else {
                let is_new = lbu(0x3b0044);
                if result == 1 {
                    let index = lw(0x3b0048);
                    if is_new == 0 {
                        let entry = index * 40 + lw(g::mapdir1);
                        copy_entry(entry + 12, ENTRY);
                    } else {
                        let made = mmfileEntryNew(ENTRY);
                        sw(0x3b0048, made);
                    }
                    updateDir(0);
                    return;
                }
            }
            report_card_error();
            return;
        }
        14 => {
            if mcseqGetState() != 3_i64 {
                return;
            }
            let result = mcseqGetResult();
            let is_new = lbu(0x3b0044);
            if result == 1 {
                mcseqWindowClose();
                let flags = lw(g::editflags);
                if (flags & 0x20000) != 0 {
                    sw(g::firstmap_3ae9bc, 0);
                    sb(STATE, 2);
                    return;
                }
                let index = lw(0x3b0048) - lw(0x3b0054);
                let page = lw(sp);
                sw(g::editflags, flags & -257);
                sw(0x3b004c, index);
                mmExitMenu(page);
                let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3916);
                mmStartMessage(text, MESSAGE_STYLE, 0);
                return;
            }
            if is_new != 0 {
                mmfileEntryDelete(lw(0x3b0048));
            }
            report_card_error();
            return;
        }
        15 => {
            let message = messageGetState_24c908();
            if message == 1 {
                mmExitMenu(lw(sp));
                return;
            }
            if message < 2 {
                return;
            }
            if message == 2 {
                restart_directory();
                return;
            }
            return;
        }
        16 => {
            let message = messageGetState_24c908();
            if message == 0 {
                return;
            }
            if message < 0 {
                return;
            }
            if message >= 3 {
                return;
            }
            restart_directory();
            return;
        }
        _ => {
            let card_state = mcseqGetState();
            if card_state != 1_i64 {
                remake_menu(lw(sp));
                return;
            }
            mcseqWindowClose();
            let result = mcseqGetResult();
            if (result as i64) != card_state {
                mmExitMenu(lw(sp));
                let message = mcseqGetMsg(3);
                mmStartMessage(message, MESSAGE_STYLE, 0);
                sb(STATE, 2);
                remake_menu(lw(sp));
                return;
            }
            let buffer = lw(g::mmtempbuffer);
            let length = lhu(ENTRY + 38);
            let crc = lw(buffer + length - 4);
            let bad = crcCheck(buffer, length - 4, crc);
            if bad != 0 {
                load_map(lw(sp));
                return;
            }
            mmExitMenu(lw(sp));
            let strings = lw((lw(g::cur_language) << 2) + g::languages);
            let fourth = lw(strings + 3932);
            let first = lw(strings + 3920);
            let second = lw(strings + 3924);
            let third = lw(strings + 3928);
            let text = vafmt(&[0x3a2180, first as i64, ENTRY as i64, second as i64, third as i64, fourth as i64]);
            mmStartMessage(text, MESSAGE_STYLE, 0);
            return;
        }
    }
    if go_on_to_input {
        build_menu(sp);
    }
    handle_input(sp);
}

/// Make the menu of the page from the directory (@ the states 2 and 3): the item to go up or to make a new map, the 10 maps from the first one shown, and the item to go back.
fn build_menu(sp: i32) {
    let flags = lw(g::editflags);
    let mut item = g::mfmi_356bb8;
    sw(sp + 4, 25);
    sw(item - 16, 0);
    let flags_now;
    if (flags & 0x10000) == 0 {
        let long_name = mcseqSlotGetLongName();
        let language = lw(g::cur_language);
        if long_name != 0 {
            let strings = lw((language << 2) + g::languages);
            let short_name = mcseqSlotGetShortName();
            let format = lw(strings + 3888);
            let text = vafmt(&[(g::firstmap_3ae9bc + 4) as i64, format as i64, short_name as i64]);
            strcpy(0x1fc3e80, text);
        }
        flags_now = lw(g::editflags);
    } else {
        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1952);
        strcpy(0x1fc3e80, text);
        flags_now = lw(g::editflags);
    }
    'b28: {
        if (flags_now & 0x10000) == 0 {
            let long_name = mcseqSlotGetLongName();
            if long_name == 0 {
                break 'b28;
            }
        }
        copy_item(item, g::mi_dirtop_356ae8);
        item = item + 32;
        sw(g::mapfile_menu_356ba8, lw(g::mapfile_menu_356ba8) + 1);
    }
    let mut dir = lw(g::mapdir1);
    let mut parent = lw(0x3b0054);
    let first = lw(g::firstmap_3ae9bc);
    let total = lw(dir + 8) + parent;
    let shown = total - first;
    sw(0x3b0058, total);
    sw(0x3b0050, shown);
    if shown >= 11 {
        sw(0x3b0050, 10);
    }
    let mut n = 0;
    if lw(0x3b0050) > 0 {
        let mut index = first + n;
        loop {
            let next_item = item + 32;
            let next_n;
            let next_y;
            if index >= parent {
                copy_item(item, g::mi_dirfile_356b08);
                let mut length = 0;
                strcpy(n * 34 + 0x1fc3ec0, lw(g::mapdir1) + ((index - parent) * 40 + 12));
                let y = lw(sp + 4);
                next_n = n + 1;
                sw(sp + 8, item + 32);
                next_y = y + 10;
                let buffer = 0x1fc3ec0;
                let start = n * 34;
                let name = n * 34 + buffer;
                loop {
                    let width = textWidth(name);
                    if width < 261 {
                        break;
                    }
                    length = strlen(name);
                    if length == 1 {
                        break;
                    }
                    sb(length + (start - 1) + buffer, 0);
                }
                if length != 0 {
                    sb(length + (start - 1) + buffer, 46);
                    sb(length + start + buffer, 46);
                    sb(length + (start + 1) + buffer, 46);
                    sb(length + (start + 2) + buffer, 0);
                }
                sw(item, name);
            } else {
                sw(sp + 8, next_item);
                next_n = n + 1;
                copy_item(item, g::mi_dirnew);
                let flags_of_item = lhu(item + 10);
                next_y = lw(sp + 4) + 10;
                sh(item + 10, if lw(lw(g::mapdir1) + 8) < 100 { flags_of_item } else { flags_of_item | 64 });
            }
            sh(item + 8, next_n);
            if n <= 0 {
                sh(item + 16, 0);
            } else {
                sh(item + 16, n);
            }
            let last = lw(0x3b0050);
            let mut below = 0;
            if n < last - 1 {
                below = lhu(item + 8) + 1;
            }
            let menu_count = lw(g::mapfile_menu_356ba8);
            n = next_n;
            sh(item + 6, lw(sp + 4));
            sh(item + 18, below);
            item = lw(sp + 8);
            sw(g::mapfile_menu_356ba8, menu_count + 1);
            sw(sp + 4, next_y);
            if n >= last {
                break;
            }
            parent = lw(0x3b0054);
            index = lw(g::firstmap_3ae9bc) + n;
        }
        dir = lw(g::mapdir1);
    }
    'b40: {
        'b41: {
            let flags_last = lw(g::editflags);
            if lw(dir + 4) != -666 {
                if (flags_last & 2048) != 0 {
                    if lw(dir + 8) == 0 {
                        break 'b41;
                    }
                }
                let back = if (flags_last & 0x10000) == 0 { g::mi_dirdelback } else { g::mi_dirback_356b68 };
                copy_item(item, back);
                sw(g::mapfile_menu_356ba8, lw(g::mapfile_menu_356ba8) + 1);
                break 'b40;
            }
        }
        copy_item(item, g::mi_dirempty_356b48);
        sw(g::mapfile_menu_356ba8, lw(g::mapfile_menu_356ba8) + 1);
        copy_item(item + 32, g::mi_dirback_356b68);
        sw(g::mapfile_menu_356ba8, lw(g::mapfile_menu_356ba8) + 1);
    }
    sb(STATE, 4);
}

/// Read the controller for the page (the end of the states 2 to 4): scroll the list when the cursor goes past the ends, or start deleting (button 128), naming a new map
/// or loading the map the cursor is on (button 64), or leave the page (button 16).
fn handle_input(sp: i32) {
    let page = lw(sp);
    if (lw(g::editflags) & 0x10000) == 0 {
        let card_state = mcseqGetState();
        if card_state >= 6_i64 {
            if card_state < 8_i64 {
                restart_directory();
                return;
            }
            let dir = lw(g::mapdir1);
            if card_state == 8_i64 {
                if lw(dir + 8) != 0 {
                    restart_directory();
                    return;
                }
            }
        }
        let polls = mcseqPollCount();
        if polls == 0 {
            remake_menu(page);
            return;
        }
    }
    'b46: {
        let cursor = lhu(page + 4);
        sw(0x3b0048, lw(g::firstmap_3ae9bc) + cursor - (lw(0x3b0054) + 1));
        if cursor == 1 {
            if (lw(lw(g::front) + 12) & 4) != 0 {
                soundStart(140, 0);
                let first = lw(g::firstmap_3ae9bc) - 1;
                sw(g::firstmap_3ae9bc, first);
                if first >= 0 {
                    break 'b46;
                }
                let total = lw(0x3b0058);
                sw(g::firstmap_3ae9bc, if total < 10 { 0 } else { total - 10 });
                sh(page + 4, if total < 11 { total } else { 10 });
                break 'b46;
            }
        }
        let front = lw(g::front);
        if cursor == lw(0x3b0050) {
            if (lw(front + 12) & 8) != 0 {
                soundStart(140, 0);
                let first = lw(g::firstmap_3ae9bc) + 1;
                let past_end = ((lw(0x3b0058) - 10) < first) as i32;
                sw(g::firstmap_3ae9bc, first);
                if past_end == 0 {
                    break 'b46;
                }
                sw(g::firstmap_3ae9bc, 0);
                sh(page + 4, 1);
                break 'b46;
            }
        }
        let buttons = joyNewBut(0);
        let flags = lw(g::editflags);
        if (buttons & 16) == 0 {
            if (flags & 0x10000) == 0 {
                let buttons = joyNewBut(0);
                let dir = lw(g::mapdir1);
                if (buttons & 128) != 0 {
                    let index = lw(0x3b0048);
                    if (index as u32) < (lw(dir + 8) as u32) {
                        let strings = g::languages;
                        let language_strings = lw((lw(g::cur_language) << 2) + strings);
                        let slot = mcseqSlotGetLongName();
                        let third = lw(language_strings + 3896);
                        let first = lw(language_strings + 3892);
                        let entry_name = dir + (index * 40 + 12);
                        let fourth = lw(lw((lw(g::cur_language) << 2) + strings) + 3900);
                        let text = vafmt(&[0x3a2140, first as i64, entry_name as i64, third as i64, slot as i64, fourth as i64]);
                        mmStartMessage(text, MESSAGE_STYLE, 0x24d268);
                        sb(STATE, 5);
                        return;
                    }
                }
            }
            let buttons = joyNewBut(0);
            let flags = lw(g::editflags);
            if (buttons & 64) != 0 {
                let dir = lw(g::mapdir1);
                if (flags & 2048) == 0 {
                    let index = lw(0x3b0048);
                    if lw(0x3b0054) != 0 {
                        if index == -1 {
                            let room = (lw(lw(g::mapdir1) + 8) < 100) as i32;
                            sw(g::editflags, flags & -0x200001);
                            if room == 0 {
                                soundStart(142, 0);
                                return;
                            }
                            sb(ENTRY, lbu(0x3ae9c8));
                            let block = mmfileGetFreeBlock();
                            sh(ENTRY + 36, block);
                            sh(ENTRY + 38, 0);
                            sb(STATE, 8);
                            return;
                        }
                    }
                    sb(STATE, 6);
                    return;
                }
                let index = lw(0x3b0048);
                if (index as u32) < (lw(dir + 8) as u32) {
                    if (flags & 0x10000) == 0 {
                        let block = lhu(index * 40 + dir + 48);
                        let offset = mmfileOffset(block);
                        let strings = lw((lw(g::cur_language) << 2) + g::languages);
                        let entry = lw(0x3b0048) * 40 + lw(g::mapdir1);
                        copy_entry(ENTRY, entry + 12);
                        let buffer = lw(g::mmtempbuffer);
                        let title = lw(strings + 1888);
                        let size = lhu(0x1fc403e);
                        mcseqLoad(1, title, 0, 0x3a2100, buffer, size, offset);
                        sb(STATE, 17);
                        return;
                    }
                    let loaded = mmfileLoadBuiltinMap(index);
                    if loaded != 0 {
                        load_map(page);
                        return;
                    }
                    mmExitMenu(page);
                    mmStartMessage(0x3a2158, 0x7f7f3c7f, 0);
                    return;
                }
            }
            menutick_24c3a8(page, g::mapfile_menu_356ba8);
            remake_menu(page);
            return;
        }
        if (flags & 2048) != 0 {
            restart_directory();
            return;
        }
        mmExitMenu(page);
        return;
    }
    let front = lw(g::front);
    sw(front + 12, lw(front + 12) & -61);
    sb(STATE, 3);
    remake_menu(page);
}
