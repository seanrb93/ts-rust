//! Route tables of the bots, links by pad, kills, the plane height and more.

use crate::*;

/// Note a base (a prop of type 201 or 202) for Last Stand: the base list (8 bytes each, 10 at most) gets the prop and the pad nearest to it (@ 0x2c7e58).
pub fn enemyAddToPropList(prop: i32) {
    let n = lw(g::numBases);
    if ((lw(prop + 4) - 201) as u32) < 2 {
        let bases = g::bases;
        sw(bases + (n << 3) + 4, prop);
        let pad = closestpad(prop + 48, 0.0);
        let n = lw(g::numBases);
        sw((n << 3) + bases, pad);
        if n < 10 {
            sw(g::numBases, n + 1);
        }
    }
}

/// Put a player's gun into its resting state: clear its animation and blend words, set the state words and reset the reload (@ 0x291138).
pub fn resetPlayerGun(player: i32, gun: i32) {
    sw(player + 4, 0);
    let model = lw(player + 12);
    for at in [68, 72, 76, 60, 64] {
        sw(player + at, 0);
    }
    sw(model + 88, 0);
    sw(model + 72, 0);
    sw(model + 76, 0);
    for at in [80, 92, 96, 100, 116, 120, 124, 136, 84, 88, 140, 144, 188] {
        sw(player + at, 0);
    }
    sw(player, 5);
    sw(player + 156, 1);
    resetPlayerGunReload(player, gun);
    sw(player + 148, 1000);
    sw(gun + 68, 1000);
}

/// The height of the plane through the three points `a`, `b`, `c` at the horizontal position (`x`, `z`) (@ 0x27c4a8).
pub fn calcfloorheight(a: i32, b: i32, c: i32, x: f32, z: f32) -> f32 {
    let (ax, ay, az) = (lwc1(a), lwc1(a + 4), lwc1(a + 8));
    let mut ab = [0.0_f32; 3];
    let mut bc = [0.0_f32; 3];
    for i in 0..3 {
        let (pa, pb, pc) = (lwc1(a + 4 * i), lwc1(b + 4 * i), lwc1(c + 4 * i));
        ab[i as usize] = pb - pa;
        bc[i as usize] = pc - pb;
    }
    let nx = ab[1] * bc[2] - bc[1] * ab[2];
    let ny = ab[2] * bc[0] - bc[2] * ab[0];
    let nz = ab[0] * bc[1] - bc[0] * ab[1];
    ((nx * ax + ny * ay + nz * az - nx * x - nz * z)).pdiv(ny)
}

/// Whether the score of the challenge just played is a new best, by the way its condition (kind at +12: 0-1 lower is better, 2 equal, 3-4 higher is better) compares (@ 0x21f088).
pub fn challengeIsBestScore() -> i32 {
    let stats = statsGetPlayer(0);
    let params = g::curgameparms;
    let entry = stats + ((lw(params + 68) << 3) + 1400);
    if lw(params + 80) & 0x10000000 != 0 {
        let score = challengeGetScore();
        if lw(entry) == 0 {
            return 1;
        }
        let best = lw(entry + 4);
        let kind = lw(challengeGetScoreCondition(lw(params + 68)) + 12);
        if (kind as u32) < 5 {
            return match kind {
                0 | 1 => (score < best) as i32,
                2 => (score == best) as i32,
                _ => (best < score) as i32,
            };
        }
    }
    0
}

/// The Chinese story: with a player in the game, on difficulty 0 the portal between rooms 30 and 33 opens and door 1 unlocks; on difficulty 1
/// the portal between 36 and 35 and door 2 (@ 0x2cbf00).
pub fn enemyAIChineseHandler1() {
    if lw(g::story_state1) == 0 {
        let players = lw(g::lvnumplayers);
        let chr_info = lw(g::chrinfo);
        let mut any = false;
        for i in 0..players {
            if lw(i * 0x1210 + chr_info + 484) != 0 {
                any = true;
            }
        }
        if any {
            let params = g::curgameparms;
            let mut level = lw(params + 76);
            if level == 0 {
                bgPortalEnableFromRooms(30, 33);
                doorUnlock(1);
                level = lw(params + 76);
            }
            if level != 1 {
                sw(g::story_state1, 1);
            } else {
                bgPortalEnableFromRooms(36, 35);
                doorUnlock(2);
                sw(g::story_state1, 1);
            }
        }
    }
}

