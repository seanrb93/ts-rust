"""Draw the button icons for the controllers and the keyboard, in the style of the game's own PlayStation buttons (dark bevelled discs and pills with a thin coloured symbol).

    python tools/make-button-icons.py <output folder>

Writes, for each size (`24/` is the size of the game's own buttons, `96/` four times that for the HD pictures):
    xbox/    a b x y  lb rb lt rt  menu view  ls rs  dpad
    switch/  a b x y  l r zl zr  plus minus  ls rs  dpad
    keys/    one cap for each key of the list KEYS (letters, digits, F1 to F12, space, enter, shift, ctrl, alt, tab, esc, backspace, the arrows), and the mouse
    preview.png   all of them on the menu's colour
The pictures are drawn from nothing (the letters with the system's Segoe UI Semibold or Arial when it is there, else DejaVu or Pillow's own font), at 8 times the size and reduced, so the edges are smooth.
"""
import os
import sys

from PIL import Image, ImageDraw, ImageFont

SS = 8  # the drawing is made this many times larger and reduced

# the colours of the game's buttons, read from its own pictures (the rim's light, the body, the outline)
OUTLINE = (6, 7, 10)
BODY = (40, 45, 52)
BODY_DARK = (35, 39, 46)
RIM_LIGHT = (112, 122, 136)
RIM_MID = (62, 68, 78)
TEXT = (176, 182, 190)

GREEN, RED, BLUE, YELLOW = (130, 214, 120), (240, 104, 104), (128, 184, 246), (246, 206, 84)
CREAM = (226, 228, 234)


def font(size):
    for name in ('segoeuisb.ttf', 'segoeui.ttf', 'arial.ttf', 'DejaVuSans.ttf'):
        for folder in (os.path.join(os.environ.get('WINDIR', 'C:/Windows'), 'Fonts'), '/usr/share/fonts/truetype/dejavu'):
            path = os.path.join(folder, name)
            if os.path.isfile(path):
                return ImageFont.truetype(path, size)
    return ImageFont.load_default()


def canvas(w, h):
    return Image.new('RGBA', (w * SS, h * SS), (0, 0, 0, 0))


def finish(img, w, h, scale):
    return img.resize((w * scale, h * scale), Image.LANCZOS)


