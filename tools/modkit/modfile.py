"""A mod folder and the text files in it (`weapons.txt`, `characters.txt`, `modes.txt`): read in blocks, changed by block, written back."""
import hashlib
import os
import re
import shutil


def safe_name(text):
    return re.sub(r'[^A-Za-z0-9._-]+', '-', text.strip()).strip('-.') or 'My-Mod'


class Mod:
    def __init__(self, folder):
        self.folder = folder
        self.name = os.path.basename(folder)

    def create(self):
        os.makedirs(self.folder, exist_ok=True)
        ini = os.path.join(self.folder, 'mod.ini')
        if not os.path.exists(ini):
            open(ini, 'w').write('enabled = true\n')

    def path(self, *parts):
        return os.path.join(self.folder, *parts)

    def put_file(self, src, *dest):
        """Copy a file into the mod at the given place (folders are made)."""
        target = self.path(*dest)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        shutil.copyfile(src, target)
        return target

    def is_on(self):
        ini = self.path('mod.ini')
        if not os.path.exists(ini):
            return True
        text = open(ini, errors='replace').read()
        return not re.search(r'^\s*enabled\s*=\s*(false|no|0|off)\s*$', text, re.I | re.M)

    def set_on(self, on):
        ini = self.path('mod.ini')
        text = open(ini).read() if os.path.exists(ini) else ''
        line = f'enabled = {"true" if on else "false"}'
        if re.search(r'^\s*enabled\s*=', text, re.M):
            text = re.sub(r'^\s*enabled\s*=.*$', line, text, flags=re.M)
        else:
            text = line + '\n' + text
        open(ini, 'w').write(text)

    def block_file(self, name):
        return BlockFile(self.path(name))


class BlockFile:
    """A text file made of blocks that each start with a header line at the left margin (`weapon 30`, `mode deathmatch`); the lines after it belong to it.  Lines before the first
    header are kept as they are."""

    def __init__(self, path):
        self.path = path
        self.head = []
        self.blocks = []
        if os.path.exists(path):
            cur = None
            for line in open(path, encoding='utf8', errors='replace').read().splitlines():
                if re.match(r'^[A-Za-z]', line):
                    cur = (line.strip(), [])
                    self.blocks.append(cur)
                elif cur is None:
                    self.head.append(line)
                else:
                    cur[1].append(line)

    def find(self, header):
        for b in self.blocks:
            if b[0].lower() == header.lower():
                return b
        return None

    def set(self, header, body):
        b = self.find(header)
        if b:
            b[1][:] = body
        else:
            self.blocks.append((header, list(body)))

    def remove(self, header):
        self.blocks = [b for b in self.blocks if b[0].lower() != header.lower()]

    def numbers(self, keyword):
        """The numbers of the blocks `keyword <number>`."""
        out = []
        for h, _ in self.blocks:
            m = re.match(rf'{keyword}\s+(\d+)\b', h, re.I)
            if m:
                out.append(int(m.group(1)))
        return out

    def save(self):
        lines = list(self.head)
        for h, body in self.blocks:
            while body and not body[-1].strip():
                body.pop()
            lines.append(h)
            lines += body
            lines.append('')
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        open(self.path, 'w', encoding='utf8').write('\n'.join(lines).rstrip('\n') + '\n')


def used_numbers(mods_dir, filename, keyword, skip_mod=None):
    """The numbers of the added weapons or characters that the mods of `mods_dir` use (not `skip_mod`'s)."""
    out = set()
    for name in os.listdir(mods_dir):
        if name == skip_mod:
            continue
        p = os.path.join(mods_dir, name, filename)
        if os.path.exists(p):
            out.update(BlockFile(p).numbers(keyword))
    return out


def pick_number(mods_dir, mod, filename, keyword, low, high, key):
    """A number for something new in `mod`: made from its name, so that mods made apart seldom share one; the first free one from there on."""
    used = used_numbers(mods_dir, filename, keyword, skip_mod=mod.name) | set(mod.block_file(filename).numbers(keyword))
    span = high - low + 1
    start = int(hashlib.md5((mod.name + '/' + key).encode()).hexdigest(), 16) % span
    for k in range(span):
        n = low + (start + k) % span
        if n not in used:
            return n
    raise SystemExit(f'All the numbers from {low} to {high} are used by mods.')
