"""Make the HD font mod: the five fonts of the game drawn again, as curves and strokes, at many times the size (the game's characters are only measured, no pixel of them is used).

    python tools/make-hd-font.py <extracted textures folder> <mods folder> [--refit]

Writes <mods folder>/hd_font/MISC.PAK/textures/misc/<font>.png and a mod.ini.  The game's art stays out of the repository: the mod is written next to the game, and this script is what is kept.

Every character is drawn here once, in GLYPHS, as the middle lines of its strokes: straight lines and quarter curves in a box from 0 to 1, as a type designer would lay it out.  What differs between the
fonts is measured from the game's textures (tools/extract_textures.py): each number in a drawing is moved, and the width of the pen and the squareness of the curves chosen, until the drawing made as small as the
original gives the original's pixels.  A number used twice in a drawing is one measurement, so a stem stays straight and a bowl stays round whatever the pixels say; the same pen is then used for all the characters
of a kind in a font, so that the font looks of one piece.  The measuring takes minutes: its result is kept in <extracted textures folder>/hd-font-fit.json (--refit does it again).

The pictures have the proportions of the screen: a texel of the game's fonts is 1 of 640 across and 1 of 224 down, so they are stretched more downwards than across.  Where each character is comes from
tools/hd-font-rects.json (the game's tables of characters).  The marks of accented capitals in squarefont are not characters of a table: they are made bigger from the original.
"""
import json
import multiprocessing
import os
import re
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SCALE_X = 8
SCALE_Y = 16
MEASURE_SAMPLES = 5    # samples across a texel of the original when comparing
DRAW_SAMPLES = 3       # samples across a pixel of the picture
ARC_STEPS = 10
REGULAR = 0.004        # how much a measurement is held to the drawing (against the error of the pixels)

