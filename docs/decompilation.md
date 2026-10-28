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
and take back the other limits the port raised (instance and prop counts of a level, the big memory pool, the room lists by depth, the list of rooms in view of a player) in the same way.

A patch in `tools/patches.py` is dropped when the function it changes is decompiled by hand, because the hand-written function has to match the original.
To put back later, on top of the matching code: the frame-rate change of the particle streams (`particleStreamTick` used the whole frames of the game
tick, the port used the float frame time) and the changes the port made to `propGfxRoomPreproc` and `hudTick`, and the wider table of rooms by level of `bgGetRoomsTick` (the original keeps 200 one-byte room numbers a level, the port 1024 two-byte ones; `bgGfx` still has that change), and the front end's page numbers mapped through `av_front_page` (`frontPageSet`), the dual weapon combination of `frontmpgGetGunComboNum`, and the links of the sign-on menu made again from the items that were made (`makesignonmenu`: the list wraps round, and an arrow no longer vanishes when a sign-on gets no item).

The same goes for the port changes dropped from these functions when they were written by hand (all to put back later, on top of the matching code): the shortcut of `levelAvail` (all levels open), `own_get` in `gunShouldSwitchTo`/`gunPickedUpAmmo`/`getmostpowerfulcombo`/`getnextcombo`/`getprevcombo`/`playerSetStartGun` and the combination count of the added pairs (`combo_count`, `own_set`), the `custom_setup` of `lvGetARCADESetup`, the float frame time in the water and spaceship animations (`vtxlistWaterxyz`, `vtxlistWaterst`, `spaceship1PropTick`, `reflectionsPropTick`: the original uses the integer tick counter `gameframe`), `running_i` in `particleBurstTick`, the device names of the buttons in `textSize`, `textPrint` and `textOutline`, the 1000 object instances of `obReset`, the big memory pool of `memReset` and `memAlloc`, the map maker's limit of `mmUpdateMapCost`, `mmPlaceObject` and `mmDrawMemFree` (`mm_limit`, `mm_unlimited`), the menu's time step in `frontfxBgTick` and `frontfxGoodiesTick` (`menu_dt`), and the bare-handed bots of `findweapon`.

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

