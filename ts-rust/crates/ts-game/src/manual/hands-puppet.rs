//! First person arms.
//!
//! Without a weapon the player attacks with an arm: the game makes a second copy of the player's own character (the "puppet", `remote_hand` in
//! `player-gun-update.rs`), puts it beside the view and plays a swipe on it, so the arm that is seen has the character's own skin and sleeve.  With `first_person_arms`
//! on, the same copy is made while a gun is held.  It plays an animation of the character's own in which the gun is held out (the one the game has for a character
//! that shoots a gun at a crate for the guns of two hands, one with the arm across the body for the guns of one), holds the gun with `gunChrUpdate` (the model of the gun
//! that the characters of the game hold, in the hand), and is placed and turned so that the gun points where the player aims.  The gun of the first person is not shown.
//! The arms are the character's own whoever the character is.
//!
//! Not done: the left hand of a two handed gun is out of the view, a pair of guns and the thrown weapons keep the guns of the first person, and the gun does not recoil or
//! reload.  `TS_HANDANIM=<animation>`, `TS_HANDYAW=<degrees>` and `TS_HANDOFF=<across>,<up>,<forward>` set the animation, the turn and the place by hand, to find the values
//! for a gun.

use crate::*;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// The setting `first_person_arms` (the program's settings file).
pub fn set_first_person_arms(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// The first person gun that the character of each player holds (0: none).
static GUNS_HELD: [AtomicI32; 4] = [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)];
/// The gun (number in the table) that each player's character holds.
static GRIPS: [AtomicI32; 4] = [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)];
/// The characters (puppets) of the players 0 to 3 (0: none yet).
static PUPPETS: [AtomicI32; 4] = [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)];
/// Scratch space for the matrix of the heading and the place.
const SCRATCH: i32 = 0x240_0000 + 0x17_2800;

/// A game starts: the characters of the one before are gone.
pub fn hands_reset() {
    for p in &PUPPETS {
        p.store(0, Ordering::Relaxed);
    }
}

/// How a gun is held: the animation, the turn about the height (degrees) and the place (across, up, forward) of the character.
struct Pose {
    anim: i32,
    yaw: f32,
    place: [f32; 3],
}

/// The pose every gun is held in: the character's own animation for a gun held in both hands, the place of the character (across, up, forward of the view; the hand that holds the gun
/// is the one that is seen) and the angles (degrees, in the view's space) that the gun of the first person is turned to.
const HELD: Pose = Pose { anim: 478, yaw: 0.0, place: [0.7, 0.5, 0.5] };
const GUN_ANGLES: [f32; 3] = [-5.0, 155.0, -10.0];

/// The grip of a gun, in the gun's own space: the point that is put in the hand.  None for the guns that keep their own view (mines, TNT, the brick, the unnamed ones).
fn grip_of(gun: i32) -> Option<[f32; 3]> {
    Some(match gun {
        0 => [-0.022, -0.04, -0.04],
        7 => [-0.02, -0.06, -0.06],
        9 => [-0.02, -0.11, 0.05],
        23 => [-0.03, -0.12, -0.04],
        21 => [-0.02, -0.09, -0.04],
        1 => [-0.02, -0.10, -0.05],
        2 => [-0.02, -0.06, 0.0],
        19 => [-0.02, -0.08, -0.06],
        24 => [-0.02, -0.04, -0.03],
        20 => [-0.02, -0.08, -0.05],
        10 => [-0.02, -0.04, 0.08],
        3 | 4 | 5 | 6 | 8 | 11 | 12 | 22 => [-0.02, -0.08, -0.03],
        _ => return None,
    })
}

/// The pose of a gun (a number of the table of guns).
fn pose_of(gun: i32) -> Option<&'static Pose> {
    grip_of(gun).map(|_| &HELD)
}

fn env_number(name: &str) -> Option<f32> {
    std::env::var(name).ok().and_then(|v| v.parse().ok())
}