ROUND_O = 'M !.45,0 Ah 1,.3 L 1,!.3 Av !.45,1 L .45,1 Ah 0,!.3 L 0,.3 Av .45,0 Z'
ELLIPSE = 'M #.5,0 Ah 1,#.5 Av #.5,1 Ah 0,#.5 Av #.5,0 Z'
ROUND_S = 'M 1,.2 Av !.42,0 L .42,0 Ah 0,.25 Av .42,.5 L !.42,.5 Ah 1,.75 Av !.42,1 L .42,1 Ah 0,!.2'
# the drawings.  Letters: M move, L line, Ah a quarter curve that starts level, Av one that starts upright, Z close.  y goes down.  0 and 1 are the edges of the character's box and are fixed, #n is a fixed number,
# !n is 1 less the measurement n (for what is the same on both sides).  A character can have several drawings (a list): the first is used unless a later one fits clearly better.
GLYPHS = {
    '!': 'M #.5,0 L #.5,.7 M #.5,.84 L #.5,1',
    '"': 'M 0,0 L 0,1 M 1,0 L 1,1',
    '#': 'M !.2,0 L .55,1 M !.55,0 L .2,1 M 0,.33 L 1,.33 M 0,!.33 L 1,!.33',
    '$': 'M #.5,0 L #.5,1 M 1,.25 Av !.42,.1 L .42,.1 Ah 0,.3 Av .42,.5 L !.42,.5 Ah 1,.7 Av !.42,.9 L .42,.9 Ah 0,.75',
    '%': ['M .22,0 Ah .42,.25 Av .22,.5 Ah 0,.25 Av .22,0 Z M !.22,.5 Ah 1,.75 Av !.22,1 Ah .58,.75 Av !.22,.5 Z M .85,#-.05 L .15,#1.05'],
    '&': 'M 1,1 L .2,.36 Av .45,0 L .55,0 Ah .78,.25 Av .55,.5 L .36,.55 Ah 0,.76 Av .36,1 L .55,1 Ah 1,.55',
    "'": 'M #.5,0 L #.5,1',
    '(': 'M 1,0 Ah 0,#.5 Av 1,1',
    ')': 'M 0,0 Ah 1,#.5 Av 0,1',
    '*': 'M #.5,0 L #.5,1 M 0,.25 L 1,!.25 M 1,.25 L 0,!.25',
    '+': 'M #.5,0 L #.5,1 M 0,#.5 L 1,#.5',
    ',': 'M 1,0 L 1,.4 L 0,1',
    '-': 'M 0,#.5 L 1,#.5',
    '.': 'M #.5,0 L #.5,1',
    '/': 'M 0,#1.1 L 1,#-.1',
    '0': [ELLIPSE, ROUND_O],
    '1': ['M 0,.25 L .6,0 L .6,1', 'M 0,.25 L .55,0 L .55,1 M 0,1 L 1,1'],
    '2': 'M 0,.28 Av .42,0 L !.42,0 Ah 1,.28 Av .7,.5 L 0,1 L 1,1',
    '3': 'M 0,.2 Av .42,0 L .6,0 Ah .96,.24 Av .6,.48 L .35,.48 M .6,.48 Ah 1,.74 Av .6,1 L .42,1 Ah 0,!.2',
    '4': 'M .72,1 L .72,0 L 0,.72 L 1,.72',
    '5': 'M .95,0 L .1,0 L .03,.48 M .03,.46 L .55,.4 Ah 1,.7 Av .55,1 L .4,1 Ah 0,.82',
    '6': 'M .96,.2 Av .55,0 L .45,0 Ah 0,.35 L 0,.7 Av .45,1 L .58,1 Ah 1,.72 Av .58,.42 L .42,.42 Ah 0,.7',
    '7': 'M 0,0 L 1,0 L .3,#1.1',
    '8': 'M .58,0 Ah .96,.24 Av .58,.48 L !.58,.48 Ah .04,.24 Av !.58,0 Z M .58,.48 Ah 1,.74 Av .58,1 L !.58,1 Ah 0,.74 Av !.58,.48 Z',
    '9': 'M .04,.8 Av .45,1 L .55,1 Ah 1,.65 L 1,.3 Av .55,0 L .42,0 Ah 0,.28 Av .42,.58 L .58,.58 Ah 1,.3',
    ':': 'M #.5,0 L #.5,.25 M #.5,!.25 L #.5,1',
    ';': 'M .8,0 L .8,.22 M .8,.58 L .8,.8 L 0,1',
    '<': 'M 1,0 L 0,#.5 L 1,1',
    '=': 'M 0,.2 L 1,.2 M 0,!.2 L 1,!.2',
    '>': 'M 0,0 L 1,#.5 L 0,1',
    '?': 'M 0,.22 Av .42,0 L !.42,0 Ah 1,.25 Av .58,.5 Ah .5,.72 M .5,.86 L .5,1',
    '@': 'M .62,.72 Ah 1,.45 L 1,.42 Av .55,0 L .45,0 Ah 0,.5 Av .45,1 L .8,1 M .68,.3 L .62,.72 M .68,.32 L .5,.28 Ah .3,.5 Av .45,.72 L .62,.72',
    'A': 'M 0,#1.2 L #.5,#-.15 L 1,#1.2 M .2,.68 L !.2,.68',
    'B': 'M 0,0 L 0,1 M 0,0 L .7,0 Ah .96,.24 Av .7,.48 L 0,.48 M .7,.48 Ah 1,.74 Av .7,1 L 0,1',
    'C': 'M 1,.22 Av !.45,0 L .45,0 Ah 0,.3 L 0,!.3 Av .45,1 L !.45,1 Ah 1,!.22',
    'D': 'M 0,0 L .55,0 Ah 1,.3 L 1,!.3 Av .55,1 L 0,1 L 0,0 Z',
    'E': 'M 1,0 L 0,0 L 0,1 L 1,1 M 0,.5 L .9,.5',
    'F': 'M 1,0 L 0,0 L 0,1 M 0,.5 L .85,.5',
    'G': 'M 1,.22 Av !.45,0 L .45,0 Ah 0,.3 L 0,!.3 Av .45,1 L !.45,1 Ah 1,.75 L 1,.55 L .6,.55',
    'H': 'M 0,0 L 0,1 M 1,0 L 1,1 M 0,.5 L 1,.5',
    'I': 'M #.5,0 L #.5,1',
    'J': 'M 1,0 L 1,.7 Av .55,1 L .45,1 Ah 0,.75',
    'K': 'M 0,0 L 0,1 M .98,#-.1 L 0,.65 M .3,.42 L 1,#1.1',
    'L': 'M 0,0 L 0,1 L 1,1',
    'M': 'M 0,1 L 0,0 L #.5,.92 L 1,0 L 1,1',
    'N': 'M 0,1 L 0,0 L 1,1 L 1,0',
    'O': [ELLIPSE, ROUND_O],
    'P': 'M 0,1 L 0,0 L .68,0 Ah 1,.28 Av .68,.56 L 0,.56',
    'Q': [ELLIPSE + ' M .6,.72 L #1.05,#1.05', ROUND_O + ' M .6,.72 L #1.05,#1.05'],
    'R': 'M 0,1 L 0,0 L .68,0 Ah 1,.26 Av .68,.52 L 0,.52 M .6,.52 L 1,#1.1',
    'S': ROUND_S,
    'T': 'M 0,0 L 1,0 M #.5,0 L #.5,1',
    'U': 'M 0,0 L 0,.7 Av .45,1 L !.45,1 Ah 1,.7 L 1,0',
    'V': 'M 0,#-.15 L #.5,#1.15 L 1,#-.15',
    'W': 'M 0,#-.15 L .25,#1.15 L #.5,.05 L !.25,#1.15 L 1,#-.15',
    'X': 'M 0,#-.1 L 1,#1.1 M 1,#-.1 L 0,#1.1',
    'Y': 'M 0,#-.1 L #.5,.55 L 1,#-.1 M #.5,.55 L #.5,1',
    'Z': 'M 0,0 L 1,0 L 0,1 L 1,1',
    '[': 'M 1,0 L 0,0 L 0,1 L 1,1',
    '\\': 'M 0,#-.1 L 1,#1.1',
    ']': 'M 0,0 L 1,0 L 1,1 L 0,1',
    '^': 'M 0,#1.2 L #.5,#-.15 L 1,#1.2',
    '_': 'M 0,#.5 L 1,#.5',
    '`': 'M 0,#-.1 L 1,#1.1',
    'a': ['M .03,.22 Av .42,0 L .6,0 Ah 1,.25 L 1,1 M 1,.45 L .4,.45 Ah 0,.72 Av .4,1 L .6,1 Ah 1,.75', 'M 1,0 L 1,1 ' + ROUND_O],
    'b': 'M 0,0 L 0,1 M .55,.3 Ah 1,.55 L 1,.75 Av .55,1 L .4,1 Ah 0,.75 L 0,.55 Av .4,.3 Z',
    'c': 'M 1,.22 Av !.45,0 L .45,0 Ah 0,.35 L 0,!.35 Av .45,1 L !.45,1 Ah 1,!.22',
    'd': 'M 1,0 L 1,1 M .45,.3 Ah 0,.55 L 0,.75 Av .45,1 L .6,1 Ah 1,.75 L 1,.55 Av .6,.3 Z',
    'e': 'M 0,.52 L 1,.52 L 1,.4 Av !.45,0 L .45,0 Ah 0,.35 L 0,.65 Av .45,1 L .6,1 Ah .98,.82',
    'f': 'M 1,0 L .65,0 Ah .3,.25 L .3,1 M 0,.32 L .85,.32',
    'g': 'M 1,0 L 1,.8 Av .55,1 L .4,1 Ah .05,.88 M .55,0 Ah 1,.25 L 1,.5 Av .55,.75 L .45,.75 Ah 0,.5 L 0,.25 Av .45,0 Z',
    'h': 'M 0,0 L 0,1 M 0,.5 Av .45,.28 L .55,.28 Ah 1,.55 L 1,1',
    'i': 'M #.5,.3 L #.5,1 M #.5,0 L #.5,.15',
    'j': 'M 1,.3 L 1,.78 Av .55,1 L 0,1 M 1,0 L 1,.15',
    'k': 'M 0,0 L 0,1 M .95,.25 L 0,.72 M .32,.55 L 1,#1.1',
    'l': 'M #.5,0 L #.5,1',
    'm': 'M 0,0 L 0,1 M 0,.35 Av .25,0 L .3,0 Ah #.5,.35 L #.5,1 M #.5,.35 Av !.3,0 L !.25,0 Ah 1,.35 L 1,1',
    'n': 'M 0,0 L 0,1 M 0,.35 Av .45,0 L !.45,0 Ah 1,.35 L 1,1',
    'o': [ELLIPSE, ROUND_O],
    'p': 'M 0,0 L 0,1 M .55,0 Ah 1,.25 L 1,.5 Av .55,.75 L .4,.75 Ah 0,.5 L 0,.25 Av .4,0 Z',
    'q': 'M 1,0 L 1,1 M .45,0 Ah 0,.25 L 0,.5 Av .45,.75 L .6,.75 Ah 1,.5 L 1,.25 Av .6,0 Z',
    'r': 'M 0,0 L 0,1 M 0,.42 Av .55,0 L 1,0',
    's': ROUND_S,
    't': 'M .35,0 L .35,.75 Av .65,1 L 1,1 M 0,.28 L .9,.28',
    'u': 'M 0,0 L 0,.65 Av .45,1 L !.45,1 Ah 1,.65 M 1,0 L 1,1',
    'v': 'M 0,#-.15 L #.5,#1.15 L 1,#-.15',
    'w': 'M 0,#-.15 L .25,#1.15 L #.5,.05 L !.25,#1.15 L 1,#-.15',
    'x': 'M 0,#-.1 L 1,#1.1 M 1,#-.1 L 0,#1.1',
    'y': 'M 0,#-.1 L #.5,.72 M 1,#-.1 L .25,#1.1',
    'z': 'M 0,0 L 1,0 L 0,1 L 1,1',
    '{': 'M 1,0 L .75,0 Ah .45,.2 L .45,.36 Av .15,#.5 L 0,#.5 M .15,#.5 Ah .45,!.36 L .45,!.2 Av .75,1 L 1,1',
    '|': 'M #.5,0 L #.5,1',
    '}': 'M 0,0 L .25,0 Ah .55,.2 L .55,.36 Av .85,#.5 L 1,#.5 M .85,#.5 Ah .55,!.36 L .55,!.2 Av .25,1 L 0,1',
    '~': 'M 0,.85 Av .28,0 Ah .55,.5 Av .78,1 Ah 1,.15',
}
# (font07 and font09 have a percent sign where the dollar should be, and an exclamation mark where the percent should be)
SWAPPED = {'font07': {'$': '%', '%': '!'}, 'font09': {'$': '%', '%': '!'}}


