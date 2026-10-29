Texture sample mod: high resolution pictures drawn from nothing, in place of the game's own.

    python examples/texture_sample_mod_src/make-textures.py mods/texture_sample_mod/MISC.PAK/textures [size]
    (and a mod.ini with: enabled = true)

The pictures are not kept in the repository (28 MB at 2048): the script draws them.  What it replaces:
  misc/back1, misc/back4, misc/clouds   the cloud pictures behind the menus (256 x 256 in the game), a 2048 x 2048 teal and magenta nebula
  1181, 1183                            the floor tiles of the Prison level (64 x 64), 2048 x 2048 plates with a glowing border; 1183 has the red mark
  1180, 1182                            the walls of the Prison (128 x 64), 2048 x 1024 brushed metal with panel seams; 1182 has the red line
A PNG of any size (up to hd_texture_max in ts-rust.ini, 4096 by default) goes in MISC.PAK/textures/ under the name of the texture it replaces (see docs/reference.md).
Look at it with: ts-rust.exe --level Prison --character 5 --spawn 12,0,-26 --cam 3,1.6
