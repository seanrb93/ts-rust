//! The end screens of a story level (not completed, completed) and the list of awards.

use crate::*;

/// Take half of `width` off `x`, rounding the half towards 0, as the centred text is placed.
fn centred(x: i32, width: i32) -> i32 {
    x - ((((width as u32) >> 31) as i32 + width) >> 1)
}

/// The height of a line of the current font, halved when the font flag 1 is set.
fn line_height() -> i32 {
    if (lbu(g::fontflags) & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    }
}

/// Draw the end screen of a story level that was not completed. In game state 3 (and for a player whose kind at +296 is not 4) the title (language text +1792), the briefing, the
/// text at +1796 and the time in the game (bag tag format); otherwise the failed title (+1804) and, for the first player when flag 0x20000000 of the game's options is set, the
/// retry-or-quit box, else the text at +1688 near the bottom (@ 0x211630).
pub fn drawStoryNotCompleted() {
    let window = g::window;
    let players = ilinkGetNumLocalPlayers();
    let big = if 1 < players { 0 } else { 2 };
    let players_now = ilinkGetNumLocalPlayers();
    let small = (players_now < 2) as i32 ^ 1;
    let old_font = textSelect(big);
    let split = lw(window + 92);
    let top = lw(window + 20);
    let base = if split != 0 { top + 20 } else { top + 38 };
    let y = if split != 0 { base + 10 } else { base + 20 };
    if lw(g::gamestate) == 3 {
        let language = lw(g::cur_language);
        if lw(lw(g::currentplayer) + 296) != 4 {
            let title = lw(lw((language << 2) + g::languages) + 1792);
            let width = textWidth(title);
            textOutline(centred(lw(window + 8), width), y, title, 0x40787880, 0x5080);
            let line = lw(lw(g::curfont) + 12) >> (lbu(g::fontflags) & 1 & 31);
            let y = (if lw(window + 92) == 0 { y + 16 } else { y + 2 }) + line;
            let briefing = StoryGetBriefing();
            let fg = lw(g::menuitem_fgcol);
            let bg = lw(g::menuitem_bgcol);
            guiDrawFormattedText(briefing, 0, y, small, fg, bg);
            let y = if lw(g::window + 92) == 0 { y + 48 } else { y + 28 };
            textSelect(1);
            let label = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1796);
            let width = textWidth(label);
            let fg = lw(g::menuitem_fgcol);
            textOutline(centred(lw(g::window + 8), width), y, label, fg, 80);
            let y = y + 1 + line_height();
            textSelect(big);
            let time = gameFormatBagTagTime(lw(g::gameframe));
            sb(g::fontflags, lbu(g::fontflags) | 2);
            let width = textWidth(time);
            textOutline(centred(lw(g::window + 8), width), y, time, 0x40787880, 0x5080);
            sb(g::fontflags, lbu(g::fontflags) & 253);
            textSelect(old_font);
            return;
        }
    }
    let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1804);
    let width = textWidth(title);
    textOutline(centred(lw(g::window + 8), width), y, title, 0x7f202080, 80);
    let mut language = lw(g::cur_language);
    if lw(lw(g::currentplayer)) == 0 {
        let window = g::window;
        if (lw(g::curgameparms + 80) & 0x20000000) != 0 {
            let bottom = lw(window + 24);
            drawRetryQuit(if lw(window + 92) == 0 { bottom - 56 } else { bottom - 30 });
            textSelect(old_font);
            return;
        }
        language = lw(g::cur_language);
    }
    let text = lw(lw((language << 2) + g::languages) + 1688);
    textSelect(small);
    let width = textWidth(text);
    let window = g::window;
    let fg = lw(g::menuitem_fgcol);
    let y = lw(window + 24) - 30;
    let bg = lw(g::menuitem_bgcol);
    textOutline(centred(lw(window + 8), width), y, text, fg, bg);
    textSelect(old_font);
}

