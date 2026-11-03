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

const TWO_HANDED: Pose = Pose { anim: 478, yaw: 0.0, place: [0.3, 0.3, 0.8] };
const ONE_HANDED: Pose = Pose { anim: 36, yaw: -165.0, place: [0.35, 0.35, 0.9] };

/// The pose of a gun (a number of the table of guns); None for the guns that keep their own view (mines, TNT, the brick, the unnamed ones).
fn pose_of(gun: i32) -> Option<&'static Pose> {
    match gun {
        0 | 4 | 7 | 9 | 21 | 23 => Some(&ONE_HANDED),
        1 | 2 | 3 | 5 | 6 | 8 | 10 | 11 | 12 | 19 | 20 | 22 | 24 => Some(&TWO_HANDED),
        _ => None,
    }
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
    if std::env::var_os("TS_FPATTACH").is_some() {
        // the gun of the first person is held by the character's hand: it is attached to the bone of the right hand as the gun of a character is
        let gun = lw(a0 + 12);
        if gun != 0 && gun != puppet {
            if lw(gun + 188) != puppet {
                let bone = lw(lw(lw(puppet + 32) + 92) + 28);
                propSetAttached(gun, lw(puppet + 32), bone, 1);
                // (a prop of the kind of the guns of the characters is drawn with its holder's matrices, the kind of the first person gun is drawn in front of the view)
                sw(gun + 8, 32);
            }
            sw(gun + 16, lw(gun + 16) | 0x40000000 | view_bit(number));
            sw(gun + 12, gun_room(number));
            if std::env::var_os("TS_HANDLOG").is_some() && lw(g::gameframe) % 60 == 0 {
                eprintln!("[hands] gun prop {gun:#x} kind {} flags {:#x} room {} parent {:#x} bone {} pos ({:.2},{:.2},{:.2}) mtx {:#x}", lw(gun + 8), lw(gun + 16), lw(gun + 12), lw(gun + 188), lw(gun + 192), lwc1(gun + 48), lwc1(gun + 52), lwc1(gun + 56), lw(lw(gun + 32) + 4));
            }
        }
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
