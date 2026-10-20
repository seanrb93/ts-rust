//! Ends of the special effects and other small functions that only call another one.

use crate::*;

/// End the front end's multiplayer game set-up: delete the characters' props (@ 0x241538).
pub fn frontmpgEnd() {
    resetchrprops();
}

/// Draw the pause page (@ 0x2110c0).
pub fn pause_pageGfx() {
    drawPause();
}

/// Copy the matrix `from` to `to` (the first argument is not used) (@ 0x2b5c08).
pub fn calInbetweenMatrix(_unused: i32, from: i32, to: i32) {
    sceVu0CopyMatrix(to, from);
}

/// Keep the time at the end of the tick in `cpuendtime` (@ 0x2b6860).
pub fn timeTickEnd() {
    swc1(g::cpuendtime, timeGet());
}

/// Free all 56 groups of special effects (@ 0x2a9850).
pub fn specialfxEnd() {
    let mut i = 0;
    loop {
        specialfxFree(i);
        i += 1;
        if i >= 56 {
            break;
        }
    }
}

/// Copy the first 0x10480 bytes of the image at `image` to the scratch buffer (the big buffer; the download buffer is 66688 bytes after it) and return its format (@ 0x2b9c90).
pub fn ImageGetDetailsFromMem(image: i32) -> i32 {
    let buffer = g::scratchbuffer;
    sw(g::downloadbuffer, g::scratchbuffer + 66688);
    sw(g::bigbuffer, buffer);
    bcopy(image, buffer, 0x10480);
    GetFormat(lw(g::bigbuffer))
}

/// Get the memory for the header of a demo (112 bytes of the level area); the game exits when there is none (@ 0x2256e0).
pub fn demoMakeHeader() {
    let memory = memAlloc(112, 1);
    sw(g::demoheader, memory);
    if memory != 0 {
        return;
    }
    exit(-1);
}

/// Get and clear the temporary buffer of the map maker (0x2000 bytes of the eternal area), clear the edit flags and make the dialogs (@ 0x2483a8).
pub fn mapmakerMake() {
    let memory = memAlloc(0x2000, 2);
    sw(g::mmtempbuffer, memory);
    memset(memory, 0, 0x2000);
    sw(g::editflags, 0);
    mmDialogsMake();
}

/// Free the group of special effects that is at the address `group` (nothing when it is not one of the 56) (@ 0x2aae88).
pub fn specialfxFreePtr(group: i32) {
    if group != 0 {
        let mut i = 0;
        let mut at = g::fxgroups;
        while group != at {
            i += 1;
            at += 92;
            if i >= 56 {
                return;
            }
        }
        specialfxFree(i);
    }
}

/// Add the files of the 10 built-in maps (records of 8 bytes at `builtinmaps`, the file name at +4) to the files to preload (@ 0x253620).
pub fn mmfilePreloadBuiltinMaps() {
    let mut left = 9;
    let mut at = g::builtinmaps + 4;
    loop {
        let name = lw(at);
        at += 8;
        left -= 1;
        preloadAddFile(name, 0);
        if left < 0 {
            break;
        }
    }
}

/// The tick of the glow of the planet that spawns characters, for the effects group `group`: the glow's prop (at +56) is deleted when the group's time (+48) is past the
/// game's constant at 0x3ad700, and the group is freed when it is past 4 (@ 0x2acac0).
pub fn specialfxPlanetSpawnGlowTick(group: i32) {
    let groups = g::fxgroups;
    let slot = group * 92 + (groups + 56);
    let prop = lw(slot);
    if prop != 0 && lwc1(0x3ad700) < lwc1(groups + group * 92 + 48) {
        specialPropDelete(prop);
        sw(slot, 0);
    }
    if !(4.0_f32 < lwc1(group * 92 + (g::fxgroups + 48))) {
        return;
    }
    specialfxFree(group);
}
