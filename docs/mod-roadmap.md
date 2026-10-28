# Roadmap: custom characters, weapon mods and game mode mods

This is a plan, not a feature.  It says what the game's data and code allow today, what the port already has to build on, what is not known yet, and in
what order to find out and build.  Nothing here changes the default build: everything would sit behind the `extensions` and `extended-tables` features like
the rest.

What a mod can do now (see `docs/reference.md`, "Mods"): replace or add files of the game's packs, textures (PNG), sounds and music (Opus, WAV), edit the
game's data in memory with `patches.txt`, and add levels.  A mod cannot add game logic.  Everything below is about going further than that.

## What the port already has to build on

| Piece | Where | What it gives |
|---|---|---|
| Moving a fixed table into the port's memory and growing it | `manual/weapontable.rs` (weapon combinations), `manual/proptable.rs` (props), `manual/customlevels.rs` (levels and their texts) | The pattern for adding entries: the generated code reads the table through a symbol that `tools/patches.py` points at the port's copy. |
| Pointing a character at a new text | `manual/chrnames.rs` | How to give a character a name that is not in the game's text table (a copy of the tables with texts added). |
| Choosing sets of weapons or bots with more slots than the game has | `manual/weaponsets.rs`, `manual/botsets.rs` | A way to swap a mod's choice into the one place the game reads (`weaponset_effective`, `chrset_effective`). |
| Editing tables from a mod file | `patches.txt` | Any table that has a symbol (`guns`, `ammoinfo`, `guncombos`, `chrs`, `gamemodedata`) can already be changed by a mod, byte by byte. |
| Replacing a pack's file, texture or sound | the mod loader | A character's or weapon's files can be replaced by files of the same name. |
| Reading the game's packs | `tools/pakls.py`, `tools/extract_textures.py`, `tools/lvdump.py` | Listing, and decoding of textures and level files. |

## Custom characters

How the game describes a character (read from the code, not all verified):

* `chrs` is a table of 44 bytes for each character (about 96 fit in the space before `ammoinfo`).  Seen so far: a halfword at +0 and +2 that number the
  character's names in the text table, a pointer at +4 to the list of the character's sounds (up to 18 halfwords, then a second list of 6, each ended by 0), the
  number of the body model at +8 and of the front-end model at +12 (both are numbers of the table of props, `proptable`), a flag at +16 and a float at +40.
  `chrPreload` loads the body model, the sounds and the head prop.
* Models are objects of `OB.PAK` found through the table of props.  Animations are in `ANIM.PAK`, found through `pAnimations`.
* The character select pages and the bot sets list characters by number (`characterorder`, `chrpresets`, the `chrset*` tables, `chartab1`/`chartab2`).

What a data-only mod could do: change a character's textures (works today, a PNG), sounds (works today), name and sound list (`patches.txt` on `chrs`),
and swap one character's model for another's (point `chrs+8` at another prop number).

What it cannot do yet, and is not known:

* The format of a model file (vertices, the chunks the vector unit draws, the skeleton and the skinning, the table of textures at its start) and of an animation
  file.  Neither is documented in this repository; `lvdump.py` reads level files only.  A new body needs a converter from a modelling tool's file to this
  format, the way `obj2level.py` writes levels, so the formats have to be worked out first.  This is the largest piece of work in the whole plan.
* How a character's rig is matched to the game's animations (the skeleton has to have the bones the animations move).  Reusing the game's skeleton and
  animations with a new mesh is the realistic first goal.
* Adding characters past the game's own: `chrs` is in the game's data with other data close behind, so it would be moved like `guncombos`, and everything that is
  indexed by character (the select pages, the unlock flags, the bot sets, `charcolours`, the ranking tables) has to be checked for a fixed size.

Plan:

1. Document the model format: read `obLoad`, `obRelocate`, `obPrecalcGfxData` and the drawing code, write a reader (`tools/obdump.py`) that prints a character's meshes, bones and
   texture table, and check it by drawing the result.
2. Document the animation format the same way.
3. Write a model exporter for a skinned mesh onto the game's skeleton (glTF in, game object out), starting from a known character so a round trip can be compared.
4. Add a `characters.txt` to the mod format (name, model, sounds, which animations) and the loader code to move `chrs` and add entries, with the select pages and bot sets
   listing them.  A sample mod.

## Weapon mods

How the game describes a weapon (read from the code):

