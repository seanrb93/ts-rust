//! Playing a match again without going back to the menus: the End Game page of the menu of the game has "Restart" above "End Game" (in the story too), and the results page at the end of a match
//! restarts it with the cross button (the game does this itself for the game modes 4 and 6, `tickRetryQuit`).  The level starts again with the settings it had (`gameLevelRestart`).

use crate::*;

const BASE: i32 = 0x240_0000;
const MENU_AT: i32 = BASE + 0x17_3b00;
const TEXT_AT: i32 = BASE + 0x17_3a80;

/// Can the match be played again from here: a game of the arcade modes on this machine (not one on a network, and not the story)?
pub fn restart_available() -> bool {
    (1..=6).contains(&lw(g::curgameparms + 72)) && ilinkIsILinkGame() == 0 && lw(g::lvnumplayers) > 0
}

/// Can the level be started again from the End Game page: a match of the arcade modes or a level of the story, on this machine (not on a network).
pub fn restart_in_menu() -> bool {
    let mode = lw(g::curgameparms + 72);
    (restart_available() || mode == 0) && ilinkIsILinkGame() == 0 && lw(g::lvnumplayers) > 0
}

/// The menu of the End Game page: "Restart" and, below it, "End Game" (the game's one entry, which said "Press (A) to end game" in a text of its own
/// with the button's picture, and did not line up with the other).
pub fn restart_menu(own: i32) -> i32 {
    if !restart_in_menu() {
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
    sw(item(0), crate::manual::avmenu::put_str(&mut at, "Restart"));
    sh(item(0) + 8, 2);
    sw(item(1), crate::manual::avmenu::put_str(&mut at, "End Game"));
    sh(item(1) + 6, lh(item(0) + 6) + 20);
    // (the cursor goes down from Restart to End Game, and up again: the links name the items by their numbers, 2 and the game's)
    let end_id = lhu(item(1) + 8) as i32;
    sw(item(0) + 16, end_id << 16);
    sw(item(1) + 16, 2);
    item(0) - 16
}

/// The End Game page's tick: the button 64 on "Restart" (the first entry, where the cursor starts) starts the level again.  True when it did; on
/// "End Game" the game's own tick ends the game.
pub fn restart_page_pressed(page: i32, player: i32) -> bool {
    if !restart_in_menu() {
        return false;
    }
    // (the page keeps the number of the item the cursor is on: Restart's is 2)
    if lhu(page + 4) == 0 {
        sh(page + 4, 2);
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

/// What says that the match can be left or played again, under the results: the game's texts for Quit and Restart (a button and a word each) side by side on one row, so that both are drawn at the
/// same height, and so the same size (the text near the bottom of the screen is sized by the renderer with the HUD: two rows came out in two sizes).
pub fn restart_results_hint() {
    if lw(g::gamestate) != 4 || !restart_available() || lw(lw(g::currentplayer)) != 0 {
        return;
    }
    let window = g::window;
    let font = if lw(window + 92) == 0 { 2 } else { 1 };
    let old = textSelect(font);
    let flags = lbu(g::fontflags);
    let bottom = lw(window + 24);
    let y = if lw(window + 92) == 0 { bottom - 40 } else { bottom - 24 };
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    let (fg, bg) = (lw(g::menuitem_fgcol), lw(g::menuitem_bgcol));
    // (the button and the word of each: the button is a code of two characters in the game's text, then a space)
    let mut parts: Vec<(i32, i32, i32, i32)> = Vec::new();
    for (k, offset) in [1780, 1776].into_iter().enumerate() {
        let text = lw(texts + offset);
        let (icon, word) = if lbu(text) == b'^' as i32 && lbu(text + 2) == b' ' as i32 { (text, text + 3) } else { (0, text) };
        let icon_text = BASE + 0x17_3e00 + 8 * k as i32;
        if icon != 0 {
            sb(icon_text, lbu(icon));
            sb(icon_text + 1, lbu(icon + 1));
            sb(icon_text + 2, 0);
        }
        textSelect(font);
        let icon_width = if icon != 0 { textWidth(icon_text) } else { 0 };
        parts.push((if icon != 0 { icon_text } else { 0 }, icon_width, word, textWidth(word)));
    }
    let gap = 10;
    let between = 6 * gap;
    let total: i32 = parts.iter().map(|p| p.1 + if p.0 != 0 { gap } else { 0 } + p.3).sum::<i32>() + between;
    let mut x = lw(window + 8) - (total >> 1);
    for (icon_text, icon_width, word, word_width) in parts {
        textSelect(font);
        sb(g::fontflags, flags);
        if icon_text != 0 {
            textOutline(x, y, icon_text, fg, bg);
            x += icon_width + gap;
        }
        textOutline(x, y, word, fg, bg);
        x += word_width + between;
    }
    textSelect(old);
}
