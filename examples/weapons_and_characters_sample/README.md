# Weapons and characters sample

A mod made of `weapons.txt` and `characters.txt`, which set the game's tables of weapons and characters by the names of their fields.  Copy the folder into `mods/` next to `ts-rust.exe` and set `enabled = true` in `mod.ini`.
The log says how many changes were made and reports every line that was left out, with its number.  The fields and their limits are listed at the top of `ts-rust/crates/ts-port/src/moddata.rs`; `raw +<offset> <b|h|w|f> <value>` sets a field that has no name yet.
