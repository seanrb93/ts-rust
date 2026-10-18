# ts-rust — TimeSplitters (PS2) native Rust port

A native PC port of the PS2 game *TimeSplitters* (`SLUS_200.90`). The game's MIPS (R5900) code is **statically
decompiled to Rust**; it is not run in an emulator and the PS2 hardware (EE, GS, VUs, DMA, SPU2, IOP) is not emulated.
Hardware services are replaced by native ones.

You need your own copy of the game. Its files (`SLUS_200.90`, `PAK/`, `MUSIC/`) have to be in the same folder as the executable
(`ts-port.exe`); without them the program says so and stops.  The saves are in a `saves/` folder there too.
**Never commit game files.** The game's files and `saves/` must stay out of any repository.

## Build

The game's code is not in this repository.  It is generated on your machine from your own copy of the game: put the game's `SLUS_200.90`
in a folder `Game Files/` next to `tools/`, then (Python 3):

```
python tools/gen_rust.py        # writes ts-rust/crates/ts-game/src/dec/ (the game's code as Rust)
python tools/vu1dec.py          # writes ts-rust/crates/ts-gfx/src/vu1_gen.rs (the graphics microcode)
```

Changes to the generated code are kept in `tools/patches.py` and applied by `gen_rust.py`.

## Run

```
cd ts-rust
cargo build -p ts-port --release
cd ..
ts-rust/target/release/ts-port.exe                      # front end, in a window
ts-rust/target/release/ts-port.exe --level Village      # jump into a level
```

The executable's folder holds `PAK/`, `MUSIC/` and `SLUS_200.90` from the disc.  The game's initialised data (strings, tables, defaults) is
read from `SLUS_200.90` at every start (only its data, no code of it is used).  A crash or GPU error is written to `ts-port-crash.log` next to the port.

Options: `--game-dir DIR`, `--level NAME`, `--players N`, `--save-dir DIR` (virtual memory card, default `saves/` in the executable's folder; `--game-dir` and `--save-dir` are for testing),
`--headless`, `--frames N`, `--screenshot FRAME file.png`, `--press FRAME:button[+button][:frames]`,
`--wav file.wav` (headless audio dump), `--dump-frames a,b`, `--profile A B`.
Level names: Prison, Village, NSA1, Outskirts, Chemical Plant, Asylum, Mansion, Planet-X, Compound, Streets,
Warzone, Site, Cyberden, Bank, Graveyard, Spaceship, Mall, Chinese, Castle, Tomb, Docks, ...

Keyboard and mouse (port 0): W, A, S, D walk (Shift walks slowly), the mouse looks, its left button fires (R2), its right button is R1 (alternate fire), its
middle button aims (L2), the wheel changes the weapon (or zooms while aiming).  Keys: Enter / Space / F cross (accept, use), Backspace / R triangle (back,
reload), C circle, X square, Ctrl L1 (crouch), Z L2, V R1, G R2, Q / Left Arrow and E / Right Arrow d-pad left and right (the previous and next weapon, and
left and right in the menus), Up and Down Arrow d-pad up and down (zoom), Esc / P start, Tab select; F11 window / borderless, F2 the game's developer (debug) menu in
a level once it is switched on in Game Options (see "Developer menu"), F10 quit.  The keys of every button are the `key_*` lines of `ts-port.ini` (`key_cross =
Enter, Space, F`: names separated by commas); the game's texts name them ("(F) Select"; where a button's picture has little room the first key of the list
that has one or two characters).  Gamepads (any, via SDL3) map to the virtual DualShock 2; the first two feed ports 0 and 1.

## Display and graphics options

The settings are rows of the game's own **Audio / Video Options** page (main menu, "Audio / Video Options"), below the music and
sound settings; the old "Screen Adjust" row is gone.
Changes apply at once and are saved to `ts-port.ini` next to the executable (created on first start, commented; `--set key=value` on the command line overrides it for one run).

