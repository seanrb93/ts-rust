"""Look at the hands on a gun without starting the game: the OBJ that make-hands.py writes (`work/<weapon>_hands.obj`) in four views, the hand in skin colour.

    python plot_hands.py <obj> <out.png>
"""
import sys

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.collections import PolyCollection


def main():
    v, tris, mats = [], [], []
    mat = None
    for line in open(sys.argv[1]):
        w = line.split()
        if not w:
            continue
        if w[0] == 'v':
            v.append([float(x) for x in w[1:4]])
        elif w[0] == 'usemtl':
            mat = w[1]
        elif w[0] == 'f':
            idx = [int(x.split('/')[0]) - 1 for x in w[1:]]
            for k in range(1, len(idx) - 1):
                tris.append([idx[0], idx[k], idx[k + 1]])
                mats.append(mat)
    v = np.array(v)
    t = np.array(tris)
    hand = np.array([m == 'skin' for m in mats])
    fig, axs = plt.subplots(1, 4, figsize=(22, 6))
    c = np.zeros(3)
    span = float(sys.argv[3]) if len(sys.argv) > 3 else 0.6

    def rot(p, ay, ax):
        ca, sa = np.cos(ay), np.sin(ay)
        cb, sb = np.cos(ax), np.sin(ax)
        r1 = np.array([[ca, 0, sa], [0, 1, 0], [-sa, 0, ca]])
        r2 = np.array([[1, 0, 0], [0, cb, -sb], [0, sb, cb]])
        return (p - c) @ r1.T @ r2.T

    for ax, (ay, ax2, title) in zip(axs, [(0, 0, 'from behind (x right, y up)'), (np.pi / 2, 0, 'from the side (z, y)'), (0, np.pi / 2, 'from above'), (0.6, 0.35, 'angle')]):
        q = rot(v, ay, ax2)[t]
        order = np.argsort(q[:, :, 2].mean(axis=1))[::-1]
        polys = [q[i][:, :2] for i in order]
        cols = [(0.95, 0.78, 0.65) if hand[i] else (0.45, 0.45, 0.5) for i in order]
        ax.add_collection(PolyCollection(polys, facecolors=cols, edgecolors='k', linewidths=0.1))
        ax.set_xlim(-span, span)
        ax.set_ylim(-span, span)
        ax.set_aspect('equal')
        ax.set_title(title)
        ax.grid(True, alpha=0.3)
        ax.set_xticks(np.arange(-0.5, 0.55, 0.1))
        ax.set_yticks(np.arange(-0.5, 0.55, 0.1))
    plt.tight_layout()
    plt.savefig(sys.argv[2], dpi=55)


main()
