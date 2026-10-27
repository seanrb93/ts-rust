//! Decals, ammo, preload order, bullet colours, glass, lights and the village story trigger.

use crate::*;

/// Remove decal `n` (240 bytes each): mark it free and take it off the draw lists (@ 0x2a4ac8).
pub fn decalRemoveDecal(n: i32) {
    if n < lw(g::s_MaxNumDecals) && n >= 0 {
        let decal = n * 240 + lw(g::s_Decals);
        sw(decal + 204, -1);
        decalRemoveFromDrawList(decal);
    }
}

/// The tile set option of the map maker was changed: store it in the data part of the file being edited (@ 0x23ab28).
pub fn tileset_onchange(item: i32) {
    let file = lw(g::mmtempbuffer);
    if lw(file) == 0x1324626 {
        let info = mmfileGetInfo(file);
        sh(info + 8, lhu(item));
    }
}

/// Give a character `amount` more ammo of type `ammo`, but not more than the most it may have counting what is in its clips (@ 0x294ac8).
pub fn gunAddAmmo(chr: i32, ammo: i32, amount: i32) {
    let in_clips = gunGetGunAmmo(chr, ammo);
    let store = chr + 620 + (ammo << 2);
    let total = lw(store) + amount;
    sw(store, total);
    let most = lw((ammo << 3) + g::ammoinfo);
    if most < total + in_clips {
        sw(store, most - in_clips);
    }
}

/// Add a texture to the second preload pass: a record of 16 bytes from the table at 0x1fb7b40, kept sorted (@ 0x21c8d0).
pub fn preloadPass2AddTexNum(a: i32, b: i32, c: i32) {
    let n = lw(0x3afedc);
    let rec = (n << 4) + preload_pass2_table();
    sw(0x3afedc, n + 1);
    sw(rec, a);
    sw(rec + 4, b);
    sw(rec + 8, c);
    sw(rec + 12, 0);
    preloadPass2Insert(rec);
}

/// Insert a preload record into the list sorted by its word at +8 (link at +4); the head is at 0x3afee0 (@ 0x21c240).
pub fn preloadInsert(item: i32) {
    let head = lw(0x3afee0);
    if head == 0 {
        sw(0x3afee0, item);
        return;
    }
    let key = lw(item + 8);
    if lw(head + 8) < key {
        let mut at = head;
        let mut next = lw(at + 4);
        while next != 0 && lw(next + 8) < lw(item + 8) {
            at = next;
            next = lw(at + 4);
        }
        sw(item + 4, next);
        sw(at + 4, item);
    } else {
        sw(item + 4, head);
        sw(0x3afee0, item);
    }
}

/// The village story: once a player is in the game, the portal between rooms 18 and 21 opens, door 1 opens and the link between two pads is used (@ 0x2cc180).
pub fn enemyAIVillageHandler1() {
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
            bgPortalEnableFromRooms(18, 21);
            doorOpen(1);
            linkEnable(padGetLinkNum(1324, 1127));
            sw(g::story_state1, 1);
        }
    }
}

/// The pad nearest to `pos` (three floats) that has any of the flags `flags`, is not in hall `hall`, and is less than 3.5 away (and nearer than the distance at 0x3aeb24); -1 if none (@ 0x264e90).
pub fn findTwinPlugPad(pos: i32, flags: i32, hall: i32) -> i32 {
    let mut best = -1;
    let mut best_dist = lwc1(0x3aeb24);
    for i in 0..lw(g::numpads) {
        let pad = i * 28 + lw(g::levelpads);
        if lw(pad + 4) & flags == 0 || lh(pad + 8) == hall {
            continue;
        }
        let dx = ps2float::sub(lwc1(pad + 12), lwc1(pos));
        let dy = ps2float::sub(lwc1(pad + 16), lwc1(pos + 4));
        let dz = ps2float::sub(lwc1(pad + 20), lwc1(pos + 8));
        let dist = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
        if dist < best_dist && dist < 3.5 {
            best_dist = dist;
            best = i;
        }
    }
    best
}

/// How many panes of glass in the level are whole: each room has a list of pointers to glass records (ended by 0) whose value at +0 is negative when broken (@ 0x25a2f8).
pub fn bgAmountGlassLeft() -> i32 {
    let mut whole = 0;
    for room in 0..lw(g::bgNumRooms) {
        let list = lw(room * 44 + lw(g::levelrooms) + 60);
        if list != 0 {
            let mut k = 0;
            while lw(list + 4 * k) != 0 {
                if 0.0 <= lwc1(lw(list + 4 * k)) {
                    whole += 1;
                }
                k += 1;
            }
        }
    }
    whole
}

/// Add the lights of the model's parts to the list of the level's lights (300 at most): each 80-byte part has a count of lights at +6 and their address at +16 (152 bytes each) (@ 0x25a720).
pub fn bglightAdd(model: i32) {
    let end = lw(model);
    let mut part = end - lw(end) * 80;
    while (part as u32) < (end as u32) {
        let count = lb(part + 6);
        for k in 0..count.max(0) {
            let light = lw(part + 16) + k * 152;
            let n = lw(g::numbglights);
            if n < 300 {
                sw((n << 2) + 0x1fc5f18, light);
                sw(g::numbglights, n + 1);
            }
        }
        part += 80;
    }
}

/// The colour (three floats) of a bullet type's tracer: yellowish, red, green, blue, or a colour of the game's (@ 0x28d038).
pub fn bulletGetBulletColour(kind: i32, rgb: i32) {
    let v = kind - 2;
    let (r, g_, b) = if (v as u32) >= 17 {
        sw(rgb + 8, 0);
        sw(rgb, 0);
        sw(rgb + 4, 0);
        return;
    } else {
        match v {
            0..=2 => (1.0, 1.0, lwc1(0x3ad29c)),
            3 => (1.0, 0.0, 0.0),
            4 => (0.0, 1.0, 0.0),
            5 => (0.0, 0.0, 1.0),
            6 | 7 => (1.0, 1.0, 0.5),
            _ => (lwc1(0x3ad2a0), lwc1(0x3ad2a0), lwc1(0x3ad2a4)),
        }
    };
    swc1(rgb + 4, g_);
    swc1(rgb + 8, b);
    swc1(rgb, r);
}

/// Whether a player has camped (the timer at +880 is over 4) at the pad `pad`'s spot: the player's list of recent pads (from +912, its length at +908) holds the pad's number at +10 (@ 0x2cd8c0).
pub fn campingPad(pad: i32) -> i32 {
    for i in 0..lw(g::lvnumplayers) {
        let player = i * 1820 + lw(g::players);
        if 4.0_f32 < lwc1(player + 880) {
            for k in 0..lw(player + 908) {
                if lw(player + (k << 2) + 912) == lh(pad * 28 + lw(g::levelpads) + 10) {
                    return 1;
                }
            }
        }
    }
    0
}
