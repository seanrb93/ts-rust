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
taken by the loader as long as it keeps the layout above.  `TS_CAMMODE=2:300` (a test aid: the camera goes to the third person view at frame 300) with `--screenshot` shows the player's own character close up.  The head of a character in a level is part 22 of `chr01.raw` (486 triangles, joint 15); `chr01_head.raw` is a different file that a level run does not load (the file log shows only `chr01.raw`; an earlier note here that a scaled `chr01_head.raw` showed on the screen was wrong: the bigger face came from the scaled body file).  The joints of the animations do not scale with the mesh, so a scaled body would be out of joint.

`python tools/obedit.py <OB.PAK> ob/chrs/chr01.raw <mod> box 22 0.8` replaces the geometry of part 22 by a box made from nothing: the strips, positions, coordinates, colours and normals are written by the tool into the arrays the part has, and its chunk list becomes one chunk of 6 strips and 24 vertices.  In the game the head is a box, with the head's texture on it (its coordinates are those of a box, so the picture is scrambled), which shows that strips, positions, coordinates, colours and normals written from nothing are drawn.

`python tools/obedit.py <OB.PAK> ob/chrs/chr01.raw <mod> meshes <model.obj>` is the whole model at once: `python tools/obdump.py <OB.PAK> ob/chrs/chr01.raw --obj out.obj` writes every part as an object `part<N>` (materials `slot<K>_k<kind>` for the texture slot and chunk kind, the vertex colours after the position, the normals, the coordinates), and `meshes` reads such a file and replaces each part by its object (a part that blends joints keeps the bind of the nearest vertex of the original).  Put back unchanged the model draws as the original (checked on the head, close up, in the third person view); edit the OBJ in a modelling tool to change it.  The strips of the original that carry the flag `0x10000` start with the other winding, which the export takes into account.

**The size of a chunk depends on its kind.**  The first round trip of the head came out in pieces with holes, and a chunk limit was the cause: the 64 vertices of the first notes hold only for chunks of kind 0 and 2.  Over all the files of `OB.PAK` the most a chunk has is: kind 0 and 2: 64 vertices and 209 for `3 * vertices + strips`; kind 3 (normals as well): 48 vertices and 160; kind 4 (normals and binds): 36 vertices and 117.  The microcode of each kind has less room for the data of a vertex.  `obedit.py` cuts the chunks of a part by its kind (`CHUNK_LIMITS`: 64/212, 48/160 and 33/117); a part that does not stay in these limits breaks its drawing as the head did.  (A byte for byte copy of the part's arrays, moved after the file's other data, drew perfectly, and so did the strips written again with the original chunks, so the layout of a file that grew is right.)

`python tools/obedit.py <OB.PAK> ob/chrs/chr01.raw <mod> mesh 22 <mesh.obj>` goes further for one part: it replaces a part's geometry by the faces of an OBJ (triangles and quads; quads are written as the strip a, b, d, c), packed into chunks within the vector unit's limits by kind (see above), with the texture of the part's largest chunk, flat normals when the OBJ has none, and the first vertex of each chunk a multiple of 4.  There is no limit on the size of the mesh: the new arrays and chunk list are written after the file's other data, the bones and the object record move behind them, and the pointers of the part and the offset of the object (word 1 of the header) are changed (the old arrays stay in the file, unused).  Only the chunk limits remain, because they are the vector unit's own.  A sphere of 96 quads put in place of the head is drawn as a rounded dark shape (the texture is the hair's); a sphere of 3072 quads (64 by 48: 192 chunks, the file grows from 128 KB to 723 KB, far past the head's own room of 127 strips) is drawn too, as a smooth big ball in the third person screenshot.  Textures have no limit either: the OBJ's materials (`usemtl`, with `map_Kd` pictures in the MTL) each become a texture of the game, numbered from `--tex-base` (9000 by default; a level of a mod numbers its textures from 4000 up, and the game has 16384 numbers with `extended-tables`), added to the model's table of textures after the ones it has (the file is rewritten with the new table, and word 0 of its header points at it); the faces of a part are grouped by material and each chunk has one texture.  Each picture goes into the mod as `MISC.PAK/textures/<number>.qpm` (a small one, at most 256 by 256, which is what the game's loader takes) and `<number>.png` of the picture's own size, which the renderer shows in its place at any size (the mod loader accepts a PNG with a `.qpm` of the same name beside it as a texture of the mod's own).  A test sphere with a 1024 by 1024 picture and a 2048 by 2048 picture on it was drawn with sharp edges (the log shows both read).

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

## The skeleton and the animation numbers

`animMake` loads two skeleton files into the skeleton table, `anim/data/male_skeleton.raw` (the skeleton of most characters) and `anim/data/newanims/fskel.raw` (the female one), and 606 animation files by number.  `python tools/animtable.py <game folder>` lists the numbers and files from the tables of
`animation-loading.rs` and the names in the program file (for instance 3 is `spb_m0`, 4 `rf_m0`, 5 `rb_m0`, 6 `rr_m0`, 7 `rl_m0`; the female animations are in `fem/`; the last, 611 to 613, are `jumpleft` ones).  The game asks for the animation `n` and shifts it by the character's variant for the
classes of 11 (animations 3 to 442 are 20 classes of 11 for each of two kinds, as `animMake` sets their properties).

**A skeleton file is an animation of one frame**: the rest pose, with the layout above (`anidump.py` reads it: 18 channels of kind 0, each a position and a quaternion).  The channel `k` drives the `k`-th part of kind 0 of a character's model (`chr01`: parts 0, 2, 4, ... 34 in order), and the position of a channel is the offset from its parent bone
(the nearest kind 0 part up the links).  Put together from the root (a height of 1.085) down, the joints of the male skeleton come out as a person: hips (±0.102, 0.993), knees (0.593), ankles (0.138), shoulders (±0.23, 1.475), elbows (1.19), wrists (0.913), and the joint of the head part at 1.566, below
the centre of the head mesh (1.815).  These agree with the figure the vertices make, so the rest pose is known: the mesh of a character is in the model's space with this pose, and the joints are where the skeleton file puts them.  How a pose matrix is made from them for the vector unit (the world transform of the pose times the inverse of the
rest one) has not been read.

## What is not known yet

* What the third matrix of a seam is for.
* How the pose matrix of a joint is made from the pose and the rest pose, and what the numbers of the animations mean (their file names such as `rf_m0` are not decoded).
* How a gun is attached to a character (its model is a prop with parts that carry the muzzle points).

The plan for characters (`docs/mod-roadmap.md`) starts here: read those three, make `obdump.py` print a model's triangles and joints and check it by drawing the result, and only then write a
converter that makes the game's format from a skinned mesh.
