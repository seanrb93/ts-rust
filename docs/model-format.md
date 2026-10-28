# Model (object) files

The models of `OB.PAK` (`ob/...raw`: characters, weapons, props, doors, pickups) are read by `obLoad`, `obRelocate` and `obPrecalcGfxData`.  A model that does not move is laid out as the
model of a room of a level (`docs/level-formats.md`, "A room's model": the header, the texture table, the parts and their chunks, strips, positions, coordinates and colours); this document is what
a character adds.  `python tools/obdump.py <OB.PAK> <name>` prints the structure of a file (`--list [filter]` lists the objects).  It was written from the loader's code and from dumping the game's
own files; what is marked (guess) has been read from one file or one use only.

## What the loader does

* The first word of the file is the offset of the texture table, the second the offset of the object record.  All pointers in the file are offsets from its start; `obRelocate` adds the address
  the file was loaded at to the pointers of every part (the words at +8, +12, +16, +20 to +64 and +72 of its 80 bytes, when they are not 0) and clears +76.
* The object record is at the end of the file and the parts are the `n` records of 80 bytes right before it (`n` is the first word of the record).  The second word of the record of a
  character is the number of joints (26 in `chr01`, which has 35 parts).
* `obPrecalcGfxData` makes four arrays of a word for each part and, for each part and each of its two sets of chunks (solid, see-through), the vector unit and DMA data to draw it.

## Parts of a character

A part's record is that of a level's (`docs/level-formats.md`) with these differences in use:

| Field | A character |
|---|---|
| `+0` kind | 0 a mesh that is rigid on a joint, 1 and 2 parts that join two joints (guess: a skin that is drawn between a joint and the next, with the extra array at +40) |
| `+1` joint | the joint (bone) the part is drawn with; 255 for a part of kind 1 that has no joint of its own (guess) |
| `+2` to `+5` links | four signed bytes that make the parts a tree: in `chr01` the first is the part before (the parent), the second the next part (the first child), the third the start of a second branch (an arm), the fourth the part that the branch is the sibling of (all guess, from the numbers of `chr01`) |
| `+20` to `+40` | the strip table, positions, texture coordinates, colours, normals and (kind 1 and 2 only) a sixth array at +40 that a level does not have (guess: what the skin needs) |
| chunk kind | 3 for a rigid part (positions and normals), 4 for a part that joins joints |

The vectors at `+72` are what the code of a gun reads for the places on a model (the bones the muzzle flash and sparks come from, `gun + 340`, `+360` and `+364` of the table of weapons).

## What is not known yet

* The meaning of the sixth array and of the kind 1 and 2 parts: how a vertex is weighted between two joints, and in what order the joints' matrices are made.
* The skeleton's rest pose: where the joints are.  The animations (`ANIM.PAK`, `anim/data/*.raw`) hold the joints' motion by number, and what they are matched to is not read.
* How a gun is attached to a character (its model is a prop with parts that carry the muzzle points).

The plan for characters (`docs/mod-roadmap.md`) starts here: read those three, make `obdump.py` print a model's triangles and joints and check it by drawing the result, and only then write a
converter that makes the game's format from a skinned mesh.
