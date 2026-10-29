"""Draw the button icons for the controllers and the keyboard, in the style of the game's own PlayStation buttons (dark bevelled discs and pills with a thin coloured symbol).

    python tools/make-button-icons.py <output folder>

Writes, for each size (`24/` is the size of the game's own buttons, `96/` four times that for the HD pictures):
    xbox/    a b x y  lb rb lt rt  menu view  ls rs  dpad
    switch/  a b x y  l r zl zr  plus minus  ls rs  dpad
    keys/    one cap for each key of the list KEYS (letters, digits, F1 to F12, space, enter, shift, ctrl, alt, tab, esc, backspace, the arrows), and the mouse
    keys_sq/ a square cap (24 x 24) for each key, with a short label or a symbol (Enter, Space, Shift, Back ...), for the places that have room for one square picture
    preview.png   all of them on the menu's colour
The pictures are drawn from nothing (the letters with the system's Segoe UI Semibold or Arial when it is there, else DejaVu or Pillow's own font), at 8 times the size and reduced, so the edges are smooth.
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

SS = 8  # the drawing is made this many times larger and reduced

# the colours of the game's buttons, read from its own pictures (the rim's light, the body, the outline)
OUTLINE = (8, 9, 13)
BODY = (45, 50, 58)
BODY_DARK = (38, 42, 50)
RIM_LIGHT = (110, 120, 134)
RIM_MID = (62, 68, 78)
TEXT = (176, 182, 190)

GREEN, RED, BLUE, YELLOW = (130, 214, 120), (240, 104, 104), (128, 184, 246), (246, 206, 84)
CREAM = (226, 228, 234)


def font(size, bold=False):
    for name in (('segoeuib.ttf', 'arialbd.ttf', 'DejaVuSans-Bold.ttf') if bold else ('segoeuisb.ttf', 'segoeui.ttf', 'arial.ttf', 'DejaVuSans.ttf')):
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
    if x1 - x0 == y1 - y0 and radius * SS * 2 >= x1 - x0:
        # (the game's round buttons have a thin light arc along the top of the rim and nothing else)
        k = int(1.2 * SS)
        d.arc([x0 + k, y0 + k, x1 - k, y1 - k], start=212, end=328, fill=RIM_LIGHT, width=int(1.1 * SS))
    else:
        # (a cap with corners: the light is a short straight line along the top edge)
        r = radius * SS
        d.line([(x0 + r, y0 + int(1.4 * SS)), (x1 - r, y0 + int(1.4 * SS))], fill=RIM_LIGHT, width=int(1.1 * SS))


def centred(d, text, cx, cy, f, colour):
    """The text with the middle of its ink at `cx` and the middle of the height of a capital at `cy` (both in pixels of the drawing), so that the letters of a button are centred and the
    labels of the keys ('Space', 'Esc') sit on the same line whatever letters hang below it."""
    ink = d.textbbox((0, 0), text, font=f, anchor='la')
    cap = d.textbbox((0, 0), 'H', font=f, anchor='la')
    x = cx - (ink[0] + ink[2]) / 2
    y = cy - (cap[1] + cap[3]) / 2
    d.text((x, y), text, font=f, fill=colour, anchor='la')


def smooth(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


def round_plate(size):
    """The disc of a round button as the game draws its own, measured from `mm_button01` to `04` (the median of the four has no symbol): a soft edge that fades over the last pixels,
    a near-black band inside it, a grey rim at 9.6 of the radius of 12 (bright at the top, only 45 grey at the sides and the bottom), and the body of (45, 50, 58) that is a little darker
    in its lower half.  Returns the picture at SS times the size and the ImageDraw to put the symbol on."""
    n = size * SS
    yy, xx = np.mgrid[0:n, 0:n].astype(float)
    c = (n - 1) / 2
    unit = n / 24.0
    dx, dy = xx - c, yy - c
    ru = np.hypot(dx, dy) / unit
    ang = np.arctan2(dy, dx)
    lit = np.clip(-np.sin(ang), 0, 1) ** 3
    side, top = np.array([62.0, 66.0, 76.0]), np.array([118.0, 125.0, 136.0])
    rim = side + (top - side) * lit[..., None]
    below = smooth(-1.0, 4.0, dy / unit)[..., None]
    body = np.array(BODY, float) * (1 - below) + np.array(BODY_DARK, float) * below
    rim_w = np.exp(-(((ru - 9.6) / 0.8) ** 2))[..., None]
    col = body * (1 - rim_w) + rim * rim_w
    edge_w = smooth(10.2, 10.9, ru)[..., None]
    col = col * (1 - edge_w) + np.array([14.0, 14.0, 17.0]) * edge_w
    alpha = 1 - smooth(10.2, 12.2, ru)
    out = np.dstack([np.clip(col, 0, 255), alpha * 255]).astype(np.uint8)
    return Image.fromarray(out, 'RGBA')


def letter(d, text, cx, cy, size, colour, bold=False):
    centred(d, text, cx * SS, cy * SS, font(size * SS, bold), colour)


def disc_button(label, colour, size=24):
    img = round_plate(size)
    d = ImageDraw.Draw(img)
    letter(d, label, size / 2, size / 2, size * 0.78, colour)
    return img


def pill(label, size=24, wide=None, h=None, colour=TEXT, fs=0.42):
    w = wide or int(size * 1.6)
    h = h or int(size * 0.72)
    img = canvas(w, h)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, w - 1, h - 1), h // 2 - 1, w, h)
    letter(d, label, w / 2, h / 2, size * 0.5, colour)
    return img


def small_disc(draw_symbol, size=24, colour=TEXT):
    img = round_plate(size)
    d = ImageDraw.Draw(img)
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
    img = round_plate(size)
    d = ImageDraw.Draw(img)
    c = size / 2 * SS
    r = size * 0.3 * SS
    d.ellipse([c - r, c - r, c + r, c + r], outline=TEXT, width=int(size * 0.07 * SS))
    letter(d, label, size / 2, size / 2, size * 0.4, TEXT)
    return img


def keycap(label, size=24):
    f = font(int(size * 0.5 * SS))
    tmp = ImageDraw.Draw(Image.new('RGBA', (8, 8)))
    tw = tmp.textlength(label, font=f) / SS
    w = max(size, int(tw + size * 0.55))
    img = canvas(w, size)
    d = ImageDraw.Draw(img)
    plate(d, (0, 0, w - 1, size - 1), max(size // 5, 3), w, size)
    centred(d, label, w / 2 * SS, size / 2 * SS, f, TEXT)
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


def keycap_sq(label, size=24):
    """A square cap: the label as large as fits (a long one smaller), or a symbol for Enter, Space, Shift, Tab and Back.  The cap is smaller than the picture (a margin of 2 pixels
    all round), so that it is not larger to the eye than the round buttons beside it."""
    img = canvas(size, size)
    d = ImageDraw.Draw(img)
    m = 2
    plate(d, (m, m, size - 1 - m, size - 1 - m), 3, size, size)
    c = size / 2 * SS
    u = size * SS * 0.84
    w = int(size * SS * 0.07)
    key = label.lower()
    if key == 'enter':
        # the return arrow: down, then left with a head
        d.line([(c + u * 0.2, c - u * 0.22), (c + u * 0.2, c + u * 0.08), (c - u * 0.18, c + u * 0.08)], fill=TEXT, width=w, joint='curve')
        d.polygon([(c - u * 0.28, c + u * 0.08), (c - u * 0.14, c - u * 0.04), (c - u * 0.14, c + u * 0.2)], fill=TEXT)
    elif key == 'space':
        d.line([(c - u * 0.24, c - u * 0.06), (c - u * 0.24, c + u * 0.1), (c + u * 0.24, c + u * 0.1), (c + u * 0.24, c - u * 0.06)], fill=TEXT, width=w, joint='curve')
    elif key == 'shift':
        d.polygon([(c, c - u * 0.26), (c + u * 0.24, c + u * 0.02), (c + u * 0.11, c + u * 0.02), (c + u * 0.11, c + u * 0.24), (c - u * 0.11, c + u * 0.24), (c - u * 0.11, c + u * 0.02), (c - u * 0.24, c + u * 0.02)], outline=TEXT, width=w)
    elif key == 'back':
        d.polygon([(c - u * 0.28, c), (c - u * 0.08, c - u * 0.18), (c + u * 0.26, c - u * 0.18), (c + u * 0.26, c + u * 0.18), (c - u * 0.08, c + u * 0.18)], outline=TEXT, width=w)
        d.line([(c - u * 0.02, c - u * 0.07), (c + u * 0.16, c + u * 0.07)], fill=TEXT, width=w)
        d.line([(c - u * 0.02, c + u * 0.07), (c + u * 0.16, c - u * 0.07)], fill=TEXT, width=w)
    elif key == 'tab':
        d.line([(c - u * 0.26, c), (c + u * 0.18, c)], fill=TEXT, width=w)
        d.polygon([(c + u * 0.26, c), (c + u * 0.1, c - u * 0.12), (c + u * 0.1, c + u * 0.12)], fill=TEXT)
        d.line([(c + u * 0.28, c - u * 0.2), (c + u * 0.28, c + u * 0.2)], fill=TEXT, width=w)
    else:
        text = {'ctrl': 'Ctl'}.get(key, label)
        fs = size * 0.84 * (0.66 if len(text) <= 2 else 0.4 if len(text) == 3 else 0.3)
        centred(d, text, c, c, font(int(fs * SS), bold=True), TEXT)
    return img


KEYS = (list('ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789') + ['F%d' % i for i in range(1, 13)]
        + ['Space', 'Enter', 'Shift', 'Ctrl', 'Alt', 'Tab', 'Esc', 'Back', 'Ins', 'Del', 'Home', 'End', 'PgUp', 'PgDn', ',', '.', '/', ';', "'", '[', ']', '\\', '-', '='])
NAMES = {'`': 'grave', '/': 'slash', '\\': 'backslash', ',': 'comma', '.': 'period', ';': 'semicolon', "'": 'quote', '[': 'lbracket', ']': 'rbracket', '-': 'minus', '=': 'equals'}


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
    sets['keys_sq'] = {NAMES.get(k, k.lower()): keycap_sq(k) for k in KEYS + ['`']}
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


def emit_rust(assets, rust):
    """The 96 pixel pictures the port draws (the face buttons of the Xbox and Switch and the square key caps) as a Rust table of `include_bytes!`."""
    entries = []
    for group in ('xbox', 'switch', 'keys_sq'):
        folder = os.path.join(assets, group)
        for name in sorted(os.listdir(folder)):
            if group != 'keys_sq' and name[:-4] not in ('a', 'b', 'x', 'y'):
                continue
            entries.append((group + '/' + name[:-4], '../assets/icons/' + group + '/' + name))
    nl = chr(10)
    with open(rust, 'w', newline='') as f:
        f.write('// Written by tools/make-button-icons.py: the pictures of the buttons and keys (96 pixels), by name.' + nl)
        f.write('pub static ICONS: &[(&str, &[u8])] = &[' + nl)
        for n, p in entries:
            f.write('    ("' + n + '", include_bytes!("' + p + '")),' + nl)
        f.write('];' + nl)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    out = sys.argv[1]
    for scale in (1, 4):
        sets = build(scale, os.path.join(out, str(24 * scale)))
    preview(sets, os.path.join(out, 'preview.png'))
    if len(sys.argv) > 3:
        # (the port's copy: python tools/make-button-icons.py <out> <port assets/icons folder> <port src/icons_data.rs>)
        import shutil
        shutil.rmtree(sys.argv[2], ignore_errors=True)
        shutil.copytree(os.path.join(out, '96'), sys.argv[2])
        # (the game's 24 x 24 button textures sit in 32 x 32 of texture memory and are addressed that way: a picture that replaces one has to have its 24 x 24 in the top left of a 32 x 32,
        # here 96 x 96 in the top left of 128 x 128)
        for group in ('xbox', 'switch', 'keys_sq'):
            for name in os.listdir(os.path.join(sys.argv[2], group)):
                path = os.path.join(sys.argv[2], group, name)
                pic = Image.open(path).convert('RGBA')
                if pic.size == (96, 96):
                    padded = Image.new('RGBA', (128, 128), (0, 0, 0, 0))
                    padded.paste(pic, (0, 0))
                    padded.save(path)
        emit_rust(sys.argv[2], sys.argv[3])
    print('icons written to', out, '(' + ', '.join(f'{k}: {len(v)}' for k, v in sets.items()) + ')')


if __name__ == '__main__':
    main()
