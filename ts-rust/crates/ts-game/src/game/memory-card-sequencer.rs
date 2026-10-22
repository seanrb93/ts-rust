//! Functions: the sequencer that the menus use to talk to the memory card, and its page.
//!
//! The sequencer has a state (`mcseq_state`) that `mcseqTick` moves along. The states are: 0 wait for the memory card to finish (the state to go to when it is good is at 0x3afffc, and when not at 0x3b0000, the result at 0x3afff8), 1 done, 2 failed, 3 a message is shown, 4 look for the cards, 5 choose a card, 6 and 7 and 8 check all the cards one after the other, 9 the card is chosen, 10 ask whether to format it, 11 the format is done, 12 to 17 the checks before a load or a save, 18 to 22 the write of a save, 23 and 24 check that the card is still there. The cards are 8 (4 slots of 2 ports); their state is at 0x1fc3710, the free space at 0x1fc3730 and the kind at 0x1fc3750.

use crate::*;

/// A text of the current language, from the table of texts at `offset`.
fn text(offset: i32) -> i32 {
    lw(lw((lw(g::cur_language) << 2) + g::languages) + offset)
}

/// Show a message with the two lines `a1` and `a2` (0 for none), and the buttons `a0` (flags). The state of the sequencer becomes 3.
pub fn mcseqMessage(a0: i32, a1: i32, a2: i32) {
    let first = 0x1fc3470;
    let second = 0x1fc3570;
    if a1 != 0 {
        strncpy(first, a1, 255);
    }
    sb(first + 255, 0);
    if a2 != 0 {
        strncpy(second, a2, 255);
    }
    sw(0x3affc4, first);
    sw(g::mcseq_msgflags, a0);
    sw(0x3affc8, second);
    sw(g::mcseq_state, 3);
    sb(second + 255, 0);
}

/// Start a load (`saving` 0) or a save (1) of the file named `name` with the sequencer, for the pad `pad`: `address` and `length` say where the data is (and `extra` is for the memory card). `title` and `second` are the texts of the first window (the second is the title if 0); with a title the window opens.
fn start_sequence(saving: i32, pad: i32, title: i32, second: i32, name: i32, address: i32, length: i32, extra: i32) {
    sw(0x3affe0, pad);
    sw(0x3affe4, saving);
    sw(0x3affd4, 0);
    sw(0x3affdc, 0);
    strcpy(0x1fc3670, name);
    let flags = lw(g::mcseq_flags);
    sw(0x3affe8, address);
    sw(0x3affec, length);
    sw(g::mcseq_flags, flags & 6);
    sw(0x3afff0, extra);
    sw(g::mcseq_state, 4);
    if title != 0 {
        sw(0x3affbc, title);
        let open = lbu(0x3affb8);
        sw(0x3affc0, if second != 0 { second } else { title });
        if open == 0 {
            mcseqWindowOpen();
            frontPageSet(lw(lw(g::front) + 20), g::mcseq_page);
        }
    }
}

/// Start loading the file named `a3` for the pad `a0` (see `start_sequence`).
pub fn mcseqLoad(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) {
    start_sequence(0, a0, a1, a2, a3, a4, a5, a6);
}

/// Start saving the file named `a3` for the pad `a0` (see `start_sequence`).
pub fn mcseqSave(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) {
    start_sequence(1, a0, a1, a2, a3, a4, a5, a6);
}

/// The text for a slot of the memory cards: the word for a memory card of the language, the number of the port from 1, and with a multitap the letter of the slot (A to D) after a dash. It is made at 0x1fc33d0.
pub fn mcseqSlotMakeName(a0: i32) -> i32 {
    let port = a0 >> 2;
    let buffer = 0x1fc33d0;
    let slots = joyMtapGetSlots(port);
    let word = text(1460);
    let length = strlen(word);
    sprintf(&[buffer as i64, 0x3ae8a0, text(1460) as i64]);
    sb(length + 1 + buffer, port + 49);
    if slots >= 2 {
        sb(length + 2 + buffer, 45);
        sb(length + buffer + 3, (a0 & 3) + 65);
        sb(length + buffer + 4, 0);
    } else {
        sb(length + 2 + buffer, 0);
    }
    buffer
}

