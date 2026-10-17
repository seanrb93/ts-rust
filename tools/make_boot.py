"""One-time helper: copy the generated main_/cpuMain into ts-game/src/manual/boot.rs with the PS2 thread and
semaphore machinery removed.  After this the file is hand-maintained."""
import re, sys
src = open('ts-rust/crates/ts-game/src/dec/part_000.rs', encoding='utf8').read()

def grab(name):
    m = re.search(r'(pub fn %s\(.*?\n\}\n)' % name, src, re.S)
    return m.group(1)

main_ = grab('main_')
cpu = grab('cpuMain')

# --- main_: replace the thread creation / interrupt enable / sleep sequence by a direct call of the frame loop
a = main_.index('    a1_5 = 0x20000;')
b = main_.index('    SleepThread(&[]);') + len('    SleepThread(&[]);')
main_ = main_[:a] + '    // The PS2 original created a CPU thread and a GS thread here and put the main thread to sleep.\n' \
                    '    // The port runs the frame loop directly on this thread.\n    cpuMain();' + main_[b:]
main_ = main_.replace('pub fn main_(', '/// Game entry point (`main` in the original executable).\npub fn main_(')

# --- cpuMain: no semaphores; present each finished display list
cpu = cpu.replace('pub fn cpuMain() {\n', 'pub fn cpuMain() {\n    sw(g::gsdone, 1);\n', 1)
cpu = re.sub(r'\n\s*a0_3 = lw\(g::bufffree_sid\);\n\s*WaitSema\(a0_3\);', '', cpu)
cpu = re.sub(r'\n\s*a0_5 = lw\(g::bufffree_sid\);\n\s*WaitSema\(a0_5\);', '', cpu)
cpu = cpu.replace("sw(g::curcpudmabuf, 1 - lw(g::curcpudmabuf));", "present_frame();\n                    sw(g::curcpudmabuf, 1 - lw(g::curcpudmabuf));")
open('ts-rust/crates/ts-game/src/manual/boot_gen.rs', 'w', encoding='utf8').write(
    '//! Boot flow ported from the PS2 executable (threads and GS/VBL interrupt machinery removed).\n'
    '#![allow(unused_variables, unused_mut, unused_assignments, unreachable_code, non_snake_case, dead_code, '
    'unused_parens, unused_braces, unused_labels, arithmetic_overflow)]\nuse crate::*;\n\n' + main_ + '\n' + cpu)
print(len(main_), len(cpu))
