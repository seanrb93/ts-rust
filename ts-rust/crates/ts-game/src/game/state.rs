//! Getters and setters of the game's global state, and the stubs of features the console build left empty.

use crate::*;

/// Camera mode (@ 0x27e318); modes are 0 game view, 1 third person, 3 move view (see `debugflags.rs`).
pub fn camGetMode() -> i32 {
    lw(g::cammode)
}

/// @ 0x27e310
pub fn camSetMode(mode: i32) {
    sw(g::cammode, mode);
}

/// Score of the challenge that is being played (@ 0x21f080).
pub fn challengeGetScore() -> i32 {
    lw(0x3aff00)
}

/// Address of the definition of the current challenge: a table of 64-byte entries by the game's parameters (@ 0x21f060).
pub fn challengeGetSetup() -> i32 {
    (lw(g::curgameparms + 68) << 6) + g::challengedefs
}

/// @ 0x225788
pub fn demoGetHeader() -> i32 {
    lw(g::demoheader)
}

/// @ 0x2c7ed0
pub fn enemyGetNumBases() -> i32 {
    lw(g::numBases)
}

/// Leave the level for the front end: not a restart, game state 5 (@ 0x21d338).
pub fn gameLevelQuit() {
    sw(g::restart, 0);
    sw(g::gamestate, 5);
}

/// Play the level again: game state 5 with the restart flag (@ 0x21d320).
pub fn gameLevelRestart() {
    sw(g::restart, 1);
    sw(g::gamestate, 5);
}

/// Empty in the build (@ 0x22e518).
pub fn guiGetImageHeight(_image: i32) {}

/// Empty in the build (@ 0x22e4e8).
pub fn guiGetImageWidth(_image: i32) {}

/// Number of players on this machine (@ 0x2cdde0).  The `ilink` functions are for the PS2's network link; the build has no link.
pub fn ilinkGetNumLocalPlayers() -> i32 {
    lw(g::lvnumplayers)
}

/// @ 0x2cddf0
pub fn ilinkGetStartLocalPlayerNum() -> i32 {
    0
}

/// @ 0x2cddf8
pub fn ilinkIsILinkGame() -> i32 {
    0
}

/// @ 0x2cddd8
pub fn ilinkOKToTick() -> i32 {
    1
}

/// Track number of the tune chosen in the map maker (@ 0x24c870).
pub fn mmGetTuneNum() -> i32 {
    lbu(lw(g::gv_tune) + g::disptunenums)
}

/// Size of a map maker file directory (@ 0x2538e8).
pub fn mmfileDirSize() -> i32 {
    4016
}

/// Empty in the build (@ 0x205ad0).
pub fn musicGetState() {}

/// @ 0x21ffb0
pub fn statsGet() -> i32 {
    lw(0x3aff04)
}

/// Whether the game is paused (@ 0x2b6850).
pub fn timeGetPause() -> i32 {
    (lw(g::gamepaused) != 0) as i32
}

/// Empty in the build (@ 0x2671a0).
pub fn propGetName(_prop: i32) {}

/// The body part that was hit last (@ 0x285060).
pub fn chrGetLastHitPart() -> i32 {
    lw(g::s_LastHitPart_3afa2c)
}

/// @ 0x285068
pub fn chrSetLastHitPart(part: i32) {
    sw(g::s_LastHitPart_3afa2c, part);
}

/// Empty in the build (@ 0x201e80).
pub fn memGetFreeEter() {}

/// Empty in the build (@ 0x201e70).
pub fn memGetFreeLevel() {}

/// Whether the level creation flags `flags` allow the mode being played: mode 1 to 6 test bit 0 to 5, mode 7 (story) the bit of its difficulty (64, 128 or 256) (@ 0x228d88).
pub fn lvCheckCreateFlags(flags: i32) -> i32 {
    let params = g::curgameparms;
    let mode = lw(params + 72) - 1;
    if (mode as u32) < 7 {
        if mode < 6 {
            return (flags & (1 << mode) != 0) as i32;
        }
        let difficulty = lw(params + 76);
        if difficulty == 0 && flags & 64 != 0 {
            return 1;
        }
        if lw(params + 76) == 1 && flags & 128 != 0 {
            return 1;
        }
        if lw(params + 76) != 2 {
            return 0;
        }
        return (flags & 256 != 0) as i32;
    }
    0
}
