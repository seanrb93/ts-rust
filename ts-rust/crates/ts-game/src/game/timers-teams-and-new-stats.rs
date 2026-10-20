//! The timers of the game, the teams a map allows, and clearing a set of statistics.

use crate::*;

/// Run the timers (records of 24 bytes at `timers`: the state at +0, 0 counting down, 1 repeating, 2 finished; the time at +4; the frame it was last looked at at +8; the period
/// at +12; the argument at +16; the function at +20): the frames passed since the last look are taken off a counting down timer, which calls its function when it reaches 0
/// and is finished, or added to a repeating one, which calls its function each time it passes its period and keeps what is over; then the finished timers are released (@ 0x2cdbc0).
pub fn timerTick() {
    let mut count = lw(g::numtimers);
    let mut i = 0;
    if count > 0 {
        loop {
            let timers = g::timers;
            let offset = i * 24;
            let frame = lw(g::gameframe);
            let seen_at = offset + (timers + 8);
            let record = offset + timers;
            let last = lw(seen_at);
            sw(seen_at, frame);
            let passed = frame - last;
            let state = lw(record);
            if state == 0 {
                let time_at = offset + (timers + 4);
                let time = lw(time_at) - passed;
                sw(time_at, time);
                if (time as f32) <= 0.0_f32 {
                    let rec = timers + offset;
                    let argument = lw(rec + 16);
                    call_indirect(lw(rec + 20), [argument, rec, offset, timers, 0, 0, 0, 0], [0.0; 8]);
                    count = lw(g::numtimers);
                    sw(record, 2);
                }
            } else if state == 1 {
                let time_at = offset + (timers + 4);
                let period_at = offset + (timers + 12);
                let time = lw(time_at) + passed;
                let due = lwc1(period_at) <= time as f32;
                sw(time_at, time);
                if due {
                    let rec = timers + offset;
                    let argument = lw(rec + 16);
                    call_indirect(lw(rec + 20), [argument, rec, offset, timers, 0, 0, 0, 0], [0.0; 8]);
                    count = lw(g::numtimers);
                    sw(time_at, (lw(time_at) as f32 - lwc1(period_at)) as i32);
                }
            }
            i += 1;
            if i >= count {
                break;
            }
        }
    }
    let mut i = count - 1;
    if i >= 0 {
        loop {
            if lw(i * 24 + g::timers) == 2 {
                timerRelease(i);
            }
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }
}

/// Work out which of the four teams (the bytes at 0x1fc1234, all 1 to start with) and the four team start groups (bytes from +60 of the page record at 0x1fc11f8) the map in the
/// buffer `mmtempbuffer` can be played with: those with no item of their kind in the map are turned off; in the game mode 4 the first two start groups are set on and the other two off (@ 0x2353e8).
pub fn checkAllowedTeams() {
    let page = 0x1fc11f8;
    memset(0x1fc1234, 1, 4);
    if lw(page + 36) == 18 && lw(lw(g::mmtempbuffer)) == 0x1324626 {
        let mode = lw(page + 48);
        let mut check_teams = false;
        let mut skip = false;
        if (mode == 3 || mode == 1) && lw(g::gv_teamplay) != 0 && lw(g::gv_useteamstarts) != 0 {
            check_teams = true;
        } else if lw(lw(0x1fc1228) * 12 + (g::gamemodedata + 4)) == 0 {
            skip = true;
        } else {
            check_teams = true;
        }
        if !skip && check_teams {
            let mut team = 0;
            loop {
                let kind = team + 1;
                if mmfileGetItemCount(lw(g::mmtempbuffer), kind) == 0 {
                    sb(team + 0x1fc1234, 0);
                }
                team = kind;
                if team >= 4 {
                    break;
                }
            }
        }
        if lw(lw(page + 48) * 12 + (g::gamemodedata + 8)) != 0 {
            let mut group = 0;
            loop {
                let kind = group + 5;
                if mmfileGetItemCount(lw(g::mmtempbuffer), kind) == 0 {
                    sb(group + (page + 60), 0);
                }
                group += 1;
                if group >= 4 {
                    break;
                }
            }
        }
    }
    if lw(page + 48) == 4 {
        sw(page + 60, 0);
        sb(page + 61, 1);
        sb(page + 60, 1);
    }
}

/// Clear a set of statistics (2804 bytes) and set up the three lives in it (at +1616, +2012 and +2408) (@ 0x220080).
pub fn statsNew(stats: i32) {
    memset(stats, 0, 2804);
    StatsLifeReset(stats + 1616);
    StatsLifeReset(stats + 2012);
    StatsLifeReset(stats + 2408);
}
