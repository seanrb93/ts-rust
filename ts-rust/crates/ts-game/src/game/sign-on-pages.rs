//! Functions: the sign on screens (the players that are made and kept on the memory cards) and their load, update and delete ticks.
//!
//! The players are `signonarray` (144 records of 2936 bytes) and `signoninfo` (8 bytes for each: the card the player is on or -1 when it is only in memory, 8 when it is new, and its id). The files are 0xb790 bytes. The state of each of the (up to 4) local players of the page is a byte at 0x3aff28; 1 start, 2 the list of the players, 3 make a name, 4 type the name, 5 ready. The bytes at 0x3aff40, 0x3aff4a and 0x3aff48 say which player (from 1) is loading, updating and deleting, and 0x3aff94 which one has a message with two parts. The state of the operation that is going on is the byte at 0x3aff4b.

use crate::*;

/// A text of the current language, from the table of texts at `offset`.
fn text(offset: i32) -> i32 {
    lw(lw((lw(g::cur_language) << 2) + g::languages) + offset)
}

/// Copy `count` doublewords from `from` to `to`.
fn copy_doublewords(to: i32, from: i32, count: i32) {
    for i in 0..count {
        sd(to + i * 8, ld(from + i * 8));
    }
}

/// Add `bits` to the flags of the sign on window (at 0x1fc11fc).
fn set_window_flags(bits: i32) {
    sw(0x1fc11fc, lw(0x1fc11fc) | bits);
}

/// Show the message `message` for the player `a0` with the buttons `buttons` and the second line `second`.
fn show_message(a0: i32, buttons: i32, message: i32, second: i32) {
    mcseqMessage(buttons, GetPlayerSignOnMessage(a0, message), second);
}

/// Close the window of the sequencer and clear the flag that it is used.
fn close_window() {
    mcseqWindowClose();
    sw(0x1fc11fc, lw(0x1fc11fc) & -2);
}

/// Make the array of the players: 144 records, all empty, and no card chosen by the 4 local players.
pub fn signonMake() {
    let array = memAlloc(0x67380, 2);
    sw(g::signonarray, array);
    memset(array, 0, 0x67380);
    for index in 0..144 {
        signonDelete(index);
    }
    for player in 0..4 {
        sw(0x1fbc158 + player * 4, -1);
    }
    sw(0x3aff04, 0);
}

/// What `signonLoadTick` does next: give the result, or wait (with the clock of the window set to 180).
enum LoadEnd {
    Result(i32),
    Wait,
}

/// The message for a card that has no sign on file.
fn no_file_message(a0: i32) -> LoadEnd {
    soundStart(142, 0);
    let lang = lw((lw(g::cur_language) << 2) + g::languages);
    let name = mcseqSlotGetLongName();
    let message = vafmt(&[0x3a19a0, lw(lang + 556) as i64, lw(lang + 560) as i64, name as i64]);
    show_message(a0, 2, message, text(1612));
    LoadEnd::Result(0)
}

/// The message for a card that cannot be read for a reason that is not the others.
fn cannot_load_message(a0: i32) -> LoadEnd {
    soundStart(142, 0);
    show_message(a0, 2, mcseqGetMsg(3), text(1612));
    LoadEnd::Result(0)
}