def plate(d, box, radius, w, h):
    """The dark bevelled plate of a button: outline, a light rim along the top left, the body (a rounded rectangle; a disc when it is as wide as it is high)."""
    x0, y0, x1, y1 = [v * SS for v in box]
    d.rounded_rectangle([x0, y0, x1, y1], radius=radius * SS, fill=OUTLINE)
    i = SS
    d.rounded_rectangle([x0 + i, y0 + i, x1 - i, y1 - i], radius=max(radius - 1, 1) * SS, fill=RIM_MID)
    # (the light along the top, fading to the sides)
    d.rounded_rectangle([x0 + i, y0 + i, x1 - i, y1 - i], radius=max(radius - 1, 1) * SS, outline=RIM_LIGHT, width=i)
    d.rounded_rectangle([x0 + i, y0 + int(1.6 * i), x1 - i, y1 - i], radius=max(radius - 1, 1) * SS, fill=RIM_MID)
    j = int(2.1 * SS)
    d.rounded_rectangle([x0 + j, y0 + j, x1 - j, y1 - j], radius=max(radius - 2, 1) * SS, fill=BODY)
    # (a little darker towards the bottom)
    d.rounded_rectangle([x0 + j, y0 + (y1 - y0) // 2, x1 - j, y1 - j], radius=max(radius - 2, 1) * SS, fill=BODY_DARK)
    d.rounded_rectangle([x0 + j, y0 + j, x1 - j, y0 + (y1 - y0) // 2 + 2 * SS], radius=max(radius - 2, 1) * SS, fill=BODY)


def letter(d, text, cx, cy, size, colour):
    f = font(size * SS)
    d.text((cx * SS, cy * SS), text, font=f, fill=colour, anchor='mm')


def disc_button(label, colour, size=24):
    img = canvas(size, size)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, size - 1, size - 1), size // 2, size, size)
    letter(d, label, size / 2, size / 2 + 0.3, size * 0.66, colour)
    return img


def pill(label, size=24, wide=None, h=None, colour=TEXT, fs=0.42):
    w = wide or int(size * 1.6)
    h = h or int(size * 0.72)
    img = canvas(w, h)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, w - 1, h - 1), h // 2 - 1, w, h)
    letter(d, label, w / 2, h / 2 + 0.2, size * 0.5, colour)
    return img


def small_disc(draw_symbol, size=24, colour=TEXT):
    img = canvas(size, size)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, size - 1, size - 1), size // 2, size, size)
    draw_symbol(d, size / 2 * SS, size / 2 * SS, size * SS, colour)
    return img


def sym_menu(d, cx, cy, s, c):
    for k in (-1, 0, 1):
        y = cy + k * s * 0.15
        d.rounded_rectangle([cx - s * 0.22, y - s * 0.03, cx + s * 0.22, y + s * 0.03], radius=s * 0.03, fill=c)


def sym_view(d, cx, cy, s, c):
    w = s * 0.045
    d.rectangle([cx - s * 0.2, cy - s * 0.12, cx + s * 0.08, cy + s * 0.16], outline=c, width=int(w))
    d.rectangle([cx - s * 0.08, cy - s * 0.2, cx + s * 0.2, cy + s * 0.08], outline=c, width=int(w), fill=BODY)


def sym_plus(d, cx, cy, s, c):
    w = s * 0.06
    d.rectangle([cx - s * 0.22, cy - w, cx + s * 0.22, cy + w], fill=c)
    d.rectangle([cx - w, cy - s * 0.22, cx + w, cy + s * 0.22], fill=c)


def sym_minus(d, cx, cy, s, c):
    w = s * 0.06
    d.rectangle([cx - s * 0.22, cy - w, cx + s * 0.22, cy + w], fill=c)


def sym_dpad(d, cx, cy, s, c):
    a, b = s * 0.11, s * 0.28
    d.polygon([(cx - a, cy - b), (cx + a, cy - b), (cx + a, cy - a), (cx + b, cy - a), (cx + b, cy + a), (cx + a, cy + a), (cx + a, cy + b), (cx - a, cy + b), (cx - a, cy + a), (cx - b, cy + a), (cx - b, cy - a), (cx - a, cy - a)], fill=c)


def stick(label, size=24):
    img = canvas(size, size)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, size - 1, size - 1), size // 2, size, size)
    c = size / 2 * SS
    r = size * 0.3 * SS
    d.ellipse([c - r, c - r, c + r, c + r], outline=TEXT, width=int(size * 0.07 * SS))
    letter(d, label, size / 2, size / 2 + 0.2, size * 0.4, TEXT)
    return img


def keycap(label, size=24):
    f = font(int(size * 0.5 * SS))
    tmp = ImageDraw.Draw(Image.new('RGBA', (8, 8)))
    tw = tmp.textlength(label, font=f) / SS
    w = max(size, int(tw + size * 0.55))
    img = canvas(w, size)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, w - 1, size - 1), max(size // 5, 3), w, size)
    d.text((w / 2 * SS, (size / 2 + 0.2) * SS), label, font=f, fill=TEXT, anchor='mm')
    return img


def arrow_key(direction, size=24):
    img = canvas(size, size)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, size - 1, size - 1), max(size // 5, 3), size, size)
    c = size / 2 * SS
    a = size * 0.22 * SS
    pts = {'up': [(c, c - a), (c + a, c + a * 0.7), (c - a, c + a * 0.7)], 'down': [(c, c + a), (c + a, c - a * 0.7), (c - a, c - a * 0.7)],
           'left': [(c - a, c), (c + a * 0.7, c - a), (c + a * 0.7, c + a)], 'right': [(c + a, c), (c - a * 0.7, c - a), (c - a * 0.7, c + a)]}[direction]
    d.polygon(pts, fill=TEXT)
    return img


def mouse(which, size=24):
    """A mouse seen from above (the egg of its body, the line between the buttons) with the button that is meant lit (left, right, middle)."""
    w = int(size * 0.78)
    img = canvas(w, size)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, w - 1, size - 1), w // 2 - 1, w, size)
    cx = w / 2 * SS
    x0, x1 = w * 0.26 * SS, w * 0.74 * SS
    y0, y1 = size * 0.16 * SS, size * 0.84 * SS
    mid = size * 0.46 * SS
    lw = max(int(size * 0.045 * SS), SS)
    pad = lw
    lit = {'left': [x0 + pad, y0 + pad, cx, mid], 'right': [cx, y0 + pad, x1 - pad, mid], 'middle': [cx - w * 0.06 * SS, y0 + size * 0.05 * SS, cx + w * 0.06 * SS, mid - size * 0.03 * SS]}[which]
    d.rectangle(lit, fill=BLUE)
    d.rounded_rectangle([x0, y0, x1, y1], radius=(x1 - x0) / 2, outline=TEXT, width=lw)
    d.line([(x0, mid), (x1, mid)], fill=TEXT, width=lw)
    d.line([(cx, y0), (cx, mid)], fill=TEXT, width=lw)
    return img


