//! Scores of characters and teams, and creating links between pads.

use crate::*;

/// A character's score for the mode: its kills for mode 3, else the score word of its record (+248); 0 for modes beyond 7 (@ 0x289160).
pub fn chrGetScore(chr: i32) -> i32 {
    let mode = lw(g::curgameparms + 72) - 1;
    if (mode as u32) >= 7 {
        return 0;
    }
    if mode == 2 {
        chrCountKills(lw(g::chrinfo) + chr * 0x1210)
    } else {
        lw(chr * 0x1210 + lw(g::chrinfo) + 248)
    }
}

/// A team's score: the sum of the scores of its characters (@ 0x289078).
pub fn teamGetScore(team: i32) -> i32 {
    let mut total = 0;
    let mut i = 0;
    while i < lw(g::lvnumplayers) + lw(g::numactivechars) {
        let info = lw(g::chrinfo);
        if lw(i * 0x1210 + info + 24) == team {
            let mode = lw(g::curgameparms + 72) - 1;
            if (mode as u32) < 7 {
                if mode == 2 {
                    total += chrCountKills(i * 0x1210 + info);
                } else {
                    total += lw(i * 0x1210 + info + 248);
                }
            }
        }
        i += 1;
    }
    total
}

/// Make a link between two pads (addresses) in the pad editor, unless they are not valid, the same, or already linked; the link mode (1 or 2)
/// sets the flags (3, or 1 and 4), and the counters of links inside a hall and between halls count it (@ 0x264af8).
pub fn createlink(a: i32, b: i32) {
    if padvalid(a) != 0 && padvalid(b) != 0 && a != b && findlink(a, b) == 0 {
        let link = newlink();
        let mode = lw(g::linkmode);
        sw(link, 1);
        if mode == 1 {
            sw(link, 3);
        }
        let mut last = lw(g::lastpadnum);
        if mode == 2 {
            sw(link, lw(link) | 4);
            last = lw(g::lastpadnum);
        }
        let current = lw(g::curpadnum);
        let pads = lw(g::levelpads);
        sw(link + 8, current);
        sw(link + 4, last);
        let between = lw(0x3b00a8);
        if lh(last * 28 + pads + 8) != lh(current * 28 + pads + 8) {
            sw(0x3b00a8, between + 1);
        } else {
            sw(0x3b00a4, lw(0x3b00a4) + 1);
        }
    }
}

/// Counted the links of the pads and printed nothing (@ 0x265828).
pub fn padCheckLinks() {}