/// Do the step of the load for the state the sequencer is in. See `signonLoadTick`.
fn load_step(a0: i32) -> LoadEnd {
    let state = mcseqGetState();
    if (state as u64) >= 11 {
        return LoadEnd::Wait;
    }
    match state as i32 {
        0 => {
            if lw(g::mcseq_msgflags) & 15 != 0 {
                return LoadEnd::Result(0);
            }
            let left = lw(0x1fc11f8 + 24) - 1;
            sw(0x1fc11f8 + 24, left);
            LoadEnd::Result((left as u32 == 0) as i32)
        }
        1 | 4 => LoadEnd::Result(1),
        2 => {
            if lw(0x1fc11fc) & 12 == 0 {
                return LoadEnd::Result(1);
            }
            signonLoadRetry(a0);
            LoadEnd::Result(0)
        }
        3 => {
            if mcseqGetResult() != 4 {
                if lw(0x1fc11fc) & 12 == 0 {
                    return LoadEnd::Result(1);
                }
                if mcseqGetNumCards() < 2 {
                    return LoadEnd::Result(2);
                }
            }
            signonLoadStart(a0);
            LoadEnd::Result(0)
        }
        5..=9 => LoadEnd::Wait,
        _ => {
            let result = mcseqGetResult();
            let index = result - 1;
            if (index as u32) < 8 {
                match index {
                    0 => {
                        let file = lw(0x3aff3c);
                        if memcardFileCheck(file, 0x21315354, 0xb790) == 0 {
                            corruptLoadMessage(a0);
                            return LoadEnd::Result(0);
                        }
                        let record = file + 4;
                        if signonFileCount(file + 20) == 0 {
                            return no_file_message(a0);
                        }
                        // Is the player of the file one that is there already?
                        let id = lw(record + 4);
                        let mut known = false;
                        let mut i = 0;
                        while i < signonCount() {
                            let other = lw((i << 3) + g::signoninfo + 4);
                            i += 1;
                            if other == id {
                                known = true;
                                break;
                            }
                        }
                        sw(0x3aff44, id);
                        signonAddFile(record, mcseqGetCurCard());
                        let message = if !known {
                            mcseqGetMsg(4)
                        } else {
                            let lang = lw((lw(g::cur_language) << 2) + g::languages);
                            let name = mcseqSlotGetLongName();
                            vafmt(&[0x3a19b0, lw(lang + 564) as i64, name as i64, text(568) as i64, text(572) as i64])
                        };
                        show_message(a0, 2, message, text(1612));
                        LoadEnd::Result(0)
                    }
                    1 => {
                        corruptLoadMessage(a0);
                        LoadEnd::Result(0)
                    }
                    2 | 5 => {
                        soundStart(142, 0);
                        let message = if index == 2 { mcseqGetMsg(3) } else { mcseqGetMsg(2) };
                        show_message(a0, 6, message, text(1616));
                        sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) | 4);
                        LoadEnd::Result(0)
                    }
                    3 => {
                        soundStart(142, 0);
                        let message = if mcseqGetCurCard() != -1 { mcseqGetMsg(1) } else { mcseqGetMsg(0) };
                        show_message(a0, 6, message, text(1612));
                        sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) | 4);
                        LoadEnd::Result(0)
                    }
                    4 => no_file_message(a0),
                    6 => cannot_load_message(a0),
                    _ => {
                        soundStart(142, 0);
                        LoadEnd::Result(1)
                    }
                }
            } else {
                cannot_load_message(a0)
            }
        }
    }
}

/// The tick of the load of a player from the memory card for the local player `a0`: it runs the sequencer, and reads the file when it is there. Gives 1 when it is over (2 if there are not the 2 cards it needs), or 0 while it goes on.
pub fn signonLoadTick(a0: i32) -> i32 {
    let result = match load_step(a0) {
        LoadEnd::Result(result) => result,
        LoadEnd::Wait => {
            sw(0x1fc1210, 180);
            0
        }
    };
    if result != 0 {
        close_window();
    }
    result
}

/// The message for the delete that is not in the cases of the result of the check (the others of the sequencer).
fn delete_message(a0: i32, message_id: i32) {
    show_message(a0, 2, mcseqGetMsg(message_id), text(1612));
    sb(0x3aff4b, 3);
}

