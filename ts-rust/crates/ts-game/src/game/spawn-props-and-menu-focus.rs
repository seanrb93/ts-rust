//! Spawn props, menu focus, the bullet node pool, links and more.

use crate::*;

/// Counted the barrels and did nothing with the count (@ 0x2756b0).
pub fn propAllBallelsBlownUp() {
    propNumBallelsNotBlownUp();
}

/// Looked at the challenge's data and printed nothing (@ 0x21f780).
pub fn DisplayChallengeDebug() {}

/// Whether the rank is shown on the HUD: in a challenge, not for challenges 0, 1 and 17 and always for 2 and 26, else when the challenge has a
/// score condition; in other games not in mode 7 (@ 0x21cc30).
pub fn gameModeDisplayRank() -> i32 {
    let params = g::curgameparms;
    if lw(params + 80) & 32 == 0 {
        return (lw(params + 72) != 7) as i32;
    }
    let setup = challengeGetSetup();
    let challenge = lw(params + 68);
    if (challenge as u32) < 27 {
        match challenge {
            0 | 1 | 17 => return 0,
            2 | 26 => return 1,
            _ => {}
        }
    }
    (-1 < lw(lw(setup + 48) + 16)) as i32
}

/// What a map costs in the map maker: each object's cost (+8) and, for objects of kind 1, the number of items on them (@ 0x248c20).
pub fn mmMapCost() -> i32 {
    let mut cost = 0;
    if lw(g::nummapobjects) != 0 {
        let mut object = lw(g::mapobjects);
        while object != 0 {
            cost += lw(object + 8);
            if lw(object) == 1 {
                cost += mmItemListCount(lw(object + 52), -1);
            }
            object = lw(object + 56);
        }
    }
    cost
}

/// Whether a player stands on the pad with external reference `extref` (@ 0x2cbb40).
pub fn enemyStoryPlayerAtPad(extref: i32) -> i32 {
    let mut i = 0;
    while i < lw(g::lvnumplayers) {
        let info = i * 0x1210 + lw(g::chrinfo);
        let pad = padNumFromExtref(extref);
        if lw(info + 680) == pad {
            return 1;
        }
        i += 1;
    }
    0
}

/// A free link record (20 bytes, first word 0 when free) of the bots' map: the first free one, or a new one at the end (@ 0x264868).
pub fn newlink() -> i32 {
    let count = lw(g::numlinks);
    let links = lw(g::levellinks);
    let mut free = -1;
    for i in 0..count.max(0) {
        if lw(i * 20 + links) == 0 {
            free = i;
            break;
        }
    }
    if free < 0 {
        free = count;
        sw(g::numlinks, free + 1);
    }
    free * 20 + links
}

/// The pad where a drop for this character's team is made, or -1; the character takes that drop (its flags 128 set, 64 cleared) (@ 0x2c1408).
pub fn selectDrop(chr: i32) -> i32 {
    let count = lw(g::numPickupDrops);
    let data = lw(chr + 352);
    if count > 0 {
        let team = lw(data + 24);
        for i in 0..count {
            let entry = g::dropspawn + i * 8;
            if lw(lw(lw(entry + 4) + 352) + 16) == team {
                sw(data + 2704, (lw(data + 2704) | 128) & -65);
                return lw(entry);
            }
        }
    }
    -1
}

/// Whether the point is behind a portal as seen from room `room`: the dot product of the vector from the portal's corner (+24) with its normal (+8)
/// is negative, or positive if the portal's room is `room` (@ 0x257860).
pub fn bgPortalBackFaceTest(room: i32, point: i32, portal: i32) -> i32 {
    let dy = ps2float::sub(lwc1(point + 4), lwc1(portal + 28));
    let dx = ps2float::sub(lwc1(point), lwc1(portal + 24));
    let dz = ps2float::sub(lwc1(point + 8), lwc1(portal + 32));
    let dot = ps2float::add(ps2float::add(ps2float::mul(dx, lwc1(portal + 8)), ps2float::mul(dy, lwc1(portal + 12))), ps2float::mul(dz, lwc1(portal + 16)));
    if lw(portal) != room {
        (dot < 0.0) as i32
    } else {
        (0.0 < dot) as i32
    }
}

/// A blast spawn prop grows: turns at 10 degrees a tick and each of its parts' matrices is a scale from the prop's data (+48, +44) (@ 0x27bcb8).
pub fn blastspawnPropTick(prop: i32) {
    let data = lw(prop + 352);
    let mut model = lw(prop + 32);
    let count = lw(lw(model));
    swc1(prop + 76, ps2float::add(lwc1(prop + 76), ps2float::mul(lwc1(g::updateframesf), 10.0)));
    if count > 0 {
        let mut i = 0;
        loop {
            let s = lwc1(data + 48);
            let y = lwc1(data + 44);
            matrixScaleSet(lw(model + 4) + (i << 6), s, y, s);
            i += 1;
            model = lw(prop + 32);
            if i >= lw(lw(model)) {
                break;
            }
        }
    }
}

/// A ring spawn prop: its parts scaled by a constant of the game and the prop's +44; turns at 6 degrees a tick (@ 0x27bd58).
pub fn ringspawnPropTick(prop: i32) {
    let model = lw(prop + 32);
    let data = lw(prop + 352);
    if lw(lw(model)) > 0 {
        let s = lwc1(0x3acf98);
        let mut model = model;
        let mut i = 0;
        loop {
            matrixScaleSet(lw(model + 4) + (i << 6), s, lwc1(data + 44), s);
            i += 1;
            model = lw(prop + 32);
            if i >= lw(lw(model)) {
                break;
            }
        }
    }
    swc1(prop + 76, ps2float::add(lwc1(prop + 76), ps2float::mul(lwc1(g::updateframesf), 6.0)));
}