/// At the end of the update of a gun (`gunUpdate`, the gun slot `a0` of the hand `a1`, `mind` the player's record): keep the character that holds the gun of the right hand.
pub fn hands_puppet_tick(a0: i32, a1: i32, mind: i32) {
    let manual = env_number("TS_HANDANIM");
    if !ENABLED.load(Ordering::Relaxed) && manual.is_none() {
        return;
    }
    // (only when a gun is held: the state of a gun below 20 and not 0, which is the arm of the fists)
    let state = lw(a0);
    if a1 != 0 || state == 0 || (state as u32) >= 20 {
        return;
    }
    let player = lw(g::currentplayer);
    let number = lw(player);
    if !(0..4).contains(&number) {
        return;
    }
    let combo = (lw(mind + 260) << 3) + GUNCOMBOS;
    let (right, left) = (lh(combo + 4), lh(combo + 2));
    let body = lw(player + 384);
    let hand_made;
    let pose = match manual {
        Some(a) => {
            hand_made = Pose { anim: a as i32, yaw: env_number("TS_HANDYAW").unwrap_or(0.0), place: [0.3, 0.3, 0.8] };
            Some(&hand_made)
        }
        None if left < 0 => pose_of(right),
        None => None,
    };
    let mut puppet = PUPPETS[number as usize].load(Ordering::Relaxed);
    let Some(pose) = pose else {
        // (a gun that keeps its own view: the character is not shown)
        if puppet != 0 {
            show(puppet, number, false);
        }
        return;
    };
    if puppet == 0 {
        let kind = lh(mind + 4);
        puppet = chrNew(kind, kind, 0, number + 104, -1, 0, lwc1(body + 48), lwc1(body + 52), lwc1(body + 56), lwc1(body + 76));
        if puppet == 0 {
            return;
        }
        propRemovePropFromCollisionList(puppet);
        sw(puppet + 8, 16);
        let inst = lw(puppet + 32);
        sh(inst + 292, lhu(inst + 292) & 0x3fff);
        sw(puppet + 12, gun_room(number));
        obInstSetAnim(inst, pose.anim, 1.0);
        PUPPETS[number as usize].store(puppet, Ordering::Relaxed);
        // (a gun slot of a new character holds 0, which is a gun: it holds no gun yet)
        let m = lw(puppet + 352);
        sw(m + 376, -1);
        sw(m + 268, -1);
    }
    let puppet_mind = lw(puppet + 352);
    show(puppet, number, true);
    // the animation of the pose (set again when the gun changed to one of another kind)
    let inst = lw(puppet + 32);
    if curAnim(puppet, pose.anim) == 0 {
        obInstSetAnim(inst, pose.anim, 1.0);
    }
    // a test aid: `TS_DUMPARM=<file>` writes the matrices of the joints of the character at the frame 250 of the level (the pose of its animation), to make the arm of the pose a part of a gun's model
    if let Ok(path) = std::env::var("TS_DUMPARM") {
        if lw(g::gameframe) == 250 {
            let matrices = lw(lw(puppet + 32) + 4);
            let mut text = String::new();
            for joint in 0..40 {
                let m = matrices + (joint << 6);
                text.push_str(&(0..16).map(|k| format!("{:.6}", lwc1(m + 4 * k))).collect::<Vec<_>>().join(" "));
                text.push_str("\n");
            }
            // (and the matrices of the model itself, which are what the vertices of the file are in)
            let rest = lw(lw(lw(puppet + 32)) + 4);
            text.push_str("rest
");
            for joint in 0..40 {
                let m = rest + (joint << 6);
                text.push_str(&(0..16).map(|k| format!("{:.6}", lwc1(m + 4 * k))).collect::<Vec<_>>().join(" "));
                text.push_str("
");
            }
            let _ = std::fs::write(path, text);
        }
    }
    let yaw = env_number("TS_HANDYAW").unwrap_or(pose.yaw);
    swc1(puppet + 76, lwc1(body + 76) + yaw);
    swc1(puppet + 80, lwc1(body + 80) + yaw);
    swc1(puppet + 84, lwc1(body + 84));
    swc1(puppet_mind + 2968, lwc1(player + 284));
    swc1(puppet_mind + 2972, lwc1(player + 276));
    // where the character is: the place of the pose, turned by the way the player faces
    let by_hand: Vec<f32> = std::env::var("TS_HANDOFF").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default();
    let keep_gun = std::env::var_os("TS_KEEPGUN").is_some();
    for k in 0..3 {
        let mut v = if by_hand.len() == 3 { by_hand[k] } else { pose.place[k] };
        if keep_gun {
            // the gun of the first person stays and the arm is put on it: where the gun is held in the view (`view_x`, `view_y`, `view_z` of the gun) moves the arm with it
            let reference = if matches!(right, 0 | 4 | 7 | 9 | 21 | 23) { [0.15_f32, -0.21, -0.39] } else { [0.31, -0.33, -0.55] };
            v += (lwc1(GUNS_TABLE + 400 * right + 32 + 4 * k as i32) - reference[k]) * env_number("TS_HANDK").unwrap_or(1.0);
        }
        swc1(SCRATCH + 64 + 4 * k as i32, v);
    }
    swc1(SCRATCH + 76, 0.0);
    matrixRotY(SCRATCH, ps2float::mul(lwc1(body + 80), lwc1(0x3ad3bc)).pdiv(1.8e+02_f32));
    matrixVecMulAligned(&[SCRATCH as i64, (SCRATCH + 64) as i64]);
    let data = lw(body + 356);
    swc1(puppet + 48, lwc1(data + 140) + lwc1(SCRATCH + 64));
    swc1(puppet + 52, lwc1(data + 156) - lwc1(data + 104) + lwc1(SCRATCH + 68));
    swc1(puppet + 56, lwc1(data + 148) + lwc1(SCRATCH + 72));
    if keep_gun {
        return;
    }
    if std::env::var_os("TS_GPA").is_none() {
        // the gun of the first person is held by the hand of the character: its matrices are made again after the game made them (`hands_gun_matrices`)
        let gun = lw(a0 + 12);
        GUNS_HELD[number as usize].store(if gun != puppet { gun } else { 0 }, Ordering::Relaxed);
        GRIPS[number as usize].store(right, Ordering::Relaxed);
        return;
    }
    // the character holds the gun of the player (`gunChrUpdate`: the model of the gun, in the hand)
    sw(puppet_mind + 260, lw(mind + 260));
    gunChrUpdate(puppet, 0);
    let held = lw(puppet_mind + 376 + 64);
    if held != 0 && std::env::var_os("TS_HANDLOG").is_some() && lw(g::gameframe) % 60 == 0 {
        eprintln!("[hands] held prop {held:#x} kind {} flags {:#x} parent {:#x}", lw(held + 8), lw(held + 16), lw(held + 188));
    }
    if held != 0 {
        sw(held + 16, lw(held + 16) | 0x40000000 | view_bit(number));
        sw(held + 12, gun_room(number));
    }
    // (and the gun of the first person is not shown)
    let first_person = lw(a0 + 12);
    if first_person != 0 && first_person != puppet {
        sw(first_person + 16, lw(first_person + 16) & !view_bit(number));
    }
}

/// The character and the gun it holds are seen by the player (`on`) or not.
fn show(puppet: i32, number: i32, on: bool) {
    let flags = lw(puppet + 16);
    sw(puppet + 16, if on { flags | 0x40000000 | view_bit(number) } else { flags & !view_bit(number) });
    let held = lw(lw(puppet + 352) + 376 + 64);
    if held != 0 {
        let f = lw(held + 16);
        sw(held + 16, if on { f | 0x40000000 | view_bit(number) } else { f & !view_bit(number) });
    }
    let inst = lw(puppet + 32);
    let f = lhu(inst + 292);
    // (the flag 0x8000 of the model is what shows the arm: the fists set it for a swipe and take it off at rest)
    sh(inst + 292, if on { f | 0x8000 } else { f & 0x7fff });
}

/// The name of the first person model of a gun with the arms of the player's character on it, for a name of the file of a gun (`ob/gun2/gun2.raw`, with or without `host0:`): the
/// same with `_c<number of the character>` before the extension (the files the hands mod makes for the characters that have arms of their own: `examples/first_person_hands_src`).
/// None for another file.  The caller uses the file with this name when the packs have it and the original when they have not.
/// Set while the gun of the left hand is being made (a gun in each hand): its model has only the hand on its grip, `gunN_cNN_l.raw`.
pub static LEFT_GUN_BUILD: AtomicBool = AtomicBool::new(false);

/// `gunNew` for a gun held in both hands: the prop is made from the model of the gun that has the hand on the grip only (`gunN_cNN_l.raw`), which is loaded the first time and kept by
/// prop number as long as the model of the plain gun is the one it was loaded beside; the prop table keeps the plain model for the next gun.
pub fn dual_gun_new(prop: i32, left_hand: bool) -> i32 {
    use std::sync::Mutex;
    static LEFT_MODELS: Mutex<Vec<(i32, i32, i32, bool)>> = Mutex::new(Vec::new());
    let def = prop * 28 + PROPTABLE;
    if lw(def + 4) == 0 {
        // (the plain model is loaded by making a prop of it)
        let view = gunNew(prop, 0.0_f32, -1e+02_f32, 0.0_f32, 0.0_f32);
        propDelete(view);
    }
    let plain = lw(def + 4);
    if plain == 0 || !ENABLED_LEFT.load(Ordering::Relaxed) {
        return gunNew(prop, 0.0_f32, -1e+02_f32, 0.0_f32, 0.0_f32);
    }
    let known = LEFT_MODELS.lock().ok().and_then(|l| l.iter().find(|e| e.0 == prop && e.1 == plain && e.3 == left_hand).map(|e| e.2));
    let left = match known {
        Some(left) => left,
        None => {
            let left = match left_model_name(lw(def + 12), left_hand) {
                Some(name) => obLoad(name),
                None => 0,
            };
            if let Ok(mut l) = LEFT_MODELS.lock() {
                l.retain(|e| !(e.0 == prop && e.3 == left_hand));
                l.push((prop, plain, left, left_hand));
            }
            left
        }
    };
    if left == 0 {
        return gunNew(prop, 0.0_f32, -1e+02_f32, 0.0_f32, 0.0_f32);
    }
    sw(def + 4, left);
    let view = gunNew(prop, 0.0_f32, -1e+02_f32, 0.0_f32, 0.0_f32);
    sw(def + 4, plain);
    view
}

static ENABLED_LEFT: AtomicBool = AtomicBool::new(true);

/// The name of the model of the gun held in both hands (`gunN_cNN_l.raw`) when the mod has one for the first player's character.
fn left_model_name(name: i32, left_hand: bool) -> Option<i32> {
    let at = arms_variant_name(name)?;
    let mut text = Vec::new();
    for i in 0.. {
        let b = lbu(at + i) as u8;
        if b == 0 {
            break;
        }
        text.push(b);
    }
    let s = String::from_utf8_lossy(&text).replace(".raw", if left_hand { "_m.raw" } else { "_l.raw" });
    let left = at + 0x80;
    for (i, b) in s.bytes().enumerate() {
        sb(left + i as i32, b as i32);
    }
    sb(left + s.len() as i32, 0);
    (packfileSeekPlain(left) != 0).then_some(left)
}

/// The names to try, first to last, for the model of a gun: the one of the left hand's gun (when it is being made), the one with the arms of the first player's character, then the plain one.
pub fn arms_variant_names(name: i32) -> Vec<i32> {
    let mut out = Vec::new();
    if let Some(at) = arms_variant_name(name) {
        if LEFT_GUN_BUILD.load(Ordering::Relaxed) {
            let mut text = Vec::new();
            for i in 0.. {
                let b = lbu(at + i) as u8;
                if b == 0 {
                    break;
                }
                text.push(b);
            }
            let s = String::from_utf8_lossy(&text).replace(".raw", "_l.raw");
            let left = at + 0x80;
            for (i, b) in s.bytes().enumerate() {
                sb(left + i as i32, b as i32);
            }
            sb(left + s.len() as i32, 0);
            out.push(left);
        }
        out.push(at);
    }
    out
}

pub fn arms_variant_name(name: i32) -> Option<i32> {
    // ("ob/gun" and a number, "/gun", the same number and ".raw": not the guns of the characters)
    let mut text = [0u8; 48];
    for (i, b) in text.iter_mut().enumerate() {
        *b = lbu(name + i as i32) as u8;
        if *b == 0 {
            break;
        }
    }
    let end = text.iter().position(|&b| b == 0)?;
    let s = std::str::from_utf8(&text[..end]).ok()?;
    let s = s.strip_prefix("host0:").unwrap_or(s);
    let rest = s.strip_prefix("ob/gun")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || rest != format!("{digits}/gun{digits}.raw") {
        return None;
    }
    let id = signonGet(0);
    if id < 0 {
        return None;
    }
    // (the test aid `TS_PLAYERCHR` sets the character later than the models are loaded: it is looked at here too)
    let character = std::env::var("TS_PLAYERCHR").ok().and_then(|v| v.parse().ok()).unwrap_or_else(|| lw(lw(g::signonarray) + id * 2936 + 32 + 84));
    if !(0..128).contains(&character) {
        return None;
    }
    if std::env::var_os("TS_ARMLOG").is_some() {
        eprintln!("[arms] f{} {} state {} left {}", ts_sdk::frame_count(), s, lw(g::gamestate), LEFT_GUN_BUILD.load(Ordering::Relaxed));
    }
    let prefix = if name_has_prefix(name) { "host0:" } else { "" };
    let variant = format!("{prefix}ob/gun{digits}/gun{digits}_c{character:02}.raw");
    let at = 0x240_0000 + 0x17_2a00;
    for (i, b) in variant.bytes().enumerate() {
        sb(at + i as i32, b as i32);
    }
    sb(at + variant.len() as i32, 0);
    Some(at)
}

fn name_has_prefix(name: i32) -> bool {
    (0..6).all(|i| lbu(name + i) as u8 == b"host0:"[i as usize])
}

fn env_list(name: &str, n: usize) -> Option<Vec<f32>> {
    let v: Vec<f32> = std::env::var(name).ok()?.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    (v.len() == n).then_some(v)
}

/// After `animMtxTick`: the matrices of the first person gun that a character holds are the matrices of the character's hand bone (made for the same view) times a turn and a place
/// that put the grip of the gun in the hand, times the gun's own bone matrices.  `TS_GUNROT=<a>,<b>,<c>` (degrees), `TS_GUNPOS=<x>,<y>,<z>` and `TS_GRIP=<x>,<y>,<z>` (the grip, in the gun's own space) set them.
pub fn hands_gun_matrices() {
    for n in 0..4usize {
        let gun = GUNS_HELD[n].load(Ordering::Relaxed);
        let puppet = PUPPETS[n].load(Ordering::Relaxed);
        if gun == 0 || puppet == 0 {
            continue;
        }
        let (pi, gi) = (lw(puppet + 32), lw(gun + 32));
        let parent_list = lw(pi + 8 + 4 * n as i32);
        let gun_list = lw(gi + 8 + 4 * n as i32);
        if parent_list == 0 || gun_list == 0 || lw(gi + 4) == 0 {
            continue;
        }
        let bone = lw(lw(pi + 92) + 28);
        let rot = env_list("TS_GUNROT", 3).unwrap_or(vec![0.0, 0.0, 0.0]);
        let pos = env_list("TS_GUNPOS", 3).unwrap_or(vec![0.0, 0.0, 0.0]);
        let grip = env_list("TS_GRIP", 3).unwrap_or_else(|| grip_of(GRIPS[n].load(Ordering::Relaxed)).map(|g| g.to_vec()).unwrap_or(vec![0.0, 0.0, 0.0]));
        let radians = lwc1(0x3acd94);
        let angle = |d: f32| ps2float::mul(d, radians).pdiv(1.8e+02_f32);
        // (the matrices are made in the scratch space: the turn and place, the grip taken out of the hand, the hand's matrix, then each bone of the gun)
        let f = SCRATCH + 256;
        let g = SCRATCH + 320;
        let h = SCRATCH + 384;
        matrixTransRotZXY(f, pos[0], pos[1], pos[2], angle(rot[0]), angle(rot[1]), angle(rot[2]));
        matrixTrans(g, -grip[0], -grip[1], -grip[2]);
        sceVu0MulMatrix(f, f, g);
        sceVu0MulMatrix(h, parent_list + (bone << 6), f);
        {
            let e = env_list("TS_GUNEULER", 3).unwrap_or(GUN_ANGLES.to_vec());
            // the gun turned as the view's own gun is (angles in the view's space, the place of the hand only): the way the hand is turned does not matter
            let m = parent_list + (bone << 6);
            matrixTransRotZXY(f, lwc1(m + 48) + pos[0], lwc1(m + 52) + pos[1], lwc1(m + 56) + pos[2], angle(e[0]), angle(e[1]), angle(e[2]));
            sceVu0MulMatrix(h, f, g);
        }
        let bones = lw(lw(gi) + 4);
        for b in 0..bones {
            sceVu0MulMatrix(gun_list + (b << 6), h, lw(gi + 4) + (b << 6));
        }
    }
}
