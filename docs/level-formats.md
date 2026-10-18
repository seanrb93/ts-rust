# Level file formats

This is what the game reads for one level, found by reading the game's own loading code (`bgLoad`, `obRelocate`, `obCalcPartBounds`,
`obCalcChunkBounds`, `padLoad`, `padSave`, `propEditSave`, `bgRoomInitBB`, `bgPortalInitBB` and the functions around them) and by checking every
level of the real game files against it.  `tools/lvdump.py <PAK folder>` reads all three files of every level with the layouts below and checks
every offset, count and terminator that is claimed here; it passes for all 30 levels.  Where a field is only guessed from its values this is
said so.  Nothing in this document is a converter: it is what a converter has to write.

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
offset and a size.  `tools/pakls.py` reads them.

## Pad file (`pad/data/levelN.raw`)

The game's own tool writes this as assembler text (`pad<rev> id, flags, room, x, y, z, w` and `link<rev> flags, id, id, f`, from `padSave`);
the file in the pack is the binary of that text.

```
u32 revision        1
u32 pads
u32 links
pad  x pads         28 bytes
link x links        20 bytes
```

A pad (28 bytes): `i32 id` (the "extref": 1001 and up in a level, the number links and props use), `u32 flags` (bit 0: the pad is used; the
game counts these), `u16 room` (the room it is in), `u16` (0), `f32 x, y, z` (the position), `f32 w` (the fourth number of the text line; 0
in all of the levels read).

A link (20 bytes): `u32 flags` (bit 0 set: the link is used), `i32 id_a`, `i32 id_b` (the pads it joins, by id), `f32` (0 in all of the levels
read), `u32` (0).  `padLoad` turns the ids into pad numbers and drops links whose pads do not exist.  A file with a link dropped is 20 bytes
shorter than its counts say (`tile1_17.raw` is such a file); the loader does not mind.

Where the bots go, where players start and where pickups appear all come from these pads (the arcade set-up refers to them by id).  So a level
needs pads, and links between them for the bots to walk: the links are the bots' map.

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

16 bytes each, ended by an entry whose first number is -1: `i32 texture_number` (the number of the game's texture; `ts_rt::texnames` and
`tools/extract_textures.py` map numbers to files), then three words that the game overwrites at load time (seen: 0 or 1, the same for the
first two; the third has small numbers and once 0x40000000).  A model's chunks name a texture by its place in this table.

### Rooms

44 byte records; record 0 is all zeros and is not a room.  Rooms are numbered from 1 and the table ends with the first record whose first word
is 0.

```
+0   u32 model        offset of the room's model (see below)
+4   u32 neighbours   offset of a list: u32 count, then count portal numbers
+8   u32 floors       offset of a list of offsets, ended by 0 (an entry of 0 inside the list is a deleted floor)
+12  u32 walls        the same, for walls
+16  u32 glass        the same, for glass (0 if there is none)
+20  f32 x   +24 f32 y   +28 f32 z       one corner of the room's box
+32  f32 x   +36 f32 y   +40 f32 z       the other corner
```

The first corner is the minimum and the second the maximum in every room read.  The game also recomputes a box from the floors and walls when
it loads (`bgRoomInitBB`), and the stored one is read before that.

A floor: `u32 vertices` (3 or 4 are seen), `u16` (0), `u16 flags` (bit 4, 16, is set by the loader when the first three vertices have the same
height; 2 is seen once in the file), then the vertices, 12 bytes each (`f32 x, y, z`), from +8.  A floor is 8 + 12 * vertices bytes.

A wall (36 bytes): `u32` (0), then two upright edges as `f32 x, y_low, z, y_high` each, from +4 and from +20.

Glass (196 bytes) is not decoded.

### Portals

`portals` points at an array of offsets; entry 0 is not used, entries 1 and on are portals, ended by 0.  A portal: `u32 room_a`, `u32 room_b`,
`u16 vertices` at +20 (3 or more), and the vertices from +24 (12 bytes each).  Bytes +8 to +19 and +22 are not decoded.  The portal numbers in
a room's neighbour list count from 1.