/// The result of the sequencer, for the menus that call it: the pad in the states 1 and 2, in the state 3 the button chosen in the message (1 first, 2 second, 3 third, 4 fourth), -1 if it is busy, or what `mcseqTick` gives in the states 23 and 24.
pub fn mcseqGetState() -> i64 {
    let index = lw(g::mcseq_state) - 1;
    if (index as u32) >= 24 {
        return -1;
    }
    match index {
        0 | 1 => lw(0x3affe0) as i64,
        2 => {
            let flags = lw(g::mcseq_msgflags);
            if flags & 16 != 0 {
                1
            } else if flags & 32 != 0 {
                2
            } else if flags & 64 == 0 {
                ((flags as i64 as u64) >> 5) as i64 & 4_i64
            } else {
                3
            }
        }
        22 | 23 => mcseqTick() as i64,
        _ => -1,
    }
}

/// The text of the message `a0` of the sequencer (0 to 12; the others give the last text), with the name of the slot and the space needed put in where the text asks for them.
pub fn mcseqGetMsg(a0: i32) -> i32 {
    let frame = Frame::new(112);
    let sp = frame.sp();
    if (a0 as u32) >= 13 {
        return text(1608);
    }
    match a0 {
        0 => text(1496),
        1 => {
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            let name = mcseqSlotGetLongName();
            vafmt(&[0x3a2000, lw(lang + 1500) as i64, lw(lang + 1504) as i64, name as i64])
        }
        2 => {
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            let name = mcseqSlotGetLongName();
            vafmt(&[0x3a2010, lw(lang + 1508) as i64, name as i64, text(1512) as i64])
        }
        3 => text(1516),
        4 => text(1520),
        5 => text(1524),
        6 => {
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            vafmt(&[0x3ae8b0, lw(lang + 1528) as i64, lw(lang + 1532) as i64, lang as i64])
        }
        7 => text(1536),
        8 => text(1540),
        9 => text(1544),
        10 => {
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            let name = mcseqSlotGetLongName();
            let first = lw(lang + 1548) as i64;
            let second = lw(lang + 1552) as i64;
            vafmt(&[0x3a2020, first, second, name as i64, text(1556) as i64, text(1560) as i64])
        }
        11 => {
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            let name = mcseqSlotGetLongName();
            let lang_again = lw((lw(g::cur_language) << 2) + g::languages);
            let space = mcseqGetRequiredSpace();
            vafmt(&[
                0x3a2030,
                lw(lang + 1564) as i64,
                lw(lang + 1568) as i64,
                name as i64,
                lw(lang_again + 1572) as i64,
                space as i64,
                text(1576) as i64,
                text(1580) as i64,
            ])
        }
        _ => {
            let name = mcseqSlotGetLongName();
            let space = mcseqGetRequiredSpace();
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            // The ninth argument of the call is on the stack.
            sw(sp, lw(lang + 1604));
            vafmt(&[
                0x3a2048,
                lw(lang + 1584) as i64,
                lw(lang + 1588) as i64,
                name as i64,
                lw(lang + 1592) as i64,
                space as i64,
                lw(lang + 1596) as i64,
                lw(lang + 1600) as i64,
            ])
        }
    }
}

/// Wait for the memory card to finish what it was asked: go to the state `good` if it did it, and to `bad` if it failed.
fn wait_for_card(good: i32, bad: i32) {
    sw(0x3afffc, good);
    sw(0x3b0000, bad);
    sw(g::mcseq_state, 0);
}

/// Fail with the result `result` (the state 2).
fn fail(result: i32) {
    sw(g::mcseq_state, 2);
    sw(0x3afff8, result);
}

