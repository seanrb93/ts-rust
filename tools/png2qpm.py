"""Make a game texture (.qpm) from a PNG, for a texture mod.

    python tools/png2qpm.py picture.png out.qpm [--check]

The picture is reduced to 256 colours (with alpha) and written the way the game's own textures are: a text header (`Q8`, `width height`,
a line of numbers padded so that the header is a multiple of 4 bytes long), the palette (256 entries of red, green, blue, alpha; alpha 127 =
opaque, 0 = clear) and one palette index per pixel.

Limits of the game's loader (it reads a whole file into a 66688 byte buffer and decodes it into the 66432 bytes behind):
the file has to fit in 66688 bytes, so width * height can be at most 65536 pixels (256x256, 512x128, 128x512, ...).  Make each side a power of two
(64, 128, 256, ...): a side that is not one does not display properly in the game (a model's surface comes out washed out).  The size need not be the original's: on a
model the picture is stretched over the same surface (a 128x128 picture on a 64x64 texture is four times as sharp), while a picture the game draws as a
sprite (the HUD's ammo icon, the menus) is drawn at its own size.

The replacement goes into a mod as  <mod folder>/MISC.PAK/textures/<name>.qpm  with the name of the texture it replaces (see
tools/extract_textures.py: its folders and index.csv name every texture).
"""
import sys

from PIL import Image

MAX_FILE = 66688


def make_qpm(img):
    img = img.convert('RGBA')
    w, h = img.size
    q = img.quantize(colors=256, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE)
    pal = q.getpalette(rawmode='RGBA')
    if pal is None or len(pal) < 1024:
        pal = list(q.convert('RGBA').getpalette() or [])
    # the alpha of each palette entry: the (mean) alpha of the pixels that use it
    rgba = img.getdata()
    idx = list(q.getdata())
    sums = [0] * 256
    count = [0] * 256
    for p, i in zip(rgba, idx):
        sums[i] += p[3]
        count[i] += 1
    entries = bytearray()
    for i in range(256):
        r, g, b = pal[i * 4], pal[i * 4 + 1], pal[i * 4 + 2]
        a = round(sums[i] / count[i]) if count[i] else 255
        entries += bytes((r, g, b, min(127, a * 128 // 255)))
    third = '255 v 256'
    head = 'Q8\n%d %d\n' % (w, h)
    while (len(head) + len(third) + 1) % 4:
        third += ' '
    head += third + '\n'
    return head.encode('latin1') + bytes(entries) + bytes(idx)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    if len(args) != 2:
        print(__doc__)
        sys.exit(1)
    img = Image.open(args[0])
    w, h = img.size
    data = make_qpm(img)
    if len(data) > MAX_FILE:
        print(f'warning: {len(data)} bytes is more than the {MAX_FILE} the game can load ({w}x{h} pixels; at most 65536)')
    if (w & (w - 1)) or (h & (h - 1)):
        print(f'note: {w}x{h} has a side that is not a power of two; the game does not display that properly')
    open(args[1], 'wb').write(data)
    print(f'{args[1]}: {w}x{h}, {len(data)} bytes')


if __name__ == '__main__':
    main()
