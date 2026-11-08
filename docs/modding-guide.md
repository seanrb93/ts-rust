# Making mods

Mods are made and shared by the people who make them, apart from the program: the repository has none.  The mods these documents name as examples (`example_mods/`) are a local folder of copies of the play folder's `mods/`, which git ignores.

## The easy way: the mod kit

`modkit.exe` is one program with all the mod making tools.  It is in `tools/`, not in the play folder (`python tools/build-modkit.py` makes it); double-click it: it asks where the game is the first time.  A menu asks what you
want to make; every question has a default you can accept with Enter, and your files go into the mod you are working on (the first thing you make creates one).

```
1. A level (from a map or a 3D model)         Source / GoldSrc maps, OBJ, glTF, Blender: the level converter
2. A character (from a 3D model)              rigged to the game's skeleton, with its seams; or get a game character as a model to start from
3. Weapons (change or add)                    pick a weapon, change the numbers that matter (clip, damage, sound ...), or copy it as a new weapon
4. Game modes (ways to play)                  what a mode starts with, and variants with a rule that ends the match
5. Sounds, music and pictures                 your own sounds, music tracks and textures in place of the game's
6. Try it in the game                         starts the game; looks at your character walking; tries your weapon on bots
7. Share                                      a zip file of the mod, and installing one from a zip
8. Mods                                       choose, make, turn on and off
```

Without a build of `modkit.exe`: `pip install numpy pillow imageio-ffmpeg`, then `python tools/modkit-entry.py` (or double-click `tools/modkit.bat`).  To make the exe: `pip install pyinstaller` and
`python tools/build-modkit.py` (it ends up in `tools/`).  `modkit weapons`, `modkit character`, `modkit level`, `modes`, `sounds`, `play`, `share`, `mods` start a part of the menu directly.

Adding weapons, characters and mode names needs the game build with the extended tables.  The numbers of added weapons and characters are chosen by the kit from the name, so that mods made apart seldom
use the same one; the game gives them its own numbers when it starts, in the order of the numbers.

Below is what the kit does, by hand, for those who prefer files.

This is the short route to each kind of mod.  `docs/reference.md` ("Mods") lists every file and key; `docs/mod-roadmap.md` says how each part was found and what is still missing.

A mod is a folder in `mods/` next to the program, with a `mod.ini` that says `enabled = true`.  The **Mods** page of the main menu switches mods on and off; the program reads them when it starts.  Adding weapons, characters and mode names needs a build with the `extended-tables` feature (`cargo build --release -p ts-port --features extended-tables`, or `all-extensions`).  A build without it uses the game's own tables and tells the log that a mod asked for more.

A model file that a mod names but does not hold (and the packs do not have) is left out with a line in the log; the weapon or character then uses the model of the one it copies.

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

A weapon that is thrown can be given a lift: `throw_lift 0.07` adds that to the upward speed of its shot (units a frame, as `bullet_speed`), so that a brick, which has gravity, goes up and comes down again like a throw instead of falling at the thrower's feet (`example_mods/brick_throw_mod`; `TS_BULLETLOG=1` prints where a brick is each frame).

To add a weapon, give a number from 30 on, `copy` the weapon it starts from and a `name`:

```
weapon 30
  copy 0
  name "Super Pistol"
  clip 99
  view_model_file ob/gun90/gun90.raw
```

The new weapon is a single weapon and a pair, and shows in the weapon sets and when weapons are changed.  Its models come from files the mod holds under `OB.PAK/` (`view_model_file` for the gun in the first person, `model_file` for the gun a character holds, `ammo_model_file` for the pickup).  An added weapon fires with the sound of the one it copies unless you give `fire_sound N` (the number of a sound of the game; `TS_SNDLOG=1` prints the numbers of the sounds as they play, so fire the weapon you want the sound of).

## Add a character

1. Get a mesh of the figure: `python tools/obdump.py <OB.PAK> ob/chrs/chr01.raw --obj figure.obj` writes a game character in the pose of the skeleton, to model on.  Keep the pose and the height (about 1.95); for a bigger or smaller character add `--scale 1.4` to the command below, which scales the mesh and the character's `scale` field together (the skeleton does not change with the mesh alone).
2. `python tools/obchar.py <game folder> figure.obj --out mods/Robo --name "Robo" [--base 0] [--voice 5]` makes the mod: the model, the textures from the OBJ's materials (any size) and a `characters.txt`.  Each triangle is given to the bone nearest to it, and the triangles near a joint go to the seam parts that the game's own characters have there.
3. Start the game.  The character is on the character select page after the game's own (left from the first character, or right through the list), on the bot sets page, and can be a bot.

By hand the same is `character 80`, `copy 0`, `name "Robo"`, `voice 5`, `model_file ob/chrs/chr90.raw`.  Numbers go from 80 up to 127 in turn.  A character with its own model uses two of the 27 model props that mods share, so about 13 can have one.  A solid figure (a robot) made all on the pelvis (`o bone0`) can have the line `upright`: while it lives the pelvis keeps the turn it has at rest (the other bones keep theirs, so the hands still hold the guns the right way), so it stands straight the way it faces instead of leaning with the people's poses (their hips are turned when they stand); its bones still walk and turn it, and it falls when it dies.  A small figure sets the height of its view with `raw +20 f <standing>` and `raw +24 f <crouching>` (every character of the game has 1.8 and 1.0).  `example_mods/blue_robot_mod` is the game's unused Blue Robot made so: its model, which has no skeleton, rebuilt by `obchar.py` with all of it on the pelvis (`o bone0`).

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

