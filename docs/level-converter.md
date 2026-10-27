# Making levels from other formats

`tools/obj2level.py` makes a level for the port (a mod folder, see `docs/level-formats.md` for what is in it) from a 3D model or from a map of
another game:

```
python tools/obj2level.py <file> --out mods/MyLevel --number 33 --name "My Level" --like 25 [options]
```

Python 3 with `numpy` and `Pillow` is needed (`pip install numpy pillow`).  The mod appears in the Mods page and its level in the arcade list.

| format | file | notes |
| --- | --- | --- |
| Blender | `.blend` | Blender is run in the background to export the file (it is looked for in the places it installs to, the `BLENDER` environment variable and `--blender <exe>`).  Modifiers are applied. |
| glTF 2.0 | `.glb`, `.gltf` | What Blender (File > Export > glTF 2.0), and most tools, export to for games.  Pictures inside the file or next to it. |
| Wavefront OBJ | `.obj` | With its `.mtl`: `map_Kd` pictures (PNG, JPEG, ...), `Kd` colours, `d` for see-through, and `v x y z r g b` vertex colours. |
| Source engine map | `.bsp`, `.bsp.bz2` | Half-Life 2, Garry's Mod, Counter-Strike: Source, Team Fortress 2, Portal, Left 4 Dead, ... (versions 17 to 21).  The world's faces and displacements (terrain), with the map's light baked into a brightness for each face. |
| Source map source | `.vmf` | The world's brushes (cut from their planes) and displacements. |
| GoldSrc map | `.bsp` (version 30) | Half-Life, Counter-Strike 1.6, Team Fortress Classic, Day of Defeat, ... |

## Markers

Where players start and where weapons lie is taken from the file:

- Blender, glTF: an Empty (a node) named `spawn...` (where players and bots start), `weapon1` .. `weapon5` (a weapon of that slot of the weapon set the
  player chose) or `pickup<type>` (a pickup by the game's number, see `docs/level-formats.md`).  A mesh named like that is a marker too (its origin is the
  spot) and is not part of the level.
- OBJ: an object or group (`o` / `g`) with such a name: its centre is the spot.
- Source and GoldSrc maps: the player starts (`info_player_start`, `info_player_deathmatch`, `info_player_teamspawn`, `info_player_terrorist`, ... ) and
  the weapons (`weapon_*`, whose slot is taken in turn).

Without any, spots are picked on the floors (`--spawns` of them, and `--weapons` of each slot).  A marker is moved to the nearest pad.

## What becomes of the level

- Faces that point up (less than 60 degrees from level) are floors that players stand on and bots walk on; the others, apart from ceilings, are walls.  A
  wall stops only what comes from the side the face points to: a thin wall needs a face for each side.
