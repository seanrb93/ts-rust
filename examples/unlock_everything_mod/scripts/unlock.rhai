// Unlock Everything: everything the game keeps locked (levels, characters, bot sets, challenges and cheats) is open.
// The game has one switch for it (what its cheat code sets); the script keeps it on while the game runs.

fn unlock_switch() { 0x3ae2e0 }   // the game's variable `val1`

fn on_frame() {
    poke(unlock_switch(), 1);
}