/// Go to the next of the 8 cards that has to be checked (the ones of the list at 0x1fc36f0, which has the slots there are 2 for), starting after the last one, and ask the memory card to check it. The state 8 waits for it.
fn check_next_card() {
    for i in 0..8 {
        sw(0x1fc36f0 + i * 4, 2);
    }
    for slot in joyMtapGetSlots(0)..4 {
        sw(0x1fc36f0 + slot * 4, 0);
    }
    for slot in (joyMtapGetSlots(1) + 4)..8 {
        sw(0x1fc36f0 + slot * 4, 0);
    }
    let mut index = lw(0x3b0004);
    let mut current = index;
    let mut tried = 0;
    let mut at;
    loop {
        at = index << 2;
        if tried >= 8 {
            break;
        }
        index += 1;
        current = index;
        if index >= 8 {
            index = 0;
            current = 0;
        }
        at = index << 2;
        if lw(0x1fc36f0 + (index << 2)) != 0 {
            break;
        }
        tried += 1;
    }
    sw(0x3b0004, current);
    let entry = at + 0x1fc36f0;
    if lw(entry) == 0 {
        return;
    }
    sw(entry, 1);
    mcardCheckCards(0x1fc36f0, 0x1fc3730, 0x1fc3750);
    sw(0x3afffc, 7);
    sw(g::mcseq_state, 8);
}