/// The level chooser (frontred): d-pad down / up (0x4000 / 0x1000) move through the levels, cross or start (2112) pick one (@ 0x22ff48).
pub fn frontredTick() {
    let levels = lw(0x3aff1c);
    if joyNewBut(0) & 0x4000 != 0 {
        let cur = lw(g::curselect);
        if cur < levels - 1 {
            sw(g::curselect, cur + 1);
        }
    }
    let up = joyNewBut(0) & 0x1000 != 0;
    let cur = lw(g::curselect);
    if up && cur > 0 {
        sw(g::curselect, cur - 1);
    }
    let pick = joyNewBut(0) & 2112 != 0;
    let cur = lw(g::curselect);
    if pick {
        sw(g::lvend, lw((cur << 3) + (g::levels + 4)));
    }
}

/// Swap the menu being shown with the debug menu: the 10 words of the stacks, the stack positions and the current menus; the mode flips (@ 0x20c940).
pub fn toggledebug() {
    let pos = lw(g::menu_stackpos);
    let debug_pos = lw(g::debugmenu_stackpos);
    let cur = lw(g::curmenu);
    let debug_cur = lw(g::debugcurmenu);
    let mode = lw(g::debugmode);
    for i in 0..10 {
        let a = g::debugmenustack + 4 * i;
        let b = g::menustack + 4 * i;
        let kept = lw(a);
        sw(a, lw(b));
        sw(b, kept);
    }
    sw(g::menu_stackpos, debug_pos);
    sw(g::debugmenu_stackpos, pos);
    sw(g::curmenu, debug_cur);
    sw(g::debugcurmenu, cur);
    sw(g::debugmode, 1 - mode);
}

/// Vibration mode option: 0 off, 1 -> bit 64, 2 -> bit 32, 3 -> both (@ 0x20f870).
pub fn vibmode_onchange(item: i32) {
    let settings = lw(lw(g::currentplayer) + 24);
    let mode = lw(item);
    let flags = lhu(settings + 2) & 0xff9f;
    sh(settings + 2, flags);
    let bits = match mode {
        1 => 64,
        2 => 32,
        3 => 96,
        _ => 0,
    };
    sh(settings + 2, flags | bits);
}

/// Set up the 50 nodes of the bullet pool (276 bytes each) as two lists: nodes 0 to 19 and 20 to 49, each linked in order (@ 0x289200).
pub fn bulletRestart() {
    let nodes = lw(0x3b0138);
    for i in 0..50 {
        let node = nodes + 276 * i;
        sw(node + 8, i - 1);
        for at in [0, 96, 144, 140, 112, 116, 204, 208] {
            sw(node + at, 0);
        }
        sw(node + 4, i + 1);
        sw(node + 108, -1);
    }
    sw(0x3b0140, 0);
    sw(0x3b0148, 19);
    sw(nodes + 8, -1);
    sw(lw(0x3b0148) * 276 + nodes + 4, -1);
    sw(0x3b014c, 49);
    sw(0x3b0144, 20);
    sw(nodes + 0x1598, -1);
    sw(lw(0x3b014c) * 276 + nodes + 4, -1);
}

/// Whether an enemy is making progress: when its timer (+2896) has run to 0.75 and it has moved less than a limit from where it was, it is stuck (flag 1, and a counter at +3000 that counts while it is in state 512) (@ 0x2bfc98).
pub fn checkProgress(chr: i32) {
    let data = lw(chr + 352);
    let waiting = lwc1(data + 2896) < 0.75;
    let flags = lw(data + 2704) & -2;
    sw(data + 2704, flags);
    if !waiting {
        let dx = ps2float::sub(lwc1(chr + 48), lwc1(data + 2988));
        let dy = ps2float::sub(lwc1(chr + 52), lwc1(data + 2992));
        let dz = ps2float::sub(lwc1(chr + 56), lwc1(data + 2996));
        if !(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) < lwc1(0x3ad88c)) || lw(data + 2708) & 8 != 0 {
            sw(data + 2896, 0);
        } else {
            if lw(data + 2716) == 512 {
                let count = lw(data + 3000);
                sw(data + 2704, flags | 1);
                sw(data + 3000, count + 1);
            } else {
                sw(data + 3000, 0);
                sw(data + 2704, flags | 1);
            }
            sw(data + 2896, 0);
        }
        swc1(data + 2988, lwc1(chr + 48));
        swc1(data + 2992, lwc1(chr + 52));
        swc1(data + 2996, lwc1(chr + 56));
    }
}

/// Move the focus of the current menu to the next item that can take focus, wrapping to the first (@ 0x20c9a8).
pub fn focusnext() {
    let menu = lw(g::curmenu);
    let (mut first, mut next, mut seen) = (0, 0, 0);
    let mut i = 0;
    if lbu(menu + 4) != 0 {
        loop {
            let item = menu + 16 + 32 * i;
            if lbu(item + 20) & 1 != 0 {
                if first == 0 {
                    first = item;
                }
                if next == 0 {
                    next = if seen == 0 { 0 } else { item };
                }
                if item == lw(menu + 8) {
                    seen = 1;
                }
            }
            i += 1;
            if i >= lbu(menu + 4) {
                break;
            }
        }
    }
    sw(menu + 8, if next != 0 { next } else { first });
}