/// Do the step of the delete that the state at 0x3aff4b says. See `signonDeleteTick`.
fn delete_step(a0: i32) -> i32 {
    let state = lbu(0x3aff4b);
    if (state as u32) >= 5 {
        return 0;
    }
    match state {
        0 => {
            mcseqSetCurCard(lw((lbu(0x3aff49) << 3) + g::signoninfo));
            let title = text(1900);
            let size = signonFileSize();
            mcseqLoad(10, title, 0, 0x3a1960, lw(0x3aff3c), size, 0);
            mcseqController(a0);
            sb(0x3aff4b, 1);
            0
        }
        1 => {
            if mcseqGetState() != 10 {
                return 0;
            }
            let result = mcseqGetResult();
            let index = result - 1;
            if (index as u32) < 8 {
                match index {
                    0 => {
                        let file = lw(0x3aff3c);
                        let player = lbu(0x3aff49);
                        if memcardFileCheck(file, 0x21315354, 0xb790) == 0 {
                            corruptLoadMessage(a0);
                            sb(0x3aff4b, 3);
                            return 0;
                        }
                        let record = lw(0x3aff3c);
                        if lw(record + 8) == lw((player << 3) + g::signoninfo + 4) {
                            let files = record + 20;
                            let found = signonFileGetName(files, lw(g::signonarray) + player * 2936);
                            if found >= 0 {
                                signonFileDelete(files, found);
                                if lw(record + 4) == found {
                                    sw(record + 4, -1);
                                }
                                memcardFileMakeCRC(lw(0x3aff3c), 0x21315354, 0xb790);
                                let title = text(1896);
                                let size = signonFileSize();
                                mcseqSave(11, title, 0, 0x3a1960, lw(0x3aff3c), size, 0);
                                mcseqController(a0);
                                sb(0x3aff4b, 2);
                                return 0;
                            }
                        }
                        let lang = lw((lw(g::cur_language) << 2) + g::languages);
                        let name = mcseqSlotGetLongName();
                        let message = vafmt(&[0x3a1b20, lw(lang + 644) as i64, lw(lang + 648) as i64, name as i64, text(652) as i64, text(656) as i64]);
                        show_message(a0, 2, message, text(1612));
                        sb(0x3aff4b, 3);
                    }
                    1 => {
                        corruptLoadMessage(a0);
                        sb(0x3aff4b, 3);
                    }
                    3..=6 => {
                        let lang = lw((lw(g::cur_language) << 2) + g::languages);
                        let name = mcseqSlotGetLongName();
                        let message = vafmt(&[0x3a19b0, lw(lang + 660) as i64, name as i64, text(664) as i64, text(668) as i64]);
                        show_message(a0, 2, message, text(1612));
                        sb(0x3aff4b, 3);
                    }
                    _ => delete_message(a0, 8),
                }
            } else {
                delete_message(a0, 8);
            }
            0
        }
        2 => {
            if mcseqGetState() != 11 {
                return 0;
            }
            if mcseqGetResult() == 1 {
                signonDelete(lbu(0x3aff49));
                show_message(a0, 2, mcseqGetMsg(9), text(1612));
                sb(0x3aff4b, 4);
                return 0;
            }
            delete_message(a0, 8);
            0
        }
        3 => {
            soundStart(142, 0);
            sb(0x3aff4b, 4);
            0
        }
        _ => {
            if mcseqGetState() != 2 {
                if mcseqGetState() != 3 {
                    return 0;
                }
                if lw(0x1fc11fc) & 8 == 0 {
                    return 2;
                }
                signonDeleteStart();
                return 0;
            }
            1
        }
    }
}

/// The tick of the delete of a player from the memory card for the local player `a0`. Gives 1 or 2 when it is over, or 0 while it goes on.
pub fn signonDeleteTick(a0: i32) -> i32 {
    let result = delete_step(a0);
    if result != 0 {
        close_window();
    }
    result
}

/// What `signonUpdateTick` does next: finish (the result is at `sp`), or finish without closing the window of the sequencer.
#[derive(PartialEq)]
enum UpdateEnd {
    Normal,
    NoClose,
}

/// The message the update gives when the sequencer says there is something wrong: the text 5 or 11 and the state 9.
fn update_warning(a0: i32, message_id: i32) -> UpdateEnd {
    show_message(a0, 2, mcseqGetMsg(message_id), text(1612));
    sb(0x3aff4b, 9);
    UpdateEnd::Normal
}

