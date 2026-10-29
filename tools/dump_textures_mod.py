"""Make a mod that holds the game's level textures and its map maker tile sets as PNG files, sorted into folders, to edit (the pictures are the game's own: the mod is written next
to the game, not into the repository).

    python tools/dump_textures_mod.py <extracted textures folder> <mods folder> [mod name]

First extract the textures: `python tools/extract_textures.py <PAK folder> <folder>`.  The mod (switched off, `enabled = false`) gets
    MISC.PAK/textures/<Level>/<number>.png           a folder with the name of each level (Prison, Village, Tomb ...; the numbered textures stored under the level)
    MISC.PAK/textures/Tilesets/Tileset<N>/<number>.png   the textures of the five tile sets of the map maker (a texture two sets share is in the lower one)
    _index/textures.csv                              each texture's size, where it is, and every level and model that uses it (a folder starting with _ is not loaded)
Only the number of a file counts to the game, so a file may be moved to another folder; see "Textures" in docs/reference.md.
"""
import csv
import os
import re
import shutil
import sys


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)
    src, mods = sys.argv[1], sys.argv[2]
    name = sys.argv[3] if len(sys.argv) > 3 else 'level_textures'
    root = os.path.join(mods, name)
    shutil.rmtree(root, ignore_errors=True)
    out = os.path.join(root, 'MISC.PAK', 'textures')
    os.makedirs(out)
    os.makedirs(os.path.join(root, '_index'))
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import extract_textures
    names = extract_textures.LEVEL_NAMES

    def level_name(folder):
        m = re.match(r'level(\d+)', os.path.basename(folder))
        n = int(m.group(1)) if m else 0
        return names.get(n, 'Level%02d' % n)

    rows = list(csv.DictReader(open(os.path.join(src, 'index.csv'), newline='')))
    index = []
    levels = tilesets = 0
    for r in rows:
        folder = r['folder'].replace('\\', '/')
        number = os.path.basename(r['texture']).replace('.qpm', '')
        png = os.path.join(src, folder, number + '.png')
        if not number.isdigit() or not os.path.isfile(png):
            continue
        users = r['used by'].replace('\\', '/')
        sets = sorted({int(m) for m in re.findall(r'objects/tile(\d+)_', users)})
        if folder.startswith('levels/'):
            where = level_name(folder)
            levels += 1
        elif sets:
            where = 'Tilesets/Tileset%d' % sets[0]
            tilesets += 1
        else:
            continue
        os.makedirs(os.path.join(out, where), exist_ok=True)
        shutil.copy(png, os.path.join(out, where, number + '.png'))
        # (the names of the levels in the list of users)
        nice = re.sub(r'levels/level(\d+)(_\w+)?', lambda m: 'levels/' + names.get(int(m.group(1)), 'Level' + m.group(1)), users)
        index.append([number + '.png', r['width'], r['height'], where, nice])
    with open(os.path.join(root, '_index', 'textures.csv'), 'w', newline='') as f:
        w = csv.writer(f)
        w.writerow(['texture', 'width', 'height', 'folder', 'used by'])
        w.writerows(index)
    with open(os.path.join(root, 'mod.ini'), 'w') as f:
        f.write('enabled = false\n')
    with open(os.path.join(root, 'README.txt'), 'w') as f:
        f.write(
            "The textures of the game's levels and of the map maker's tile sets as PNG files, at their own size, to edit.\n\n"
            f"MISC.PAK/textures/<Level>/<number>.png               {levels} textures, in a folder named after the level they are stored under.\n"
            f"MISC.PAK/textures/Tilesets/Tileset<N>/<number>.png   {tilesets} textures of the five tile sets of the map maker (the game gives them no names).\n"
            "_index/textures.csv                                  the size of each, where it is, and every level or model that uses it.  (A folder starting with _ is not loaded.)\n\n"
            "Only the number of a file counts: a file can be moved to another folder.  A picture may be made larger (up to hd_texture_max in ts-rust.ini, 4096 by default); keep its proportions.\n"
            "A texture used by several levels is stored once, under the first of them, and changes everywhere it is used.\n"
            "The mod is switched off: edit the files you want, delete the ones you do not change (each file adds to the loading time), then switch it on (the Mods page, or enabled = true in mod.ini).\n"
            "The pictures are the game's own: keep them to yourself.\n")
    print(f'{levels} level textures and {tilesets} tile set textures written to {root}')


if __name__ == '__main__':
    main()
