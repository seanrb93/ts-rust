"""Turn the game's music (MUSIC/*.MSC, raw ADPCM, about 13 MB for a track) into Opus files (about 1 MB), which the game uses in place of the .MSC next to them.

    python tools/convert_music.py <game folder> [--bitrate 128] [--delete-originals]

The game folder is the one with ts-rust.exe and MUSIC/.  Each MUSIC/NNNN.MSC is decoded by `ts-rust.exe --msc-to-wav` and encoded with ffmpeg
(libopus; ffmpeg has to be on the PATH, or imageio_ffmpeg installed) as MUSIC/NNNN.opus, stereo, 44.1 kHz, at the bitrate (kbit/s, default 128) in variable
bit rate.  The port plays an .opus when there is one next to the .MSC; with `--delete-originals` the .MSC is removed once its Opus file is
made (keep a copy of the game's files if you want to go back).
"""
import os
import shutil
import subprocess
import sys
import tempfile


def ffmpeg():
    exe = shutil.which('ffmpeg')
    if exe:
        return exe
    try:
        import imageio_ffmpeg
        return imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:  # noqa: BLE001
        sys.exit('ffmpeg was not found (put it on the PATH, or pip install imageio-ffmpeg)')


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    bitrate = 128
    if '--bitrate' in sys.argv:
        bitrate = int(sys.argv[sys.argv.index('--bitrate') + 1])
        args = [a for a in args if a != str(bitrate)]
    if len(args) != 1:
        print(__doc__)
        sys.exit(1)
    game = args[0]
    exe = os.path.join(game, 'ts-rust.exe')
    music = os.path.join(game, 'MUSIC')
    if not (os.path.isfile(exe) and os.path.isdir(music)):
        sys.exit('the folder needs ts-rust.exe and MUSIC/')
    ff = ffmpeg()
    before = after = 0
    for name in sorted(os.listdir(music)):
        if not name.lower().endswith('.msc'):
            continue
        src = os.path.join(music, name)
        dst = src[:-4] + '.opus'
        with tempfile.TemporaryDirectory() as tmp:
            wav = os.path.join(tmp, 'a.wav')
            subprocess.run([exe, '--msc-to-wav', src, wav], check=True)
            subprocess.run([ff, '-y', '-loglevel', 'error', '-i', wav, '-c:a', 'libopus', '-b:a', f'{bitrate}k', '-vbr', 'on', dst], check=True)
        before += os.path.getsize(src)
        after += os.path.getsize(dst)
        print(f'{name}: {os.path.getsize(src) / 1e6:.1f} MB -> {os.path.getsize(dst) / 1e6:.2f} MB')
        if '--delete-originals' in sys.argv:
            os.remove(src)
    print(f'{before / 1e6:.0f} MB -> {after / 1e6:.0f} MB')


if __name__ == '__main__':
    main()