/// Draw the end screen of a story level that was completed: the title (+1800), the text at +1808 (or, when the level's bit for the front end's player is set in the game's
/// options, the text at +1816 flashing with the tick), the time of the game, and, unless that bit is set, the best time of the level (+328 of the stats for one player, +340 for
/// more) under the text at +1812 (@ 0x2110d0).
pub fn drawStoryCompleted() {
    let window = g::window;
    let players = ilinkGetNumLocalPlayers();
    let big = if 1 < players { 0 } else { 2 };
    ilinkGetNumLocalPlayers();
    let old_font = textSelect(big);
    let top = lw(window + 20);
    let split = lw(window + 92);
    let base = if split != 0 { top + 20 } else { top + 38 };
    let time_now = lw(g::gameframe);
    let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1800);
    let y = if split != 0 { base + 9 } else { base + 14 };
    let width = textWidth(title);
    textOutline(centred(lw(window + 8), width), y, title, 0x40787880, 0x5080);
    let line = lw(lw(g::curfont) + 12) >> (lbu(g::fontflags) & 1 & 31);
    let mut y = (if lw(window + 92) == 0 { y + 16 } else { y + 2 }) + line;
    let mut colour = lw(g::menuitem_fgcol);
    textSelect(1);
    let text;
    if (lw(g::curgameparms + 80) & 0x40000000 << (lw(lw(g::front)) & 31)) == 0 {
        text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1808);
    } else {
        y = y + 1 + line_height();
        colour = if (lw(g::tickframenum) & 32) != 0 { 0x7f7f7f80 } else { 0x50505080 };
        text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1816);
    }
    let width = textWidth(text);
    textOutline(centred(lw(g::window + 8), width), y, text, colour, 80);
    let y = y + 1 + line_height();
    let window = g::window;
    textSelect(big);
    let time = gameFormatTime(time_now);
    let width = textWidth(time);
    textOutline(centred(lw(window + 8), width), y, time, 0x40787880, 0x5080);
    let line = lw(lw(g::curfont) + 12) >> (lbu(g::fontflags) & 1 & 31);
    let y = (if lw(window + 92) == 0 { y + 10 } else { y + 2 }) + line;
    let parms = g::curgameparms;
    if (lw(parms + 80) & 0x40000000 << (lw(lw(g::front)) & 31)) != 0 {
        textSelect(old_font);
        return;
    }
    let best;
    if lw(g::lvnumplayers) != 1 {
        let stats = statsGet();
        best = lw(stats + ((lw(parms + 76) << 2) + lw(g::lvlevelnum) * 36) + 340);
    } else {
        let stats = statsGet();
        best = lw(stats + ((lw(parms + 76) << 2) + lw(g::lvlevelnum) * 36) + 328);
    }
    textSelect(1);
    let label = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1812);
    let width = textWidth(label);
    let fg = lw(g::menuitem_fgcol);
    textOutline(centred(lw(g::window + 8), width), y, label, fg, 80);
    let y = y + 1 + line_height();
    textSelect(big);
    let best_text = gameFormatTime(best);
    let width = textWidth(best_text);
    textOutline(centred(lw(g::window + 8), width), y, best_text, 0x40787880, 0x5080);
    textSelect(old_font);
}

/// Draw the list of the awards of the current player: the title (+0x11b4), the text of each of the three awards that was given (`vafmt` of the format at 0x3ae530), the heading
/// (+0x1170) and the name of the gun of the last award stat 43 (the fixed gun for 33, the name of the combo's gun otherwise, the text at +0x1174 for none); the list is moved
/// up for 2 to 4 awards (@ 0x2103f0).
pub fn drawAwards() {
    let players = ilinkGetNumLocalPlayers();
    let small = (players < 2) as i32 ^ 1;
    let old_font = textSelect(small);
    let window = g::window;
    let half_width = lw(window + 12);
    let split = lw(window + 92);
    let top = lw(window + 20);
    let step = if split != 0 { 10 } else { 18 };
    let across = lw(g::window) + ((half_width + ((half_width as u32) >> 31) as i32) >> 1);
    let player = lw(lw(g::currentplayer));
    let mut shift = if split == 0 { step << 1 } else { step + 3 };
    let awards = StatsGetNumAwards(player);
    if awards == 3 {
        shift -= if lw(window + 92) == 0 { 10 } else { 6 };
    } else if awards >= 4 {
        if awards == 4 {
            shift -= if lw(window + 92) == 0 { 15 } else { 8 };
        }
    } else if awards == 2 {
        shift -= if lw(window + 92) == 0 { 5 } else { 4 };
    }
    let language = lw(g::cur_language);
    let y = (if split != 0 { top + 20 } else { top + 38 }) + shift;
    let title = lw(lw((language << 2) + g::languages) + 0x11b4);
    let width = textWidth(title);
    let mut next = y + step;
    textOutline(centred(across, width), y, title, 0x40787880, 0x5080);
    let mut i = 0;
    loop {
        let award = StatsGetAward(lw(lw(g::currentplayer)), i);
        if award != 0 {
            let tier = StatsGetAward(lw(lw(g::currentplayer)), i);
            let text = vafmt(&[0x3ae530, tier as i64]);
            let width = textWidth(text);
            let at = next;
            next += if split != 0 { 10 } else { 16 };
            textOutline(centred(across, width), at, text, 0x7f7f7f80, 80);
        }
        i += 1;
        if i >= 3 {
            break;
        }
    }
    let languages = g::languages;
    let y = next + step;
    let heading = lw(lw((lw(g::cur_language) << 2) + languages) + 0x1170);
    let width = textWidth(heading);
    textOutline(centred(across, width), y, heading, 0x40787880, 0x5080);
    let stat = StatsGet(43, lw(lw(g::currentplayer)));
    let language = lw(g::cur_language);
    let text;
    if stat == 0.0_f32 {
        text = lw(lw((language << 2) + languages) + 0x1174);
    } else {
        let stat = StatsGet(43, lw(lw(g::currentplayer)));
        let language = lw(g::cur_language);
        if !(stat == 33.0_f32) {
            let stat = StatsGet(43, lw(lw(g::currentplayer)));
            let texts = lw((lw(g::cur_language) << 2) + languages);
            let name = lw((lh(((stat as i32) << 3) + (GUNCOMBOS + 6)) << 2) + texts);
            text = vafmt(&[0x3ae530, name as i64, texts as i64]);
        } else {
            text = lw((lh(g::fixedgunname) << 2) + lw((language << 2) + languages));
        }
    }
    let width = textWidth(text);
    textOutline(centred(across, width), y + step, text, 0x7f7f7f80, 80);
    textSelect(old_font);
}
