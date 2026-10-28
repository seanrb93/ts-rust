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
| `+0` kind | 0 a mesh that is rigid on a joint; 1 to 3 a skin drawn with up to three joint matrices at once (the vertices have weights in the sixth array: guess); 2 is also a "fillet": `calDoubleFillets` makes the matrix of a kind 2 part from its own joint and its parent's with `calInbetweenMatrix` (an elbow or knee that bends smoothly); 3 stops the walk down the tree |
| `+1` joint | the joint (bone) the part is drawn with; 255 for a part of kind 1 that has no joint of its own (guess) |
| `+2` to `+5` links | four signed bytes.  For a rigid part (kind 0) they make the tree: `+2` the parent, `+3` the first child, `+4` the next sibling, `+5` the previous sibling (`calDoubleFillets` follows `+3` and `+4`, and the numbers of `chr01` fit: part 5 has parent 4, child 6, next sibling 13, no previous; part 13 has parent 4, child 14, next 21, previous 5).  For a part that blends joints (kinds 1 to 3) `+2` and `+3` are the numbers of the two parts whose joints are blended (`partGfx` loads the matrices of those two parts' joints and then the part's own, `+1`, when it is not negative: three matrices for the vector unit) |
| `+20` to `+40` | the strip table, positions, texture coordinates, colours, normals and (kind 1 and 2 only) a sixth array at +40 that a level does not have (guess: what the skin needs) |
| chunk kind | 3 for a rigid part (positions and normals), 4 for a part that joins joints |

The vectors at `+72` are what the code of a gun reads for the places on a model (the bones the muzzle flash and sparks come from, `gun + 340`, `+360` and `+364` of the table of weapons).

## Animations

The animation record the game holds for each number (`pAnimations`) has the number of key frames at +4, flags at +12 (bit 1: it loops), the end time in seconds at +16 and a table of the key frames' times in 60ths of a second at +20 (`getKeyFrames` finds the two frames round a time).  `calMatrices` moves the time of a character's two animations (the second blends in while it
changes from the first), blends the position and rotation of every bone between the two key frames of each, turns the heading with the root, adds the hit reaction, makes the global matrices (and the fillets), tilts the head and body of characters that look about, applies the size of the cheats and keeps the
movement of the root.  The files of `ANIM.PAK` (`anim/data/*.raw`) start with small numbers, the second of them the key frame count (20 in `rf_m0`); the layout of the key frames is not read yet.

## What is not known yet

* The sixth array: how the weights of a vertex are stored for the three matrices.
* The skeleton's rest pose: where the joints are.  The animations (`ANIM.PAK`, `anim/data/*.raw`) hold the joints' motion by number, and what they are matched to is not read.
* How a gun is attached to a character (its model is a prop with parts that carry the muzzle points).

The plan for characters (`docs/mod-roadmap.md`) starts here: read those three, make `obdump.py` print a model's triangles and joints and check it by drawing the result, and only then write a
converter that makes the game's format from a skinned mesh.
