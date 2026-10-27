//! Game mode options, death and bag messages, the pivot and stand place of a remote prop, the direction of a swing, a new map maker tile, and the preload list.

use crate::*;

/// Set the game variables for the game mode `mode` from its record of 8 words (32 bytes at 0x1fc2c98): team play, friendly fire, radar, time limit, kill limit, score limit,
/// scoring method and one shot kills (@ 0x238f40).
pub fn getGameModeOptions(mode: i32) {
    let record = (mode << 5) + 0x1fc2c98;
    gvSetValue(g::gv_teamplay, lw(record));
    gvSetValue(g::gv_friendfire, lw(record + 4));
    gvSetValue(g::gv_dispradar, lw(record + 8));
    gvSetValue(g::gv_timelimit, lw(record + 12));
    gvSetValue(g::gv_killlimit, lw(record + 16));
    gvSetValue(g::gv_scorelimit, lw(record + 20));
    gvSetValue(g::gv_scoremethod, lw(record + 24));
    gvSetValue(g::gv_oneshotkill, lw(record + 28));
}

/// When the player `player`'s character was killed by a character of a player (kind 8), say so to the player ("killed by" and the killer's name, or nothing when it killed
/// itself), and clear the killer's own message (@ 0x2803c8).
pub fn playerDeathMessage(player: i32) {
    let state = lw(lw(player + 384) + 352);
    let killer = lw(state + 256);
    if killer != 0 && lw(killer + 8) == 8 {
        let killer_state = lw(killer + 352);
        let language = lw(g::cur_language);
        if lw(killer_state) == lw(state) {
            playerMessage(player, 0);
        } else {
            let name = lw(killer_state + 16);
            let format = lw(lw((language << 2) + g::languages) + 0x11d4);
            let text = vafmt(&[0x3af9f0, format as i64, name as i64]);
            playerMessage(player, text);
        }
        let own = lw(player + 384);
        hudMessage(lw(lw(own + 352)), 0);
    }
}

/// A bag was secured by the player of the prop `bag`: the statistic 38 goes up, the players of its team are told ("bag secured") except the one who did it, who gets the other
/// message (@ 0x26b430).
pub fn bagSecuredMessage(bag: i32) {
    let mut i = 0;
    StatsAdd(38, lw(bag), 1.0_f32);
    let mut players = lw(g::lvnumplayers);
    let mut owner = lw(bag);
    if players > 0 {
        loop {
            let record = i * 1820 + lw(g::players);
            if lw(lw(lw(record + 384) + 352) + 24) == lw(bag + 24) {
                let language = lw(g::cur_language);
                if i != owner {
                    playerMessage(record, lw(lw((language << 2) + g::languages) + 0x11fc));
                    players = lw(g::lvnumplayers);
                    owner = lw(bag);
                }
            }
            i += 1;
            if i >= players {
                break;
            }
        }
    }
    if !(owner < players) {
        return;
    }
    let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x1200);
    playerMessage(lw(g::players) + owner * 1820, text);
}

/// The place of the pivot of the remote prop `prop`: its pivot offset turned by the prop's direction (+76) and added to its position (@ 0x269508).
pub fn remoteGetPivotePos(prop: i32, out: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let offset = remoteGetPivoteOffset(prop);
    let angle = ps2float::mul(lwc1(prop + 76), lwc1(0x3acbd0)).pdiv(180.0_f32);
    swc1(sp, lwc1(offset));
    swc1(sp + 4, lwc1(offset + 4));
    swc1(sp + 8, lwc1(offset + 8));
    matrixRotY(sp + 16, angle);
    matrixVecRotAligned(&[(sp + 16) as i64, sp as i64]);
    let y = lwc1(sp + 4);
    let z = lwc1(sp + 8);
    swc1(out, ps2float::add(lwc1(sp), lwc1(prop + 48)));
    swc1(out + 4, ps2float::add(y, lwc1(prop + 52)));
    swc1(out + 8, ps2float::add(z, lwc1(prop + 56)));
}

