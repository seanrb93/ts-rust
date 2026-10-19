//! The coop target, controller axes, prop searches, reload reset, editing, the tomb story trigger, quaternion matrices and portal visibility.

use super::controllers::controller;
use super::stats::stats_slot_in_play;
use crate::*;

/// The target of a character in modes where two players are on the same side: the first player's (or second's) character when the enemy was told to
/// follow (flag 0x4000, cleared here), else its own target (@ 0x2cda28).
pub fn checkCoopTarget(chr: i32) -> i32 {
    let mode = lw(g::curgameparms + 72);
    let data = lw(chr + 352);
    let target = lw(data + 2788);
    if mode != 7 && (mode != 6 || lw(g::lvlevelnum) != 8) {
        return target;
    }
    if target != 0 && lw(target + 8) == 8 && lw(data + 2772) == 0 {
        let flags = lw(data + 2704);
        if flags & 0x4000 != 0 {
            let info = lw(g::chrinfo);
            if lw(lw(target + 352)) != 0 {
                let players = lw(g::players);
                if lw(info + 2708) != 32 {
                    let result = lw(players + 384);
                    sw(data + 2704, flags & -0x4001);
                    return result;
                }
            } else {
                let players = lw(g::players);
                if lw(info + 0x1ca4) != 32 {
                    let result = lw(players + 2204);
                    sw(data + 2704, flags & -0x4001);
                    return result;
                }
            }
        }
    }
    target
}

/// A stick axis of a player's controller (0 left x, 1 left y, 2 right x, 3 right y), with the player mapped to its slot (@ 0x203bb8).
pub fn joyGetAxis(player: i32, axis: i32) -> f32 {
    let c = controller(player);
    match axis {
        0 => joyFixVal(lw(c + 300), lwc1(c + 316)),
        1 => -joyFixVal(lw(c + 308), lwc1(c + 316)),
        2 => joyFixVal(lw(c + 304), lwc1(c + 316)),
        3 => -joyFixVal(lw(c + 312), lwc1(c + 316)),
        _ => 0.0,
    }
}

/// The next prop (592 bytes each) after `from` (or the first) whose type word (+8) is `kind`, or when `kind` is -1 the next that is not of type 2;
/// the search goes round the list up to 4 times and returns 0 when it gets back to `from` (@ 0x270358).
pub fn propGetNext(from: i32, kind: i32) -> i32 {
    let props = lw(g::levelprops);
    let mut at = (if from != 0 { from } else { props }) + 592;
    let last = props + (lw(g::num_props) * 592 - 592);
    let mut rounds = 0;
    if (last as u32) < (at as u32) {
        at = props;
        rounds = 1;
    }
    loop {
        if at == from {
            return 0;
        }
        let t = lw(at + 8);
        if t == kind {
            return at;
        }
        if kind != -1 {
            at += 592;
        } else {
            if t != 2 {
                return at;
            }
            at += 592;
        }
        if (last as u32) < (at as u32) {
            rounds += 1;
            at = props;
            if rounds >= 4 {
                return 0;
            }
        }
    }
}

/// Reset a player's gun reload state: clear the reload words; for guns 11 and 13 also the two at +84, and set the model flags of the first
/// clips (flag 2 for one, then 256, 128, ... for the others) (@ 0x291070).
pub fn resetPlayerGunReload(player: i32, gun: i32) {
    let kind = lw(gun);
    sw(player + 128, 0);
    sw(player + 132, 0);
    if kind != 11 && kind != 13 {
        sw(player + 184, 0);
    } else {
        sw(player + 84, 0);
        sw(player + 88, 0);
        sw(player + 184, 0);
    }
    let kind = lw(gun);
    if kind == 11 || kind == 13 {
        let clips = lw(gun + (lw(kind * 400 + (g::guns + 16)) << 2) + 4);
        if clips > 0 {
            sh(lw(lw(player + 12) + 32) + 292, 2);
        }
        let limit = clips - 1;
        if limit > 0 {
            let model = lw(lw(player + 12) + 32);
            for i in 0..limit {
                sh(model + 292, lhu(model + 292) | (256 >> (i & 31)));
            }
        }
    }
    sw(player + 160, 0);
    sw(player + 152, -1);
    sw(player + 164, 0);
}

/// Read a statistic of `slot` about another character `other` (@ 0x224020).
pub fn StatsGetChar(stat: i32, slot: i32, other: i32) -> f32 {
    if stats_slot_in_play(slot) && stats_slot_in_play(other) {
        lwc1((stat << 2) + other * 24 + slot * 1368 + (g::matchstats + 936))
    } else {
        0.0
    }
}

