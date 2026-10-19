# Decompiling the game by hand

The port started as a static recompilation: `tools/gen_rust.py` translates every function of the PS2 executable into Rust that works on the
console's flat memory (`lw(a0 + 48)`, `v1_3`, ...).  That code runs, but it cannot be read or changed much, and it is not shared (it is made
on your own machine from your own copy of the game).  This project turns it, function by function, into source that people can read and edit.

## Where things are

- `ts-rust/crates/ts-game/src/game/` is the decompiled game: readable Rust, committed.  One file per subsystem (`random.rs`, `math.rs`, ...).
- `ts-rust/crates/ts-game/src/dec/` is the machine translation that is still left (generated, not committed).  A function that exists in
  `game/` is skipped by the generator, so every function is in exactly one place.
- `ts-rust/crates/ts-game/src/manual/` is port code (menus, options, platform glue) that is not part of the original game.
- `python tools/progress.py [-v]` says how many functions are done.

## Rules for a decompiled function

1. Same name, same signature as the original.  Callers that are still machine-translated pass raw values, so a pointer is an `i32` address.
   Inside, use the typed views of `game/types.rs` (`Mat4`, `Quat`, ...) and add the fields of a structure there as it is understood.
2. Do what the machine code does, in the same order.  This includes quirks (reading an input after part of the output was written).  A
   function may be improved later, on purpose, in a separate commit that says so; the decompilation itself changes no behaviour.
3. A doc comment says what the function does and its address (`@ 0x2b4678`).  Name constants and fields; leave no magic numbers whose
   meaning is known.
4. No game data in the repository: no extracted tables, no text, no pictures.  Data stays in the game's files and is read at run time.

## File names

Files follow the Google developer documentation style for file names (https://developers.google.com/style/filenames): lowercase, words
separated by hyphens, no spaces or special characters, and a name that says what is in the file (`debug-flags.rs`, `route-tables-and-kills.rs`, never
`misc2.rs`).  Rust module names cannot contain hyphens, so `game/mod.rs` declares such a file with `#[path = "debug-flags.rs"] pub mod debug_flags;`.
A file holds one subsystem or theme; when a theme grows, split it and name the parts by what they hold.

## Workflow

1. Pick a function (`python tools/disasm.py <name>` shows the machine code; the generated Rust is in `dec/`).
2. Write it in `game/` under the right subsystem file and add the module to `game/mod.rs` if the file is new.
3. Regenerate (`python tools/gen_rust.py`, with `Game Files/SLUS_200.90` in place) so the generated copy is dropped, and build.
4. Prove it equal to the machine code: `ts-verify --elf <SLUS_200.90> --n 500 name1 name2 ...` runs the original instructions and the Rust on
   random inputs and compares results and every memory write.  Functions that call others are checked with those calls included.
   `python tools/verify-game.py [--regen]` does this for every function in `game/`.  Functions whose original code does not finish on random
   data (loops over lists) get a second pass with small numbers (`TS_VERIFY_SMALL=1`); a few that use instructions the interpreter lacks
   (the 128-bit stores of `memMark`) are only reviewed.  Functions that read the port's relocated tables (the prop table, `GUNCOMBOS`) differ
   from the machine code on purpose and are listed in `BY_DESIGN` in that script.  The live shadow mode (`TS_SHADOW`, see the README)
   only hooks machine-translated functions, not those of `game/`.
5. Commit with the functions' names in the message.

Order of work: leaf functions first (maths, strings, lists, memory), then the systems built on them, working towards `gameTick` and the
front end.  When a structure is understood (its size and fields), give it a type in `game/types.rs` and use it in every function that touches it.

## Fidelity first, extensions later

The decompilation has to match the original executable exactly before anything is changed.  The port used to move four of the game's tables into
its own memory with more room: the prop table and the walls and floors of props (480 rooms instead of about 200), the lists of the props of each
room, the weapon combinations (with the added pairs of weapons for dual wielding), and the level table (levels of mods).  These are now behind the
Cargo feature `extended-tables` of `ts-game` (`ts-port` passes it on), which is **off**: the generated and the hand-written code read the game's
own tables at the game's own addresses and with its limits, and `tools/verify-game.py` finds no function that differs.  With the feature off,
dual wield pairs and levels of mods are not available; the converter's big levels need the feature on (build with
`--features extended-tables`), and the code that uses it is not kept equal to the original.

To do later, once the code matches: bring the extended tables back as real data structures of the decompiled code instead of moved addresses,
and take back the other limits the port raised (instance and prop counts of a level, the big memory pool, the room lists by depth) in the same way.

## Later

When most of a subsystem is decompiled, its data moves from raw addresses into Rust types, the guest-memory calls of that subsystem go away,
and the subsystem stops depending on the console's address space.  That is what lifts the limits the static recompilation has (fixed memory
layout, 32 MB pool, tables of fixed size).
