"""Makes the four groans of the Virus mode (SFX.PAK/sfx/*.wav), which fill four of the game's silent sounds.

    python make-groans.py

A groan is a voice (a pulse train with a falling pitch and a little vibrato) through three resonances of a vowel, with some breath noise and a rasp.
The files are the mod's own sounds, made here; nothing of the game is in them.
"""
import os
import wave

import numpy as np

RATE = 22050
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'SFX.PAK', 'sfx')

# (file name of the silent sound it replaces, length in seconds, pitch from and to in Hz, formants, rasp, seed)
GROANS = [
    ('chain_fence4_22_blank', 0.95, 120, 70, (500, 850, 2400), 0.55, 1),
    ('feathers1d_22_blank', 0.80, 170, 105, (650, 1100, 2600), 0.45, 2),
    ('bullet_hit_robot22_01_blank', 1.15, 95, 55, (420, 760, 2300), 0.70, 3),
    ('bullet_hit_face22_01_blank', 0.65, 150, 90, (560, 960, 2500), 0.50, 4),
]


def resonator(x, freq, bandwidth):
    """A two pole resonance at `freq` Hz."""
    r = np.exp(-np.pi * bandwidth / RATE)
    a1 = -2 * r * np.cos(2 * np.pi * freq / RATE)
    a2 = r * r
    gain = 1 - r
    y = np.zeros_like(x)
    p1 = p2 = 0.0
    for i, v in enumerate(x):
        out = gain * v - a1 * p1 - a2 * p2
        y[i] = out
        p2 = p1
        p1 = out
    return y


def groan(length, f_from, f_to, formants, rasp, seed):
    rng = np.random.default_rng(seed)
    n = int(length * RATE)
    t = np.arange(n) / RATE
    u = t / length
    # (the pitch falls, wavers a little and has a start that dips)
    pitch = f_from * (f_to / f_from) ** (u ** 0.8) * (1 + 0.025 * np.sin(2 * np.pi * 5.5 * t) + 0.01 * rng.standard_normal(n).cumsum() / np.sqrt(n) * 8)
    phase = np.cumsum(pitch) / RATE
    saw = 2 * (phase % 1.0) - 1
    # (a rough voice: the pulses are uneven)
    voice = saw * (1 + rasp * 0.5 * np.sign(np.sin(2 * np.pi * 37 * t + rng.random() * 6)))
    breath = rng.standard_normal(n) * 0.25
    source = voice + breath
    out = np.zeros(n)
    for f, bw, g in zip(formants, (90, 110, 160), (1.0, 0.7, 0.35)):
        out += g * resonator(source, f, bw)
    # (the mouth opens as the groan goes on)
    out = out + 0.4 * resonator(source, formants[0] * (1 + 0.25 * u.mean()), 120)
    env = np.minimum(1, t / 0.07) * np.exp(-1.6 * u) * np.minimum(1, (length - t) / 0.12)
    out *= env
    out = np.tanh(out * (2.0 + 2.0 * rasp))
    out /= np.abs(out).max()
    return (out * 0.85 * 32767).astype(np.int16)


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, length, f_from, f_to, formants, rasp, seed in GROANS:
        data = groan(length, f_from, f_to, formants, rasp, seed)
        path = os.path.join(OUT, name + '.wav')
        with wave.open(path, 'wb') as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(RATE)
            w.writeframes(data.tobytes())
        print(path, len(data) / RATE, 's')


if __name__ == '__main__':
    main()