KEYS = (list('ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789') + ['F%d' % i for i in range(1, 13)]
        + ['Space', 'Enter', 'Shift', 'Ctrl', 'Alt', 'Tab', 'Esc', 'Back', 'Ins', 'Del', 'Home', 'End', 'PgUp', 'PgDn', ',', '.', '/', ';', "'", '[', ']', '\\', '-', '='])
NAMES = {'/': 'slash', '\\': 'backslash', ',': 'comma', '.': 'period', ';': 'semicolon', "'": 'quote', '[': 'lbracket', ']': 'rbracket', '-': 'minus', '=': 'equals'}


def build(scale, out):
    """All the icons at `scale` times the game's 24 pixels."""
    sets = {}
    sets['xbox'] = {
        'a': disc_button('A', GREEN), 'b': disc_button('B', RED), 'x': disc_button('X', BLUE), 'y': disc_button('Y', YELLOW),
        'lb': pill('LB'), 'rb': pill('RB'), 'lt': pill('LT'), 'rt': pill('RT'),
        'menu': small_disc(sym_menu), 'view': small_disc(sym_view), 'ls': stick('L'), 'rs': stick('R'), 'dpad': small_disc(sym_dpad),
    }
    sets['switch'] = {
        'a': disc_button('A', CREAM), 'b': disc_button('B', CREAM), 'x': disc_button('X', CREAM), 'y': disc_button('Y', CREAM),
        'l': pill('L'), 'r': pill('R'), 'zl': pill('ZL'), 'zr': pill('ZR'),
        'plus': small_disc(sym_plus), 'minus': small_disc(sym_minus), 'ls': stick('L'), 'rs': stick('R'), 'dpad': small_disc(sym_dpad),
    }
    keys = {NAMES.get(k, k.lower()): keycap(k) for k in KEYS}
    keys.update({'up': arrow_key('up'), 'down': arrow_key('down'), 'left': arrow_key('left'), 'right': arrow_key('right'),
                 'mouse_left': mouse('left'), 'mouse_right': mouse('right'), 'mouse_middle': mouse('middle')})
    sets['keys'] = keys
    for group, icons in sets.items():
        os.makedirs(os.path.join(out, group), exist_ok=True)
        for name, img in icons.items():
            w, h = img.width // SS, img.height // SS
            finish(img, w, h, scale).save(os.path.join(out, group, name + '.png'))
    return sets


def preview(sets, path):
    bg = (74, 72, 120, 255)
    zoom = 3
    rows = []
    for group, icons in sets.items():
        rows.append((group, [finish(i, i.width // SS, i.height // SS, zoom) for i in icons.values()]))
    width = 1500
    sheet = Image.new('RGBA', (width, 2400), bg)
    d = ImageDraw.Draw(sheet)
    y = 10
    for group, imgs in rows:
        d.text((10, y), group, font=font(18), fill=(255, 255, 255, 255))
        y += 28
        x = 10
        rowh = max(i.height for i in imgs)
        for i in imgs:
            if x + i.width > width - 10:
                x = 10
                y += rowh + 10
            sheet.alpha_composite(i, (x, y))
            x += i.width + 10
        y += rowh + 30
    sheet.crop((0, 0, width, y)).convert('RGB').save(path)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    out = sys.argv[1]
    for scale in (1, 4):
        sets = build(scale, os.path.join(out, str(24 * scale)))
    preview(sets, os.path.join(out, 'preview.png'))
    print('icons written to', out, '(' + ', '.join(f'{k}: {len(v)}' for k, v in sets.items()) + ')')


if __name__ == '__main__':
    main()