def drawings_of(font, ch):
    drawing = GLYPHS[SWAPPED.get(font, {}).get(ch, ch)]
    return drawing if isinstance(drawing, list) else [drawing]


def parse(text):
    """A drawing as a list of (command, x, y), each number as (fixed value, measurement's name, whether it is 1 less it), and the measurements with their first values."""
    def number(tok, axis, params):
        if tok.startswith('#'):
            return (float(tok[1:]), None, False)
        flip = tok.startswith('!')
        value = float(tok[1:] if flip else tok)
        if value in (0.0, 1.0):
            return (value, None, False)
        key = axis + (tok[1:] if flip else tok)
        params.setdefault(key, value)
        return (None, key, flip)
    params = {}
    commands = []
    for cmd, args in re.findall(r'(Ah|Av|M|L|Z)\s*([^A-Z]*)', text):
        if cmd == 'Z':
            commands.append(('Z', None, None))
        else:
            x, y = args.strip().split(',')
            commands.append((cmd, number(x, 'x', params), number(y, 'y', params)))
    return commands, params


def value_of(n, params):
    fixed, key, flip = n
    if key is None:
        return fixed
    return 1.0 - params[key] if flip else params[key]


def strokes(commands, m):
    """The drawing as segments in texels (ax, ay, bx, by, whether a is a cut end, whether b is), and the corners that have the pen's box.  `m` has the box of the character, the pen and the curves' squareness."""
    rx, ry = m['wx'] / 2.0, m['wy'] / 2.0
    left, top = m['x0'] + rx, m['y0'] + ry
    span_x, span_y = max(0.0, m['x1'] - m['x0'] - 2 * rx), max(0.0, m['y1'] - m['y0'] - 2 * ry)
    power = 2.0 / m['n']
    paths = []
    for cmd, xn, yn in commands:
        if cmd == 'M':
            paths.append(dict(points=[], kinds=[], closed=False))
        if cmd == 'Z':
            paths[-1]['closed'] = True
            continue
        x, y = left + value_of(xn, m) * span_x, top + value_of(yn, m) * span_y
        path = paths[-1]
        if cmd in 'ML':
            path['points'].append((x, y))
            path['kinds'].append(cmd)
            continue
        x0, y0 = path['points'][-1]
        for i in range(1, ARC_STEPS + 1):
            t = (np.pi / 2.0) * i / ARC_STEPS
            s, c = np.sin(t) ** power, np.cos(t) ** power
            if cmd == 'Ah':
                path['points'].append((x0 + (x - x0) * s, y + (y0 - y) * c))
            else:
                path['points'].append((x + (x0 - x) * c, y0 + (y - y0) * s))
            path['kinds'].append('A')
    segments = []
    corners = []
    for path in paths:
        pts = path['points']
        count = len(pts) - 1
        for i in range(count):
            first, last = (i == 0 and not path['closed']), (i == count - 1 and not path['closed'])
            segments.append((pts[i][0], pts[i][1], pts[i + 1][0], pts[i + 1][1], first, last))
            # (a corner between straight lines where one is level or upright is square: the pen's box; a pointed one stays pointed)
            if 0 < i and path['kinds'][i] == 'L' and path['kinds'][i + 1] == 'L':
                a, b, c = pts[i - 1], pts[i], pts[i + 1]
                if min(abs(a[0] - b[0]), abs(a[1] - b[1])) < 1.0e-3 or min(abs(b[0] - c[0]), abs(b[1] - c[1])) < 1.0e-3:
                    corners.append(b)
    return segments, corners


