"""Weapons: change the game's, or make a new one from a copy of one of them."""
from . import ui
from .modfile import pick_number

# what a person can change: (key in weapons.txt, what it is called, how it is asked, how to show it)
FIELDS = [
    ('clip', 'Rounds in a clip', 'int'),
    ('ammo_reserve', 'Rounds you start with in reserve', 'int'),
    ('max_ammo', 'Most rounds you can carry', 'int'),
    ('damage', 'Damage of a bullet (percent of a character\'s health)', 'percent'),
    ('spread', 'Spread (0 = perfectly accurate, 0.05 = sprays)', 'float'),
    ('recoil', 'Kick back when firing', 'float'),
    ('bullet_speed', 'Bullet speed', 'float'),
    ('fire_sound', 'Firing sound', 'sound'),
]


def _parse_body(body):
    out = {}
    for line in body:
        text = line.split('#')[0].strip()
        if text and ' ' in text:
            k, v = text.split(None, 1)
            out[k] = v.strip()
    return out


def _current(game_weapon, key):
    if key == 'max_ammo':
        return None
    v = game_weapon.get(key)
    return v


def _show(key, value, kind):
    if value is None:
        return 'as the game has it'
    if kind == 'percent':
        return f'{float(value) * 100:g} %'
    return f'{value:g}' if isinstance(value, float) else str(value)


def menu(ctx):
    game = ctx.game()
    if not game:
        return
    while True:
        what = ui.choose('Weapons', [
            ('Change a weapon of the game', 'change'),
            ('Make a new weapon (a copy of one, with changes)', 'new'),
            ('See the game\'s weapons', 'list'),
            ('Take back my changes to a weapon', 'undo'),
        ])
        if what is None:
            return
        if what == 'list':
            show_list(game)
        elif what == 'change':
            edit(ctx, game, new=False)
        elif what == 'new':
            edit(ctx, game, new=True)
        elif what == 'undo':
            undo(ctx, game)


def _named(game):
    """The weapons a player can pick up (the table has some without a name that the game does not use)."""
    return [w for w in game.weapons() if not w['name'].startswith('weapon ')]


def show_list(game):
    ui.heading('The weapons of the game')
    ui.say(f'{"No.":>3}  {"Name":<22} {"Clip":>4} {"Reserve":>7} {"Damage":>7}')
    for w in _named(game):
        ui.say(f'{w["number"]:>3}  {w["name"]:<22} {w["clip"]:>4} {w["ammo_reserve"]:>7} {w["damage"] * 100:>6.0f}%')
    ui.pause()


def pick_weapon(game, prompt):
    weapons = game.weapons()
    for w in _named(game):
        ui.say(f'  {w["number"]:>2}  {w["name"]}')
    n = ui.ask_number(prompt, kind=int, low=0, high=len(weapons) - 1)
    return weapons[n]


def edit(ctx, game, new):
    mod = ctx.need_mod()
    if not mod:
        return
    ui.heading('Make a new weapon' if new else 'Change a weapon')
    base = pick_weapon(game, 'Which weapon' + (' should it start as a copy of' if new else ' do you want to change') + '? (number)')
    wfile = mod.block_file('weapons.txt')
    if new:
        name = ui.ask('Name of the new weapon', f'{base["name"]} 2')
        number = pick_number(ctx.setup.mods_dir, mod, 'weapons.txt', 'weapon', 30, 63, name)
        header = f'weapon {number}'
        body = [f'  copy {base["number"]}', f'  name "{name}"']
        values = {}
    else:
        header = f'weapon {base["number"]}'
        old = wfile.find(header)
        body = []
        values = _parse_body(old[1]) if old else {}
    changed = dict(values)
    while True:
        options = []
        for key, label, kind in FIELDS:
            now = changed.get(key)
            if now is None:
                shown = _show(key, _current(base, key), kind)
            else:
                shown = (f'{float(now) * 100:g} %' if kind == 'percent' else now) + '  (changed)'
            options.append((f'{label}: {shown}', key))
        options.append(('Finished', '__done'))
        key = ui.choose('What do you want to change?', options, back='Cancel (keep nothing)', default=len(options))
        if key is None:
            ui.say('Nothing was saved.')
            return
        if key == '__done':
            break
        kind = next(k for kk, _, k in FIELDS if kk == key)
        changed[key] = ask_value(game, key, kind, base)
    lines = [f'  {k} {v}' for k, v in changed.items() if k not in ('copy', 'name')]
    wfile.set(header, body + lines if new else lines)
    if not lines and not new:
        wfile.remove(header)
    wfile.save()
    ui.say(f'\nSaved to {wfile.path}')
    if new:
        ui.say('The new weapon is in the weapon sets and in the weapon cycle; it needs the game build with the extended tables.')


def ask_value(game, key, kind, base):
    if kind == 'percent':
        return f'{ui.ask_number("Percent of health a bullet takes (the pistol is 30)", default=round(base["damage"] * 100), low=0, high=100000) / 100:g}'
    if kind == 'int':
        cur = base.get(key)
        return str(ui.ask_number('New value', default=cur if cur is not None else 100, kind=int, low=-1, high=99999))
    if kind == 'float':
        return f'{ui.ask_number("New value", default=base.get(key, 0.01)):g}'
    if kind == 'sound':
        ui.say('The firing sound can be the one of another weapon.')
        other = pick_weapon(game, 'Use the firing sound of which weapon? (number)')
        return str(other['fire_sound'])
    return ui.ask('New value')


def undo(ctx, game):
    mod = ctx.need_mod()
    if not mod:
        return
    wfile = mod.block_file('weapons.txt')
    items = [(h, h) for h, _ in wfile.blocks]
    if not items:
        ui.say('This mod changes no weapons.')
        return
    h = ui.choose('Remove which?', items)
    if h:
        wfile.remove(h)
        wfile.save()
        ui.say('Removed.')