Each variant is listed as a mode on the Select Level page, after the game's own (a level lists up to seven modes in all); choosing it opens the Options page with the variant's options set and its name at the top.  A `rule` ends the match by itself: `win_score N`, `win_score_seconds N` (BagTag) and `win_lead N`.  `hide` (in a variant) leaves options off the Options page: `hide team_play team_starts friendly_fire kill_limit score_method score_limit one_shot_kill` (also `time_limit`, `radar`, `handicaps`, `start_with_gun`); set their values with the lines above it.  `script <file>` gives the variant a script, which can make its own rules, move the computer characters, add effects and say what the results say (`docs/scripting.md`; `example_mods/virus_mode/` is a whole mode made this way, with no change to the program).  A mode with objects or bases of its own is not possible yet.

## Add a level

`tools/convert-level.bat` (double-click it on Windows; or `python tools/obj2level.py`) asks for a map (Blender, glTF, OBJ, a Source or GoldSrc map) and installs it as a mod.  See `docs/level-converter.md`.  Keep the level to what the game's tables hold (the converter warns, and `--original-limits` keeps to the game's own).

## Settings a player can change

A mod can have settings: named values in a `[settings]` section at the end of its `mod.ini`, so that a player changes what the mod does without reading its tables.  Write a line of explanation over each:

```
name = Weapon Balance
enabled = true

[settings]
# How strong the explosions are, in percent of this mod's.
explosion_strength = 100

# Balance the shotguns too (yes or no).
balance_shotguns = yes
```

The player changes a value, saves the file and starts the game again.  The data files of the mod (`weapons.txt`, `characters.txt`, `modes.txt`, `patches.txt`) use the values:

```
weapon 8
  blast {explosion_strength / 100}         # in braces: a setting, a number, or a sum of them with + - * / and brackets
  alt_blast {0.6 * explosion_strength / 100}
if balance_shotguns                        # read only when the setting is yes, or a number other than 0
weapon 24
  damage 0.19
else
  # (the game's own)
end
```

`name`, `enabled` and `script` are read only above the first section, so a setting may have any name.  (A `settings.ini` of `name = value` lines in the mod's folder is read too, for mods made that way; `mod.ini` wins where both have a setting.)

`if` takes a setting (`if name`, `if not name`) or a comparison (`if name = word`, `!=`, `<`, `<=`, `>`, `>=`), and the blocks can be put inside each other.  A setting that is a word, in braces on its own, goes in as that word.  A line that uses a setting that is missing or is not a number is left out, and the log names the lines, so a wrong value in the settings never writes a wrong value into the game.  A script reads a setting with `setting("name", default)` (`docs/scripting.md`).

All the machines of a network game need the same settings: a machine with other values cannot join (the lobby says "other game files or mods").  `example_mods/balance_mod`, `bots_punch_mod`, `brick_throw_mod`, `virus_mode`, `first_person_hands_mod`, `dual_wield_everything_mod` and `smarter_bots_mod` have settings.

## Replace sounds, music and textures

Put a file in the mod under the pack's name: `SFX.PAK/sfx/<name>.mp3` (or `.ogg`, `.flac`, `.m4a`, `.wav`, `.opus`) replaces a sound, `MUSIC/<name>.mp3` (or the same formats) a track, `MISC.PAK/textures/<number>.png` a texture.  See `example_mods/sample_mod/`.

## Seeing a result

* The test harness (a build with the `extensions` feature) starts a level with what you want to look at: `--level Prison --character 80 --bots 13,80 --spawn 12,0,-26 --cam 5,2`.  `--character N` makes player 1 the character N, `--bots a,b,...` puts these characters in as bots (also for a match from the menus), `--spawn x,y,z` puts the player there 20 frames after the level starts (`TS_WHERE=1` prints where the player is, `TS_BOTLOG=1` where the first bot is), and `--cam back,up` turns on the third person camera that far behind and above the player.  They set `TS_PLAYERCHR`, `TS_BOTCHRS`, `TS_SPAWN` and `TS_CAM`.  Prison is the level with open ground.  `TS_GIVE=40:49 TS_EQUIP=1` puts an added weapon in the hands, `--press 60:r2:700 --press 60:rright:700` holds the trigger and sweeps the right stick, and `TS_BOTLOG=1` prints the kills (the damage of a weapon is a fraction of a character's health: the pistol's is 0.3).
* `--screenshot FRAME file.png` and `--press FRAME:button` drive the game from a script (`docs/reference.md`, `docs/decompilation.md`).
* `TS_CAMMODE=2:200 TS_CAMDIST=3,0.5 --level Prison` shows a character from behind in the Prison level.
* `TS_CAMREAL=1 TS_CAMMODE=2:1100 TS_CAMDIST=3,1.2` does the same in a match started from the front end (the frame is the port's, and the camera mode is kept from it on): the player is seen from behind, with the bots about; where the spawn point is against a wall the view is the wall.
* `TS_GIVE=60:<combo> TS_EQUIP=1` puts a weapon in the player's hands.
* `TS_COMBOLOG=1` logs the weapons, props and texts that a mod adds.
