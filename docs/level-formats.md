# Level file formats

This is what the game reads for one level, found by reading the game's own loading code (`bgLoad`, `obRelocate`, `obCalcPartBounds`,
`obCalcChunkBounds`, `obMergeChunkPolys`, `partGfx`, `padLoad`, `padReset`, `padSave`, `propEditSave`, `bgRoomInitBB`, `bgPortalInitBB`,
`lvLevelSetup` and the functions around them) and by checking every level of the real game files against it.  `tools/lvdump.py <PAK folder>`
reads all three files of every level with the layouts below and checks every offset, count and terminator that is claimed here; it passes for
all 30 levels.  Where a field is only guessed from its values this is said so.  `tools/obj2level.py` is a converter that writes these files
from an OBJ model, and levels made with it have been played.

All numbers are little endian.  A level `N` is three files (plus a reduced pad file for level 27):

| file | pack | path |
| --- | --- | --- |
| the world: rooms, their models, floors, walls, portals | `BG.PAK` | `bg/levelN/levelN.raw` |
| the points that bots and players use | `MISC.PAK` | `pad/data/levelN.raw` (and `levelN_reduced.raw`) |
| the props placed in the world | `MISC.PAK` | `propsetup/data/levelN.raw` |

Levels 1, 3, 4, 6, 7, 19 and 28 have a tiny BG file (two rooms, 58 vertices, 3792 bytes): their world is built from tile sets when the level
starts (`bgTileLoad`, the files `pad/data/tileX_Y.raw`).  Those are not described here; the BG layout below is for the levels that carry their
own world (2, 5, 8 to 18, 21 to 27).  Levels 20, 29 and 30 have no BG file.

## Packs

`P4CK`, then the offset and size of the directory (two 32 bit numbers).  The directory is 60 byte records: a name (48 bytes, ended by 0), an
offset and a size.  `tools/pakls.py` reads them.  A texture is the file `textures/NNNN.qpm` of `MISC.PAK`.

## Pad file (`pad/data/levelN.raw`)

The game's own tool writes this as assembler text (`pad<rev> id, flags, hall, x, y, z, w` and `link<rev> flags, id, id, f`, from `padSave`);
the file in the pack is the binary of that text.

```
u32 revision        1
u32 pads
u32 links
pad  x pads         28 bytes
link x links        20 bytes
```

A pad (28 bytes): `i32 id` (the "extref": 1001 and up in a level, the number links, props and the set-up use), `u32 flags` (bit 0: the pad is
used; the game counts these; bit 1 is set on some pads and is not decoded), `u16 hall`, `u16 room` (see below), `f32 x, y, z` (the position:
the height is the floor's, plus about 0.01), `f32 w` (the direction the pad faces, in degrees).

The hall is a group of pads: the game works out the bots' routes between pads by halls (`padReset` makes a table of them, as many as the
highest hall number says, and the links of each).  The levels of the game have 20 to 45 halls (a corridor, a room).  A level with one hall for all of
its pads works (a hall for each BG room was tried: the route search did not finish in minutes).  The room is the BG room the pad is in; the files have 0 there.

A link (20 bytes): `u32 flags` (bit 0 set: the link is used; bit 1: it can be walked in one direction only), `i32 id_a`, `i32 id_b` (the pads it
joins, by id), `f32` (0 in all of the levels read), `u32` (0).  `padLoad` turns the ids into pad numbers and drops links whose pads do not exist.  A
file with a link dropped is 20 bytes shorter than its counts say (`tile1_17.raw` is such a file); the loader does not mind.  The game sets
aside room for 2400 pads and 4400 links.  The bots work out routes between spawn spots and weapons before the match starts, which takes
longer the more links there are: about two links to a pad is what the game's own levels have.

Where the bots go, where players start and where pickups appear all come from these pads (the arcade set-up refers to them by id).

## Prop file (`propsetup/data/levelN.raw`)

```
u32 revision        1
u32 props
prop x props        24 bytes
```

A prop (24 bytes): `i32 id` (1001 and up), `u32 type` (an index of the prop table of the game: 184, 178, 179, 183 are seen), `f32 x, y, z`,
`f32` (the rotation).  Only the story levels (2, 5, 21, 23, 24) have props in this file (8 bytes, no props, in the others).  The text form is
`prop1 id, type, x, y, z, r`.  Arcade matches get their pickups and weapons from the level's set-up in the game's code, not from here.

## BG file (`bg/levelN/levelN.raw`)

All offsets in the file count from the start of the file (the game adds the address the file is loaded at to every one of them).

### Header (16 bytes)

```
u32 textures        offset of the texture table
u32 rooms           offset of the room table
u32 portals         offset of the portal table
u32 volumes         offset of the volume list, 0 if there is none
```

### Texture table