/// Move the sequencer along. It gives the result for the menu that waits for it: -1 while it is busy (and 0 to 8 and the others in the states of the checks of the card), see `mcseqGetState`.
pub fn mcseqTick() -> i32 {
    let frame = Frame::new(464);
    let sp = frame.sp();
    let state = lw(g::mcseq_state);
    if (state as u32) >= 25 {
        return -1;
    }
    match state {
        0 => {
            let result = mcardGetResult();
            sw(0x3afff8, result);
            if result == 0 {
                return -1;
            }
            if result == 1 {
                sw(g::mcseq_state, lw(0x3afffc));
                return -1;
            }
            sw(g::mcseq_state, lw(0x3b0000));
            return -1;
        }
        1 => return lw(0x3affe0),
        2 => {
            let pad = lw(0x3affe0);
            sw(g::mcseq_flags, lw(g::mcseq_flags) & -25);
            return pad;
        }
        3 => return 0,
        4 => {
            let chosen = lw(0x3b0008);
            if chosen != -1 && lw((chosen << 2) + 0x1fc3710) != 1 {
                sw(g::mcseq_state, 9);
                return -1;
            }
            for slot in joyMtapGetSlots(0)..4 {
                sw(0x1fc3710 + (slot << 2), 0);
            }
            for slot in (joyMtapGetSlots(1) + 4)..8 {
                sw(0x1fc3710 + (slot << 2), 0);
            }
            mcardCheckCards(0x1fc3710, 0x1fc3730, 0x1fc3750);
            let next = if lw(0x3b0008) != -1 { 9 } else { 5 };
            sw(g::mcseq_flags, lw(g::mcseq_flags) | 16);
            wait_for_card(next, 2);
            return -1;
        }
        5 => {
            let asked = lw(0x3affd4);
            let mut cards = 0;
            for i in 0..8 {
                if lw(0x1fc3710 + (i << 2)) >= 4 {
                    cards += 1;
                }
            }
            if asked != 0 || cards != 1 {
                sw(g::mcseq_state, 6);
                return -1;
            }
            let mut first = 0;
            if lw(0x1fc3710) < 4 {
                first = 1;
                let mut at = 0x1fc3710;
                loop {
                    at += 4;
                    if first >= 8 || lw(at) >= 4 {
                        break;
                    }
                    first += 1;
                }
            }
            sw(0x3b0008, first);
            sw(g::mcseq_state, 9);
            return -1;
        }
        6 => {
            sw(g::mcseq_flags, lw(g::mcseq_flags) & -17);
            for i in 0..8 {
                sw(0x1fc36f0 + (i << 2), (lw(0x1fc3710 + (i << 2)) >= 4) as i32);
            }
            sw(0x3b0004, -1);
            sw(g::mcseq_state, 7);
            sw(0x3affd0, 0);
            check_next_card();
            return -1;
        }
        7 => {
            check_next_card();
            return -1;
        }
        8 => {
            let result = mcardGetResult();
            if result == 0 {
                return -1;
            }
            if result == 1 {
                let at = lw(0x3b0004) << 2;
                sw(at + 0x1fc3710, lw(at + 0x1fc36f0));
            }
            let count = lw(0x3affd0);
            sw(0x3affd0, count + 1);
            sw(g::mcseq_state, lw(0x3afffc));
            return -1;
        }
        9 => {
            let chosen = lw(0x3b0008);
            mcardSelectCard(chosen >> 2, chosen & 3);
            if lw((lw(0x3b0008) << 2) + 0x1fc3710) != 5 {
                let flags = lw(g::mcseq_flags);
                sw(g::mcseq_flags, flags & -17);
                if flags & 6 == 0 {
                    sw(g::mcseq_state, 12);
                } else {
                    sw(g::mcseq_state, 16);
                }
                return -1;
            }
            if lw(0x3affe4) == 0 && lw(g::mcseq_flags) & 32 == 0 {
                fail(6);
                return -1;
            }
            sw(g::mcseq_state, 10);
            return -1;
        }
        10 => {
            let name = mcseqSlotGetLongName();
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            let again = lw((lw(g::cur_language) << 2) + g::languages);
            let message = vafmt(&[0x3a1eb0, lw(lang + 1464) as i64, name as i64, lw(again + 1468) as i64, lw(again + 1472) as i64]);
            strcpy(0x1fc33f0, message);
            let pad = lw(0x3affdc);
            sw(0x3affc4, 0x1fc33f0);
            sw(0x3affc8, text(1476));
            if joyNewBut(pad) & 32 == 0 {
                if joyNewBut(lw(0x3affdc)) & 16 == 0 {
                    return -1;
                }
                sw(g::mcseq_state, 6);
                return -1;
            }
            soundStart(140, 0);
            sw(g::mcseq_flags, lw(g::mcseq_flags) | 8);
            mcardFormat();
            sw((lw(0x3b0008) << 2) + 0x1fc3710, 1);
            wait_for_card(11, 2);
            return -1;
        }
        11 => {
            sw(g::mcseq_flags, lw(g::mcseq_flags) & -9 & -17);
            mcardCheckCards(0x1fc3710, 0x1fc3730, 0x1fc3750);
            wait_for_card(9, 2);
            return -1;
        }
        12 => {
            strcpy(sp, 0x1fc3670);
            strcpy(sp + 21, 0x1fc3670);
            sb(sp + 42, 0);
            mcardFileExists(sp);
            wait_for_card(16, 13);
            return -1;
        }
        13 => {
            if lw(0x3afff8) == 5 && lw(0x3affe4) != 0 {
                mcardGetEntSpace(0x3ae8a8, 0x3b000c);
                wait_for_card(14, 2);
                return -1;
            }
        }
        14 => {
            // A copy of the name of the file, of 21 bytes with a last 0.
            let name = 0x1fc3670;
            let middle = ld(name + 8);
            let end = lw(name + 16);
            sd(sp + 128, ld(name));
            sd(sp + 136, middle);
            sw(sp + 144, end);
            sb(sp + 148, lb(name + 20));
            sw(0x3b0010, 3);
            sb(sp + 149, 0);
            let is_signon;
            let size;
            if strncmp(sp + 128, 0x3a1ec0, 21) != 0 {
                is_signon = 1;
                let clusters = mcardIconClusters(1);
                sw(0x3b0010, lw(0x3b0010) + clusters);
                size = mmfileMaxSize();
            } else {
                is_signon = 0;
                let clusters = mcardIconClusters(0);
                sw(0x3b0010, lw(0x3b0010) + clusters);
                size = signonFileSize();
            }
            let total = lw(0x3b0010) + (((size + 1023) as u32) >> 10) as i32;
            sw(0x3b0010, total);
            if lw((lw(0x3b0008) << 2) + 0x1fc3730) < total {
                fail(7);
                return -1;
            }
            mcardMkdir(is_signon, 1, 1 - is_signon);
            wait_for_card(15, 2);
            return -1;
        }
        15 => {
            let at = (lw(0x3b0008) << 2) + 0x1fc3710;
            sw(g::mcseq_flags, lw(g::mcseq_flags) | 4);
            sw(at, 1);
            sw(g::mcseq_state, 4);
            return -1;
        }
        16 => {
            if lw(0x3affe4) != 0 {
                let name = 0x1fc3670;
                let middle = ld(name + 8);
                let end = lw(name + 16);
                sd(sp + 256, ld(name));
                sd(sp + 264, middle);
                sw(sp + 272, end);
                sb(sp + 276, lb(name + 20));
                sb(sp + 277, 0);
                let is_signon;
                let size;
                if strncmp(sp + 256, 0x3a1ec0, 21) != 0 {
                    is_signon = 1;
                    size = mmfileMaxSize() + 1023;
                } else {
                    is_signon = 0;
                    size = signonFileSize() + 1023;
                }
                sw(0x3b0010, ((size as u32) >> 10) as i32);
                mcardCheckDir(is_signon);
                wait_for_card(17, 2);
                return -1;
            }
            sw(g::mcseq_state, 17);
            return -1;
        }
        17 => {
            let saving = lw(0x3affe4);
            sw(g::mcseq_flags, lw(g::mcseq_flags) | 2);
            if saving == 0 {
                mcardFileRead(0x1fc3670, lw(0x3affe8), lw(0x3affec), lw(0x3afff0));
                wait_for_card(1, 2);
                return -1;
            }
            if saving == 1 {
                mcardFileExists(0x1fc3670);
                wait_for_card(20, 19);
                return -1;
            }
        }
        18 => return -1,
        19 => {
            sw(0x3afff4, 0);
            sw(g::mcseq_state, 21);
            return -1;
        }
        20 => {
            sw(0x3afff4, 1);
            sw(g::mcseq_state, 21);
            return -1;
        }
        21 => {
            mcardFileWrite(0x1fc3670, lw(0x3affe8), lw(0x3affec), lw(0x3afff4), lw(0x3afff0));
            wait_for_card(22, 2);
            return -1;
        }
        22 => {
            let at = lw(0x3b0008) << 2;
            sw(g::mcseq_state, 1);
            sw(at + 0x1fc3710, 1);
            return -1;
        }
        23 => {
            let count = lw(0x3affd0);
            let mut next = count + 1;
            if count == 60 {
                sw((lw(0x3b0008) << 2) + 0x1fc3710, 1);
                mcardCheckCards(0x1fc3710, 0, 0x1fc3750);
                sw(0x3affd0, 0);
                sw(g::mcseq_state, 24);
                next = 1;
            }
            sw(0x3affd0, next);
            return -1;
        }
        _ => {
            let result = mcardGetResult();
            sw(0x3afff8, result);
            if result == 0 {
                return -1;
            }
            if result == 1 {
                let at = lw(0x3b0008) << 2;
                sw(g::mcseq_state, 23);
                sw(0x3affcc, 1);
                let kind = lw(at + 0x1fc3750);
                if kind == 1 {
                    return 6;
                }
                if kind >= 2 {
                    if kind == 2 {
                        return 7;
                    }
                    if kind == 3 {
                        return 8;
                    }
                } else if kind == 0 {
                    return 5;
                }
            }
            sw(0x3affcc, 1);
            sw(g::mcseq_state, 23);
            return 3;
        }
    }
    sw(g::mcseq_state, 2);
    -1
}

