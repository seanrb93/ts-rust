//! The derived statistics of a game, and the drawing of the challenge page.

use crate::*;

/// Whether the statistics slot `id` is kept in this game: the slots below the number of players and characters, or below 4 in the modes 6 and 7 and in games with option flag 32.
fn slot_kept(id: i32) -> bool {
    let parms = g::curgameparms;
    if ((lw(parms + 72) - 6) as u32) >= 2 && (lw(parms + 80) & 32) == 0 {
        id < lw(g::lvnumplayers) + lw(g::numactivechars)
    } else {
        id < 4
    }
}

/// Work out the derived statistics of every slot that is kept: the shots per hit (10, 11) and the shares of the hits by place (18 to 22) from the counts, the gun with the most
/// kills (43, 44, 45) over the 33 guns (or the special 33 when the stat 63 beats it), and the damage per second (59) over the game's time (@ 0x221468).
pub fn StatsCalc() {
    let mut seconds = lw(g::gameframe) as f32 * lwc1(0x3ac874);
    if seconds == 0.0_f32 {
        seconds = lwc1(0x3ac878);
    }
    let mut slot = 0;
    loop {
        if !slot_kept(slot) {
            break;
        }
        let shots = StatsGet(8, slot);
        if shots == 0.0_f32 {
            StatsSet(10, slot, 0.0_f32);
            StatsSet(11, slot, 1.0_f32);
        } else {
            let hits = StatsGet(9, slot);
            let total = StatsGet(8, slot);
            StatsSet(10, slot, hits / total);
            let total = StatsGet(8, slot);
            let hits = StatsGet(9, slot);
            let again = StatsGet(8, slot);
            StatsSet(11, slot, (total - hits) / again);
        }
        let hits = StatsGet(9, slot);
        if hits == 0.0_f32 {
            for stat in 18..=22 {
                StatsSet(stat, slot, 0.0_f32);
            }
        } else {
            for (stat, part) in [(18, 13), (19, 15), (20, 16), (21, 17), (22, 12)] {
                let top = StatsGet(part, slot);
                let all = StatsGet(9, slot);
                StatsSet(stat, slot, top / all);
            }
        }
        slot += 1;
    }
    let mut slot = 0;
    loop {
        if !slot_kept(slot) {
            break;
        }
        let mut best_gun = 0;
        let mut total = 0.0_f32;
        let mut best_kills = 0;
        let mut gun = 0;
        let mut best = total;
        loop {
            let score = StatsGetGun(3, slot, gun);
            if best < score {
                best_gun = gun;
                best_kills = StatsGetGun(0, slot, best_gun) as i32;
                best = StatsGetGun(3, slot, best_gun);
            }
            let value = StatsGetGun(4, slot, gun);
            gun += 1;
            total += value;
            if gun >= 33 {
                break;
            }
        }
        let special = StatsGet(63, slot);
        if best < special {
            StatsGet(63, slot);
            best_gun = 33;
            best_kills = StatsGet(62, slot) as i32;
        }
        let kills = best_kills as f32;
        StatsSet(43, slot, best_gun as f32);
        StatsSet(44, slot, kills);
        let shots = StatsGet(8, slot);
        if !(0.0_f32 < shots) {
            StatsSet(45, slot, 0.0_f32);
        } else {
            let shots = StatsGet(8, slot);
            StatsSet(45, slot, kills / shots);
        }
        StatsSet(59, slot, total / seconds);
        slot += 1;
    }
}

/// Take half of `width` off the window's middle `x`, as the centred text is placed.
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

