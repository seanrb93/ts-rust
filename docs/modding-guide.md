# Making mods: a walk through

This is the short route to each kind of mod.  `docs/reference.md` ("Mods") lists every file and key; `docs/mod-roadmap.md` says how each part was found and what is still missing.

A mod is a folder in `mods/` next to the program, with a `mod.ini` that says `enabled = true`.  The **Mods** page of the main menu switches mods on and off; the program reads them when it starts.  Adding weapons, characters and mode names needs a build with the `extended-tables` feature (`cargo build --release -p ts-port --features extended-tables`, or `all-extensions`).  A build without it uses the game's own tables and tells the log that a mod asked for more.

Test a mod on the stock levels, with no other mod in `mods/`.

## Change a weapon

1. List the game's weapons: `python tools/dump-guns.py <game folder>`.
2. Write `weapons.txt` in the mod:

```
weapon 0
  clip 15
  max_ammo 250
```

   The names of the fields are in `docs/mod-roadmap.md`.  A line with a wrong name or a number out of its range is left out and the log says which line.
3. Start the game.  The log (`[mods]`) says how many changes each file made.

To add a weapon, give a number from 30 on, `copy` the weapon it starts from and a `name`:

```
weapon 30
  copy 0
  name "Super Pistol"
  clip 99
  view_model_file ob/gun90/gun90.raw
```

The new weapon is a single weapon and a pair, and shows in the weapon sets and when weapons are changed.  Its models come from files the mod holds under `OB.PAK/` (`view_model_file` for the gun in the first person, `model_file` for the gun a character holds, `ammo_model_file` for the pickup).  The sound numbers of a weapon (at +56 and +96 to +104 of its entry) are not mapped yet, so an added weapon sounds like the one it copies.

## Add a character

1. Get a mesh of the figure: `python tools/obdump.py <OB.PAK> ob/chrs/chr01.raw --obj figure.obj` writes a game character in the pose of the skeleton, to model on.  Keep the pose and the height (about 1.95); for a bigger or smaller character add `--scale 1.4` to the command below, which scales the mesh and the character's `scale` field together (the skeleton does not change with the mesh alone).
2. `python tools/obchar.py <game folder> figure.obj --out mods/Robo --name "Robo" [--base 0] [--voice 5]` makes the mod: the model, the textures from the OBJ's materials (any size) and a `characters.txt`.  Each triangle is given to the bone nearest to it, and the triangles near a joint go to the seam parts that the game's own characters have there.
3. Start the game.  The character is on the character select page after the game's own (left from the first character, or right through the list), on the bot sets page, and can be a bot.

By hand the same is `character 80`, `copy 0`, `name "Robo"`, `voice 5`, `model_file ob/chrs/chr90.raw`.  Numbers go from 80 up to 127 in turn.  A character with its own model uses two of the 27 model props that mods share, so about 13 can have one.

Not made yet: sounds of its own (use `voice`), a different skeleton or animations, the character in the story.

## Make a mode

`modes.txt` sets what a mode starts with and adds ways to play it:

```
mode deathmatch
  name "Quick Match"
  kill_limit 10
  variant "First to 8"
  rule win_score 8
  variant "Lead by 5"
  rule win_lead 5
```

The arcade Options page then has a **Rules** row with Standard and the variants.  A `rule` ends the match by itself: `win_score N`, `win_score_seconds N` (BagTag) and `win_lead N`.  A mode with objects, bases or its own rules of play (new code) is not possible yet; a variant changes the numbers the game already keeps and adds these rules.

## Add a level

`tools/convert-level.bat` (double-click it on Windows; or `python tools/obj2level.py`) asks for a map (Blender, glTF, OBJ, a Source or GoldSrc map) and installs it as a mod.  See `docs/level-converter.md`.  Keep the level to what the game's tables hold (the converter warns, and `--original-limits` keeps to the game's own).

## Replace sounds, music and textures

Put a file in the mod under the pack's name: `SFX.PAK/sfx/<name>.opus` (or `.wav`) replaces a sound, `MUSIC/<name>.opus` a track, `MISC.PAK/textures/<number>.png` a texture.  See `examples/sample_mod/`.

## Seeing a result

* `--screenshot FRAME file.png` and `--press FRAME:button` drive the game from a script (`docs/reference.md`, `docs/decompilation.md`).
* `TS_CAMMODE=2:200 TS_CAMDIST=3,0.5 --level Prison` shows a character from behind in the Prison level.
* `TS_GIVE=60:<combo> TS_EQUIP=1` puts a weapon in the player's hands.
* `TS_COMBOLOG=1` logs the weapons, props and texts that a mod adds.