/// The link (address) in use between the pads at addresses `a` and `b`, either way round, or 0 (@ 0x2648e0).
pub fn findlink(a: i32, b: i32) -> i32 {
    let pads = lw(g::levelpads);
    let number_a = (a - pads).wrapping_mul(-0x49249249) >> 2;
    let number_b = (b - pads).wrapping_mul(-0x49249249) >> 2;
    let links = lw(g::levellinks);
    for i in 0..lw(g::numlinks) {
        let link = i * 20 + links;
        if lw(link) & 1 != 0 {
            let from = lw(link + 4);
            if from == number_a && lw(link + 8) == number_b {
                return link;
            }
            if from == number_b && lw(link + 8) == number_a {
                return link;
            }
        }
    }
    0
}

/// Put a route entry (16 bytes: next at +0, three halfwords at +2, +4, +6, a float at +8 and its cost at +12) from the free list into the list that starts at the word `head_at`, sorted by cost; `free_at` holds the first free entry.
fn add_route(table: i32, free_at: i32, head_at: i32, a: i32, b: i32, c: i32, f12: f32, cost: f32) {
    let free = lw(free_at);
    let head = lw(head_at);
    let rec = (free << 4) + table;
    sw(free_at, lh(rec));
    if head >= 0 {
        let first = (head << 4) + table;
        if !(cost < lwc1(first + 12)) {
            let mut at = first;
            loop {
                let next = lh(at);
                if next < 0 {
                    sh(at, free);
                    sh(rec, next);
                    break;
                }
                let next_rec = (next << 4) + table;
                if !(cost < lwc1(next_rec + 12)) {
                    at = next_rec;
                    continue;
                }
                sh(at, free);
                sh(rec, next);
                break;
            }
        } else {
            sw(head_at, free);
            sh(rec, head);
        }
    } else {
        sw(head_at, free);
        sh(rec, -1);
    }
    swc1(rec + 12, cost);
    sh(rec + 2, a);
    sh(rec + 4, b);
    sh(rec + 6, c);
    swc1(rec + 8, f12);
}

/// Add an entry to the route table of the first route finder (@ 0x265f08).
pub fn hr_addroute(a: i32, b: i32, c: i32, f12: f32, cost: f32) {
    add_route(0x1fc6490, 0x3b00c0, 0x3b00b0, a, b, c, f12, cost);
}

/// Add an entry to the route table of the second route finder (@ 0x2665c8).
pub fn r_addroute(a: i32, b: i32, c: i32, f12: f32, cost: f32) {
    add_route(0x1fca310, 0x3b00e0, 0x3b00d0, a, b, c, f12, cost);
}

/// A character's score: the kills of the others' teams recorded in its table (halfwords from +32), less its own deaths by the others when flag 16 is set (@ 0x288f68).
pub fn chrCountKills(chr: i32) -> i32 {
    let n = lw(g::lvnumplayers) + lw(g::numactivechars);
    let mut kills = 0;
    if n > 0 {
        let chr_info = lw(g::chrinfo);
        let team = lw(chr + 24);
        for i in 0..n {
            if lw(i * 0x1210 + chr_info + 24) != team {
                kills += lh(chr + (i << 1) + 32);
            }
        }
    }
    if lw(g::curgameparms + 80) & 16 != 0 && n > 0 {
        let number = lw(chr);
        let chr_info = lw(g::chrinfo);
        for i in 0..n {
            kills -= lh(i * 0x1210 + chr_info + (number << 1) + 32);
        }
    }
    kills
}

/// Add a point (x, y, colour 0xrrggbbaa) to the display list as a GIF packet of 64 bytes (@ 0x2b8768).
pub fn dlDrawPoint(x: i32, y: i32, rgba: i32) {
    let packet = memdbAlloc(64);
    sb(lw(g::gfxtag) + 3, 48);
    sw(packet, 0x6c038000);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    let words = [
        0x8001,
        0x20204000,
        81,
        0,
        ((rgba as u32) >> 24) as i32,
        rgba >> 16 & 255,
        rgba >> 8 & 255,
        rgba & 255,
        (x << 4) + 0x6c00,
        (y << 4) + 0x7900,
        -1,
        0,
        0x140007fc,
        0x11000000,
        0,
    ];
    for (i, w) in words.iter().enumerate() {
        sw(packet + 4 + 4 * i as i32, *w);
    }
    sh(tag, 4);
    sw(g::gfxtag, tag + 16);
}
