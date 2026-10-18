# Sample mod

A small mod that shows what the mod loader does.  Copy this folder into `mods/` next to `ts-port.exe`, set `enabled = true` in its `mod.ini` and start the game (the log, from
`ts-port.exe` in a console or with `--headless`, says what was loaded).

| File | What it does |
|---|---|
| `MISC.PAK/textures/misc/ammo_big.qpm` | Replaces the game's texture `textures/misc/ammo_big.qpm` (the picture of a bullet next to the ammo of some weapons) with an original blue bullet.  The folder `MISC.PAK/` stands for the pack of that name: the path under it is the file's name inside the pack. |
| `SFX.PAK/sfx/walther_and_bullet22_01e.opus` | A sound as Ogg Opus where the game has `sfx/walther_and_bullet22_01e.vag` (the pistol's shot).  The loader makes the game's sound format from it, at the rate and with the looping of the original (the first run says so in the log). |
| `MUSIC/1015.opus` | A music stream (the Tomb's) as Ogg Opus, in place of `MUSIC/1015.MSC`.  An Opus file is a fraction of the size. |
| `patches.txt` | Changes the game's data when it starts: the pistol gets a clip of 15 (was 10) and carries 100 rounds (was 50), and the game mode KnockOut is called "Knock-Out!". |
| `mod.ini` | `enabled = false` switches the mod off without removing it. |
| `art/` | The sources (a folder called `art`, `source`, `src` or `docs`, or one that starts with `_`, is not loaded).  `tools/make_sample_mod_art.py` and `tools/make_sample_mod_sound.py` make the picture and the sounds. |
| `art/ammo_big.png` | The picture as a PNG; `tools/png2qpm.py art/ammo_big.png ammo_big.qpm` makes the game's format from it (256 colours with alpha). |

How it works: the game's packs are never changed.  When a mod is new or has changed, the loader builds copies of the packs that have the mods'
files in `mods/.cache/` and the game reads those.  A mod's folders are the pack names (`ANIM.PAK`, `BG.PAK`, `FRONTEND.PAK`, `MISC.PAK`,
`OB.PAK`, `SFX.PAK`); `MUSIC/` holds music; `patches.txt` edits data.  Mods are applied in alphabetical order of their folders, a later one wins.
To find the name of a texture, extract them with `tools/extract_textures.py` (its folders and `index.csv` list every texture).
