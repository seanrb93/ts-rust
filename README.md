# ts-rust

A native PC port of the PS2 game *TimeSplitters*. The game's MIPS code is hand-decompiled to Rust and runs natively; there is no emulator.
Put an image of the game's disc (Version 2.0, SLUS 20090: a `.cue` and its `.bin`, or a `.chd`) next to the program. The original executable is not needed: its code is Rust here, and its data is in this repository.

## Build and run

```
python tools/build-game.py             # builds and puts ts-rust.exe in the play folder, ts-rust/target/release/
python tools/build-game.py --original  # the build that behaves exactly as the original game
python tools/build-game.py --kit       # also builds tools/modkit.exe (needs pyinstaller)
ts-rust/target/release/ts-rust.exe --level Village   # straight into a level (no option: the front end)
```

The play folder holds the program, the image of the game's disc, and what the game makes there (`ts-rust.ini`, `saves/`, `mods/`).

## The game's disc

The game is read straight from an image of your own disc in the play folder; nothing needs to be extracted.

- A cue sheet and its BIN file (a dump of the CD in raw 2352-byte sectors, MODE2/2352 as PS2 dumps are, or MODE1), or a CHD (`chdman createcd -i game.cue -o game.chd`, or `createdvd` from an ISO). A CHD is about a third of the size.
- When the folder has several, a `.chd` is used before a `.cue`, and the first by name of each.
- Without an image, or with one that is not TimeSplitters (no `PAK/BG.PAK` on it), the program says so and stops.
- Mods are unchanged: their files stand in for the disc's as before, and a mod's music (`MUSIC/<number>.opus`, `.mp3`, ...) plays in place of the disc's.
- `ts-rust --extract-disc <folder>` writes the disc's files to a folder, for the Python tools that read the game's packs; the mod kit does this by itself, once, into `mods/.cache/disc`.
Settings are in `ts-rust.ini` (commented, made on the first start) and in the Audio and Video Options pages; `--set key=value` changes one for a run.
Other options: `--players N`, `--headless`, `--frames N`, `--screenshot FRAME file.png`, `--press FRAME:button[:frames]`.
Controls: keyboard and mouse (WASD, the mouse looks and fires, the wheel changes weapon) or any gamepad; the button pictures follow the controller. F11 switches window and borderless, F10 quits.

## Builds

The default `cargo build` is the original game: 60 Hz, 4:3, the original's logic. The extras are features of `ts-port`:

| Feature | What it adds |
|---|---|
| `extensions` | widescreen, field of view, extra option pages, mouse and keyboard play, mods, HD textures, weapon and bot sets, the developer menu |
| `extended-tables` | dual versions of every weapon, levels added by mods, bigger tables |
| `high-framerate` | frame rates above 60 Hz, up to the monitor's |
| `all-extensions` | all of them |

`tools/build-game.py` builds with all three; `--original` builds without them.

The program is built for x86-64-v3 (a CPU with AVX2: Intel from 2013, AMD from 2015), which makes the graphics and the game's float maths about a quarter
faster with the same results (`ts-rust/.cargo/config.toml`).  For an older CPU build with `RUSTFLAGS="-C target-cpu=x86-64"`.

## Mods

A mod is a folder in `mods/` with a `mod.ini` (`name = ...`, `enabled = true`); the Mods page switches them on and off. Mods can add levels, characters, weapons,
game modes, sounds, music and HD textures, and can turn on features (Unlock Everything, Dual Wield Everything, Bots Punch: see `example_mods/`).
A mod's choices are in the `[settings]` section of its `mod.ini` (a value and a line saying what it does, e.g. `explosion_strength = 100` in Weapon Balance): change one and start the game again.
`tools/modkit.exe` makes them from menus. See `docs/modding-guide.md`.

## Verification

`ts-verify` runs each hand-written function and the original machine code on the same random memory and compares the results and every byte written, and also on snapshots of the running game.
Every function passes except a short list of known artefacts (`docs/decompilation.md`).

## Layout

| Crate | Role |
|---|---|
| `ts-game` | the game: decompiled functions in `src/game`, platform glue and extensions in `src/manual` |
| `ts-rt`, `ts-sdk` | the console's memory and arithmetic; native replacements for its libraries |
| `ts-gfx`, `ts-render`, `ts-audio` | graphics (display lists, vector unit programs, renderer) and sound |
| `ts-verify` | the verifier |
| `ts-port` | the program: window, input, settings |

## Linux, Steam Deck and Windows builds

GitHub Actions builds the game (`.github/workflows/linux.yml` and `windows.yml`; Actions tab, Run workflow). Unpack the artifact next to the image of the disc
(the game's own files are not in the repository or the artifacts). The Linux build is untried on real hardware; on a Steam Deck, unpack it in Desktop mode and add `ts-rust` to Steam as a non-Steam game.

## Documents

`docs/reference.md` (options, extensions, mods, tools), `docs/modding-guide.md`, `docs/decompilation.md` (decompiling and verifying), `docs/damage.md`, `docs/game-modes.md`, `docs/level-converter.md`, `docs/level-formats.md`, `docs/model-format.md`, `docs/mod-roadmap.md`.

Known gaps: the console's reverb is not implemented (the game never uses it); fisheye and a few screen effects are approximate; multiplayer and the second memory card are lightly tested.