/// The angle (0 to 360 degrees) from the point `from` to the pivot of the remote prop (@ 0x26c318).
pub fn remoteGetStandAngle(prop: i32, from: i32) -> f32 {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let offset = remoteGetPivoteOffset(prop);
    let scale = lwc1(0x3acc38);
    let to_degrees = 180.0_f32;
    let angle = ps2float::mul(lwc1(prop + 76), scale);
    swc1(sp, lwc1(offset));
    swc1(sp + 4, lwc1(offset + 4));
    swc1(sp + 8, lwc1(offset + 8));
    matrixRotY(sp + 16, (angle).pdiv(to_degrees));
    matrixVecRotAligned(&[(sp + 16) as i64, sp as i64]);
    let x = ps2float::add(lwc1(sp), lwc1(prop + 48));
    let z = ps2float::add(lwc1(sp + 8), lwc1(prop + 56));
    let from_z = lwc1(from + 8);
    let from_x = lwc1(from);
    let y = ps2float::add(lwc1(sp + 4), lwc1(prop + 52));
    swc1(sp, x);
    let dz = ps2float::sub(z, from_z);
    swc1(sp + 8, z);
    let dx = ps2float::sub(x, from_x);
    swc1(sp + 4, y);
    let mut degrees = ps2float::mul(atan2f(dx, dz), to_degrees).pdiv(scale);
    if !(degrees < 0.0_f32) {
        let full = 360.0_f32;
        if full <= degrees {
            degrees = ps2float::sub(degrees, full);
        }
    } else {
        degrees = ps2float::add(degrees, 360.0_f32);
    }
    degrees
}

/// The place to stand at to use the remote prop when facing `angle` degrees: the pivot's place less a vector turned by the angle (@ 0x26c1c8).
pub fn remoteGetStandPos(prop: i32, out: i32, angle: f32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let offset = remoteGetPivoteOffset(prop);
    let scale = lwc1(0x3acc34);
    let to_degrees = 180.0_f32;
    let turn = ps2float::mul(lwc1(prop + 76), scale);
    swc1(sp + 16, lwc1(offset));
    swc1(sp + 20, lwc1(offset + 4));
    swc1(sp + 24, lwc1(offset + 8));
    matrixRotY(sp + 32, (turn).pdiv(to_degrees));
    matrixVecRotAligned(&[(sp + 32) as i64, (sp + 16) as i64]);
    let z = lwc1(prop + 56);
    let stand_turn = ps2float::mul(angle, scale).pdiv(to_degrees);
    let pz = lwc1(sp + 24);
    let px = ps2float::add(lwc1(sp + 16), lwc1(prop + 48));
    let py = ps2float::add(lwc1(sp + 20), lwc1(prop + 52));
    sw(sp, 0);
    swc1(sp + 16, px);
    swc1(sp + 20, py);
    swc1(sp + 24, ps2float::add(pz, z));
    let height = lwc1(offset + 4);
    swc1(sp + 8, 0.75_f32);
    swc1(sp + 4, height);
    matrixRotY(sp + 32, stand_turn);
    matrixVecRotAligned(&[(sp + 32) as i64, sp as i64]);
    let ez = lwc1(sp + 24);
    let sz = lwc1(sp + 8);
    let dy = ps2float::sub(lwc1(sp + 20), lwc1(sp + 4));
    swc1(out, ps2float::sub(lwc1(sp + 16), lwc1(sp)));
    swc1(out + 4, dy);
    swc1(out + 8, ps2float::sub(ez, sz));
}