/// The pad nearest to `pos` (three floats) and nearer than `radius` (+ a constant) squared and than the distance at 0x3aeb30; -1 if none (@ 0x266b90).
pub fn closestpad(pos: i32, radius: f32) -> i32 {
    let mut best_dist = lwc1(0x3aeb30);
    let mut best = -1;
    let limit = if !(radius <= 0.0) { radius * radius + lwc1(0x3acbc0) } else { 0.0 };
    let count = lw(g::numpads);
    if count > 0 {
        let (z, x, y) = (lwc1(pos + 8), lwc1(pos), lwc1(pos + 4));
        let pads = lw(g::levelpads);
        for i in 0..count {
            let pad = i * 28 + pads;
            let dx = lwc1(pad + 12) - x;
            let dy = lwc1(pad + 16) - y;
            let dz = lwc1(pad + 20) - z;
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 < best_dist && !(limit <= d2) {
                best_dist = d2;
                best = i;
            }
        }
    }
    best
}

/// The prop editor's rotation: with a button newly pressed or the button 8 held, the left / right buttons (0x2000 / 0x8000) turn the angle at `angle` by a degree; kept in 0 to 360 (@ 0x265dc0).
pub fn editRotate(angle: i32) {
    let held = joyNewBut(0) != 0 || joyBut(0) & 8 != 0;
    if held {
        if joyBut(0) & 0x2000 != 0 {
            swc1(angle, lwc1(angle) + 1.0);
        }
        if joyBut(0) & 0x8000 != 0 {
            swc1(angle, lwc1(angle) - 1.0);
        }
    }
    let mut a = lwc1(angle);
    if a < 0.0 {
        a += 360.0;
        swc1(angle, a);
    }
    if 360.0 <= a {
        swc1(angle, a - 360.0);
    }
}

/// The tomb story: when a player is in the game, on difficulty 2 two doors open and their pads are linked; the state is set (@ 0x2cbe50).
pub fn enemyAITombHandler1() {
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
            if lw(g::curgameparms + 76) == 2 {
                doorOpen(6);
                linkEnable(padGetLinkNum(1169, 1505));
                doorOpen(1);
                linkEnable(padGetLinkNum(1029, 1428));
            }
            sw(g::story_state1, 1);
        }
    }
}

/// The rotation matrix of the quaternion `q` (x, y, z, w; it need not be unit length) in `m` (@ 0x2b3f90).
pub fn quaternionToMatrix(m: i32, q: i32) {
    let (x, y, z, w) = (lwc1(q), lwc1(q + 4), lwc1(q + 8), lwc1(q + 12));
    let one = 1.0_f32;
    swc1(m + 60, one);
    for at in [48, 44, 28, 12, 56, 52] {
        swc1(m + at, 0.0);
    }
    let s = 2.0_f32 / (x * x + y * y + z * z + w * w);
    let zs = z * s;
    let ys = y * s;
    let xs = x * s;
    let zz = z * zs;
    let yy = y * ys;
    let xx = x * xs;
    let wz = w * zs;
    let xz = x * zs;
    let wx = w * xs;
    let yz = y * zs;
    let wy = w * ys;
    let xy = x * ys;
    swc1(m + 32, xz + wy);
    swc1(m + 16, xy - wz);
    swc1(m + 36, yz - wx);
    swc1(m, one - (yy + zz));
    swc1(m + 20, one - (xx + zz));
    swc1(m + 40, one - (xx + yy));
    swc1(m + 4, xy + wz);
    swc1(m + 8, xz - wy);
    swc1(m + 24, yz + wx);
}

/// Work out the back face test of the portals of `room` and, through the ones that face the player and are open, of the rooms beyond them (once a room) (@ 0x2578f0).
pub fn bgPortalCalcBackFace(room: i32) {
    let list = lw(room * 44 + lw(g::levelrooms) + 4);
    let seen = (room << 2) + lw(0x3b0084);
    let count = lw(list);
    if lw(seen) == 0 {
        sw(seen, 1);
        if count > 0 {
            for k in 0..count {
                let number = lw(list + 4 + 4 * k);
                let portal = lw((number << 2) + lw(g::levelportals));
                let mut other = lw(portal);
                let data = lw(g::portaldata) + number * 48;
                if room == other {
                    other = lw(portal + 4);
                }
                if bgPortalBackFaceTest(room, lw(g::currentplayer) + 884, portal) != 0 {
                } else if lhu(portal + 22) & 1 != 0 {
                } else {
                    bgPortalCalcBackFace(other);
                    sw(data + 40, 0);
                }
            }
        }
    }
}