/// Copy the record of 32 bytes at `from` to `to`.
fn copy_record(to: i32, from: i32) {
    let middle = ld(from + 8);
    let end = ld(from + 16);
    sd(to, ld(from));
    sd(to + 8, middle);
    sd(to + 16, end);
    sd(to + 24, ld(from + 24));
}

/// The menu of the message, the page for the states with nothing special: a title, a line and one or two buttons. Gives the end of the items.
fn message_menu(sp: i32, items: i32) -> i32 {
    let records = 0x1fc3770;
    let zoom = g::zoomboxes;
    sh(sp, 0);
    sh(sp + 2, lhu(zoom + 24) + 10);
    copy_record(records, g::mi_msg);
    let flags = lw(g::mcseq_flags);
    let line = if flags & 16 == 0 {
        if flags & 8 != 0 { text(1904) } else { lw(0x3affc0) }
    } else {
        lw(0x3affbc)
    };
    sw(records, line);
    sh(records + 10, lhu(records + 10) | 256);
    std_menuitemmake(items, sp, records);
    let second = records + 32;
    sh(sp + 2, lhu(zoom + 24) + 45);
    copy_record(second, g::mi_msg);
    sw(second, text(1492));
    std_menuitemmake(items + 24, sp, second);
    let third = second + 32;
    copy_record(third, g::mi_msg);
    sw(third, text(1912));
    sh(sp + 2, lhu(zoom + 24) + lhu(zoom + 32) - 30);
    std_menuitemmake(items + 48, sp, third);
    items + 72
}

