//! The debug menu's switches: each one is a word in the game's data; `toggle...` flips it and `get...` gives the text the menu shows
//! for it ("On" / "Off", the game's own strings).

use crate::*;

/// The text of the game for an on / off setting.
fn on_off(flag: i32) -> i32 {
    if lw(flag) != 0 {
        lw(g::ontext)
    } else {
        lw(g::offtext)
    }
}

fn flip(flag: i32) {
    sw(flag, 1 - lw(flag));
}

/// Flip `collisions` (@ 0x20d7d0).
pub fn togglecollisions() {
    flip(g::collisions);
}

/// Flip `displayperfstats` (@ 0x20d828).
pub fn toggledisplayperfstats() {
    flip(g::displayperfstats);
}

/// Flip `displaypos` (@ 0x20d6e0).
pub fn toggledisplaypos() {
    flip(g::displaypos);
}

/// Flip `displayrooms` (@ 0x20d840).
pub fn toggledisplayrooms() {
    flip(g::displayrooms);
}

/// Flip `displayspeed` (@ 0x20d810).
pub fn toggledisplayspeed() {
    flip(g::displayspeed);
}

/// Flip `invisible` (@ 0x20d798).
pub fn toggleinvisible() {
    flip(g::invisible);
}

/// Flip `turtlecpu` (@ 0x20daf0).
pub fn toggleturtlecpu() {
    flip(g::turtlecpu);
}

/// Flip `turtlegs` (@ 0x20db08).
pub fn toggleturtlegs() {
    flip(g::turtlegs);
}

/// Flip `viewbg` (@ 0x20da08).
pub fn toggleviewbg() {
    flip(g::viewbg);
}

/// Flip `viewfloors` (@ 0x20d928).
pub fn toggleviewfloors() {
    flip(g::viewfloors);
}

/// Flip `viewguns` (@ 0x20d9d0).
pub fn toggleviewguns() {
    flip(g::viewguns);
}

/// Flip `viewportalrects` (@ 0x20d8f0).
pub fn toggleviewportalrects() {
    flip(g::viewportalrects);
}

/// Flip `viewportals` (@ 0x20d8b8).
pub fn toggleviewportals() {
    flip(g::viewportals);
}

/// Flip `viewvolumes` (@ 0x20d998).
pub fn toggleviewvolumes() {
    flip(g::viewvolumes);
}

/// Flip `viewwalls` (@ 0x20d960).
pub fn toggleviewwalls() {
    flip(g::viewwalls);
}

/// The on / off text for `collisions` (@ 0x20d7e8).
pub fn getcollisions() -> i32 {
    on_off(g::collisions)
}

/// The on / off text for `displayperfstats` (@ 0x20d878).
pub fn getdisplayperfstats() -> i32 {
    on_off(g::displayperfstats)
}

/// The on / off text for `displaypos` (@ 0x20d6f8).
pub fn getdisplaypos() -> i32 {
    on_off(g::displaypos)
}

/// The on / off text for `displayrooms` (@ 0x20d898).
pub fn getdisplayrooms() -> i32 {
    on_off(g::displayrooms)
}

/// The on / off text for `displayspeed` (@ 0x20d858).
pub fn getdisplayspeed() -> i32 {
    on_off(g::displayspeed)
}

/// The on / off text for `invincible` (@ 0x20d778).
pub fn getinvincible() -> i32 {
    on_off(g::invincible)
}

/// The on / off text for `invisible` (@ 0x20d7b0).
pub fn getinvisible() -> i32 {
    on_off(g::invisible)
}

/// The on / off text for `padlinkedit` (@ 0x20da58).
pub fn getpadlinkedit() -> i32 {
    on_off(g::padlinkedit)
}

/// The on / off text for `propedit` (@ 0x20da90).
pub fn getpropedit() -> i32 {
    on_off(g::propedit)
}

/// The on / off text for `turtlecpu` (@ 0x20dab0).
pub fn getturtlecpu() -> i32 {
    on_off(g::turtlecpu)
}

/// The on / off text for `turtlegs` (@ 0x20dad0).
pub fn getturtlegs() -> i32 {
    on_off(g::turtlegs)
}

/// The on / off text for `viewfloors` (@ 0x20d940).
pub fn getviewfloors() -> i32 {
    on_off(g::viewfloors)
}

/// The on / off text for `viewguns` (@ 0x20d9e8).
pub fn getviewguns() -> i32 {
    on_off(g::viewguns)
}

/// The on / off text for `viewportalrects` (@ 0x20d908).
pub fn getviewportalrects() -> i32 {
    on_off(g::viewportalrects)
}

/// The on / off text for `viewportals` (@ 0x20d8d0).
pub fn getviewportals() -> i32 {
    on_off(g::viewportals)
}

/// The on / off text for `viewvolumes` (@ 0x20d9b0).
pub fn getviewvolumes() -> i32 {
    on_off(g::viewvolumes)
}

/// The on / off text for `viewwalls` (@ 0x20d978).
pub fn getviewwalls() -> i32 {
    on_off(g::viewwalls)
}

/// Switch the pad link editor on or off; it and the prop editor are never on together (@ 0x20da40).
pub fn togglepadlinkedit() {
    let was = lw(g::padlinkedit);
    sw(g::propedit, 0);
    sw(g::padlinkedit, 1 - was);
}

/// Switch the prop editor on or off (@ 0x20da78).
pub fn togglepropedit() {
    let was = lw(g::propedit);
    sw(g::padlinkedit, 0);
    sw(g::propedit, 1 - was);
}

/// Flip invincibility, with a sound for the new state (150 on, 151 off) (@ 0x20d750).
pub fn toggleinvincible() {
    let now = 1 - lw(g::invincible);
    sw(g::invincible, now);
    soundStart(if now == 0 { 151 } else { 150 }, 0);
}

/// "On" when the camera is the game view (mode 0) (@ 0x20d568).
pub fn getgameview() -> i32 {
    if camGetMode() == 0 { lw(g::ontext) } else { lw(g::offtext) }
}

/// "On" when the camera is the third-person view (mode 2) (@ 0x20d598).
pub fn getthirdpersonview() -> i32 {
    if camGetMode() == 2 { lw(g::ontext) } else { lw(g::offtext) }
}

/// The on / off text for `viewbg` (@ 0x20da20).
pub fn getviewbg() -> i32 {
    on_off(g::viewbg)
}

/// "On" when the camera is in move mode (mode 3) (@ 0x20d530).
pub fn getmoveview() -> i32 {
    if camGetMode() == 3 { lw(g::ontext) } else { lw(g::offtext) }
}

/// Camera: game view (@ 0x20d560).
pub fn startgameview() {
    camSetMode(0);
}

/// Camera: move view (@ 0x20d528).
pub fn startmoveview() {
    camSetMode(3);
}

/// Camera: third-person view (@ 0x20d590).
pub fn startthirdpersonview() {
    camSetMode(1);
}
