"""The PlayStation part of the Universal Controller Icons mod: the game's PlayStation button pictures drawn again from nothing, as exact shapes, at many times the size.

    python tools/make-playstation-icons.py <mods folder>

Writes <mods folder>/universal_controller_icons_mod/MISC.PAK/textures/misc/<texture>.png and a mod.ini.  No picture of the game and no font is read: each picture is worked out from distances to circles, lines and rounded
boxes, whose sizes and colours were measured from the game's pictures, so the edges are clean at any size.  The textures are 24 by 24 and 32 by 32 (the shoulder buttons) in a surface of 32 by 32, and a picture replaces the
whole surface: the part the game draws is at the top left and the rest is empty.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

SCALE = 16          # pixels of the picture to a texel of the game's
SURFACE = 32        # the side of the surface of each texture, in texels

OUTLINE = (9, 10, 13)
SEAM = (24, 26, 32)
RIM_LIT = (120, 130, 144)
RIM_SIDE = (46, 49, 57)
RIM_LOW = (30, 32, 38)
BODY_TOP = (34, 40, 48)
BODY_LOW = (44, 48, 56)
LABEL = (124, 128, 138)
RED, BLUE, GREEN, PINK = (226, 88, 94), (112, 178, 246), (84, 204, 146), (216, 134, 216)
WHITE = (255, 255, 255)
CENTRE = 12.0       # of a round button, in texels
RADIUS = 11.85
STROKE = 1.35       # of the symbols


class Picture:
    """A picture of `side` texels, painted from the bottom layer up."""

    def __init__(self):
        n = SURFACE * SCALE
        xs = (np.arange(n) + 0.5) / SCALE
        self.x, self.y = np.meshgrid(xs, xs)
        self.rgb = np.zeros((n, n, 3))     # (with the alpha multiplied in)
        self.alpha = np.zeros((n, n))

    def paint(self, distance, colour):
        """Paint where the distance to the shape's edge is negative (a pixel's width of edge is shaded).  `colour` is one colour or an array of them."""
        cover = np.clip(0.5 - distance * SCALE, 0.0, 1.0)
        colour = np.asarray(colour, dtype=float)
        self.rgb = self.rgb * (1.0 - cover[..., None]) + colour * cover[..., None]
        self.alpha = self.alpha * (1.0 - cover) + cover

    def save(self, path):
        alpha = np.maximum(self.alpha, 1.0e-6)
        rgb = np.where(self.alpha[..., None] > 0.0, self.rgb / alpha[..., None], 255.0)
        out = np.dstack([np.clip(rgb, 0.0, 255.0), self.alpha * 255.0])
        Image.fromarray((out + 0.5).astype(np.uint8), 'RGBA').save(path, optimize=True)


def circle(p, cx, cy, r):
    return np.hypot(p.x - cx, p.y - cy) - r


def box(p, x0, y0, x1, y1, r=0.0):
    """A box, its corners rounded by `r`."""
    cx, cy, hx, hy = (x0 + x1) / 2.0, (y0 + y1) / 2.0, (x1 - x0) / 2.0 - r, (y1 - y0) / 2.0 - r
    dx, dy = np.abs(p.x - cx) - hx, np.abs(p.y - cy) - hy
    return np.hypot(np.maximum(dx, 0.0), np.maximum(dy, 0.0)) + np.minimum(np.maximum(dx, dy), 0.0) - r


def bar(p, ax, ay, bx, by, width):
    """A stroke with square ends from a to b."""
    length = math.hypot(bx - ax, by - ay)
    ux, uy = (bx - ax) / length, (by - ay) / length
    along = (p.x - ax) * ux + (p.y - ay) * uy
    across = -(p.x - ax) * uy + (p.y - ay) * ux
    dx, dy = np.abs(along - length / 2.0) - length / 2.0, np.abs(across) - width / 2.0
    return np.hypot(np.maximum(dx, 0.0), np.maximum(dy, 0.0)) + np.minimum(np.maximum(dx, dy), 0.0)


def strokes(p, lines, width):
    """Several bars, as one shape.  A line of many points is a curve."""
    out = None
    for line in lines:
        for (ax, ay), (bx, by) in zip(line, line[1:]):
            if math.hypot(bx - ax, by - ay) < 1.0e-6:
                continue
            d = bar(p, ax, ay, bx, by, width)
            out = d if out is None else np.minimum(out, d)
        # (the joins of a curve are round, or its outer side has notches)
        if len(line) > 2:
            for x, y in line[1:-1]:
                out = np.minimum(out, circle(p, x, y, width / 2.0))
    return out


def arc(cx, cy, rx, ry, start, end, steps=24):
    """Points of an ellipse from the angle `start` to `end` (degrees, 0 is right, 90 is down)."""
    return [(cx + rx * math.cos(math.radians(start + (end - start) * i / steps)), cy + ry * math.sin(math.radians(start + (end - start) * i / steps))) for i in range(steps + 1)]


def mix(a, b, t):
    return np.asarray(a, dtype=float) + (np.asarray(b, dtype=float) - np.asarray(a, dtype=float)) * t[..., None]


def rim(p, cx, cy):
    """The colour of each place of a rim: lit at the top, plain at the sides, shaded at the bottom."""
    up = (cy - p.y) / np.maximum(np.hypot(p.x - cx, p.y - cy), 1.0e-6)
    lit = np.clip((up - 0.2) / 0.7, 0.0, 1.0)
    low = np.clip(-up, 0.0, 1.0)
    return mix(mix(RIM_SIDE, RIM_LOW, low), RIM_LIT, lit * lit * (3.0 - 2.0 * lit))


def disc():
    """The body of a round button."""
    p = Picture()
    p.paint(circle(p, CENTRE, CENTRE, RADIUS), OUTLINE)
    p.paint(circle(p, CENTRE, CENTRE, RADIUS - 0.8), rim(p, CENTRE, CENTRE))
    p.paint(circle(p, CENTRE, CENTRE, RADIUS - 2.0), SEAM)
    low = np.clip((p.y - (CENTRE - RADIUS)) / (2.0 * RADIUS), 0.0, 1.0)
    p.paint(circle(p, CENTRE, CENTRE, RADIUS - 2.3), mix(BODY_TOP, BODY_LOW, low))
    return p


def button_circle():
    p = disc()
    p.paint(np.abs(circle(p, CENTRE, CENTRE, 6.9)) - STROKE / 2.0, RED)
    return p


def button_cross():
    p = disc()
    s = 6.25
    p.paint(strokes(p, [[(CENTRE - s, CENTRE - s), (CENTRE + s, CENTRE + s)], [(CENTRE - s, CENTRE + s), (CENTRE + s, CENTRE - s)]], STROKE), BLUE)
    return p


def button_triangle():
    p = disc()
    # (equilateral, about the middle of the button; its corners go on past the lines so that they are points)
    r = 8.2
    corners = [(CENTRE + r * math.cos(math.radians(a)), CENTRE + 1.1 + r * math.sin(math.radians(a))) for a in (-90.0, 30.0, 150.0)]
    edges = [corners[0], corners[1], corners[2], corners[0]]
    d = None
    for a, b in zip(edges, edges[1:]):
        length = math.hypot(b[0] - a[0], b[1] - a[1])
        ux, uy = (b[0] - a[0]) / length, (b[1] - a[1]) / length
        side = (p.x - a[0]) * uy - (p.y - a[1]) * ux
        d = side if d is None else np.maximum(d, side)
    p.paint(np.maximum(d, -d - STROKE), GREEN)
    return p


def button_square():
    p = disc()
    s = 5.75
    outer = box(p, CENTRE - s, CENTRE - s, CENTRE + s, CENTRE + s)
    p.paint(np.maximum(outer, -outer - STROKE), PINK)
    return p


def shoulder(digit):
    """A shoulder button as the game has it: a pill at the top of the surface, 31 by 17."""
    p = Picture()
    x0, y0, x1, y1 = 0.0, 0.0, 31.0, 17.0
    p.paint(box(p, x0, y0, x1, y1, 4.0), OUTLINE)
    light = np.clip((4.0 - p.y) / 3.0, 0.0, 1.0)
    p.paint(box(p, x0 + 0.9, y0 + 0.9, x1 - 0.9, y1 - 0.9, 3.2), mix(RIM_LOW, RIM_LIT, light * 0.7))
    p.paint(box(p, x0 + 1.9, y0 + 1.9, x1 - 1.9, y1 - 1.9, 2.4), SEAM)
    low = np.clip(p.y / y1, 0.0, 1.0)
    p.paint(box(p, x0 + 2.2, y0 + 2.2, x1 - 2.2, y1 - 2.2, 2.1), mix(BODY_TOP, BODY_LOW, low))
    top, bottom, w = 4.6, 12.4, 1.7
    lines = [[(10.3, top), (10.3, bottom)], [(9.45, bottom - w / 2.0), (14.6, bottom - w / 2.0)]]
    if digit == 1:
        lines += [[(18.5, top), (18.5, bottom)], [(16.2, bottom - w / 2.0), (20.8, bottom - w / 2.0)], [(18.9, top + 0.3), (16.1, top + 2.3)]]
    else:
        lines += [arc(18.5, top + 2.4, 2.45, 1.55, 180.0, 400.0)]
        lines += [[(20.3, top + 3.5), (16.55, bottom - w / 2.0 - 0.1)], [(16.0, bottom - w / 2.0), (21.2, bottom - w / 2.0)]]
    # (nothing stands above the tops of the letters)
    p.paint(np.maximum(strokes(p, lines, w), top - p.y), LABEL)
    return p


def select():
    """The Select button as the game has it: the button in two pieces above its name, on black."""
    p = Picture()
    p.paint(box(p, 0.0, 0.0, 23.2, 7.0), OUTLINE)
    p.paint(box(p, 0.0, 9.0, 24.0, 23.0), OUTLINE)
    shade = np.clip((p.y - 1.4) / 5.0, 0.0, 1.0)
    p.paint(box(p, 1.0, 1.4, 22.4, 6.4, 0.5), mix((160, 164, 166), (94, 100, 104), shade))
    p.paint(box(p, 10.2, 1.0, 10.9, 7.0), OUTLINE)
    shade = np.clip((p.y - 10.0) / 4.4, 0.0, 1.0)
    p.paint(box(p, 1.0, 10.0, 23.0, 14.4, 0.5), mix((108, 114, 118), (70, 76, 80), shade))
    top, bottom, w = 17.9, 22.6, 0.95
    half = w / 2.0
    letters = []
    for i, ch in enumerate('SELECT'):
        x0 = 0.35 + i * 4.0
        x1 = x0 + 3.1
        mid = (top + bottom) / 2.0
        a, b, c = top + half, mid, bottom - half
        if ch == 'S':
            letters += [[(x1, a), (x0 + 0.5, a)] + arc(x0 + 0.95, (a + b) / 2.0, 0.95, (b - a) / 2.0, 270.0, 90.0) + [(x1 - 0.95, b)] + arc(x1 - 0.95, (b + c) / 2.0, 0.95, (c - b) / 2.0, -90.0, 90.0) + [(x0, c)]]
        elif ch == 'E':
            letters += [[(x0 + half, top), (x0 + half, bottom)], [(x0, a), (x1, a)], [(x0, b), (x1 - 0.3, b)], [(x0, c), (x1, c)]]
        elif ch == 'L':
            letters += [[(x0 + half, top), (x0 + half, bottom)], [(x0, c), (x1, c)]]
        elif ch == 'C':
            letters += [[(x1, a), (x0 + 1.1, a)] + arc(x0 + 1.1, mid, 1.1 - half + 0.47, (c - a) / 2.0, 270.0, 90.0) + [(x1, c)]]
        else:
            letters += [[(x0, a), (x1, a)], [((x0 + x1) / 2.0, top), ((x0 + x1) / 2.0, bottom)]]
    p.paint(strokes(p, letters, w), WHITE)
    return p


PICTURES = {
    'mm_button01': button_circle, 'mm_button02': button_cross, 'mm_button03': button_triangle, 'mm_button04': button_square, 'mm_button05': select, 'mm_circle_24x24': button_circle,
    'mm_cursor13': lambda: shoulder(1), 'mm_cursor14': lambda: shoulder(2),
}


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(1)
    mod = os.path.join(sys.argv[1], 'universal_controller_icons_mod')
    out = os.path.join(mod, 'MISC.PAK', 'textures', 'misc')
    os.makedirs(out, exist_ok=True)
    for name, make in PICTURES.items():
        make().save(os.path.join(out, name + '.png'))
    with open(os.path.join(mod, 'mod.ini'), 'w', encoding='utf-8') as f:
        f.write('name = Universal Controller Icons\nenabled = true\n')


if __name__ == '__main__':
    main()