/// The menu to choose a card: the title, the line with the choice, the cards that were found (or the names of the other files), and the button to go back. `page` is the page with the cursor (+4) and the first item (+8). Gives the end of the items.
fn card_menu(page: i32, sp: i32, items: i32) -> i32 {
    let records = 0x1fc3770;
    let zoom = g::zoomboxes;
    let cursor = lhu(page + 4);
    let extra = lw(0x3affd4);
    let title = lw(0x3affbc);
    // The cards that are there, kept as a list at sp+16.
    let mut found = 0;
    for slot in 0..8 {
        if lw(0x1fc3710 + (slot << 2)) >= 4 {
            sw(sp + 16 + (found << 2), slot);
            found += 1;
        }
    }
    let total = found + extra;
    let mut small = (total < 4) as i32;
    if ((cursor - 1) as u32) >= (total as u32) {
        sh(page + 4, 1);
        small = (total < 4) as i32;
    }
    if small != 0 {
        sh(sp, 0);
    } else {
        let y = lw(zoom + 24) - (((total - 3) * 10) >> 1);
        sw(zoom + 32, lw(zoom + 32) + (total - 3) * 10);
        sw(zoom + 24, y);
        sh(sp, 0);
    }
    sh(sp + 2, lhu(zoom + 24) + 10);
    copy_record(records, g::mi_msg);
    sw(records, title);
    sh(records + 10, lhu(records + 10) | 256);
    std_menuitemmake(items, sp, records);
    let next_y = lhu(sp + 2) + 15;
    sh(sp + 2, next_y);
    if total == 0 {
        sh(sp + 2, next_y + 20);
    }
    let mut record = records + 32;
    copy_record(record, g::mi_msg);
    let line = if found == 0 {
        if lw(0x3affd4) == 0 { text(1908) } else { text(1880) }
    } else {
        text(1884)
    };
    sw(record, line);
    std_menuitemmake(items + 24, sp, record);
    let mut item = items + 48;
    sh(sp + 2, lhu(sp + 2) + 20);
    let mut i = 0;
    while i < total {
        record += 32;
        copy_record(record, g::mi_card);
        if i >= found {
            sw(record, lw(((i - found) << 2) + 0x3affd8));
        } else {
            let name = (i << 5) + 0x1fc38f0;
            strcpy(name, mcseqSlotMakeName(lw(sp + (i << 2) + 16)));
            sw(record, name);
        }
        i += 1;
        sh(record + 8, i);
        std_menuitemmake(item, sp, record);
        item += 24;
        sh(sp + 2, lhu(sp + 2) + 10);
    }
    let last = record + 32;
    sh(sp + 2, lhu(zoom + 24) + lhu(zoom + 32) - 30);
    copy_record(last, g::mi_msg);
    sw(last, if total == 0 { text(1484) } else { text(1480) });
    std_menuitemmake(item, sp, last);
    let end = item + 24;

    // The buttons: up and down move the cursor, one button chooses and another goes back.
    let moved = 'moved: {
        if joyNewBut(lw(0x3affdc)) & 0x1000 != 0 {
            let at = lhu(page + 4);
            if (at as u32) >= 2 {
                break 'moved Some(at - 1);
            }
        }
        if joyNewBut(lw(0x3affdc)) & 0x4000 != 0 {
            let at = lhu(page + 4);
            if at < total {
                break 'moved Some(at + 1);
            }
        }
        None
    };
    if let Some(cursor) = moved {
        sh(page + 4, cursor);
        soundStart(141, 0);
        return end;
    }
    if joyNewBut(lw(0x3affdc)) & 64 == 0 {
        let back = joyNewBut(lw(0x3affdc));
        let language = lw(g::cur_language);
        if back & 16 != 0 {
            sw(g::mcseq_state, 2);
            sw(0x3afff8, 8);
            sw(0x3affc4, lw(lw((language << 2) + g::languages) + 1488));
        }
        return end;
    }
    let mut extra = lw(0x3affd4);
    if found != 0 {
        if ((lhu(page + 4) - 1) as u32) < (found as u32) {
            soundStart(140, 0);
            sw(0x3afffc, 9);
            sw(g::mcseq_state, 8);
            sw(0x3b0008, lw(sp + ((lhu(page + 4) - 1) << 2) + 16));
            return end;
        }
        extra = lw(0x3affd4);
    }
    if extra <= 0 {
        return end;
    }
    if ((lhu(page + 4) - (found + 1)) as u32) < (extra as u32) {
        soundStart(140, 0);
        sw(0x3b0008, -1);
        sw(g::mcseq_state, 1);
        sw(0x3afff8, 9);
    }
    end
}

