"""Game modes: the options a mode starts with, and ways to play it (variants, with rules that end the match)."""
from . import ui

MODES = [('Deathmatch', 'deathmatch'), ('BagTag', 'bagtag'), ('Capture the Bag', 'capturebag'), ('Escort', 'escort'), ('KnockOut', 'knockout'), ('Last Stand', 'laststand')]

OPTIONS = [
    ('kill_limit', 'Score limit (points to win; 0 = none)', 'int'),
    ('time_limit', 'Time limit (minutes; 0 = none)', 'int'),
    ('team_play', 'Team play', 'onoff'),
    ('friendly_fire', 'Friendly fire', 'onoff'),
    ('radar', 'Radar', 'onoff'),
    ('one_shot_kill', 'One shot kills', 'onoff'),
]


def menu(ctx):
    while True:
        what = ui.choose('Game modes', [
            ('Change what a mode starts with (options, name)', 'mode'),
            ('Add a way to play a mode (a variant, with a rule)', 'variant'),
            ('Take back my changes to a mode', 'undo'),
        ])
        if what is None:
            return
        mod = ctx.need_mod()
        if not mod:
            return
        if what == 'undo':
            undo(mod)
        else:
            edit(mod, what)


def pick_mode():
    return ui.choose('Which game mode?', MODES, default=1)


def ask_options(skip=()):
    lines = []
    ui.say('Press Enter to leave an option as it is.')
    for key, label, kind in OPTIONS:
        if key in skip:
            continue
        if kind == 'int':
            got = ui.ask(label, '-', lambda v: None if v == '-' or v.isdigit() else 'Please type a whole number, or - to leave it.')
            if got != '-':
                lines.append(f'  {key} {got}')
        else:
            got = ui.ask(label + ' (on/off)', '-', lambda v: None if v in ('-', 'on', 'off') else 'Type on or off, or - to leave it.')
            if got != '-':
                lines.append(f'  {key} {got}')
    return lines


def ask_rules():
    lines = []
    kind = ui.choose('How does a match end by itself? (the time and score limits still apply)', [
        ('The first to reach a score', 'win_score'),
        ('When one is ahead of the second by some points', 'win_lead'),
        ('BagTag: holding the bag for some seconds', 'win_score_seconds'),
        ('No extra rule', None),
    ], back=None, default=4)
    if kind:
        n = ui.ask_number({'win_score': 'Score to reach', 'win_lead': 'Lead needed (points)', 'win_score_seconds': 'Seconds of holding the bag'}[kind], kind=int, low=1, high=100000)
        lines.append(f'  rule {kind} {n}')
    return lines


def edit(mod, what):
    mode_key = pick_mode()
    if not mode_key:
        return
    mfile = mod.block_file('modes.txt')
    header = f'mode {mode_key}'
    block = mfile.find(header)
    body = list(block[1]) if block else []
    if what == 'mode':
        ui.heading('What the mode starts with')
        name = ui.ask('Name of the mode in the menus (Enter to keep the game\'s)', '-')
        cut = next((i for i, l in enumerate(body) if l.strip().startswith('variant')), len(body))
        keep = body[cut:]
        lines = ([f'  name "{name}"'] if name != '-' else []) + ask_options()
        mfile.set(header, lines + keep)
    else:
        ui.heading('A way to play ' + mode_key)
        vname = ui.ask('Name of the variant (up to 20 letters)', 'Quick', lambda v: None if len(v) <= 20 else 'At most 20 letters.')
        ui.say('What does it change?  (Enter to leave an option as the mode has it)')
        lines = [f'  variant "{vname}"'] + ask_options() + ask_rules()
        mfile.set(header, body + lines)
    mfile.save()
    ui.say(f'\nSaved to {mfile.path}')
    ui.say('In the game: Arcade, pick the mode; on the Options page the "Rules" row chooses the variant.')


def undo(mod):
    mfile = mod.block_file('modes.txt')
    items = [(h, h) for h, _ in mfile.blocks]
    if not items:
        ui.say('This mod changes no modes.')
        return
    h = ui.choose('Remove which?', items)
    if h:
        mfile.remove(h)
        mfile.save()
        ui.say('Removed.')