def inside(px, py, commands, m):
    """Which of the points (arrays) are inked."""
    segments, corners = strokes(commands, m)
    rx, ry = max(m['wx'] / 2.0, 1.0e-3), max(m['wy'] / 2.0, 1.0e-3)
    out = np.zeros(px.shape, dtype=bool)
    x = px / rx
    y = py / ry
    for ax, ay, bx, by, cut_a, cut_b in segments:
        ax, ay, bx, by = ax / rx, ay / ry, bx / rx, by / ry
        dx, dy = bx - ax, by - ay
        length = dx * dx + dy * dy
        if length < 1.0e-9:
            continue
        t = ((x - ax) * dx + (y - ay) * dy) / length
        near = np.ones(px.shape, dtype=bool)
        if cut_a:
            near &= t >= 0.0
        if cut_b:
            near &= t <= 1.0
        t = np.clip(t, 0.0, 1.0)
        out |= near & ((x - ax - t * dx) ** 2 + (y - ay - t * dy) ** 2 <= 1.0)
    for cx, cy in corners:
        out |= (np.abs(px - cx) <= rx) & (np.abs(py - cy) <= ry)
    return out & (px >= m['x0']) & (px <= m['x1']) & (py >= m['y0']) & (py <= m['y1'])


def grid(w, h, samples):
    xs = (np.arange(w * samples) + 0.5) / samples
    ys = (np.arange(h * samples) + 0.5) / samples
    return np.meshgrid(xs, ys)