/// The tick of the page of the sequencer: it moves the sequencer, and makes the items of the menu for its state: the buttons of the message in the state 3 and 10, nothing in the state 6, the choice of a card in the states 7 and 8, and a title with a line and buttons in the others. `a0` is the page. The page gets the number of items at +12.
pub fn mcseq_pageTick(a0: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let start = lw(a0 + 8);
    let mut items = start;
    mcseqTick();
    mcseqWindowSet();
    let state = lw(g::mcseq_state);
    let mut cursor: Option<i32> = Some(666);
    if (state as u32) < 11 {
        match state {
            0 | 1 | 2 | 4 | 5 | 9 => {
                items = message_menu(sp, items);
            }
            3 | 10 => {
                // The buttons that were pressed are noted in the flags of the message.
                let mut flags = lw(g::mcseq_msgflags);
                if flags & 1 != 0 {
                    let pressed = joyNewBut(lw(0x3affdc));
                    flags = lw(g::mcseq_msgflags);
                    if pressed & 32 != 0 {
                        flags |= 16;
                        sw(g::mcseq_msgflags, flags);
                    }
                }
                if flags & 2 != 0 {
                    let pressed = joyNewBut(lw(0x3affdc));
                    flags = lw(g::mcseq_msgflags);
                    if pressed & 64 != 0 {
                        flags |= 32;
                        sw(g::mcseq_msgflags, flags);
                    }
                }
                if flags & 4 != 0 {
                    let pressed = joyNewBut(lw(0x3affdc));
                    flags = lw(g::mcseq_msgflags);
                    if pressed & 16 != 0 {
                        flags |= 64;
                        sw(g::mcseq_msgflags, flags);
                    }
                }
                if flags & 8 != 0 {
                    let pressed = joyNewBut(lw(0x3affdc));
                    let now = lw(g::mcseq_msgflags);
                    if pressed & 128 != 0 {
                        sw(g::mcseq_msgflags, now | 128);
                    }
                }
                joyClearAll(lw(0x3affdc));
                let line = lw(0x3affc4);
                if line != 0 {
                    // The box is made wider if the text does not fit.
                    let wanted = textWidth(line) + 8;
                    let boxes = g::zoomboxes;
                    let width = lw(boxes + 28);
                    if width < wanted {
                        let more = wanted - width;
                        if wanted < lw(g::window + 12) {
                            let left = lw(boxes + 20);
                            sw(boxes + 28, wanted);
                            sw(boxes + 20, left - ((more + ((more as u32) >> 31) as i32) >> 1));
                        }
                    }
                }
                cursor = None;
            }
            6 => {
                cursor = Some(1);
                items = start;
            }
            _ => {
                // The states 7 and 8.
                items = card_menu(a0, sp, items);
                cursor = None;
            }
        }
    } else {
        items = message_menu(sp, items);
    }
    if let Some(cursor) = cursor {
        sh(a0 + 4, cursor);
    }
    sw(a0 + 12, (items - lw(a0 + 8)).wrapping_mul(-0x55555555) >> 3);
}
