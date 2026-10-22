//! Functions: the page of the player list of the sign on screen and the page of the screen itself (see `sign-on-pages.rs` for the state of the players).

use crate::*;

/// A text of the current language, from the table of texts at `offset`.
fn text(offset: i32) -> i32 {
    lw(lw((lw(g::cur_language) << 2) + g::languages) + offset)
}

/// The page of the player list: do what the buttons of the player ask. Gives the menu that has to be ticked (0 for none), or `None` when the page is done for this frame.
fn player_step(a0: i32) -> Option<i32> {
    let front = lw(g::front);
    let player = lw(front);
    let menu = g::signonplr_menu;
    let state = lbu(player + 0x3aff28) - 1;
    if (state as u32) >= 5 {
        return Some(0);
    }
    match state {
        0 => {
            signonSet(player, -1);
            sh(a0 + 4, lhu((player << 1) + 0x3aff88));
            sb(player + 0x3aff28, 2);
            None
        }
        1 => {
            let memo_at = (player << 1) + 0x3aff30;
            let memo = lhu(memo_at);
            if memo != 0 {
                sh(a0 + 4, memo);
                sh(memo_at, 0);
            }
            let mut cursor = lhu(a0 + 4);
            if (cursor as u32) < 2000 {
                if signonCount() < (cursor & 0xffff) {
                    sh(a0 + 4, 2000);
                    cursor = 2000;
                } else {
                    cursor = lhu(a0 + 4);
                }
            }
            sh((player << 1) + 0x3aff88, cursor);
            let item = menuitem(menu, lhu(a0 + 4));
            let mut updating = lbu(0x3aff4a);
            if item != 0 {
                updating = lbu(0x3aff4a);
                if lhu(item + 10) & 64 == 0 {
                    let at = lhu(a0 + 4) - 1;
                    let flags_at = (player << 2) + 0x1fc1200;
                    sw(flags_at, lw(flags_at) | 2);
                    let count = signonCount();
                    if (at as u32) < (count as u32) {
                        sw(flags_at, lw(flags_at) | 8);
                    }
                    updating = lbu(0x3aff4a);
                }
            }
            if updating != 0 || lbu(0x3aff40) != 0 || lbu(0x3aff48) != 0 || lbu(0x3aff94) != 0 {
                return Some(menu);
            }
            if joyNewBut(player) & 16 == 0 {
                let mut sound = 142;
                let mut marked = false;
                if joyNewBut(player) & 128 != 0 {
                    let free = lbu(0x3aff48) == 0;
                    if lw(0x1fc11f8 + (player << 2) + 8) & 8 != 0 && free {
                        // Ask to delete the player that is under the cursor.
                        let flags = lw(0x1fc11f8 + 4);
                        let cursor = lhu(a0 + 4);
                        sb(0x3aff48, player + 1);
                        let found = lbu(cursor + 0x1fc330f) + 1;
                        sw(0x1fc11f8 + 4, flags & -3);
                        sb(player + 0x3aff90, found);
                        sound = 140;
                        marked = true;
                    }
                }
                if !marked {
                    if joyNewBut(player) & 64 == 0 {
                        return Some(menu);
                    }
                    sound = 142;
                    if lw(0x1fc11f8 + (player << 2) + 8) & 2 != 0 {
                        let at = lhu(a0 + 4) - 1;
                        if (at as u32) < (signonCount() as u32) {
                            sh(item + 10, lhu(item + 10) | 64);
                            soundStart(140, 0);
                            signonSet(player, lbu(lhu(a0 + 4) + 0x1fc330f));
                            sb(player + 0x3aff28, 5);
                            return Some(g::signon_ready_waiting);
                        }
                        let now = lhu(a0 + 4);
                        if now != 2000 {
                            if now != 2001 {
                                return Some(menu);
                            }
                            if lbu(0x3aff40) != 0 {
                                return Some(menu);
                            }
                            sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) & -3);
                            sb(0x3aff40, player + 1);
                            soundStart(141, 0);
                            return Some(0);
                        }
                        if signonCount() < 144 {
                            soundStart(141, 0);
                            sb(player * 30 + 0x1fc3298, 0);
                            sb(player + 0x3aff28, 3);
                            return None;
                        }
                        sound = 142;
                    }
                }
                soundStart(sound, 0);
                Some(menu)
            } else {
                joyClearButs(player, 0xffff);
                if player == 0 {
                    // The first player can leave if all the players are ready (or not there).
                    let count = lw(0x3aff20);
                    let mut ready = 0;
                    if count > 0 && lw(0x1fc1488) == 0 {
                        let first = lbu(0x3aff28);
                        if first == 2 || first == 5 {
                            ready = 1;
                            while ready < count && lw(ready * 260 + 0x1fc1488) == 0 {
                                let other = lbu(ready + 0x3aff28);
                                if other != 2 && other != 5 {
                                    break;
                                }
                                ready += 1;
                            }
                        }
                    }
                    if ready == count {
                        pageOut();
                        return Some(0);
                    }
                }
                soundStart(142, 0);
                Some(0)
            }
        }
        2 => {
            stringeditStart(player, player * 30 + 0x1fc3298, 30, 1, 174, 0, 0);
            sb(lw(lw(g::front)) + 0x3aff28, 4);
            Some(0)
        }
        3 => {
            if lbu(0x3aff48) == 0 && lbu(0x3aff4a) == 0 && lbu(0x3aff40) == 0 && lbu(0x3aff94) == 0 {
                stringeditTick(player);
                let box_at = lw(g::textbox);
                if lw(box_at + 32) == 0 {
                    if lw(box_at + 36) != 0 {
                        sb(lw(lw(g::front)) + 0x3aff28, 2);
                        sh(a0 + 4, 2000);
                    }
                } else {
                    let id = signonNew(lw(lw(g::front)) * 30 + 0x1fc3298);
                    signonSet(lw(lw(g::front)), id);
                    let front_now = lw(g::front);
                    sb(0x3aff4a, lbu(front_now) + 1);
                    sb(lw(front_now) + 0x3aff28, 2);
                }
            }
            Some(0)
        }
        _ => {
            sh(a0 + 4, 0);
            let mut i = 0;
            while i < signonCount() {
                let wanted = lbu(i + 0x1fc3310);
                let current = signonGet(player);
                i += 1;
                if wanted == current {
                    sh((player << 1) + 0x3aff88, i);
                }
            }
            if joyNewBut(player) & 16 != 0 {
                sh(a0 + 4, lhu((player << 1) + 0x3aff88));
                signonSet(player, -1);
                sb(player + 0x3aff28, 2);
                joyClearButs(player, 16);
                soundStart(142, 0);
            }
            Some(g::signon_ready_waiting)
        }
    }
}

