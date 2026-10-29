# ts-rust

A native PC port of the PS2 game *TimeSplitters*. The game's MIPS code is hand-decompiled to Rust and runs natively; there is no emulator.
Put the `PAK` and `MUSIC` folders of the game (Version 2.0, SLUS 20090) next to the program. The original executable is not needed: its code is Rust here, and its data is in this repository.

## Build and run

```
python tools/build-game.py             # builds and puts ts-rust.exe in the play folder, ts-rust/target/release/
python tools/build-game.py --original  # the build that behaves exactly as the original game
python tools/build-game.py --kit       # also builds tools/modkit.exe (needs pyinstaller)
ts-rust/target/release/ts-rust.exe --level Village   # straight into a level (no option: the front end)
```

The play folder holds the program, the game's `PAK/` and `MUSIC/`, and what the game makes there (`ts-rust.ini`, `saves/`, `mods/`).
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

## Mods

A mod is a folder in `mods/` with a `mod.ini` (`name = ...`, `enabled = true`); the Mods page switches them on and off. Mods can add levels, characters, weapons,
game modes, sounds, music and HD textures, and can turn on features (Unlock Everything, Dual Wield Everything, Bots Punch: see `examples/`).
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

GitHub Actions builds the game (`.github/workflows/linux.yml` and `windows.yml`; Actions tab, Run workflow). Unpack the artifact next to `PAK/` and `MUSIC/`
(the game's own files are not in the repository or the artifacts). The Linux build is untried on real hardware; on a Steam Deck, unpack it in Desktop mode and add `ts-rust` to Steam as a non-Steam game.

## Documents

`docs/reference.md` (options, extensions, mods, tools), `docs/modding-guide.md`, `docs/decompilation.md` (decompiling and verifying), `docs/damage.md`, `docs/game-modes.md`, `docs/level-converter.md`, `docs/level-formats.md`, `docs/model-format.md`, `docs/mod-roadmap.md`.

Known gaps: the console's reverb is not implemented (the game never uses it); fisheye and a few screen effects are approximate; multiplayer and the second memory card are lightly tested.

## Android (proof of concept)

The `android` branch builds an arm64 app (`.github/workflows/android.yml`, artifact `ts-rust-android`): touch controls, no controller support yet. Install the apk, give the app "All files access" (Settings > Apps),
start it once so it makes a `TimeSplitters` folder in the device's storage, copy the game's `PAK/` and `MUSIC/` (and `mods/`) into it, and start the app again. Tried on a Pixel 10 Pro and an AYN Thor.