| Option | What it does |
|---|---|
| `aspect` | `auto` (follow the window), `4:3`, `16:10`, `16:9`, `21:9` or a number. Wider than 4:3 shows more of the world to the sides (the game's own camera and culling are widened, nothing is stretched). |
| `fov` | Field of view in degrees, as it would be on a 4:3 screen (0 = the game's own); wider pictures show more at the sides at the same height. Applies in levels only. |
| `hud` | `edges` (original proportions, in-game HUD sticks to the screen edges, menus centred), `center`, or `stretch`. |
| `render_scale` | Internal resolution relative to the window (2.0 = supersampling). |
| `internal_resolution` | The height of the internal picture in pixels, independent of the window: 0 (the window's size times `render_scale`), 480, 720, 1080, 1440 or 2160; the width follows the aspect ratio (480 at 4:3 is the console's 640x480), and the picture is scaled to the window. Row *Internal Res.* of Video Options. |
| `upscale_sharp` | `true` scales the internal picture up with sharp pixels instead of smoothing it (for 640x480 in a big window). Row *Upscaling*. |
| `msaa` | 1 / 2 / 4 / 8 samples (limited by the GPU). |
| `texture_filter`, `anisotropy` | nearest / bilinear / trilinear (mip maps), anisotropic filtering up to 16x. |
| `window_mode` | `windowed`, `borderless` (a window without borders over the whole desktop) or `fullscreen` (the monitor is switched to the video mode of `fullscreen_width` x `fullscreen_height`, 0 = the desktop's; the mode with the highest refresh rate of that size is used). The old `fullscreen = true/false` still reads as borderless / windowed. |
| `width`, `height` | Window size in windowed mode (also set by dragging the window's edge or by the Resolution row). |
| `vsync` | as named. |
| `stick_deadzone`, `stick_sensitivity` | Gamepad sticks, with PCSX2's defaults: dead zone 0 (none; the game has its own) and sensitivity 1.33 (the stick's range is stretched by a third and cut off at the ends, so a round modern gate reaches the console pad's values). Both sticks. |
| `framerate` | Game frame rate while a level is played: `60` (the original), a higher number, or `display` (the monitor's refresh rate). Above 60 the game logic itself runs at that rate (movement, aiming, animation and effects are all advanced by the real frame time, so no in-between pictures are needed (the old frame interpolation option was removed); menus, loading, pause and the retry screens stay at 60. Frame times are measured, so a PC that cannot keep up simply gets a lower rate at the same game speed. See "Frame rates above 60 Hz" below. |

## How it works

```
tools/ (Python)                ts-rust/ (Rust workspace)
 ps2elf, r5900, disasm   --->   ts-rt     flat 32-bit address arena at the original addresses, loads ELF data
 decomp/ (IR, liveness,         ts-game   GENERATED dec/*.rs (1978 functions) + hand-written manual/
  webs, structuring, emit)      ts-sdk    native replacements for Sony SDK, libc/libm, soft-float, VU0 helpers,
 gen_rust.py  (~9 s)                      virtual disc, pad, memory card, SIF/IOP RPC
 vu1dec.py -> ts-gfx/vu1_gen.rs ts-gfx    VIF/GIF display-list reader + decompiled VU1 microcode -> draw lists
                                ts-render wgpu renderer (4:3 letterbox of the 640x224 field)
                                ts-audio  SPU2-style voice mixer + ADPCM music streamer
                                ts-port   launcher: window (winit), input, audio device (cpal), debug tools
                                ts-verify DEV ONLY: tiny R5900 interpreter for differential testing
```

Pipeline (`python tools/gen_rust.py`): decode R5900 -> register IR -> interprocedural signatures (EABI) -> def-use
webs (variables) -> inlining / dead-code / type inference -> structured control flow -> Rust. Regenerate after any
decompiler change; names defined in `ts-game/src/manual/*.rs` or `ts-sdk/src/native/*.rs` win over generated ones.

Frame path: decompiled game code builds a DMA display list -> `ts-gfx` walks VIF packets, runs the **decompiled VU1
microcode** (not an interpreter) for vertex transform/clip, interprets GIF packets into draw calls -> `ts-render`.
The VU1/GIF stage is a game-specific replacement of the command stream, not a general GS emulator.

The VU1 translation (`tools/vu1dec.py`) is the main cost of the graphics thread, so it is kept lean: the Q register is
resolved only where the program reads it (`Regs::q_now`), a MAC flag update is only computed when an `FMAND/FMEQ/FMOR` can
still read it before the next update (`mac_liveness`), and results are clamped with SSE (`fz`, `fz4`) while the graphics thread
runs with flush-to-zero / denormals-are-zero set (`enter_vu_mode`).  The timing counters that `TS_BENCH` prints cost clock reads
in the inner loops, so they only run when a report asks for them.  Together this is about 3x faster per vertex than the first
translation (a heavy level frame of 60000 vertices takes about 4.5 ms to translate).

The page has two columns of evenly spaced lines (audio and display settings on the left, quality settings on the right); 15 lines
do not fit one column at a readable spacing with the game's fonts.  Up / Down move through the rows and Left / Right jump to the other column.  X starts editing the row under the cursor: its two
arrows appear, Left / Right then change the value (the volume sliders move in steps of 5 %), the cursor stays on the row, and X or
triangle ends the editing (triangle does not leave the page then).  The hint line at the bottom shows "Edit" or "Change / OK".
The same page is in the pause menu too: the SETUP tab has a fourth item, "Audio / Video Options...", with a page description of its
own (`audiovideo_pageTick` serves both places; in the pause menu it uses the small menu font and a table without the front end's
back button but with the invisible item that makes the triangle button go back).
An **Exit Game** item ends the program: it is the last item of the front end's game type page and of the pause menu's SETUP page.
It opens a "page" at an address of the port's own, which `frontPageSet` / `inPage` (patched) turn into a confirmation page.  In the
front end it is laid out like the game type page (the question, then Yes and No centred in the large menu font, "Back" and "Select"
legend at the bottom; the table is built by `confirm_menu`, which also does the Back / No choice itself with `pageOut`).  In the pause
menu it is the game's own end game page with Cancel / Confirm (`confirmendgame_pageTick`, patched: while the flag
`CONFIRM_GAME` is set, Confirm ends the program and Cancel / triangle return to SETUP).  Only a confirmed choice ends the program.
The page is built by `ts-game/src/manual/avmenu.rs`: the game's menu items accept pointers to text for labels and value names,
so the port keeps its own copy of the page's item table (in RAM above the console's 32 MiB), adds one item per setting
(`menu::PortAv` supplies the rows and takes the changes) and drops the screen-position item; `patches.py` makes
`audiovideo_pageTick` use it.

## Frame rates above 60 Hz

The game measures how long a frame takes in 60 Hz *ticks* (`updateframesf`, a float that scales movement, animation and
timers; `updateframesi`, the whole ticks, for counters and timeouts) and was written to cope with frames of 1-5 ticks.  With
`framerate` above 60, `timeTickStart` (`ts-game/src/manual/boot.rs`) hands the game fractions of a tick instead:
`updateframesf` is the frame time, `updateframesi` is 1 on the frames that complete a whole tick and 0 on the others, and
`gameframe_f()` / `tickframenum_f()` give the continuous time for animations.  Frames are paced by `wait_frame`
(`ts-sdk/src/native/gs.rs`); a frame that takes longer than its slot stands for the real time it took.  Only a level that is
being played with no menu on top runs at the high rate (`hirate_ok`); everything written for exactly 60 Hz (menus, demos,
loading, pause) keeps its frame-by-frame logic.  The front-end menus redraw at the high rate too: their page logic and
pad sampling still run once per 60 Hz tick (`menu_skip` frames skip `gameTick`/`lvTickBefore`/`lvTickAfter`/`joyTick`, but
still run `memdbTick`, the per-frame arena reset), while the animated background, logo glow and unlock-page effects advance
by the frame time (`menu_effects`, `menu_dt`).  Two things had to be handled for the frames in between: the packets that the logic of a tick builds for the 3D previews
(characters, MapMaker tiles) live in the per-frame arena, so a frame that lands in the same half as the tick continues after
the logic's allocations instead of starting over (`menu_memdb_tick` / `menu_memdb_mark`), and the few places where drawing
itself moves something (`drawChar` adds the slide of the characters as it draws them) only do so on tick frames (`patches.py`).

Code that treats `updateframesi` as "the game is not paused" (bullets, firing, hits, sounds, animation changes) would lose
events on frames without a whole tick, so `tools/patches.py` rewrites those tests to look at the frame time instead and turns
a few integer-tick animation phases into continuous ones.  The patches are applied by `gen_rust.py`, each is checked to
match exactly once.  Per-tick random effects (flicker, smoke) keep their whole-tick gates so that their rate does not change.

Debug aids: `TS_TICKCLOCK=1` makes `--press` / `--screenshot` / `--profile` / `--frames` frame numbers count game ticks
(starting at 1000000) while a level runs, `TS_NOPACE=1` removes the frame wait, `TS_SNAP=t1,t2` + `TS_SNAPFILE` dump RAM at
game ticks (compare runs at different rates word by word), `TS_WATCH=addr` + `TS_WATCHFROM=tick` (calltrace build) names
the functions that store to an address, `TS_PLAYERLOG=1` prints the player's state every frame.

## Verification

- `ts-rust/target/release/ts-verify.exe --n 10`: runs decompiled functions and the original machine code on the same
  inputs. At last run: 1634 functions identical, 6 intentional differences (the timer replacements `timeGet`, `profileTime*`, `timeTick*`), 289 skipped (infinite loops on random data, or SDK calls with no model). The functions changed by `tools/patches.py` (high frame rate support) differ from the machine code by design; check the verifier with those patches disabled. The interpreter now also runs VU0 macro code, so the hand-written VU0 natives are covered.
- `tools/vu1ref.py` is an independent VU1 interpreter used to cross-check `vu1_gen.rs` against dumps (`TS_VUDUMP=dir`).
  It found real bugs (FTOI/ITOF destination register, `WAITQ` stalling the whole instruction pair).
- `cargo build --release -p ts-port --features ts-gfx/vucheck --target-dir target_vc`: every VU1 program run is done twice,
  by `vu1_gen.rs` and by the untouched first translation kept in `vu1_gen_ref.rs` / `vu_ref.rs`, and the memory, registers
  and kicked packets are compared (a mismatch panics with the difference).  Used after every change to `tools/vu1dec.py` or
  `ts-gfx/src/vu.rs`; a run of a few levels and the menus checks several million program runs.
- The decompiler used to compare 128-bit registers (MMI code keeps some 32-bit values there) as unsigned numbers, which broke
  signed tests such as the MapMaker's "is this tile inside the map" (tiles could not be placed in the lower half of the
  grid).  Such comparisons now use the low 64 bits as a signed number, as the machine does (`tools/decomp/emit.py`).

## Status

Working: boot, front end, menus, level loading, 3D rendering with textures, HUD, pad/keyboard input, virtual memory
card, music streaming, sound effects (voices, ADSR; output only checked as non-silent, not by ear).

Notes: levels Prison and Asylum load a plain test-grid map and the `Test` level crashes in `bgPortalCrossed` on a bad
pointer (the decompiled function passes `ts-verify`, so this is likely bad/missing level data, not a decompiler bug).

Known gaps / TODO:
- Music runs at 44100 Hz (the stream driver programs SPU pitch 0xEB3); reverb, noise and pitch-modulation are not implemented because the game never uses them (it sets the effect volume to 0 at start-up and never routes a voice to the effect unit, and issues no noise or pitch-mod switches).
- Screen-as-texture effects (image processing: invert, defocus, motion blur, fisheye) work through frame-buffer snapshots in `ts-render`; fisheye shows a thin centre line and some moiré. The z-buffer-as-texture debug view is not implemented. (Depth read-back for light-glow occlusion is done on the CPU: `ts-gfx/src/depth.rs`, exactly the previous frame's: the game waits for it in `depth_request`, because its glow tests pair glow positions with the depth of the frame they were drawn in; a stale or mismatched depth made glows show through walls and flicker at high frame rates).
- Interlace/resolution/widescreen enhancements; mouse look; key remapping.
- Remaining generated SDK stubs (`ts-sdk/src/stubs.rs`) are SDK internals the port never reaches; unimplemented stubs
  log once and return 0. 7 functions were left irreducible by the structurer; check `gen_rust.py` output.
- Multiplayer/split-screen and the second memory-card port are untested.

## Debugging

`cargo build -p ts-port --features calltrace` enables a call stack watchdog (dumps if no frame for 4 s) and
`TS_TRACE=fn,..`. Other env vars: `TS_DRAWS`, `TS_BBOX`, `TS_DUMP_DRAW=n`, `TS_ONLY/TS_SKIP/TS_FROM`, `TS_VUTRACE` (needs the `vutrace` feature of ts-gfx), `TS_IP=fisheye|blur|invert|defocus|viewzb` (force an image-processing effect), `TS_CDLOG`, `TS_SCREENLOG`, `TS_MCLOG` (memory card calls), `TS_ZLOG` (glow depth probes), `TS_AUDLOG`, `TS_SPULOG` (sound key-ons), `TS_SHOTW`/`TS_SHOTH` (screenshot size), `TS_PAGELOG` (front-end page stack), `TS_ALPHALOG` (texture alpha histograms), `TS_LEGACYALPHA` (screenshot A/B), `TS_OVERLAY` (overlay text in screenshots), `TS_NOSHIFT` (screenshot A/B of the texture sampling shift), `TS_VERBOSE` (periodic status lines on stderr; off by default because console writes can stall a frame), `TS_PERFLOG` (print the F3 statistics every 120 frames), `TS_SPIKELOG` (print frames over 26 ms with a stage breakdown), `TS_PIPELOG` (log render pipeline creation), `TS_BENCH` (headless render benchmark), `TS_REPEAT` (re-translate heavy display lists to time them; also prints the cold first run), `TS_VUPROF` (time per VU1 program entry), `TS_STRESSFX=<period>` / `TS_STRESSN` / `TS_STRESSDIST` / `TS_STRESSKIND` (set off explosions in front of the player in a level, to test particle-heavy scenes), `TS_SHOTRAW=1` (screenshot frame numbers are raw frame counts), `TS_FRAMESNAP=frame:file,..` (dump RAM after a frame, to diff what a redraw changes), `TS_SYNC_GFX=1` (run graphics on the game thread instead of the two worker threads), `TS_SHADOW_REAL=fn,..` (shadow testing: run these library functions as real machine code), `TS_INTERPT=<0..1>` / `TS_INTERPMULTI` (screenshot of the blend of the previous and current frame), `TS_VIEWLOG` (log the camera constants used for widescreen).

**Live differential testing** (`cargo build --release -p ts-port --features shadow`, use a separate `CARGO_TARGET_DIR`): `TS_SHADOW=fn1,fn2,..` runs the original machine code (ts-verify's interpreter) on the live game state at every outermost call of the named functions, rolls memory back, runs the decompiled Rust and reports any byte that differs (`[shadow] MISMATCH`; counts printed every 300 frames). Naming `gameTick`, `lvTickPlayer`, `lvGfx` checks whole frames of game logic. Also covers the hand-written VU0 natives (`quaternionSlerp`, `matrixVec*Aligned`, ...).
Run Python tools as files (`python tools/gen_rust.py`), not with `python -I`.

## Developer menu (F2)

The retail game still contains its developer menu but nothing opens it; the **Debug Menu (F2)** row of the Game Options page (`debug_menu` in `ts-port.ini`, off by default) lets F2 open it while a level is played (and close it).
Triangle or start goes back one menu.  Most of its entries were compiled out of the retail code (the portal / wall / volume / floor
overlays, position, speed and performance displays, memory statistics, editors, turtle switches), so only the working ones are
shown (`ts-game/src/manual/debugmenu.rs` trims the menu tables before the first use): Move View, Game View, 3rd Person View, Prop
View, View Items (View Guns, View Background), Cheats (Invincible, Invisible, Noclip) and Exit Level.  The entries show their state
("Noclip: On") in their own text: the On / Off displays of the original returned nothing because the generator took their
return values for unused (`RETURNING_CALLBACKS` in `tools/decomp/analysis.py`).  Noclip flies the player through the level (switching it on closes the menu, which otherwise takes the sticks): the
stick that walks moves along the view direction and sideways, circle (F) rises and cross (Space) sinks (the game centres the
look angle again while you walk, so looking up does not climb); `moveTestDoSlide` (the game's move-against-the-level
test) is skipped for the player's own move.  Everything the menu changes is reset when a level is loaded.

## Option pages

The game type page (and the SETUP page of the pause menu) lead to separate pages: **Controls** (configuration, inverse look, aim mode, crouch
mode, auto lookahead, auto aim, weapon change, vibration, vibration mode: the game's Setup Controls and Preferences pages in one),
**Audio Options**, **Video Options** (the port's graphics settings) and, in the front end only, **Game Options** (Unlock Everything, Debug Menu, Dual Wield Everything, Bots Punch, FPS Counter,
View Statistics, and View Credits once the game has been completed).  All of them work the same way: X starts editing the row under the cursor
(its arrows appear), left / right change the value, X or triangle ends it; left / right otherwise move between the columns or the pages.
Choosing X on the Custom configuration of the Controls page (after editing it) opens the page where each button is chosen.  `avmenu.rs` builds the tables.

The Controls page also has **Look / Mouse Sensitivity** (the right stick and the mouse) and **Control Method** (Controller or Keyboard & Mouse; `input_method`
in `ts-port.ini`; both devices always work, and the method follows the one used last; choosing the other one by hand holds until that device is used).  The sensitivities scale the look speed in `playerTick`.  Button names in legends and in the Custom controls page follow the device
in use: Xbox letters and LB / RB / LT / RT, the Switch's letters and L / R / ZL / ZR, or the keys of the keyboard; a PlayStation controller keeps the
original pictures (`nativetext.rs`, `Pads::styles`).  The same file replaces the texts that mention the PS2 memory card, so the saves read as the
program's own saved data.

The **Audio Options** page has a second page (Left / Right) with three more volume sliders, like the game's own: **Weapon** (gunfire, explosions, reloads, punches and what
they hit: bullet impacts, ricochets, bodies falling, glass), **Voice** (characters and announcers) and **Environment** (everything else: ambience, machines, pickups, menus),
from 0 to 100 (`volume_weapons`, `volume_voices`, `volume_environment` in `ts-port.ini`).  The game's own Sound volume still applies to all of them.  The mixer (`ts-audio`)
asks, when a voice starts, which group the sample at its address is in; `manual/soundmix.rs` finds the sound in the game's table of sounds and sorts it by the words in its file
name.  The **FPS Counter** row of Game Options shows the frame rate in a corner.

The **Options** page of an arcade match (the page with the match's settings, before GO!) has **Level Music** after Bot Set: *Level's Own* (what each level plays, the default), *Random*
(a different track each time a level starts, never the same one twice in a row) or one of the 27 tracks (`MUSIC\1001.MSC` to `1027.MSC`, each named after the first level that uses it).
It is `level_music` in `ts-port.ini` (`own`, `random` or a track number) and applies to every level, MapMaker maps included; the pages of the front end keep their own music.  The level
table (`lvinfo`) says which tune a level plays; `musicRestart` is patched to call `av_level_music` on the tune it asks the table for (`manual/arcadeopts.rs`, which also adds the row: the
page's table is a copy in the port's memory, as it has no room for another item).

The MapMaker's Options page works like the others (X edits the Grid Intensity slider, arrows show) and has **No Limit** (Yes / No): Yes removes the
limit on what a map can hold and hides the FREE bar (`mmoptions.rs`, `mapmaker_unlimited` in `ts-port.ini`).

## Dual versions of every weapon, and story with more players

The weapon page of the custom weapon sets lists a dual version ("Uzi (x2)") after the weapons for each weapon that the game only has in one hand
(`manual/dualwield.rs`, `manual/weapontable.rs`).  The dual versions are weapon combinations of their own, like the game's own pairs
(Pistol (x2), ...): the table of the combinations (`guncombos`, 33 entries, with `gunsinventory` right behind it) is moved to the port's memory
(`GUNCOMBOS` / `GUNSINV`, `tools/patches.py` GLOBAL) with room for 64, and a combination with the same gun in both hands is added for each of the
16 weapons that have none (numbers 33 to 48, the rank a little below the single weapon's, after it in the order of the cycling, the name with
" (x2)").  A set can therefore have a weapon and its dual version, with separate pickups.  What the game keeps per combination in other places had
room for 33 only: what a character owns (`chr + 488`, right before the ammo) is kept for the added ones in a table of the port (`own_get`, `own_set`,
`own_clear`, patched into `gunShouldSwitchTo`, `gunPickedUpAmmo`, `pickupPropTick`, `playerSetStartGun`, `chrClearGuns`), the statistics count a
dual version with its single weapon (`stats_combo`), the cycling (`getnextcombo`, `getprevcombo`, `getmostpowerfulcombo`) and the range checks of
`pickupPropTick`, `propDrop` and `playerSetStartGun` use `combo_count()`.  With the **Dual Wield Everything** option off the added versions are not
listed and an entry of a set that is one is the single weapon.  Test aids: `TS_COMBOLOG` (the table, the weapon and what player 1 owns, and every
pickup of an added version), `TS_GIVE=<frame>:<combo>`.  Left hands: `gunUpdate` only let a hand take its turn when the weapon had a clip, so rockets,
the blunderbuss, mines, TNT and the brick fired from the right hand only; the turn is now taken from the hands' timers for those weapons too (patch),
and the dynamite and the brick are rolled over by 95 degrees in the left hand (their models are not mirrored by the attachment).  The weapon page's
list box is wider and the selected-weapons box (with the set's value and arrows) is 24 units further right.  With four players the black frame around
the panes is only the gap between them.

## Two fists

Unarmed, the player attacks with an arm model (a prop of its own) that plays a swipe; the game has no animation for the left arm.  Every
second swipe is the same swipe mirrored left to right (`punch_swing`, `punch_mirrored` in `manual/dualwield.rs`): `propGfxRoom` writes the GS
register LABEL (`dlLabel`) with `MIRROR_LABEL` before and 0 after the arm's picture, and `ts-gfx/gif.rs` mirrors the vertices drawn in
between (x -> 640 - x).  The hit test of the game is unchanged (the left swipe hits where the right one would).

## Dual wield toggle

The **Dual Wield Everything** row of Game Options (`dual_wield_all` in `ts-port.ini`, on by default) switches the added "(x2)" weapons and the left
fist on and off together.  Off, the weapon page lists only the game's own weapons, dual entries already in a set act as the single weapon, and
unarmed punches are all right-handed.  The left fist's hit test (`punch_mirror_points`) mirrors the two points of the swipe across the plane through
the player along his view direction.

## Five custom bot sets, and bots without a weapon

The bot set choice has five custom sets (Custom 1 to 5; `manual/botsets.rs`, the same way as the weapon sets: the sets 3, 4 and 5 share the
storage of the game's third and are swapped in whenever the game asks, kept in `bot_sets.txt` in the save folder).  The Square button on
the bot set page fills a custom set with ten random bots (the level only has room for some; the others show a red cross).

A bot that starts without a weapon (Always start with gun: No) looks for one (`findweapon`).  In a level with no weapon to pick up (an empty
custom weapon set) it used to stand still playing its idle animation; it now goes after its target and hits it bare handed when it is close
(`bot_punch` in `manual/dualwield.rs`, called from `enemyTick`: one punch every 0.75 s within two and a half units on the same floor, never while the bot is dying or spawning or at a dead target, damage as the player's fist).  This is the **Bots Punch** row of Game Options (`bots_punch` in `ts-port.ini`, on by default; off, bots stand idle as in the original).  In modes with teams the bot set randomiser also puts the bots in random teams.

## Randomisers

On the arcade character select page the Square button gives the player a random character among those that can be chosen (the legend
shows it).  On the weapon set page (Weapon Set set to Custom), Square with the cursor on the set, not while editing it, fills the custom set
with five different random weapons of those available (on the bot set page Square fills a custom bot set with six random bots); in edit mode Square still removes a weapon.  `randomiser.rs`; the hooks are in
`tools/patches.py` (`chrplayer_pageTick`, `weapons_pageTick`).

## Custom weapon sets

The weapon set choice has five custom sets (Custom 1 to 5) in front of the presets.  The game has only one (`gunzpack`), so
`manual/weaponsets.rs` swaps the chosen set in and out of it whenever the game asks which set is chosen (the patch replaces the game's reads
of `gv_weaponset` by `weaponset_effective`, which answers 0 for any custom set).  The sets are kept in `weapon_sets.txt` in the save folder
(the console game forgot its custom set when it was switched off).

## Mods

Every folder in `mods/` next to the program is a mod (folders starting with `.` or `_`, or with `enabled = false` in a `mod.ini`, are skipped).  The **Mods** page of the main menu (between Game Options and Exit Game) lists the folders of `mods/` and switches
each On or Off (it writes `enabled = true` or `false` into the mod's `mod.ini`, keeping the other lines).  The mods are loaded when the program starts, so a change shows as "(after
restart)" until the game is started again.
A mod can hold:

* `PACK.PAK/<path in the pack>`: a file that replaces the file of that name in the game's pack (`ANIM.PAK`, `BG.PAK`, `FRONTEND.PAK`, `MISC.PAK`,
  `OB.PAK`, `SFX.PAK`) or is added to it.  Everything the game loads (models, textures, animations, sounds, levels) is read by name from
  a pack, so characters, weapons, sounds, textures and levels are all files like this.  A path that does not start with a pack's name replaces the
  file of that name in whichever pack has it (`sfx/pistol.vag`).  Names are lower case and at most 47 characters; the files have to be in the
  game's own formats, except pictures (PNG for a texture) and sounds and music (Opus or WAV), which the port converts (below).
* `MUSIC/<name>.MSC`: replaces a music stream; or `MUSIC/<name>.opus` (Ogg Opus, mono or stereo): the same stream as an Opus file, which is far smaller
  (decoded as it plays).  The name is the stream's (`1015` for the Tomb's).  The game asks for the music to loop, and the whole file loops: when the stream ends it starts again from the beginning (an Opus file is read again from
  its first sample, with the encoder's padding cut off by the length the file states).  There are no loop points: a track with an intro that plays only once is
  not possible, so make the file the part that repeats.
* A sound as `.opus` or `.wav` where the game has a `.vag`: `SFX.PAK/sfx/<name>.opus` replaces `sfx/<name>.vag`.  The loader makes the game's sound
  format from it when the mod changes (`mods/.cache/converted/`): mono, at the sample rate and with the looping of the sound it replaces (the game's table
  says how fast to play a sound, not the file), at most about 130 KB (10 s at 22 kHz).  A sound that loops in the game (an engine, an alarm) is made to loop from its
  first sample to its last; one that plays once plays once; a sound that is not in the game's packs plays once.
* `patches.txt`: edits of the game's data in memory at start, one per line: `<symbol or 0xADDRESS>[+offset] <b|h|w|f> <value> [...]`
  (byte, halfword, word, float, `#` comments), or `<symbol or 0xADDRESS>[+offset] s <text>` (a text and a zero byte, in the room of the old text).  The symbols are those of the game's executable (`guncombos`, `guns`, ...).  This is how a table (the
  weapon combinations, the guns' stats) is changed.

Levels: a mod can add levels to the arcade list with a `levels.txt` (see `examples/levels_sample/`): `level <number 31..99> "<name>" like <game level 1..30> [tune <track>] [modes dm,bagtag,ctb,escort,ko,ls] [setup <file>]`.
`manual/customlevels.rs` copies the level table (`lvinfo`) and the tables of texts into the port's memory with the new levels (up to 28), and the arcade page lists them after Ron
(`av_arcade_levels`); a new level has the record, the arcade set-up and the data files of the level it is `like` unless the mod has files of its own under its number.  Story and Challenge
are not changed.  A level's own files (the formats are in `docs/level-formats.md`) can be supplied, and `tools/obj2level.py` makes them from a 3D model in the OBJ format (see
`examples/obj_level_sample/`): the world with its textures, floors and walls to collide with, rooms and portals for large levels, the pads the bots walk by, and a `setup.txt` (the line
`setup <file>` of `levels.txt`) with the spawns and weapons by the numbers of the pads.  `python tools/lvdump.py <PAK folder>` reads the game's own level files with these layouts.

Textures: put a PNG where the game has a texture: `MISC.PAK/textures/<name>.png` replaces `textures/<name>.qpm` (`<name>` as in `index.csv` of "Extracting the textures": `0807`,
`misc/ammo_big`, ...).  The game still loads its own texture; the renderer shows the PNG in its place, over the same surface.  The PNG can be of any size (4096x4096 and 2048x1024
were tried; a side longer than `hd_texture_max` of `ts-port.ini`, 4096 by default, is made smaller when the picture is read, and a 4096x4096 picture takes about 85 MB of video
memory with its mip maps) and has full colour and alpha, no palette.  The picture is stretched to the original's shape, so keep its proportions; a picture the game draws as a sprite (the HUD's
ammo icon, the menus) stays the size on the screen the game gives it, but sharper.  The renderer finds the texture by the name of the file the game's loader was given (`manual/texhook.rs`
notes it, `ts_rt::texnames`; `ts_gfx::hd` keeps the pictures), so a texture the game never loads is never replaced, and a picture that is not read yet is read the first time its texture is
drawn (about 70 ms for 4096x4096).  The game's own palette tricks do not reach a replaced texture.  A ready-made `.qpm` in the mod still replaces the file in the pack (the game's format:
256 colours, at most 65536 pixels, a power of two on each side; `tools/png2qpm.py` makes one), and `TS_PNG_QPM=1` makes the loader turn a PNG into a `.qpm` that way too.
`examples/sample_mod/` is a small working mod (a texture, a sound, a music stream and `patches.txt`).
`tools/make_hd_sample.py <extracted textures> <mods folder>` makes an HD sample mod from ten wall textures of the Tomb (16 times as big, teal and grainy: easy to see); it is written next to the game, not into the repository.

`tools/convert_music.py <game folder>` turns the game's own music (`MUSIC/*.MSC`, 344 MB) into Opus files next to them (about 85 MB at 96 kbit/s); the port plays an
`.opus` when there is one beside the `.MSC` (`--delete-originals` removes the `.MSC`).  It needs ffmpeg.

The packs are never changed: when the mods change, the changed packs are built again in `mods/.cache/` and the game reads those (a new
file is added to the pack's directory, a replaced one points to the new data).  Mods are applied in alphabetical order, a later one wins.  The
log (`--headless` or a console) says what was loaded.  Not done: new code (a mod cannot add new game logic, only change data and files) and
tools that make the game's formats (that needs the formats to be understood first).

## Extracting the textures

`python tools/extract_textures.py <PAK folder> <output folder>` writes every texture of the game as a PNG (2642 files, about 34 MB) into folders by
what they belong to: `characters/chrNN`, `weapons/gunN`, `objects/<name>`, `levels/levelNN[_Name]` (the names of ten levels are known: Prison,
Village, ChemicalPlant, Asylum, Mansion, PlanetX, Streets, Mall, Tomb, Ron), `misc/` (fonts, menus, effects, decals, ammo, bags, the sea) and
`unreferenced/`; `index.csv` has the format, the size and every owner of each texture.  The game's textures are `.qpm` files of `MISC.PAK`: a
text header (`Q8`, `width height`, ...), a palette of 256 entries (alpha 0..128) and the indices (`M8` has the smaller mip levels after them,
`Q6` is 32 bit with no palette); the PNGs have exactly the game's pixels (there is nothing of a higher quality in the files).  Which model uses
a texture is read from the table at the start of the model's file (`tools/pakls.py` lists a pack).  The pictures are the game's: do not put
them in a repository (the folder `ts-rust/target/release/textures_extracted` is inside the ignored `target/`).  `TS_FILELOG` lists the files the
game loads, in order.

The PlayStation button pictures of the menus are textures of `MISC.PAK` too: `mm_button01` to `mm_button04` (circle, cross, triangle, square, 24x24), `mm_button05` (Select) and
`mm_cursor13` / `mm_cursor14` (L1, L2), all in the `misc/mapmaker` folder of the extraction (the other buttons, such as R1, R2 and Start, are drawn as text).  A mod can replace them like any
texture.

## Unlock Everything

The **Unlock Everything** row of the **Game Options** page (an item of the game type page; `unlock_all` in `ts-port.ini`, or `--set unlock_all=true` for one
run) opens everything the game keeps locked: the levels of the story and arcade modes, the characters, bot sets, the challenge mode and the
unlockable cheats (the Cheats Menu button on the level select page).  Every unlock in the game is a test on the profile's statistics
(`unlockedEx`); the game has a switch that makes all of those tests pass (what one of its cheat codes sets), and the option keeps it on.  The
save is not changed: switching the option off locks everything again as the profile has it.

## Things added to the original game

- **Arcade level Ron** (the console game has an arcade set-up for it but never lists it; `av_arcade_levels` in `avmenu.rs`, patches
  `arcade_pageTick` / `levelAvail`).  It comes after MapMaker in the list: a green test arena with Deathmatch, BagTag, Capture the Bag and Escort.
  KnockOut (no bag spawn points: the game divides by their number) and LastStand (`enemySetup` calls `exit()` for a level without a set-up)
  are taken out of its mode list because they crash.  Its bots are all BlueRobots: the game itself preloads character 79 for level 25.  Tech was
  tried and dropped (one small room).  The other unused levels (Prison / tileset, Asylum, NSA1, Test) have no level data or no set-up.
- **BlueRobot** (character 79) works in Ron as the bots' model, but cannot be added to the character select: it has no front-end model
  (its second model number is -1, which crashes `obInstNew`; pointing it at its in-game model shows Harry Tipper's model instead).
- **Mall / Bank clocks** show the PC's local time (`sceCdReadClock`).
- The in-game HUD in "centre" mode moves every piece to the centre of the 4:3 area; thin lines at the edges of the title screen's overhanging glow
  sprites are faded out (`trim` in `shader.wgsl`).
- **Audio / Video Options in a split screen**: the pause menu's page has three pages (audio, display, quality; left / right turn them) because
  two columns do not fit the half-height pictures (`SPLIT` in `avmenu.rs`).
- Texture sampling (filter, repeat or clamp) is chosen per draw, not per cached texture; before, the first draw that used a texture decided it
  for the whole frame, which showed as stretched wall textures and black patches on characters.
- **Past TimeSplitter / Future TimeSplitter**: the two playable TimeSplitters (characters 71 and 69; the story's level presets give 71 to the levels of the
  past and 69 to those of the future) have their own names (`chrnames.rs`).
- **Controllers** are read through SDL3 (built into the executable): Xbox / XInput, PlayStation 4 and 5 (USB and Bluetooth), Nintendo Switch and
  Switch 2 pads and anything else SDL has a mapping for (`gamepads.rs`).  Buttons are used by position (the bottom button is the cross).
  With three or more controllers connected the game gets a multitap on port 0 (up to four players). In a one-player level any controller plays; in the menus and in split-screen levels each controller is its own player, in the order they connected.
- **Keyboard and mouse** (port 0, together with a controller if there is one):

  | Input | Action |
  |---|---|
  | W A S D | move (diagonals are not faster) |
  | Shift (hold) | walk (65% of the running speed) |
  | Ctrl | crouch |
  | E (Enter) | use / activate, and select in the menus (cross) |
  | Q / F | previous / next weapon |
  | Left click / right click | fire / alternate fire (R2 / R1) |
  | Middle click | aim (crosshair); each click turns it on or off, whether the game's Aim Mode is Hold or Toggle |
  | Mouse wheel | previous / next weapon; while aiming with a weapon that zooms (the sniper rifles): zoom in / out |
  | Mouse | look: the view is turned by the mouse's own movement (`mouselook.rs`; `mouse_sensitivity` in `ts-port.ini`, a zoomed view turns less) |
  | Escape | pause |
  | Backspace | cancel / back (triangle; R is the manual reload, also triangle) |
  | Arrow keys | the directional buttons |

  The pointer is captured while a level is played and free in the menus and the pause menu.  (The key letters are those of the game's default
  controller set-up: fire R2, alternate fire R1, aim L2, crouch L1, activate cross, zoom and weapon change on the directional buttons.)
- On the options page the **resolution** is applied only when the choice is accepted with X (triangle gives it up), and the frame rate 60 is
  labelled "60 (Original)".
- In the running game's widescreen "edges" HUD the aiming crosshair is placed with the picture, so it no longer jumps when it crosses a third of the screen.
- The **aim assistance** is off while the keyboard and mouse were used last and on again as soon as a controller is touched (`autoaim_off`, patch of `playerTick`); it is the player's own Auto Aim setting otherwise.

## Diagnostics

A crash writes `ts-port-crash.log` next to the program.  If a frame is not presented for a few seconds, or the game stays on its loading
state, `ts-port-hang.txt` gets a line with what the game was waiting for (game state, level, music wait counter, music state).  The two
graphics threads drop a frame that panics instead of leaving the game waiting for them; after a long wait on the loading state the music is
closed and the level is ended by hand.  `TS_FORCEPLAYERS=<n>` (test aid) starts the next match with n players.

