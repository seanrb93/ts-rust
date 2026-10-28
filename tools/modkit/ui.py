"""Questions and answers on the console: plain, short, with a default for every question."""
import os
import sys


def say(text=''):
    print(text)


def heading(text):
    print()
    print(text)
    print('-' * len(text))


def ask(prompt, default=None, check=None):
    """A line of text.  Enter takes the default.  `check(value)` returns an error text or None."""
    while True:
        shown = f' [{default}]' if default not in (None, '') else ''
        try:
            got = input(f'{prompt}{shown}: ').strip().strip('"')
        except EOFError:
            raise SystemExit('No more input.')
        if got == '' and default is not None:
            got = str(default)
        if got == '' and default is None:
            continue
        if check:
            err = check(got)
            if err:
                print('  ' + err)
                continue
        return got


def ask_number(prompt, default=None, low=None, high=None, kind=float):
    def check(v):
        try:
            x = kind(v)
        except ValueError:
            return 'Please type a number.'
        if low is not None and x < low:
            return f'It has to be at least {low}.'
        if high is not None and x > high:
            return f'It has to be at most {high}.'
        return None
    return kind(ask(prompt, default, check))


def yes(prompt, default=True):
    while True:
        got = input(f'{prompt} ({"Y/n" if default else "y/N"}): ').strip().lower()
        if got == '':
            return default
        if got[0] in 'yj':
            return True
        if got[0] == 'n':
            return False


def choose(title, options, back='Back', default=None):
    """Numbered choices; returns the value of the one picked, or None for Back.  `options` is a list of (label, value)."""
    print()
    if title:
        print(title)
    for i, (label, _) in enumerate(options, 1):
        print(f'  {i}. {label}')
    if back:
        print(f'  0. {back}')
    while True:
        got = input(f'Choose [{default if default is not None else (0 if back else 1)}]: ').strip()
        if got == '':
            got = str(default if default is not None else (0 if back else 1))
        if got.isdigit():
            n = int(got)
            if n == 0 and back:
                return None
            if 1 <= n <= len(options):
                return options[n - 1][1]
        print('  Please type one of the numbers.')


def pick_file(title, types=None, folder=False, must_exist=True):
    """A path: typed or pasted, or Enter to browse with a window.  Empty when none is chosen."""
    print(f'{title}')
    while True:
        got = input('  Paste the path here, or press Enter to browse (0 to cancel): ').strip().strip('"')
        if got == '0':
            return ''
        if got == '':
            got = _browse(title, types, folder)
            if not got:
                return ''
        got = os.path.expanduser(got)
        if not must_exist or (os.path.isdir(got) if folder else os.path.isfile(got)):
            return got
        print(f'  {got} is not there.')


def _browse(title, types, folder):
    try:
        import tkinter
        from tkinter import filedialog
        root = tkinter.Tk()
        root.withdraw()
        root.attributes('-topmost', True)
        got = filedialog.askdirectory(title=title) if folder else filedialog.askopenfilename(title=title, filetypes=types or [('All files', '*.*')])
        root.destroy()
        return got or ''
    except Exception:
        return ''


def pause():
    try:
        input('\nPress Enter to continue.')
    except EOFError:
        pass


def fail(text):
    print(f'\n! {text}')
