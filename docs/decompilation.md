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


**Read the generated code with the patches in mind.**  `tools/patches.py` changes the generated code after it is made, and some of its changes apply to every function (`GLOBAL`: the
frame counter read as `gameframe_f()`, the weapon and bot set reads as `weaponset_effective()` and `chrset_effective()`, the relocated tables).  A hand-written function must do what the
**original** does: `lw(g::gameframe) as f32`, `lw(g::gv_weaponset)`, and so on.  `register.py`-style checks list the per-function patches of a function; a per-function patch is removed
(and noted below) when the function is written by hand.  The verifier compares with the original machine code, so a copied change shows up only if the random data reaches it.

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

A patch in `tools/patches.py` is dropped when the function it changes is decompiled by hand, because the hand-written function has to match the original.
To put back later, on top of the matching code: the frame-rate change of the particle streams (`particleStreamTick` used the whole frames of the game
tick, the port used the float frame time) and the changes the port made to `propGfxRoomPreproc` and `hudTick`, and the wider table of rooms by level of `bgGetRoomsTick` (the original keeps 200 one-byte room numbers a level, the port 1024 two-byte ones; `bgGfx` still has that change), and the front end's page numbers mapped through `av_front_page` (`frontPageSet`), the dual weapon combination of `frontmpgGetGunComboNum`, and the links of the sign-on menu made again from the items that were made (`makesignonmenu`: the list wraps round, and an arrow no longer vanishes when a sign-on gets no item).

## The platform layer

Functions that only drive the console (system calls, the memory card, the CD, the clock, the GS, the sound chip, the start-up and exit code, the interrupt handlers) are not decompiled:
the port replaces them (`ts-port`, `ts-sdk`).  They are left as generated code and not counted.

## Functions the verifier cannot check

`tools/verify-game.py` runs each function on random memory in the original machine code and in the Rust.  Some hand-written functions are **not** machine-verified (the script prints the current list at the end of each run; it was sixteen when this was written, and grows with the functions that call the C library or a function pointer, which the interpreter does not run: `memFree`, `gvSetValue`, `gameGetTeamName`, `padFlagsToString`, `gameFormatBagTagTime`, `FDiv`, `demoFileName`, `hudGetBagIconFileName`, `gettextfromfunction`) and are only
reviewed against the generated code: `memMark`, `memMake`, `obSetTexList`, `partSetTextureOffset`, `partTransform`, `obTransform`, `partColour`, `obColour`, `calDoubleFilletTrans`
(recursion over a model that the random data makes endless overflows the host stack), `pickupposGfx` (a loop over a random count takes minutes), `mmAllocTile`,
`mmMakeTileRotations`, `bgTileSetTextureNums`, `texinfoRelocate`, `padAddTilePads` and `dlDrawTriangleStrip` (the interpreter stops at an instruction it does not run: a special,
syscall or vector opcode), and `obMergeChunkPolys` (a random count makes the write log run out of memory).  Earlier the second pass of the script ran all the skipped functions in
one process, and a crash in it hid the rest; it now runs one process for each, and the list above is what it reports.  The way to check them is to replay real game memory (record the
arguments and memory at a call during play, run the original on it, compare).

## Transliterated functions

Two functions are too long and too tangled to understand yet, and are kept as the machine translation with the frame's scratch words turned into named locals (`slot_N`) and the
names of the original's registers: `obMergePolys` (`polygon-merging.rs`, matches the original on the verifier's small-number pass) and `glassSetQuad` (`glass-shard.rs`; the verifier
does not pass stack arguments, so every run faults and it is **not** machine-verified).  They are counted as decompiled but should be rewritten when the structures they use are typed.
The two functions that the generator could only write as a loop over the address of the block (`emGetNextPos`, `enemyReselectTarget`) have been rewritten by hand from their blocks; note that random data seldom reaches the deep branches of `enemyReselectTarget` (the game mode there is usually out of range), so its verification is weak there.

## Verifier: faults past the port's RAM

`ts-verify` ignores a native fault on an address at or above 0x0800_0000 when the interpreter ran the original function to the end without a fault.
The interpreter wraps addresses past the end of the port's RAM, the port faults on them, so a function given garbage pointers (first seen with
`obCalcChunkBounds`) failed for a reason that is not a difference in the code.  The guard could hide a real bug where the port reads a wild address that the
original does not.  To do later: give the interpreter and the port the same behaviour for such addresses (both fault, or both wrap), then remove the guard
in `ts-verify/src/main.rs` and check that no function fails.

## Later

When most of a subsystem is decompiled, its data moves from raw addresses into Rust types, the guest-memory calls of that subsystem go away,
and the subsystem stops depending on the console's address space.  That is what lifts the limits the static recompilation has (fixed memory
layout, 32 MB pool, tables of fixed size).