/// The message with the buttons `buttons` for the state 9 of the update, with the flag 4 of the window.
fn update_message(buttons: i32, message: i32) -> UpdateEnd {
    mcseqMessage(buttons, message, text(1612));
    sb(0x3aff4b, 9);
    sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) | 4);
    UpdateEnd::Normal
}

/// The save of the player in the slot `slot` of the card to the file: find a free place for the record, copy it there and start the write. `title` is the text for the dialog.
fn update_save(a0: i32, title: i32) -> UpdateEnd {
    let file = lw(0x3aff3c);
    let card = mcseqGetCurCard();
    let header = file + 4;
    let records = file + 20;
    signonCheckSlot(header, card);
    let slot = signonGet(a0);
    let free = signonFileGetFree(records);
    if free >= 0 {
        let found = signonFileGetName(records, lw(g::signonarray) + slot * 2936);
        let lang = lw((lw(g::cur_language) << 2) + g::languages);
        if found < 0 {
            copy_doublewords(free * 2936 + header + 16, slot * 2936 + lw(g::signonarray), 367);
            sw(header, free);
            memcardFileMakeCRC(lw(0x3aff3c), 0x21315354, 0xb790);
            let size = signonFileSize();
            mcseqSave(10, title, lw(lang + 1892), 0x3a1960, lw(0x3aff3c), size, 0);
            mcseqController(a0);
            sb(0x3aff4b, 4);
            return UpdateEnd::Normal;
        }
        let name = mcseqSlotGetLongName();
        let message = vafmt(&[
            0x3a1a28,
            lw(lang + 596) as i64,
            (lw(g::signonarray) + slot * 2936) as i64,
            lw(lang + 600) as i64,
            lw(lang + 604) as i64,
            name as i64,
        ]);
        return update_message(6, GetPlayerSignOnMessage(a0, message));
    }
    let lang = lw((lw(g::cur_language) << 2) + g::languages);
    let name = mcseqSlotGetLongName();
    let message = vafmt(&[0x3a1a10, lw(lang + 580) as i64, lw(lang + 584) as i64, lw(lang + 588) as i64, name as i64, text(592) as i64]);
    update_message(6, GetPlayerSignOnMessage(a0, message))
}