/// The tick of the page of the player list: it handles the buttons for the player of the screen (`player_step`) and ticks the menu.
pub fn signonplr_pageTick(a0: i32) {
    let front = lw(g::front);
    sw((lw(front) << 2) + 0x1fc1200, 0);
    if lbu(0x3aff4a) != 0 || lbu(0x3aff40) != 0 || lbu(0x3aff48) != 0 || lbu(0x3aff94) != 0 {
        let flags = lw(front + 12);
        let pad = lw(front);
        sw(front + 12, flags & -61);
        joyClearAll(pad);
    }
    let Some(menu) = player_step(a0) else {
        return;
    };
    if menu == 0 {
        return;
    }
    let front = lw(g::front);
    if lbu(lw(front) + 0x3aff28) == 2 {
        let list = g::signonplr_list;
        let rows = if lw(g::window + 92) > 0 { 5 } else { 8 };
        let first = lhu(list);
        sh(list + 2, rows);
        if (first as u32) < (rows as u32) {
            sh(list + 2, first);
        }
    }
    let players = lw(0x3aff20);
    sh(g::signon_ready_waiting + 6, if players == 2 { 140 } else { 80 });
    sh(g::signonplr_list + 6, if players == 2 { 130 } else { 40 });
    menutick_230238(a0, menu);
    let added = std_menumake(lw(a0 + 8) + lw(a0 + 12) * 24, menu);
    sw(a0 + 12, lw(a0 + 12) + added);
}

/// Where the idle logic of the page went on to: the clock at 0x3aff9c has to be cleared (`Cleared`), or not (`Kept`).
#[derive(PartialEq)]
enum Idle {
    Cleared,
    Kept,
}

