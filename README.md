# ts-rust

A native PC port of the PS2 game *TimeSplitters*. The game's MIPS code (R5900) is hand-decompiled to Rust and runs natively; there is no
emulator, and the console's hardware (EE, GS, VUs, DMA, SPU2, IOP) is replaced by native code.

You need to provide the game's files yourself, from Version 2.0 (SLUS 20090) of TimeSplitters.  Put them next to the executable; the
program tells you if something is missing.

## Build and run

Some tables and the graphics microcode are generated from your copy of the game once (Python 3, run from the repository's root, see the top of each tool):

```
python tools/gen_rust.py
python tools/vu1dec.py
```

Then:

```
cd ts-rust
cargo build -p ts-port --release
target/release/ts-port.exe                      # the front end, in a window
target/release/ts-port.exe --level Village      # straight into a level
```

Options: `--level NAME`, `--players N`, `--headless`, `--frames N`, `--screenshot FRAME file.png`, `--press FRAME:button[:frames]`,
`--set key=value` (a setting of `ts-port.ini` for one run).  Settings are saved in `ts-port.ini` beside the executable (created on the first start,
commented) and can be changed in the game's Audio / Video Options page.  A crash is written to `ts-port-crash.log`.

Controls: keyboard and mouse (WASD, the mouse looks and fires, the wheel changes weapon; the keys are the `key_*` lines of `ts-port.ini`) or any
gamepad.  F11 switches between window and borderless, F10 quits.

## The default build is the original

The port is built to play exactly like the original: its game code is the original's logic, 60 Hz, a 640x448 picture in 4:3.  Everything it adds is behind
features of `ts-port`, which are off by default:

| Feature | What it adds |
|---|---|
| `extensions` | widescreen and field of view, a Game Options page, weapon sets, bot sets, randomisers, mouse and keyboard play, mods, HD textures, the developer menu, PC wording, unlock everything |
| `extended-tables` | dual versions of every weapon, levels added by mods, bigger room and prop tables |
| `high-framerate` | frame rates above 60 Hz |
| `all-extensions` | all of the above |

```
cargo build -p ts-port --release --features all-extensions
```

Where each extension hooks into the game's code, and how to use them, is in `docs/reference.md`.

## Verification

`ts-verify` runs each hand-written function and the original machine code (in an interpreter) on the same random memory and compares the results and every
byte written; it also runs on memory snapshots of the running game.  The port can also check itself against the original while it plays
(`--features shadow`).  Every function passes except a short list of known artefacts, and scripted runs of every challenge, 12 story levels and 20 arcade
levels show no differences.  The method, the tools and the known artefacts are in `docs/decompilation.md`.

## Making mods

`modkit.exe` (see `docs/modding-guide.md`) makes levels, characters, weapons, game mode variants, sounds, music and textures from menus; the guide also shows the files it writes.

## Layout

| Crate | Role |
|---|---|
| `ts-game` | the game: hand-written functions in `src/game`, platform glue and extensions in `src/manual` |
| `ts-rt` | the console's memory and arithmetic (no NaN, clamped overflow) |
| `ts-sdk` | native replacements for the console's libraries, disc, pad, memory card |
| `ts-gfx`, `ts-render` | the graphics: display lists, the vector unit programs, the renderer |
| `ts-audio` | sound and music |
| `ts-verify` | the verifier |
| `ts-port` | the program: window, input, settings |

## Notes

- Known gaps: the console's reverb is not implemented (the game never uses it); fisheye and a few screen effects are approximate; multiplayer and the
  second memory card are lightly tested.
- Documents: `docs/decompilation.md` (how the code was decompiled and verified), `docs/reference.md` (options, extensions, mods, tools),
  `docs/level-converter.md` and `docs/level-formats.md` (making levels).