The verifier has been extended while the callers were written: a run of the original that reaches `exit` ends like a trap in both the interpreter and the port (`ts_rt::VERIFYING`
makes `exit` panic with "trap:" instead of ending the process), so the functions that call `memAlloc` can be checked; the allocator's globals (`mem_level_free`, `mem_level_at`,
`mem_level_end`, `mem_local_at`, `mem_eternal_free`, `mem_eternal_at`) are given room in the scratch area before each run, or every run would end in `exit`; the interpreter runs
the signalling forms of the FPU compares and `__divdi3`; and in the small-number pass a run in which the original writes round past the end of RAM (and so over its own code) is left
out like a timeout.  `TS_VERIFY_FROM=n` starts a run at test number `n` (to find a failing test).  Not machine-verified for their own reasons: `memReset` (it clears a buffer the size of
which is empty in the test memory), `profileTimeGet` and `timeTickEnd` (they return the port's clock), and the functions that call `vafmt`/`vsprintf` or a function pointer
(`challengeFormatScore`, `gameFormatTime`, `menuitemGfx`, `menuitemCalcSize`, `hudRestart`, `obFreeTextures`, `hudReset`).  `bgSmashGlass` fails its random test 4 for a reason that is not in the port: `particleNewGlass` writes zeros through the random pane data over the machine code of `soundStartRange` (words at 0x205638 and 0x205644), so the interpreter, which runs the code from memory, never queues the sound, while the port, which has no code in memory, does.

## Functions that were machine translated

Every function of `game/` is now written by hand.  The last ones that had been kept as the machine translation (`obMergePolys`, `glassSetQuad`, `playerTick`, `bulletTick`, and 62 platform functions) were rewritten with named locals, small helper functions for the repeated pieces, and the records described in the file headers.  Where the machine code spilled the address of a scratch word into the frame and loaded it again before each call (`bulletTick` keeps `sp+208` and the like in slots at `sp+244`), the hand-written code passes the address directly: the value is the same, only the stack words that nothing else reads are not written.

## Verifier: game memory snapshots

Random memory seldom reaches the deep branches of the big functions, so `ts-verify` can run on real game memory.  `TS_VERIFY_SNAPSHOT=a.bin,b.bin,...` loads a dump of the console's 32 MB (taken during play) into both machines before each run, and run `t` uses dump `t mod count`.  Three more variables make the snapshots reach more:

- `TS_VERIFY_PERTURB=1` changes the first player at random before each run (pad buttons and sticks, the movement and aim modes, the aim weights, the characters in view, the records of the radar), from the same random numbers on both machines.  This is what verified `playerTick` (6000 runs).
- `TS_VERIFY_PERTURB=bullets` fills the bullet table: with `TS_VERIFY_LONE=1` one bullet of a random kind (`TS_VERIFY_KINDS=low-high` limits the kinds), half of them shot at a character from close by, some stuck to it or with the prop of an earlier line test set.  This found the call of `updateProximityProps` that every way out of a mine's work makes.
- `TS_VERIFY_COVERAGE=1` prints which instructions of the original function the interpreter never ran, so the branches that were not tested are known.

Other switches for finding a failure: `TS_VERIFY_FAULTS=1` prints the faults of runs that are not compared, `TS_VERIFY_HISTORY=1` prints every store to the first differing byte on both sides (with the program counter of the original), `TS_VERIFY_SHOWBULLET=1` prints the perturbed bullet, and `TS_VERIFY_IGNORE=low-high` (hex) leaves a range out of the comparison.  The range is for words that the original takes from stack garbage: `bulletInitialiseStraightLineTest` puts the last hit back from words that `obGetLastHitInst` leaves unwritten when nothing was hit, and the original gets whatever an earlier call left in that stack slot, which a hand-written callee that keeps its scratch in locals cannot reproduce.  That is the only known difference of this kind (the last hit globals at 0x3aeaa0 to 0x3aeab0).

`TS_VERIFY_PERTURB` takes several modes separated by commas (`1,world,bullets`).  `world` changes words of the players' and characters' records at random (modes, flags, counters and floats, not what looks like a pointer; `TS_VERIFY_DENSITY=n` changes one word in `n`).  `partColour`, `partTransform`, `partSetTextureOffset`, `obColour`, `obTransform` and `obSetTexList` get a real model of the level as their first argument on snapshots (random ones recurse for ever), and pass.

**Whole frames.**  `ts-verify --n 600 frames` runs the frame of the game (`gameTick`, `lvTickBefore`, `lvTickPlayer` for each player, `lvTickAfter`, as `bossMainLoop` does) on each snapshot in turn, `TS_VERIFY_FRAMES=k` frames in a row (3 by default), first in the interpreter and then in the port, and compares every byte of memory that either changed.  A difference that only shows after the state has moved on is found this way, and the functions with no test of their own that the frame calls (about 400 of them) are checked as part of it.  `TS_VERIFY_COVERAGE=all` lists every function the frames went through and how much of it ran.  The sound and cache calls of the console that change nothing in the game's memory are stubbed in the interpreter; a run that reaches the disc (`sceCdRead`), a wild pointer or an unsupported instruction is skipped.  Runs whose perturbation made the original write to the first 64 KB, or leave the sound of stack garbage behind (the last hit and the wall normal of `moveStep`), compare without those words.  One difference is known and left: `playerselectanim` takes its third argument from a register (`$f31` of `playerTick`) that the original leaves unset on some paths, so in player mode 7 the original's walking speed is stale register contents.

**Checking against the original while the game runs.**  `python tools/capture-build.py --shadow` builds, in the temporary folder and without touching the sources, a copy of the port and of the verifier in which every decompiled function starts with a hook.  Copy `SLUS_200.90`, `PAK` and `MUSIC` next to the program there and run it with `TS_SHADOW=*` (or a list of function names): at the first `TS_SHADOW_FIRST` calls of each function (3) and then one in `TS_SHADOW_EVERY` (1000), the original machine code is run in the interpreter on the live memory, the memory is rolled back, the Rust function runs, and every byte either of them changed is compared (`[shadow] MISMATCH` lines, a count for each function when the run ends).  `--press FRAME:BUTTON[:FRAMES]` drives the game headlessly: pressing cross every 40 frames from frame 330 goes through the front end into a match with bots, so menus, loading, bots and shots are all checked.  `TS_SHADOW_DUMP=name` writes the memory from before the first mismatching call of that function and ends the run; `TS_VERIFY_CAPTURE=file.cap ts-verify` replays that call (the verifier built with the shadow hooks, `TS_SHADOW=callee,...`, then says which function called from it differs).  `TS_CAPTURE=name:n+m` writes the call numbers `n` and `m` of a function as such a file without waiting for a mismatch.  This is what found the bugs that random data and snapshots of running levels missed: `obMergeChunkPolys` and `propNew` (a pointer read once too often), `gunReload` (the table of guns used as a pointer) and `gunChrFire` (the wrong prop as the attacker), the first of which stopped the game from starting a level at all.  `TS_SNAP=ticks TS_SNAPFILE=prefix` in such a run writes snapshots of the match for `ts-verify frames`.  Differences that come from the original reading stack words that nothing wrote (the interpreter and the port leave different stack contents, because a hand-written function keeps its scratch values in locals) show up in `ts-verify frames` only: the same call under `TS_SHADOW=*` (the shadow scrubs the stack first) agrees function by function.  The cases seen: `timeTickEnd`, the wall normal of `moveStep`, the last hit that `bulletInitialiseStraightLineTest` puts back, the speed `playerselectanim` takes from `$f31` in player mode 7, and the gun fire of one frame in a perturbed run.

Not run with snapshots: functions that recurse without end on a perturbed model (`obSetTexList` and the others listed above), which overflow the host stack.

The verifier now runs `vsprintf` (so the text-formatting functions such as `gameFormatTime`, `challengeFormatScore` and the `hud*Gfx` ones are checked), compares NaN results by value and not by sign, seeds the allocator and screen globals before each run, and treats `exit` as a trap.  Functions that are still reviewed by hand only, each for a stated reason: `OpenXPM` (uses `sscanf`, which the interpreter does not run), `zbtestCopyZB` (calls the platform image store, which hangs), `profileTimeStart` and `profileTimeEnd` (read the hardware timer through `profileTimeGet`, so the two runs cannot agree), and the ones that fault or exit on every random run (`mapmakermain_pageTick`, `GetHeader`, `frontfxWaveyTexture`); `statsCombine`, `transformPart` and `main_pageDraw` pass only in the small-number pass because the full pass times out on large random counts.
The two functions that the generator could only write as a loop over the address of the block (`emGetNextPos`, `enemyReselectTarget`) have been rewritten by hand from their blocks; note that random data seldom reaches the deep branches of `enemyReselectTarget` (the game mode there is usually out of range), so its verification is weak there.

**Functions that call the console's routines.**  The sound, pad, memory card, graphics and thread routines of the console cannot run in the interpreter, so the functions that call them were left out of every check above.  With `TS_SHADOW_EXTERN=1` in a live run (the development build puts a hook at the start of each platform routine of `ts-sdk`) and `TS_VERIFY_EXTERN=1` in the verifier, those routines are not run while a function is checked: the original and the port both get the same made-up answers (a table in `ts-verify/src/interp.rs`, varied from check to check so that the callers' branches are reached; the memory card's answer is written through its pointer too), and the calls each makes, with their arguments, must be the same.  Addresses on the stack count as equal, since the port's locals are not where the original's were.  The disc routines, the files of the virtual disc and `sscanf` are not stubbed: the interpreter runs the port's own versions of them, so the loading functions are checked with real files.  The game misses the platform calls of the checked calls (one pad read, one sound command), so use this on a sample (`TS_SHADOW_EVERY`), not for play.  Left out on purpose: `bossMakeAll` (the port does not set up the console, see `manual/boot.rs`) and `zbtestCopyZB` (the depth buffer copy is replaced in `manual/gs.rs`); the console's debug print `scePrintf` is not compared, because the original prints a warning inside the image transfer that the port replaces.  The interrupt handlers `vblIntHandler` and `gsIntHandler` use opcodes the interpreter does not run and are reviewed only.  `TS_VERIFY_STEPS=n` changes the number of instructions after which the interpreter gives up on a run.

**Other game modes.**  The front end can be driven by `--press` as well: a new profile is made by choosing New Player on the sign-on page, adding letters with cross, moving to END and pressing cross, and then saving; with that profile, cross at frames 330, 370, 410 and 450 reaches the game type page, up and cross start the story, cross opens the arcade, down and right on the arcade level page choose the game mode and the level, and screenshots at chosen frames (`--screenshot`) show where a script is.  Runs of the story and of the arcade modes (deathmatch, bag tag, capture the bag, knockout on levels 1, 3 and 6) with random pad input, and of the statistics and audio pages, found one more bug: `arcade_pageTick` asked for the map file before adding the map maker's items, which the original does not.  Two to four players go through the title page (right once for each extra player, then cross), where every player loads a sign on with the pad of the same number (`--press FRAME:BUTTON:FRAMES:PAD`); each needs a profile of their own, so make as many as there are players first, and give the later players a few frames more for each "Loading" box.  Runs of two, three and four players on the arcade level Tomb with random input from every pad also found nothing.  `TS_VERIFY_STORES=hexaddress` in a replay prints every store of both sides to that address, and `TS_VERIFY_SHOW=n` shows more of the differing bytes.

**A profile from nothing, in one script (checked 2026-10).**  With an empty `--save-dir`: `cross` at frames 300, 340 and 380 (the box "No TimeSplitters Sign Ons", then the page), `up` at 480 and `cross` at 520 (New Player), `cross` at 640 (adds the letter A), `down` at 680, 700, 720 and 740 and `right` at 760, 780 and 800 (the cursor on END), `cross` at 840 (END), then `cross` at 1000 (MY SAVES), 1050, 1100 and 1150, which ends on the arcade level page with Deathmatch chosen (each press `:cross:6` long, or 4 for the d-pad).  Copy that save folder; with the copy, `cross` every 40 frames from 330 to 1000 (`:cross:3`) enters a Deathmatch with bots.  `TS_BOTSET=80,81` (the characters of custom bot set 1, set outside levels) then puts those characters in as the bots: the match runs with a character that a mod added (the radar shows both bots).  `--level` starts no bots at all.  The Options page of a mode (with a mod's `modes.txt`: the mode's name, its numbers, the Rules row) is on screen from about frame 580 when `cross:3` is pressed at 330, 370, ... 530 only (up and right move on it).  An earlier version of the aid that also wrote the team flags at 0x1fc1234 corrupted the match (an unmapped address), so it does not.

**Challenge mode.**  With the profile's unlock_all set (`--set unlock_all=true`) the game type page has a third item, Challenge: from the page, down and cross open the challenge list, right steps through the challenges, and cross twice more reaches character select and the start.  The list has 24 challenges (the screen after 24 steps shows the first one again).  Runs of all 24 (with random pad input and the live check, `*` every 25 frames) and of story levels 4, 6 and 8 and the unlocked arcade levels 4-4, 5-4, 4-7 and 5-7 show no difference at all since the float clamps below; before them the animation and tick functions (`animUpdate`, `calMatrices`, `lvTickBefore`, `propTickBefore`) showed the overflow differences, and challenge 24 once showed `lvTickBefore` differing at one stack word, which that also removed.

**The console's arithmetic.**  The console's floating point unit has no infinity and no NaN: a division by zero (also 0 / 0) gives the largest float with the sign of the result, and the square root of a negative number is the root of its absolute value.  Plain host arithmetic gives infinity or NaN there, and a NaN position made the ray of `obinstCalcAmbientLight` (the ambient light of an object) walk the faces of a room for ever on the story level 5, with the game stuck.  `ts_rt::ps2float` has the console's division and roots (`a.pdiv(b)`, `x.psqrt()`, `ps2float::rsqrt`); the interpreter of `ts-verify` uses the same, and every float division and square root of the hand-written code was changed to them (`pdiv` on an integer is a compile error, which is how the integer divisions were told apart).  Every float addition, subtraction and multiplication of the hand-written code now goes through `ps2float::add`, `sub` and `mul`, which clamp an overflow to the largest float with the result's sign as the console does (the rewrite was done by a script and audited; a hand-written line using plain `+ - *` on floats is a difference to fix).  The interpreter clamps cop1 and vector unit operations the same way, `TS_VERIFY_CLAMP=0` turns that off for a run, and `tools/verify-game.py` no longer does.  The vector helpers of `ts-sdk` (`vu0.rs`) clamp too.  A live check counts a word that is NaN, infinity or the largest float on both sides as equal.  A division written over two lines was not caught by the division rewrite: look for ` / ` on a line of its own when adding code.

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
