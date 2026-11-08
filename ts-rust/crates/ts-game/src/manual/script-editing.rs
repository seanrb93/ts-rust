//! What the scripts of mods have for editing a level and drawing in the view: the buttons of the players' controllers, the pads and links of the level (the
//! places the computer characters go between) and the props, and the drawing of a cylinder and of the blast ring.  Most of it calls the original game's
//! pad and prop editor and its drawing code, which the game has but never calls (`createpad`, `deletepad`, `createlink`, `padTogglePadFlag`,
//! `padInfoGfx`, `propEditCreate`, `propEditMove`, `propEditChange`, `moveCylGfx`, `blastringNew`, `blastringGfx`); the files they save are written in the
//! form the game loads, not as the assembler source the original's `padSave` and `propEditSave` wrote.

use crate::*;
use rhai::Engine;
use std::cell::RefCell;

const PAD_SIZE: i32 = 28;
const LINK_SIZE: i32 = 20;
const PROP_SIZE: i32 = 592;
/// The memory of the pads and links (`padPreload`).
const PADS_MOST: i32 = 0x10680 / PAD_SIZE;
const LINKS_MOST: i32 = 0x157c0 / LINK_SIZE;
/// The debug memory one cylinder takes (`moveCylGfx`).
const CYLINDER_BYTES: i32 = 784;
/// How far from the eye of the view a cylinder has to be to be drawn.
const CYLINDER_CLEAR: f32 = 1.5;
const DEBUG_NEXT: i32 = 0x3afe14;
const DEBUG_END: i32 = 0x3afe10;
/// How far the view is tested for where it meets the level, and the place taken where it meets nothing.
const AIM_REACH: f32 = 50.0;
const AIM_NOTHING: f32 = 3.0;