/// The player `deleting - 1` is deleting a player: wait for the dialog, ask the sequencer to delete, and tell what happened.
fn deleting_player(deleting: i32) -> Idle {
    let player = deleting - 1;
    let target_at = player + 0x3aff90;
    let target = lbu(target_at) - 1;
    if lw(0x3aff9c) != 0 {
        let flags = lw(0x1fc11fc);
        if flags & 2 != 0 {
            if flags & 1 != 0 {
                if signonDeleteTick(player) == 0 {
                    return Idle::Kept;
                }
                sb(target_at, 0);
                sb(0x3aff48, 0);
                return Idle::Cleared;
            }
            let result = splitMessageTick(player);
            if result != 2 {
                if result == 3 {
                    sb(target_at, 0);
                    sb(0x3aff48, 0);
                    sw(0x3aff9c, 0);
                    soundStart(142, 0);
                }
            } else {
                let in_memory = (lw((target << 3) + g::signoninfo) as u32) < 8;
                sh((player << 1) + 0x3aff30, 2000);
                if !in_memory {
                    signonDelete(target);
                    sb(target_at, 0);
                    sb(0x3aff48, 0);
                    sw(0x3aff9c, 0);
                } else {
                    sb(0x3aff49, target);
                    signonDeleteStart();
                }
                soundStart(140, 0);
            }
            if result != 0 {
                splitMessageClear(player);
            }
            joyClearAll(player);
            return Idle::Kept;
        }
        let card = lw((target << 3) + g::signoninfo);
        let message;
        if (card as u32) >= 8 {
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            let record = lw(g::signonarray) + target * 2936;
            message = vafmt(&[0x3a1e40, lw(lang + 1364) as i64, record as i64, lw(lang + 1368) as i64, lw(lang + 1372) as i64]);
        } else {
            let lang = lw((lw(g::cur_language) << 2) + g::languages);
            let record = lw(g::signonarray) + target * 2936;
            let name = mcseqSlotMakeName(card);
            message = vafmt(&[
                0x3a1e28,
                lw(lang + 1344) as i64,
                record as i64,
                lw(lang + 1348) as i64,
                name as i64,
                text(1352) as i64,
                text(1356) as i64,
            ]);
        }
        splitMessageStart(player, 1, GetPlayerSignOnMessage(player, message));
        let flags = lw(0x1fc11f8 + 4);
        sw(0x3aff98, -1);
        sw(0x1fc11f8 + 4, flags | 2);
        return Idle::Kept;
    }
    soZoomStart(player);
    Idle::Kept
}

/// A player that is new has the same name as one that is there, or there is no room for more: make the message for it (0 if there is nothing to say).
fn same_name_message(player: i32, slot: i32) -> i32 {
    let in_memory = signonMemCount();
    let lang = lw(g::cur_language);
    if in_memory < 16 {
        let count = signonCount();
        let name_at = player * 30 + 0x1fc3298;
        let mut other = 0;
        let mut found;
        loop {
            found = other < count;
            if !found {
                break;
            }
            if other == slot {
                other += 1;
            } else if strcmp(lw(g::signonarray) + other * 2936, name_at) != 0 {
                other += 1;
            } else if lw((other << 3) + g::signoninfo) == 8 {
                break;
            } else {
                other += 1;
            }
        }
        if found {
            let lang_at = lw((lw(g::cur_language) << 2) + g::languages);
            return vafmt(&[0x3a1e18, lw(lang_at + 1328) as i64, name_at as i64, lw(lang_at + 1332) as i64, lw(lang_at + 1336) as i64]);
        }
        0
    } else {
        let lang_at = lw((lang << 2) + g::languages);
        vafmt(&[
            0x3a1e08,
            lw(lang_at + 1312) as i64,
            lw(lang_at + 1316) as i64,
            (player * 30 + 0x1fc3298) as i64,
            lw(lang_at + 1320) as i64,
            lw(lang_at + 1324) as i64,
        ])
    }
}

/// The player `updating - 1` is saving: the zoom of the window first, then the ticks of the update and what to do with the result.
fn updating_player(updating: i32, zoomed: i32) -> Idle {
    let player = updating - 1;
    if zoomed == 0 {
        soZoomStart(player);
        return Idle::Kept;
    }
    if lw(0x1fc11fc) & 1 == 0 {
        signonUpdateStart();
        return Idle::Kept;
    }
    let result = signonUpdateTick(player);
    if result == 0 {
        if lw(0x3aff98) != 0 {
            return Idle::Kept;
        }
        sw(0x3aff98, -1);
        return Idle::Kept;
    }
    let slot = signonGet(player);
    if result != 2 {
        let card = lw((slot << 3) + g::signoninfo);
        if card == -1 {
            signonDelete(slot);
            signonSet(player, -1);
            sh((player << 1) + 0x3aff30, 2000);
            soundStart(142, 0);
            sb(player + 0x3aff28, 2);
        } else {
            if card == 8 {
                let message = same_name_message(player, slot);
                if message != 0 {
                    signonDelete(slot);
                    signonSet(player, -1);
                    sh((player << 1) + 0x3aff30, 2000);
                    soundStart(142, 0);
                    sb(player + 0x3aff28, 2);
                    splitMessageStart(player, 2, message);
                    let flags = lw(0x1fc11f8 + 4);
                    sb(0x3aff94, player + 1);
                    sw(0x1fc11f8 + 4, flags | 2);
                }
            }
            if signonGet(player) >= 0 {
                sb(player + 0x3aff28, 5);
                soundStart(140, 0);
                sb(0x3aff4a, 0);
                return Idle::Cleared;
            }
        }
    } else {
        signonDelete(slot);
        signonSet(player, -1);
        sh((player << 1) + 0x3aff30, 2000);
        soundStart(142, 0);
        sb(player + 0x3aff28, 3);
    }
    sb(0x3aff4a, 0);
    Idle::Cleared
}

