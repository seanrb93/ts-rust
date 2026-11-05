"""Turn the samples of `TS_SAMPLE=<file>` (ts-port's feature `sampler`) into a table of functions.

    python tools/sample-report.py <samples> <the program> [how many rows] [--thread NAME] [--lines FUNCTION]

Self is the part of a thread's samples in the function itself, total the part with the function anywhere on the stack.  With --lines, the lines of the source that the samples in the function fall on.
"""
import collections
import ctypes
import ctypes.wintypes as w
import os
import sys


class Symbol(ctypes.Structure):
    _fields_ = [('SizeOfStruct', w.ULONG), ('TypeIndex', w.ULONG), ('Reserved', ctypes.c_uint64 * 2), ('Index', w.ULONG), ('Size', w.ULONG), ('ModBase', ctypes.c_uint64), ('Flags', w.ULONG), ('Value', ctypes.c_uint64),
                ('Address', ctypes.c_uint64), ('Register', w.ULONG), ('Scope', w.ULONG), ('Tag', w.ULONG), ('NameLen', w.ULONG), ('MaxNameLen', w.ULONG), ('Name', ctypes.c_char * 512)]


class Line(ctypes.Structure):
    _fields_ = [('SizeOfStruct', w.DWORD), ('Key', ctypes.c_void_p), ('LineNumber', w.DWORD), ('FileName', ctypes.c_char_p), ('Address', ctypes.c_uint64)]


def symboliser(exe):
    d = ctypes.windll.dbghelp
    h = w.HANDLE(0x7777)
    d.SymSetOptions(0x2)  # undecorate
    d.SymInitialize.argtypes = [w.HANDLE, ctypes.c_char_p, w.BOOL]
    d.SymInitialize(h, os.path.dirname(os.path.abspath(exe)).encode(), False)
    d.SymLoadModuleEx.restype = ctypes.c_uint64
    d.SymLoadModuleEx.argtypes = [w.HANDLE, w.HANDLE, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint64, w.DWORD, ctypes.c_void_p, w.DWORD]
    base = d.SymLoadModuleEx(h, None, exe.encode(), None, 0x140000000, 0, None, 0)
    d.SymFromAddr.argtypes = [w.HANDLE, ctypes.c_uint64, ctypes.POINTER(ctypes.c_uint64), ctypes.POINTER(Symbol)]
    cache = {}

    def name(offset):
        if offset not in cache:
            s = Symbol(SizeOfStruct=88, MaxNameLen=500)
            disp = ctypes.c_uint64()
            cache[offset] = s.Name.decode(errors='replace') if d.SymFromAddr(h, base + offset, ctypes.byref(disp), ctypes.byref(s)) else '?'
        return cache[offset]

    def line(offset):
        l = Line(SizeOfStruct=ctypes.sizeof(Line))
        disp = w.DWORD()
        if d.SymGetLineFromAddr64(h, ctypes.c_uint64(base + offset), ctypes.byref(disp), ctypes.byref(l)):
            return os.path.basename(l.FileName.decode(errors='replace')) + ':' + str(l.LineNumber)
        return '?'
    name.line = line
    return name


def main():
    args = sys.argv[1:]
    only = None
    if '--thread' in args:
        i = args.index('--thread')
        only = args[i + 1]
        del args[i:i + 2]
    lines_of = None
    if '--lines' in args:
        i = args.index('--lines')
        lines_of = args[i + 1]
        del args[i:i + 2]
    lines = collections.Counter()
    path, exe = args[0], args[1]
    rows = int(args[2]) if len(args) > 2 else 40
    name = symboliser(exe)
    base = 0
    threads = {}
    per = collections.defaultdict(lambda: (collections.Counter(), collections.Counter(), [0]))
    for line in open(path):
        p = line.split()
        if not p:
            continue
        if p[0] == 'base':
            base = int(p[1], 16)
        elif p[0] == 'thread':
            threads[p[1]] = p[2] if len(p) > 2 else 'thread' + p[1]
        elif p[0] == 's':
            th = threads.get(p[1], p[1])
            self_, total, n = per[th]
            n[0] += 1
            names = []
            for x in p[2:]:
                off = int(x, 16) - base
                names.append(name(off) if 0 <= off < 1 << 31 else '(system)')
            self_[names[0]] += 1
            if lines_of and lines_of in names[0] and (not only or th == only):
                lines[name.line(int(p[2], 16) - base)] += 1
            for f in set(names):
                total[f] += 1
    if lines_of:
        all_ = sum(lines.values()) or 1
        for l, v in lines.most_common(rows):
            print(f'{100 * v / all_:6.2f}%  {l}')
        return
    for th, (self_, total, n) in sorted(per.items(), key=lambda kv: -kv[1][2][0]):
        if only and th != only:
            continue
        print(f'== {th}: {n[0]} samples, {100 * self_["(system)"] / n[0]:.0f}% in the system')
        for f, v in self_.most_common(rows):
            print(f'{100 * v / n[0]:6.2f}% self {100 * total[f] / n[0]:6.2f}% total  {f[:90]}')


main()
