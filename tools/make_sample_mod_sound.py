"""Make the sounds of the sample mod (examples/sample_mod): a short "pew" for a weapon sound and a small looping tune for a music stream,
both written here (not taken from the game) and stored as Ogg Opus.  Needs ffmpeg with libopus (imageio_ffmpeg's will do)."""
import os
import subprocess
import wave

import numpy as np

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'examples', 'sample_mod')
try:
    import imageio_ffmpeg
    FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()
except Exception:  # noqa: BLE001
    FFMPEG = 'ffmpeg'


def to_opus(samples, rate, path, kbps):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + '.wav'
    pcm = np.clip(samples, -1, 1)
    with wave.open(tmp, 'wb') as w:
        w.setnchannels(2 if pcm.ndim == 2 else 1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes((pcm * 32767).astype('<i2').tobytes())
    subprocess.run([FFMPEG, '-y', '-loglevel', 'error', '-i', tmp, '-c:a', 'libopus', '-b:a', f'{kbps}k', path], check=True)
    os.remove(tmp)
    print(path, os.path.getsize(path), 'bytes')


# the "pew": a falling tone with a quick fade
r = 48000
t = np.arange(int(0.35 * r)) / r
freq = 1800 * np.exp(-t * 9) + 200
pew = np.sin(2 * np.pi * np.cumsum(freq) / r) * np.exp(-t * 7) * 0.8
pew += 0.15 * np.sign(np.sin(2 * np.pi * np.cumsum(freq * 0.5) / r)) * np.exp(-t * 12)
to_opus(pew, r, os.path.join(root, 'SFX.PAK', 'sfx', 'walther_and_bullet22_01e.opus'), 24)

# the tune: four bars of a square-wave arpeggio over a bass, 20 seconds, to loop
bpm, notes = 96, [0, 4, 7, 12, 7, 4]
beat = 60 / bpm / 2
song = np.zeros((int(20 * r), 2))
chords = [0, 5, 7, 3]
i = 0
for bar in range(4):
    root_note = 57 + chords[bar % 4]
    for step in range(16):
        n = root_note + notes[step % len(notes)] + (12 if step % 8 >= 4 else 0)
        f = 440 * 2 ** ((n - 69) / 12)
        start = int((bar * 16 + step) * beat * r)
        length = int(beat * r * 0.9)
        tt = np.arange(length) / r
        env = np.exp(-tt * 6)
        tone = np.sign(np.sin(2 * np.pi * f * tt)) * env * 0.12
        if start + length <= len(song):
            song[start:start + length, step % 2] += tone
            song[start:start + length, (step + 1) % 2] += tone * 0.4
    bass = 440 * 2 ** ((root_note - 12 - 69) / 12)
    s = int(bar * 16 * beat * r)
    bl = int(16 * beat * r)
    tt = np.arange(bl) / r
    song[s:s + bl] += (np.sin(2 * np.pi * bass * tt) * 0.2 * (0.6 + 0.4 * np.sin(2 * np.pi * tt * 2)))[:, None]
to_opus(song, r, os.path.join(root, 'MUSIC', '1015.opus'), 48)
