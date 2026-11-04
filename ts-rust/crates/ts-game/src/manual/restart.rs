//! Playing a match again without going back to the menus: the End Game page of the menu of the game has a second entry, "Restart match", and the results page at the end of a match
//! restarts it with the cross button (the game does this itself for the game modes 4 and 6, `tickRetryQuit`).  The level starts again with the settings it had (`gameLevelRestart`).

use crate::*;

const BASE: i32 = 0x240_0000;
const MENU_AT: i32 = BASE + 0x17_3b00;
const TEXT_AT: i32 = BASE + 0x17_3a80;

/// Can the match be played again from here: a game of the arcade modes on this machine (not one on a network, and not the story)?
pub fn restart_available() -> bool {
    (1..=6).contains(&lw(g::curgameparms + 72)) && ilinkIsILinkGame() == 0 && lw(g::lvnumplayers) > 0
}

/// The menu of the End Game page: the game's one entry and, below it, "Restart match".
pub fn restart_menu(own: i32) -> i32 {
    if !restart_available() {
        return own;
    }
    for i in 0..4 {
        sw(MENU_AT + 4 * i, lw(own + 4 * i));
    }
    sw(MENU_AT, 2);
    let item = |k: i32| MENU_AT + 16 + 32 * k;
    for j in 0..8 {
        sw(item(0) + 4 * j, lw(own + 16 + 4 * j));
        sw(item(1) + 4 * j, lw(own + 16 + 4 * j));
    }
    let mut at = TEXT_AT;
    let text = crate::manual::avmenu::put_str(&mut at, "Restart match");
    sw(item(1), text);
    sh(item(1) + 6, lh(item(0) + 6) + 20);
    sh(item(1) + 8, 2);
    // (the cursor goes down from the first to the second, and up again)
    sw(item(0) + 16, 2 << 16);
    sw(item(1) + 16, 1);
    item(0) - 16
}

/// The End Game page's tick: the button 64 on "Restart match" starts the match again.  True when it did.
pub fn restart_page_pressed(page: i32, player: i32) -> bool {
    if !restart_available() {
        return false;
    }
    if lhu(page + 4) == 0 {
        sh(page + 4, 1);
    }
    if joyNewBut(player) & 64 != 0 && lhu(page + 4) == 2 {
        joyClearAll(player);
        restart_now();
        return true;
    }
    false
}

/// Start the match again: the menu of the game closes and the level is loaded as it was.
pub fn restart_now() {
    let players = lw(g::lvnumplayers);
    for p in 0..players {
        let menu = p * 92 + IGMENU;
        sh(menu + 8, 0);
    }
    gameLevelRestart();
}

/// The results page (the match is over): the game's retry or quit, as the story's results have it: the cross button plays the match again, the triangle button quits.
pub fn restart_results_tick() {
    if lw(g::gamestate) == 4 && restart_available() && lw(lw(g::currentplayer)) == 0 {
        tickRetryQuit();
    }
}

/// The two lines at the bottom of the results that say so: the game's texts for Quit and Restart (a button and a word each), drawn the same size and a line apart (`drawRetryQuit` draws the second a little bigger).
pub fn restart_results_hint() {
    if lw(g::gamestate) != 4 || !restart_available() || lw(lw(g::currentplayer)) != 0 {
        return;
    }
    let window = g::window;
    let font = if lw(window + 92) == 0 { 2 } else { 1 };
    let old = textSelect(font);
    let bottom = lw(window + 24);
    let y = if lw(window + 92) == 0 { bottom - 34 } else { bottom - 20 };
    let flags = lbu(g::fontflags);
    let line = if (lbu(g::fontflags) & 1) == 0 { lw(lw(g::curfont) + 12) } else { lw(lw(g::curfont) + 12) >> 1 };
    let row_gap = 2 >> (lw(window + 92) & 31);
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    let (fg, bg) = (lw(g::menuitem_fgcol), lw(g::menuitem_bgcol));
    for (k, offset) in [1780, 1776].into_iter().enumerate() {
        textSelect(font);
        sb(g::fontflags, flags);
        let text = lw(texts + offset);
        // (the game's text is a button, `^3`, a space and the word: the word after a button comes out larger, so the button and the word are drawn apart, at the same size)
        let (icon, word) = if lbu(text) == b'^' as i32 && lbu(text + 2) == b' ' as i32 { (text, text + 3) } else { (0, text) };
        let icon_text = BASE + 0x17_3e00 + 8 * k as i32;
        if icon != 0 {
            sb(icon_text, lbu(icon));
            sb(icon_text + 1, lbu(icon + 1));
            sb(icon_text + 2, 0);
        }
        let icon_width = if icon != 0 { textWidth(icon_text) } else { 0 };
        let gap = if icon != 0 { 10 } else { 0 };
        let word_width = textWidth(word);
        let left = lw(window + 8) - ((icon_width + gap + word_width) >> 1);
        let top = y + k as i32 * (line + row_gap);
        if icon != 0 {
            textOutline(left, top, icon_text, fg, bg);
        }
        textOutline(left + icon_width + gap, top, word, fg, bg);
    }
    textSelect(old);
}
