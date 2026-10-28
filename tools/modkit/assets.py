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


def to_audio(src, dst, mono, rate=None):
    """Make `dst` (.wav or .opus) from any audio file with ffmpeg; a .wav or .opus that is already right is copied."""
    ext = os.path.splitext(dst)[1].lower()
    if os.path.splitext(src)[1].lower() == ext:
        shutil.copyfile(src, dst)
        return True
    exe = ffmpeg()
    if not exe:
        ui.fail('This file needs ffmpeg to convert it. Use a .wav file (sounds) or an .opus file (music), or install ffmpeg.')
        return False
    cmd = [exe, '-y', '-loglevel', 'error', '-i', src, '-vn']
    if mono:
        cmd += ['-ac', '1']
    else:
        cmd += ['-ac', '2', '-ar', '48000', '-c:a', 'libopus', '-b:a', '112k', '-vbr', 'on']
    if rate:
        cmd += ['-ar', str(rate)]
    cmd += [dst]
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
    src = ui.pick_file('Choose your sound (wav, mp3, ogg, flac, opus; short, mono works best)', [('Audio', '*.wav *.mp3 *.ogg *.flac *.opus *.m4a'), ('All files', '*.*')])
    if not src:
        return
    base = os.path.splitext(pick)[0]
    ext = '.opus' if src.lower().endswith('.opus') else '.wav'
    dst = mod.path('SFX.PAK', base + ext)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if to_audio(src, dst, mono=True, rate=22050 if ext == '.wav' else None):
        ui.say(f'\nSaved: your sound now replaces {pick} (at most about 10 seconds are used).')


def music(ctx, mod):
    ui.heading('Replace a music track')
    ui.say('The game has 27 tracks, 1001 to 1027; each level plays one of them.  On the Options page of an arcade match, the Level Music row lets you listen to each.')
    track = ui.ask_number('Track number', default=1001, kind=int, low=1001, high=1027)
    src = ui.pick_file('Choose your music (mp3, ogg, wav, flac, opus)', [('Audio', '*.mp3 *.ogg *.wav *.flac *.opus *.m4a'), ('All files', '*.*')])
    if not src:
        return
    dst = mod.path('MUSIC', f'{track}.opus')
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if to_audio(src, dst, mono=False):
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