thread_local! {
    /// Drawing in the view is allowed (the scripts' `on_gfx`).
    static IN_GFX: RefCell<bool> = const { RefCell::new(false) };
    /// Writing text on the screen is allowed (the scripts' `on_hud`).
    static IN_HUD: RefCell<bool> = const { RefCell::new(false) };
    /// Where the blast rings are centred (the original centres them on the first player).
    static RING_AT: RefCell<[f32; 3]> = const { RefCell::new([0.0; 3]) };
    /// The rings as they were before the first view of the frame drew them, and after (`blastringGfx` moves them on as it draws them, so the other
    /// views of a split screen draw them from the same state and leave them as the first view did).
    static RINGS_BEFORE: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
    static RINGS_AFTER: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

const RING_WORDS: i32 = 180 / 4;

fn rings_save() -> Vec<i32> {
    let mut v: Vec<i32> = (0..RING_WORDS).map(|k| lw(g::br + 4 * k)).collect();
    v.push(lw(g::numrings));
    v
}

fn rings_load(v: &[i32]) {
    if v.len() as i32 != RING_WORDS + 1 {
        return;
    }
    for k in 0..RING_WORDS {
        sw(g::br + 4 * k, v[k as usize]);
    }
    sw(g::numrings, v[RING_WORDS as usize]);
}

pub(super) fn set_in_gfx(on: bool) {
    IN_GFX.with(|g| *g.borrow_mut() = on);
}

pub(super) fn set_in_hud(on: bool) {
    IN_HUD.with(|g| *g.borrow_mut() = on);
}

fn player_record(p: i64) -> i32 {
    lw(g::players) + (p as i32) * 1820
}

fn valid_player(p: i64) -> bool {
    p >= 0 && (p as i32) < lw(g::lvnumplayers) && lw(g::players) != 0
}

/// Run `f` with the game's current player set to the player `p` (the editor's functions work on the current player).
fn as_player<R>(p: i64, f: impl FnOnce(i32) -> R) -> R {
    let old = lw(g::currentplayer);
    let player = player_record(p);
    sw(g::currentplayer, player);
    let r = f(player);
    sw(g::currentplayer, old);
    r
}

fn pad_at(k: i64) -> i32 {
    lw(g::levelpads) + (k as i32) * PAD_SIZE
}

fn valid_pad(k: i64) -> bool {
    lw(g::levelpads) != 0 && k >= 0 && (k as i32) < lw(g::numpads)
}

fn pad_used(k: i64) -> bool {
    valid_pad(k) && padvalid(pad_at(k)) != 0
}

fn link_at(i: i64) -> i32 {
    lw(g::levellinks) + (i as i32) * LINK_SIZE
}

fn valid_link(i: i64) -> bool {
    lw(g::levellinks) != 0 && i >= 0 && (i as i32) < lw(g::numlinks)
}

/// The live prop with the external reference `extref`, or 0.
fn prop_by_extref(extref: i64) -> i32 {
    let props = lw(g::levelprops);
    if props == 0 {
        return 0;
    }
    for i in 0..lw(g::num_props) {
        let prop = props + i * PROP_SIZE;
        if lw(prop + 16) >= 0 && lw(prop) == extref as i32 {
            return prop;
        }
    }
    0
}

fn room_props_full() -> bool {
    lw(g::num_props) >= lw(g::max_props)
}

/// Where the view of the player `p` meets the level, as the editor's `propEditMove` tests it: from the eye to the aim point.
fn aim_point(p: i64) -> Option<[f32; 3]> {
    if !valid_player(p) {
        return None;
    }
    let player = player_record(p);
    let chr = lw(player + 384);
    if chr == 0 {
        return None;
    }
    let frame = Frame::new(64);
    let sp = frame.sp();
    let room = match lw(player + 796) {
        0 => 1,
        r => r,
    };
    // (the player's own prop and guns are left out of the test)
    let record = lw(chr + 352);
    let parts = [chr, lw(record + 332), lw(record + 440)];
    for &part in &parts {
        if part != 0 {
            sw(part + 16, lw(part + 16) & -33);
        }
    }
    // (the line goes 50 along the view, the direction of the aim at +2976 of the character's record; where it meets nothing the place is 3 ahead)
    let eye = [lwc1(player + 152), lwc1(player + 156), lwc1(player + 160)];
    let dir = [lwc1(record + 2976), lwc1(record + 2980), lwc1(record + 2984)];
    for k in 0..3 {
        swc1(sp + 48 + 4 * k as i32, dir[k] * AIM_REACH);
        swc1(sp + 16 + 4 * k as i32, eye[k] + dir[k] * AIM_NOTHING);
    }
    sw(sp, 0);
    sw(sp + 8, 0);
    let hit = bgLineTest(room, player + 152, sp + 48, 0, sp + 16, sp + 32, 0, 0);
    for &part in &parts {
        if part != 0 {
            sw(part + 16, lw(part + 16) | 32);
        }
    }
    if hit == 0 {
        for k in 0..3 {
            swc1(sp + 16 + 4 * k as i32, eye[k] + dir[k] * AIM_NOTHING);
        }
    }
    Some([lwc1(sp + 16), lwc1(sp + 20), lwc1(sp + 24)])
}

/// The direction the player `p` looks in, in degrees about the up axis as the pads and props have it (0 along z).
fn view_heading(p: i64) -> f32 {
    let Some(at) = aim_point(p) else { return 0.0 };
    let eye = player_record(p) + 152;
    let (dx, dz) = (at[0] - lwc1(eye), at[2] - lwc1(eye + 8));
    if dx == 0.0 && dz == 0.0 {
        return 0.0;
    }
    dx.atan2(dz).to_degrees().rem_euclid(360.0)
}

/// The pad file the level was loaded from, as the packs name it.
fn pad_file_name() -> String {
    let at = padFileInName();
    let mut text = Vec::new();
    for i in 0..120 {
        let b = lbu(at + i) as u8;
        if b == 0 {
            break;
        }
        text.push(b);
    }
    let name = String::from_utf8_lossy(&text).to_string();
    name.strip_prefix("host0:").unwrap_or(&name).to_string()
}

/// The pads and links of the level in the form `padLoad` reads: a version (1), the counts, the pads (28 bytes: the external reference, the flags, the
/// hall, the place and the heading) and the links (20 bytes: the flags and the external references of the two pads).
/// Pads that were deleted and links that were deleted or join a deleted pad are left out.
fn pad_file_bytes() -> Vec<u8> {
    let mut pads = Vec::new();
    for k in 0..lw(g::numpads) as i64 {
        if pad_used(k) {
            pads.push(k);
        }
    }
    let mut links = Vec::new();
    for i in 0..lw(g::numlinks) as i64 {
        let link = link_at(i);
        let (a, b) = (lw(link + 4) as i64, lw(link + 8) as i64);
        if lw(link) & 1 != 0 && pad_used(a) && pad_used(b) {
            links.push(link);
        }
    }
    let mut out = Vec::new();
    let word = |out: &mut Vec<u8>, w: i32| out.extend_from_slice(&w.to_le_bytes());
    word(&mut out, 1);
    word(&mut out, pads.len() as i32);
    word(&mut out, links.len() as i32);
    for &k in &pads {
        let pad = pad_at(k);
        word(&mut out, lw(pad));
        word(&mut out, lw(pad + 4) & !0x10000);
        word(&mut out, lw(pad + 8));
        for off in [12, 16, 20, 24] {
            word(&mut out, lw(pad + off));
        }
    }
    for &link in &links {
        word(&mut out, lw(link));
        word(&mut out, lw(pad_at(lw(link + 4) as i64)));
        word(&mut out, lw(pad_at(lw(link + 8) as i64)));
        word(&mut out, 0);
        word(&mut out, 0);
    }
    out
}

pub(super) fn register(engine: &mut Engine) {
    // the buttons of a player's controller: held, newly pressed, and taken from the game (it does not see them until they are let go)
    engine.register_fn("buttons", |p: i64| if valid_player(p) { (lw(crate::game::controllers::controller(p as i32) + 288) & 0xffff) as i64 } else { 0 });
    engine.register_fn("pressed", |p: i64| if valid_player(p) { (joyNewBut(p as i32) & 0xffff) as i64 } else { 0 });
    engine.register_fn("consume", |p: i64, mask: i64| {
        if valid_player(p) {
            joyClearButs(p as i32, mask as i32);
        }
    });
    engine.register_fn("level", || lw(g::lvlevelnum) as i64);

    // the pads: a pad's number is its place in the level's list (pad_count, pad_x, pad_y and pad_z are in the scripts' own functions)
    engine.register_fn("pad_used", |k: i64| pad_used(k));
    engine.register_fn("pad_extref", |k: i64| if valid_pad(k) { lw(pad_at(k)) as i64 } else { -1 });
    engine.register_fn("pad_flags", |k: i64| if valid_pad(k) { lw(pad_at(k) + 4) as i64 } else { 0 });
    engine.register_fn("pad_toggle_flag", |k: i64, mask: i64| {
        if pad_used(k) && mask & 1 == 0 {
            padTogglePadFlag(pad_at(k), mask as i32);
        }
    });
    // a new pad where the player `p` stands, facing where the player faces; its number, or -1 when the list is full
    engine.register_fn("pad_add", |p: i64| -> i64 {
        if !valid_player(p) || lw(g::levelpads) == 0 || lw(g::numpads) >= PADS_MOST {
            return -1;
        }
        let prop = lw(player_record(p) + 384);
        if prop == 0 {
            return -1;
        }
        let k = lw(g::numpads) as i64;
        as_player(p, |player| {
            sh(g::curhall, lw(player + 796));
            createpad();
        });
        let pad = pad_at(k);
        swc1(pad + 12, lwc1(prop + 48));
        swc1(pad + 16, lwc1(prop + 52));
        swc1(pad + 20, lwc1(prop + 56));
        swc1(pad + 24, view_heading(p));
        k
    });
    engine.register_fn("pad_delete", |k: i64| {
        if pad_used(k) {
            deletepad(pad_at(k));
        }
    });
    engine.register_fn("pad_link", |a: i64, b: i64| -> bool {
        if !pad_used(a) || !pad_used(b) || a == b || lw(g::numlinks) >= LINKS_MOST {
            return false;
        }
        let before = findlink(pad_at(a), pad_at(b));
        if before != 0 {
            return false;
        }
        sw(g::lastpadnum, a as i32);
        sw(g::curpadnum, b as i32);
        sw(g::linkmode, 0);
        createlink(pad_at(a), pad_at(b));
        findlink(pad_at(a), pad_at(b)) != 0
    });
    engine.register_fn("pad_unlink", |a: i64, b: i64| -> bool {
        if !valid_pad(a) || !valid_pad(b) {
            return false;
        }
        let link = findlink(pad_at(a), pad_at(b));
        if link != 0 {
            deletelink(link);
        }
        link != 0
    });
    engine.register_fn("link_count", || lw(g::numlinks) as i64);
    engine.register_fn("link_used", |i: i64| valid_link(i) && lw(link_at(i)) & 1 != 0);
    engine.register_fn("link_from", |i: i64| if valid_link(i) { lw(link_at(i) + 4) as i64 } else { -1 });
    engine.register_fn("link_to", |i: i64| if valid_link(i) { lw(link_at(i) + 8) as i64 } else { -1 });
    // the original's text about a pad, in the top right of the view (only in `on_hud`)
    engine.register_fn("pad_info", |k: i64| {
        if IN_HUD.with(|h| *h.borrow()) && valid_pad(k) {
            // (the original puts it 360 to the right of the window's corner, for its small letters: the port's are larger, so it is put 80 in)
            let x = lw(g::window);
            sw(g::window, x - 280);
            padInfoGfx(pad_at(k));
            sw(g::window, x);
        }
    });
    // the file the pads were loaded from (pad/data/level<n>.raw)
    engine.register_fn("pad_file", || pad_file_name());
    engine.register_fn("pad_file_bytes", || -> rhai::Blob { pad_file_bytes() });

    // the props, by their external reference
    engine.register_fn("aim_point", |p: i64| -> rhai::Array {
        match aim_point(p) {
            Some(v) => v.iter().map(|&f| rhai::Dynamic::from(f as f64)).collect(),
            None => rhai::Array::new(),
        }
    });
    // the name of a kind of prop (the name of its model in the table of props)
    engine.register_fn("prop_kind_name", |kind: i64| -> String {
        if !(0..=PROP_LAST as i64).contains(&kind) {
            return String::new();
        }
        let at = lw(PROPTABLE + (kind as i32) * 28 + 8);
        if at <= 0 {
            return String::new();
        }
        let mut text = Vec::new();
        for i in 0..40 {
            let b = lbu(at + i) as u8;
            if b == 0 {
                break;
            }
            text.push(b);
        }
        String::from_utf8_lossy(&text).to_string()
    });
    engine.register_fn("prop_kind_editable", |kind: i64| (0..=PROP_LAST as i64).contains(&kind) && propEditValidPropnum(kind as i32) != 0);
    // a new prop of the kind `kind` where the player `p` looks; its external reference, or -1
    engine.register_fn("prop_place", |p: i64, kind: i64| -> i64 {
        if !valid_player(p) || lw(g::levelprops) == 0 || room_props_full() || !(0..=PROP_LAST as i64).contains(&kind) || propEditValidPropnum(kind as i32) == 0 {
            return -1;
        }
        let Some(at) = aim_point(p) else { return -1 };
        let extref = propGetNewExtref();
        let heading = view_heading(p);
        as_player(p, |player| {
            // (the editor makes the prop at its camera, +832 of the player, turned by the direction at +844: they are lent the place where the
            // view meets the level and the view's direction, and given back, as the game uses them for the player's aim)
            let saved: Vec<i32> = (0..7).map(|k| lw(player + 828 + 4 * k)).collect();
            for (k, v) in at.iter().enumerate() {
                swc1(player + 832 + 4 * k as i32, *v);
            }
            let radians = heading.to_radians();
            swc1(player + 844, radians.sin());
            swc1(player + 848, 0.0);
            swc1(player + 852, radians.cos());
            propEditCreate(kind as i32, extref);
            for (k, w) in saved.iter().enumerate() {
                sw(player + 828 + 4 * k as i32, *w);
            }
        });
        if prop_by_extref(extref as i64) != 0 { extref as i64 } else { -1 }
    });
    // the prop of the next or the last kind the editor can make, in its place; the kind, or -1
    engine.register_fn("prop_change", |p: i64, extref: i64, step: i64| -> i64 {
        let prop = prop_by_extref(extref);
        if prop == 0 || !valid_player(p) {
            return -1;
        }
        let (x, y, z, angle) = (lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56), lwc1(prop + 76));
        as_player(p, |player| {
            // (the new prop is made at the editor's camera: the old prop's place)
            let saved: Vec<i32> = (0..7).map(|k| lw(player + 828 + 4 * k)).collect();
            swc1(player + 832, x);
            swc1(player + 836, y);
            swc1(player + 840, z);
            swc1(player + 848, 1.0);
            propEditChange(prop, if step < 0 { -1 } else { 1 });
            for (k, w) in saved.iter().enumerate() {
                sw(player + 828 + 4 * k as i32, *w);
            }
        });
        let new = prop_by_extref(extref);
        if new == 0 {
            return -1;
        }
        swc1(new + 76, angle);
        lw(g::lasteditpropnum) as i64
    });
    engine.register_fn("prop_delete", |extref: i64| {
        let prop = prop_by_extref(extref);
        if prop != 0 {
            propEditDelete(prop);
        }
    });
    engine.register_fn("prop_exists", |extref: i64| prop_by_extref(extref) != 0);
    engine.register_fn("prop_kind", |extref: i64| match prop_by_extref(extref) { 0 => -1, p => lw(p + 4) as i64 });
    engine.register_fn("prop_x", |extref: i64| match prop_by_extref(extref) { 0 => 0.0, p => lwc1(p + 48) as f64 });
    engine.register_fn("prop_y", |extref: i64| match prop_by_extref(extref) { 0 => 0.0, p => lwc1(p + 52) as f64 });
    engine.register_fn("prop_z", |extref: i64| match prop_by_extref(extref) { 0 => 0.0, p => lwc1(p + 56) as f64 });
    engine.register_fn("prop_angle", |extref: i64| match prop_by_extref(extref) { 0 => 0.0, p => lwc1(p + 76) as f64 });
    // (a prop is made again to turn it: its walls and floors are worked out when it is made)
    engine.register_fn("prop_turn", |extref: i64, degrees: f64| -> bool {
        let prop = prop_by_extref(extref);
        if prop == 0 {
            return false;
        }
        let (kind, x, y, z) = (lw(prop + 4), lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56));
        let a = (lwc1(prop + 76) + degrees as f32).rem_euclid(360.0);
        propEditDelete(prop);
        let new = propNew(kind, x, y, z, a);
        if new != 0 {
            sw(new, extref as i32);
        }
        new != 0
    });
    // a prop of an editable kind at a place (for putting saved props back); its external reference, or -1
    engine.register_fn("prop_new", |kind: i64, x: f64, y: f64, z: f64, angle: f64| -> i64 {
        if lw(g::levelprops) == 0 || room_props_full() || !(0..=PROP_LAST as i64).contains(&kind) || propEditValidPropnum(kind as i32) == 0 {
            return -1;
        }
        let prop = propNew(kind as i32, x as f32, y as f32, z as f32, angle as f32);
        if prop == 0 {
            return -1;
        }
        let extref = propGetNewExtref();
        sw(prop, extref);
        extref as i64
    });
    // the live prop nearest to a place within `most`; its external reference, or -1
    engine.register_fn("prop_nearest", |x: f64, y: f64, z: f64, most: f64| -> i64 {
        let props = lw(g::levelprops);
        if props == 0 {
            return -1;
        }
        let mut best = (-1_i64, most * most);
        for i in 0..lw(g::num_props) {
            let prop = props + i * PROP_SIZE;
            if lw(prop + 16) < 0 || lw(prop) == -1 {
                continue;
            }
            let (dx, dy, dz) = (lwc1(prop + 48) as f64 - x, lwc1(prop + 52) as f64 - y, lwc1(prop + 56) as f64 - z);
            let d = dx * dx + dy * dy + dz * dz;
            if d < best.1 {
                best = (lw(prop) as i64, d);
            }
        }
        best.0
    });

    // drawing in the view (only in `on_gfx`): a cylinder of eight sides, as the original drew a character's move test
    engine.register_fn("draw_cylinder", |x: f64, y: f64, z: f64, height: f64, radius: f64| {
        // (the debug memory of the frame, from its next free byte at 0x3afe14 to its end at 0x3afe10, has room for it and some more)
        if !IN_GFX.with(|g| *g.borrow()) || lw(DEBUG_NEXT) + CYLINDER_BYTES + 0x400 > lw(DEBUG_END) {
            return;
        }
        // (the original draws it with the clipping off, so one that reaches the eye of the view covers the screen: it is not drawn)
        let eye = lw(g::currentplayer) + 152;
        let (dx, dz) = (x as f32 - lwc1(eye), z as f32 - lwc1(eye + 8));
        let near = radius as f32 + CYLINDER_CLEAR;
        if dx * dx + dz * dz < near * near {
            return;
        }
        moveCylGfx(x as f32, y as f32, z as f32, height as f32, radius as f32);
    });
    // a blast ring at a place: an upright ring that grows for a second; one at a time (the original's `blastringNew` always starts the first ring of
    // its table again, so a second ring would start the first again); true when it started
    engine.register_fn("blast_ring", |x: f64, y: f64, z: f64| -> bool {
        if lw(g::numrings) != 0 {
            return false;
        }
        RING_AT.with(|r| *r.borrow_mut() = [x as f32, y as f32, z as f32]);
        blastringNew();
        true
    });
}