def error_of(target, commands, m, params0, px, py):
    h, w = target.shape
    ink = inside(px, py, commands, m).reshape(h, MEASURE_SAMPLES, w, MEASURE_SAMPLES).mean(axis=(1, 3))
    held = sum((m[k] - v) ** 2 for k, v in params0.items())
    return float(((ink - target) ** 2).sum()) + REGULAR * held * target.size


def tight_box(target):
    """The box of the character in the texels of its rectangle (fractions from how faint the outermost row and column are)."""
    peak = max(float(target.max()), 0.01)
    cols, rows = target.max(axis=0) / peak, target.max(axis=1) / peak
    xs, ys = np.where(cols > 0.15)[0], np.where(rows > 0.15)[0]
    if len(xs) == 0:
        return None
    return dict(x0=float(xs[0] + 1 - cols[xs[0]]), x1=float(xs[-1] + cols[xs[-1]]), y0=float(ys[0] + 1 - rows[ys[0]]), y1=float(ys[-1] + rows[ys[-1]]))


def refine(target, commands, m, params0, free, sizes):
    """Move each of the free measurements in turn while it makes the error smaller."""
    px, py = grid(target.shape[1], target.shape[0], MEASURE_SAMPLES)
    best = error_of(target, commands, m, params0, px, py)
    for size in sizes:
        for _ in range(3):
            changed = False
            for key, step in free:
                for sign in (1.0, -1.0):
                    keep = m[key]
                    m[key] = keep + sign * step * size
                    if key in ('wx', 'wy'):
                        m[key] = max(0.25, m[key])
                    if key == 'n':
                        m[key] = min(6.0, max(2.0, m[key]))
                    e = error_of(target, commands, m, params0, px, py)
                    if e < best - 1.0e-6:
                        best = e
                        changed = True
                        break
                    m[key] = keep
            if not changed:
                break
    return best