- Pads for the bots are laid on the floors (`--pad-spacing`; at most `--max-pads`) and linked to their neighbours, in groups the game finds routes by.
- Pictures are made into the game's texture files: sides of powers of two, at most 256, at most 65536 pixels.  Surfaces with see-through pixels are drawn
  after the solid ones.  Of a map with more than `--max-textures` (90) materials, those that cover least are drawn with the picture of the most alike colour among the rest.  The
  texture files are made smaller until they add up to `--texture-budget` (640 KB, the most the game's own levels use).
- Axes and size: Source and GoldSrc maps are z up and 72 units tall for a player: they are turned and scaled by 0.025 (`--scale`, `--up`).  A model is taken as it
  is: the game's rooms are about 20 units across and a player about 1.8 high.  `--flip` turns the faces round if the level comes out inside out.
- **Music.**  `--music song.mp3` (any audio file; ffmpeg is needed, `pip install imageio-ffmpeg` will do, unless it is already `.opus`) makes the level play that song: it is
  written to the mod as `MUSIC/10<number>.opus` and the level's `tune` is its number.  A mod can also give any level of the game's a tune above 27 in `levels.txt` (`tune <1..99>`) and
  put `MUSIC/10<tune>.opus` in the mod.  The file loops when it ends.  `--tune 1..27` picks one of the game's own tracks instead.
- `--like <1..30>` chooses the level whose sky, fog and light the new one has (a level of the game's is used for the arcade modes, the music and these).

## Source and GoldSrc maps

- **Pictures.**  A Source material (`.vmt`) names a texture (`.vtf`).  Custom ones are in the map's own pack and are found there; the game's own
  (`concrete/...`, `brick/...`) are not in the map: unpack them with a tool such as GCFScape or Crowbar and give the `materials` folder with `--materials <dir>` (a
  `materials` folder next to the map or in one of its parent folders, as Crowbar unpacks a workshop map, is found by itself).  A surface whose picture is not found is
  drawn in the average colour the map recorded for it.  GoldSrc textures are in the map, or in the `.wad` files its worldspawn lists (found next to the map, or in the
  folders given with `--wad <dir or file>`).
- **Light.**  Both engines light surfaces with lightmaps (see Light below for what is made of them).
- **Brush entities.**  The ones that are only a shape are part of the level, with their collision: `func_brush`, `func_wall`, `func_wall_toggle`, `func_illusionary`,
  `func_breakable`, `func_button` and, where they start, the rotating and moving parts (`func_rotating`, `func_movelinear`, `func_tracktrain`, `func_physbox` in Source;
  `func_conveyor`, `func_pendulum`, `func_plat`, `func_train` in GoldSrc).  Entities that are not drawn (render mode 10, or a render amount of 0) or start disabled are left out.
  Doors (`func_door`, `func_door_rotating`) are left out too, because a closed door would shut rooms off: `--doors closed` makes them part of the level, closed.  `--no-entities`
  leaves all of them out.  Source maps (BSP and VMF) and GoldSrc maps are handled alike; nothing moves, so a part that is meant to move stays where the map starts it.
- **Props.**  The models a map places are put in as geometry, where the map starts them: the static props of a BSP (`prop_static`), and entities with a model
  (`prop_dynamic`, `prop_physics`, `prop_door_rotating`, ...) in a BSP or VMF.  A model is its `.mdl`, `.vvd` and `.dx90.vtx` files, in its most detailed level and its first skin; bones,
  animation, physics and the models' lights are not used, and props do not collide.  The map's own models are in its pack or in a `models` folder beside the map (as Crowbar
  unpacks a workshop map); the game's own (`props_c17/...`) are unpacked with GCFScape or Crowbar and given with `--models <dir>`; a prop whose model is not found is left out.
  All the props are put in by default (Station Square: 219 props, 48,000 polygons, and the level plays); `--prop-budget N` keeps the polygons they add to N by leaving out the ones that are small for their polygons, and `--no-props` leaves them all out.
- **What is left out.**  The 3D skybox, entities that move or act (triggers, water, ladders), sky, trigger, clip and other tool faces.  Parts that are not connected to the main level (a
  skybox room, a room of tools) are dropped.
- **Size.**  The game's own levels have at most 70,000 vertices, 109 rooms and a 2.7 MB BG file.  The port's tables were made larger (`manual/proptable.rs` and the patches of
  `tools/patches.py`), so a level can have up to 480 rooms (props in the rooms numbered 205 to 212 are drawn in the passes the game keeps for those numbers, which may show as a prop drawn through
  a wall), and a level of a mod (the numbers 31 to 99) makes its large allocations (256 KB and more: the BG file, the biggest textures, ...) in a pool of 32 MB of the port's own (`manual/bigpool.rs`;
  the game's pool of 21 MB cannot be longer, for the stacks and static data are after it, but the port's memory is not the console's).  The full Station Square of Garry's Mod (6.9 MB, 40,000 polygons,
  232 rooms) was played.  `--max-size` (default 28 MB; the port's big pool for the levels of mods is 32 MB) leaves the smallest polygons that are not floors out above that size.  Station Square with its props (90,000 polygons, 331 rooms, a 14 MB BG file) loads and plays.  A big map can also be made smaller by choosing a part of it:
  `--radius 70 --center x,z` (game units, a square; the first spawn is the default centre) keeps what is within 70 units (a Source map of 16000 units is 400 units across).
- **Rooms.**  `--room-size 25` splits the level into rooms of 25 x 25 units joined by portals: what is far away is not drawn.  A map of Source or GoldSrc size wants it.

## Collision and rooms

- **Source maps: collision from the brushes.**  The solid brushes of the world (solid, window, grate, player clip: the invisible walls too) give the floors and walls, with the sides that
  are inside other brushes left out.  `--no-brushes` makes them from the drawn faces instead.  Displacements (terrain) collide by their triangles.
- **The game's room boxes.**  The game works out a room's box from its floors and walls and clips what it sees of the room with it, so the box has to hold everything drawn of the room.  The
  converter adds two walls a thousandth of a unit long at the corners of what is drawn; without them, rooms whose floors and walls cover less than the drawing were cut off or missing (the black ground
  of the first Station Square mod).
- **Winding.**  The faces of a map have points along their edges, so the first three points of a face may lie on a line: the facing of a face is taken from all its points (a face that comes out
  upside down is not drawn, and is not a floor).  This was the cause of the missing ground in the first Station Square mods.
- **Light.**  Every vertex gets the light of the map's lightmap at its place; a face whose light varies much is cut along the lightmap's grid (`--light-detail`, luxels between cuts, default 16; 0 gives each
  face one brightness).
- **Visibility.**  A compiled Source map has its own visibility data (which clusters see which), but no portals, and the game works with rooms and portals, so the converter makes its own: a grid of rooms
  (`--room-size`) split where too full, with portals between touching rooms.  The map's visibility data is not used.
- **Portals** are low openings (14 units) at the heights where both rooms have a floor.

## Limits worth knowing

The game keeps the lists of the props that are held, thrown or floating (the guns of the players, water, effects) under the room numbers 201 to 212, since
its levels have fewer rooms.  A level with more rooms than 200 gets twelve empty rooms at these numbers and the rooms after them are numbered 12 higher.

The game's bots work out their routes when the match starts.  This takes longer the more pads and links there are, and a level with all its pads in one group
took minutes; pads are therefore grouped (about 16 each) and `--max-pads` is 450.  Textures that are not powers of two draw washed out, so the converter makes them powers of two.
Reflections, glass that breaks and moving parts are not made, props do not collide, and doors are only closed with `--doors closed`: a level is static geometry.