/// Draw the blast rings that are going on in the view of the player `view`, centred where the script asked (`blastringGfx` centres them on the first
/// player: its place is lent to them).
pub(super) fn draw_rings(view: i64) {
    if view == 0 {
        RINGS_BEFORE.with(|r| *r.borrow_mut() = rings_save());
    } else {
        RINGS_BEFORE.with(|r| rings_load(&r.borrow()));
    }
    if lw(g::numrings) <= 0 || lw(g::players) == 0 {
        if view == 0 {
            RINGS_AFTER.with(|r| *r.borrow_mut() = rings_save());
        } else {
            RINGS_AFTER.with(|r| rings_load(&r.borrow()));
        }
        return;
    }
    let at = lw(g::players) + 152;
    let saved = [lw(at), lw(at + 4), lw(at + 8)];
    let centre = RING_AT.with(|r| *r.borrow());
    for (k, v) in centre.iter().enumerate() {
        swc1(at + 4 * k as i32, *v);
    }
    blastringGfx();
    for (k, w) in saved.iter().enumerate() {
        sw(at + 4 * k as i32, *w);
    }
    if view == 0 {
        RINGS_AFTER.with(|r| *r.borrow_mut() = rings_save());
    } else {
        RINGS_AFTER.with(|r| rings_load(&r.borrow()));
    }
}

/// A file the script writes in its mod's folder (`mod_write`): the name must stay inside the folder.
pub(super) fn write_in_mod(dir: &std::path::Path, file: &str, bytes: &[u8]) -> bool {
    if file.is_empty() || file.contains("..") || file.starts_with('/') || file.starts_with('\\') || file.contains(':') {
        return false;
    }
    let path = dir.join(file);
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    std::fs::write(&path, bytes).is_ok()
}
