# Level from a 3D model

`tools/obj2level.py` turns a 3D model (OBJ here; also glTF, `.blend`, and maps of Source and GoldSrc games: see `docs/level-converter.md`) into a mod folder with a level that shows up in the
arcade level list.  This sample makes a small arena to try it on; the pictures are drawn by the script, nothing of the game's is used.

```
python examples/obj_level_sample/make_arena.py arena
python tools/obj2level.py arena/arena.obj --out mods/Arena --number 33 --name "Arena" --like 25
```

Then start the port.  The mod loader lists the level after the game's own ones.  Pick it in Arcade, choose a mode and play; bots run about on
the floors and pick up the weapons that were laid out.

## What a model needs

- Faces that point up (less than 60 degrees from level) are floors: players stand on them and bots walk on them.  A level with none is refused.
- Steeper faces are walls that stop players.  Faces that point down (ceilings) are only drawn.
- Each material of the OBJ is a texture: the `map_Kd` picture of the MTL file (a PNG), or the `Kd` colour.  Pictures are made into the game's
  texture files, with sides that are powers of two (at most 256) and at most 65536 pixels in all.  A material whose picture has see-through
  pixels is drawn after the solid ones.  At most 36 materials.
- Texture coordinates are used as they are (they may go outside 0 to 1: the picture repeats).  `v x y z r g b` colours are used as lighting.
- A quad is drawn as it is; a face with more than four corners is split into triangles.
- Objects (or groups) that are markers: `spawn...` (where players and bots start), `weapon1` .. `weapon5` (where a weapon lies: the number is
  the slot of the weapon set the player chose) and `pickup<type>` (a pickup by the game's number, see `docs/level-formats.md`).  The spot is
  the centre of the marker's vertices; make it a small triangle on the floor.  Without markers, spots are picked on their own.

## Options

`--scale` (a player is about 1.8 units high, the game's own rooms about 20 wide), `--up z` (for models with z up), `--flip` (when the level is
inside out), `--room-size R` (split a large level into rooms of R x R units joined by portals, so that only what can be seen is drawn),
`--modes dm,bagtag,ctb,escort,ko,ls`, `--tune <track 1..27>`, `--like <game level 1..30>` (the level whose sky, fog and colours it gets;
the colour of the light a level has comes from this), `--pad-spacing` (how far apart the pads of the bots' map are), `--number` (31 to 99).

What the converter writes is described in `docs/level-formats.md`; `python tools/lvdump.py --mod mods/Arena 33` checks the files.
