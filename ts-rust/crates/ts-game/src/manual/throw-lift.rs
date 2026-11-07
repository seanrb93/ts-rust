//! The lift of a thrown weapon (the `throw_lift` of a weapon in `weapons.txt` of a mod).
//!
//! A shot goes where the weapon is aimed, as fast as the weapon's `bullet_speed` says, and the brick, which has gravity, falls to the floor within a few steps of leaving the
//! hand.  A weapon with a lift gets that much added to the upward speed of its shot (units a frame, the same as `bullet_speed`), so that it goes up and comes down again:
//! a throw.  The lifts are in memory of the port's own, a float for each weapon number, written when the mods are loaded; no lift, no change.

use crate::*;

/// The lifts: a float for each weapon number below `MAX_GUNS`.
pub const THROW_LIFT: i32 = 0x240_0000 + 0x17_2400;

/// How strong the blast of a kind of shot is (the `blast` and `alt_blast` of a weapon in `weapons.txt`): a float for each kind of bullet below 64, which
/// multiplies the damage of every stage of the explosion (the game's own blast is 2.3 at its middle, which kills); 0, as all are until a mod sets one, is the game's.
pub const BLAST_SCALE: i32 = 0x240_0000 + 0x17_2C00;

/// The strength of the blasts of the kind of bullet `kind`, as a factor (1 for the game's own).
pub fn blast_scale(kind: i32) -> f32 {
    if (0..64).contains(&kind) {
        let v = lwc1(BLAST_SCALE + 4 * kind);
        if v > 0.0 {
            return v;
        }
    }
    1.0
}

/// The lift of the weapon `gun` (0 for most).
pub fn throw_lift(gun: i32) -> f32 {
    if (0..MAX_GUNS).contains(&gun) {
        lwc1(THROW_LIFT + 4 * gun)
    } else {
        0.0
    }
}
