"""Sounds, music and textures: put your own files in place of the game's."""
import os
import re
import shutil
import subprocess
import sys

from . import ui


def ffmpeg():
    exe = shutil.which('ffmpeg')
    if exe:
        return exe
    try:
        import imageio_ffmpeg
        return imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:
        return ''


def menu(ctx):
    while True:
        what = ui.choose('Sounds, music and pictures', [
            ('Replace a sound effect of the game', 'sound'),
            ('Replace a music track of the game', 'music'),
            ('Replace a texture (a picture on a model or a level)', 'texture'),
            ('Get all the game\'s textures as pictures to look at and edit', 'export'),
        ])
        if what is None:
            return
        if what == 'export':
            export_textures(ctx)
            continue
        mod = ctx.need_mod()
        if not mod:
            return
        {'sound': sound, 'music': music, 'texture': texture}[what](ctx, mod)


# the formats the game reads as they are (no conversion): Opus, Ogg (Vorbis or Opus), MP3, FLAC, M4A/AAC, WAV
GAME_AUDIO = ('.opus', '.ogg', '.oga', '.mp3', '.flac', '.m4a', '.aac', '.wav')
AUDIO_TYPES = [('Audio', '*.wav *.mp3 *.ogg *.flac *.opus *.m4a *.aac *.oga'), ('All files', '*.*')]


def put_audio(src, dst_stem, big_ok=True):
    """Put the audio file `src` where the game will read it, as `dst_stem` + the file's own extension (the game reads those formats itself).  A very large file (a WAV or FLAC of music)
    can be made much smaller as Opus with ffmpeg: asked.  Returns the file written, or '' when none."""
    ext = os.path.splitext(src)[1].lower()
    os.makedirs(os.path.dirname(dst_stem), exist_ok=True)
    if ext not in GAME_AUDIO:
        exe = ffmpeg()
        if not exe:
            ui.fail(f'The game cannot read {ext} files, and converting needs ffmpeg.  Use one of: mp3, ogg, flac, m4a, wav, opus.')
            return ''
        dst = dst_stem + '.opus'
        return dst if _opus(exe, src, dst) else ''
    size = os.path.getsize(src)
    if ext in ('.wav', '.flac') and size > 12_000_000 and ffmpeg() and ui.yes(f'This file is {size / 1e6:.0f} MB.  Make it a much smaller Opus file (almost the same sound)?', True):
        dst = dst_stem + '.opus'
        if _opus(ffmpeg(), src, dst):
            return dst
    dst = dst_stem + ext
    shutil.copyfile(src, dst)
    return dst


def _opus(exe, src, dst):
    cmd = [exe, '-y', '-loglevel', 'error', '-i', src, '-vn', '-ac', '2', '-ar', '48000', '-c:a', 'libopus', '-b:a', '128k', '-vbr', 'on', dst]
    try:
        subprocess.run(cmd, check=True)
    except Exception as e:
        ui.fail(f'The conversion failed: {e}')
        return False
    return True


def sound_names(ctx):
    import pakls
    _, es = pakls.entries(os.path.join(ctx.setup.game_dir, 'PAK', 'SFX.PAK'))
    return sorted(n for n, _, _ in es if n.lower().endswith('.vag'))


def sound(ctx, mod):
    ui.heading('Replace a sound effect')
    ui.say('The game has about 1100 sounds.  Type a word from the sound\'s file name to search (for example: gun, reload, scream, door).')
    names = sound_names(ctx)
    while True:
        word = ui.ask('Search for', 'gun').lower()
        found = [n for n in names if word in n.lower()]
        if not found:
            ui.say('  Nothing found.')
            continue
        shown = found[:30]
        pick = ui.choose(f'{len(found)} found' + (' (the first 30)' if len(found) > 30 else ''), [(n, n) for n in shown] + [('Search again', '__again')], back='Cancel')
        if pick is None:
            return
        if pick == '__again':
            continue
        break
    src = ui.pick_file('Choose your sound (wav, mp3, ogg, flac, m4a, opus; a short one: at most about 10 seconds are used)', AUDIO_TYPES)
    if not src:
        return
    base = os.path.splitext(pick)[0]
    done = put_audio(src, mod.path('SFX.PAK', base))
    if done:
        ui.say(f'\nSaved: your sound now replaces {pick} (the game turns it into its own sound format when it starts).')


def music(ctx, mod):
    ui.heading('Replace a music track')
    ui.say('The game has 27 tracks, 1001 to 1027; each level plays one of them.  On the Options page of an arcade match, the Level Music row lets you listen to each.')
    track = ui.ask_number('Track number', default=1001, kind=int, low=1001, high=1027)
    src = ui.pick_file('Choose your music (mp3, ogg, flac, m4a, wav, opus)', AUDIO_TYPES)
    if not src:
        return
    done = put_audio(src, mod.path('MUSIC', str(track)))
    if done:
        ui.say(f'\nSaved: your music replaces track {track}; it plays again from its start when it ends, so make it the part that repeats.')


def texture(ctx, mod):
    ui.heading('Replace a texture')
    ui.say('A texture is a picture on a character, weapon or level.  "Get all the game\'s textures" in this menu makes a folder of them; each file is named with its number.')
    ui.say('Edit one of them (keep the same size or make it larger) and choose it here.')
    src = ui.pick_file('Choose the edited picture (a .png named with the texture number, like 1234.png)', [('PNG picture', '*.png'), ('All files', '*.*')])
    if not src:
        return
    m = re.search(r'(\d+)(?=\.png$)', os.path.basename(src), re.I) or re.search(r'(\d+)', os.path.basename(src))
    number = ui.ask_number('The texture\'s number', default=int(m.group(1)) if m else None, kind=int, low=0, high=65535)
    mod.put_file(src, 'MISC.PAK', 'textures', f'{number:04d}.png')
    ui.say(f'\nSaved: texture {number}.  It may be shown at any size (a larger picture makes the texture sharper).')


def export_textures(ctx):
    import extract_textures
    out = os.path.join(ctx.setup.mods_dir, '_textures')
    if os.path.isdir(out) and os.listdir(out) and not ui.yes(f'{out} already has pictures.  Make them again?', False):
        ui.say(f'They are in {out}')
        return
    ui.say('Making the pictures (a minute or two)...')
    old = sys.argv
    sys.argv = ['extract_textures', os.path.join(ctx.setup.game_dir, 'PAK'), out]
    try:
        extract_textures.main()
    except SystemExit as e:
        if e.code not in (None, 0):
            ui.fail(str(e.code))
    finally:
        sys.argv = old
    ui.say(f'\nThe game\'s textures are in {out}  (they are the game\'s own: keep them to yourself).')