def fit_one(job):
    """Measure one character: (font, character, the texels of its rectangle, the pen if it is decided already)."""
    font, ch, target, pen = job
    box = tight_box(target)
    if box is None:
        return font, ch, None
    best = None
    for index, drawing in enumerate(drawings_of(font, ch)):
        commands, params0 = parse(drawing)
        m = dict(params0)
        m.update(box)
        sizes = (1.0, 0.5, 0.25)
        boxes = [(k, 0.25) for k in ('x0', 'x1', 'y0', 'y1')]
        moves = [(k, 0.08) for k in params0]
        if pen is None:
            # (the pen of this character alone, the first pass: the best of a few widths, then together with the rest)
            m.update(n=2.6)
            start = None
            px, py = grid(target.shape[1], target.shape[0], MEASURE_SAMPLES)
            for wx in (0.8, 1.2, 1.6, 2.2, 3.0, 4.0):
                for wy in (0.5, 0.8, 1.1, 1.5, 2.0):
                    m.update(wx=wx, wy=wy)
                    e = error_of(target, commands, m, params0, px, py)
                    if start is None or e < start[0]:
                        start = (e, wx, wy)
            m.update(wx=start[1], wy=start[2])
            free = [('wx', 0.3), ('wy', 0.2), ('n', 0.5)] + boxes + moves
        else:
            m.update(pen)
            free = boxes + moves
        e = refine(target, commands, m, params0, free, sizes)
        if best is None or e < best[0] * 0.9:
            best = (e, index, m)
    return font, ch, dict(error=best[0], drawing=best[1], m=best[2])


def kind_of(ch):
    if ch.isupper():
        return 'upper'
    if ch.islower():
        return 'lower'
    if ch.isdigit():
        return 'digit'
    return 'other'


def load(src, font):
    path = os.path.join(src, 'misc', 'fonts', font + '.png')
    return np.asarray(Image.open(path).convert('RGBA'), dtype=np.float32)[..., 3] / 255.0


def glyphs_of(font, rects):
    """The characters of a font that are drawn (squarefont's J to Z have the accent marks in their rectangles: the texture has no such capitals)."""
    out = {}
    for code, rect in rects[font].items():
        ch = chr(int(code))
        if ch not in GLYPHS or (font == 'squarefont' and 'J' <= ch <= 'Z'):
            continue
        out[ch] = rect
    return out


def measure(src, rects):
    """All the characters of all the fonts, in two passes: each with its own pen, then each with the pen of its kind."""
    atlases = {font: load(src, font) for font in rects}

    def jobs(pens):
        for font in rects:
            for ch, (u, v, w, h, _) in glyphs_of(font, rects).items():
                yield font, ch, atlases[font][v:v + h, u:u + w], (pens[font][kind_of(ch)] if pens else None)

    with multiprocessing.Pool() as pool:
        first = {}
        for font, ch, res in pool.imap_unordered(fit_one, list(jobs(None)), chunksize=4):
            if res:
                first.setdefault(font, {})[ch] = res
        pens = {}
        for font, results in first.items():
            pens[font] = {}
            everything = [r['m'] for r in results.values()]
            for kind in ('upper', 'lower', 'digit', 'other'):
                ms = [r['m'] for ch, r in results.items() if kind_of(ch) == kind] or everything
                pens[font][kind] = {k: float(np.median([m[k] for m in ms])) for k in ('wx', 'wy')}
                # (the squareness only from characters with curves: the others say nothing about it)
                curved = [r['m']['n'] for ch, r in results.items() if kind_of(ch) == kind and 'A' in ' '.join(drawings_of(font, ch)).replace('A,', '')]
                pens[font][kind]['n'] = float(np.median(curved)) if curved else 2.6
            print(font, {k: {a: round(b, 2) for a, b in v.items()} for k, v in pens[font].items()})
        second = {}
        for font, ch, res in pool.imap_unordered(fit_one, list(jobs(pens)), chunksize=4):
            if res:
                second.setdefault(font, {})[ch] = res
    return second


