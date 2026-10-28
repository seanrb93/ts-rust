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
| `+0` kind | 0 a mesh that is rigid on a joint; 1 to 3 a seam between joints, drawn with up to three joint matrices at once (each vertex names the joint it follows in the bind array: below); 2 is also a "fillet": `calDoubleFillets` makes the matrix of a kind 2 part from its own joint and its parent's with `calInbetweenMatrix` (an elbow or knee that bends smoothly); 3 stops the walk down the tree |
| `+1` joint | the joint (bone) the part is drawn with; 255 for a part of kind 1 that has no joint of its own (guess) |
| `+2` to `+5` links | four signed bytes.  For a rigid part (kind 0) they make the tree: `+2` the parent, `+3` the first child, `+4` the next sibling, `+5` the previous sibling (`calDoubleFillets` follows `+3` and `+4`, and the numbers of `chr01` fit: part 5 has parent 4, child 6, next sibling 13, no previous; part 13 has parent 4, child 14, next 21, previous 5).  For a part that blends joints (kinds 1 to 3) `+2` and `+3` are the numbers of the two parts whose joints are blended (`partGfx` loads the matrices of those two parts' joints and then the part's own, `+1`, when it is not negative: three matrices for the vector unit) |
| `+20` to `+40` | the strip table (+20), positions (+24, 12 bytes a vertex), texture coordinates (+28, 16 bytes), colours (+32, 4 bytes), the *bind* array (+36, 4 bytes a vertex, only for the parts that blend joints) and the normals (+40, 12 bytes a vertex, for chunk kinds 3 and 4) (read from `partPrecalcDmaData`, which sends them to the vector unit in that order) |
| chunk kind | 3 for a rigid part (positions and normals), 4 for a part that joins joints |

The vectors at `+72` are what the code of a gun reads for the places on a model (the bones the muzzle flash and sparks come from, `gun + 340`, `+360` and `+364` of the table of weapons).

### The bind array

For a part that blends joints the bind array has four bytes for each vertex, of which the first is the joint the vertex follows and the others are 0.  In `chr01` part 1 (blending the joints of parts 0 and 2, which are joints 0 and 1) binds its vertices to 0 or 1, and part 3 (parts 2 and 4: joints 1 and 2) to 1 or 2.  So a vertex of a seam
is rigid on one of the two joints, and a seam changes shape because its two halves move with their joints (a vertex does not carry a weight between them: nothing in the data is a fraction).  What the third matrix is for (the part's own joint, when `+1` is not negative) is not known.

### Drawn

`python tools/obdump.py <OB.PAK> ob/chrs/chr01.raw --obj out.obj` writes the triangles of a model (the strips, as the vector unit draws them) as an OBJ.  The vertices of a character are in the model's own space in a rest pose: `chr01` comes out as a figure 1.95 high and 0.65 wide with the head at the top, the arms hanging
at the sides and the feet on the ground (1598 triangles, 4794 vertices), as a plot of the positions shows.  So a converter can write the rest pose of a mesh directly; what remains is the joints (their rest positions and which vertices go with which joint) and the animations.

### Writing a model

`python tools/obedit.py <OB.PAK> <name> <mods folder>/<mod> scale <factor>` makes a mod that holds a changed copy of a model (every position multiplied).  Run with that mod, the game builds `OB.PAK`, loads the changed `chr01.raw` (the file log shows it in Village) and runs 600 frames without a fault, so a model file made by a tool is
taken by the loader as long as it keeps the layout above.  This was not looked at on the screen (a headless screenshot of the level showed no character in view), so how it looks is not checked: the joints of the animations do not scale with the mesh, so a scaled character would be out of joint.

## Animations

The animation record the game holds for each number (`pAnimations`) has the number of key frames at +4, flags at +12 (bit 1: it loops), the end time in seconds at +16 and a table of the key frames' times in 60ths of a second at +20 (`getKeyFrames` finds the two frames round a time).  `calMatrices` moves the time of a character's two animations (the second blends in while it
changes from the first), blends the position and rotation of every bone between the two key frames of each, turns the heading with the root, adds the hit reaction, makes the global matrices (and the fillets), tilts the head and body of characters that look about, applies the size of the cheats and keeps the
movement of the root.  The layout of the files is in the next section.

### The animation file

Read from `loadAnimation` and checked with `python tools/anidump.py <ANIM.PAK> <name>` on `anim/data/rf_m0.raw` (every quaternion comes out of length 1, and the data ends exactly at the end of the file).  All offsets are from the start of the file; the file is laid out as the memory the loader sets up, which starts at an aligned address.

| At | What |
|---|---|
| +0 | the number of frames (60 a second; the length in seconds is `(frames - 1) / 60`) |
| +4 | the number of key frames |
| +8 | the number of bones the animation drives |
| +12 | flags: bit 1 loops |
| +28 | the number of channels (one for each bone) |
| +48 | the key frames' times, a 32 bit number each, in 60ths of a second (0, 4, 8, ... in `rf_m0`) |
| `(keys * 4 + 63) & -16` | the channels, 32 bytes each: `+0` the bone, `+4` the kind, `+8` the frame count as a float, `+12` the number of key frames |
| right after the channels | the channels' data, each part starting on a 16 byte boundary |

A pose is seven floats: a position (x, y, z) then a rotation as a quaternion (x, y, z, w).  The data of a channel by its kind (the sizes are those of `loadAnimation`; the reading is that of `extractAnimData`):

| Kind | The data | Bytes |
|---|---|---|
| 0 | one pose, for every frame | 28 |
| 2 | one position (12 bytes) for every frame, then a rotation of 16 bytes for each key frame | `keys * 16 + 12` |
| 4 | one pose for the first frame, then a position of 12 bytes for each later key frame; the rotation of the first pose is used for all | `(keys - 1) * 12 + 28` |
| 8 | a whole pose for each key frame | `keys * 28` |

The root's channel is kind 8 (it moves, `rf_m0` has it at a height of about 1.05); the other bones are kind 2: a fixed offset from their parent along the bone and a rotation for each key frame.  The game blends the two key frames round the time (`quaternionSlerp` for the rotations, a straight blend for the positions).

## What is not known yet

* What the third matrix of a seam is for.
* The skeleton's rest pose (the character's own file has no joint positions that were found: they come from the animations, whose kind 2 channels give each bone's offset from its parent).  Which animation file is which (a character plays an animation by its number in a table, and what the numbers are matched to is not read).
* How a gun is attached to a character (its model is a prop with parts that carry the muzzle points).

The plan for characters (`docs/mod-roadmap.md`) starts here: read those three, make `obdump.py` print a model's triangles and joints and check it by drawing the result, and only then write a
converter that makes the game's format from a skinned mesh.
