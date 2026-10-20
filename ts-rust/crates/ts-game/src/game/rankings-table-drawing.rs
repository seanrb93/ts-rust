//! The drawing of the table of rankings at the end of a game.

use crate::*;

/// Draw the rankings table for the kind `kind` (1 and 2 rank the characters by their kills against the current player (1) or against each other (2); 3 ranks the teams): the
/// scores and the records are put in pairs (8 bytes each from sp), at most 18 of them sorted (`bubbleSort`, into the pointers at sp+864) and, up to 8 rows (6 with more than one
/// player, and fewer when there are fewer), each row is drawn as a gradient bar with the rank (tied scores share the better rank), the name of the character or team and the
/// score (or the bag tag time) at its end; when only one row can be shown it is the player's own row (@ 0x20e5a0).
pub fn drawRankingsTable(kind: i32) {
    let frame = Frame::new(1264);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    sw(sp + 1056, kind);
    sw(sp + 1068, lw(lw(player + 384) + 352));
    let count;
    if kind != 3 {
        let players = lw(g::lvnumplayers);
        let characters = lw(g::numactivechars);
        let mut i = 0;
        if players + characters <= 0 {
            count = 0;
        } else {
            let chrinfo = lw(g::chrinfo);
            loop {
                let record = i * 0x1210 + chrinfo;
                let score;
                if lw(sp + 1056) == 1 {
                    score = lh(record + (lw(player) << 1) + 32);
                } else {
                    let mut value = 0;
                    if lw(sp + 1056) == 2 {
                        value = lh(lw(sp + 1068) + (i << 1) + 32);
                    }
                    score = value;
                }
                let offset = i << 3;
                i += 1;
                sw(sp + offset, score);
                sw(sp + 4 + offset, record);
                if i >= players + characters {
                    break;
                }
            }
            count = i;
        }
    } else {
        memset(sp + 944, 0, 109);
        let mut i = 0;
        let mut players = lw(g::lvnumplayers);
        let mut characters = lw(g::numactivechars);
        let mut found = 0;
        if players + characters > 0 {
            let mut chrinfo = lw(g::chrinfo);
            loop {
                let offset = i * 0x1210;
                let team = lw(i * 0x1210 + chrinfo + 24);
                let seen_at = sp + 944 + team;
                if lbu(seen_at) != 0 {
                    i += 1;
                } else {
                    sb(seen_at, 1);
                    let score = teamGetScore(team);
                    let slot = found << 3;
                    chrinfo = lw(g::chrinfo);
                    sw(sp + slot, score);
                    found += 1;
                    players = lw(g::lvnumplayers);
                    sw(sp + 4 + slot, chrinfo + offset);
                    characters = lw(g::numactivechars);
                    i += 1;
                }
                if i >= players + characters {
                    break;
                }
            }
        }
        count = found;
    }
    let rows = if count < 19 { count } else { 18 };
    bubbleSort(&[sp as i64, (sp + 864) as i64, rows as i64]);
    sw(sp + 1060, 8);
    let players = lw(g::lvnumplayers);
    sw(sp + 1072, -1);
    let limit = if players >= 2 { 6 } else { lw(sp + 1060) };
    sw(sp + 1060, limit);
    let mode = lw(sp + 1056);
    sw(sp + 1060, if rows < limit { rows } else { lw(sp + 1060) });
    if mode == 3 {
        let mut i = 0;
        if rows > 0 {
            let mut at = sp + 864;
            loop {
                sw(sp + 1072, if (lw(lw(at) + 4) ^ lw(sp + 1068)) == 0 { i } else { lw(sp + 1072) });
                i += 1;
                at += 4;
                if i >= rows {
                    break;
                }
            }
        }
    }
    let players_here = ilinkGetNumLocalPlayers();
    let old_font = textSelect((players_here < 2) as i32 ^ 1);
    sw(sp + 1064, old_font);
    let players_here = ilinkGetNumLocalPlayers();
    let window = g::window;
    let width = if 2 >= players_here { 320 } else { 246 };
    let split = lw(window + 92);
    let left = lw(window + 8) - ((width as u32) >> 1) as i32;
    let top = lw(window + 20);
    sw(sp + 1076, left);
    let shift = 10 >> (split & 31);
    sw(sp + 1080, left + width + 8);
    let first = if split == 0 { shift + 38 } else { shift + 20 };
    let mut y = top + first;
    let mut remaining = lw(sp + 1060);
    let mut row = 0;
    if remaining != 0 {
        let mode = lw(sp + 1056);
        if rows > 0 {
            sw(sp + 1092, mode - 1);
            let mut offset = row << 2;
            loop {
                'row: {
                    let pair = lw(sp + 864 + offset);
                    let ranked_by_kills = ((lw(sp + 1092) as u32) < 2) as i32;
                    sw(sp + 1084, lw(pair + 4));
                    let mut next = row + 1;
                    if ranked_by_kills != 0 && lw(pair) == 0 {
                        row = next;
                        break 'row;
                    }
                    if row < lw(sp + 1072) && remaining == 1 {
                        row = next;
                        break 'row;
                    }
                    let mut flags = lbu(g::fontflags);
                    if remaining == 1 {
                        if (lw(sp + 1060) - 1) < row {
                            y += 4 >> (lw(g::window + 92) & 31);
                        }
                        flags = lbu(g::fontflags);
                    }
                    let font = lw(g::curfont);
                    let bottom = if (flags & 1) == 0 { y + lw(lw(g::curfont) + 12) } else { y + (lw(font + 12) >> 1) };
                    guiHGradient(lw(sp + 1076), y, lw(sp + 1080), bottom, lw(lw(sp + 1084) + 20), 80, 16);
                    let colour;
                    if lw(lw(sp + 1084)) != lw(lw(sp + 1068)) {
                        sw(sp + 1088, 80);
                        colour = 0x7f7f7f80;
                    } else if (lw(sp + 1092) as u32) >= 2
                        && lw(lw(sp + 864 + offset)) >= 0
                        && !(remaining == 1 && (lw(sp + 1060) - 1) < row)
                    {
                        sw(sp + 1088, 0x5080);
                        colour = 0x40787880;
                    } else {
                        sw(sp + 1088, 80);
                        colour = 0x7f1e1e80;
                    }
                    let mut first_tied = row;
                    let rows_left = remaining - 1;
                    next = row + 1;
                    let name_x = lw(sp + 1076) + 4;
                    if row > 0 {
                        let score_at = lw(lw(sp + 864 + offset));
                        while first_tied > 0 && lw(lw(sp + 864 + ((first_tied - 1) << 2))) == score_at {
                            first_tied -= 1;
                        }
                    }
                    let rank = vafmt(&[(g::lastgfxendtime + 4) as i64, (first_tied + 1) as i64]);
                    let width = textWidth(rank);
                    textOutline(lw(sp + 1076) - width - 8, y, rank, colour, lw(sp + 1088));
                    let mode_now = lw(sp + 1056);
                    let name;
                    if (lw(g::curgameparms + 80) & 1) != 0 && mode_now == 3 {
                        name = gameGetTeamName(lw(lw(sp + 1084) + 24));
                    } else {
                        name = lw(lw(sp + 1084) + 16);
                    }
                    textOutline(name_x, y, name, colour, lw(sp + 1088));
                    let text;
                    let game_mode = lw(g::curgameparms + 72);
                    let shown = lw(sp + 1056);
                    if game_mode == 1 && shown != 2 && shown != game_mode {
                        text = gameFormatBagTagTime(lw(lw(sp + 864 + offset)));
                    } else {
                        text = vafmt(&[(g::lastgfxendtime + 4) as i64, lw(lw(sp + 864 + offset)) as i64]);
                    }
                    sb(g::fontflags, lbu(g::fontflags) | 2);
                    let width = textWidth(text);
                    textOutline(lw(sp + 1080) - (width + 4), y, text, colour, lw(sp + 1088));
                    let flags = lbu(g::fontflags) & 253;
                    let gap = 2 >> (lw(g::window + 92) & 31);
                    sb(g::fontflags, flags);
                    let height = if (flags & 1) == 0 { lw(lw(g::curfont) + 12) } else { lw(lw(g::curfont) + 12) >> 1 };
                    y += height + gap;
                    remaining = rows_left;
                    row = next;
                }
                if remaining == 0 {
                    break;
                }
                offset = row << 2;
                if row >= rows {
                    break;
                }
            }
        }
    }
    textSelect(lw(sp + 1064));
}