/// How the direction of a swing compares with a way: -1 to 1 by the angle (0 to 360) between the unit vector of `swing` and the vector `way` turned by the three angles at `angles`
/// (-1 at 0 degrees, 0 at 90, 1 at 180, 0 at 270, back to -1 at 360) (@ 0x2794a0).
pub fn getSwingDirectionMultiplier(swing: i32, way: i32, angles: i32) -> f32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut result = 0.0_f32;
    let x = lwc1(swing);
    let y = lwc1(swing + 4);
    let z = lwc1(swing + 8);
    let length = ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt();
    let angle_y = lwc1(angles + 4);
    let scale = lwc1(0x3aceb4);
    let c = ps2float::mul(lwc1(angles + 8), scale);
    let a = ps2float::mul(lwc1(angles), scale);
    swc1(sp + 72, (z).pdiv(length));
    swc1(sp + 64, (x).pdiv(length));
    let c = (c).pdiv(180.0_f32);
    let a = (a).pdiv(180.0_f32);
    let b = ps2float::mul(angle_y, scale).pdiv(180.0_f32);
    swc1(sp + 68, (y).pdiv(length));
    matrixRotZXY(sp, a, b, c);
    matrixVecRot(sp, sp + 64);
    let dot = ps2float::add(ps2float::add(ps2float::mul(lwc1(way), lwc1(sp + 64)), ps2float::mul(lwc1(way + 4), lwc1(sp + 68))), ps2float::mul(lwc1(way + 8), lwc1(sp + 72)));
    let mut degrees = ps2float::mul(acosf(dot), 180.0_f32).pdiv(scale);
    if degrees < result {
        degrees = ps2float::add(degrees, 360.0_f32);
    }
    if degrees < 90.0_f32 {
        result = -((ps2float::sub(90.0_f32, degrees)).pdiv(90.0_f32));
    }
    if 90.0_f32 <= degrees && degrees < 180.0_f32 {
        result = (ps2float::sub(degrees, 90.0_f32)).pdiv(90.0_f32);
    }
    if 180.0_f32 <= degrees && degrees < 270.0_f32 {
        result = (ps2float::sub(90.0_f32, ps2float::sub(degrees, 180.0_f32))).pdiv(90.0_f32);
    }
    if !(270.0_f32 <= degrees) {
        result
    } else {
        -((ps2float::sub(degrees, 270.0_f32)).pdiv(90.0_f32))
    }
}

/// Make a new map maker object of kind 1 of the tile set `set` at the place (`x`, `y`) on level `level` with the rotation `rotation` (+12 to +24 of the object), lit in the
/// middle grey (@ 0x248910).
pub fn mmNewTile(set: i32, x: i32, y: i32, level: i32, rotation: i32) -> i32 {
    let object = mmAllocObject();
    let tile = lw((set << 2) + g::tiles);
    sw(object, 1);
    let second = lw(tile + 112);
    let layers = lw(tile + 24);
    let third = lw(second + 112);
    sw(object + 4, 0);
    let fourth = lw(third + 112);
    sw(object + 8, layers);
    sw(object + 48, fourth);
    sw(object + 36, tile);
    sw(object + 40, second);
    sw(object + 44, third);
    mmSetTileLight(object, 0, 0, 0x7f7f7f80);
    sw(object + 12, x);
    sw(object + 16, y);
    sw(object + 20, level);
    sw(object + 24, rotation);
    object
}

/// Find the preload entry (records of 24 bytes at 0x1fb4c60: the kind at +0, the name at +12) of kind `kind` called `name`, or 0 (@ the three functions below).
fn preload_find(kind: i32, name: i32) -> i32 {
    let count = lw(g::numpreload);
    let mut i = 0;
    if count > 0 {
        loop {
            let entry = i * 24 + preload_table();
            if lw(entry) == kind && strcmp(lw(entry + 12), name) == 0 {
                return entry;
            }
            i += 1;
            if i >= lw(g::numpreload) {
                break;
            }
        }
    }
    0
}

/// The preload entry of a file called `name` (@ 0x21c500).
pub fn preloadGetFile(name: i32) -> i32 {
    preload_find(0, name)
}

/// The preload entry of an object called `name` (@ 0x21c598).
pub fn preloadGetOb(name: i32) -> i32 {
    preload_find(1, name)
}

/// The preload entry of a texture called `name` (@ 0x21c630).
pub fn preloadGetTexName(name: i32) -> i32 {
    preload_find(2, name)
}

/// Show the stats update page to the first sign-on that is not in the state 8: its number is kept at 0x3aff38 and its split message cleared; the page is set and the flags 3 of the
/// first page record cleared; returns 1, or 0 if there is none (@ 0x240000).
pub fn setStatsUpdatePage() -> i32 {
    let mut i = 0;
    let mut count;
    loop {
        count = lw(0x3aff20);
        if i >= count {
            break;
        }
        let sign_on = signonGet(i);
        count = lw(0x3aff20);
        if lw((sign_on << 3) + g::signoninfo) != 8 {
            break;
        }
        i += 1;
    }
    if i != count {
        sw(0x3aff38, i);
        splitMessageClear(i);
        let page = 0x1fc11f8;
        sw(page + 4, lw(page + 4) & -4);
        frontPageSet(0, g::statsupdate_page);
        1
    } else {
        0
    }
}
