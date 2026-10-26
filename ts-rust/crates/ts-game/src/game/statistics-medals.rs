//! Function: working out the medals (the awards at the end of a match) from the statistics of the players.

use crate::*;

/// How many players have statistics: 4 for the modes that do not count the computer characters, else the players and the computer characters.
fn count_limit() -> i32 {
    if ((lw(g::curgameparms + 72) - 6) as u32) >= (2 as u32) {
        if (lw(g::curgameparms + 80) & 32) == 0 {
            return lw(g::lvnumplayers) + lw(g::numactivechars);
        }
    }
    4
}

/// Where the flag of the medal `medal` of the player `player` is in the table of flags (48 floats of 4 bytes for each of up to 4 players, in the frame at `sp`).
fn flag(sp: i32, player: i32, medal: i32) -> i32 {
    sp + 64 + player * 192 + medal * 4
}

/// Give the medal `medal` to the player with the highest value of the statistic `stat` among `count` players when it is more than `more_than` (nobody when no one has
/// a value).
fn best_gets(sp: i32, stat: i32, count: i32, more_than: f32, medal: i32) {
    let best = StatsGetMax(stat, count);
    if best >= 0 {
        if more_than < StatsGet(stat, best) {
            swc1(flag(sp, best, medal), 1.0_f32);
        }
    }
}

/// The statistic of the match kept for each player at +20, +24 and so on in the table of the match (1368 bytes for each player).
fn matchstat(player: i32, at: i32) -> f32 {
    lwc1(g::matchstats + at + player * 1368)
}