/// Draw the challenge page: the challenge's name (from the setup) and its description or, in game state 3, the first and the second briefing text; otherwise the result of the
/// challenge (the score of a finished or failed one, or the flashing "new record" for a record), the best score so far (unless option flag 0x40000000) and the retry-or-quit
/// box (@ 0x210910).
pub fn challenge_pageGfx() {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let window = g::window;
    let old_font = textSelect(2);
    sw(sp + 12, old_font);
    let setup = challengeGetSetup();
    let top = lw(window + 20);
    textSelect(2);
    let line_a = top + 56;
    let line_b = top + 112;
    if lw(g::gamestate) == 3 {
        let languages = g::languages;
        let name = lw((lh(setup) << 2) + lw((lw(g::cur_language) << 2) + languages));
        let width = textWidth(name);
        textOutline(centred(lw(window + 8), width), top + 82, lw((lh(setup) << 2) + lw((lw(g::cur_language) << 2) + languages)), 0x40787880, 0x5080);
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        let text = lw((lh(setup + 60) << 2) + lw((lw(g::cur_language) << 2) + languages));
        guiDrawFormattedText(text, 0, line_b, 1, fg, bg);
        textSelect(lw(sp + 12));
        return;
    }
    let languages = g::languages;
    let name = lw((lh(setup) << 2) + lw((lw(g::cur_language) << 2) + languages));
    sw(sp + 16, top + 128);
    let width = textWidth(name);
    textOutline(
        centred(lw(window + 8), width),
        line_a,
        lw((lh(setup) << 2) + lw((lw(g::cur_language) << 2) + languages)),
        0x40787880,
        0x5080,
    );
    let flag = lw(g::curgameparms + 80) & 0x10000000;
    let description = if flag == 0 {
        lw(lw((lw(g::cur_language) << 2) + languages) + 1848)
    } else {
        lw(lw((lw(g::cur_language) << 2) + languages) + 1844)
    };
    let width = textWidth(description);
    let x = centred(lw(g::window + 8), width);
    let y = line_a + line_height();
    let colour = if flag != 0 { 0x40787880 } else { 0x7f202080 };
    let parms = g::curgameparms;
    textOutline(x, y, description, colour, 64);
    let options = lw(parms + 80);
    if (options & 0x40000000) == 0 {
        sw(sp, 0);
        sw(sp + 4, 0);
        sw(sp + 8, 0);
        if (options & 0x10000000) == 0 {
            challengeGetDisplayFailed(sp, sp | 4, sp | 8);
        } else {
            sw(sp, lw(lw((lw(g::cur_language) << 2) + g::languages) + 1852));
            let score = challengeGetScore();
            let text = challengeFormatScore(lw(parms + 68), score);
            sw(sp + 4, text);
        }
        let fg = lw(g::menuitem_fgcol);
        if lw(sp + 8) == 0 {
            let value = lw(sp + 4);
            if lw(sp) != 0 && value != 0 {
                textSelect(1);
                let label = lw(sp);
                let fg = lw(g::menuitem_fgcol);
                let width = textWidth(label);
                let bg = lw(g::menuitem_bgcol);
                textOutline(centred(lw(g::window + 8), width), line_b - 16, lw(sp), fg, bg);
                let y = line_b - 15 + line_height();
                textSelect(2);
                let width = textWidth(lw(sp + 4));
                textOutline(centred(lw(g::window + 8), width), y, lw(sp + 4), 0x40787880, 0x5080);
            }
        } else {
            textSelect(2);
            let width = textWidth(lw(sp + 8));
            let bg = lw(g::menuitem_bgcol);
            textOutline(centred(lw(g::window + 8), width), line_b - 16, lw(sp + 8), fg, bg);
        }
    } else {
        textSelect(1);
        let label = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1860);
        let y = line_b - line_height() + 5;
        let flashing = lw(g::tickframenum) & 32;
        let fg = if flashing == 0 { lw(g::menuitem_fgcol) } else { 0x7f7f7f80 };
        let width = textWidth(label);
        let bg = lw(g::menuitem_bgcol);
        textOutline(centred(lw(g::window + 8), width), y, label, fg, bg);
        let y = y + 1 + line_height();
        textSelect(2);
        let score = challengeGetScore();
        let text = challengeFormatScore(lw(g::curgameparms + 68), score);
        let width = textWidth(text);
        let colour = if flashing != 0 { 0x7f7f7f80 } else { 0x40787880 };
        textOutline(centred(lw(g::window + 8), width), y, text, colour, 0x5080);
    }
    let window_now = g::window;
    if (lw(g::curgameparms + 80) & 0x40000000) == 0 {
        let best_y = lw(sp + 16);
        textSelect(1);
        let label = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1856);
        let width = textWidth(label);
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        textOutline(centred(lw(g::window + 8), width), best_y, label, fg, bg);
        let y = best_y + 1 + line_height();
        textSelect(2);
        let parms = g::curgameparms;
        let player = statsGetPlayer(0);
        let language = lw(g::cur_language);
        let text;
        if lw(player + (lw(parms + 68) << 3) + 1400) != 0 {
            let player = statsGetPlayer(0);
            let kind = lw(parms + 68);
            let best = lw(player + (kind << 3) + 1404);
            text = challengeFormatScore(kind, best);
        } else {
            text = lw(lw((language << 2) + g::languages) + 1760);
        }
        let width = textWidth(text);
        textOutline(centred(lw(g::window + 8), width), y, text, 0x40787880, 0x5080);
    }
    let bottom = lw(window_now + 24);
    drawRetryQuit(if lw(window_now + 92) == 0 { bottom - 56 } else { bottom - 30 });
    textSelect(lw(sp + 12));
}
