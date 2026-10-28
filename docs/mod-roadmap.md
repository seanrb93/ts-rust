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