* `guns` has 400 bytes for each of 30 weapons: +16 ammo kind, +20 clip size, +52 bullet type, +172 a pointer to the entry the bullet type is found through.
  `ammoinfo` has 8 bytes for each kind of ammo (most carried, then a pointer).  `guncombos` (33, and the port's extra ones) pair a left and right gun with a rank and a name.
* What a weapon does when it is fired is code: `gunPlayerFire`, `gunChrFire`, `gunUpdateParts`, `gunReload`, the bullet tick, and the effects.  They
  choose by the weapon's number and its bullet type.  The tables decide numbers (damage, rate, spread and so on, once the fields are known), not the shape of a
  behaviour such as a grenade's arc or a beam.

What a data-only mod could do now: change the stats in `guns` and `ammoinfo`, the rank and name in `guncombos`, sounds and textures of a weapon, and put weapons into
the weapon sets.  What is missing is a map of the fields of the 400 bytes (only a few are known), so a mod file that names fields (`damage`, `clip`, `rate`) instead of
offsets, and a check of what the others do.

What it cannot do: a weapon with a new behaviour, and a new model (the same missing model format as for characters; a weapon is a prop with its parts and muzzle points).

Plan:

1. Map the fields of a `guns` entry by finding every read of each offset in the code and testing by patching (the method that found the clip size), and write
   them down in a table with names.
2. Add a `weapons.txt` to the mod format that sets fields by name, with checks (ranges), so a mod does not need addresses.
3. Add weapons: move `guns` into the port's memory like `guncombos`, and add entries that copy an existing weapon's behaviour (same bullet type and handlers) with new stats, name,
   sounds and textures.  This gives "a weapon like the shotgun but with these numbers".
4. Behaviour: a small set of parameters per bullet type, or a hook the mod names, only after the fields are known.  Models wait for the model format.

## Game mode mods

How the game runs a mode (read from the code):

* `curgameparms+72` holds the mode; `game-status-and-level-tick.rs` dispatches it with a fixed match: 1 BagTag (`gmBagTagTick`), 2 Capture the Bag (`gmCtfTick`), 3 Deathmatch
  (`gmDeathmatchTick`), 4 Escort (`gmEscortTick`), 5 KnockOut (`gmKnockOutTick`), 6 Last Stand (`gmLastStandTick`), anything else the story (`gmStoryTick`).
* The options a mode shares are globals: `gv_teamplay`, `gv_friendfire`, `gv_timelimit`, `gv_killlimit`, `gv_scoremethod`, `gv_scorelimit`, `gv_oneshotkill`, `gv_spawngun`, `gv_handicaps`
  and the rest of the Game Options.  The modes' texts and the mode list are in `gamemodedata` and `gamemodelist`.
* The ticks read the state of the players (`chrinfo`, 0x1210 bytes each) and `curgameparms`, and end the level with the game's own calls (`gameLevelCompleted`).

What a data-only mod could do now: change the options a mode starts with, the mode's name, and which modes a level offers (`modes` in `levels.txt`).

What it cannot do: new rules.  The logic is code.  The realistic way is a small, fixed set of building blocks in Rust behind the extension features, and a mod
that chooses and sets them, not a mod that runs its own code:

* A mod mode is a rule set made of parts the port provides: the score method (kills, deaths, time held, objects), the win condition (score, time, last alive), team rules,
  the objects (bags, bases, zones), respawn rules and the HUD text.  The parts are written once, in Rust, and checked like the rest.
* The mode shows up in the mode list after the game's own and is dispatched by an added case in the match above (the match is in code the port owns, so this is a hook of the kind
  already used elsewhere).

Plan:

1. Read the six ticks and the code they call, and write the state they keep (which fields of `curgameparms` and of the players each uses) in the documents.
2. Find what the menus and the end-of-game pages need for a mode (the rank lists and the stats pages are keyed by mode number).
3. Make the mode list and the dispatch extensible (a table of modes in the port's memory, the way `customlevels.rs` does for levels).
4. Add `modes.txt` to the mod format: a mode name, a base mode to copy, and its parts and numbers.  Start with variations of the existing modes (a deathmatch that is
   won by time held, a KnockOut with other scoring) as the proof, then objects.
5. Only if that is not enough: a scripting layer.  It needs a decision on the language, a sandbox, and a rule that a mod cannot touch the verified game code's memory except through the
   port's calls.  This is a large design of its own and is not planned yet.

## Done so far

* `weapons.txt` and `characters.txt` (`ts-port/src/moddata.rs`): the shared format with names, ranges and line numbers in the log, for the weapon fields known so far (ammo kind, clip, flags, bullet type, most carried) and character fields (name texts, models, flags, scale), and `raw` for the rest.  Tested by unit tests and a load in the game.  Also `tools/dump-guns.py` and the table of mapped weapon fields below.  * New weapons (`weapon 30` and on, `copy` and `name`): the table of weapons (`guns`) is moved into the port's memory with room for 64 (`GUNS_TABLE`, as the combinations are), the added weapon is a copy of one that exists with fields changed, and it gets a single combination, a pair and a name text (`customlevels::register_text`).  Tested by unit tests and by loading two added weapons in a level (the combination table and the texts are right); not yet played.  Not done: new models and sounds for an added weapon, the remaining fields of a weapon, adding weapons and characters past the game's tables, `modes.txt`, any model format.

### Weapon fields mapped so far

Found by reading what the game's functions do with each offset of an entry of `guns` (400 bytes), and checked against the values of the game's weapons (`python tools/dump-guns.py <game folder>`
lists them: weapon 3, which has a clip of 5, a zoom from 20 to 8 degrees and a spread of 0.001, is the sniper rifle).  Names marked (guess) are read from one use only.

| Field | At | What it does |
|---|---|---|
| `ammo_kind` | +16 | the kind of ammo (an entry of `ammoinfo`) |
| `clip` | +20 | rounds in a clip (-1: none) |
| `ammo_reserve` | +24 | the ammo a player starts with or is given by the level setup, and what a dead enemy drops (guess) |
| `flags` | +28 | bits 1, 2 and 8 choose animations and what an enemy does with the weapon (guess on the meaning of each) |
| `view_x`, `view_y`, `view_z` | +32, +36, +40 | where the gun is held on the screen; x is mirrored for the left hand |
| `fov_a`, `fov_b` | +44, +48 | the fields of view of the zoom (player +268 and +272) |
| `ai_burst` | +60 | compared with an enemy's shot counter: the length of its burst (guess) |
| `flash_bone`, `spark_bone`, `muzzle_bone` | +340, +360, +364 | bones of the model that the flash, sparks and shot come from |
| `bullet_type`, `damage`, `ricochet`, `bullet_speed`, `spread`, `recoil`, `kick`, `pull` | +52, +72, +68, +80, +84, +108, +112, +124 | the shot: what it makes, what it takes from what it hits (the bullet's damage, +76 of its record), the chance to bounce off a wall, the scale of the bullet's direction, how far it can leave the aim, the gun's recoil, the multiplier of the recoil and the pull on the view |
| the same with `alt_` in front | +188 and on | the second shot (alternate fire), laid out like the first (the same offsets plus 136) |

Not yet understood: the other words of the entry (+0 to +12 are numbers of props and animations, +56 and +96 to +104 are sound numbers, +396 is compared with a limit in `gunReload`),
and what the word at +76 of a shot (+24 of it) does (it goes to +80 of the bullet).  The kinds of bullet (shapes: shots, rockets, grenades, mines, thrown things) are code, chosen by `bullet_type`.  `max_ammo` is not in the entry: it is in `ammoinfo` by the weapon's kind of ammo.

* `modes.txt` (`ts-game/src/manual/mode-presets.rs`): presets for the six modes: the name and the options a mode starts with, written into the game's records of options right after `gameoptionsMake`.  Tested by unit tests and a load in the game (the records and the name text are right).  Variants (`variant "Name"` in `modes.txt`) are ways to play a mode that the player picks on a new Rules row of the arcade Options page (`arcadeopts.rs`).  The mode list itself is not extended: the arcade page that lists the modes loops over a table of six in the game's own code and its menu has no room for more, so a seventh mode is a variant of one of the six (it keeps the base mode's rules).  Compiles and the parser is tested; the Rules row has not been seen on screen yet.

* Step 1 of the modes plan: `docs/game-modes.md` writes down how the six ticks work and what state they use (the parameters record, the options records, a character's score field, the teams' scores).  The ticks turned out to be small: a win condition is a few lines over those fields.

* Rules of a variant (`rule win_score N`, `rule win_score_seconds N`, `rule win_lead N`: the best team ahead of the second by N): applied after the mode's own tick by `custom_rules_tick` (`mode-presets.rs`), which completes the level when a team's score reaches the goal.  Tested in a headless run with the test aids `TS_VARIANT` and `TS_RULEGOAL`: the level completes once with the variant on and not with Standard.  The first two parts of the plan's list (score methods, win conditions); team rules, objects, respawn and HUD text are not done.

* Characters, step 1 begun: `tools/obdump.py` and `docs/model-format.md` describe the structure of a character model (parts as a tree of 26 joints, three kinds of part, the arrays of a part), with what is not known marked.  The skin arrays, the rest pose and the animation files are the next things to read.

* Animations: the file layout is decoded and checked (`tools/anidump.py`, `docs/model-format.md`).  Models: the bind array (the joint each vertex of a seam follows) is read, and `obdump.py --obj` writes a character's triangles, which plot as a standing figure: the first drawing check passes.

* Writing a model: `tools/obedit.py` makes a mod with a changed character model; the game loads it, runs, and a head replaced by a sphere of quads made by `obedit.py mesh` is drawn too, and a head replaced by a box made from nothing is drawn by the game (`TS_CAMMODE=2:300` puts the camera in the third person view for a screenshot).  (An earlier line said a scaled `chr01_head.raw` was seen: that was wrong, the file is not loaded in a level.)

* Models: no limit on the size of a replacement mesh and on its textures (any number of materials, pictures of any size, `obedit.py mesh --tex-base`).  The size of one draw (64 vertices, 24 strips, 212 words of the vector unit's memory) is not a limit on a mesh: a mesh is cut into as many chunks as it needs.  Lifting that would mean changing the vector unit's own program.

* Characters: `character 80` and on in `characters.txt` add a character (the table `chrs` moved into the port's memory with room for 128, `chartable.rs`): a copy of a character with changed fields, a name text, and a model file of the mod's own that gets a prop of its own (prop numbers from 1073).  Tested by making the player's character use a new character's prop: the game loads the mod's `OB.PAK/ob/chrs/chr90.raw` (a character with a box for a head) and draws it.  The sign-on page's left/right reaches added characters (after the game's 64). Not done: the bot sets' lists, the availability masks, its sounds, a front end model, and the rest of the rig (joints, gun attachment) for a model that does not use the game's skeleton.
* Characters from a mesh: `tools/obchar.py` makes a mod with a new character from an OBJ of a figure (nearest-bone assignment, no seams yet); the game loads and draws it.  Rough: see `docs/model-format.md`.
* Weapon models: an added weapon can name model files of its own (`view_model_file`, `model_file`, `ammo_model_file`); each becomes a prop copying the weapon's own (`modelprops.rs`, shared with the characters).  Checked by the log: the prop is made with the mod's file.  Not seen on the screen.
* An added weapon in play (checked on screen with `TS_GIVE=60:49 TS_EQUIP=1`, the pair of an added pistol): it is held in both hands with its clip and reserve shown, and firing takes a round and makes a tracer.  This found a bug: `gunSetAmmoToMax` only knew the game's 30 weapons, so the ammo of an added weapon was not filled and the game dropped the weapon at once (now `gun_count()`).  Damage and the models of a weapon of its own are not yet seen at work.
* Models: the roles of the bones are read (the 7th and 11th parts of kind 0 are the hands, the 12th the head), so a custom character that keeps the 18 parts of kind 0 in the game's order holds guns and plays the animations.
* Models: the whole model can be exported to an OBJ and put back (`obdump.py --obj`, `obedit.py meshes`), and the head, done that way, draws as before; the limits of a chunk depend on its kind (found when the first round trip drew in pieces).
* Models: the rest pose is found (a skeleton file is a one frame animation, `tools/animtable.py` lists the animation numbers), and put together it is a person that agrees with the mesh.

## Order of work

1. The mod file format and the loader: one place (`mod.ini` sections or `characters.txt`, `weapons.txt`, `modes.txt`) where all three are described, with validation and clear errors in
   the log.  Small, and the three features share it.
2. Weapons first (the stats are tables and need only mapping), then game modes built from parts, then characters (they wait on the model format).
3. Each step: write what is known into `docs/`, add the table move and the hooks behind the features with the original expression when the feature is off, run the
   verifier, make a sample mod in `examples/`.
4. The default build stays the original: the checks of `docs/decompilation.md` run after every change.

## Decisions

* **Who writes the mods:** both kinds of people, those who edit text files and those who use a modelling tool.  So the text formats (`weapons.txt`, `modes.txt`, `characters.txt`) have
  to be readable and checked with clear messages, and the converters (a model exporter, as `obj2level.py` is for levels) have to be as easy to start as the level converter now is
  (answers for what is left out, defaults, a wizard when started without options).
* **First characters:** a mod that reuses the game's skeleton and animations with a new mesh and textures is good enough.  New skeletons and animations are not planned.
* **Modes:** a mode mod can add pages and menus of its own, not only use the game's.  This needs the menu and page code (`avmenu.rs` and the page tables) to be extensible
  the way the mode list is, and a step of the plan for it is added: after the mode list can be extended, make the pages a mode shows (rules, options, end of game, stats) come from
  a table the mod can add rows to.