/// The player `loading - 1` is loading a player from a card: the zoom of the window first, then the ticks of the load, and what to do when it is over.
fn loading_player(loading: i32, zoomed: i32) -> Idle {
    let player = loading - 1;
    if zoomed == 0 {
        soZoomStart(player);
        return Idle::Kept;
    }
    if lw(0x1fc11fc) & 1 == 0 {
        signonLoadStart(player);
        sw(0x3aff98, -1);
        return Idle::Kept;
    }
    let done = signonLoadTick(player);
    let id = lw(0x3aff44);
    if done == 0 {
        return Idle::Kept;
    }
    let count = lw(0x3aff20);
    if id == 0 {
        soundStart(142, 0);
    } else {
        // The players that have no player any more and were ready have to choose again.
        if count > 0 {
            let mut i = 0;
            loop {
                if signonGet(i) == -1 && lbu(i + 0x3aff28) == 5 {
                    sb(i + 0x3aff28, 2);
                    sh((i << 1) + 0x3aff30, 2000);
                }
                i += 1;
                if i >= lw(0x3aff20) {
                    break;
                }
            }
        }
        // Put the cursor on the player that was loaded.
        let mut at = g::signoninfo + 4;
        let mut index = 0;
        let mut found = false;
        loop {
            let total = signonCount();
            let wanted = lw(0x3aff44);
            if index >= total {
                break;
            }
            let other = lw(at);
            at += 8;
            if other == wanted {
                found = true;
                break;
            }
            index += 1;
        }
        if found {
            soundStart(140, 0);
            sh((player << 1) + 0x3aff30, index + 1);
        }
    }
    sb(0x3aff40, 0);
    Idle::Cleared
}

/// The logic of the page when nothing is waiting: the player that is loading, updating or deleting gets its turn.
fn idle_players() -> Idle {
    let loading = lbu(0x3aff40);
    let zoomed = lw(0x3aff9c);
    if loading == 0 {
        let updating = lbu(0x3aff4a);
        if updating == 0 {
            let deleting = lbu(0x3aff48);
            if deleting == 0 {
                return Idle::Kept;
            }
            return deleting_player(deleting);
        }
        return updating_player(updating, zoomed);
    }
    loading_player(loading, zoomed)
}

/// The tick of the page of the sign on screen: the players go along their states, and when all of them are ready the settings are made and the page of the type of the game opens. `a0` is the page.
pub fn signon_pageTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    sw(sp, a0);
    pageSet(text(472), g::signon_page);
    frontfxTick(0);
    let wait = lw(0x3aff98);
    if lbu(lw(lw(g::front)) + 0x3aff28) != 1 {
        if wait <= 0 {
            let split = lbu(0x3aff94);
            if split == 0 {
                if idle_players() == Idle::Cleared {
                    sw(0x3aff9c, 0);
                }
            } else {
                let player = split - 1;
                let result = splitMessageTick(player);
                if ((result - 2) as u32) < 2 {
                    sb(0x3aff94, 0);
                    sw(0x3aff9c, 0);
                    soundStart(140, 0);
                    splitMessageClear(player);
                    sb(player + 0x3aff28, 3);
                }
                joyClearAll(player);
            }
        } else {
            sw(0x3aff98, wait - 1);
        }
    } else {
        splitpageInit(g::signonplr_page);
        sb(0x3aff40, 0);
        sw(0x3aff98, -2);
        sb(0x3aff48, 0);
        sb(0x3aff4a, 0);
        sb(0x3aff94, 0);
        sw(0x3aff9c, 0);
        memset(0x3aff90, 0, 4);
    }
    makesignonmenu();
    splitpageTick(lw(sp), g::signonplr_page);
    let players = lw(0x3aff20);
    let mut ready = 0;
    if players > 0 && lbu(0x3aff28) == 5 {
        ready = 1;
        while ready < players && lbu(ready + 0x3aff28) == 5 {
            ready += 1;
        }
    }
    if ready != players {
        sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) & -513);
        return;
    }
    settingsToMenus();
    statsCombine(lw(0x3aff20));
    let window = 0x1fc11f8;
    sw(window + 44, 0);
    sw(window + 32, 0);
    sw(window + 36, 0);
    if lw(0x3aff20) > 0 {
        let mut i = 0;
        loop {
            let record = lw(g::signonarray) + signonGet(i) * 2936 + 32;
            if chrPlayable(lw(record + 84)) == 0 {
                sw(record + 84, 0);
            }
            i += 1;
            if i >= lw(0x3aff20) {
                break;
            }
        }
    }
    sw(window + 4, lw(window + 4) | 512);
    for i in 0..4 {
        let front = frontGet(i);
        sw(lw(front + 4) + 12, 0);
    }
    checkUnavailableThings();
    frontPageSet(0, g::gametype_page);
    pageIn(lw(sp));
}
