# Levels sample mod

Adds two levels to the arcade level list, after Ron: *Tomb Remix* (level 31: the Tomb's data, music track 7, and only Deathmatch, BagTag and Capture the Bag) and *Docks Night*
(level 32: the Docks).  `levels.txt` has one line for each level: `level <number 31..99> "<name>" like <game level 1..30> [tune <track 1..27>] [modes dm,bagtag,ctb,escort,ko,ls] [setup <file>]`.  `setup` names a file
with the level's own spawns and pickups by the numbers of its pads (`spawn <pad>`, `weapon <slot 1..5> <pad>`, `pickup <type> <pad>`, `modes ...`; `tools/obj2level.py` writes one).

A new level is made from the level it is `like`: it has that level's set-up (the modes, unless `modes` is given), and, where the mod has no files of its own for it, copies of its data
under the new number.  A mod puts its own data where the level's files are: `BG.PAK/bg/level<number>/level<number>.raw`, `MISC.PAK/pad/data/level<number>.raw`,
`MISC.PAK/pad/data/level<number>_reduced.raw` and `MISC.PAK/propsetup/data/level<number>.raw` (the game's own binary formats, in `docs/level-formats.md`; `tools/obj2level.py` writes them from a 3D model).