16 bytes each, ended by an entry whose first number is -1: `i32 texture_number` (the number of the game's texture: the file `textures/NNNN.qpm`
of `MISC.PAK`; `ts_rt::texnames` and `tools/extract_textures.py` map numbers to files), then `i32 0 or 1` twice (the game overwrites these when it
loads the textures; 0 is fine) and `u32 material`.  The material is 0 for most surfaces; the other values (1, 2, 3, 11 to 15, 27, 29, ...) are
most likely the kind of surface (what footsteps and bullet marks do) and are not decoded; 0x40000000 marks a texture drawn as a reflection (see
the chunk kind below).  A model's chunks name a texture by its place in this table.

### Rooms

44 byte records; record 0 is all zeros and is not a room.  Rooms are numbered from 1 and the table ends with the first record whose first word
is 0.

```
+0   u32 model        offset of the room's model (see below)
+4   u32 neighbours   offset of a list: u32 count, then count portal numbers.  Needed (a count of 0 will do)
+8   u32 floors       offset of a list of offsets, ended by 0 (an entry of 0 inside the list is a deleted floor)
+12  u32 walls        the same, for walls
+16  u32 glass        the same, for glass (0 if there is none)
+20  f32 x   +24 f32 y   +28 f32 z       the room's box: the least corner
+32  f32 x   +36 f32 y   +40 f32 z       the greatest corner
```

The box is what the game tests a position against (`bgCalcRoomArray`); the game also recomputes a box from the floors and walls when it
loads (`bgRoomInitBB`).

A floor: `u32 vertices` (3 or 4 are seen), `u16` (0), `u16 flags` (bit 4, 16, is set by the loader when the first three vertices have the same
height; 2 is seen once in the file), then the vertices, 12 bytes each (`f32 x, y, z`), from +8, going round counter-clockwise seen from above.  A
floor is 8 + 12 * vertices bytes.

A wall (36 bytes): `u32` (0), then two upright edges as `f32 x, y_low, z, y_high` each, from +4 and from +20.  A wall stops only what comes
from one side: the open side is to the right of the direction from the first edge to the second (`dx * oz - dz * ox < 0` for a vector
`(ox, oz)` to the open side).  A wall seen from its other side lets things through, so a thin wall needs one wall for each side.

Glass (196 bytes) is not decoded beyond its place in the list; a level can leave the list out (0).

### Portals

`portals` points at an array of offsets; entry 0 is not used, entries 1 and on are portals, ended by 0 (an empty array is two zero words).  A
portal is a flat polygon in the opening between two rooms:

```
+0   u32 room_a        +4  u32 room_b
+8   f32 x, y, z       the normal of the polygon: it points into room_b
+20  u16 vertices      3 or more (4 in every level read)
+22  u16 0
+24  the vertices, 12 bytes each (f32 x, y, z), going round counter-clockwise seen from the side the normal points to
```

That is, the cross product of the first two edges is the normal.  Real portals reach from the floor to the ceiling of the opening.  The portal
numbers in a room's neighbour list count from 1.  A level with a single room has no portals.  What the player sees is the room the player is in and
the rooms that can be seen through portals from it: a surface that is in a room that is not reached through portals is not drawn, so a polygon
must be cut at the edges of the rooms it passes through (the converter does this).

### Volumes

At `volumes`: `u32 count`, then `count` records of 44 bytes.  `u32 id` first (the game looks a volume up by it), a position (`f32 x, y, z`) at
+8, a direction at +20, an angle in degrees at +36; the rest is not decoded.  Levels 2, 8, 14, 21, 22, 23, 24 and 27 have volumes.  A level
can have none (the header word is 0).

### A room's model

The model offset points at the END of a block: a `u32 part_count` (1 in every room of every level read), and the parts are the `part_count`
records of 80 bytes that come directly before it.  After the count the block has more words that the game fills in (the pointer to the texture
table at +12, four arrays of per-part drawing data at +20 to +32, a flag at +36); a converter writes 64 bytes after the count, all 0.

A part (80 bytes; the offsets that the game relocates are marked *):

```
+0   u8 kind            0: a model that does not move (all rooms)
+1   u8 matrix          the joint of a skeleton the part is attached to (0)
+2   i8 -1, +3 i8 -1, +4 i8 -1, +5 i8 -1     links to other parts (none)
+6   u8 lamps           the number of records at +16
+7   u8 0               the number of vectors at +72
+8   *  chunks A        the solid surfaces
+12  *  chunks B        the see-through surfaces; 0 if there are none
+16  *  lamps           `lamps` records of 152 bytes: two outlines of five points (f32 x, y, z) at +8 and at +68, a direction at +128;
                        light fittings that the game draws a glow at.  0 if there are none.
+20  *  A: strip table      16 byte records (below)
+24  *  A: positions        12 bytes each: f32 x, y, z
+28  *  A: texture coords   16 bytes each: f32 s, t, 1.0, 1.0
+32  *  A: colours          4 bytes each: r, g, b, a (the game's colours: 0x7f is full, a of 0x7f is opaque)
+36  *  A: normals          12 bytes each, only for chunks of kind 3 and 4 (not used in the levels)
+40  *  A: (0)
+44  *  B: strip table      +48 B: positions   +52 B: coords   +56 B: colours   +60 B: normals   +64 B: (0)
+68  u16 mask            0: the part is always drawn (otherwise the game draws it when the model instance's mask has one of these bits)
+70  u16 triangles       how many triangles the part has in both sets (statistics only)
+72  *  vectors          `+7` vectors (0)
+76  runtime            the game puts a pointer here
```