/// Work out the medals of the players at the end of a match (@ 0x221910): some per second rates are stored first, then the flag of each of the 48 medals is set for the
/// players that earned it, and each player gets up to 3 of the medals they earned, picked at random and never the same for two players or twice; the text of each is
/// kept in the awards.
pub fn StatsCalcMedals() {
    let frame = Frame::new(3056);
    let sp = frame.sp();
    let mut seconds = lw(g::gameframe) as f32 * lwc1(0x3ac87c);
    let mut total_use = 0.0_f32;
    if seconds == total_use {
        seconds = lwc1(0x3ac880);
    }
    if ((lw(g::curgameparms + 72) - 6) as u32) < (2 as u32) {
        return;
    }
    if (lw(g::curgameparms + 80) & 32) != 0 {
        return;
    }
    // the rates per second
    let mut p = 0;
    while p < count_limit() {
        let mut q = 0;
        while q < count_limit() {
            let value = StatsGetChar(4, p, q);
            StatsSetChar(5, p, q, (value).pdiv(seconds));
            q += 1;
        }
        if StatsGet(47, p) == 0.0_f32 {
            StatsSet(49, p, 0.0_f32);
            StatsSet(51, p, 0.0_f32);
        } else {
            let a = (StatsGet(48, p)).pdiv(StatsGet(47, p));
            StatsSet(49, p, a);
            let b = (StatsGet(50, p)).pdiv(StatsGet(47, p));
            StatsSet(51, p, b);
        }
        StatsSet(53, p, (StatsGet(52, p)).pdiv(seconds));
        StatsSet(55, p, (StatsGet(54, p)).pdiv(seconds));
        StatsSet(57, p, (StatsGet(56, p)).pdiv(seconds));
        p += 1;
    }
    // how well each player used the guns, as part of all
    let mut p = 0;
    while p < count_limit() {
        let at = sp + (p << 2);
        sw(at, 0);
        let mut gun = 0;
        loop {
            let used = StatsGetGun(3, p, gun);
            let combo = lh(g::guncombos + gun * 8);
            swc1(at, lwc1(at) + (used).pdiv(seconds) * (1.0_f32 - (combo as f32).pdiv(1e+02_f32)));
            gun += 1;
            if gun >= 33 {
                break;
            }
        }
        total_use = total_use + lwc1(sp + (p << 2));
        p += 1;
    }
    let mut p = 0;
    while p < count_limit() {
        let mut rate = 0.0_f32;
        if !(total_use == rate) {
            let own = lwc1(sp + (p << 2));
            let share = if ((lw(g::curgameparms + 72) - 6) as u32) < (2 as u32) {
                total_use * 0.25_f32
            } else if (lw(g::curgameparms + 80) & 32) != 0 {
                total_use * 0.25_f32
            } else {
                (total_use).pdiv((lw(g::lvnumplayers) + lw(g::numactivechars)) as f32)
            };
            rate = (own).pdiv(share);
        }
        StatsSet(42, p, rate);
        p += 1;
    }
    let mut p = 0;
    while p < count_limit() {
        StatsSet(58, p, 0.0_f32);
        p += 1;
    }
    let players = lw(g::lvnumplayers);
    if players > 0 {
        let mut p = 0;
        loop {
            let mut q = 0;
            if lw(g::lvnumplayers) > 0 {
                loop {
                    let value = StatsGetChar(5, q, p);
                    q += 1;
                    StatsAdd(58, p, value);
                    if q >= lw(g::lvnumplayers) {
                        break;
                    }
                }
            }
            p += 1;
            if p >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    let limit = count_limit();
    // clear the flags of the medals
    let mut p = 0;
    while p < count_limit() {
        let mut k = 0;
        while k < 48 {
            sw(flag(sp, p, k), 0);
            k += 1;
        }
        p += 1;
    }
    let np = lw(g::lvnumplayers);
    best_gets(sp, 0, limit, 15.0_f32, 0);
    best_gets(sp, 1, limit, 1e+01_f32, 1);
    let mut p = 0;
    while p < count_limit() {
        if StatsGet(0, p) == 0.0_f32 {
            swc1(flag(sp, p, 40), 1.0_f32);
        }
        p += 1;
    }
    best_gets(sp, 2, np, 4.0_f32, 2);
    let mut p = 0;
    while p < limit {
        if 7.0_f32 <= matchstat(p, 20) {
            swc1(flag(sp, p, 3), 1.0_f32);
        }
        p += 1;
    }
    let mut p = 0;
    while p < count_limit() {
        if 4.0_f32 <= matchstat(p, 24) {
            swc1(flag(sp, p, 4), 1.0_f32);
        }
        p += 1;
    }
    let mut p = 0;
    while p < count_limit() {
        if 1e+01_f32 < matchstat(p, 28) {
            swc1(flag(sp, p, 5), 1.0_f32);
        }
        p += 1;
    }
    // the rate of fire of a statistic, with a second one to meet
    let rated = [(10, 8, 20.0_f32, 0x3ac884, 6), (11, 8, 20.0_f32, 0x3ac888, 7), (22, 8, 30.0_f32, 0x3ac88c, 8), (13, 8, 15.0_f32, 0x3ac890, 9)];
    for (stat, base, enough, limit_at, medal) in rated {
        let best = StatsGetMax(stat, lw(g::lvnumplayers));
        if best >= 0 {
            if enough < StatsGet(base, best) {
                let second = if medal == 9 { 18 } else { stat };
                if lwc1(limit_at) < StatsGet(second, best) {
                    swc1(flag(sp, best, medal), 1.0_f32);
                }
            }
        }
    }
    best_gets(sp, 14, count_limit(), 6.0_f32, 10);
    best_gets(sp, 28, count_limit(), 6.0_f32, 11);
    best_gets(sp, 23, count_limit(), 5.0_f32, 12);
    best_gets(sp, 24, count_limit(), 3.0_f32, 13);
    let average = StatsGetAverage(25, count_limit());
    let mut p = 0;
    while p < count_limit() {
        if 0.0_f32 < average {
            if 2e+01_f32 <= StatsGet(25, p) {
                swc1(flag(sp, p, 14), 1.0_f32);
            }
        }
        p += 1;
    }
    let average = StatsGetAverage(32, count_limit());
    let ratio = lwc1(0x3ac894);
    let mut p = 0;
    while p < count_limit() {
        if 0.0_f32 < average {
            if 2e+01_f32 <= StatsGet(32, p) {
                if ratio <= (StatsGet(32, p)).pdiv(StatsGet(8, p)) {
                    swc1(flag(sp, p, 15), 1.0_f32);
                }
            }
        }
        p += 1;
    }
    // the first to some, and the ones that were slow after
    for (stat, more_than, medal, late_medal, scale_at) in [(33, 3.0_f32, 16, 18, 0x3ac898), (34, 2.0_f32, 17, 19, 0x3ac89c)] {
        let best = StatsGetMax(stat, count_limit());
        if best >= 0 {
            if more_than < StatsGet(stat, best) {
                swc1(flag(sp, best, medal), 1.0_f32);
                let mut q = 0;
                let mut count = lw(g::lvnumplayers);
                if count > 0 {
                    let scale = lwc1(scale_at);
                    loop {
                        if 1.2e+02_f32 < lw(g::gameframe) as f32 * scale {
                            if StatsGet(stat, q) < 1.0_f32 {
                                swc1(flag(sp, q, late_medal), 1.0_f32);
                            }
                            count = lw(g::lvnumplayers);
                        }
                        q += 1;
                        if q >= count {
                            break;
                        }
                    }
                }
            }
        }
    }
    let average = StatsGetAverage(46, count_limit());
    let mut p = 0;
    while p < count_limit() {
        let value = matchstat(p, 184);
        let mut part = 0.0_f32;
        if part < average {
            part = (value).pdiv(average);
        }
        if 0.0_f32 < part {
            swc1(flag(sp, p, 21), (1.0_f32).pdiv(part) * 0.5_f32);
        }
        p += 1;
    }
    let average = StatsGetAverage(46, count_limit());
    let mut count = lw(g::lvnumplayers);
    if count > 0 {
        let mut p = 0;
        loop {
            let mut part = 0.0_f32;
            let value = lwc1(p * 1368 + (g::matchstats + 184));
            if part < average {
                part = (value).pdiv(average);
            }
            if 0.0_f32 < part {
                let bonus = StatsGet(59, p);
                count = lw(g::lvnumplayers);
                swc1(flag(sp, p, 20), (1.0_f32).pdiv(part) * 65.0_f32 * bonus);
            }
            p += 1;
            if p >= count {
                break;
            }
        }
    }
    let count = lw(g::lvnumplayers);
    let mut sum = StatsGetAverage(46, count_limit());
    let mut p = 0;
    while p < count_limit() {
        sum = sum + lwc1(p * 1368 + (g::matchstats + 184));
        p += 1;
    }
    let mean = if ((lw(g::curgameparms + 72) - 6) as u32) >= (2 as u32) && (lw(g::curgameparms + 80) & 32) == 0 {
        (sum).pdiv((count + lw(g::numactivechars)) as f32)
    } else {
        sum * 0.25_f32
    };
    let scale = lwc1(0x3ac8a0);
    let mut p = 0;
    while p < count_limit() {
        let mut part = 0.0_f32;
        let value = matchstat(p, 184);
        if part < mean {
            part = (value).pdiv(mean);
        }
        let result = part * scale;
        swc1(flag(sp, p, 42), result);
        swc1(flag(sp, p, 22), result);
        p += 1;
    }
    let scale = lwc1(0x3ac8a4);
    let mut p = 0;
    while p < count_limit() {
        let value = matchstat(p, 196);
        swc1(flag(sp, p, 23), value * scale);
        p += 1;
    }
    if count > 0 {
        let mut p = 0;
        loop {
            swc1(flag(sp, p, 24), lwc1(p * 1368 + (g::matchstats + 204)) * lwc1(0x3ac8a8));
            p += 1;
            if p >= count {
                break;
            }
        }
        let limit_at = lwc1(0x3ac8ac);
        let mut p = 0;
        loop {
            if limit_at < StatsGet(53, p) {
                swc1(flag(sp, p, 25), 1.0_f32);
            }
            p += 1;
            if p >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    let count = lw(g::lvnumplayers);
    if count >= 2 {
        let mut p = 0;
        loop {
            swc1(flag(sp, p, 27), (1.0_f32 - lwc1(p * 1368 + (g::matchstats + 232))) * lwc1(0x3ac8b0));
            p += 1;
            if p >= count {
                break;
            }
        }
    }
    if count > 0 {
        let mut p = 0;
        loop {
            if 1.0_f32 <= lwc1(flag(sp, p, 27)) {
                let own = StatsGet(0, p);
                let average = StatsGetAverage(0, count_limit());
                if average < own {
                    swc1(flag(sp, p, 26), 1.0_f32);
                    swc1(flag(sp, p, 41), 1.0_f32);
                }
            }
            p += 1;
            if p >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    let count = lw(g::lvnumplayers);
    if count > 0 {
        let mut p = 0;
        loop {
            swc1(flag(sp, p, 28), lwc1(p * 1368 + (g::matchstats + 220)) * 15.0_f32);
            p += 1;
            if p >= count {
                break;
            }
        }
    }
    let scale = lwc1(0x3ac8b4);
    let mut p = 0;
    while p < count_limit() {
        let rank = matchstat(p, 168);
        let result = (1.0_f32).pdiv(rank) - 1.0_f32 + scale;
        swc1(flag(sp, p, 37), result);
        swc1(flag(sp, p, 29), result);
        p += 1;
    }
    let scale = lwc1(0x3ac8b8);
    let mut p = 0;
    while p < count_limit() {
        let rank = matchstat(p, 168);
        swc1(flag(sp, p, 30), (rank - 1.0_f32) * 0.5_f32 + scale);
        p += 1;
    }
    let mut players = lw(g::lvnumplayers);
    if (lw(g::curgameparms + 80) & 1) == 0 {
        let mut best_value = 0.0_f32;
        let mut best = -1;
        let mut p = 0;
        while p < count_limit() {
            let value = matchstat(p, 160);
            if best_value < value {
                best_value = value;
                best = p;
            }
            p += 1;
        }
        if best >= 0 {
            let team = lw(best * 0x1210 + lw(g::chrinfo) + 24);
            if gameGetRankTeam(team) != 0 {
                swc1(flag(sp, best, 31), 1.0_f32);
            }
        }
        players = lw(g::lvnumplayers);
    }
    let best = StatsGetMax(60, players);
    if best >= 0 {
        if 15.0_f32 <= StatsGet(60, best) {
            swc1(flag(sp, best, 32), 1.0_f32);
        }
    }
    if lw(g::curgameparms + 72) == 2 {
        let lim = if (lw(g::curgameparms + 80) & 32) == 0 { lw(g::lvnumplayers) + lw(g::numactivechars) } else { 4 };
        let best = StatsGetMax(38, lim);
        if best >= 0 {
            if 4.0_f32 <= StatsGet(38, best) {
                swc1(flag(sp, best, 33), 1.0_f32);
            }
        }
    }
    let mode = lw(g::curgameparms + 72);
    if mode == 5 || mode == 2 {
        let lim = lw(g::lvnumplayers) + lw(g::numactivechars);
        let mut highest = -1;
        let mut highest_at = -1;
        let mut p = 0;
        loop {
            let within = if ((mode - 6) as u32) >= (2 as u32) && (lw(g::curgameparms + 80) & 32) == 0 { p < lim } else { p < 4 };
            if !within {
                break;
            }
            let value = matchstat(p, 156);
            if (highest as f32) <= value {
                highest = value as i32;
                highest_at = p;
            }
            p += 1;
        }
        if highest >= 4 {
            swc1(flag(sp, highest_at, 34), 1.0_f32);
        }
        let lim = lw(g::lvnumplayers) + lw(g::numactivechars);
        let on_limit = ((lw(g::curgameparms + 72) - 6) as u32) < (2 as u32);
        let mut lowest = 0xf423f;
        let mut lowest_at = -1;
        let mut p = 0;
        loop {
            let within = if !on_limit && (lw(g::curgameparms + 80) & 32) == 0 { p < lim } else { p < 4 };
            if !within {
                break;
            }
            let value = matchstat(p, 156);
            if value <= lowest as f32 {
                lowest = value as i32;
                lowest_at = p;
            }
            p += 1;
        }
        let own = StatsGet(0, lowest_at);
        let average = StatsGetAverage(0, count_limit());
        if own < average {
            swc1(flag(sp, lowest_at, 35), 1.0_f32);
        }
    }
    let mut p = 0;
    while p < count_limit() {
        if 5.0_f32 <= StatsGet(61, p) {
            swc1(flag(sp, p, 36), 1.0_f32);
        }
        p += 1;
    }
    let mut p = 0;
    while p < count_limit() {
        'next: {
            if !(2e+01_f32 <= StatsGet(0, p)) {
                break 'next;
            }
            if ((lw(g::curgameparms + 72) - 6) as u32) >= (2 as u32) {
                if (lw(g::curgameparms + 80) & 32) == 0 {
                    if (lw(g::lvnumplayers) + lw(g::numactivechars)) < 3 {
                        break 'next;
                    }
                }
            }
            let mut sum = 0;
            let mut q = 0;
            while q < count_limit() {
                let value = StatsGetChar(0, p, q);
                q += 1;
                sum = (sum as f32 + value) as i32;
            }
            let mean;
            if ((lw(g::curgameparms + 72) - 6) as u32) >= (2 as u32) && (lw(g::curgameparms + 80) & 32) == 0 {
                let divisor = lw(g::lvnumplayers) + lw(g::numactivechars);
                if divisor == 0 {
                    trap("break");
                }
                mean = sum.wrapping_div(divisor) as f32;
            } else {
                mean = ((if -1 < sum { sum } else { sum + 3 }) >> 2) as f32;
            }
            if 0.0_f32 < mean {
                let mut q = 0;
                while q < count_limit() {
                    if 3.0_f32 < (StatsGetChar(0, p, q)).pdiv(mean) {
                        swc1(flag(sp, p, 39), 1.0_f32);
                        let hits = StatsGet(1, q);
                        let np = lw(g::lvnumplayers);
                        if 2e+01_f32 < hits {
                            if p < np {
                                swc1(flag(sp, q, 38), 1.0_f32);
                            }
                        }
                    }
                    q += 1;
                }
            }
        }
        p += 1;
    }
    let lim = lw(g::lvnumplayers) + lw(g::numactivechars);
    let on_limit = ((lw(g::curgameparms + 72) - 6) as u32) < (2 as u32);
    let mut p = 0;
    loop {
        let within = if !on_limit && (lw(g::curgameparms + 80) & 32) == 0 { p < lim } else { p < 4 };
        if !within {
            break;
        }
        if !(lwc1(flag(sp, p, 39)) == 0.0_f32) {
            sw(flag(sp, p, 38), 0);
        }
        p += 1;
    }
    let worst = StatsGetMin(66, count_limit());
    if worst >= 0 {
        if (lw(g::lvnumplayers) + lw(g::numactivechars)) >= 4 {
            swc1(flag(sp, worst, 44), 1.0_f32);
        }
    }
    let best = StatsGetMax(67, count_limit());
    let mut players = lw(g::lvnumplayers);
    'b183: {
        if best >= 0 {
            if (players + lw(g::numactivechars)) < 4 {
                break 'b183;
            }
            if 25.0_f32 < StatsGet(67, best) {
                swc1(flag(sp, best, 45), 1.0_f32);
                swc1(flag(sp, best, 46), 1.0_f32);
            }
        }
        players = lw(g::lvnumplayers);
    }
    let best = StatsGetMax(68, players);
    if best >= 0 {
        if 4.0_f32 < StatsGet(68, best) {
            if lwc1(0x3ac8bc) < (StatsGet(68, best)).pdiv(StatsGet(0, best)) {
                swc1(flag(sp, best, 47), 1.0_f32);
            }
        }
    }
    // pick the medals of every player, three each
    let mut players = lw(g::lvnumplayers);
    let mut pass = 0;
    loop {
        if players > 0 {
            let mut p = 0;
            loop {
                sw(p * 12 + (pass << 2) + g::medals, -1);
                if 6e+01_f32 < lw(g::gameframe) as f32 * lwc1(0x3ac8c0) {
                    let slot = p * 12 + (pass << 2) + g::medals;
                    let mut tries = 0;
                    loop {
                        let pick = ((newrnd() as i32 as u32) % (48 as u32)) as i32;
                        tries += 1;
                        if 1.0_f32 <= lwc1(flag(sp, p, pick)) {
                            let mut taken = false;
                            let mut earlier = 0;
                            loop {
                                if earlier != pass {
                                    let mut q = 0;
                                    while q < lw(g::lvnumplayers) {
                                        if lw(q * 12 + (earlier << 2) + g::medals) == pick {
                                            taken = true;
                                        }
                                        q += 1;
                                    }
                                } else {
                                    let mut q = 0;
                                    while q < p {
                                        if lw(q * 12 + (pass << 2) + g::medals) == pick {
                                            taken = true;
                                        }
                                        q += 1;
                                    }
                                }
                                earlier += 1;
                                if pass < earlier {
                                    break;
                                }
                            }
                            if !taken {
                                let own = p * 12 + g::medals;
                                if pass == 1 {
                                    if lw(own) != -1 {
                                        sw(own + 4, pick);
                                    }
                                } else if pass >= 2 {
                                    if pass == 2 {
                                        if lw(own) != -1 {
                                            if lw(own + 4) != -1 {
                                                sw(own + 8, pick);
                                            }
                                        }
                                    }
                                } else {
                                    sw(slot, pick);
                                }
                            }
                        }
                        if lw(slot) != -1 {
                            break;
                        }
                        if !((tries as f32) < 24.0_f32) {
                            break;
                        }
                    }
                    players = lw(g::lvnumplayers);
                }
                'single: {
                    if (players + lw(g::numactivechars)) == 1 {
                        let roll = newrnd();
                        if (((roll as i32 as u32) % (10 as u32)) as i32) == 0 {
                            if pass == 0 {
                                players = lw(g::lvnumplayers);
                                sw(p * 12 + g::medals, 43);
                                break 'single;
                            }
                        }
                        players = lw(g::lvnumplayers);
                        sw(p * 12 + (pass << 2) + g::medals, -1);
                    }
                }
                let medal = lw(p * 12 + (pass << 2) + g::medals);
                if medal != -1 {
                    sw(
                        p * 12 + (pass << 2) + g::awards,
                        lw((lh((medal << 1) + g::medaltext) << 2) + lw((lw(g::cur_language) << 2) + g::languages)),
                    );
                } else if pass != 0 {
                    sw(p * 12 + (pass << 2) + g::awards, 0);
                } else {
                    sw(p * 12 + g::awards, lw(lw((lw(g::cur_language) << 2) + g::languages) + 3080));
                }
                p += 1;
                if p >= players {
                    break;
                }
            }
        }
        pass += 1;
        if pass >= 3 {
            break;
        }
    }
    // the same text is not given twice to one player
    let mut p = 0;
    if players > 0 {
        loop {
            let first = lw(p * 12 + g::awards);
            if first == lw(p * 12 + (g::awards + 4)) {
                if first != 0 {
                    sw(p * 12 + (g::awards + 4), 0);
                }
            }
            let second = lw(p * 12 + (g::awards + 4));
            if second == lw(p * 12 + (g::awards + 8)) {
                if second != 0 {
                    sw(p * 12 + (g::awards + 8), 0);
                }
            }
            let first = lw(p * 12 + g::awards);
            if first == lw(p * 12 + (g::awards + 8)) {
                if first != 0 {
                    sw(p * 12 + (g::awards + 8), 0);
                }
            }
            p += 1;
            if p >= players {
                break;
            }
        }
    }
}
