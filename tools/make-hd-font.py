"""Make the HD font mod: the five fonts of the game drawn again, as curves and strokes, at many times the size (the game's characters are only measured, no pixel of them is used).

    python tools/make-hd-font.py <extracted textures folder> <mods folder> [--refit]

Writes <mods folder>/hd_font/MISC.PAK/textures/misc/<font>.png and a mod.ini.  The game's art stays out of the repository: the mod is written next to the game, and this script is what is kept.

Every character is drawn here once, in GLYPHS, as the middle lines of its strokes: straight lines and quarter curves in a box from 0 to 1, as a type designer would lay it out.  What differs between the
fonts is measured from the game's textures (tools/extract_textures.py): each number in a drawing is moved, and the width of the pen and the squareness of the curves chosen, until the drawing made as small as the
original gives the original's pixels.  A number used twice in a drawing is one measurement, so a stem stays straight and a bowl stays round whatever the pixels say.  The first pass measures each character alone; from all of
them come the font's pen and its lines (the tops of capitals, digits and small letters, the ascenders, the baseline, the descenders), and the second pass measures each character again on those, so that the font looks of one
piece and the characters line up.  The game draws a character of h texels over h - 1 of its texture's rows, so the lines are worked out where the characters land on the screen.  The measuring takes minutes: its result is kept in <extracted textures folder>/hd-font-fit.json (--refit does it again).

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
SOFTEN = 3             # how far the corners are rounded, in pixels of the picture
ARC_STEPS = 10         # lines in a curve when measuring (DRAW_STEPS when drawing)
DRAW_STEPS = 48
REGULAR = 0.004        # how much a measurement is held to the drawing (against the error of the pixels)
REACH = 0.18           # and how far it can go from the drawing, so that a shape stays the shape that was drawn

ELLIPSE = 'R 0,0 1,1 #0,#360 Z'
ROUND_S = 'R .03,0 .97,#.5 #-32,#-270 T 0,#.5 1,1 #-90,#148'
# the drawings.  Letters: M move, L line, Ah a quarter curve that starts level, Av one that starts upright, C a curve by two control points, R an arc of the ellipse in the box from corner to corner, between two angles (0 is right, -90 the top), T the same going on from the line before it, F a filled shape with straight sides (corners in order, no pen), Z close.  y goes down.  0 and 1 are the edges of the character's box and are fixed, #n is a fixed number,
# !n is 1 less the measurement n (for what is the same on both sides).  A character can have several drawings (a list): the first is used unless a later one fits clearly better.
GLYPHS = {
    '!': 'M #.5,0 L #.5,.7 M #.5,.84 L #.5,1',
    '"': 'M 0,0 L 0,1 M 1,0 L 1,1',
    '#': 'M !.2,0 L .55,1 M !.55,0 L .2,1 M 0,.33 L 1,.33 M 0,!.33 L 1,!.33',
    '$': 'M #.5,0 L #.5,1 M 1,.25 Av #.5,.1 Ah 0,.3 Av #.5,.5 Ah 1,.7 Av #.5,.9 Ah 0,.75',
    '%': ['M .22,0 Ah .42,.25 Av .22,.5 Ah 0,.25 Av .22,0 Z M !.22,.5 Ah 1,.75 Av !.22,1 Ah .58,.75 Av !.22,.5 Z M .85,#-.05 L .15,#1.05'],
    '&': 'M 1,#1.03 L #.3,#.36 C #.16,#.2 #.26,0 #.46,0 C #.66,0 #.74,#.2 #.58,#.34 L #.2,#.6 C #.02,#.72 #.04,1 #.36,1 C #.66,1 #.9,#.8 1,#.52',
    "'": 'M #.5,0 L #.5,1',
    '(': 'M 1,0 Ah 0,#.5 Av 1,1',
    ')': 'M 0,0 Ah 1,#.5 Av 0,1',
    '*': 'M #.5,0 L #.5,1 M 0,.25 L 1,!.25 M 1,.25 L 0,!.25',
    '+': 'M #.5,0 L #.5,1 M 0,#.5 L 1,#.5',
    ',': 'M 1,0 L 1,.4 L 0,1',
    '-': 'M 0,#.5 L 1,#.5',
    '.': 'M #.5,0 L #.5,1',
    '/': 'M 0,#1.1 L 1,#-.1',
    '0': ELLIPSE,
    '1': ['M 1,0 L 1,1 F 1,#-.12 #-.3,#.24 #-.3,#.4 1,#.1', 'M .55,0 L .55,1 F .55,#-.12 #-.3,#.24 #-.3,#.4 .55,#.1 M 0,1 L 1,1'],
    '2': 'M 0,.28 Av #.5,0 Ah 1,.28 C 1,.5 .6,.62 0,.97 L 0,1 L 1,1',
    '3': 'R 0,0 .96,#.5 #-155,#90 L .42,#.5 R 0,#.5 1,1 #-90,#155',
    '4': 'M .74,0 L .74,1 M .74,#-.08 L 0,.72 L 1,.72',
    '5': 'M 1,0 L .08,0 L .08,.5 R 0,.36 1,1 #-135,#148',
    '6': 'R 0,.4 1,1 #0,#360 Z R 0,0 1.16,1.4 #-58,#-180',
    '7': 'M 0,0 L 1,0 L .3,#1.1',
    '8': 'R .04,0 .96,.48 #0,#360 Z R 0,.48 1,1 #0,#360 Z',
    '9': 'R 0,0 1,.6 #0,#360 Z R -.16,-.4 1,1 #122,#0',
    ':': 'M #.5,0 L #.5,.25 M #.5,!.25 L #.5,1',
    ';': 'M .8,0 L .8,.22 M .8,.58 L .8,.8 L 0,1',
    '<': 'M 1,0 L 0,#.5 L 1,1',
    '=': 'M 0,0 L 1,0 M 0,1 L 1,1',
    '>': 'M 0,0 L 1,#.5 L 0,1',
    '?': 'M 0,.22 Av #.5,0 Ah 1,.25 Av .58,.5 Ah .5,.72 M .5,.86 L .5,1',
    '@': 'M #.5,#.28 Ah #.68,#.5 Av #.5,#.72 Ah #.32,#.5 Av #.5,#.28 Z M #.68,#.26 L #.68,#.62 Av #.82,#.74 Ah 1,#.5 Av #.5,0 Ah 0,#.5 Av #.5,1 L #.8,1',
    'A': 'M 0,#1.2 L #.5,#-.15 L 1,#1.2 M .2,.68 L !.2,.68',
    'B': 'M 0,1 L 0,0 L .7,0 Ah .96,.24 Av .7,.48 L 0,.48 M .7,.48 Ah 1,.74 Av .7,1 L 0,1 L 0,#.5',
    'C': 'R 0,0 1,1 #-40,#-320',
    'D': 'M 0,#.5 L 0,0 L .5,0 Ah 1,#.5 Av .5,1 L 0,1 L 0,#.5',
    'E': 'M 1,0 L 0,0 L 0,1 L 1,1 M 0,.5 L .9,.5',
    'F': 'M 1,0 L 0,0 L 0,1 M 0,.5 L .85,.5',
    'G': 'R 0,0 1,1 #-40,#-360 L .6,#.5',
    'H': 'M 0,0 L 0,1 M 1,0 L 1,1 M 0,.5 L 1,.5',
    'I': 'M #.5,0 L #.5,1',
    'J': 'M 1,0 L 1,.66 Av #.5,1 Ah 0,.72',
    'K': 'M 0,0 L 0,1 M .98,#-.1 L 0,.65 M .3,.42 L 1,#1.1',
    'L': 'M 0,0 L 0,1 L 1,1',
    'M': 'M 0,1 L 0,0 L #.5,.92 L 1,0 L 1,1',
    'N': 'M 0,1 L 0,0 L 1,1 L 1,0',
    'O': ELLIPSE,
    'P': 'M 0,1 L 0,0 L .68,0 Ah 1,.28 Av .68,.56 L 0,.56',
    'Q': ELLIPSE + ' M .6,.72 L #1.25,#1.25',
    'R': 'M 0,1 L 0,0 L .68,0 Ah 1,.26 Av .68,.52 L 0,.52 M .6,.52 L 1,#1.1',
    'S': ROUND_S,
    'T': 'M 0,0 L 1,0 M #.5,0 L #.5,1',
    'U': 'M 0,0 L 0,.6 Av #.5,1 Ah 1,.6 L 1,0',
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
    'a': ['R .02,0 1,.7 #-150,#0 L 1,1 R 0,.42 1,1 #0,#360 Z', 'M 1,0 L 1,1 ' + ELLIPSE],
    'b': 'M 0,0 L 0,1 R 0,.3 1,1 #0,#360 Z',
    'c': 'R 0,0 1,1 #-40,#-320',
    'd': 'M 1,0 L 1,1 R 0,.3 1,1 #0,#360 Z',
    'e': 'M 0,#.5 L 1,#.5 R 0,0 1,1 #0,#-305',
    'f': 'M 1,0 L .65,0 Ah .3,.25 L .3,1 M 0,.32 L .85,.32',
    'g': 'R 0,0 1,.75 #0,#360 Z M 1,0 L 1,.7 T -.02,.4 1,1 #0,#140',
    'h': 'M 0,0 L 0,1 M 0,.56 Av #.5,.28 Ah 1,.6 L 1,1',
    'i': 'M #.5,.3 L #.5,1 M #.5,0 L #.5,.15',
    'j': 'M 1,.3 L 1,.78 Av .55,1 L 0,1 M 1,0 L 1,.15',
    'k': 'M 0,0 L 0,1 M .95,.25 L 0,.72 M .32,.55 L 1,#1.1',
    'l': 'M #.5,0 L #.5,1',
    'm': 'M 0,0 L 0,1 M 0,.38 Av .27,0 Ah #.5,.38 L #.5,1 M #.5,.38 Av !.27,0 Ah 1,.38 L 1,1',
    'n': 'M 0,0 L 0,1 M 0,.4 Av #.5,0 Ah 1,.4 L 1,1',
    'o': ELLIPSE,
    'p': 'M 0,0 L 0,1 R 0,0 1,.75 #0,#360 Z',
    'q': 'M 1,0 L 1,1 R 0,0 1,.75 #0,#360 Z',
    'r': 'M 0,0 L 0,1 M 0,.42 Av .55,0 L 1,0',
    's': ROUND_S,
    't': 'M .35,0 L .35,.75 Av .65,1 L 1,1 M 0,.28 L .9,.28',
    'u': 'M 0,0 L 0,.6 Av #.5,1 Ah 1,.6 M 1,0 L 1,1',
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
# The game's tables of kerning pull some pairs together by up to 4 texels (T and O, A and T), so that their rectangles overlap: right for its blurred letters, too close for sharp ones.
KERNING = '''// HD Font: the kerning of the sharp letters.  The game's pairs that are pulled together by up to 4 texels are pulled together by at most 1.
fn on_frame() {
    if this.contains("done") { return; }
    let seen = [];
    for f in 0..5 {
        let table = peek(0x3198a8 + 24 * f + 8);
        if table == 0 { return; }
        if seen.contains(table) { continue; }
        seen.push(table);
        for i in 0..121 {
            let v = peek(table + 4 * i);
            if v < -1 { poke(table + 4 * i, -1); } else if v == -1 { poke(table + 4 * i, 0); }
        }
    }
    this.done = true;
}
'''


# the lines of a font that each character stands on (top, bottom: None is its own)
TOPS = {**{c: 'cap' for c in 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'}, **{c: 'digit' for c in '0123456789'}, **{c: 'small' for c in 'acegmnopqrsuvwxyz'}, **{c: 'tall' for c in 'bdfhkl'}}
BOTTOMS = {**{c: 'base' for c in 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefhiklmnorstuvwxz'}, **{c: 'deep' for c in 'gjpqy'}}
# measurements inside a character that lie on a line: (line, which edge of the stroke)
ON_LINES = {
    'b': {'y.3': ('small', 'top')}, 'd': {'y.3': ('small', 'top')}, 'h': {'y.28': ('small', 'top')}, 'k': {'y.25': ('small', 'top')}, 'i': {'y.3': ('small', 'top')}, 'j': {'y.3': ('small', 'top')},
    'f': {'y.32': ('small', 'top')}, 't': {'y.28': ('small', 'top')}, 'p': {'y.75': ('base', 'bottom')}, 'q': {'y.75': ('base', 'bottom')}, 'g': {'y.75': ('base', 'bottom')},
}
# what the lines are measured from: the flat ones, not those whose tips the pixels blur
LINE_FROM = {'cap': ('top', 'EFHTZLDBPRKN'), 'digit': ('top', '123457'), 'small': ('top', 'xzvwnmur'), 'tall': ('top', 'bdhkl'), 'base': ('bottom', 'EFHLTZDBIKNRxzmnhik'), 'deep': ('bottom', 'pqgy')}
# round characters go this much of their height past the lines, or they look smaller than the flat ones
OVERSHOOT = 0.012
ROUND_TOP = 'CGOQS0235689acdegopqs'
ROUND_BOTTOM = 'CGJOQSU035689abcdeostu'
SNAP = 1.5             # (a character further than this from its line, in texels, is meant to be: font09's J hangs below the baseline)


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
    for cmd, args in re.findall(r'(Ah|Av|M|L|Z|C|R|T|F)\s*([^A-Z]*)', text):
        if cmd == 'Z':
            commands.append(('Z', None, None, None))
            continue
        pairs = [pair.split(',') for pair in args.split()]
        points = [(number(x, 'x', params), number(y, 'y', params)) for x, y in pairs]
        commands.append((cmd, points[-1][0], points[-1][1], points[:-1]))
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
    steps = m.get('steps', ARC_STEPS)
    fills = []
    for cmd, xn, yn, controls in commands:
        if cmd == 'F':
            fills.append([(left + value_of(cx, m) * span_x, top + value_of(cy, m) * span_y) for cx, cy in controls + [(xn, yn)]])
            continue
        if cmd in 'MR':
            paths.append(dict(points=[], kinds=[], closed=False))
        if cmd == 'Z':
            paths[-1]['closed'] = True
            continue
        path = paths[-1]
        if cmd in 'RT':
            (ax, ay), (bx, by) = [(left + value_of(cx, m) * span_x, top + value_of(cy, m) * span_y) for cx, cy in controls]
            start, end = np.radians(value_of(xn, m)), np.radians(value_of(yn, m))
            count = max(2, int(steps * abs(end - start) / (np.pi / 2.0)))
            for i in range(count + 1):
                t = start + (end - start) * i / count
                c, d = np.cos(t), np.sin(t)
                path['points'].append(((ax + bx) / 2.0 + (bx - ax) / 2.0 * np.sign(c) * abs(c) ** power, (ay + by) / 2.0 + (by - ay) / 2.0 * np.sign(d) * abs(d) ** power))
                path['kinds'].append('A')
            continue
        x, y = left + value_of(xn, m) * span_x, top + value_of(yn, m) * span_y
        if cmd in 'ML':
            path['points'].append((x, y))
            path['kinds'].append(cmd)
            continue
        x0, y0 = path['points'][-1]
        if cmd == 'C':
            (ax, ay), (bx, by) = [(left + value_of(cx, m) * span_x, top + value_of(cy, m) * span_y) for cx, cy in controls]
            for i in range(1, steps + 1):
                t = i / steps
                c0, c1, c2, c3 = (1 - t) ** 3, 3 * t * (1 - t) ** 2, 3 * t * t * (1 - t), t ** 3
                path['points'].append((c0 * x0 + c1 * ax + c2 * bx + c3 * x, c0 * y0 + c1 * ay + c2 * by + c3 * y))
                path['kinds'].append('A')
            continue
        for i in range(1, steps + 1):
            t = (np.pi / 2.0) * i / steps
            s, c = np.sin(t) ** power, np.cos(t) ** power
            if cmd == 'Ah':
                path['points'].append((x0 + (x - x0) * s, y + (y0 - y) * c))
            else:
                path['points'].append((x + (x0 - x) * c, y0 + (y - y0) * s))
            path['kinds'].append('A')
    def to_edge(at, inner):
        # (a stroke that ends on the edge of the box goes on to the edge: the middle of the pen stops short of it, by half the pen)
        dx, dy = at[0] - inner[0], at[1] - inner[1]
        if abs(dx) >= abs(dy):
            on_edge = min(abs(at[0] - left), abs(at[0] - left - span_x)) < 1.0e-6
            return (at[0] + (rx if dx > 0 else -rx), at[1]) if on_edge and dx != 0.0 else at
        on_edge = min(abs(at[1] - top), abs(at[1] - top - span_y)) < 1.0e-6
        return (at[0], at[1] + (ry if dy > 0 else -ry)) if on_edge else at

    outlines = []
    corners = []
    for path in paths:
        pts = list(path['points'])
        kinds = path['kinds']
        for i in range(1, len(pts) - 1):
            # (a corner between a level and an upright straight line is square: the pen's box; a pointed one stays pointed)
            if kinds[i] == 'L' and kinds[i + 1] == 'L':
                a, b, c = pts[i - 1], pts[i], pts[i + 1]
                if min(abs(a[0] - b[0]), abs(a[1] - b[1])) < 1.0e-3 and min(abs(b[0] - c[0]), abs(b[1] - c[1])) < 1.0e-3:
                    corners.append(b)
        if not path['closed'] and len(pts) > 1:
            pts[0] = to_edge(pts[0], pts[1])
            pts[-1] = to_edge(pts[-1], pts[-2])
        outlines.append((pts, path['closed']))
    return outlines, corners, fills


def triangles(pts, closed):
    """A stroke as triangles and round joins, with the pen a circle (the points are given divided by the pen's size): the line down its middle offset to both sides, so that its ends are flat and a curve is smooth."""
    pts = [p for i, p in enumerate(pts) if i == 0 or abs(p[0] - pts[i - 1][0]) + abs(p[1] - pts[i - 1][1]) > 1.0e-9]
    if closed and len(pts) > 1 and abs(pts[0][0] - pts[-1][0]) + abs(pts[0][1] - pts[-1][1]) < 1.0e-9:
        pts = pts[:-1]
    count = len(pts)
    edges = count if closed else count - 1
    if edges < 1:
        return [], []
    tangents = []
    for i in range(edges):
        (ax, ay), (bx, by) = pts[i], pts[(i + 1) % count]
        size = np.hypot(bx - ax, by - ay)
        tangents.append(((bx - ax) / size, (by - ay) / size))
    out = []
    joins = []

    def side(i, own):
        """The offset at point i for the edge `own`: shared with the edge next to it where the line bends little, its own at a corner."""
        other = None
        if own == i and (closed or i > 0):
            other = (i - 1) % edges
        elif own != i and (closed or i < edges):
            other = i % edges
        if other is None or tangents[other][0] * tangents[own][0] + tangents[other][1] * tangents[own][1] < 0.9:
            return -tangents[own][1], tangents[own][0]
        nx, ny = -(tangents[own][1] + tangents[other][1]), tangents[own][0] + tangents[other][0]
        size = max(np.hypot(nx, ny), 1.0e-9)
        return nx / size, ny / size

    for i in range(edges):
        j = (i + 1) % count
        (ax, ay), (bx, by) = pts[i], pts[j]
        (nax, nay), (nbx, nby) = side(i, i), side(j, i)
        out.append((ax, ay, ax + nax, ay + nay, bx + nbx, by + nby))
        out.append((ax, ay, bx + nbx, by + nby, bx, by))
        out.append((ax, ay, bx, by, bx - nbx, by - nby))
        out.append((ax, ay, bx - nbx, by - nby, ax - nax, ay - nay))
        # (a corner has the round of the pen; a line that bends little needs none)
        if (closed or i + 1 < edges) and tangents[i][0] * tangents[(i + 1) % edges][0] + tangents[i][1] * tangents[(i + 1) % edges][1] < 0.9:
            joins.append((bx, by))
    return out, joins


def inside(px, py, commands, m):
    """Which of the points (arrays) are inked."""
    outlines, corners, fills = strokes(commands, m)
    rx, ry = max(m['wx'] / 2.0, 1.0e-3), max(m['wy'] / 2.0, 1.0e-3)
    out = np.zeros(px.shape, dtype=bool)
    xs, ys = px[0, :] / rx, py[:, 0] / ry

    def window(x0, x1, y0, y1):
        # (only the points that the shape can reach)
        return slice(np.searchsorted(ys, y0), np.searchsorted(ys, y1, side='right')), slice(np.searchsorted(xs, x0), np.searchsorted(xs, x1, side='right'))

    def triangle(ax, ay, bx, by, cx, cy):
        area = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
        if abs(area) < 1.0e-12:
            return
        rows, columns = window(min(ax, bx, cx), max(ax, bx, cx), min(ay, by, cy), max(ay, by, cy))
        x, y = xs[columns][None, :], ys[rows][:, None]
        if x.size == 0 or y.size == 0:
            return
        flip = 1.0 if area > 0.0 else -1.0
        hit = ((bx - ax) * (y - ay) - (by - ay) * (x - ax)) * flip >= 0.0
        hit &= ((cx - bx) * (y - by) - (cy - by) * (x - bx)) * flip >= 0.0
        hit &= ((ax - cx) * (y - cy) - (ay - cy) * (x - cx)) * flip >= 0.0
        out[rows, columns] |= hit

    for corners_of in fills:
        pts = [(x / rx, y / ry) for x, y in corners_of]
        for i in range(1, len(pts) - 1):
            triangle(pts[0][0], pts[0][1], pts[i][0], pts[i][1], pts[i + 1][0], pts[i + 1][1])
    for pts, closed in outlines:
        shapes, joins = triangles([(x / rx, y / ry) for x, y in pts], closed)
        for shape in shapes:
            triangle(*shape)
        for jx, jy in joins:
            rows, columns = window(jx - 1.0, jx + 1.0, jy - 1.0, jy + 1.0)
            x, y = xs[columns][None, :], ys[rows][:, None]
            out[rows, columns] |= (x - jx) ** 2 + (y - jy) ** 2 <= 1.0
    for cx, cy in corners:
        out |= (np.abs(px - cx) <= rx) & (np.abs(py - cy) <= ry)
    # (a tail may go past the box: Q)
    return out & (px >= m['x0']) & (px <= m['x1']) & (py >= m['y0']) & (py <= m.get('clip', m['y1']))


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
                    if key in params0:
                        m[key] = min(params0[key] + REACH, max(params0[key] - REACH, m[key]))
                    if key in ('x0', 'y0'):
                        m[key] = max(0.0, m[key])
                    if key == 'x1':
                        m[key] = min(float(target.shape[1]), m[key])
                    if key == 'y1':
                        m[key] = min(float(target.shape[0]), m[key])
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
    """Measure one character: (font, character, the texels of its rectangle, the pen and what the lines fix if they are decided already)."""
    font, ch, target, pen, fixed = job
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
            free = [('wx', 0.3), ('wy', 0.2), ('n', 0.5)] + boxes + [(k, 0.08) for k in params0]
        else:
            m.update(pen)
            m.update(fixed['box'])
            span = max(0.01, m['y1'] - m['y0'] - m['wy'])
            for key, (line, edge) in ON_LINES.get(ch, {}).items():
                if key in params0:
                    at = fixed['lines'][line] if line in fixed['lines'] else None
                    if at is None:
                        continue
                    at = texel_of(at, fixed['rect'])
                    m[key] = (at - m['y0'] - (m['wy'] if edge == 'bottom' else 0.0)) / span
                    del params0[key]
            free = [b for b in boxes if b[0] not in fixed['box']] + [(k, 0.08) for k in params0]
            # (the bars of these signs have their own thickness)
            if ch in '=-_':
                free.append(('wy', 0.2))
        e = refine(target, commands, m, params0, free, sizes)
        if best is None or e < best[0] * 0.9:
            best = (e, index, m)
    return font, ch, dict(error=best[0], drawing=best[1], m=best[2])


def screen_of(y, rect):
    """Where a row of the texture lands on the screen, from the top of the line (the game draws h texels over h - 1 rows)."""
    _, _, _, h, drop = rect
    return drop + y * h / max(1, h - 1)


def texel_of(at, rect):
    _, _, _, h, drop = rect
    return (at - drop) * max(1, h - 1) / h


def shown(y, rect):
    """A row kept inside what the game shows of a character's rectangle (a box measured past it is cut off)."""
    return min(max(y, 0.0), float(max(1, rect[3] - 1)))


def font_lines(results, font_rects):
    """The lines of a font on the screen, from the characters measured alone."""
    lines = {}
    for line, (edge, chars) in LINE_FROM.items():
        ats = []
        for ch in chars:
            if ch in results:
                rect = font_rects[str(ord(ch))]
                ats.append(screen_of(shown(results[ch]['m']['y0' if edge == 'top' else 'y1'], rect), rect))
        if ats:
            lines[line] = float(np.median(ats))
    return lines


def plan(ch, result, rect, lines, signs, pen):
    """What the lines fix of a character: its box up and down, and the rect and the lines to place what is inside it."""
    m = result['m']
    box = {}
    for name, table in (('y0', TOPS), ('y1', BOTTOMS)):
        line = table.get(ch)
        if line in lines:
            at = shown(texel_of(lines[line], rect), rect)
            # (J hangs below the baseline in some fonts)
            if abs(at - m[name]) <= (0.6 if ch == 'J' and name == 'y1' else SNAP):
                box[name] = at
    if ch == 'Q' and m['y1'] > box.get('y1', m['y1']):
        box['clip'] = m['y1']
    if ch in 'mnhur':
        # (an arch is round, however square the font's bowls are)
        box['n'] = 2.0
    if ch == '1':
        # (the pixels of the flag are faint: it would be measured too short, and the character would look like an l)
        box['x1'] = m['x1']
        box['x0'] = max(0.0, min(m['x0'], m['x1'] - 1.9 * pen['wx']))
    if ch in ROUND_TOP and 'y0' in box and 'y1' in box:
        box['y0'] = max(0.0, box['y0'] - OVERSHOOT * (box['y1'] - box['y0']))
    if ch in ROUND_BOTTOM and 'y0' in box and 'y1' in box:
        box['y1'] = min(float(rect[3] - 1), box['y1'] + OVERSHOOT * (box['y1'] - box['y0']))
    if ch in signs:
        # (the signs of arithmetic have one middle and, but for =, one height)
        middle, half = signs[ch]
        box['y0'] = texel_of(middle - half, rect)
        box['y1'] = texel_of(middle + half, rect)
    return dict(box=box, lines=lines, rect=rect)


def sign_lines(results, font_rects):
    out = {}
    measured = {ch: (screen_of(results[ch]['m']['y0'], font_rects[str(ord(ch))]), screen_of(results[ch]['m']['y1'], font_rects[str(ord(ch))])) for ch in '+<>=-' if ch in results}
    if not measured:
        return out
    middle = float(np.median([(a + b) / 2.0 for ch, (a, b) in measured.items()]))
    arrows = [(b - a) / 2.0 for ch, (a, b) in measured.items() if ch in '+<>']
    for ch, (a, b) in measured.items():
        half = float(np.median(arrows)) if ch in '+<>' and arrows else (b - a) / 2.0
        out[ch] = (middle, half)
    return out


def kind_of(ch):
    if ch.isupper() or ch in '&?':
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

    def jobs(pens, first):
        for font in rects:
            for ch, rect in glyphs_of(font, rects).items():
                u, v, w, h, _ = rect
                target = atlases[font][v:v + h, u:u + w]
                if not pens:
                    yield font, ch, target, None, None
                elif ch in first[font]:
                    yield font, ch, target, pens[font][kind_of(ch)], plan(ch, first[font][ch], rect, lines[font], signs[font], pens[font][kind_of(ch)])

    with multiprocessing.Pool() as pool:
        first = {}
        for font, ch, res in pool.imap_unordered(fit_one, list(jobs(None, None)), chunksize=4):
            if res:
                first.setdefault(font, {})[ch] = res
        pens = {}
        for font, results in first.items():
            pens[font] = {}
            everything = [r['m'] for r in results.values()]
            for kind in ('upper', 'lower', 'digit', 'other'):
                ms = [r['m'] for ch, r in results.items() if kind_of(ch) == kind] or everything
                pens[font][kind] = {k: float(np.median([m[k] for m in ms])) for k in ('wx', 'wy')}
                # (the squareness, one for the font: from its round characters, which say most about it)
                pens[font][kind]['n'] = float(np.median([r['m']['n'] for ch, r in results.items() if ch in 'OCGQ0ocegq']))
            print(font, {k: {a: round(b, 2) for a, b in v.items()} for k, v in pens[font].items()})
        lines = {font: font_lines(results, rects[font]) for font, results in first.items()}
        signs = {font: sign_lines(results, rects[font]) for font, results in first.items()}
        print({font: {k: round(v, 2) for k, v in ls.items()} for font, ls in lines.items()})
        second = {}
        for font, ch, res in pool.imap_unordered(fit_one, list(jobs(pens, first)), chunksize=4):
            if res:
                second.setdefault(font, {})[ch] = res
    for font, results in second.items():
        # (a character that is one stem is as wide as the pen)
        for ch in 'Il|':
            if ch in results:
                m = results[ch]['m']
                middle = (m['x0'] + m['x1']) / 2.0
                m['x0'], m['x1'] = middle - m['wx'] / 2.0, middle + m['wx'] / 2.0
    return second


def soften(alpha):
    """Round the corners a little, as the edge of real type is: a spike comes to a small curve, and a straight edge stays where it is."""
    def blur(a, axis, width):
        for _ in range(3):
            sums = np.cumsum(np.insert(a, 0, 0.0, axis=axis), axis=axis)
            half = width // 2
            size = a.shape[axis]
            top = np.take(sums, np.clip(np.arange(size) + half + 1, 0, size), axis=axis)
            bottom = np.take(sums, np.clip(np.arange(size) - half, 0, size), axis=axis)
            a = (top - bottom) / width
        return a
    wide = blur(blur(alpha.astype(np.float64), 1, SOFTEN), 0, SOFTEN)
    return np.clip((wide - 0.5) * SOFTEN + 0.5, 0.0, 1.0).astype(np.float32)


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
        ink = inside(px, py, commands, dict(res['m'], steps=DRAW_STEPS)).reshape(tile_h, DRAW_SAMPLES, tile_w, DRAW_SAMPLES).mean(axis=(1, 3))
        at_y, at_x = v * SCALE_Y, u * SCALE_X
        place = alpha[at_y:at_y + tile_h, at_x:at_x + tile_w]
        np.maximum(place, ink[:place.shape[0], :place.shape[1]], out=place)
    alpha = soften(alpha)
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
        f.write('name = HD Font\nenabled = true\nscript = kerning.rhai\n')
    os.makedirs(os.path.join(mods, 'hd_font', 'scripts'), exist_ok=True)
    with open(os.path.join(mods, 'hd_font', 'scripts', 'kerning.rhai'), 'w', encoding='utf-8') as f:
        f.write(KERNING)


if __name__ == '__main__':
    main()