/// Do the step of the update that the state at 0x3aff4b says. `slot` is the player and `title` the title of the dialog. The result for the page is put at `sp`.
fn update_step(a0: i32, sp: i32, slot: i32, title: i32, state: i32) -> UpdateEnd {
    match state {
        0 => {
            let card = lw((slot << 3) + g::signoninfo);
            if card == 8 {
                sw(sp, 1);
                return UpdateEnd::NoClose;
            }
            let next;
            let flags;
            if card != -1 {
                mcseqSetCurCard(card);
                let size = signonFileSize();
                mcseqLoad(10, title, 0, 0x3a1960, lw(0x3aff3c), size, 0);
                mcseqController(a0);
                flags = lw(g::mcseq_flags);
                next = 6;
            } else {
                mcseqUnvalidateCards();
                let size = signonFileSize();
                mcseqLoad(10, title, 0, 0x3a1960, lw(0x3aff3c), size, 0);
                mcseqController(a0);
                mcseqAddOption(text(1928));
                flags = lw(g::mcseq_flags);
                next = 1;
            }
            sb(0x3aff4b, next);
            sw(g::mcseq_flags, flags | 32);
            UpdateEnd::Normal
        }
        1 => {
            if mcseqGetState() != 10 {
                return UpdateEnd::Normal;
            }
            let index = mcseqGetResult() - 1;
            if (index as u32) >= 9 {
                return update_warning(a0, 5);
            }
            match index {
                0 => {
                    if memcardFileCheck(lw(0x3aff3c), 0x21315354, 0xb790) == 0 {
                        return update_corrupt(a0);
                    }
                    sb(0x3aff4b, 3);
                    UpdateEnd::Normal
                }
                1 => update_corrupt(a0),
                2 => update_warning(a0, 5),
                3 => {
                    soundStart(142, 0);
                    let message = if mcseqGetCurCard() != -1 { mcseqGetMsg(1) } else { mcseqGetMsg(0) };
                    update_message(6, GetPlayerSignOnMessage(a0, message))
                }
                4 | 5 => {
                    sb(0x3aff4b, 2);
                    UpdateEnd::Normal
                }
                6 => update_warning(a0, 11),
                7 => {
                    sw(sp, 2);
                    UpdateEnd::Normal
                }
                _ => {
                    sw(sp, 1);
                    sw((slot << 3) + g::signoninfo, 8);
                    UpdateEnd::Normal
                }
            }
        }
        2 => {
            let file = lw(0x3aff3c);
            let id = signonMakeFileId();
            sw(file + 8, id);
            memset(file + 20, 0, 0xb780);
            update_save(a0, title)
        }
        3 => update_save(a0, title),
        4 => {
            if mcseqGetState() != 10 {
                return UpdateEnd::Normal;
            }
            let index = mcseqGetResult() - 1;
            if (index as u32) < 8 {
                match index {
                    0 => {
                        sb(0x3aff4b, 5);
                        return UpdateEnd::Normal;
                    }
                    6 => return update_warning_with_flag(a0, 11),
                    7 => {
                        sw(sp, 2);
                        return UpdateEnd::Normal;
                    }
                    _ => {}
                }
            }
            update_warning_with_flag(a0, 5)
        }
        5 => {
            let header = lw(0x3aff3c) + 4;
            let card = mcseqGetCurCard();
            let at = slot << 3;
            sw(at + g::signoninfo, card);
            let id = lw(header + 4);
            sb(0x3aff4b, 8);
            sw(g::signoninfo + at + 4, id);
            UpdateEnd::Normal
        }
        6 => {
            if mcseqGetState() != 10 {
                return UpdateEnd::Normal;
            }
            let index = mcseqGetResult() - 1;
            if (index as u32) >= 8 {
                return update_warning(a0, 5);
            }
            let lang_table = g::languages;
            let name_args;
            match index {
                0 => {
                    let file = lw(0x3aff3c);
                    if memcardFileCheck(file, 0x21315354, 0xb790) == 0 {
                        return update_corrupt(a0);
                    }
                    let header = file + 4;
                    if lw(header + 4) == lw((slot << 3) + g::signoninfo + 4) {
                        let found = signonFileGetName(file + 20, lw(g::signonarray) + slot * 2936);
                        if found >= 0 {
                            copy_doublewords(found * 2936 + header + 16, slot * 2936 + lw(g::signonarray), 367);
                            sw(header, found);
                            memcardFileMakeCRC(lw(0x3aff3c), 0x21315354, 0xb790);
                            let lang = lw((lw(g::cur_language) << 2) + g::languages);
                            let size = signonFileSize();
                            mcseqSave(11, title, lw(lang + 1892), 0x3a1960, lw(0x3aff3c), size, 0);
                            mcseqController(a0);
                            sb(0x3aff4b, 7);
                            return UpdateEnd::Normal;
                        }
                    }
                    let lang = lw((lw(g::cur_language) << 2) + lang_table);
                    let first = mcseqSlotGetLongName();
                    let lang_again = lw((lw(g::cur_language) << 2) + lang_table);
                    let player = lw(g::signonarray) + slot * 2936;
                    let second = mcseqSlotGetLongName();
                    name_args = [lw(lang + 608), first, lw(lang_again + 612), lw(lang_again + 616), player, lw(lang_again + 620), second];
                }
                1 => return update_corrupt(a0),
                2 | 7 => return update_warning(a0, 5),
                _ => {
                    let lang = lw((lw(g::cur_language) << 2) + lang_table);
                    let first = mcseqSlotGetLongName();
                    let lang_again = lw((lw(g::cur_language) << 2) + lang_table);
                    let player = lw(g::signonarray) + slot * 2936;
                    let second = mcseqSlotGetLongName();
                    name_args = [lw(lang + 624), first, lw(lang_again + 628), lw(lang_again + 632), player, lw(lang_again + 636), second];
                }
            }
            let message = vafmt(&[
                0x3a1a40,
                name_args[0] as i64,
                name_args[1] as i64,
                name_args[2] as i64,
                name_args[3] as i64,
                name_args[4] as i64,
                name_args[5] as i64,
                name_args[6] as i64,
            ]);
            mcseqMessage(6, GetPlayerSignOnMessage(a0, message), text(1616));
            sb(0x3aff4b, 9);
            sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) | 32);
            UpdateEnd::Normal
        }
        7 => {
            if mcseqGetState() != 11 {
                return UpdateEnd::Normal;
            }
            let index = mcseqGetResult() - 1;
            if (index as u32) >= 8 {
                return update_warning(a0, 5);
            }
            match index {
                0 => {
                    sb(0x3aff4b, 8);
                    UpdateEnd::Normal
                }
                1 | 2 | 3 | 4 | 5 | 7 => update_warning(a0, 5),
                _ => update_warning_with_flag(a0, 11),
            }
        }
        8 => {
            show_message(a0, 2, mcseqGetMsg(7), text(1612));
            sb(0x3aff4b, 10);
            UpdateEnd::Normal
        }
        9 => {
            soundStart(142, 0);
            sb(0x3aff4b, 10);
            UpdateEnd::Normal
        }
        _ => {
            if mcseqGetState() == 1 {
                let flags = lw(0x1fc11f8 + 4);
                if flags & 8 != 0 {
                    sb(0x3aff4b, 2);
                    sw(0x1fc11f8 + 4, flags & -9);
                }
            }
            if mcseqGetState() != 2 {
                if mcseqGetState() != 3 {
                    return UpdateEnd::Normal;
                }
                let flags = lw(0x1fc11fc);
                if flags & 12 == 0 {
                    sw(sp, 1);
                    sw(sp, if flags & 32 != 0 { 2 } else { 1 });
                    return UpdateEnd::Normal;
                }
            } else if lw(0x1fc11fc) & 36 == 0 {
                sw(sp, 1);
                return UpdateEnd::Normal;
            }
            signonUpdateStart();
            UpdateEnd::Normal
        }
    }
}