def draw_font(font, atlas, rects, fit):
    """The picture: the font's surface with the characters drawn on it."""
    h, w = atlas.shape
    surface_w, surface_h = 1, 1
    while surface_w < w:
        surface_w *= 2
    while surface_h < h:
        surface_h *= 2
    alpha = np.zeros((surface_h * SCALE_Y, surface_w * SCALE_X), dtype=np.float32)
    if font == 'squarefont':
        # (the marks of the accented capitals)
        rows = atlas.copy()
        rows[:60] = 0.0
        rows[94:] = 0.0
        for ch, (u, v, gw, gh, _) in glyphs_of(font, rects).items():
            rows[v:v + gh, u:u + gw] = 0.0
        big = Image.fromarray(rows, mode='F').resize((w * SCALE_X, h * SCALE_Y), Image.BICUBIC)
        marks = np.clip((np.asarray(big, dtype=np.float32) - 0.35) * 5.0, 0.0, 1.0)
        alpha[:marks.shape[0], :marks.shape[1]] = marks
    for ch, res in fit.items():
        u, v, gw, gh, _ = rects[font][str(ord(ch))]
        commands, _ = parse(drawings_of(font, ch)[res['drawing']])
        tile_w, tile_h = gw * SCALE_X, gh * SCALE_Y
        xs = (np.arange(tile_w * DRAW_SAMPLES) + 0.5) / (SCALE_X * DRAW_SAMPLES)
        ys = (np.arange(tile_h * DRAW_SAMPLES) + 0.5) / (SCALE_Y * DRAW_SAMPLES)
        px, py = np.meshgrid(xs, ys)
        ink = inside(px, py, commands, res['m']).reshape(tile_h, DRAW_SAMPLES, tile_w, DRAW_SAMPLES).mean(axis=(1, 3))
        at_y, at_x = v * SCALE_Y, u * SCALE_X
        place = alpha[at_y:at_y + tile_h, at_x:at_x + tile_w]
        np.maximum(place, ink[:place.shape[0], :place.shape[1]], out=place)
    out = np.full(alpha.shape + (4,), 255, dtype=np.uint8)
    out[..., 3] = (alpha * 255.0 + 0.5).astype(np.uint8)
    return Image.fromarray(out, 'RGBA')


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    if len(args) != 2:
        print(__doc__)
        sys.exit(1)
    src, mods = args
    with open(os.path.join(HERE, 'hd-font-rects.json'), encoding='utf-8') as f:
        rects = json.load(f)
    cache = os.path.join(src, 'hd-font-fit.json')
    if '--refit' in sys.argv or not os.path.exists(cache):
        fit = measure(src, rects)
        with open(cache, 'w', encoding='utf-8') as f:
            json.dump(fit, f)
    else:
        with open(cache, encoding='utf-8') as f:
            fit = json.load(f)
    out = os.path.join(mods, 'hd_font', 'MISC.PAK', 'textures', 'misc')
    os.makedirs(out, exist_ok=True)
    for font in rects:
        picture = draw_font(font, load(src, font), rects, fit[font])
        picture.save(os.path.join(out, font + '.png'), optimize=True)
        errors = sorted(((r['error'], ch) for ch, r in fit[font].items()), reverse=True)
        print(f'{font}: {picture.width}x{picture.height}, worst {[ch for _, ch in errors[:10]]}')
    with open(os.path.join(mods, 'hd_font', 'mod.ini'), 'w', encoding='utf-8') as f:
        f.write('name = HD Font\nenabled = true\n')


if __name__ == '__main__':
    main()
