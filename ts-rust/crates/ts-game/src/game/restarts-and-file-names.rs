//! Clearing the state of a subsystem when a level starts, the names of files, and calls through function pointers.

use crate::*;

/// Clear the 976 bytes of the cartridge groups (@ 0x295268).
pub fn cartridgesRestart() {
    memset(g::cartridge_groups, 0, 976);
}

/// Clear the 60 bytes of ammo counts of a character record (at +620) (@ 0x288918).
pub fn chrResetAmmo(chr: i32) {
    memset(chr + 620, 0, 60);
}

/// Clear the character records (0x1210 bytes each): 4 of them in the level 102, else 108 (@ 0x284fc8).
pub fn chrRestart() {
    let records = if (lw(g::lvlevelnum) ^ 102) == 0 { 4 } else { 108 };
    memset(lw(g::chrinfo), 0, records * 0x1210);
}

/// Set the four state bytes at 0x3aff28 to the low byte of `value` (@ 0x2300a8).
pub fn setStates(value: i32) {
    memset(0x3aff28, value & 255, 4);
}

/// Clear the 64 bytes of the weather (@ 0x2a8018).
pub fn weatherRestart() {
    memset(g::weather, 0, 64);
}

/// Clear the table of 21 tile pointers and the tile count (@ 0x248ee8).
pub fn mmClearTiles() {
    memset(g::tiles, 0, 84);
    sw(g::numtiles, 0);
}

/// The file name of demo `number` (format at 0x3a0d80) (@ 0x225718).
pub fn demoFileName(number: i32) -> i32 {
    vafmt(&[0x3a0d80, number as i64])
}

/// The file name of the icon of bag `number` (format at 0x3a9900, name from `bagiconfiles`) (@ 0x2b04f8).
pub fn hudGetBagIconFileName(number: i32) -> i32 {
    let name = lw((number << 2) + g::bagiconfiles);
    vafmt(&[0x3a9900, name as i64])
}

/// Make the name of the file of the props of this level in the buffer at 0x1fe98f0 (format at 0x3a8748) (@ 0x27c138).
pub fn propFileInName() {
    let level = lw(g::lvlevelnum);
    sprintf(&[0x1fe98f0, 0x3a8748, level as i64]);
}

/// Call the function at `function` with `argument` (twice) and return its result (@ 0x20caa0).
pub fn gettextfromfunction(function: i32, argument: i32) -> i32 {
    call_indirect(function, [argument, argument, 0, 0, 0, 0, 0, 0], [0.0; 8]).0
}

/// Call the message ticker of the debug menu, when a message is showing and there is one (@ 0x20db60).
pub fn debugmenuTick() {
    if lw(g::in_message) != 0 {
        let ticker = lw(g::in_message_ticker);
        if ticker != 0 {
            call_indirect(ticker, [0; 8], [0.0; 8]);
        }
    }
}

/// Make page `page` of the front end the one being shown, with the number `number` (+28 and +24 of the front end record) (@ 0x22ceb0).
pub fn frontPageSet(page: i32, number: i32) {
    let front = lw(g::front);
    sw(front + 28, page);
    sw(front + 24, number);
}