A list of chunks is 24 byte records ended by a record whose last word is negative:

```
+0   i32 texture slot     the place in the texture table (0 .. count - 1)
+4   i32 first strip      index into the strip table
+8   i32 strips           how many strip records the chunk uses
+12  i32 first vertex     index into the positions, coords and colours: a multiple of 4 (see below)
+16  i32 vertices         how many vertices the chunk draws (the sum over its strips): at most 64
+20  i32 kind             0: an ordinary surface.  2: the surface is drawn as a reflection (the texture has material 0x40000000 and the
                          coords are made from the direction of the surface).  3 and 4 add normals.  Negative ends the list.
```

A chunk is one draw of the console's vector unit, so it holds at most 64 vertices (the largest in the game has 64).  The data of a chunk goes
to the unit by DMA from the arrays at `array + first * size`; the console's DMA wants these addresses on a 16 byte boundary, so the first vertex
of a chunk is a multiple of 4 (the arrays have padding after a chunk whose vertex count is not) and each array starts on a 16 byte boundary.

The strip table is the GIF data the game draws with: 16 bytes each, `u32 vertex_count | 0x8000 when it is the chunk's last strip | 0x10000 for
a strip that starts with the other winding` (the game's own strips use the last bit in about one in five; the converter does not), `u32 0x302e4000`,
`u32 0x412`, `u32 0`.  The last three words are the same in every strip of every level (about 129,000 read): a GIF tag that draws a triangle strip
(Gouraud, textured, alpha blended, ST coordinates) whose vertices are given as ST, RGBAQ, XYZF2.  So every strip is a triangle strip of
`vertex_count` vertices taken one after another from the chunk's range: triangle i is vertices i, i+1, i+2, and every other one is turned round.  A
polygon that is wound counter-clockwise (seen from the front, the normal by the right hand rule) is drawn as a front face; a quad a, b, c, d (going
round) is the strip a, b, d, c.  Texture coordinates are not normalised to a picture: they repeat.

`tools/lvdump.py` checks, for every chunk, that its strips add up to its vertex count, that its strip count is the one it says, and that the
arrays hold its vertices.

## The arcade set-up (not in the files)

Where players start and what lies about is not in these three files but in a level's set-up in the game's code (`setup_tomb_arcade` and the
like, found by `lvGetARCADESetup(level)`).  It is 112 bytes, and refers to the level's pads by their ids:

```
+0   u32 modes          the arcade modes the level has: 1 deathmatch, 2 bagtag, 4 capture the bag, 8 escort, 16 knockout, 32 last stand
+4   u32                a small number (0 to 8), copied from the level it is like
+8   +12                props (28 bytes each) and their count: set pieces of the level, 0 for none
+16  +20                pickups (48 bytes each) and their count
+32  +36                spawns (16 bytes each) and their count
+40 .. +111             more lists of props (a pointer and a count each), 0 for none
```

A spawn: `u32 flags` (bit n of 0x3f: the record is used in mode n + 1), `i32 pad` (the pad's id; the game puts the pad's number into +12 at
the start of a level), `u32 who` (bit 0: bots, bit 1: players; 3 is both), `u32 0`.

A pickup: `u32 flags` (as for a spawn), `i32 type`, `u32 kind`, `u32 0, 0, 0`, `i32 pad` at +24, `f32 0, 0` at +28 and +32, `f32 90` (the angle) at
+36, `f32 0.15` at +40, `u32 0`.  The types are those of the game's set-ups: -1 to -5 are the weapons of a weapon set's slots (the player's
choice of weapon set says which weapon a slot is; kind 2), -6 to -9 (kind 16), -21 to -25 (kind 32), -30 (kind 4) and -32, -33 (kind 8) are
others that are not decoded (ammunition and armour among them), and 324 to 338 are the game's numbered pickups (kind 1; 338 has kind 16).

The port's levels have this in a `setup.txt` (`levels.txt` says `setup <file>`): `modes ...`, `spawn <pad id>`, `weapon <slot 1..5> <pad id>`,
`pickup <type> <pad id>`.

## What a converter has to produce

- A BG file: a texture table; one room (a part with a chunk per run of up to 64 vertices of one texture, and strips) for the whole level or for
  each box of it; floors (what players stand on) and walls (what stops them), as polygons; portals between neighbouring rooms, which decide what
  is drawn (a single room works, with no portals); the rooms' neighbour lists.
- A pad file: pads on the floors, with links between the pads that bots may walk between (the links are the bots' map).
- A prop file: can have no props (8 bytes).
- The textures as `.qpm` files and a set-up (above).

## Not decoded

Glass, the volumes' remaining fields (volumes are only used by the tile sets' levels), the meaning of material numbers, normals (chunks of kind 3
and 4 are in no level), floor flags other than the one the loader sets, bit 1 of a pad's flags, how halls should be chosen so that bots
route well in a large level, and the tile sets that make the story's tile levels.