### Volumes

At `volumes`: `u32 count`, then `count` records of 44 bytes.  `u32 id` first (the game looks a volume up by it), a position (`f32 x, y, z`) at
+8, a direction at +20, an angle in degrees at +36; the rest is not decoded.  Levels 2, 8, 14, 21, 22, 23, 24 and 27 have volumes.

### A room's model

The model offset points at the END of a block: a `u32 part_count` (1 in every room of every level read), and the parts are the `part_count`
records of 80 bytes that come directly before it.  After the count the block has more words: +12 gets the game's pointer to the texture table
at load, +20 to +32 are four arrays of pointers that the game fills in when it prepares the drawing data.  In the file they are 0.

A part (80 bytes; the offsets that the game relocates are marked *):

```
+0   u16 0, u16 0xffff
+4   u16 0xffff, u16 n      n has been 0 to 24 in the levels read (not decoded)
+8   *  chunks A            list of chunks (below), the solid surfaces
+12  *  chunks B            list of chunks, the second kind (see-through surfaces); 0 if there is none
+16  *  (not decoded)       0 in most parts
+20  *  A: strip table      16 byte records (below)
+24  *  A: positions        12 bytes each: f32 x, y, z
+28  *  A: texture coords   16 bytes each: f32 s, t, q (1.0) and a fourth float
+32  *  A: colours          4 bytes each: r, g, b, a (a of 0x7f is opaque)
+36  *  A: normals          12 bytes each, only for chunks of kind 3 and 4 (not seen in the levels read)
+40  *  A: (not decoded)
+44  *  B: strip table      +48 B: positions   +52 B: coords   +56 B: colours   +60 B: normals   +64 B: (not decoded)
+68  (not decoded)
+72  *  (not decoded)
+76  runtime              the game puts a pointer here
```

A list of chunks is 24 byte records ended by a record whose last word is negative:

```
+0   i32 texture slot     the place in the texture table (0 .. count - 1)
+4   i32 first strip      index into the strip table
+8   i32 strips           how many strip records the chunk uses
+12  i32 first vertex     index into the positions, coords and colours
+16  i32 vertices         how many vertices the chunk draws (the sum over its strips)
+20  i32 kind             0 or 2 here; 3 and 4 add normals; negative ends the list
```

The strip table is the GIF data the game draws with: 16 bytes each, `u32 vertex_count | 0x8000 when it is the chunk's last strip` (bits 16 to
31 of this word are 0 or 1, not decoded), `u32 0x302e4000`, `u32 0x412`, `u32 0`.  The last three words are the same in every strip of every
level (about 129,000 read): a GIF tag that draws a triangle strip (Gouraud, textured, alpha blended, ST coordinates) whose vertices are given as
ST, RGBAQ, XYZF2.  So every strip is a triangle strip of `vertex_count` vertices, taken one after another from the chunk's range.

`tools/lvdump.py` checks, for every chunk, that its strips add up to its vertex count, that its strip count is the one it says, and that the
arrays hold its vertices.

## What a converter has to produce

- A BG file: a texture table; one room (a part with one chunk per texture and triangle strips) per box of the level; floors and walls for
  collisions (the rooms' floors are what players stand on, walls stop them); portals between neighbouring rooms, which decide what is drawn
  (a single room works, with no portals); the rooms' neighbour lists.
- A pad file: pads for the spawns, pickups and bot waypoints, with links between the pads that bots may walk between.
- A prop file: can have no props (8 bytes).

## Not decoded

The part fields marked above, the three words of a texture table entry, a wall's and floor's use of the game's flags, glass, the volumes'
remaining fields, portal bytes +8 to +19, the high half of a strip's first word, the arrays that the game prepares at load, and the tile sets
that make the story's tile levels.  Those are the places to look first if a level made by a converter does not draw or play correctly.