/// The message for a file that cannot be read (the cases 1 of the results): the message for a bad file, the buttons for ok, with the state 9 and the flag 8.
fn update_corrupt(a0: i32) -> UpdateEnd {
    show_message(a0, 5, mcseqGetMsg(10), text(1624));
    sb(0x3aff4b, 9);
    sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) | 8);
    UpdateEnd::Normal
}

/// The warning that goes with the message `message_id` and the flag 4 of the window (the last of the update).
fn update_warning_with_flag(a0: i32, message_id: i32) -> UpdateEnd {
    let message = GetPlayerSignOnMessage(a0, mcseqGetMsg(message_id));
    update_message(2, message)
}

/// The tick of the update of a player to the memory card for the local player `a0` (a number from 0). The result is 1 when it is over, 2 when the player is not saved because the card is not there, and 0 while it goes on. The window of the sequencer is closed when it is not 0.
pub fn signonUpdateTick(a0: i32) -> i32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    sw(sp, 0);
    let slot = signonGet(a0);
    let title = GetPlayerDialogTitle(a0, text(1920));
    let state = lbu(0x3aff4b);
    let end = if (state as u32) < 11 { update_step(a0, sp, slot, title, state) } else { UpdateEnd::Normal };
    if end == UpdateEnd::Normal {
        if lw(sp) == 0 {
            return lw(sp);
        }
        mcseqWindowClose();
    }
    sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) & -2);
    lw(sp)
}
